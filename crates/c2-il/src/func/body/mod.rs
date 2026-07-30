pub(crate) mod chain;
pub(crate) mod expr;
pub(crate) mod shapes;

use self::chain::{
    additive_chain_canonical, canonicalize_chain, has_repeated_leaf, leaves_ascending,
    straight_line_is_out_of_class,
};
use self::expr::{eat_return_plumbing, intrinsic_name, parse_expr, parse_formals};
use self::shapes::{
    parse_call_shape, try_parse_assign_body_detail, try_parse_compare, try_parse_float_leaf,
    try_parse_indirect_load_leaf,
};
use super::readers::{eat_byte, eat_opt_stmt_marker, find_subslice, read_token_var};
use super::{CompareLeaf, IlOp};

/// One recognized whole-body shape of a single `.ex` function segment. Every
/// accepted body is *exactly* one of these — the parser (see [`parse_segment`])
/// is a positive whole-stream parse that reaches the segment's end, so anything
/// it does not model produces `None` and the caller reports `NotImplemented`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum BodyShape {
    /// Straight-line all-`int` arithmetic leaf (`return a+b+c`, `return a+5`,
    /// `return 42`, …): a postfix LOAD/LIT/ADD/SUB/MUL stream returning `int`.
    StraightLine { params: Vec<u32>, ops: Vec<IlOp> },
    /// Bare terminal void tail call (`void f(){ g(); }`): exactly one CALL whose
    /// void result is discarded, with **nothing** after its `4C 4B` void
    /// call-end but the return plumbing → codegen emits a single `b <callee>`.
    VoidTailCall { callee_tok: u32 },
    /// Integer tail call `return g(<arg>)` (and the identity-fold `g(a) + 0`):
    /// exactly one int-returning CALL whose single argument is a modeled
    /// sub-expression (`arg_ops`), a `55 <int> 4C` call-end, and a **net-identity
    /// post-op** (absent, or `+ 0` folded away). Codegen computes the argument
    /// into r3 (the leaf selector), then `b <callee>` — a 5-section leaf, the
    /// integer analog of [`BodyShape::VoidTailCall`]. `arg_ops` is a bare
    /// `[Load]` for the passthrough `g(a)`, or e.g. `[Load, Lit, Add]` for the
    /// arg-setup `g(a + 1)` (→ `addi r3,r3,1 ; b g`). `params` are the formals
    /// (token→register mapping the arg-setup needs).
    IntTailCall { params: Vec<u32>, arg_ops: Vec<IlOp>, callee_tok: u32 },
    /// `return g(a1, …, an)` with `n >= 2`, every argument a bare parameter.
    /// `arg_sources[i]` indexes `params` for the value argument slot `i` wants;
    /// codegen turns that into a register permutation plus the tail branch.
    MultiArgTailCall { params: Vec<u32>, arg_sources: Vec<usize>, callee_tok: u32 },
    /// Framed non-leaf `return g(a) + k` (k ≠ 0): exactly one int-returning CALL
    /// whose argument region is exactly the single passthrough LOAD, a `55 <int>`
    /// call-end, then exactly one literal `+ k` (ADD, commutative), returned. A
    /// zero `k` is NOT framed — it folds to [`BodyShape::IntTailCall`].
    FramedCall { add_k: i32, callee_tok: u32 },
    /// W6 comparison leaf: `return <formal> <rel> <literal>;` materialized to a
    /// boolean branchlessly and converted back to `int`/`unsigned`.
    Compare(CompareLeaf),
    /// W13a floating-point leaf: a straight-line chain over float (or double)
    /// *parameters* — no constants, no conversions, no contraction.
    FloatLeaf { params: Vec<u32>, ops: Vec<IlOp>, double: bool },
    /// An **empty function body** (`void f() {}`): the body opens directly on the
    /// `3A` assign of the return plumbing with no expression before it. Emits a
    /// bare `blr`.
    EmptyBody,
    /// An **indirect-load leaf**: the whole body is one load through a pointer
    /// (`return *p;`, `return s->m;`, `return p[k];`, `return mMember;`), which c2
    /// lowers to a single `lwz rD, off(rBase)`. `ops` is always exactly
    /// `[Load(base), LoadInd { off }]` and `params` includes a member function's
    /// `this` at index 0. See [`try_parse_indirect_load_leaf`].
    IndirectLoad { params: Vec<u32>, ops: Vec<IlOp> },
}

/// **Why** a function segment fell outside the modeled class (P2b census).
///
/// The positive parser fails closed at the *first* byte it cannot account for.
/// Recording that point — the grammar production it was in, the offending byte,
/// and the offset — turns an opaque `None` into a rankable census key: over a
/// real workload the histogram of [`Block::feature`] *is* the widening order
/// (docs/ROADMAP.md §G5/P2b). Purely diagnostic: acceptance is unchanged, and
/// [`parse_segment`] still returns a bare `Option`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Block {
    /// The grammar production the parse was inside (`"expr"`, `"call-end"`, …).
    pub ctx: &'static str,
    /// The byte that could not be consumed (`None` at end-of-segment).
    pub byte: Option<u8>,
    /// Byte offset within the function segment.
    pub off: usize,
    /// Context payload for the operand-*type* blocks, where the single blocking
    /// byte is uninformative: an operand's 3-byte inline type differs from the
    /// modeled `86 41 74` (int), but the first byte `86` is shared by every
    /// type, so reporting it buckets `unsigned`, `float`, `pointer`, … together.
    /// Packed big-endian in the low 24 bits; 0 when unused.
    pub aux: u32,
}

impl Block {
    /// A short, stable census key naming the blocking *feature*.
    ///
    /// Operand-stream opcodes get a named bucket when the byte's meaning is
    /// verified against a live capture, and a `expr-op-0xNN` bucket otherwise —
    /// the point of the census is to *measure* the unknown vocabulary, so an
    /// honest hex bucket is a result, not a placeholder. Structural blocks
    /// (call-end, return plumbing, formals) name their production instead.
    pub fn feature(self) -> String {
        // Intrinsic-call blocks report their **selector**, which [`Block::aux`]
        // carries as the decoded id (see [`intrinsic_selector`]). This is the
        // whole point of decoding the production: `0x40` alone is one opaque 9 %
        // bucket, while the selector splits it into a handful of named
        // constructs with wildly different lowerings — `fabs` is one
        // instruction, `memcmp` is a 15-instruction loop, and the dominant
        // 2113–2119 class-layout family is a pointer adjustment whose emission
        // depends on its *literal* arguments, not on the id.
        if self.ctx == "expr-intrinsic" || self.ctx == "call-intrinsic" {
            return format!("{}-{}", self.ctx, intrinsic_name(self.aux as i32));
        }
        // Operand-type blocks report the whole 3-byte type: that triple *is* the
        // feature (int vs unsigned vs float vs pointer), and it is what the next
        // widening step must teach `parse_expr` to accept.
        if self.aux != 0 {
            return format!(
                "{}-{:02X}{:02X}{:02X}",
                self.ctx,
                (self.aux >> 16) & 0xFF,
                (self.aux >> 8) & 0xFF,
                self.aux & 0xFF
            );
        }
        let b = match self.byte {
            Some(b) => b,
            None => return format!("{}:eof", self.ctx),
        };
        if self.ctx == "expr" {
            // Operand-stream opcodes VERIFIED against live-toolchain captures
            // (docs/CODEGEN_W6_COMPARE.md pins the relational and logical ones
            // by compiling a probe per relation and reading the emitted byte).
            //
            // Only add a name here once a capture has established it. An earlier
            // revision of this table guessed the relational opcodes from their
            // numeric order and got `!=`, `<=` and `>=` wrong while missing `==`
            // entirely — which silently mislabelled census buckets, the one
            // thing this instrument exists to avoid. A hex bucket is a result;
            // a wrong name is a lie that survives into the roadmap.
            //
            // Signedness is NOT in the opcode: signed and unsigned probes emit
            // the same byte and differ only in the operand type (`86 41 74` int
            // vs `86 42 75` unsigned).
            let named = match b {
                0x1F => Some("cmp-eq"),   // ==
                0x20 => Some("cmp-ne"),   // !=
                0x21 => Some("cmp-le"),   // <=
                0x22 => Some("cmp-lt"),   // <
                0x23 => Some("cmp-ge"),   // >=
                0x24 => Some("cmp-gt"),   // >
                0x1A => Some("not"),      // !
                0x1B => Some("or-or"),    // ||
                0x1C => Some("and-and"),  // &&
                0x09 => Some("shl"),      // <<
                0x0A => Some("shr"),      // >>
                0x0B => Some("bit-and"),  // &
                0x0C => Some("bit-or"),   // |
                0x0D => Some("bit-xor"),  // ^
                0x2C => Some("convert"),  // `2C <TYPE> <varint>` — the real cast
                // `0x40` is a SECOND call token — the intrinsic call — not a
                // cast. It occupies the slot `BD` occupies:
                //   33 <int-TYPE> <selector>  40 <TYPE result>  (<expr> 55 <TYPE>)*  4C
                // An earlier revision of this table guessed "cast" from a single
                // witness where it followed a literal. It follows a bare `int`
                // constant at 6838 of 6839 aligned sites across three real TUs —
                // which is the selector, not a cast operand. Selectors seen:
                // 15 abs, 17 fabs, 159/160 _rotl/_rotr, 164 strcpy, 165 strcmp,
                // 167 strlen, 170 memcmp, 172 memcpy, 173 memset, 1973 sqrt,
                // and the dominant 2113-2119 class-layout adjustment family.
                0x40 => Some("intrinsic-call"),
                // The class-pair descriptor of that same family — NOT a call.
                0x66 => Some("class-descriptor"),
                0x43 => Some("ternary"),  // `43 42 ...` conditional select
                0x26 => Some("call-in-expr"),
                _ => None,
            };
            return match named {
                Some(n) => format!("expr-{n}"),
                None => format!("expr-op-0x{b:02X}"),
            };
        }
        format!("{}-0x{b:02X}", self.ctx)
    }
}

/// Build a [`Block`] at the current parse position.
pub(crate) fn blk(seg: &[u8], p: usize, ctx: &'static str) -> Block {
    Block { ctx, byte: seg.get(p).copied(), off: p, aux: 0 }
}

/// Build an operand-*type* [`Block`]: `p` points at the 3-byte inline type that
/// is not the modeled int (`86 41 74`), `report_at` at the operand it belongs
/// to. Packs the triple into [`Block::aux`] so the census buckets by type.
pub(crate) fn blk_type(seg: &[u8], p: usize, report_at: usize, ctx: &'static str) -> Block {
    let g = |i: usize| seg.get(p + i).copied().unwrap_or(0) as u32;
    Block {
        ctx,
        byte: seg.get(p).copied(),
        off: report_at,
        aux: (g(0) << 16) | (g(1) << 8) | g(2),
    }
}

/// **The positive whole-body parser (W4b2-v).** Parse a single `.ex` function
/// segment as *exactly one* of the recognized [`BodyShape`]s, tokenizing
/// the entire operand stream from the `4C 4F 11` ('LO') marker to the end of the
/// segment. Acceptance is by a complete positive match — every token is
/// consumed through a fixed-pattern `eat` or a typed read, and the parse must
/// reach the segment end — so a second CALL, any computation after a terminal
/// call, a non-trivial call-argument region, or any unmodeled byte fails the
/// whole function closed (`None` → the caller reports `NotImplemented`). This
/// replaces the earlier trio of neighborhood-scanning gates (`parse_body`,
/// `is_tail_call`, `parse_framed_call`) that each accepted on a *local* byte
/// pattern and so over-accepted trailing/second-call computation.
///
/// Grammar (verified against live-toolchain captures of every fixture + probe):
/// ```text
///   body   := 'LO'(4C 4F 11) 'SS'(53) stmt?  ( arith | vcall | icall )
///   stmt   := 4F 01 NN                                    (multi-fn only)
///   arith  := expr(→41)  <return int>                     LOAD:=B9 tok INT
///   vcall  := 26 tok  CALL  4C 4B  <return void>          LIT :=33 INT varint
///   icall  := 26 tok  CALL  expr(→55)  55 INT 4C  postop  <return int>
///   postop := ε | 33 INT k 02                             expr:=(LOAD|LIT|02|03|04)+
///   CALL   := BD <ret TYPE> <conv> <varint fn-type-id>    (8-13 bytes, decoded)
/// ```
/// The `CALL` line used to read `BD <3-byte ret type> 00 80 01 10 00 00 (fixed 10
/// bytes)`. That was never an anchor: the trailing value is a per-TU **function-type
/// id**, keyed on the signature and assigned in declaration order of distinct
/// function types, so `0x1001` is merely the first one a single-callee fixture TU
/// happens to create. Every field is self-delimiting and is decoded — see
/// [`parse_call_shape`].
/// `<return …>` is the shared plumbing consumed by [`eat_return_plumbing`]
/// (result-type for int, then assign/return/tail/segment-or-module end). An
/// `icall` is classified by its `postop`: **absent, or `+ 0`** → an integer
/// tail call [`BodyShape::IntTailCall`] (the argument `expr` computed into r3,
/// then `b <callee>`; `g(a)`, `g(a)+0`, `g(a+1)` all land here). A **non-zero
/// `+ k`** over a *bare passthrough* argument (`expr == [Load]`) → the framed
/// [`BodyShape::FramedCall`] (whose `k` fits a signed-16-bit `addi`). A non-zero
/// `+ k` over a *computed* argument (`g(a+1)+1`), or a `* k`/`- k`/wide `k`/a
/// second literal/a second call, all reject. The `callee` name is not in `.ex`;
/// the caller pairs it from `.gl`.
pub(crate) fn parse_segment(seg: &[u8], locals: &[u32]) -> Option<BodyShape> {
    parse_segment_detail(seg, locals).ok()
}

/// [`parse_segment`] with the fail-closed *reason* preserved (P2b census).
/// Acceptance is identical — `parse_segment` is `.ok()` of this — so the census
/// can never disagree with the gate about what is in class.
pub(crate) fn parse_segment_detail(seg: &[u8], locals: &[u32]) -> Result<BodyShape, Block> {
    let lo = find_subslice(seg, &[0x4C, 0x4F, 0x11]).ok_or(Block {
        ctx: "lo-marker",
        byte: None,
        off: 0,
        aux: 0,
    })?;
    let mut p = lo + 3;
    // 'SS' statement-start, then an optional statement/label marker.
    if !eat_byte(seg, &mut p, 0x53) {
        return Err(blk(seg, p, "stmt-start"));
    }
    eat_opt_stmt_marker(seg, &mut p);

    match *seg.get(p).ok_or(blk(seg, p, "body"))? {
        // An EMPTY body opens directly on the return plumbing's `3A` assign —
        // there is no expression at all. `eat_return_plumbing` still has to
        // reach the segment end, so any trailing statement or unexpected operand
        // fails the function closed exactly as it does for every other shape.
        0x3A => {
            eat_return_plumbing(seg, &mut p, false)?;
            Ok(BodyShape::EmptyBody)
        }
        // `26 <tok>` opens BOTH a call (the callee push) and an assignment
        // statement (the destination push), and the two are told apart by exactly
        // one byte: whether a `BD` CALL opcode follows the pushed token.
        //
        // Dispatching on that byte rather than trying the assignment parse and
        // falling back matters for the *measurement*, not for what is accepted.
        // Falling back meant every assignment-body refusal was re-reported as
        // whatever byte `parse_call_shape` then tripped over — nearly always the
        // RHS's `B9` — so `call-token-0xB9` was a conflated bucket holding pointer
        // operands, casts, `if` statements and more, all filed under a name that
        // described none of them. It has been the #1 entry at ~18% of blocked
        // functions and was directing the widening order at least twice this week.
        // Now each side reports its own reason.
        0x26 => {
            let mut probe = p + 1;
            let is_call = match read_token_var(seg, probe) {
                Some((_, w)) => {
                    probe += w;
                    seg.get(probe) == Some(&0xBD)
                }
                None => false,
            };
            if is_call {
                parse_call_shape(seg, &mut p, lo, None)
            } else {
                try_parse_assign_body_detail(seg, p, lo, locals)
            }
        }
        // Straight-line arithmetic opens with a LOAD or a bare literal — and so
        // does a W6 comparison leaf, which is tried first because its whole-body
        // shape is strictly more specific (a LOAD/LIT pair consumed by a
        // relational opcode). `try_parse_compare` is non-committal: it works on
        // a copy of the cursor and returns None without side effects, so a
        // non-comparison body falls through to the arithmetic parse unchanged.
        0xB9 | 0x33 => {
            if let Some(shape) = try_parse_compare(seg, p, lo) {
                return Ok(shape);
            }
            if let Some(shape) = try_parse_float_leaf(seg, p, lo) {
                return Ok(shape);
            }
            if let Some(shape) = try_parse_indirect_load_leaf(seg, p, lo) {
                return Ok(shape);
            }
            let ops = parse_expr(seg, &mut p, 0x41)?;
            eat_return_plumbing(seg, &mut p, true)?;
            let params = parse_formals(seg, lo)?;
            // A parameter used twice licenses c2's algebraic rewriter.
            if has_repeated_leaf(&ops) {
                return Err(Block { ctx: "expr-repeated-leaf", byte: None, off: p, aux: 0 });
            }
            // Gates that used to live in codegen; see `straight_line_is_out_of_class`.
            if straight_line_is_out_of_class(&ops, &params) {
                return Err(Block { ctx: "expr-out-of-class", byte: None, off: p, aux: 0 });
            }
            let ops = match canonicalize_chain(&ops, &params) {
                Some(c) => c,
                None => {
                    if !leaves_ascending(&ops, &params) {
                        return Err(Block {
                            ctx: "expr-noncanonical-order",
                            byte: None,
                            off: p,
                            aux: 0,
                        });
                    }
                    if !additive_chain_canonical(&ops) {
                        return Err(Block {
                            ctx: "expr-noncanonical-additive",
                            byte: None,
                            off: p,
                            aux: 0,
                        });
                    }
                    ops
                }
            };
            Ok(BodyShape::StraightLine { params, ops })
        }
        _ => Err(blk(seg, p, "body")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::func::test_fixtures::*;

    // ---- positive whole-body parser (W4b2-v) --------------------------------
    //
    // Every fixture below is a REAL `.ex` function segment captured from the
    // live 16.00.11886.00 toolchain (`/Bd /d2nop /Ox /GS- /c`), transcribed from
    // the `4F 1F` split point. Straight-line segments include the `46` formals
    // marker; call segments start at the `LO` marker (call shapes carry no
    // formal list). Each accepted segment is a *last* function, so it ends at
    // the module end `… 4F 02 20 00 4F 01 NN 4D` — the parser must reach it.

    #[test]
    fn parse_segment_accepts_straight_line_add3() {
        // `int add3(int a,int b,int c){ return a+b+c; }` (mvp_add3, single fn).
        let seg: &[u8] = &[
            0x46, 0x2D, 0xE5, 0x09, 0x2D, 0xE4, 0x09, 0x2D, 0xE3, 0x09, // formals c,b,a
            0x4C, 0x4F, 0x11, 0x53, // LO SS
            0xB9, 0xE3, 0x09, 0x86, 0x41, 0x74, // LOAD a
            0xB9, 0xE4, 0x09, 0x86, 0x41, 0x74, // LOAD b
            0x02, // ADD
            0xB9, 0xE5, 0x09, 0x86, 0x41, 0x74, // LOAD c
            0x02, // ADD
            0x41, 0x86, 0x41, 0x74, // result-type int
            0x3A, 0xE7, 0x09, // ASSIGN
            0x54, 0x02, 0x29, 0xE7, 0x09, // RETURN
            0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00, // separator + GT terminate
            0x4F, 0x02, 0x20, 0x00, 0x4F, 0x01, 0x02, 0x4D, // module end
        ];
        assert_eq!(
            parse_segment(seg, NO_LOCALS),
            Some(BodyShape::StraightLine {
                params: vec![0xE309, 0xE409, 0xE509], // a, b, c
                ops: vec![
                    IlOp::Load(0xE309),
                    IlOp::Load(0xE409),
                    IlOp::Add,
                    IlOp::Load(0xE509),
                    IlOp::Add,
                ],
            })
        );
    }

    #[test]
    fn parse_segment_accepts_bare_literal_return_and_wide() {
        // `int konst(){ return 42; }` — empty formal list (`46` then `LO`), a
        // bare literal, and the multi-function statement markers `4F 01 NN`.
        let konst: &[u8] = &[
            0x46, // formals marker, empty list
            0x4C, 0x4F, 0x11, 0x53, 0x4F, 0x01, 0x0E, // LO SS + stmt marker
            0x33, 0x86, 0x41, 0x74, 0x2A, // LIT 42
            0x41, 0x86, 0x41, 0x74, // result-type
            0x3A, 0xEA, 0x09, 0x4F, 0x01, 0x0F, // ASSIGN + stmt marker
            0x54, 0x02, 0x29, 0xEA, 0x09, // RETURN
            0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00, 0x4F, 0x02, 0x20, 0x00, 0x4F, 0x01, 0x10,
            0x4D,
        ];
        assert_eq!(
            parse_segment(konst, NO_LOCALS),
            Some(BodyShape::StraightLine {
                params: vec![],
                ops: vec![IlOp::Lit(42)],
            })
        );
        // `int kw(){ return 70000; }` — the wide (`0x80` + 4-byte LE) varint.
        let kw: &[u8] = &[
            0x46, 0x4C, 0x4F, 0x11, 0x53, 0x4F, 0x01, 0x0D, // formals/LO/stmt
            0x33, 0x86, 0x41, 0x74, 0x80, 0x70, 0x11, 0x01, 0x00, // LIT 70000 (wide)
            0x41, 0x86, 0x41, 0x74, 0x3A, 0xEA, 0x09, 0x4F, 0x01, 0x0E, 0x54, 0x02, 0x29, 0xEA,
            0x09, 0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00, 0x4F, 0x02, 0x20, 0x00, 0x4F, 0x01,
            0x0F, 0x4D,
        ];
        assert_eq!(
            parse_segment(kw, NO_LOCALS),
            Some(BodyShape::StraightLine {
                params: vec![],
                ops: vec![IlOp::Lit(70000)],
            })
        );
    }

    #[test]
    fn parse_segment_accepts_nonlast_function_reaching_segment_end() {
        // `int add2(int a,int b){ return a+b; }` as the FIRST function of a
        // multi-fn TU: the segment is split before the next `4F 1F`, so it ends
        // right after `47 54 01 54 00` (no module end). The parse must accept by
        // reaching that segment end, not by finding a module marker.
        let seg: &[u8] = &[
            0x46, 0x2D, 0xE4, 0x09, 0x2D, 0xE3, 0x09, // formals b,a
            0x4C, 0x4F, 0x11, 0x53, 0x4F, 0x01, 0x07, // LO SS + stmt marker
            0xB9, 0xE3, 0x09, 0x86, 0x41, 0x74, // LOAD a
            0xB9, 0xE4, 0x09, 0x86, 0x41, 0x74, // LOAD b
            0x02, // ADD
            0x41, 0x86, 0x41, 0x74, // result-type
            0x3A, 0xE6, 0x09, 0x4F, 0x01, 0x08, // ASSIGN + stmt marker
            0x54, 0x02, 0x29, 0xE6, 0x09, // RETURN
            0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00, // GT terminate = segment end
        ];
        assert_eq!(
            parse_segment(seg, NO_LOCALS),
            Some(BodyShape::StraightLine {
                params: vec![0xE309, 0xE409],
                ops: vec![IlOp::Load(0xE309), IlOp::Load(0xE409), IlOp::Add],
            })
        );
    }

    #[test]
    fn parse_segment_accepts_bare_void_tail_call() {
        // `void f(){ g(); }` (mvp_call): exactly one void call, `4C 4B`, then
        // only the return plumbing → a bare `b g` tail call.
        assert_eq!(
            parse_segment(MVP_CALL, NO_LOCALS),
            Some(BodyShape::VoidTailCall { callee_tok: 0xE309 })
        );
    }

    #[test]
    fn parse_segment_accepts_framed_call() {
        // `int f(int a){ return g(a) + 1; }` (mvp_framed): int call, single
        // passthrough arg, `55` call-end, exactly one `+1` post-op.
        assert_eq!(
            parse_segment(MVP_FRAMED, NO_LOCALS),
            Some(BodyShape::FramedCall { add_k: 1, callee_tok: 0xE409 })
        );
    }

    #[test]
    fn parse_segment_accepts_int_tail_call_family() {
        // The three int tail-call shapes (formals `46 2d e509` = param a → r3):
        //   passthrough `g(a)` and identity-fold `g(a)+0` → arg `[Load a]`;
        //   arg-setup `g(a+1)` → arg `[Load a, Lit 1, Add]`. All are
        //   `IntTailCall` (a net-identity post-op is a tail call, not framed).
        assert_eq!(
            parse_segment(INT_TAILRET, NO_LOCALS),
            Some(BodyShape::IntTailCall {
                params: vec![0xE509],
                arg_ops: vec![IlOp::Load(0xE509)],
                callee_tok: 0xE409,
            }),
            "passthrough g(a)"
        );
        assert_eq!(
            parse_segment(INT_PLUS0, NO_LOCALS),
            Some(BodyShape::IntTailCall {
                params: vec![0xE509],
                arg_ops: vec![IlOp::Load(0xE509)],
                callee_tok: 0xE409,
            }),
            "identity-fold g(a)+0 routes to a tail call, not FramedCall{{add_k:0}}"
        );
        assert_eq!(
            parse_segment(INT_ARGTAIL, NO_LOCALS),
            Some(BodyShape::IntTailCall {
                params: vec![0xE509],
                arg_ops: vec![IlOp::Load(0xE509), IlOp::Lit(1), IlOp::Add],
                callee_tok: 0xE409,
            }),
            "arg-setup g(a+1)"
        );
    }

    #[test]
    fn parse_segment_routes_framed_nonzero_but_folds_zero_k() {
        // Routing contrast at the post-op: a NON-zero `+k` over a bare
        // passthrough arg is FramedCall (6-section frame); a ZERO `+k` folds to
        // an IntTailCall (5-section leaf). Same shape but for the immediate.
        assert_eq!(
            parse_segment(MVP_FRAMED, NO_LOCALS),
            Some(BodyShape::FramedCall { add_k: 1, callee_tok: 0xE409 }),
            "g(a)+1 is framed"
        );
        assert!(
            matches!(parse_segment(INT_PLUS0, NO_LOCALS), Some(BodyShape::IntTailCall { .. })),
            "g(a)+0 must NOT be FramedCall{{add_k:0}}"
        );
    }

    #[test]
    fn parse_segment_rejects_all_out_of_class_call_shapes() {
        // The W4b2-i/-v out-of-class probes — each a real captured segment the
        // positive parse must reject at the parser level (→ None →
        // NotImplemented), never mis-emit. Named by their `.cpp`. (The bare
        // arg-setup tail calls `g(a)`/`g(a)+0`/`g(a+1)` are now ACCEPTED —
        // see `parse_segment_accepts_int_tail_call_family`.)
        let cases: &[(&str, &[u8])] = &[
            ("g(a) - 1 (submod)", GA_SUBMOD),
            ("g(a) * 5 (mulmod)", GA_MULMOD),
            ("g(a) + 70000 (widemod)", GA_WIDEMOD),
            ("g(); g(); (two_calls)", TWO_CALLS),
            ("g(); return a+1; (call_then_stmt)", CALL_THEN_STMT),
            ("g(a + 1) + 1 (argframed_plusk)", ARGFRAMED_PLUSK),
            ("g(a) + g(a + 1) (two_framed_calls)", TWO_FRAMED_CALLS),
            ("g(a) + 1 + 2 (plus1plus2)", PLUS1PLUS2),
        ];
        for (label, seg) in cases {
            assert_eq!(parse_segment(seg, NO_LOCALS), None, "must reject: {label}");
        }
    }

    #[test]
    fn parse_segment_rejects_unmodeled_arithmetic_ops() {
        // add3.cpp seg with a comparison/ternary (`24` GT, `43 42` CB) — the
        // parser must fail closed on the first unmodeled opcode, not skip it.
        let cmp: &[u8] = &[
            0x46, 0x2D, 0xEE, 0x09, 0x2D, 0xED, 0x09, // formals
            0x4C, 0x4F, 0x11, 0x53, 0x4F, 0x01, 0x10, // LO SS stmt
            0xB9, 0xED, 0x09, 0x86, 0x41, 0x74, // LOAD
            0xB9, 0xEE, 0x09, 0x86, 0x41, 0x74, // LOAD
            0x24, // GT — unmodeled → reject
            0x43, 0x42, 0x00, 0x00, 0x41, 0x86, 0x41, 0x74,
        ];
        assert_eq!(parse_segment(cmp, NO_LOCALS), None);
    }

    #[test]
    fn a_four_byte_token_parses_as_one_operand_not_two() {
        // The misalignment this fixes: reading a 4-byte token as 2 bytes leaves
        // the parse standing on the token's own tail, which then looks like an
        // unknown opcode. Build a straight-line body whose single LOAD carries a
        // wide token and check it decodes as exactly one Load of that token.
        let seg: &[u8] = &[
            0x46, 0x2D, 0xA4, 0x96, 0x03, 0x00, // formals: one wide token
            0x4C, 0x4F, 0x11, 0x53, // LO SS
            0xB9, 0xA4, 0x96, 0x03, 0x00, 0x86, 0x41, 0x74, // LOAD <wide> int
            0x41, 0x86, 0x41, 0x74, // result-type
            0x3A, 0xE7, 0x09, // ASSIGN
            0x54, 0x02, 0x29, 0xE7, 0x09, // RETURN
            0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00, // fn tail = segment end
        ];
        assert_eq!(
            parse_segment(seg, NO_LOCALS),
            Some(BodyShape::StraightLine {
                params: vec![0xA496_0300],
                ops: vec![IlOp::Load(0xA496_0300)],
            })
        );
    }

    // ---- P2b function-level census ------------------------------------------

    #[test]
    fn census_agrees_with_the_gate_on_every_pinned_segment() {
        // This used to compare `parse_segment` with `parse_segment_detail` and
        // could not fail: the former is literally `.ok()` of the latter, so it
        // asserted a function equals itself. It protected only against someone
        // re-forking the two, which is worth keeping — hence the first assertion —
        // but it never checked the invariant its name claims.
        //
        // The invariant that matters is that **everything the parser accepts, the
        // emitter can emit**. That cannot be tested from this crate (c2-il cannot
        // depend on c2-core), so what is pinned here is the half that can be: the
        // specific shapes whose emission gates used to live in codegen, and which
        // the census therefore counted as in-class while the port refused them.
        // Each must now be refused by the parser. The other half — that no
        // *accepted* shape is refused downstream — is guarded by the fixture
        // differential and `scripts/expr_sweep.sh`.
        let all: &[&[u8]] = &[
            MVP_CALL, MVP_FRAMED, INT_TAILRET, INT_PLUS0, INT_ARGTAIL, GA_SUBMOD, GA_MULMOD,
            GA_WIDEMOD, TWO_CALLS, CALL_THEN_STMT, ARGFRAMED_PLUSK, TWO_FRAMED_CALLS, PLUS1PLUS2,
        ];
        for seg in all {
            assert_eq!(
                parse_segment(seg, NO_LOCALS).is_some(),
                parse_segment_detail(seg, NO_LOCALS).is_ok(),
                "the two entry points have been re-forked"
            );
        }

        // Shapes that parse as a well-formed straight-line body but that
        // `select_text` declines. Each is refused in the parser now.
        let params = vec![0x10u32, 0x11];
        let a = IlOp::Load(0x10);
        let b = IlOp::Load(0x11);
        for (ops, why) in [
            (vec![a, IlOp::Lit(3), IlOp::Mul], "multiply by a constant"),
            (vec![b], "bare non-first parameter"),
            (vec![IlOp::Lit(5), a, IlOp::Sub], "const - reg needs subfic"),
            (vec![IlOp::Lit(-70000)], "negative wide constant"),
        ] {
            assert!(
                straight_line_is_out_of_class(&ops, &params),
                "parser must refuse: {why}"
            );
        }
        // ...and the neighbours that really do emit must stay accepted.
        for (ops, why) in [
            (vec![a, b, IlOp::Add], "a + b"),
            (vec![a], "bare first parameter"),
            (vec![IlOp::Lit(70000)], "positive wide constant"),
        ] {
            assert!(
                !straight_line_is_out_of_class(&ops, &params),
                "parser must accept: {why}"
            );
        }
    }

    #[test]
    fn census_names_the_first_blocking_opcode() {
        // A comparison (`24` GT) in the operand stream buckets as `expr-cmp-gt`,
        // and the offset points at the `24` itself — not at some later byte.
        let cmp: &[u8] = &[
            0x46, 0x2D, 0xEE, 0x09, 0x2D, 0xED, 0x09, 0x4C, 0x4F, 0x11, 0x53, 0x4F, 0x01, 0x10,
            0xB9, 0xED, 0x09, 0x86, 0x41, 0x74, 0xB9, 0xEE, 0x09, 0x86, 0x41, 0x74,
            0x24, // GT
            0x43, 0x42, 0x00, 0x00, 0x41, 0x86, 0x41, 0x74,
        ];
        let b = parse_segment_detail(cmp, NO_LOCALS).unwrap_err();
        assert_eq!(b.feature(), "expr-cmp-gt");
        assert_eq!(cmp[b.off], 0x24);
    }

    #[test]
    fn census_reports_the_whole_operand_type_not_its_shared_first_byte() {
        // An `unsigned` operand's inline type shares its first byte (`86`) with
        // `int`, so bucketing on that byte would merge every non-int type into
        // one meaningless class. The bucket must carry the full triple.
        let mut seg = INT_TAILRET.to_vec();
        // Corrupt the argument LOAD's type `86 41 74` → `86 41 75` (a distinct
        // type), leaving everything else intact.
        let load = seg.windows(6).position(|w| w == [0xB9, 0xE5, 0x09, 0x86, 0x41, 0x74]).unwrap();
        seg[load + 5] = 0x75;
        let b = parse_segment_detail(&seg, NO_LOCALS).unwrap_err();
        assert_eq!(b.feature(), "expr-load-type-864175");
        assert_eq!(seg[b.off], 0xB9, "reported at the LOAD, not mid-type");
    }
}
