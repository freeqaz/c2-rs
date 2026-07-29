//! Minimal IL parse for the MVP function classes: a straight-line all-`int`
//! left-associative arithmetic leaf (`int add3(int,int,int)` and friends), a
//! bare terminal void tail call (`void f(){ g(); }`), the integer tail-call
//! family `return g(<arg>)` (passthrough `g(a)`, the `g(a)+0` identity fold, and
//! arg-setup `g(a+1)`), and a framed non-leaf `return g(a) + k` (k ≠ 0). This is
//! deliberately NOT a general IL disassembler.
//!
//! **Acceptance is a positive whole-body parse (W4b2-v).** [`parse_segment`]
//! tokenizes the entire `.ex` operand stream of a function segment — from the
//! `4C 4F 11` ('LO') marker to the segment end — and accepts only if the whole
//! token sequence is exactly one of the four recognized [`BodyShape`]s; the
//! parse must *reach the end*, so trailing statements, a second call, a
//! non-trivial call-argument region, or any unmodeled byte fail the function
//! closed (`None` → the caller reports `NotImplemented`, never a mis-emit).
//! This replaced an earlier trio of gates that each matched on a *local* byte
//! neighborhood around the first CALL and so silently over-accepted (two
//! reviews caught the same two functions dropping trailing/in-argument work).
//!
//! Three `.gl`/`.ex` facts drive the emitter, per `ILPARSE` spec:
//!   * the mangled function name(s) (from `.gl`) — copied verbatim into the COFF
//!     symbol + string table (also the external callee name for call shapes);
//!   * the source path (from `.gl`) — provenance only, not embedded in the MVP
//!     obj;
//!   * the body shape (from `.ex`) — a LOAD/ADD op stream, a tail call, or a
//!     framed call, which codegen lowers to PPC.
//!
//! Reference decoder mirrored: `dc3-decomp/msvc-src/tools/il_parser.py`
//! (`ILGlobals`, `_detect_token_width`, `ILFunction._parse_body`);
//! grammar cross-checked against live-toolchain `.ex` captures of every fixture.

use crate::IlBundle;

/// A single straight-line IL operation in the integer-arithmetic class.
///
/// The binary ops are postfix (each pops two operands, pushes one result).
/// `Sub` is **non-commutative** — its operand order is load-bearing (see the
/// codegen for the `subf` operand mapping); `Add`/`Mul` are commutative.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IlOp {
    /// Load a named variable (by IL token) onto the expression stack.
    Load(u32),
    /// Push an integer literal constant (IL opcode `0x33`, `<type> <varint>`).
    Lit(i32),
    /// Push a **floating-point literal** (W13b). The payload is always an
    /// IEEE-754 **binary64** bit pattern regardless of width — a `float` literal
    /// is stored as a double whose value is already rounded to float — with the
    /// width carried separately. Held as raw bits so no rounding happens here.
    FpLit { bits: u64, double: bool },
    /// Pop rhs then lhs, push `lhs + rhs` (IL opcode `0x02`, commutative).
    Add,
    /// Pop rhs then lhs, push `lhs - rhs` (IL opcode `0x03`, NON-commutative).
    Sub,
    /// Pop rhs then lhs, push `lhs * rhs` (IL opcode `0x04`, commutative).
    Mul,
    /// Pop rhs then lhs, push `lhs / rhs` (IL opcode `0x05`, NON-commutative).
    /// Only reached on the FP path — integer division is not modeled.
    Div,
}

/// A **framed non-leaf call** of the verified `return g(a) + k` class (W4b2):
/// the call result is consumed (so `f` allocates a stack frame and is non-leaf),
/// then a small integer literal `k` is added and returned. Codegen emits the
/// constant 0x24-byte frame (prologue, `bl <callee>`, `addi r3,r3,k`, epilogue)
/// plus the `.pdata` unwind record — see `c2_core::codegen`/`coff`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FramedCall {
    /// The single external callee's mangled name (from `.gl`), e.g. `?g@@YAHH@Z`.
    pub callee: String,
    /// The post-call `+ k` literal (`k` fits a signed 16-bit `addi` immediate;
    /// commutative, so no non-commutative opt-in gate is needed).
    pub add_k: i32,
}

/// A relational operator, as encoded by a single `.ex` operand-stream opcode.
///
/// The opcode is **sign-agnostic** — signed and unsigned probes emit the same
/// byte and differ only in the operand type. Verified per relation against live
/// captures; see `docs/CODEGEN_W6_COMPARE.md` §1.1.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rel {
    /// `==`, opcode `0x1F`.
    Eq,
    /// `!=`, opcode `0x20`.
    Ne,
    /// `<=`, opcode `0x21`.
    Le,
    /// `<`, opcode `0x22`.
    Lt,
    /// `>=`, opcode `0x23`.
    Ge,
    /// `>`, opcode `0x24`.
    Gt,
}

impl IlOp {
    /// True for the binary operators a depth-2 tree node may carry.
    pub fn is_tree_binop(self) -> bool {
        matches!(self, IlOp::Add | IlOp::Sub | IlOp::Mul | IlOp::Div)
    }
}

impl Rel {
    fn from_opcode(b: u8) -> Option<Rel> {
        Some(match b {
            0x1F => Rel::Eq,
            0x20 => Rel::Ne,
            0x21 => Rel::Le,
            0x22 => Rel::Lt,
            0x23 => Rel::Ge,
            0x24 => Rel::Gt,
            _ => return None,
        })
    }
}

/// A **comparison leaf** (W6): `return <formal> <rel> <literal>;` materialized
/// to a boolean.
///
/// c2 lowers these *branchlessly* — no `cmpw`/`cmplw` at all — via carry-bit and
/// bit-extraction idioms whose exact instruction sequence depends on the
/// relation, the signedness, and (critically) on whether the literal is zero:
/// `k == 0` is folded to a shorter, different sequence rather than being a
/// special case of the general spine. See `docs/CODEGEN_W6_COMPARE.md`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CompareLeaf {
    /// The compared formal's IL token (it occupies r3, the first argument).
    pub param: u32,
    /// The relation, with the formal on the left (`<formal> <rel> <k>`).
    pub rel: Rel,
    /// Whether the *operand* type is signed (`int`) or not (`unsigned int`).
    /// The opcode does not carry this; the operand type does.
    pub signed: bool,
    /// The literal right-hand side.
    pub k: i32,
}

/// A parsed MVP function: enough to drive the codegen + COFF emitter.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IlFunction {
    /// Mangled name, e.g. `?add3@@YAHHHH@Z` (verbatim from `.gl`).
    pub mangled_name: String,
    /// Source path from `.gl`, e.g. `z:\...\mvp_add3.cpp` (provenance only).
    pub source_path: Option<String>,
    /// Formal-parameter IL tokens, in declaration order (a, b, c → r3, r4, r5).
    pub params: Vec<u32>,
    /// Straight-line body op stream (loads + adds) for an arithmetic leaf. For
    /// an **integer tail call** (`tail_call` set, int) this instead holds the
    /// single call argument's sub-expression, computed into r3 before the
    /// branch (`[Load]` passthrough, `[Load,Lit,Add]` for `g(a+1)`). Empty for a
    /// void tail call and for a framed call.
    pub ops: Vec<IlOp>,
    /// If this function is a **tail call** to a single external, its mangled
    /// name (the callee). Codegen emits a `b <callee>` with a REL24 relocation
    /// instead of an arithmetic body: a bare branch for the void tail call
    /// (`ops` empty) or `void f(){g();}`, or an argument-setup prefix + branch
    /// for an integer tail call (`ops` = the argument sub-expression). W4a:
    /// single external only.
    pub tail_call: Option<String>,
    /// If this function is a **framed non-leaf call** (`return g(a) + k`), the
    /// callee + post-op literal. Distinct from `tail_call` (which is a bare
    /// `b g`). W4b2: single-function TU, single external only.
    pub framed_call: Option<FramedCall>,
    /// If this function is a **comparison leaf** (`return a <rel> k;`, W6), the
    /// decoded comparison. Mutually exclusive with the other body kinds.
    pub compare: Option<CompareLeaf>,
    /// True iff this function's body is **empty** (`void f() {}`): no expression
    /// at all, so codegen emits a bare `blr`. Mutually exclusive with the other
    /// body kinds.
    ///
    /// If this function is a **W13a floating-point leaf**, whether it is double
    /// precision. Mutually exclusive with the other body kinds.
    pub float_leaf: Option<bool>,
    /// (These discriminators want to be one enum; that refactor is deferred
    /// until the CFG step forces a real body IR — see docs/ROADMAP.md §G4.)
    pub empty_body: bool,
}

/// The int type encoding inline in the `.ex` body (`86 41 74`), per `IL_FORMAT`.
const INT_TYPE: [u8; 3] = [0x86, 0x41, 0x74];

/// Real token-width detector (ports `il_parser._detect_token_width`): find the
/// first `4F 02`, count the bytes to the next `4F`. That gap is the token
/// width. Defaults to 2 if the anchor is not found.
pub fn detect_token_width(ex: &[u8]) -> usize {
    let mut i = 0;
    while i + 1 < ex.len() {
        if ex[i] == 0x4F && ex[i + 1] == 0x02 {
            let mut j = i + 2;
            while j < ex.len() && ex[j] != 0x4F {
                j += 1;
            }
            let gap = j - (i + 2);
            if gap == 2 || gap == 4 {
                return gap;
            }
        }
        i += 1;
    }
    2
}

/// Extract the mangled name from `.gl`: the first `?`-prefixed, NUL-terminated
/// ASCII run whose second byte is alphabetic and which contains `@@` (the
/// `__cdecl`/global marker). Mirrors `ILGlobals._parse`.
pub fn mangled_name(gl: &[u8]) -> Option<String> {
    let mut i = 0;
    while i < gl.len() {
        if gl[i] == b'?' {
            // Read to NUL.
            let start = i;
            let mut end = i;
            while end < gl.len() && gl[end] != 0 {
                end += 1;
            }
            let bytes = &gl[start..end];
            if bytes.len() >= 3
                && bytes[1].is_ascii_alphabetic()
                && contains_subslice(bytes, b"@@")
                && bytes.iter().all(|b| b.is_ascii_graphic())
            {
                return Some(String::from_utf8_lossy(bytes).into_owned());
            }
            i = end + 1;
        } else {
            i += 1;
        }
    }
    None
}

/// Extract **all** mangled names from `.gl`, in file order — one per function
/// in the translation unit. Same acceptance test as [`mangled_name`]; used for
/// multi-function TUs where `.gl` carries a name per function.
pub fn mangled_names(gl: &[u8]) -> Vec<String> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < gl.len() {
        if gl[i] == b'?' {
            let start = i;
            let mut end = i;
            while end < gl.len() && gl[end] != 0 {
                end += 1;
            }
            let bytes = &gl[start..end];
            if bytes.len() >= 3
                && bytes[1].is_ascii_alphabetic()
                && contains_subslice(bytes, b"@@")
                && bytes.iter().all(|b| b.is_ascii_graphic())
            {
                out.push(String::from_utf8_lossy(bytes).into_owned());
            }
            i = end + 1;
        } else {
            i += 1;
        }
    }
    out
}

/// Build the `.gl` **symbol index**: operand token → mangled name.
///
/// `.gl` records have the shape
/// `<kind byte> <operand token> 00 <NUL-terminated name> 00 <TYPE> …`, so a
/// record is located by its `00 <name> 00` core and the token read backwards
/// from it with the same variable-width rule the operand stream uses.
///
/// This is what binds a call to its callee. The CALL token does *not* name the
/// callee — three different callees sharing one signature produce byte-identical
/// CALL tokens — so the name comes from the `26 <tok>` symbol push that precedes
/// it, resolved through this index. Verified on a real TU: 2323 of 2323 direct
/// call sites resolve, and the complementary controlled fixtures show tokens are
/// assigned in *declaration* order but used in *call* order, and that a repeated
/// callee repeats its token.
///
/// `.sy` is deliberately not consulted: it holds local and parameter names, and
/// real callees (`?MemPushTemp@@YAXXZ`) are absent from it and present here.
///
/// Names are accepted only if they look like whole mangled identifiers, so a
/// stray NUL-delimited run inside binary payload cannot inject a false symbol.
pub fn gl_symbol_index(gl: &[u8]) -> std::collections::BTreeMap<u32, String> {
    let mut out = std::collections::BTreeMap::new();
    let mut i = 0usize;
    while i < gl.len() {
        // A record's name is a NUL-terminated printable run preceded by a NUL.
        if gl[i] != 0 {
            i += 1;
            continue;
        }
        let start = i + 1;
        let mut end = start;
        while end < gl.len() && gl[end] != 0 {
            end += 1;
        }
        if end >= gl.len() || end == start {
            i += 1;
            continue;
        }
        let name_bytes = &gl[start..end];
        let plausible = name_bytes.len() >= 3
            && name_bytes.iter().all(|b| b.is_ascii_graphic())
            && (name_bytes[0] == b'?' || name_bytes[0].is_ascii_alphabetic() || name_bytes[0] == b'_');
        if !plausible {
            i = end.max(i + 1);
            continue;
        }
        // The operand token sits immediately before the leading NUL. Try the
        // 4-byte form first, then the 2-byte one, and keep whichever decodes to
        // a token whose own width lands exactly on that NUL.
        for w in [4usize, 2] {
            if i < w {
                continue;
            }
            let p = i - w;
            if let Some((tok, got)) = read_token_var(gl, p) {
                if got == w {
                    out.entry(tok)
                        .or_insert_with(|| String::from_utf8_lossy(name_bytes).into_owned());
                    break;
                }
            }
        }
        i = end;
    }
    out
}

/// Extract the source path from `.gl`: a `<letter>:\…\<name>.cpp` NUL-terminated
/// ASCII run (case-insensitive drive + `.cpp` suffix). Provenance only.
pub fn source_path(gl: &[u8]) -> Option<String> {
    let mut i = 0;
    while i + 2 < gl.len() {
        if gl[i].is_ascii_alphabetic() && gl[i + 1] == b':' && gl[i + 2] == b'\\' {
            let start = i;
            let mut end = i;
            while end < gl.len() && gl[end] != 0 {
                end += 1;
            }
            let bytes = &gl[start..end];
            if bytes.iter().all(|b| b.is_ascii_graphic() || *b == b' ') {
                let s = String::from_utf8_lossy(bytes).into_owned();
                if s.to_ascii_lowercase().ends_with(".cpp") {
                    return Some(s);
                }
            }
            i = end + 1;
        } else {
            i += 1;
        }
    }
    None
}

fn contains_subslice(hay: &[u8], needle: &[u8]) -> bool {
    if needle.is_empty() || hay.len() < needle.len() {
        return needle.is_empty();
    }
    hay.windows(needle.len()).any(|w| w == needle)
}

/// Read one IL token, which is **2 or 4 bytes depending on the token itself**,
/// returning `(identity, width)`.
///
/// Token width is per-token, not a per-file constant: the second byte carries a
/// continuation flag in bit 7. Clear → the token is those 2 bytes; set → two
/// more bytes follow. Verified on a real capture of `system/world/Dir.cpp`,
/// where `4F 02` module markers appear as both `4f 02 e3 09` (2-byte) and
/// `4f 02 a4 96 03 00` (4-byte) in the same file, and where applying this rule
/// to every `B9` LOAD site lands on a valid 3-byte operand type at 21443 sites
/// (the residue being a third type class plus `B9` bytes occurring inside data).
///
/// [`detect_token_width`] — which returns a single width for the whole file —
/// is therefore wrong for real translation units, and its misalignment is what
/// produced the artifact census buckets `call-token-0x01…0x05` and
/// `expr-load-type-0N00A6`: a 2-byte read of a 4-byte token leaves the parse
/// standing on the token's own tail bytes. It is kept only for the K1/K2a codec
/// and the existing tests; the parser no longer consults it.
///
/// The returned identity is only ever compared for equality (token → parameter
/// register), so any injective encoding will do. The two widths cannot collide:
/// a 2-byte token's value is `< 0x10000` while a 4-byte token's byte 1 has bit 7
/// set, which lands in bits 23..16 of the result and forces it `>= 0x10000`.
fn read_token_var(ex: &[u8], p: usize) -> Option<(u32, usize)> {
    let b0 = *ex.get(p)? as u32;
    let b1 = *ex.get(p + 1)? as u32;
    if b1 & 0x80 == 0 {
        return Some(((b0 << 8) | b1, 2));
    }
    let b2 = *ex.get(p + 2)? as u32;
    let b3 = *ex.get(p + 3)? as u32;
    Some(((b0 << 24) | (b1 << 16) | (b2 << 8) | b3, 4))
}

fn find_byte(hay: &[u8], b: u8) -> Option<usize> {
    hay.iter().position(|&x| x == b)
}

fn find_subslice(hay: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() || hay.len() < needle.len() {
        return None;
    }
    hay.windows(needle.len()).position(|w| w == needle)
}

/// Read a `.ex` inline **type**: `<tag> <kind> <LEB128 id>`, returning
/// `(tag, kind, id, width)`.
///
/// This is a *third* encoding, distinct from both the operand token
/// ([`read_token_var`], 2 or 4 bytes) and the statement/literal varint
/// ([`read_varint`], 1 or 5 bytes). Total width is 3 bytes for an id below
/// `0x80`, 4 below `0x4000`, 5 below `0x200000`.
///
/// Getting this right matters more than it looks: across the 8628 well-formed
/// call sites of one real TU the return-type width splits 4157 / 3123 / 1358
/// between 3, 4 and 5 bytes, so a fixed-3 or even a "3 or 4" rule mis-parses
/// roughly one call in six. The boundaries are pinned by the fixed one-byte
/// markers that always bracket a type — the `41` result-type marker, the `55`
/// argument push, the `4C 4B` call end — against which a wrong width visibly
/// swallows the next marker.
///
/// The tag always has bit 7 set; its high bits are not understood (`0x86`,
/// `0xA6`, `0x96`, `0xC6` all occur and behave identically here), but a decoder
/// does not need them — the width rule is tag-independent. The `kind` byte is
/// treated as a fixed byte rather than a second LEB because `88 85 41`
/// (`double`) and `88 81 13` (`long long`) have bit 7 set there and would
/// otherwise run on.
fn read_type(seg: &[u8], p: usize) -> Option<(u8, u8, u32, usize)> {
    let tag = *seg.get(p)?;
    if tag & 0x80 == 0 {
        return None;
    }
    let kind = *seg.get(p + 1)?;
    let mut id: u32 = 0;
    let mut shift: u32 = 0;
    let mut i = p + 2;
    loop {
        let b = *seg.get(i)?;
        id |= ((b & 0x7F) as u32) << shift;
        i += 1;
        if b & 0x80 == 0 {
            break;
        }
        shift += 7;
        if shift > 28 {
            return None; // malformed / not a type
        }
    }
    Some((tag, kind, id, i - p))
}

/// One recognized whole-body shape of a single `.ex` function segment. Every
/// accepted body is *exactly* one of these — the parser (see [`parse_segment`])
/// is a positive whole-stream parse that reaches the segment's end, so anything
/// it does not model produces `None` and the caller reports `NotImplemented`.
#[derive(Clone, Debug, PartialEq, Eq)]
enum BodyShape {
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
fn blk(seg: &[u8], p: usize, ctx: &'static str) -> Block {
    Block { ctx, byte: seg.get(p).copied(), off: p, aux: 0 }
}

/// Build an operand-*type* [`Block`]: `p` points at the 3-byte inline type that
/// is not the modeled int (`86 41 74`), `report_at` at the operand it belongs
/// to. Packs the triple into [`Block::aux`] so the census buckets by type.
fn blk_type(seg: &[u8], p: usize, report_at: usize, ctx: &'static str) -> Block {
    let g = |i: usize| seg.get(p + i).copied().unwrap_or(0) as u32;
    Block {
        ctx,
        byte: seg.get(p).copied(),
        off: report_at,
        aux: (g(0) << 16) | (g(1) << 8) | g(2),
    }
}

/// Advance `*p` past `pat` iff the stream matches it there; return whether it
/// did. The single primitive the positive parser is built on — every grammar
/// token is consumed through an `eat` (fixed pattern) or a typed read, so an
/// unrecognized byte anywhere fails the whole parse closed.
fn eat(seg: &[u8], p: &mut usize, pat: &[u8]) -> bool {
    if seg.len() >= *p + pat.len() && &seg[*p..*p + pat.len()] == pat {
        *p += pat.len();
        true
    } else {
        false
    }
}

fn eat_byte(seg: &[u8], p: &mut usize, x: u8) -> bool {
    if seg.get(*p) == Some(&x) {
        *p += 1;
        true
    } else {
        false
    }
}

/// Consume an optional `4F 01 NN` statement/label marker (a per-statement
/// sequence index c1xx emits in multi-function TUs, absent in single-function
/// ones). Specific to `4F 01` — it never eats the `4F 12` separator or the
/// `4F 02` module marker.
fn eat_opt_stmt_marker(seg: &[u8], p: &mut usize) {
    if seg.get(*p) == Some(&0x4F) && seg.get(*p + 1) == Some(&0x01) && *p + 2 < seg.len() {
        *p += 3;
    }
}

/// Read an int literal varint `33 <int-type> <varint>` payload (the `33
/// <int-type>` prefix already consumed): a single byte if `< 0x80` (the value),
/// else `0x80` + a 4-byte LE i32. (Verified: 5→`05`, 42→`2a`, 200→`80 c8000000`,
/// 70000→`80 70110100`.) Any other lead byte → `None`.
fn read_varint(seg: &[u8], p: &mut usize) -> Option<i32> {
    let marker = *seg.get(*p)?;
    if marker == 0x80 {
        // Escape: `80` + a 4-byte LE i32. Used for anything that does not fit
        // the signed short form — including −128, whose byte encoding would
        // otherwise collide with this marker.
        //
        // NOTE: for tag-`0x88` types (`long long`) the escape payload is 8
        // bytes, not 4. Those types are rejected upstream, so this reads only
        // the 4-byte form; widening to 64-bit literals must fix this too.
        let v = i32::from_le_bytes([
            *seg.get(*p + 1)?,
            *seg.get(*p + 2)?,
            *seg.get(*p + 3)?,
            *seg.get(*p + 4)?,
        ]);
        *p += 5;
        Some(v)
    } else {
        // Short form: a **signed** byte, not an unsigned one. `-5` is `fb` and
        // `(char)200` is `c8`. An earlier revision accepted only `00..7F` and
        // rejected `81..FF` outright — fail-closed and safe, but it silently
        // blocked every negative literal in the corpus.
        *p += 1;
        Some(marker as i8 as i32)
    }
}

/// Consume the shared statement/function-tail plumbing that follows the body
/// expression of *every* accepted shape, and require the parse to reach the end
/// of the segment (the fail-closed terminal — anything trailing rejects). With
/// `has_result_type`, a `41 <int-type>` result annotation is expected first
/// (present for an int return, absent for a void call). Layout (verified):
/// `[41 <int>]?` result-type · `3A <tok>` assign · `[4F 01 NN]?` · `54 02 29
/// <tok>` return · `4F 12` · `47 54 01 54 00` GT-terminate · then EITHER the
/// segment end (a non-last function, split before the next `4F 1F`) OR the
/// module end `4F 02 20 00 · 4F 01 NN · 4D` and trailing zero-fill (the last
/// function).
fn eat_return_plumbing(seg: &[u8], p: &mut usize, has_result_type: bool) -> Result<(), Block> {
    if has_result_type && !eat(seg, p, &[0x41, INT_TYPE[0], INT_TYPE[1], INT_TYPE[2]]) {
        return Err(blk(seg, *p, "result-type"));
    }
    // ASSIGN: 3A <tok>
    if !eat_byte(seg, p, 0x3A) {
        return Err(blk(seg, *p, "assign"));
    }
    let (_, w) = read_token_var(seg, *p).ok_or(blk(seg, *p, "assign-tok"))?;
    *p += w;
    eat_opt_stmt_marker(seg, p);
    // RETURN: 54 02 29 <tok>
    if !eat(seg, p, &[0x54, 0x02, 0x29]) {
        return Err(blk(seg, *p, "return"));
    }
    let (_, w) = read_token_var(seg, *p).ok_or(blk(seg, *p, "return-tok"))?;
    *p += w;
    // Function-tail: 4F 12 · 47 54 01 54 00
    if !eat(seg, p, &[0x4F, 0x12]) || !eat(seg, p, &[0x47, 0x54, 0x01, 0x54, 0x00]) {
        return Err(blk(seg, *p, "fn-tail"));
    }
    // A non-last function's segment ends exactly here (the split cuts before the
    // next `4F 1F`). Otherwise the last function carries the module end.
    if *p == seg.len() {
        return Ok(());
    }
    if !eat(seg, p, &[0x4F, 0x02, 0x20, 0x00]) || !eat(seg, p, &[0x4F, 0x01]) {
        return Err(blk(seg, *p, "module-end"));
    }
    *p += 1; // module label index NN
    if !eat_byte(seg, p, 0x4D) {
        return Err(blk(seg, *p, "module-end"));
    }
    // Trailing zero-fill to the end of `.ex`.
    while seg.get(*p) == Some(&0) {
        *p += 1;
    }
    if *p == seg.len() {
        Ok(())
    } else {
        Err(blk(seg, *p, "trailing"))
    }
}

/// Consume a postfix LOAD/LIT/ADD/SUB/MUL operand sub-stream, stopping (without
/// consuming) at the `stop` byte that begins the following production. Two call
/// sites, same integer-expression class: the straight-line leaf body stops at
/// the `41` result-type marker (the return plumbing); the call-argument region
/// stops at the `55` call-end marker. Fail-closed: any byte that is not a
/// modeled operand/opcode (a comparison `24`, shift `09`, bitwise `0B`, ternary
/// `43 42`, …) rejects the whole function. Requires at least one op.
///
/// `stop` is only ever tested at a token boundary, so it cannot collide with an
/// int-type byte (`86 41 74` — the `41`/`74` are consumed inside the LOAD/LIT
/// arm) or a literal varint (consumed inside the `33` arm).
fn parse_expr(seg: &[u8], p: &mut usize, stop: u8) -> Result<Vec<IlOp>, Block> {
    let mut ops = Vec::new();
    loop {
        let b = *seg.get(*p).ok_or(blk(seg, *p, "expr"))?;
        if b == stop {
            break;
        }
        match b {
            0xB9 => {
                // LOAD <token> <int-type>
                let start = *p;
                *p += 1;
                let (tok, w) =
                    read_token_var(seg, *p).ok_or(blk(seg, *p, "expr-load-tok"))?;
                *p += w;
                if !eat(seg, p, &INT_TYPE) {
                    // non-int operand → out of class. Report at the LOAD so the
                    // census bucket reads as a typed-operand gap, not a stray byte.
                    return Err(blk_type(seg, *p, start, "expr-load-type"));
                }
                ops.push(IlOp::Load(tok));
            }
            0x33 => {
                // LITERAL: 33 <int-type> <varint>
                let start = *p;
                *p += 1;
                if !eat(seg, p, &INT_TYPE) {
                    return Err(blk_type(seg, *p, start, "expr-lit-type"));
                }
                ops.push(IlOp::Lit(
                    read_varint(seg, p).ok_or(blk(seg, *p, "expr-lit-varint"))?,
                ));
            }
            0x02 => {
                *p += 1;
                ops.push(IlOp::Add);
            }
            0x03 => {
                *p += 1;
                ops.push(IlOp::Sub);
            }
            0x04 => {
                *p += 1;
                ops.push(IlOp::Mul);
            }
            _ => return Err(blk(seg, *p, "expr")),
        }
    }
    if ops.is_empty() {
        Err(blk(seg, *p, "expr-empty"))
    } else {
        Ok(ops)
    }
}

/// Parse the formal-parameter list of a straight-line leaf: after the `46` ('F')
/// marker (before the `LO` marker), a run of `2D <token>` entries emitted in
/// *reverse* of declaration order. An empty list is legitimate (a zero-param
/// `int konst(){return 42;}` still emits `46` immediately before `LO`).
fn parse_formals(seg: &[u8], lo: usize) -> Result<Vec<u32>, Block> {
    let f = find_byte(&seg[..lo], 0x46).ok_or(Block {
        ctx: "formals-marker",
        byte: None,
        off: lo,
        aux: 0,
    })?;
    let mut p = f + 1;
    let mut rev = Vec::new();
    while seg.get(p) == Some(&0x2D) {
        p += 1;
        let (tok, w) = read_token_var(seg, p).ok_or(blk(seg, p, "formals-tok"))?;
        p += w;
        rev.push(tok);
    }
    rev.reverse();
    Ok(rev)
}

/// **The positive whole-body parser (W4b2-v).** Parse a single `.ex` function
/// segment as *exactly one* of the three recognized [`BodyShape`]s, tokenizing
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
///   CALL   := BD <3-byte ret type> 00 80 01 10 00 00      (fixed 10 bytes)
/// ```
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
fn parse_segment(seg: &[u8]) -> Option<BodyShape> {
    parse_segment_detail(seg).ok()
}

/// [`parse_segment`] with the fail-closed *reason* preserved (P2b census).
/// Acceptance is identical — `parse_segment` is `.ok()` of this — so the census
/// can never disagree with the gate about what is in class.
fn parse_segment_detail(seg: &[u8]) -> Result<BodyShape, Block> {
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
        // Call shapes all open with a `26 <tok>` function/result-temp ref.
        0x26 => parse_call_shape(seg, &mut p, lo),
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
            let ops = parse_expr(seg, &mut p, 0x41)?;
            eat_return_plumbing(seg, &mut p, true)?;
            let params = parse_formals(seg, lo)?;
            Ok(BodyShape::StraightLine { params, ops })
        }
        _ => Err(blk(seg, p, "body")),
    }
}

/// The `unsigned int` operand type encoding inline in the `.ex` body.
/// Distinguished from [`INT_TYPE`] only by its last two bytes; the relational
/// opcodes are sign-agnostic, so this triple is the *only* thing that says a
/// comparison is unsigned.
const UINT_TYPE: [u8; 3] = [0x86, 0x42, 0x75];

/// The `float` operand type (`86 45 40`) and the `double` one (`88 85 41`).
/// Note the *literal* forms differ again ([`FLOAT_LIT_TYPE`] /
/// [`DOUBLE_LIT_TYPE`]).
const FLOAT_TYPE: [u8; 3] = [0x86, 0x45, 0x40];
const DOUBLE_TYPE: [u8; 3] = [0x88, 0x85, 0x41];

/// The *literal* FP type tags, which are distinct from the operand ones above.
/// A float literal carries `86 4a 40`, a double one `88 8a 41`.
const FLOAT_LIT_TYPE: [u8; 3] = [0x86, 0x4A, 0x40];
const DOUBLE_LIT_TYPE: [u8; 3] = [0x88, 0x8A, 0x41];

/// Try to parse a **W13a floating-point leaf**: a straight-line chain over
/// float (or double) *parameters* only.
///
/// ```text
///   ( B9 <tok> <FT> | <op> )+     LOADs and binary ops, all of one FP type
///   41 <FT>                       result type, the SAME FP type
///   <return plumbing>
/// ```
///
/// The gate list is from `docs/CODEGEN_W13_FLOAT.md` §6 and every item is a
/// case where a naive selector emits *wrong* bytes rather than merely running
/// out of range:
///
/// * **No literal.** Every FP constant costs an `.rdata` COMDAT, a REFHI/REFLO
///   relocation pair and a GPR — that is W13b.
/// * **No `2C` convert**, and no mixing of float with double: a mixed-width
///   expression evaluates in double and may need an `frsp`.
/// * **No `*` under `+`/`-`.** Contraction to `fmadds`/`fmsubs`/`fnmsubs` is
///   *mandatory* in c2, so emitting the two separate instructions would be a
///   silent mis-emit. Approximated conservatively here by rejecting any chain
///   that contains both a `Mul` and an `Add`/`Sub`.
/// * **No repeated leaf.** `a + a` is algebraically rewritten to `a * 2.0f`,
///   which is a constant and therefore `.rdata` again.
/// * **No `0x59` marker.** It tracks source parenthesisation and is the only
///   thing distinguishing product shapes c2 flattens from those it does not;
///   its meaning is unknown, so its presence rejects.
fn try_parse_float_leaf(seg: &[u8], start: usize, lo: usize) -> Option<BodyShape> {
    let mut p = start;
    // The operand type is fixed by the first LOAD and every later one must match.
    if *seg.get(p)? != 0xB9 {
        return None;
    }
    let double = {
        let mut probe = p + 1;
        let (_, w) = read_token_var(seg, probe)?;
        probe += w;
        if seg.get(probe..probe + 3)? == FLOAT_TYPE {
            false
        } else if seg.get(probe..probe + 3)? == DOUBLE_TYPE {
            true
        } else {
            return None;
        }
    };
    let fty = if double { DOUBLE_TYPE } else { FLOAT_TYPE };

    let mut ops: Vec<IlOp> = Vec::new();
    loop {
        match *seg.get(p)? {
            0xB9 => {
                p += 1;
                let (tok, w) = read_token_var(seg, p)?;
                p += w;
                if seg.get(p..p + 3)? != fty {
                    return None; // mixed width, or a non-FP operand
                }
                p += 3;
                ops.push(IlOp::Load(tok));
            }
            0x02 => {
                p += 1;
                ops.push(IlOp::Add);
            }
            0x03 => {
                p += 1;
                ops.push(IlOp::Sub);
            }
            0x04 => {
                p += 1;
                ops.push(IlOp::Mul);
            }
            0x05 => {
                p += 1;
                ops.push(IlOp::Div);
            }
            // W13b: a floating-point literal.
            //
            //   33 <lit-TYPE> <8 bytes: IEEE binary64, little-endian> <u16 size>
            //
            // The payload is a binary64 pattern even for a `float` (already
            // rounded to binary32 precision), and the u16 trailer is the operand
            // *width* — 4 for float, 8 for double — which must agree with the
            // literal tag. Verified byte-for-byte against a live capture of
            // `float k_add(float a){return a + 1.0f;}`:
            //   33 86 4a 40 00 00 00 00 00 00 f0 3f 04 00
            0x33 => {
                p += 1;
                let lty = seg.get(p..p + 3)?;
                let lit_double = if lty == FLOAT_LIT_TYPE {
                    false
                } else if lty == DOUBLE_LIT_TYPE {
                    true
                } else {
                    return None; // an integer (or other) literal: out of class
                };
                // A literal of the other width implies a conversion.
                if lit_double != double {
                    return None;
                }
                p += 3;
                let raw: [u8; 8] = seg.get(p..p + 8)?.try_into().ok()?;
                p += 8;
                let size = u16::from_le_bytes(seg.get(p..p + 2)?.try_into().ok()?);
                p += 2;
                if size as usize != if double { 8 } else { 4 } {
                    return None;
                }
                ops.push(IlOp::FpLit {
                    bits: u64::from_le_bytes(raw),
                    double,
                });
            }
            0x41 => break,
            // 0x2C convert, 0x59 paren marker, 0x08 neg and every other byte
            // reject — see the gate list above.
            _ => return None,
        }
    }
    // Result type must be the same FP type.
    p += 1;
    if seg.get(p..p + 3)? != fty {
        return None;
    }
    p += 3;
    eat_return_plumbing(seg, &mut p, false).ok()?;

    // A `*` mixed with `+`/`-` contracts; reject rather than emit two
    // instructions where c2 emits one.
    let has_mul = ops.iter().any(|o| matches!(o, IlOp::Mul));
    let has_addsub = ops.iter().any(|o| matches!(o, IlOp::Add | IlOp::Sub));
    if has_mul && has_addsub {
        return None;
    }

    // ---- W13b constant gates ------------------------------------------------
    //
    // These live here, in the parser, rather than in codegen so that the census
    // and the emission gate cannot disagree about what is in class.
    //
    // c2 — not c1xx — evaluates floating-point constants, so the IL still holds
    // every literal the source wrote and the backend is free to fold, reassociate
    // and strength-reduce them. Three captured behaviours the port does not
    // model, each of which would be a silent mis-emit:
    let lits: Vec<(u64, bool)> = ops
        .iter()
        .filter_map(|o| match o {
            IlOp::FpLit { bits, double } => Some((*bits, *double)),
            _ => None,
        })
        .collect();
    if !lits.is_empty() {
        // (1) Two or more literals: c2 folds them where it can (`a*2.0f*b*3.0f`
        //     becomes `(a*b)*6.0f`), and where it cannot it hoists every `addis`
        //     into a prologue group and schedules the loads at first use. Either
        //     way the one-constant lowering is wrong. See `w13b_fpool.cpp`.
        if lits.len() > 1 {
            return None;
        }
        // (2) A constant divisor becomes a reciprocal multiply: `a/2.0f` emits
        //     `fmuls` against `__real@3f000000`, and `a/3.0f/7.0f` collapses to
        //     one `fmuls` by 1/21 — a value that is not even exactly
        //     representable, so this is a numeric transform, not a rewrite.
        if ops.iter().any(|o| matches!(o, IlOp::Div)) {
            return None;
        }
        let (bits, lit_double) = lits[0];
        let v = f64::from_bits(bits);
        // (3) An identity operand disappears entirely — `a + 0.0f`, `a - 0.0f`
        //     and `a * 1.0f` each compile to a bare `blr`, with no constant
        //     pooled at all. (`a * 0.0f` is *not* folded: it really does load
        //     zero and multiply.) Refuse when the literal is an identity for any
        //     operator in the body; slight over-refusal beats emitting three
        //     instructions where c2 emits none.
        if v == 0.0 && has_addsub {
            return None;
        }
        if v == 1.0 && has_mul {
            return None;
        }
        // (4) A `float` literal is carried as a binary64 pattern already rounded
        //     to binary32. If it does not narrow exactly, the four bytes we would
        //     pool are not the ones c2 pooled.
        if !lit_double && f64::from(v as f32).to_bits() != bits {
            return None;
        }
    }
    // A repeated leaf can trigger algebraic rewriting into a constant.
    let mut seen: Vec<u32> = Vec::new();
    for o in &ops {
        if let IlOp::Load(t) = o {
            if seen.contains(t) {
                return None;
            }
            seen.push(*t);
        }
    }
    let params = parse_formals(seg, lo).ok()?;
    if params.len() > 13 || !seen.iter().all(|t| params.contains(t)) {
        return None;
    }
    Some(BodyShape::FloatLeaf { params, ops, double })
}

/// Try to parse a **W6 comparison leaf** body: `return <formal> <rel> <k>;`.
///
/// ```text
///   B9 <tok> <T>        LOAD the formal          T ∈ {int, unsigned}
///   33 <T> <varint>     LITERAL k, same type T
///   <rel>               1F|20|21|22|23|24
///   2C <R> 00           convert bool → R         R ∈ {int, unsigned}
///   41 <R>              result type
///   <return plumbing>
/// ```
///
/// Fail-closed specifics that are load-bearing rather than incidental:
///
/// * The two operand types must be **equal**. c1xx always inserts a conversion
///   first, so a mismatch has never been observed; rejecting it is a cheap
///   assertion, not a dropped feature.
/// * The `2C` convert is accepted **only here**, directly over a comparison
///   result. The identical token over a narrow-integer LOAD is a real
///   `extsb`/`extsh` sign-extension, so a blanket "`2C` is free" rule would
///   silently drop those instructions.
/// * The parse must reach the segment end via the shared return plumbing, so a
///   trailing statement, a second comparison, or an arithmetic post-op (e.g.
///   `return (a > 7) + 1;`, which retargets the spine's last instruction) all
///   reject the whole function.
///
/// Returns `None` — leaving the caller's cursor untouched — for anything that is
/// not exactly this shape.
fn try_parse_compare(seg: &[u8], start: usize, lo: usize) -> Option<BodyShape> {
    let mut p = start;

    // LOAD <formal> <T>
    if !eat_byte(seg, &mut p, 0xB9) {
        return None;
    }
    let (param, w) = read_token_var(seg, p)?;
    p += w;
    let signed = if eat(seg, &mut p, &INT_TYPE) {
        true
    } else if eat(seg, &mut p, &UINT_TYPE) {
        false
    } else {
        return None;
    };
    let operand_type = if signed { INT_TYPE } else { UINT_TYPE };

    // LITERAL k, of the SAME type as the loaded operand.
    if !eat_byte(seg, &mut p, 0x33) || !eat(seg, &mut p, &operand_type) {
        return None;
    }
    let k = read_varint(seg, &mut p)?;

    // The relational opcode.
    let rel = Rel::from_opcode(*seg.get(p)?)?;
    p += 1;

    // `2C <R> 00` — convert the bool result to the return type.
    if !eat_byte(seg, &mut p, 0x2C) {
        return None;
    }
    let ret_is_int = if eat(seg, &mut p, &INT_TYPE) {
        true
    } else if eat(seg, &mut p, &UINT_TYPE) {
        false
    } else {
        return None;
    };
    if !eat_byte(seg, &mut p, 0x00) {
        return None;
    }

    // Result type + the shared return plumbing, which must reach the segment end.
    let ret_type = if ret_is_int { INT_TYPE } else { UINT_TYPE };
    if !eat_byte(seg, &mut p, 0x41) || !eat(seg, &mut p, &ret_type) {
        return None;
    }
    // Result type already consumed above, so `has_result_type` is false here.
    eat_return_plumbing(seg, &mut p, false).ok()?;

    // The compared value must be the function's FIRST formal: the spine reads it
    // from r3, and nothing here models a register move.
    let params = parse_formals(seg, lo).ok()?;
    if params.first() != Some(&param) || params.len() != 1 {
        return None;
    }

    Some(BodyShape::Compare(CompareLeaf { param, rel, signed, k }))
}

/// Parse a call shape (already positioned at the `26 <tok>` function ref): the
/// bare terminal void call, an integer tail call `return g(<arg>)` (passthrough
/// or arg-setup, plus the `g(a)+0` identity fold), or the framed
/// `return g(a) + k` (k ≠ 0). See [`parse_segment`] for the grammar;
/// fail-closed at every step. `lo` locates the formals for the arg-setup.
fn parse_call_shape(seg: &[u8], p: &mut usize, lo: usize) -> Result<BodyShape, Block> {
    // 26 <tok> function/result ref.
    if !eat_byte(seg, p, 0x26) {
        return Err(blk(seg, *p, "call-ref"));
    }
    // The `26 <tok>` symbol push NAMES THE CALLEE. The CALL token that follows
    // carries only a function-*type* id, so this token is the only thing that
    // distinguishes one callee from another; it is resolved through the `.gl`
    // symbol index (see `gl_symbol_index`).
    let (callee_tok, w) = read_token_var(seg, *p).ok_or(blk(seg, *p, "call-ref-tok"))?;
    *p += w;
    // The CALL token: `BD <TYPE ret> <flags> <varint fn-type-id>`. Nothing in it
    // is fixed but the `BD` — it is 8 to 13 bytes and self-delimiting field by
    // field, so it is decoded rather than matched.
    //
    // This replaces a hardcoded 6-byte "callee anchor" `00 80 01 10 00 00`,
    // which was never an anchor: it is `flags = 0` followed by the varint
    // `0x1001`, and `0x1001` is merely the first function type a single-function
    // fixture TU happens to create. True of every MVP fixture and of almost
    // nothing else — which is precisely what the `call-anchor-*` census buckets
    // were measuring.
    if !eat_byte(seg, p, 0xBD) {
        return Err(blk(seg, *p, "call-token"));
    }
    let (_, _, _, ret_w) = read_type(seg, *p).ok_or(blk(seg, *p, "call-ret-type"))?;
    *p += ret_w;
    // Calling convention: 0x00 = cdecl/stdcall, 0x04 = fastcall, 0x40 = varargs.
    // Only cdecl is in class — the others need argument-passing the port does
    // not implement, and accepting them would mis-emit rather than refuse.
    match seg.get(*p) {
        Some(0x00) => *p += 1,
        _ => return Err(blk(seg, *p, "call-conv")),
    }
    // The function-type id. NOT the callee: three different callees sharing one
    // signature produce byte-identical CALL tokens. The callee is bound from the
    // `26 <tok>` symbol push instead, so this field is decoded only to find the
    // token's end, then discarded.
    read_varint(seg, p).ok_or(blk(seg, *p, "call-fn-type-id"))?;

    // VOID terminal tail call: the `4C 4B` void call-end immediately follows the
    // CALL token (no argument setup, no consumed value), then only return
    // plumbing (no result type). `g();g();` and `g();return a+1;` fail here — a
    // second `26` call or a `B9` statement stands where the return plumbing must.
    if eat(seg, p, &[0x4C, 0x4B]) {
        eat_return_plumbing(seg, p, false)?;
        return Ok(BodyShape::VoidTailCall { callee_tok });
    }

    // INT call. The argument region is a single modeled sub-expression producing
    // the one call argument — a passthrough `B9 a INT` (→ `[Load]`) or an
    // arg-setup like `a + 1` (→ `[Load, Lit, Add]`) — terminated by the `55`
    // call-end. Any unmodeled operand/opcode rejects the whole function. `g(a)`,
    // `g(a)+k` and `g(a+1)` share this region; they diverge at the post-op.
    let arg_ops = parse_expr(seg, p, 0x55)?;
    if !eat_byte(seg, p, 0x55) || !eat(seg, p, &INT_TYPE) || !eat_byte(seg, p, 0x4C) {
        // a call-argument region whose call-end we do not model
        return Err(blk(seg, *p, "call-end"));
    }

    // Post-op region. EITHER the return plumbing begins directly at its `41`
    // result-type marker (no post-op → an integer tail call `return g(<arg>)`),
    // OR exactly one literal `33 <int> k` + ADD (`return g(a) + k`, framed).
    if seg.get(*p) == Some(&0x41) {
        // No post-op → integer tail call: compute the argument into r3, then
        // `b <callee>` (5-section leaf). The int analog of the void tail call;
        // `g(a)` is a bare `b g`, `g(a+1)` prepends `addi r3,r3,1`.
        eat_return_plumbing(seg, p, true)?;
        let params = parse_formals(seg, lo)?;
        return Ok(BodyShape::IntTailCall { params, arg_ops, callee_tok });
    }
    // Post-op `+ k`: EXACTLY one literal `33 <int> k` immediately followed by
    // ADD. A second call (`g(a)+g(1)` → `26 …`), a second literal (`g(a)+1+2` →
    // a second `33 …`), or SUB/MUL (`03`/`04`) all fail one of these `eat`s.
    if !eat_byte(seg, p, 0x33) || !eat(seg, p, &INT_TYPE) {
        return Err(blk(seg, *p, "call-postop"));
    }
    let k = read_varint(seg, p).ok_or(blk(seg, *p, "call-postop-varint"))?;
    if !eat_byte(seg, p, 0x02) {
        // non-ADD post-op → non-commutative / strength-reduced
        return Err(blk(seg, *p, "call-postop-op"));
    }
    // `k` must fit a single signed-16-bit `addi` immediate (the 0x24 frame).
    if !(-0x8000..=0x7FFF).contains(&k) {
        return Err(Block { ctx: "call-postop-wide", byte: None, off: *p, aux: 0 });
    }
    eat_return_plumbing(seg, p, true)?;

    // W4b2-vi identity fold: a net post-op of 0 is NOT a framed call. `g(a)+0`
    // == `g(a)`, and the optimizer folds it to the bare `b g` (verified: the
    // `g(a)+0` obj is byte-identical to `g(a)`'s). Route it to the integer
    // tail-call production so it takes the 5-section leaf path — never the
    // 6-section framed obj (which would mis-emit a frame the reference elides).
    if k == 0 {
        let params = parse_formals(seg, lo)?;
        return Ok(BodyShape::IntTailCall { params, arg_ops, callee_tok });
    }
    // A genuine `+ k` (k ≠ 0) is a framed non-leaf call — but the 6-section
    // framed path models only a **bare passthrough argument** (`g(a) + k`), not
    // arg-setup. `g(a+1) + 1` (a computed argument AND a framed post-op) is out
    // of class → reject (fail closed), never a mis-emitted framed obj.
    if matches!(arg_ops.as_slice(), [IlOp::Load(_)]) {
        return Ok(BodyShape::FramedCall { add_k: k, callee_tok });
    }
    Err(Block { ctx: "framed-computed-arg", byte: None, off: *p, aux: 0 })
}

/// The `.ex` per-function start marker (`4F 1F`). The module stream is a
/// sequence of function bodies, each introduced by this marker; the header /
/// index region before the first one is opaque zero-fill for this class.
const FN_START: [u8; 2] = [0x4F, 0x1F];

/// Split the `.ex` stream into per-function byte segments at each `4F 1F`
/// function-start marker. Segment `k` runs from marker `k` to marker `k+1`
/// (the last to end-of-stream).
/// One function's census verdict (P2b). Either the modeled shape it parsed as,
/// or the first feature that blocked it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FnVerdict {
    /// Parsed as a modeled shape. The string is a stable shape label
    /// (`straight-line`, `void-tail-call`, `int-tail-call`, `framed-call`).
    InClass(&'static str),
    /// Blocked at the first unmodeled feature.
    Blocked(Block),
}

impl FnVerdict {
    /// The census bucket key: the shape label when in class, else the blocking
    /// feature (see [`Block::feature`]).
    pub fn key(&self) -> String {
        match self {
            FnVerdict::InClass(s) => (*s).to_string(),
            FnVerdict::Blocked(b) => b.feature(),
        }
    }
    pub fn in_class(&self) -> bool {
        matches!(self, FnVerdict::InClass(_))
    }
}

/// One census row: a function segment and how it classified.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FnCensus {
    /// Index of the function within the TU (`.ex` segment order).
    pub index: usize,
    /// Mangled name, when `.gl` has one at this position.
    pub name: Option<String>,
    /// Segment length in bytes (a rough proxy for function size).
    pub seg_len: usize,
    pub verdict: FnVerdict,
    /// Raw bytes around the blocking site, for grammar work: the segment window
    /// `[off - CENSUS_HEX_BACK, off + CENSUS_HEX_FWD)` clamped to the segment,
    /// plus the index of the blocking byte within that window. Empty when the
    /// function is in class.
    pub hex: Vec<u8>,
    /// Index of the blocking byte inside [`FnCensus::hex`].
    pub hex_mark: usize,
}

/// Bytes of context kept before / after a blocking site.
pub const CENSUS_HEX_BACK: usize = 16;
pub const CENSUS_HEX_FWD: usize = 24;

/// The `.ex` body marker `4C 4F 11` (`LO`) that opens every function body.
const LO_MARKER: [u8; 3] = [0x4C, 0x4F, 0x11];

/// Split `.ex` into one segment per **function body**, anchored on the `LO`
/// marker rather than the `4F 1F` function-start marker (P2b).
///
/// `4F 1F` is only two bytes and also occurs inside token and varint payloads,
/// so a raw marker scan over a real translation unit over-counts: measured on
/// `system/world/Dir.cpp` (1.5 MB `.ex`), 5340 `4F 1F` against 5239 `LO` body
/// markers and 5243 function tails (`4F 12 47 54 01 54 00`) — the latter two
/// agree to 0.08%, the first is ~2% high. Anchoring on `LO` keeps the count
/// honest without inventing a denominator.
///
/// Each segment starts at the `4F 1F` immediately preceding its `LO` (so the
/// formals region stays inside the segment, where [`parse_formals`] looks for
/// it) and runs to the next segment's start. Two bodies sharing one preceding
/// `4F 1F` would collide; the later one then starts at its own `LO`, which
/// simply blocks it at `formals-marker` — an honest miss, never a merge that
/// would silently drop a function from the denominator.
fn split_function_bodies(ex: &[u8]) -> Vec<&[u8]> {
    // Body markers, in file order.
    let mut los: Vec<usize> = Vec::new();
    let mut i = 0;
    while i + 3 <= ex.len() {
        if ex[i] == LO_MARKER[0] && ex[i + 1] == LO_MARKER[1] && ex[i + 2] == LO_MARKER[2] {
            los.push(i);
            i += 3;
        } else {
            i += 1;
        }
    }
    if los.is_empty() {
        return Vec::new();
    }
    // Function-start markers, in file order, for the "nearest preceding" lookup.
    let mut starts: Vec<usize> = Vec::new();
    let mut i = 0;
    while i + 2 <= ex.len() {
        if ex[i] == FN_START[0] && ex[i + 1] == FN_START[1] {
            starts.push(i);
            i += 2;
        } else {
            i += 1;
        }
    }

    let mut segs_start: Vec<usize> = Vec::with_capacity(los.len());
    for &lo in &los {
        // Greatest `4F 1F` offset strictly below this body marker.
        let cand = match starts.partition_point(|&s| s < lo) {
            0 => lo,
            k => starts[k - 1],
        };
        // Never reuse a start (would merge two bodies into one segment).
        let cand = if segs_start.last() == Some(&cand) { lo } else { cand };
        segs_start.push(cand);
    }
    (0..segs_start.len())
        .map(|k| {
            let start = segs_start[k];
            let end = segs_start.get(k + 1).copied().unwrap_or(ex.len());
            &ex[start..end.max(start)]
        })
        .collect()
}

/// True iff `.ex` positively declares a module with **no function bodies**
/// (R1): it carries neither a body marker (`4C 4F 11`) nor a function-start
/// marker (`4F 1F`).
///
/// Both signals are required. `4F 1F` alone is two bytes and collides inside
/// payloads (so its *absence* is meaningful but its presence is not), while
/// `LO` is the marker every real body opens with — on a 1.5 MB real `.ex` the
/// `LO` count tracked the function-tail count to 0.08%. A capture with zero of
/// each has nothing that could be a function.
///
/// Verified against the live toolchain: a TU containing only a typedef captures
/// a 2691-byte `.ex` that is entirely zero-fill apart from a 4-byte head and a
/// 46-byte module-metadata tail, with 0 `LO` and 0 `4F 1F`, and c2 emits a
/// 720-byte four-section obj for it.
pub fn is_empty_module(ex: &[u8]) -> bool {
    let has_lo = ex
        .windows(3)
        .any(|w| w == [LO_MARKER[0], LO_MARKER[1], LO_MARKER[2]]);
    let has_fn_start = ex.windows(2).any(|w| w == FN_START);
    !has_lo && !has_fn_start
}

fn split_functions(ex: &[u8]) -> Vec<&[u8]> {
    let mut starts = Vec::new();
    let mut i = 0;
    while i + 1 < ex.len() {
        if ex[i] == FN_START[0] && ex[i + 1] == FN_START[1] {
            starts.push(i);
            i += 2;
        } else {
            i += 1;
        }
    }
    let mut segs = Vec::with_capacity(starts.len());
    for k in 0..starts.len() {
        let end = if k + 1 < starts.len() { starts[k + 1] } else { ex.len() };
        segs.push(&ex[starts[k]..end]);
    }
    segs
}

impl IlBundle {
    /// **Function-level census (P2b).** Classify *every* function in the bundle
    /// independently, so a TU whose 700th function uses an unmodeled opcode
    /// still reports the other 699 as in-class.
    ///
    /// This is the measurement [`IlBundle::functions`] cannot give: that method
    /// is all-or-nothing per TU (correctly — the port must emit a whole obj or
    /// nothing), so over a real workload it reports one `vocab-gap` per TU and
    /// cannot rank the missing classes. The census runs the *same*
    /// [`parse_segment_detail`] per segment and keeps the first blocking
    /// feature, so the histogram of [`FnVerdict::key`] over a corpus is the
    /// widening order (docs/ROADMAP.md §G5).
    ///
    /// Diagnostic only — never a gate, and never consulted by the emitter.
    /// Returns `None` only when the bundle lacks the required files.
    pub fn function_census(&self) -> Option<Vec<FnCensus>> {
        let gl = self.get("gl")?;
        let ex = self.ex()?;
        let names = mangled_names(gl);
        let segs = split_function_bodies(ex);
        // Names are paired positionally, which is only meaningful when `.gl`
        // yields exactly one name per body. On a real TU `mangled_names` finds
        // far fewer (it accepts only `?…@@…` forms, and `.gl` also lists
        // externals), so pairing there would attach wrong names to functions —
        // report none rather than a plausible-looking lie.
        let paired = names.len() == segs.len();
        Some(
            segs.iter()
                .enumerate()
                .map(|(i, seg)| {
                    let verdict = match parse_segment_detail(seg) {
                        Ok(BodyShape::StraightLine { .. }) => FnVerdict::InClass("straight-line"),
                        Ok(BodyShape::VoidTailCall { .. }) => FnVerdict::InClass("void-tail-call"),
                        Ok(BodyShape::IntTailCall { .. }) => FnVerdict::InClass("int-tail-call"),
                        Ok(BodyShape::FramedCall { .. }) => FnVerdict::InClass("framed-call"),
                        Ok(BodyShape::Compare(_)) => FnVerdict::InClass("compare-leaf"),
                        Ok(BodyShape::EmptyBody) => FnVerdict::InClass("empty-body"),
                        Ok(BodyShape::FloatLeaf { double, .. }) => {
                            FnVerdict::InClass(if double { "double-leaf" } else { "float-leaf" })
                        }
                        Err(b) => FnVerdict::Blocked(b),
                    };
                    // Keep the raw bytes around the blocking site: decoding a new
                    // grammar production always starts by staring at exactly this
                    // window, and having it in the census means that work is a
                    // report away instead of a one-off script.
                    let (hex, hex_mark) = match &verdict {
                        FnVerdict::InClass(_) => (Vec::new(), 0),
                        FnVerdict::Blocked(b) => {
                            let start = b.off.saturating_sub(CENSUS_HEX_BACK);
                            let end = (b.off + CENSUS_HEX_FWD).min(seg.len());
                            let start = start.min(end);
                            (seg[start..end].to_vec(), b.off - start)
                        }
                    };
                    FnCensus {
                        index: i,
                        name: if paired { names.get(i).cloned() } else { None },
                        seg_len: seg.len(),
                        verdict,
                        hex,
                        hex_mark,
                    }
                })
                .collect(),
        )
    }

    /// Parse this bundle as a sequence of straight-line add-chain functions
    /// (the MVP class, generalized to a multi-function TU). Returns `None` if
    /// the required files are absent, or if the `.gl` name count does not match
    /// the `.ex` function count, or if ANY function body is outside the class
    /// (the caller — `PortC2` — then reports `NotImplemented` for the whole TU).
    ///
    /// Names come from `.gl` in file order; bodies from `.ex` split at each
    /// `4F 1F`. The two are paired positionally.
    pub fn functions(&self) -> Option<Vec<IlFunction>> {
        let gl = self.get("gl")?;
        let ex = self.ex()?;
        let names = mangled_names(gl);
        let src = source_path(gl);

        // R1: a TU that defines no functions is in class, and its obj is the
        // fixed four-section shell with no `.text`. Recognized **positively**
        // (no body markers AND no function-start markers), never as "the split
        // returned nothing" — the latter would also fire on a bundle we merely
        // failed to split, and emitting an empty obj for a TU that really has
        // code is precisely the mis-emit the fail-closed rule forbids.
        if is_empty_module(ex) {
            return Some(Vec::new());
        }

        let segs = split_functions(ex);
        if segs.is_empty() || names.len() < segs.len() {
            return None;
        }
        // `.gl` lists the defined functions first, one per `.ex` segment, paired
        // positionally — that is where a *defined* function's own name comes
        // from. Callee names do NOT come from that list: they are resolved by
        // token through the `.gl` symbol index, because the CALL token carries
        // only a function-type id and cannot distinguish two callees with the
        // same signature. Resolving properly is what lifts the old
        // single-function/single-external restriction that positional pairing
        // forced.
        let n_defined = segs.len();
        let symbols = gl_symbol_index(gl);
        let resolve = |tok: u32| -> Option<String> { symbols.get(&tok).cloned() };

        let mut funcs = Vec::with_capacity(n_defined);
        for (name, seg) in names.iter().take(n_defined).zip(segs) {
            match parse_segment(seg)? {
                BodyShape::StraightLine { params, ops } => {
                    funcs.push(IlFunction {
                        mangled_name: name.clone(),
                        source_path: src.clone(),
                        params,
                        ops,
                        tail_call: None,
                        framed_call: None,
                        compare: None,
                        empty_body: false,
                        float_leaf: None,
                    });
                }
                // Tail calls: the callee is resolved BY TOKEN through the `.gl`
                // symbol index. An unresolvable token rejects the whole TU
                // rather than falling back to a positional guess — a wrong
                // callee name is a relocation against the wrong symbol, which is
                // a mis-emit, not a gap.
                BodyShape::VoidTailCall { callee_tok } => {
                    funcs.push(IlFunction {
                        mangled_name: name.clone(),
                        source_path: src.clone(),
                        params: Vec::new(),
                        ops: Vec::new(),
                        tail_call: Some(resolve(callee_tok)?),
                        framed_call: None,
                        compare: None,
                        empty_body: false,
                        float_leaf: None,
                    });
                }
                BodyShape::IntTailCall { params, arg_ops, callee_tok } => {
                    funcs.push(IlFunction {
                        mangled_name: name.clone(),
                        source_path: src.clone(),
                        params,
                        ops: arg_ops,
                        tail_call: Some(resolve(callee_tok)?),
                        framed_call: None,
                        compare: None,
                        empty_body: false,
                        float_leaf: None,
                    });
                }
                // The framed non-leaf path stays SINGLE-FUNCTION. Its obj carries
                // `.pdata` with compiler label symbols ($M2545/$M2546/$T2547)
                // whose counters are a fixed toolchain seed for the first
                // function and shift once preceding functions consume slots
                // (W-UNW-1, docs/CODEGEN_PPC_MVP.md), so a multi-function TU
                // containing one would be mis-numbered.
                BodyShape::FramedCall { add_k, callee_tok } => {
                    if n_defined != 1 {
                        return None;
                    }
                    funcs.push(IlFunction {
                        mangled_name: name.clone(),
                        source_path: src.clone(),
                        params: Vec::new(),
                        ops: Vec::new(),
                        tail_call: None,
                        framed_call: Some(FramedCall {
                            callee: resolve(callee_tok)?,
                            add_k,
                        }),
                        compare: None,
                        empty_body: false,
                        float_leaf: None,
                    });
                }
                // W6: a comparison leaf carries no op stream — codegen emits its
                // spine from the decoded relation instead.
                BodyShape::EmptyBody => {
                    funcs.push(IlFunction {
                        mangled_name: name.clone(),
                        source_path: src.clone(),
                        params: Vec::new(),
                        ops: Vec::new(),
                        tail_call: None,
                        framed_call: None,
                        compare: None,
                        empty_body: true,
                        float_leaf: None,
                    });
                }
                BodyShape::FloatLeaf { params, ops, double } => {
                    funcs.push(IlFunction {
                        mangled_name: name.clone(),
                        source_path: src.clone(),
                        params,
                        ops,
                        tail_call: None,
                        framed_call: None,
                        compare: None,
                        empty_body: false,
                        float_leaf: Some(double),
                    });
                }
                BodyShape::Compare(cmp) => {
                    funcs.push(IlFunction {
                        mangled_name: name.clone(),
                        source_path: src.clone(),
                        params: vec![cmp.param],
                        ops: Vec::new(),
                        tail_call: None,
                        framed_call: None,
                        compare: Some(cmp),
                        empty_body: false,
                        float_leaf: None,
                    });
                }
            }
        }
        Some(funcs)
    }

    /// Parse this bundle as a SINGLE MVP function. Convenience wrapper over
    /// [`IlBundle::functions`]; returns `None` unless the TU has exactly one
    /// in-class function.
    pub fn mvp_function(&self) -> Option<IlFunction> {
        let mut fs = self.functions()?;
        if fs.len() == 1 {
            fs.pop()
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mangled_name_from_gl_slice() {
        let gl = b"\x00\x00?add3@@YAHHHH@Z\x00trailing";
        assert_eq!(mangled_name(gl).as_deref(), Some("?add3@@YAHHHH@Z"));
    }

    #[test]
    fn mangled_name_rejects_stray_question_mark() {
        // `?` not followed by an alpha / no `@@`.
        assert_eq!(mangled_name(b"? not a name\x00"), None);
    }

    #[test]
    fn source_path_from_gl_slice() {
        let gl = b"\x12\x20\x00z:\\tmp\\ilcap\\mvp.cpp\x00\x10";
        assert_eq!(
            source_path(gl).as_deref(),
            Some("z:\\tmp\\ilcap\\mvp.cpp")
        );
    }

    #[test]
    fn token_width_two_from_4f02_gap() {
        // `4F 02 20 00 4F` → gap 2.
        let ex = [0x00, 0x4F, 0x02, 0x20, 0x00, 0x4F, 0x01];
        assert_eq!(detect_token_width(&ex), 2);
    }

    #[test]
    fn mangled_names_collects_all_in_order() {
        let gl = b"\x00?add2@@YAHHH@Z\x00pad\x00?add4@@YAHHHHH@Z\x00";
        assert_eq!(
            mangled_names(gl),
            vec!["?add2@@YAHHH@Z".to_string(), "?add4@@YAHHHHH@Z".to_string()]
        );
    }

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
            parse_segment(seg),
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
            parse_segment(konst),
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
            parse_segment(kw),
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
            parse_segment(seg),
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
            parse_segment(MVP_CALL),
            Some(BodyShape::VoidTailCall { callee_tok: 0xE309 })
        );
    }

    #[test]
    fn parse_segment_accepts_framed_call() {
        // `int f(int a){ return g(a) + 1; }` (mvp_framed): int call, single
        // passthrough arg, `55` call-end, exactly one `+1` post-op.
        assert_eq!(
            parse_segment(MVP_FRAMED),
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
            parse_segment(INT_TAILRET),
            Some(BodyShape::IntTailCall {
                params: vec![0xE509],
                arg_ops: vec![IlOp::Load(0xE509)],
                callee_tok: 0xE409,
            }),
            "passthrough g(a)"
        );
        assert_eq!(
            parse_segment(INT_PLUS0),
            Some(BodyShape::IntTailCall {
                params: vec![0xE509],
                arg_ops: vec![IlOp::Load(0xE509)],
                callee_tok: 0xE409,
            }),
            "identity-fold g(a)+0 routes to a tail call, not FramedCall{{add_k:0}}"
        );
        assert_eq!(
            parse_segment(INT_ARGTAIL),
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
            parse_segment(MVP_FRAMED),
            Some(BodyShape::FramedCall { add_k: 1, callee_tok: 0xE409 }),
            "g(a)+1 is framed"
        );
        assert!(
            matches!(parse_segment(INT_PLUS0), Some(BodyShape::IntTailCall { .. })),
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
            assert_eq!(parse_segment(seg), None, "must reject: {label}");
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
        assert_eq!(parse_segment(cmp), None);
    }

    // ---- variable-width tokens ----------------------------------------------

    #[test]
    fn token_is_two_bytes_when_the_continuation_bit_is_clear() {
        // Every fixture token is of this form (`e3 09`, `e5 09`, …) — bit 7 of
        // the second byte clear.
        assert_eq!(read_token_var(&[0xE3, 0x09, 0xFF], 0), Some((0xE309, 2)));
        assert_eq!(read_token_var(&[0x00, 0x7F], 0), Some((0x007F, 2)));
    }

    #[test]
    fn token_is_four_bytes_when_the_continuation_bit_is_set() {
        // Real-TU form, e.g. the module marker payload `a4 96 03 00`.
        assert_eq!(
            read_token_var(&[0xA4, 0x96, 0x03, 0x00], 0),
            Some((0xA496_0300, 4))
        );
        // Truncated 4-byte token → None, never a short read.
        assert_eq!(read_token_var(&[0xA4, 0x96, 0x03], 0), None);
    }

    #[test]
    fn token_identities_cannot_collide_across_widths() {
        // The parser compares token identities for equality (token → parameter
        // register), so a 2-byte and a 4-byte token must never produce the same
        // value. A 4-byte token's byte 1 has bit 7 set, which lands in bits
        // 23..16 and forces the value >= 0x10000; 2-byte values are < 0x10000.
        for b0 in [0x00u8, 0x7F, 0x80, 0xFF] {
            for b1 in [0x00u8, 0x7F, 0x80, 0xFF] {
                let (v, w) = read_token_var(&[b0, b1, 0x00, 0x00], 0).unwrap();
                if w == 2 {
                    assert!(v < 0x10000, "2-byte token {v:#X} must stay narrow");
                } else {
                    assert!(v >= 0x10000, "4-byte token {v:#X} must not alias a narrow one");
                }
            }
        }
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
            parse_segment(seg),
            Some(BodyShape::StraightLine {
                params: vec![0xA496_0300],
                ops: vec![IlOp::Load(0xA496_0300)],
            })
        );
    }

    #[test]
    fn varint_short_form_is_signed() {
        // `-5` is `fb`, not a rejected byte. An earlier revision accepted only
        // 00..7F, which was fail-closed but silently blocked every negative
        // literal in the corpus.
        let cases: &[(&[u8], i32, usize)] = &[
            (&[0x00], 0, 1),
            (&[0x05], 5, 1),
            (&[0x7F], 127, 1),
            (&[0xFB], -5, 1),
            (&[0xFF], -1, 1),
            (&[0x81], -127, 1),
            // Escape form: `80` + 4-byte LE i32.
            (&[0x80, 0x70, 0x11, 0x01, 0x00], 70000, 5),
            // -128 cannot use the short form (0x80 is the marker), so it is
            // forced to the escape.
            (&[0x80, 0x80, 0xFF, 0xFF, 0xFF], -128, 5),
        ];
        for (bytes, want, width) in cases {
            let mut p = 0usize;
            assert_eq!(read_varint(bytes, &mut p), Some(*want), "{bytes:02X?}");
            assert_eq!(p, *width, "width for {bytes:02X?}");
        }
    }

    // ---- `.gl` symbol index -------------------------------------------------

    #[test]
    fn gl_symbol_index_binds_tokens_to_names() {
        // A `.gl` record is `<kind> <token> 00 <name> 00 <TYPE> …`. Transcribed
        // from a controlled fixture with three externals declared a, b, c —
        // tokens are assigned in DECLARATION order (0x09E3, 0x09E4, 0x09E5),
        // which is what makes a positional pairing with call order wrong.
        let mut gl = Vec::new();
        for (tok, name) in [
            ([0xE3u8, 0x09], "?a@@YAXXZ"),
            ([0xE4, 0x09], "?b@@YAXXZ"),
            ([0xE5, 0x09], "?c@@YAXXZ"),
        ] {
            gl.push(0x04); // kind
            gl.extend_from_slice(&tok);
            gl.push(0x00);
            gl.extend_from_slice(name.as_bytes());
            gl.push(0x00);
            gl.extend_from_slice(&[0x82, 0x07, 0x04]); // TYPE
        }
        let idx = gl_symbol_index(&gl);
        assert_eq!(idx.get(&0xE309).map(String::as_str), Some("?a@@YAXXZ"));
        assert_eq!(idx.get(&0xE409).map(String::as_str), Some("?b@@YAXXZ"));
        assert_eq!(idx.get(&0xE509).map(String::as_str), Some("?c@@YAXXZ"));
        // An unknown token must not resolve — the caller rejects rather than
        // guessing, since a wrong callee is a relocation against a wrong symbol.
        assert!(idx.get(&0xFFFF).is_none());
    }

    #[test]
    fn gl_symbol_index_ignores_non_identifier_runs() {
        // Binary payload between NULs must not become a symbol.
        let gl = b"\x00\x01\x02\x03\x00\x04\xE3\x09\x00?ok@@YAXXZ\x00".to_vec();
        let idx = gl_symbol_index(&gl);
        assert_eq!(idx.get(&0xE309).map(String::as_str), Some("?ok@@YAXXZ"));
        assert_eq!(idx.len(), 1, "only the identifier-shaped run is indexed");
    }

    // ---- inline type encoding (LEB128) --------------------------------------

    #[test]
    fn read_type_widths_match_the_captured_boundaries() {
        // Each of these is pinned in a live capture by the fixed one-byte marker
        // that follows it (`41` result-type, `55` arg push, `4C 4B` call end),
        // so a wrong width visibly swallows the next marker.
        let cases: &[(&[u8], u8, u8, u32, usize)] = &[
            (&[0x86, 0x41, 0x74], 0x86, 0x41, 116, 3),        // int
            (&[0x86, 0x42, 0x75], 0x86, 0x42, 117, 3),        // unsigned
            (&[0x82, 0x07, 0x03], 0x82, 0x07, 3, 3),          // void
            (&[0x86, 0x43, 0x83, 0x08], 0x86, 0x43, 1027, 4), // void*
            (&[0x86, 0x43, 0x82, 0x20], 0x86, 0x43, 4098, 4), // int**
            (&[0x88, 0x85, 0x41], 0x88, 0x85, 65, 3),         // double
            (&[0x88, 0x81, 0x13], 0x88, 0x81, 19, 3),         // long long
            (&[0x86, 0x43, 0x9B, 0xB9, 0x02], 0x86, 0x43, 40091, 5), // 5-byte id
        ];
        for (bytes, tag, kind, id, w) in cases {
            assert_eq!(
                read_type(bytes, 0),
                Some((*tag, *kind, *id, *w)),
                "type {bytes:02X?}"
            );
        }
        // A tag without bit 7 set is not a type.
        assert_eq!(read_type(&[0x41, 0x86, 0x41], 0), None);
    }

    #[test]
    fn call_token_is_decoded_not_anchor_matched() {
        // The old model hardcoded `00 80 01 10 00 00` as a fixed "callee anchor".
        // It is really flags=0 + varint(0x1001), so any TU whose callee function
        // type is not the first one created failed to parse. All three of these
        // are real captured CALL tokens with different fn-type ids and return
        // types; all must now decode.
        let cases: &[(&[u8], &str)] = &[
            (&[0xBD, 0x82, 0x07, 0x03, 0x00, 0x80, 0x01, 0x10, 0x00, 0x00], "void, id 0x1001"),
            (&[0xBD, 0x82, 0x07, 0x03, 0x00, 0x80, 0x80, 0x10, 0x00, 0x00], "void, id 0x1080"),
            (
                &[0xBD, 0x86, 0x43, 0x83, 0x08, 0x00, 0x80, 0xAE, 0x10, 0x00, 0x00],
                "void* (4-byte type), id 0x10AE",
            ),
        ];
        for (bytes, label) in cases {
            let mut p = 1; // past the BD
            let (_, _, _, w) = read_type(bytes, p).expect(label);
            p += w;
            assert_eq!(bytes.get(p), Some(&0x00), "{label}: cdecl flags byte");
            p += 1;
            assert!(read_varint(bytes, &mut p).is_some(), "{label}: fn-type id");
            assert_eq!(p, bytes.len(), "{label}: token must end exactly here");
        }
    }

    // ---- P2b function-level census ------------------------------------------

    #[test]
    fn census_agrees_with_the_gate_on_every_pinned_segment() {
        // The census must never disagree with acceptance: it is `.ok()` of the
        // same parse. Cross-check both directions over every pinned segment.
        let all: &[&[u8]] = &[
            MVP_CALL, MVP_FRAMED, INT_TAILRET, INT_PLUS0, INT_ARGTAIL, GA_SUBMOD, GA_MULMOD,
            GA_WIDEMOD, TWO_CALLS, CALL_THEN_STMT, ARGFRAMED_PLUSK, TWO_FRAMED_CALLS, PLUS1PLUS2,
        ];
        for seg in all {
            assert_eq!(
                parse_segment(seg).is_some(),
                parse_segment_detail(seg).is_ok(),
                "census/gate disagreement"
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
        let b = parse_segment_detail(cmp).unwrap_err();
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
        let b = parse_segment_detail(&seg).unwrap_err();
        assert_eq!(b.feature(), "expr-load-type-864175");
        assert_eq!(seg[b.off], 0xB9, "reported at the LOAD, not mid-type");
    }

    #[test]
    fn census_classifies_each_function_independently() {
        // The point of P2b: one blocked function does not hide the in-class
        // ones. `functions()` (the gate) is all-or-nothing and returns None.
        let mut ex: Vec<u8> = Vec::new();
        ex.extend_from_slice(&FN_START);
        ex.extend_from_slice(MVP_CALL);
        ex.extend_from_slice(&FN_START);
        ex.extend_from_slice(TWO_CALLS);
        let bundle = crate::IlBundle {
            base_name: "t".into(),
            files: vec![
                ("ex".to_string(), ex),
                ("gl".to_string(), b"?f@@YAXXZ\x00?g@@YAXXZ\x00".to_vec()),
            ]
            .into_iter()
            .collect(),
        };
        let census = bundle.function_census().unwrap();
        assert_eq!(census.len(), 2);
        assert_eq!(census[0].verdict, FnVerdict::InClass("void-tail-call"));
        assert!(!census[1].verdict.in_class());
        assert_eq!(census[0].name.as_deref(), Some("?f@@YAXXZ"));
        // In-class functions carry no hex window; blocked ones point at the
        // offending byte inside theirs.
        assert!(census[0].hex.is_empty());
        let FnVerdict::Blocked(b) = census[1].verdict else {
            panic!("expected a block");
        };
        assert_eq!(census[1].hex[census[1].hex_mark], b.byte.unwrap());
    }

    #[test]
    fn census_hex_window_is_clamped_to_the_segment() {
        // A block at offset 0 must not underflow, and one near the end must not
        // run past it — the window is diagnostic and must never panic.
        let tiny: &[u8] = &[0x4C, 0x4F, 0x11, 0xFF];
        let bundle = crate::IlBundle {
            base_name: "t".into(),
            files: vec![
                ("ex".to_string(), tiny.to_vec()),
                ("gl".to_string(), b"?f@@YAXXZ\x00".to_vec()),
            ]
            .into_iter()
            .collect(),
        };
        let census = bundle.function_census().unwrap();
        assert_eq!(census.len(), 1);
        let c = &census[0];
        assert!(!c.verdict.in_class());
        assert!(c.hex_mark < c.hex.len().max(1));
        assert!(c.hex.len() <= CENSUS_HEX_BACK + CENSUS_HEX_FWD);
    }

    // ---- real captured segments (transcribed from live-toolchain `.ex`) -----

    /// `void f(){ g(); }` — accepted bare void tail call.
    const MVP_CALL: &[u8] = &[
        0x4C, 0x4F, 0x11, 0x53, 0x26, 0xE3, 0x09, 0xBD, 0x82, 0x07, 0x03, 0x00, 0x80, 0x01, 0x10,
        0x00, 0x00, 0x4C, 0x4B, 0x3A, 0xE5, 0x09, 0x54, 0x02, 0x29, 0xE5, 0x09, 0x4F, 0x12, 0x47,
        0x54, 0x01, 0x54, 0x00, 0x4F, 0x02, 0x20, 0x00, 0x4F, 0x01, 0x07, 0x4D,
    ];
    /// `int f(int a){ return g(a) + 1; }` — accepted framed call (k=1).
    const MVP_FRAMED: &[u8] = &[
        0x4C, 0x4F, 0x11, 0x53, 0x26, 0xE4, 0x09, 0xBD, 0x86, 0x41, 0x74, 0x00, 0x80, 0x01, 0x10,
        0x00, 0x00, 0xB9, 0xE5, 0x09, 0x86, 0x41, 0x74, 0x55, 0x86, 0x41, 0x74, 0x4C, 0x33, 0x86,
        0x41, 0x74, 0x01, 0x02, 0x41, 0x86, 0x41, 0x74, 0x3A, 0xE7, 0x09, 0x54, 0x02, 0x29, 0xE7,
        0x09, 0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00, 0x4F, 0x02, 0x20, 0x00, 0x4F, 0x01, 0x08,
        0x4D,
    ];
    /// `return g(a) - 1;` — non-commutative post-op (SUB) → reject.
    const GA_SUBMOD: &[u8] = &[
        0x4C, 0x4F, 0x11, 0x53, 0x26, 0xE4, 0x09, 0xBD, 0x86, 0x41, 0x74, 0x00, 0x80, 0x01, 0x10,
        0x00, 0x00, 0xB9, 0xE5, 0x09, 0x86, 0x41, 0x74, 0x55, 0x86, 0x41, 0x74, 0x4C, 0x33, 0x86,
        0x41, 0x74, 0x01, 0x03, 0x41, 0x86, 0x41, 0x74, 0x3A, 0xE7, 0x09, 0x54, 0x02, 0x29, 0xE7,
        0x09, 0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00, 0x4F, 0x02, 0x20, 0x00, 0x4F, 0x01, 0x09,
        0x4D,
    ];
    /// `return g(a) * 5;` — strength-reduced post-op (MUL) → reject.
    const GA_MULMOD: &[u8] = &[
        0x4C, 0x4F, 0x11, 0x53, 0x26, 0xE4, 0x09, 0xBD, 0x86, 0x41, 0x74, 0x00, 0x80, 0x01, 0x10,
        0x00, 0x00, 0xB9, 0xE5, 0x09, 0x86, 0x41, 0x74, 0x55, 0x86, 0x41, 0x74, 0x4C, 0x33, 0x86,
        0x41, 0x74, 0x05, 0x04, 0x41, 0x86, 0x41, 0x74, 0x3A, 0xE7, 0x09, 0x54, 0x02, 0x29, 0xE7,
        0x09, 0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00, 0x4F, 0x02, 0x20, 0x00, 0x4F, 0x01, 0x07,
        0x4D,
    ];
    /// `return g(a) + 70000;` — wide post-op immediate → reject.
    const GA_WIDEMOD: &[u8] = &[
        0x4C, 0x4F, 0x11, 0x53, 0x26, 0xE4, 0x09, 0xBD, 0x86, 0x41, 0x74, 0x00, 0x80, 0x01, 0x10,
        0x00, 0x00, 0xB9, 0xE5, 0x09, 0x86, 0x41, 0x74, 0x55, 0x86, 0x41, 0x74, 0x4C, 0x33, 0x86,
        0x41, 0x74, 0x80, 0x70, 0x11, 0x01, 0x00, 0x02, 0x41, 0x86, 0x41, 0x74, 0x3A, 0xE7, 0x09,
        0x54, 0x02, 0x29, 0xE7, 0x09, 0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00, 0x4F, 0x02, 0x20,
        0x00, 0x4F, 0x01, 0x07, 0x4D,
    ];
    // The three ACCEPTED integer tail-call segments (transcribed from live
    // 16.00.11886.00 `.ex` captures of `return g(a)`, `g(a)+0`, `g(a+1)`).
    // Unlike the void/framed constants above, these start at the `46` formals
    // marker (param a = token 0xE509) — the arg-setup codegen maps the argument
    // tokens to registers, so the formal list must be present.

    /// `int f(int a){ return g(a); }` — passthrough: arg region is the bare
    /// LOAD, no post-op → integer tail call (bare `b g`).
    const INT_TAILRET: &[u8] = &[
        0x46, 0x2D, 0xE5, 0x09, // formals: a = e509
        0x4C, 0x4F, 0x11, 0x53, // LO SS
        0x26, 0xE4, 0x09, 0xBD, 0x86, 0x41, 0x74, 0x00, 0x80, 0x01, 0x10, 0x00, 0x00, // 26 CALL
        0xB9, 0xE5, 0x09, 0x86, 0x41, 0x74, // LOAD a (the argument)
        0x55, 0x86, 0x41, 0x74, 0x4C, // 55 <int> 4C call-end
        0x41, 0x86, 0x41, 0x74, // result-type int (no post-op → tail call)
        0x3A, 0xE7, 0x09, 0x54, 0x02, 0x29, 0xE7, 0x09, // assign + return
        0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00, 0x4F, 0x02, 0x20, 0x00, 0x4F, 0x01, 0x03, 0x4D,
    ];
    /// `int f(int a){ return g(a) + 0; }` — identity fold: same arg LOAD, then a
    /// real `33 86 41 74 00 02` (LIT 0 + ADD) post-op that folds to a tail call.
    const INT_PLUS0: &[u8] = &[
        0x46, 0x2D, 0xE5, 0x09, // formals: a = e509
        0x4C, 0x4F, 0x11, 0x53, // LO SS
        0x26, 0xE4, 0x09, 0xBD, 0x86, 0x41, 0x74, 0x00, 0x80, 0x01, 0x10, 0x00, 0x00, // 26 CALL
        0xB9, 0xE5, 0x09, 0x86, 0x41, 0x74, // LOAD a
        0x55, 0x86, 0x41, 0x74, 0x4C, // call-end
        0x33, 0x86, 0x41, 0x74, 0x00, 0x02, // post-op LIT 0 + ADD (folds away)
        0x41, 0x86, 0x41, 0x74, // result-type int
        0x3A, 0xE7, 0x09, 0x54, 0x02, 0x29, 0xE7, 0x09, // assign + return
        0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00, 0x4F, 0x02, 0x20, 0x00, 0x4F, 0x01, 0x03, 0x4D,
    ];
    /// `int f(int a){ return g(a + 1); }` — arg-setup: the `+1` is IN the
    /// argument (LOAD+LIT+ADD before `55`), no post-op → integer tail call
    /// (`addi r3,r3,1 ; b g`). Not to be mistaken for framed `g(a)+1`.
    const INT_ARGTAIL: &[u8] = &[
        0x46, 0x2D, 0xE5, 0x09, // formals: a = e509
        0x4C, 0x4F, 0x11, 0x53, // LO SS
        0x26, 0xE4, 0x09, 0xBD, 0x86, 0x41, 0x74, 0x00, 0x80, 0x01, 0x10, 0x00, 0x00, // 26 CALL
        0xB9, 0xE5, 0x09, 0x86, 0x41, 0x74, // LOAD a
        0x33, 0x86, 0x41, 0x74, 0x01, 0x02, // LIT 1 + ADD (computes a+1 into the arg)
        0x55, 0x86, 0x41, 0x74, 0x4C, // call-end
        0x41, 0x86, 0x41, 0x74, // result-type int (no post-op → tail call)
        0x3A, 0xE7, 0x09, 0x54, 0x02, 0x29, 0xE7, 0x09, // assign + return
        0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00, 0x4F, 0x02, 0x20, 0x00, 0x4F, 0x01, 0x03, 0x4D,
    ];
    /// `void f(){ g(); g(); }` — a SECOND call stands where the void tail call's
    /// return plumbing must be → reject (defect #1).
    const TWO_CALLS: &[u8] = &[
        0x4C, 0x4F, 0x11, 0x53, 0x26, 0xE3, 0x09, 0xBD, 0x82, 0x07, 0x03, 0x00, 0x80, 0x01, 0x10,
        0x00, 0x00, 0x4C, 0x4B, 0x26, 0xE3, 0x09, 0xBD, 0x82, 0x07, 0x03, 0x00, 0x80, 0x01, 0x10,
        0x00, 0x00, 0x4C, 0x4B, 0x3A, 0xE5, 0x09, 0x54, 0x02, 0x29, 0xE5, 0x09, 0x4F, 0x12, 0x47,
        0x54, 0x01, 0x54, 0x00, 0x4F, 0x02, 0x20, 0x00, 0x4F, 0x01, 0x06, 0x4D,
    ];
    /// `int f(int a){ g(); return a + 1; }` — a second statement follows the
    /// void call's `4C 4B` (a `B9` LOAD where the return plumbing must be) →
    /// reject (defect #2).
    const CALL_THEN_STMT: &[u8] = &[
        0x4C, 0x4F, 0x11, 0x53, 0x26, 0xE3, 0x09, 0xBD, 0x82, 0x07, 0x03, 0x00, 0x80, 0x01, 0x10,
        0x00, 0x00, 0x4C, 0x4B, 0xB9, 0xE4, 0x09, 0x86, 0x41, 0x74, 0x33, 0x86, 0x41, 0x74, 0x01,
        0x02, 0x41, 0x86, 0x41, 0x74, 0x3A, 0xE6, 0x09, 0x54, 0x02, 0x29, 0xE6, 0x09, 0x4F, 0x12,
        0x47, 0x54, 0x01, 0x54, 0x00, 0x4F, 0x02, 0x20, 0x00, 0x4F, 0x01, 0x05, 0x4D,
    ];
    /// `int f(int a){ return g(a + 1) + 1; }` — in-argument arithmetic AND a
    /// framed post-op: the arg region carries LOAD+LIT+ADD before `55` → reject
    /// (defect #3; a naive post-`55` search would mis-accept as framed g(a)+1).
    const ARGFRAMED_PLUSK: &[u8] = &[
        0x4C, 0x4F, 0x11, 0x53, 0x26, 0xE4, 0x09, 0xBD, 0x86, 0x41, 0x74, 0x00, 0x80, 0x01, 0x10,
        0x00, 0x00, 0xB9, 0xE5, 0x09, 0x86, 0x41, 0x74, 0x33, 0x86, 0x41, 0x74, 0x01, 0x02, 0x55,
        0x86, 0x41, 0x74, 0x4C, 0x33, 0x86, 0x41, 0x74, 0x01, 0x02, 0x41, 0x86, 0x41, 0x74, 0x3A,
        0xE7, 0x09, 0x54, 0x02, 0x29, 0xE7, 0x09, 0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00, 0x4F,
        0x02, 0x20, 0x00, 0x4F, 0x01, 0x06, 0x4D,
    ];
    /// `int f(int a){ return g(a) + g(a + 1); }` — a SECOND call follows the
    /// first call-end where the framed post-op literal must be → reject
    /// (defect #4).
    const TWO_FRAMED_CALLS: &[u8] = &[
        0x4C, 0x4F, 0x11, 0x53, 0x26, 0xE4, 0x09, 0xBD, 0x86, 0x41, 0x74, 0x00, 0x80, 0x01, 0x10,
        0x00, 0x00, 0xB9, 0xE5, 0x09, 0x86, 0x41, 0x74, 0x55, 0x86, 0x41, 0x74, 0x4C, 0x26, 0xE4,
        0x09, 0xBD, 0x86, 0x41, 0x74, 0x00, 0x80, 0x01, 0x10, 0x00, 0x00, 0xB9, 0xE5, 0x09, 0x86,
        0x41, 0x74, 0x33, 0x86, 0x41, 0x74, 0x01, 0x02, 0x55, 0x86, 0x41, 0x74, 0x4C, 0x02, 0x41,
        0x86, 0x41, 0x74, 0x3A, 0xE7, 0x09, 0x54, 0x02, 0x29, 0xE7, 0x09, 0x4F, 0x12, 0x47, 0x54,
        0x01, 0x54, 0x00, 0x4F, 0x02, 0x20, 0x00, 0x4F, 0x01, 0x05, 0x4D,
    ];
    /// `int f(int a){ return g(a) + 1 + 2; }` — a SECOND literal+ADD follows the
    /// framed post-op where the result-type must be → reject.
    const PLUS1PLUS2: &[u8] = &[
        0x4C, 0x4F, 0x11, 0x53, 0x26, 0xE4, 0x09, 0xBD, 0x86, 0x41, 0x74, 0x00, 0x80, 0x01, 0x10,
        0x00, 0x00, 0xB9, 0xE5, 0x09, 0x86, 0x41, 0x74, 0x55, 0x86, 0x41, 0x74, 0x4C, 0x33, 0x86,
        0x41, 0x74, 0x01, 0x02, 0x33, 0x86, 0x41, 0x74, 0x02, 0x02, 0x41, 0x86, 0x41, 0x74, 0x3A,
        0xE7, 0x09, 0x54, 0x02, 0x29, 0xE7, 0x09, 0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00, 0x4F,
        0x02, 0x20, 0x00, 0x4F, 0x01, 0x03, 0x4D,
    ];
}
