pub(crate) mod chain;
pub use self::chain::{chain_form, ChainForm};
pub(crate) mod expr;
pub(crate) mod mcall;
pub(crate) mod shapes;

use self::chain::{
    additive_chain_canonical, canonicalize_chain, has_repeated_leaf, leaves_ascending,
    straight_line_out_of_class_ctx,
};
use self::expr::{
    eat_fn_tail, eat_return_head, eat_return_plumbing, eat_scopes, intrinsic_name, parse_expr,
    parse_formals, BODY_SCOPE_DEPTH,
};
use self::shapes::parse_params;
use self::shapes::{
    eat_ctor_this_epilogue, parse_call_shape, try_parse_addr_leaf, try_parse_assign_body_detail,
    try_parse_compare, try_parse_empty_dtor_delegation, try_parse_float_leaf,
    try_parse_indirect_load_leaf, try_parse_ptr_identity_leaf,
};
use super::readers::{eat_byte, find_subslice, read_token_var, read_type, read_varint};
use super::sy::SyView;
use super::{CompareLeaf, IlOp};

/// **How many CALL tokens a function segment contains** — the D6 frame measure
/// (`docs/IL_CALL_IN_EXPR.md` §18).
///
/// The question every remaining census row has to answer is whether its lowering
/// is *local*, and the coarsest form of that question is whether the body needs a
/// **frame**: a body that issues two or more calls must save LR, because the first
/// `bl` clobbers it and the return address is still needed. That is a property of
/// the body alone, so it is measurable without any codegen — and it is measurable
/// **outside** the modeled grammar, which is the point: the grammar stops at the
/// first unmodeled byte, and this walk does not stop at all.
///
/// The walk is not a parse and is **graded rather than asserted**. A `BD` counts
/// only when *every* field of the decoded CALL token
/// `BD <ret TYPE> <conv> <varint fn-type-id>` (`docs/IL_CALL_IN_EXPR.md` §0) is
/// present and reads a **measured** value, and the cursor then skips the whole
/// token so a `BD` inside a consumed payload cannot be recounted. Everything else
/// advances one byte.
///
/// The three gates, and each is a field that never varied — so it is required
/// literally and fails closed, rather than being skipped as "probably constant":
///
/// * the **calling-convention byte is `00`**. 15,095 of 16,100 `BD`-plus-TYPE
///   sites in `src/lazer/meta_ham/HamUI.cpp` read `00` and the rest are spread
///   over 200-odd distinct bytes — the signature of a payload byte, not a field.
/// * the **fn-type-id uses `read_varint`'s `80` escape form**: 15,090 of 15,095.
/// * its value is **≥ 0x1000**. Function-type ids are allocated per TU from
///   0x1000 (`parse_call_shape`), so the short varint form cannot spell one.
///   Measured range 0x1001…0x1081 across the fixtures and 0x1001…0xFA89 in the
///   wild TU; exactly one candidate site fell below and it is a false positive.
///
/// A bare `67` (virtual dispatch) is **not** counted: a virtual call carries its
/// own `BD` as well, so counting the `67` too double-counted it — measured, and
/// removing it is part of what took the grade from 98.0 % to 98.7 %.
///
/// **The grade, MEASURED.** Over the 110 fixtures plus the D6 probes — every TU
/// where `.gl` binds one name per segment, so segment *k* pairs 1:1 with emitted
/// function *k* — this count agrees with the reference obj's own `bl`/`b` count on
/// **696 of 705 functions (98.7 %)**. Both failure directions are named and both
/// are one-sided:
///
/// * **undercount** — an `0x40` intrinsic that lowers to a real branch is not a
///   `BD` (`memcpy`, `memset`, `dynamic_cast`, an aggregate copy): 6 witnesses.
/// * **overcount** — c2 inlined or folded a call the IL still spells (an intra-TU
///   callee it cloned, a destructor whose second call folded away): 3 witnesses.
///
/// The **in-class functions are the standing control group** and the census
/// reports them: a shape the whole-body parser accepted as a leaf cannot contain
/// two calls, so `calls-2plus` among `indirect-load-leaf` / `straight-line` /
/// `empty-body` is a direct read of the residual false-positive rate.
///
/// Diagnostic only. Nothing here is consulted by the emitter or by acceptance.
pub(crate) fn call_tokens(seg: &[u8]) -> usize {
    /// The floor of the per-TU function-type id space (`parse_call_shape`).
    const FN_TYPE_ID_MIN: i32 = 0x1000;
    let mut n = 0usize;
    let mut p = 0usize;
    while p < seg.len() {
        if seg[p] != 0xBD {
            p += 1;
            continue;
        }
        let ok = read_type(seg, p + 1).and_then(|(_, _, _, tw)| {
            let q = p + 1 + tw;
            // the calling-convention byte, then the escape-form fn-type id
            if seg.get(q) != Some(&0x00) || seg.get(q + 1) != Some(&0x80) {
                return None;
            }
            let mut e = q + 1;
            let id = read_varint(seg, &mut e)?;
            (id >= FN_TYPE_ID_MIN).then_some(e)
        });
        match ok {
            Some(q) => {
                n += 1;
                p = q;
            }
            None => p += 1,
        }
    }
    n
}

/// One recognized whole-body shape of a single `.ex` function segment. Every
/// accepted body is *exactly* one of these — the parser (see [`parse_segment`])
/// is a positive whole-stream parse that reaches the segment's end, so anything
/// it does not model produces `None` and the caller reports `NotImplemented`.
/// Which sub-object a [`BodyShape::EmptyDtorDelegation`] destroys, and therefore
/// which of the two receiver productions its address came from
/// (`docs/IL_CALL_IN_EXPR.md` §14.3). Recorded rather than inferred from `adjust`
/// — a **member** at offset 0 and a **base** at adjust 0 emit the identical four
/// bytes, so the emitter cannot tell them apart and only the census wants to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum DtorSubObject {
    /// The single non-virtual base, reached through the `this`-adjust intrinsic
    /// 2113 (`docs/IL_CALL_IN_EXPR.md` §5 — the D1 shape).
    Base,
    /// A data member, reached by a plain `27` byte-offset add with no intrinsic
    /// (§14.3, §15).
    Member,
}

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
    ///
    /// `params` are the formals and `arg_ops` is the argument — a bare `[Load]`
    /// of one of them, which is **not necessarily the formal already in r3**.
    /// Both are carried for the same reason [`BodyShape::IntTailCall`] carries
    /// them: the argument register move is a function of the formal's position,
    /// and dropping the list here made the emitter assume position 0 (a live
    /// wrong-bytes emit — `c2_core::codegen::framed_call_text`).
    FramedCall { add_k: i32, callee_tok: u32, params: Vec<u32>, arg_ops: Vec<IlOp> },
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
    /// The **compiler-generated empty destructor** that destroys exactly one
    /// sub-object and nothing else: either its single non-virtual **base** through
    /// the `this`-adjust intrinsic at adjust 0, or a single destructible **member**
    /// at byte offset `adjust` reached by a plain `27` offset add
    /// (`docs/IL_CALL_IN_EXPR.md` §5, §15). The call has no result and nothing
    /// follows it, so the whole function is
    /// `[addi r3,r3,adjust ;] b <sub-object-dtor>`.
    ///
    /// `adjust == 0` emits exactly what [`BodyShape::VoidTailCall`] emits; a
    /// nonzero `adjust` prepends the one `addi`, expressed as the argument-setup
    /// operand stream `[Load(this), Lit(adjust), Add]` so it lowers through the
    /// existing integer tail-call emitter rather than a new one (`bundle.rs`).
    /// Kept as its own variant so the census can attribute the movement, and
    /// because its grammar admits two opaque trailers that must not be admitted
    /// anywhere else. See [`shapes::try_parse_empty_dtor_delegation`].
    EmptyDtorDelegation {
        callee_tok: u32,
        this_tok: u32,
        adjust: i32,
        sub_object: DtorSubObject,
    },
    /// An **indirect-load leaf**: the whole body is one load through a pointer
    /// (`return *p;`, `return s->m;`, `return p[k];`, `return mMember;`), which c2
    /// lowers to a single `lwz rD, off(rBase)`. `ops` is always exactly
    /// `[Load(base), LoadInd { off }]` and `params` includes a member function's
    /// `this` at index 0. See [`try_parse_indirect_load_leaf`].
    IndirectLoad { params: Vec<u32>, ops: Vec<IlOp> },
    /// An **address leaf**: the whole body is one sub-object *address*
    /// (`return &s->m;`, `return &p->Base::m;`, `return s->arr;`), which c2
    /// lowers to a single `addi rD, rBase, off` — or to nothing at all when
    /// `off` is 0. `ops` is always exactly `[Load(base), AddrOf { off }]` and
    /// `params` includes a member function's `this` at index 0.
    ///
    /// Kept apart from [`BodyShape::IndirectLoad`] because the two differ by the
    /// single `30` token and emit different instructions — admitting one as the
    /// other is a wrong-bytes emit, not a gap. See [`shapes::try_parse_addr_leaf`].
    AddrLeaf { params: Vec<u32>, ops: Vec<IlOp> },
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
    ///
    /// **Why `u64`.** The `26`-in-expression family (`mcall`) packs a *pair* of
    /// constructs here — the receiver form and the construct that blocks the body
    /// **after** it (`docs/IL_CALL_IN_EXPR.md` §16) — and the pair does not fit in
    /// 32 bits without truncating an intrinsic selector, which would silently
    /// *merge* two census buckets. Merging buckets is the one failure a census
    /// instrument cannot survive, so the field is widened instead of squeezed.
    /// Every other producer uses the low 24 bits exactly as before.
    pub aux: u64,
}

/// Census `ctx` for a function whose body parses in class but whose
/// optimization-settings word is not one this port emits under.
///
/// Raised **after** the body parse and only for an otherwise-in-class function,
/// deliberately: gating it up front would replace every real function's actual
/// blocking feature with this one and destroy the histogram that ranks the
/// roadmap. Applied last, it removes exactly the over-claim and nothing else.
pub(crate) const OPT_MODE: &str = "opt-mode";

/// Census `ctx` for a body that parses as a call shape whose callee token has no
/// `.gl` symbol. See the census for why this is a refusal and not a fallback.
pub(crate) const CALLEE_UNRESOLVED_TAIL: &str = "callee-unresolved-tail-call";
pub(crate) const CALLEE_UNRESOLVED_DTOR: &str = "callee-unresolved-dtor-delegation";
pub(crate) const CALLEE_UNRESOLVED_FRAMED: &str = "callee-unresolved-framed-call";

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
        // The per-function optimization-settings word, when it is not one this
        // port emits under. Rendered from [`Block::aux`] for the same reason the
        // intrinsic selector is: the word IS the feature, and `ctx` is a
        // `&'static str`. `docs/OPT_MODE.md` decodes the values.
        if self.ctx == OPT_MODE {
            return format!("opt-mode-{:08x}", self.aux);
        }
        if self.ctx == "expr-intrinsic" || self.ctx == "call-intrinsic" {
            return format!("{}-{}", self.ctx, intrinsic_name(self.aux as i32));
        }
        // The `26`-in-expression family (D2, `docs/IL_CALL_IN_EXPR.md` §14). The
        // whole bucket used to be one key — 286,240 functions, 12.9 % of the
        // blocked workload, naming 0.2 % of its own contents — and `mcall` walks
        // the production far enough to say which construct the `26` opened, plus
        // whether the *whole* segment would parse if that one form were admitted.
        // Everything is in `aux` because `ctx` is a `&'static str` and neither the
        // intrinsic selector nor the residue opcode is one.
        if self.ctx == mcall::CALL_IN_EXPR {
            return mcall::feature(self.aux);
        }
        // Operand-type blocks report the type's `<tag> <kind>` — **and not its
        // id**, which is the whole content of this key's history.
        //
        // A TYPE is `<tag> <kind> <LEB128 id>` (`docs/IL_TYPE_TAGS.md` §1). The
        // first two bytes are fixed vocabulary — the tag is the slot's width plus
        // a qualifier (`86` plain, `A6` const, `96` volatile), the kind's low
        // nibble is the type *class* (1 signed · 2 unsigned · 3 data pointer ·
        // 4 code pointer · 5 real · 6 aggregate · 7 void) — so together they name
        // the construct a widening would have to implement. The **id is an index
        // into the TU's own type table**: every distinct pointee and every
        // typedef gets a fresh one, and the same construct is numbered
        // differently in every TU.
        //
        // Putting that id in the bucket *name* shattered one construct into 256
        // shards, and a ranked histogram cannot show a shattered construct at
        // all. It hid `expr-load-type-A643` — a const-qualified 4-byte pointer
        // operand, 666,907 functions, 31 % of the blocked workload — behind rows
        // a fifth its size, and it hid the same class a second time by absorbing
        // 82.9 % of the address-leaf rung's gain in shards no ranking could
        // attribute. `GAPS.md` §6 had recorded the failure since the first census
        // and it was regrouped **by hand** for one analysis instead of being
        // fixed, which is exactly why it recurred.
        //
        // The id is not discarded — [`Block::aux`] still carries the whole triple
        // packed exactly as [`blk_type`] wrote it, and [`super::census::FnCensus`]
        // keeps the raw bytes of the site. It is kept out of the *name*, which is
        // the only place it did damage.
        if self.aux != 0 {
            return format!(
                "{}-{:02X}{:02X}",
                self.ctx,
                (self.aux >> 16) & 0xFF,
                (self.aux >> 8) & 0xFF,
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
            let named = expr_opcode_name(b);
            return match named {
                Some(n) => format!("expr-{n}"),
                None => format!("expr-op-0x{b:02X}"),
            };
        }
        format!("{}-0x{b:02X}", self.ctx)
    }
}

/// The **capture-verified** names of the operand-stream opcodes, shared by the
/// `expr-*` census keys and by `mcall`'s second-blocker keys so the two can never
/// disagree about what a byte is called.
///
/// Only add a name here once a capture has established it. An earlier revision of
/// this table guessed the relational opcodes from their numeric order and got `!=`,
/// `<=` and `>=` wrong while missing `==` entirely — which silently mislabelled
/// census buckets, the one thing this instrument exists to avoid. A hex bucket is a
/// result; a wrong name is a lie that survives into the roadmap.
///
/// Signedness is NOT in the opcode: signed and unsigned probes emit the same byte
/// and differ only in the operand type (`86 41 74` int vs `86 42 75` unsigned).
pub(crate) fn expr_opcode_name(b: u8) -> Option<&'static str> {
    #[allow(clippy::match_same_arms)]
    {
            match b {
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
            }
    }
}

/// Build a [`Block`] at the current parse position.
pub(crate) fn blk(seg: &[u8], p: usize, ctx: &'static str) -> Block {
    Block { ctx, byte: seg.get(p).copied(), off: p, aux: 0 }
}

/// Build an operand-*type* [`Block`]: `p` points at the 3-byte inline type that
/// is not the modeled int (`86 41 74`), `report_at` at the operand it belongs
/// to. Packs the triple into [`Block::aux`].
///
/// The whole triple is packed, id included — an analysis that wants the id has
/// it — but [`Block::feature`] renders only `<tag> <kind>`, because the id is a
/// per-TU table index and a bucket named after one is 256 buckets. See that
/// method's comment for what the sharding cost.
pub(crate) fn blk_type(seg: &[u8], p: usize, report_at: usize, ctx: &'static str) -> Block {
    let g = |i: usize| seg.get(p + i).copied().unwrap_or(0) as u64;
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
pub(crate) fn parse_segment(seg: &[u8], sy: SyView) -> Option<BodyShape> {
    parse_segment_detail(seg, sy).ok()
}

/// [`parse_segment`] with the fail-closed *reason* preserved (P2b census).
/// Acceptance is identical — `parse_segment` is `.ok()` of this — so the census
/// can never disagree with the gate about what is in class.
pub(crate) fn parse_segment_detail(seg: &[u8], sy: SyView) -> Result<BodyShape, Block> {
    let r = parse_segment_shape(seg, sy);
    // D2's whole-body-completeness bit. `parse_expr` classified the construct but
    // has no view of the segment as a whole, and this is the one place that has
    // both the block and the `LO` offset. Refusals only, and an `Err` stays an
    // `Err` — the census key moves, acceptance does not. See
    // [`mcall::whole_body_is_one_value`] for why the bit is worth more than the
    // sub-bucket count it decorates.
    match r {
        Err(b) if b.ctx == mcall::CALL_IN_EXPR => {
            match find_subslice(seg, &[0x4C, 0x4F, 0x11]) {
                Some(lo) => Err(mcall::mark_whole(seg, lo, b)),
                None => Err(b),
            }
        }
        other => other,
    }
}

fn parse_segment_shape(seg: &[u8], sy: SyView) -> Result<BodyShape, Block> {
    let lo = find_subslice(seg, &[0x4C, 0x4F, 0x11]).ok_or(Block {
        ctx: "lo-marker",
        byte: None,
        off: 0,
        aux: 0,
    })?;

    // Every shape below maps a formal token to an argument register by its
    // **position** in the formals list. That is only the same thing as its
    // register number while each parameter occupies exactly one register, and a
    // by-value aggregate wider than 8 bytes does not: it shifts every later
    // parameter along. So the precondition is established once, here, for all of
    // them, rather than re-derived per shape.
    //
    // This is the fourth instance of the pattern in GAPS §6 — two facts sharing
    // one field, indistinguishable across the whole corpus because every fixture
    // parameter was a scalar. It emitted `lwz r3,0(r4)` for `lwz r3,0(r6)` in
    // `int gb(Big v, H* h) { return h->mi; }`, in class, on mainline, with all
    // four mode lanes and the 2,885-case sweep green.
    //
    // `.sy` is the only layer that carries a parameter's width (`.ex`'s formals
    // region is tokens alone), so a segment whose `.sy` block did not bind has
    // *undetermined* widths and refuses — it does not fall back to assuming one
    // register each, which is precisely the assumption that was wrong.
    // Only asserted when there *is* a formals list to assert it about. A segment
    // whose formals region does not parse cannot reach any shape that maps a
    // formal to a register — every one of those re-reads the same list through the
    // one anchor ([`formals_marker`]) and refuses there — so this gate declines to
    // restate a refusal it does not own, and the census keeps reporting the real
    // blocker instead of `formals-marker` for every such body.
    if let Ok(formals) = parse_formals(seg, lo) {
        if let Err(ctx) = sy.formals_are_one_register_each(&formals) {
            return Err(Block { ctx, byte: None, off: lo, aux: 0 });
        }
    }

    let locals = sy.locals;
    let mut p = lo + 3;
    // 'SS' statement-start — the body's own lexical scope — then any further brace
    // scopes and line markers. A body wrapped in braces used to refuse here as
    // `body-0x53`, the largest single blocking feature on the real workload.
    if !eat_byte(seg, &mut p, 0x53) {
        return Err(blk(seg, p, "stmt-start"));
    }
    let mut depth = BODY_SCOPE_DEPTH;
    eat_scopes(seg, &mut p, &mut depth)?;

    match *seg.get(p).ok_or(blk(seg, p, "body"))? {
        // An EMPTY body opens directly on the return plumbing's `3A` assign —
        // there is no expression at all. `eat_return_plumbing` still has to
        // reach the segment end, so any trailing statement or unexpected operand
        // fails the function closed exactly as it does for every other shape.
        0x3A => {
            eat_return_head(seg, &mut p, false, depth)?;
            // …and then, for a **constructor**, the `return this` that sits
            // between the RETURN and the tail. It emits nothing — `this` is
            // already in r3 and an empty body cannot have moved it — so the shape
            // is the same `EmptyBody` either way. Absent, this is a no-op and the
            // tail follows immediately, exactly as before.
            // [`shapes::eat_ctor_this_epilogue`] has the capture and the reason
            // the leaf restriction is not conservatism.
            eat_ctor_this_epilogue(seg, &mut p, lo);
            eat_fn_tail(seg, &mut p)?;
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
                try_parse_assign_body_detail(seg, p, lo, locals, depth)
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
            // …and the pointer *identity* leaf (`return p;` / `return this;` /
            // a ptr→ptr cast of either), which is the same production minus the
            // `30` load. Tried after it, because a body that has a `30` is a
            // getter and this one must not see it: the shape between the two —
            // an offset add with no `30`, `return &s->m;` — emits an `addi` and
            // is refused by both. Non-committal like the others: it works on a
            // copy of the cursor and returns None with no side effects.
            if let Some(shape) = try_parse_ptr_identity_leaf(seg, p, lo) {
                return Ok(shape);
            }
            // …and the **address** leaf, which is that same shape *with* the
            // offset add the identity refuses (`return &s->m;`, `return s->arr;`,
            // `return &p->Base::m;`) and which emits the one `addi` the identity
            // must not. Tried after both, so a body that has a `30` is still a
            // getter and a bare pointer is still an identity; this one is anchored
            // on the `41` result following the adds. Non-committal: it works on a
            // copy of the cursor and returns None with no side effects.
            if let Some(shape) = try_parse_addr_leaf(seg, p, lo) {
                return Ok(shape);
            }
            // …and the generated empty destructor, whose body opens on a literal
            // `0` and is otherwise a member call. Anchored on `33 <int> 0` then a
            // `26`, so it cannot collide with the intrinsic-2117 designator above
            // (whose literal is the selector `2117`) nor with a real arithmetic
            // leaf (whose literal is followed by an operand or an operator).
            // Non-committal: works on a copy of the cursor, returns None with no
            // side effects, so a declining body still reports its own blocker.
            if let Some(shape) = try_parse_empty_dtor_delegation(seg, p, lo, depth) {
                return Ok(shape);
            }
            let ops = parse_expr(seg, &mut p, 0x41)?;
            eat_return_plumbing(seg, &mut p, true, depth)?;
            let params = parse_params(seg, lo)?;
            // A parameter used twice licenses c2's algebraic rewriter.
            if has_repeated_leaf(&ops) {
                return Err(Block { ctx: "expr-repeated-leaf", byte: None, off: p, aux: 0 });
            }
            // Gates that used to live in codegen; see
            // `straight_line_out_of_class_ctx`, which names *which* of them fired
            // so the row can be ranked clause by clause.
            if let Some(ctx) = straight_line_out_of_class_ctx(&ops, &params) {
                return Err(Block { ctx, byte: None, off: p, aux: 0 });
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
            parse_segment(&free_fn(seg), NO_LOCALS),
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
            parse_segment(&free_fn(konst), NO_LOCALS),
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
            parse_segment(&free_fn(kw), NO_LOCALS),
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
            parse_segment(&free_fn(seg), NO_LOCALS),
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
            parse_segment(&free_fn(MVP_CALL), NO_LOCALS),
            Some(BodyShape::VoidTailCall { callee_tok: 0xE309 })
        );
    }

    #[test]
    fn parse_segment_accepts_framed_call() {
        // `int f(int a){ return g(a) + 1; }` (mvp_framed): int call, single
        // passthrough arg, `55` call-end, exactly one `+1` post-op.
        assert_eq!(
            parse_segment(&free_fn(MVP_FRAMED), NO_LOCALS),
            Some(BodyShape::FramedCall {
                add_k: 1,
                callee_tok: 0xE409,
                params: vec![0xE509],
                arg_ops: vec![IlOp::Load(0xE509)],
            })
        );
    }

    #[test]
    fn parse_segment_accepts_int_tail_call_family() {
        // The three int tail-call shapes (formals `46 2d e509` = param a → r3):
        //   passthrough `g(a)` and identity-fold `g(a)+0` → arg `[Load a]`;
        //   arg-setup `g(a+1)` → arg `[Load a, Lit 1, Add]`. All are
        //   `IntTailCall` (a net-identity post-op is a tail call, not framed).
        assert_eq!(
            parse_segment(&free_fn(INT_TAILRET), NO_LOCALS),
            Some(BodyShape::IntTailCall {
                params: vec![0xE509],
                arg_ops: vec![IlOp::Load(0xE509)],
                callee_tok: 0xE409,
            }),
            "passthrough g(a)"
        );
        assert_eq!(
            parse_segment(&free_fn(INT_PLUS0), NO_LOCALS),
            Some(BodyShape::IntTailCall {
                params: vec![0xE509],
                arg_ops: vec![IlOp::Load(0xE509)],
                callee_tok: 0xE409,
            }),
            "identity-fold g(a)+0 routes to a tail call, not FramedCall{{add_k:0}}"
        );
        assert_eq!(
            parse_segment(&free_fn(INT_ARGTAIL), NO_LOCALS),
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
            parse_segment(&free_fn(MVP_FRAMED), NO_LOCALS),
            Some(BodyShape::FramedCall {
                add_k: 1,
                callee_tok: 0xE409,
                params: vec![0xE509],
                arg_ops: vec![IlOp::Load(0xE509)],
            }),
            "g(a)+1 is framed"
        );
        assert!(
            matches!(parse_segment(&free_fn(INT_PLUS0), NO_LOCALS), Some(BodyShape::IntTailCall { .. })),
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
            assert_eq!(parse_segment(&free_fn(seg), NO_LOCALS), None, "must reject: {label}");
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
        assert_eq!(parse_segment(&free_fn(cmp), NO_LOCALS), None);
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
            parse_segment(&free_fn(seg), NO_LOCALS),
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
                parse_segment(&free_fn(seg), NO_LOCALS).is_some(),
                parse_segment_detail(&free_fn(seg), NO_LOCALS).is_ok(),
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
            (vec![IlOp::Load(0x99)], "bare non-formal token"),
            (vec![IlOp::Lit(5), a, IlOp::Sub], "const - reg needs subfic"),
            (vec![IlOp::Lit(-70000)], "negative wide constant"),
        ] {
            assert!(
                straight_line_out_of_class_ctx(&ops, &params).is_some(),
                "parser must refuse: {why}"
            );
        }
        // ...and the neighbours that really do emit must stay accepted. A bare
        // non-first formal is one of them now: it is the single `mr r3,rN` W18
        // grades (`fixtures/cpp/w18_reg_move.cpp`), not a refusal.
        for (ops, why) in [
            (vec![a, b, IlOp::Add], "a + b"),
            (vec![a], "bare first parameter"),
            (vec![b], "bare non-first parameter -> mr r3,r4"),
            (vec![IlOp::Lit(70000)], "positive wide constant"),
        ] {
            assert!(
                straight_line_out_of_class_ctx(&ops, &params).is_none(),
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
        // `b.off` indexes the segment that was PARSED, so hold on to it.
        let cmp = free_fn(cmp);
        let b = parse_segment_detail(&cmp, NO_LOCALS).unwrap_err();
        assert_eq!(b.feature(), "expr-cmp-gt");
        assert_eq!(cmp[b.off], 0x24);
    }

    /// Retype the argument LOAD's inline type in a copy of [`INT_TAILRET`],
    /// leaving every other byte intact, and return the resulting block.
    fn load_typed(t: [u8; 3]) -> Block {
        let mut seg = INT_TAILRET.to_vec();
        let load = seg
            .windows(6)
            .position(|w| w == [0xB9, 0xE5, 0x09, 0x86, 0x41, 0x74])
            .unwrap();
        seg[load + 3..load + 6].copy_from_slice(&t);
        let seg = free_fn(&seg);
        let b = parse_segment_detail(&seg, NO_LOCALS).unwrap_err();
        assert_eq!(seg[b.off], 0xB9, "reported at the LOAD, not mid-type");
        b
    }

    #[test]
    fn census_reports_the_operand_type_class_not_its_shared_first_byte() {
        // Every 4-byte type's inline TYPE starts `86`, so bucketing on that byte
        // would merge pointer, float and aggregate operands into one meaningless
        // class. The bucket must carry the `kind` byte, which is the class.
        //
        // The two POINTER rows this table used to carry (`8643`, `A643` — which
        // were 45.9 % of the blocked workload between them) are gone from it,
        // because the LOAD position now admits them; the test below is their
        // replacement, and the classes that still refuse are still keyed by class.
        for (t, want) in [
            ([0x86u8, 0x45, 0x40], "expr-load-type-8645"), // float
            ([0x88, 0x85, 0x41], "expr-load-type-8885"),   // double
            ([0x88, 0x81, 0x13], "expr-load-type-8881"),   // long long
            ([0x86, 0x46, 0x80], "expr-load-type-8646"),   // aggregate
            ([0x82, 0x07, 0x03], "expr-load-type-8207"),   // void
        ] {
            assert_eq!(load_typed(t).feature(), want, "type {t:02X?}");
        }
    }

    /// The rung: a 4-byte pointer TYPE at the LOAD is an operand, not a blocker.
    /// Retyping the argument LOAD of [`INT_TAILRET`] — one field, every other byte
    /// left alone — must now PARSE rather than bucket, in all four tag spellings
    /// and both pointer kinds, and the resulting shape must be the same
    /// `int-tail-call` the int spelling produced. The negative half is the table
    /// above: the classes that are not 4-byte pointers still refuse at the same
    /// position with the same key.
    #[test]
    fn a_four_byte_pointer_at_the_load_is_an_operand_not_a_blocker() {
        let int_shape = parse_segment(&free_fn(INT_TAILRET), NO_LOCALS).unwrap();
        // Three-byte spellings, so the substitution is field-for-field and no
        // other byte of the segment moves. (A real `int*` id is usually two LEB
        // bytes — `86 43 F4 08` — and `read_type` walks either.)
        for t in [
            [0x86u8, 0x43, 0x74], // a data pointer
            [0xA6, 0x43, 0x74],   // const-qualified: `int* const`, and `this`
            [0x96, 0x43, 0x74],   // volatile-qualified
            [0xB6, 0x43, 0x74],   // const volatile
            [0x86, 0x44, 0x74],   // a CODE pointer, kind class 4
        ] {
            let mut seg = INT_TAILRET.to_vec();
            let load = seg
                .windows(6)
                .position(|w| w == [0xB9, 0xE5, 0x09, 0x86, 0x41, 0x74])
                .unwrap();
            seg[load + 3..load + 6].copy_from_slice(&t);
            assert_eq!(
                parse_segment(&free_fn(&seg), NO_LOCALS).as_ref(),
                Some(&int_shape),
                "pointer type {t:02X?} must parse as the int spelling does"
            );
        }
    }

    /// The arithmetic guard, at the grammar level. `int* f(int* p){ return p+1; }`
    /// is transcribed verbatim from a live capture of `/tmp` probe `parith.cpp`
    /// (`docs/IL_CALL_IN_EXPR.md` §21.1) — note the literal is already **4**, the
    /// scaled byte offset, which is the measurement that says the guard is a
    /// conservatism and not a rescue. It refuses anyway, under its own key, and
    /// the identical body with an `int` operand still parses.
    #[test]
    fn a_pointer_operand_is_barred_from_arithmetic() {
        let ptr_add: &[u8] = &[
            0x46, 0x2D, 0xE3, 0x09, // formals: p = e309
            0x4C, 0x4F, 0x11, 0x53, // LO SS
            0xB9, 0xE3, 0x09, 0x86, 0x43, 0xF4, 0x08, // LOAD p, type int*
            0x33, 0x86, 0x41, 0x12, 0x04, // LIT (long) 4 — c1xx already scaled it
            0x02, // ADD
            0x41, 0x86, 0x43, 0xF4, 0x08, // result-type int*
            0x3A, 0xE5, 0x09, 0x54, 0x02, 0x29, 0xE5, 0x09, // assign + return
            0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00,
        ];
        let seg = free_fn(ptr_add);
        let b = parse_segment_detail(&seg, NO_LOCALS).unwrap_err();
        assert_eq!(b.feature(), "expr-ptr-arith:eof");
        assert!(parse_segment(&seg, NO_LOCALS).is_none());
        // The same body with an `int` operand and the same literal is exactly the
        // shape the port has emitted since the MVP, so the guard is keying on the
        // pointer and not on the addition. Written out rather than patched: the
        // int TYPE is three bytes where the pointer one is four, so a field-for-
        // field substitution would not be one.
        let int_add: &[u8] = &[
            0x46, 0x2D, 0xE3, 0x09, //
            0x4C, 0x4F, 0x11, 0x53, //
            0xB9, 0xE3, 0x09, 0x86, 0x41, 0x74, // LOAD p, type int
            0x33, 0x86, 0x41, 0x12, 0x04, // LIT (long) 4
            0x02, // ADD
            0x41, 0x86, 0x41, 0x74, // result-type int
            0x3A, 0xE5, 0x09, 0x54, 0x02, 0x29, 0xE5, 0x09, //
            0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00,
        ];
        assert!(
            parse_segment(&free_fn(int_add), NO_LOCALS).is_some(),
            "the int spelling of the same chain must still parse"
        );
        // …and a pointer operand with NO arithmetic is admitted, so the guard is
        // not simply refusing every pointer: drop the `33 <long> 4 02` and the
        // body is the pointer identity the rung admits.
        let mut plain = ptr_add.to_vec();
        plain.drain(15..21); // the LIT (5 bytes) and the ADD
        assert!(parse_segment(&free_fn(&plain), NO_LOCALS).is_some());
    }

    #[test]
    fn the_operand_type_bucket_does_not_shard_on_the_per_tu_type_id() {
        // THE de-sharding invariant. A TYPE's third field is an index into the
        // TU's own type table — every pointee and every typedef gets a fresh one
        // — so two ids under one `<tag> <kind>` are the *same* construct numbered
        // twice, and a key that carried the id split one construct into 256
        // buckets that no ranked histogram could add back up. `86 45 40` and
        // `86 45 83` are two `float`s numbered twice — the pointer pair this
        // used to be written over (`86 43 F4` `int*`, `86 43 83` `void*`) is no
        // longer a blocker at all, so the invariant is now carried by a class
        // that still shards.
        let a = load_typed([0x86, 0x45, 0x40]);
        let b = load_typed([0x86, 0x45, 0x83]);
        assert_eq!(a.feature(), b.feature(), "one construct, one bucket");
        // …and the id is *kept*, just not in the name: `aux` still holds the
        // whole triple, so an analysis that wants the type table index has it.
        assert_ne!(a.aux, b.aux);
        assert_eq!(a.aux, 0x864540);
        assert_eq!(b.aux, 0x864583);
    }

    #[test]
    fn the_operand_type_rekey_is_an_exact_coarsening() {
        // The re-key must be a *partition* of the old one: every block's new key
        // is a function of its old key, so functions can only merge, never move
        // sideways. Checked here at the level the property lives at — the key
        // formatter — over the four shapes `feature` can take, because the parse
        // itself is untouched and so every `Block` is bit-identical to before.
        let old = |b: Block| -> String {
            if b.ctx == "expr-intrinsic" || b.ctx == "call-intrinsic" {
                return format!("{}-{}", b.ctx, intrinsic_name(b.aux as i32));
            }
            if b.ctx == mcall::CALL_IN_EXPR {
                return mcall::feature(b.aux);
            }
            if b.aux != 0 {
                return format!(
                    "{}-{:02X}{:02X}{:02X}",
                    b.ctx,
                    (b.aux >> 16) & 0xFF,
                    (b.aux >> 8) & 0xFF,
                    b.aux & 0xFF
                );
            }
            match b.byte {
                None => format!("{}:eof", b.ctx),
                Some(x) if b.ctx == "expr" => match expr_opcode_name(x) {
                    Some(n) => format!("expr-{n}"),
                    None => format!("expr-op-0x{x:02X}"),
                },
                Some(x) => format!("{}-0x{x:02X}", b.ctx),
            }
        };
        // Only the pairings the parser can actually produce: `aux` is nonzero
        // for the operand-type blocks ([`blk_type`]), for the two intrinsic
        // contexts, and for `mcall`'s packed pair — nowhere else.
        let mut cases: Vec<(&'static str, u64)> = Vec::new();
        for ctx in ["expr-load-type", "expr-lit-type"] {
            for aux in [0x864174u64, 0x864175, 0x8643F4, 0xA64383, 0x888541, 0x000012] {
                cases.push((ctx, aux));
            }
        }
        for ctx in ["expr-intrinsic", "call-intrinsic"] {
            for aux in [15u64, 2113, 2117, 0xDF] {
                cases.push((ctx, aux));
            }
        }
        cases.push((mcall::CALL_IN_EXPR, 11));
        for ctx in ["expr", "body", "call-token", "fn-tail", "stmt-start"] {
            cases.push((ctx, 0));
        }
        let mut map: std::collections::BTreeMap<String, String> = std::collections::BTreeMap::new();
        for (ctx, aux) in cases {
            for byte in [None, Some(0x24u8), Some(0xB9)] {
                let b = Block { ctx, byte, off: 0, aux };
                let (o, n) = (old(b), b.feature());
                // Same old key ⇒ same new key. That is exactly "the new
                // partition is a coarsening of the old one", and it is what
                // makes the census difference attributable.
                match map.get(&o) {
                    Some(prev) => assert_eq!(prev, &n, "old key {o} maps to two new keys"),
                    None => {
                        map.insert(o.clone(), n.clone());
                    }
                }
                // Nothing outside the operand-type family may move at all.
                if !ctx.ends_with("-type") {
                    assert_eq!(o, n, "non-type key moved");
                }
            }
        }
        // And the family really did merge, or the test above is vacuous.
        let merged: Vec<_> = map.iter().filter(|(o, n)| o != n).collect();
        assert!(merged.len() >= 4, "expected the type family to fold: {merged:?}");
    }

    #[test]
    fn the_call_token_count_is_the_number_of_calls_the_body_issues() {
        // The D6 frame measure (§18). Pinned on the segments whose call count is
        // known from their *source*, not from a re-read of the walk — including
        // the ones with no call at all, because a counter that never returns 0
        // would report every leaf as needing a frame.
        for (seg, want, what) in [
            (MVP_CALL, 1usize, "void f(){ g(); }"),
            (MVP_FRAMED, 1, "int f(int a){ return g(a)+1; }"),
            (TWO_CALLS, 2, "void f(){ g(); g(); }"),
            (CALL_THEN_STMT, 1, "void call then a second statement"),
            (TWO_FRAMED_CALLS, 2, "two framed calls"),
            (PLUS1PLUS2, 1, "int f(int a){ return g(a)+1+2; }"),
            (GA_SUBMOD, 1, "int f(int a){ return g(a)-1; }"),
            // …and the leaves, because a counter that never returns 0 would
            // report every leaf as needing a frame.
            (IND_DEREF, 0, "return *p;"),
            (IND_THIS_GETTER, 0, "return mMember;"),
            (NARROW_LL_MEMBER, 0, "a long long member load"),
        ] {
            assert_eq!(call_tokens(&free_fn(seg)), want, "{what}");
        }
    }

    #[test]
    fn a_call_token_inside_a_consumed_payload_is_not_recounted() {
        // The walk skips the whole `BD <TYPE> <conv> <varint>` token, so a `BD`
        // byte that is *part* of one cannot be counted twice. Force the case by
        // planting `BD` in the function-type id's escape payload: `80` + 4 LE
        // bytes, one of which is `BD`.
        let mut seg = MVP_CALL.to_vec();
        let bd = seg.windows(2).position(|w| w == [0xBD, 0x82]).unwrap();
        // `BD 82 07 03 00 | 80 01 10 00 00` → keep the shape, poison the payload.
        seg[bd + 6] = 0xBD;
        seg[bd + 7] = 0xBD;
        assert_eq!(
            call_tokens(&free_fn(&seg)),
            1,
            "a BD inside the consumed token is not a second call"
        );
    }

    #[test]
    fn every_field_of_the_call_token_is_required_literally() {
        // Three fields that never varied over 15,095 wild sites. A measure that
        // skipped any of them would count a `BD` payload byte as a call — which
        // is exactly what the in-class control group caught (§18): the loose
        // version read 10,088 in-class LEAVES as `calls-2plus`.
        let base = MVP_CALL.to_vec();
        assert_eq!(call_tokens(&free_fn(&base)), 1);
        let bd = base.windows(2).position(|w| w == [0xBD, 0x82]).unwrap();
        for (off, poison, why) in [
            (4usize, 0x01u8, "calling convention must be 00"),
            (5, 0x01, "the fn-type id must use the 80 escape form"),
        ] {
            let mut seg = base.clone();
            seg[bd + off] = poison;
            assert_eq!(call_tokens(&free_fn(&seg)), 0, "{why}");
        }
        // …and the id's own value: `80 01 10 00 00` is 0x1001 little-endian, so
        // clearing the high byte of the low halfword leaves 0x0001, below the floor.
        let mut seg = base.clone();
        seg[bd + 7] = 0x00;
        assert_eq!(
            call_tokens(&free_fn(&seg)),
            0,
            "a fn-type id below 0x1000 is not one c2 allocated"
        );
    }
}
