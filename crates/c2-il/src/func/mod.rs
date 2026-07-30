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
//! token sequence is exactly one of the recognized [`BodyShape`]s; the
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

mod body;
mod bundle;
mod census;
mod gl;
mod readers;
mod sy;

pub use self::body::Block;
pub use self::bundle::{is_empty_module, OPT_WORD_O1, OPT_WORD_OX};
pub use self::census::{FnCensus, FnVerdict, CENSUS_HEX_BACK, CENSUS_HEX_FWD};
pub use self::gl::{gl_symbol_index, mangled_name, mangled_names, source_path};
pub use self::readers::detect_token_width;

/// A single straight-line IL operation in the integer-arithmetic class.
///
/// The binary ops are postfix (each pops two operands, pushes one result).
/// `Sub` is **non-commutative** — its operand order is load-bearing (see the
/// codegen for the `subf` operand mapping); `Add`/`Mul` are commutative.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IlOp {
    /// Load a named variable (by IL token) onto the expression stack.
    Load(u32),
    /// **Indirect** load: pop a pointer, push the 4-byte integer it designates,
    /// `off` bytes in (IL `30 <TYPE>`, optionally preceded by one byte-offset add
    /// — `27 <TYPE>` for a member or `28 00 00` for a subscript).
    ///
    /// Produced ONLY by [`try_parse_indirect_load_leaf`], and only as the second
    /// and last op of a two-op stream `[Load(base), LoadInd { off }]`. Nothing
    /// lowers it in combination with arithmetic, because c2 does not lower it
    /// that way: `*p + 1` is `lwz r11,0(r3) ; addi r3,r11,1` — the load lands in
    /// the *scratch* register, not the destination — and `*p * 3` is
    /// strength-reduced to `lwz r11 ; slwi r10,r11,1 ; add r3,r11,r10`. See
    /// `docs/IL_EXPR_LAYER.md` §6 and `fixtures/cpp/il_expr_load_neg.cpp`.
    LoadInd { off: i32 },
    /// **Indirect load of a non-4-byte scalar** (T3): pop a pointer, push the
    /// `width`-byte value it designates, `off` bytes in. Same IL production as
    /// [`IlOp::LoadInd`] (`30 <TYPE>`, optionally preceded by one byte-offset add)
    /// — only the pointee TYPE differs, and with it the load opcode:
    ///
    /// ```text
    ///  width 1  ->  lbz     width 2  ->  lhz     width 8  ->  ld (DS-form)
    /// ```
    ///
    /// `sext` records that the IL widens the loaded value to `int` with a
    /// `2C 86 41 74 00` **and c2 pays an instruction for it**: the load then targets
    /// r11 and an `extsb` produces r3 (`89630000 7d630774` — the r11-then-r3 rule).
    /// It is `true` only at `width == 1`, and only for a *signed* pointee:
    ///
    /// * an **unsigned** narrow pointee widens for free (`lbz`/`lhz` already
    ///   zero-extend), so its `2C` decodes to `sext: false` — the same bytes as no
    ///   conversion at all (measured: `int f(unsigned char*)`, `int f(bool*)`,
    ///   `int f(unsigned short*)`, `int f(wchar_t*)` are each a bare
    ///   `lbz`/`lhz r3` + `blr`);
    /// * a **signed 2-byte** pointee widened to int is *mode-dependent* — `/O1`
    ///   emits one `lha r3`, `/Ox` and `/O2` emit `lhz r11 ; extsh r3,r11` — and is
    ///   refused by the parser rather than represented here (see
    ///   [`try_parse_indirect_load_leaf`] and `fixtures/cpp/w12_narrow_neg.cpp`);
    /// * `width == 8` never carries a conversion (a `long long`→int truncation is
    ///   not captured), so `sext` is always `false` there.
    ///
    /// A separate variant rather than extra fields on [`IlOp::LoadInd`] so the
    /// 4-byte integer load — every currently-matching fixture — keeps its exact
    /// representation and provably identical bytes.
    LoadIndSized { off: i32, width: u8, sext: bool },
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
    pub(crate) fn from_opcode(b: u8) -> Option<Rel> {
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
    /// If this function is a **W13a floating-point leaf**, whether it is double
    /// precision. Mutually exclusive with the other body kinds.
    pub float_leaf: Option<bool>,
    /// A **multi-argument** tail call's argument permutation. `Some(sources)`
    /// means this is `return g(a1, …, an)` with `n >= 2` and every argument a bare
    /// parameter: `sources[i]` is the index into [`Self::params`] of the value that
    /// argument slot `i` (register `r(3+i)`) wants. Set together with
    /// [`Self::tail_call`], and then [`Self::ops`] is empty — the permutation, not
    /// an operand stream, is the whole argument setup.
    ///
    /// The one-argument case keeps using `ops` instead, because it can carry a
    /// computed argument (`g(a + 1)`) that the permutation form cannot express.
    pub arg_sources: Option<Vec<usize>>,
    /// True iff this function's body is **empty** (`void f() {}`): no expression at
    /// all, so codegen emits a bare `blr`. Mutually exclusive with the other body
    /// kinds.
    ///
    /// (These discriminators want to be one enum. [`BodyShape`] already *is* that
    /// enum — the parser produces it and `functions()` immediately flattens it into
    /// the parallel options above, which `PortC2::build` then re-derives through two
    /// separate priority chains. The remaining reason to defer is the CFG step's
    /// real body IR (docs/ROADMAP.md §G4), but carrying `BodyShape` here does not
    /// need that design and would remove the second decision tree. This doc block
    /// was itself misattached to `float_leaf` for a while, which is the kind of
    /// damage the sum type prevents.)
    pub empty_body: bool,
}

/// Pinned `.ex` segments and helpers shared by the per-module test suites.
/// Every byte array is transcribed verbatim from a live-toolchain capture (see
/// each item's own comment); nothing here is hand-assembled.
///
/// Each segment begins at its `53 53` statement start, which is where the
/// pre-body region the parser reads actually begins — the opaque `4F 33 …` header
/// ahead of it is excluded deliberately, and the one thing that matters about it
/// (that it can contain a stray `0x46`) is covered by
/// `parse_formals_anchors_on_the_marker_that_reaches_lo`, which synthesizes the
/// line-70 marker in front. These three used to start at the `46` formals marker
/// instead, which meant the region where the `this` binding lives — and where a
/// wrong-bytes emit lived, see `expr::formals_marker` — was in no fixture at all.
#[cfg(test)]
pub(crate) mod test_fixtures {
    /// These pinned segments are synthetic and have no `.sy` companion, so an
    /// empty local set is the honest input: nothing here is a local the parse may
    /// fold into the expression that reads it.
    pub(crate) const NO_LOCALS: &[u32] = &[];

    /// Prefix a pinned body with the `53 53 26 <fn>` statement start a real segment
    /// carries, when it does not already have one.
    ///
    /// Several segments here begin at the `46` formals marker. Without the preceding
    /// function-token push, the `this` binding is **undetermined** — there is nothing
    /// to tell a free function from a member whose `this` group was cut off — and
    /// `parse_params` refuses on undetermined by design, because conflating it with
    /// "absent" is what mis-emitted `S8::m`'s base register. Supplying the prologue
    /// says "free function, no `this`", which is what these bodies are.
    ///
    /// Idempotent, so it is safe to apply at every call site; the segments that were
    /// already transcribed whole pass through untouched.
    pub(crate) fn free_fn(body: &[u8]) -> Vec<u8> {
        if body.first() == Some(&0x53) {
            return body.to_vec();
        }
        let mut v = vec![0x53, 0x53, 0x26, 0xE2, 0x09];
        v.extend_from_slice(body);
        v
    }


    // ---- indirect-load leaf -------------------------------------------------
    //
    // Every byte below is transcribed from a live capture of
    // `fixtures/cpp/il_expr_deref.cpp` / `il_expr_member.cpp`
    // (`c2rs census <cpp> --keep-il <dir>`), not derived.

    /// `int ld_p(int* p) { return *p; }` — one formal, no offset add.
    pub(crate) const IND_DEREF: &[u8] = &[
        0x53, 0x53, 0x26, 0xEF, 0x09, // statement start, function-symbol push
        0x46, 0x2D, 0xEE, 0x09, // formals: p
        0x4C, 0x4F, 0x11, 0x53, // LO SS
        0xB9, 0xEE, 0x09, 0x86, 0x43, 0xF4, 0x08, // LOAD p (int *)
        0x30, 0x86, 0x41, 0x74, // indirect load -> int
        0x41, 0x86, 0x41, 0x74, // result type int
        0x3A, 0xF0, 0x09, 0x54, 0x02, 0x29, 0xF0, 0x09, // return plumbing
        0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00, // fn tail = segment end
    ];

    /// `int ld_m0(S* s) { return s->a; }` — a `27` byte-offset add of 0.
    pub(crate) const IND_MEMBER0: &[u8] = &[
        0x53, 0x53, 0x26, 0xFF, 0x09, // statement start, function-symbol push
        0x46, 0x2D, 0xFE, 0x09, // formals: s
        0x4C, 0x4F, 0x11, 0x53, // LO SS
        0xB9, 0xFE, 0x09, 0x86, 0x43, 0x81, 0x20, // LOAD s (S *)
        0x33, 0x86, 0x41, 0x74, 0x00, // LITERAL int 0 (byte offset)
        0x27, 0x86, 0x43, 0xF4, 0x08, // byte-offset add -> int *
        0x30, 0x86, 0x41, 0x74, // indirect load -> int
        0x41, 0x86, 0x41, 0x74, //
        0x3A, 0x00, 0x0A, 0x54, 0x02, 0x29, 0x00, 0x0A, //
        0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00,
    ];

    /// `int ld_ixneg(int* p) { return p[-1]; }` — a `28 00 00` subscript add whose
    /// offset is the **signed** short form `FC` = −4, typed `long` not `int`.
    pub(crate) const IND_SUBSCRIPT_NEG: &[u8] = &[
        0x53, 0x53, 0x26, 0x11, 0x0A, // statement start, function-symbol push
        0x46, 0x2D, 0x10, 0x0A, // formals: p
        0x4C, 0x4F, 0x11, 0x53, //
        0xB9, 0x10, 0x0A, 0x86, 0x43, 0xF4, 0x08, // LOAD p (int *)
        0x33, 0x86, 0x41, 0x12, 0xFC, // LITERAL long -4
        0x28, 0x00, 0x00, // subscript add
        0x30, 0x86, 0x41, 0x74, //
        0x41, 0x86, 0x41, 0x74, //
        0x3A, 0x12, 0x0A, 0x54, 0x02, 0x29, 0x12, 0x0A, //
        0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00,
    ];

    /// `int C::get_b() const { return b; }` — the `this` form: the pre-body
    /// region binds `this` with `B9 <tok> <TYPE> 99 <TYPE> 00` and the `2D`
    /// formals list is EMPTY, so `this` must come from that binding or the base
    /// register is wrong. The load type is `const int` and is stripped by a `2C`.
    pub(crate) const IND_THIS_GETTER: &[u8] = &[
        0x53, 0x53, 0x26, 0xE7, 0x09, // fn symbol push
        0xB9, 0xF8, 0x09, 0xA6, 0x43, 0x82, 0x20, // LOAD this (C * const)
        0x99, 0x86, 0x43, 0x84, 0x20, 0x00, // bind-member, offset 0
        0x46, 0x4C, 0x4F, 0x11, 0x53, // formals (none) LO SS
        0xB9, 0xF8, 0x09, 0xA6, 0x43, 0x82, 0x20, // LOAD this
        0x33, 0x86, 0x41, 0x74, 0x04, // LITERAL int 4
        0x27, 0xA6, 0x43, 0x8E, 0x20, // byte-offset add -> const int *
        0x30, 0xA6, 0x41, 0x8D, 0x20, // indirect load -> const int
        0x2C, 0x86, 0x41, 0x74, 0x00, // cv strip -> int
        0x41, 0x86, 0x41, 0x74, //
        0x3A, 0xF9, 0x09, 0x54, 0x02, 0x29, 0xF9, 0x09, //
        0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00,
    ];

    // ---- T3: non-4-byte pointees --------------------------------------------
    //
    // Transcribed from a live capture of `fixtures/cpp/w12_narrow_getters.cpp` and
    // `fixtures/cpp/w12_narrow_neg.cpp` (`c2rs census <cpp> --keep-il <dir>`).
    // Whole segments, `53 53` statement start through `54 00` — not suffixes.

    /// `char g_c_c(char* p) { return *p; }` — a 1-byte pointee, no conversion:
    /// `30 82 11 70` / `41 82 11 70`. Emits `lbz r3,0(r3)` and *no* sign
    /// extension, which is what makes "a signed load sign-extends" the wrong rule.
    pub(crate) const NARROW_CHAR_DEREF: &[u8] = &[
        0x53, 0x53, 0x26, 0x10, 0x0A, 0x46, 0x2D, 0x0F, 0x0A, 0x4C, 0x4F, 0x11,
        0x53, 0xB9, 0x0F, 0x0A, 0x86, 0x43, 0xF0, 0x08, 0x30, 0x82, 0x11, 0x70,
        0x41, 0x82, 0x11, 0x70, 0x3A, 0x11, 0x0A, 0x54, 0x02, 0x29, 0x11, 0x0A,
        0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00,
    ];

    /// `int g_i_c(char* p) { return *p; }` — the same load plus the widening
    /// `2C 86 41 74 00`, which costs `extsb r3,r11` and moves the load's target to
    /// r11. Differs from [`NARROW_CHAR_DEREF`] by exactly those five bytes and the
    /// result type.
    pub(crate) const NARROW_CHAR_TO_INT: &[u8] = &[
        0x53, 0x53, 0x26, 0x2B, 0x0A, 0x46, 0x2D, 0x2A, 0x0A, 0x4C, 0x4F, 0x11,
        0x53, 0xB9, 0x2A, 0x0A, 0x86, 0x43, 0xF0, 0x08, 0x30, 0x82, 0x11, 0x70,
        0x2C, 0x86, 0x41, 0x74, 0x00, 0x41, 0x86, 0x41, 0x74, 0x3A, 0x2C, 0x0A,
        0x54, 0x02, 0x29, 0x2C, 0x0A, 0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00,
    ];

    /// `int g_i_us(unsigned short* p) { return *p; }` — an *unsigned* 2-byte
    /// pointee (`30 84 22 21`) carrying the **same** widening token as
    /// [`NARROW_CHAR_TO_INT`] and emitting nothing for it (`lhz r3` already
    /// zero-extends). The pair is what pins the extension to the pointee's
    /// signedness rather than to the token.
    pub(crate) const NARROW_USHORT_TO_INT: &[u8] = &[
        0x53, 0x53, 0x26, 0x3B, 0x0A, 0x46, 0x2D, 0x3A, 0x0A, 0x4C, 0x4F, 0x11,
        0x53, 0xB9, 0x3A, 0x0A, 0x86, 0x43, 0xA1, 0x08, 0x30, 0x84, 0x22, 0x21,
        0x2C, 0x86, 0x41, 0x74, 0x00, 0x41, 0x86, 0x41, 0x74, 0x3A, 0x3C, 0x0A,
        0x54, 0x02, 0x29, 0x3C, 0x0A, 0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00,
    ];

    /// `long long m_q(S* s) { return s->q; }` — an 8-byte pointee at offset 16.
    /// The `27` type is `88 43 93 08` (a pointer tagged with the *pointee's* width
    /// and alignment) over a `30 88 81 13` load: two independent statements of
    /// "8 bytes, naturally aligned", which is what makes the DS-form `ld` legal.
    pub(crate) const NARROW_LL_MEMBER: &[u8] = &[
        0x53, 0x53, 0x26, 0x50, 0x0A, 0x46, 0x2D, 0x4F, 0x0A, 0x4C, 0x4F, 0x11,
        0x53, 0xB9, 0x4F, 0x0A, 0x86, 0x43, 0x81, 0x20, 0x33, 0x86, 0x41, 0x74,
        0x10, 0x27, 0x88, 0x43, 0x93, 0x08, 0x30, 0x88, 0x81, 0x13, 0x41, 0x88,
        0x81, 0x13, 0x3A, 0x51, 0x0A, 0x54, 0x02, 0x29, 0x51, 0x0A, 0x4F, 0x12,
        0x47, 0x54, 0x01, 0x54, 0x00,
    ];

    /// `char C::t_c() const { return c; }` — a `const` member getter through
    /// `this`. The load is `const char` (`30 A2 11 98 20`) and the `2C` strips the
    /// qualification to plain `char` (`2C 82 11 70 00`): same width, same
    /// signedness, no instruction — the *other* thing a `2C` can mean here.
    pub(crate) const NARROW_CONST_CHAR_THIS: &[u8] = &[
        0x53, 0x53, 0x26, 0xF8, 0x09, 0xB9, 0x53, 0x0A, 0xA6, 0x43, 0x86, 0x20,
        0x99, 0x86, 0x43, 0x88, 0x20, 0x00, 0x46, 0x4C, 0x4F, 0x11, 0x53, 0xB9,
        0x53, 0x0A, 0xA6, 0x43, 0x86, 0x20, 0x33, 0x86, 0x41, 0x74, 0x00, 0x27,
        0xA2, 0x43, 0x99, 0x20, 0x30, 0xA2, 0x11, 0x98, 0x20, 0x2C, 0x82, 0x11,
        0x70, 0x00, 0x41, 0x82, 0x11, 0x70, 0x3A, 0x54, 0x0A, 0x54, 0x02, 0x29,
        0x54, 0x0A, 0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00,
    ];

    /// `int nw_widen_short(short* p) { return *p; }` — **refused**. Byte-for-byte
    /// [`NARROW_USHORT_TO_INT`] with a *signed* 2-byte load (`30 84 21 11`), and
    /// the only shape in the family whose instruction count depends on the
    /// optimization mode: `/O1` emits one `lha r3`, `/Ox` and `/O2` emit
    /// `lhz r11 ; extsh r3,r11`. This lowering path has no mode, so the parser
    /// refuses instead of picking one.
    pub(crate) const NARROW_SHORT_TO_INT_REFUSED: &[u8] = &[
        0x53, 0x53, 0x26, 0xEE, 0x09, 0x46, 0x2D, 0xED, 0x09, 0x4C, 0x4F, 0x11,
        0x53, 0xB9, 0xED, 0x09, 0x86, 0x43, 0x91, 0x08, 0x30, 0x84, 0x21, 0x11,
        0x2C, 0x86, 0x41, 0x74, 0x00, 0x41, 0x86, 0x41, 0x74, 0x3A, 0xEF, 0x09,
        0x54, 0x02, 0x29, 0xEF, 0x09, 0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00,
    ];

    /// `long long nw_ds(P* s) { return s->q; }` over a `#pragma pack(1)` struct —
    /// **refused**, and the reason the width is matched as a (tag, kind) *pair*.
    /// The member is at offset 3, and a packed member's TYPE tag carries the
    /// *alignment* class, not the width: `30 82 81 13` (align 1, kind says 8 bytes)
    /// against [`NARROW_LL_MEMBER`]'s `30 88 81 13`. Deriving the width from the
    /// tag's low nibble reads this as one byte and emits `lbz` for a `long long`;
    /// c2 emits `li r11,3 ; ldx r3,r3,r11`, since offset 3 is not a DS-form
    /// displacement at all.
    pub(crate) const NARROW_LL_PACKED_REFUSED: &[u8] = &[
        0x53, 0x53, 0x26, 0x01, 0x0A, 0x46, 0x2D, 0x00, 0x0A, 0x4C, 0x4F, 0x11,
        0x53, 0xB9, 0x00, 0x0A, 0x86, 0x43, 0x81, 0x20, 0x33, 0x86, 0x41, 0x74,
        0x03, 0x27, 0x82, 0x43, 0x93, 0x08, 0x30, 0x82, 0x81, 0x13, 0x41, 0x82,
        0x81, 0x13, 0x3A, 0x02, 0x0A, 0x54, 0x02, 0x29, 0x02, 0x0A, 0x4F, 0x12,
        0x47, 0x54, 0x01, 0x54, 0x00,
    ];

    // ---- real captured segments (transcribed from live-toolchain `.ex`) -----

    /// `void f(){ g(); }` — accepted bare void tail call.
    pub(crate) const MVP_CALL: &[u8] = &[
        0x53, 0x53, 0x26, 0xE4, 0x09, 0x46, // stmt start, fn push, formals: none
        0x4C, 0x4F, 0x11, 0x53, 0x26, 0xE3, 0x09, 0xBD, 0x82, 0x07, 0x03, 0x00, 0x80, 0x01, 0x10,
        0x00, 0x00, 0x4C, 0x4B, 0x3A, 0xE5, 0x09, 0x54, 0x02, 0x29, 0xE5, 0x09, 0x4F, 0x12, 0x47,
        0x54, 0x01, 0x54, 0x00, 0x4F, 0x02, 0x20, 0x00, 0x4F, 0x01, 0x07, 0x4D,
    ];
    /// `int f(int a){ return g(a) + 1; }` — accepted framed call (k=1).
    pub(crate) const MVP_FRAMED: &[u8] = &[
        0x53, 0x53, 0x26, 0xE6, 0x09, 0x46, 0x2D, 0xE5, 0x09, // stmt start, fn push, formals: a
        0x4C, 0x4F, 0x11, 0x53, 0x26, 0xE4, 0x09, 0xBD, 0x86, 0x41, 0x74, 0x00, 0x80, 0x01, 0x10,
        0x00, 0x00, 0xB9, 0xE5, 0x09, 0x86, 0x41, 0x74, 0x55, 0x86, 0x41, 0x74, 0x4C, 0x33, 0x86,
        0x41, 0x74, 0x01, 0x02, 0x41, 0x86, 0x41, 0x74, 0x3A, 0xE7, 0x09, 0x54, 0x02, 0x29, 0xE7,
        0x09, 0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00, 0x4F, 0x02, 0x20, 0x00, 0x4F, 0x01, 0x08,
        0x4D,
    ];
    /// `return g(a) - 1;` — non-commutative post-op (SUB) → reject.
    pub(crate) const GA_SUBMOD: &[u8] = &[
        0x53, 0x53, 0x26, 0xE6, 0x09, 0x46, 0x2D, 0xE5, 0x09, // stmt start, fn push, formals: a
        0x4C, 0x4F, 0x11, 0x53, 0x26, 0xE4, 0x09, 0xBD, 0x86, 0x41, 0x74, 0x00, 0x80, 0x01, 0x10,
        0x00, 0x00, 0xB9, 0xE5, 0x09, 0x86, 0x41, 0x74, 0x55, 0x86, 0x41, 0x74, 0x4C, 0x33, 0x86,
        0x41, 0x74, 0x01, 0x03, 0x41, 0x86, 0x41, 0x74, 0x3A, 0xE7, 0x09, 0x54, 0x02, 0x29, 0xE7,
        0x09, 0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00, 0x4F, 0x02, 0x20, 0x00, 0x4F, 0x01, 0x09,
        0x4D,
    ];
    /// `return g(a) * 5;` — strength-reduced post-op (MUL) → reject.
    pub(crate) const GA_MULMOD: &[u8] = &[
        0x53, 0x53, 0x26, 0xE6, 0x09, 0x46, 0x2D, 0xE5, 0x09, // stmt start, fn push, formals: a
        0x4C, 0x4F, 0x11, 0x53, 0x26, 0xE4, 0x09, 0xBD, 0x86, 0x41, 0x74, 0x00, 0x80, 0x01, 0x10,
        0x00, 0x00, 0xB9, 0xE5, 0x09, 0x86, 0x41, 0x74, 0x55, 0x86, 0x41, 0x74, 0x4C, 0x33, 0x86,
        0x41, 0x74, 0x05, 0x04, 0x41, 0x86, 0x41, 0x74, 0x3A, 0xE7, 0x09, 0x54, 0x02, 0x29, 0xE7,
        0x09, 0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00, 0x4F, 0x02, 0x20, 0x00, 0x4F, 0x01, 0x07,
        0x4D,
    ];
    /// `return g(a) + 70000;` — wide post-op immediate → reject.
    pub(crate) const GA_WIDEMOD: &[u8] = &[
        0x53, 0x53, 0x26, 0xE6, 0x09, 0x46, 0x2D, 0xE5, 0x09, // stmt start, fn push, formals: a
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
    pub(crate) const INT_TAILRET: &[u8] = &[
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
    pub(crate) const INT_PLUS0: &[u8] = &[
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
    pub(crate) const INT_ARGTAIL: &[u8] = &[
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
    pub(crate) const TWO_CALLS: &[u8] = &[
        0x53, 0x53, 0x26, 0xE4, 0x09, 0x46, // stmt start, fn push, formals: none
        0x4C, 0x4F, 0x11, 0x53, 0x26, 0xE3, 0x09, 0xBD, 0x82, 0x07, 0x03, 0x00, 0x80, 0x01, 0x10,
        0x00, 0x00, 0x4C, 0x4B, 0x26, 0xE3, 0x09, 0xBD, 0x82, 0x07, 0x03, 0x00, 0x80, 0x01, 0x10,
        0x00, 0x00, 0x4C, 0x4B, 0x3A, 0xE5, 0x09, 0x54, 0x02, 0x29, 0xE5, 0x09, 0x4F, 0x12, 0x47,
        0x54, 0x01, 0x54, 0x00, 0x4F, 0x02, 0x20, 0x00, 0x4F, 0x01, 0x06, 0x4D,
    ];
    /// `int f(int a){ g(); return a + 1; }` — a second statement follows the
    /// void call's `4C 4B` (a `B9` LOAD where the return plumbing must be) →
    /// reject (defect #2).
    pub(crate) const CALL_THEN_STMT: &[u8] = &[
        0x53, 0x53, 0x26, 0xE4, 0x09, 0x46, // stmt start, fn push, formals: none
        0x4C, 0x4F, 0x11, 0x53, 0x26, 0xE3, 0x09, 0xBD, 0x82, 0x07, 0x03, 0x00, 0x80, 0x01, 0x10,
        0x00, 0x00, 0x4C, 0x4B, 0xB9, 0xE4, 0x09, 0x86, 0x41, 0x74, 0x33, 0x86, 0x41, 0x74, 0x01,
        0x02, 0x41, 0x86, 0x41, 0x74, 0x3A, 0xE6, 0x09, 0x54, 0x02, 0x29, 0xE6, 0x09, 0x4F, 0x12,
        0x47, 0x54, 0x01, 0x54, 0x00, 0x4F, 0x02, 0x20, 0x00, 0x4F, 0x01, 0x05, 0x4D,
    ];
    /// `int f(int a){ return g(a + 1) + 1; }` — in-argument arithmetic AND a
    /// framed post-op: the arg region carries LOAD+LIT+ADD before `55` → reject
    /// (defect #3; a naive post-`55` search would mis-accept as framed g(a)+1).
    pub(crate) const ARGFRAMED_PLUSK: &[u8] = &[
        0x53, 0x53, 0x26, 0xE6, 0x09, 0x46, 0x2D, 0xE5, 0x09, // stmt start, fn push, formals: a
        0x4C, 0x4F, 0x11, 0x53, 0x26, 0xE4, 0x09, 0xBD, 0x86, 0x41, 0x74, 0x00, 0x80, 0x01, 0x10,
        0x00, 0x00, 0xB9, 0xE5, 0x09, 0x86, 0x41, 0x74, 0x33, 0x86, 0x41, 0x74, 0x01, 0x02, 0x55,
        0x86, 0x41, 0x74, 0x4C, 0x33, 0x86, 0x41, 0x74, 0x01, 0x02, 0x41, 0x86, 0x41, 0x74, 0x3A,
        0xE7, 0x09, 0x54, 0x02, 0x29, 0xE7, 0x09, 0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00, 0x4F,
        0x02, 0x20, 0x00, 0x4F, 0x01, 0x06, 0x4D,
    ];
    /// `int f(int a){ return g(a) + g(a + 1); }` — a SECOND call follows the
    /// first call-end where the framed post-op literal must be → reject
    /// (defect #4).
    pub(crate) const TWO_FRAMED_CALLS: &[u8] = &[
        0x53, 0x53, 0x26, 0xE6, 0x09, 0x46, 0x2D, 0xE5, 0x09, // stmt start, fn push, formals: a
        0x4C, 0x4F, 0x11, 0x53, 0x26, 0xE4, 0x09, 0xBD, 0x86, 0x41, 0x74, 0x00, 0x80, 0x01, 0x10,
        0x00, 0x00, 0xB9, 0xE5, 0x09, 0x86, 0x41, 0x74, 0x55, 0x86, 0x41, 0x74, 0x4C, 0x26, 0xE4,
        0x09, 0xBD, 0x86, 0x41, 0x74, 0x00, 0x80, 0x01, 0x10, 0x00, 0x00, 0xB9, 0xE5, 0x09, 0x86,
        0x41, 0x74, 0x33, 0x86, 0x41, 0x74, 0x01, 0x02, 0x55, 0x86, 0x41, 0x74, 0x4C, 0x02, 0x41,
        0x86, 0x41, 0x74, 0x3A, 0xE7, 0x09, 0x54, 0x02, 0x29, 0xE7, 0x09, 0x4F, 0x12, 0x47, 0x54,
        0x01, 0x54, 0x00, 0x4F, 0x02, 0x20, 0x00, 0x4F, 0x01, 0x05, 0x4D,
    ];
    /// `int f(int a){ return g(a) + 1 + 2; }` — a SECOND literal+ADD follows the
    /// framed post-op where the result-type must be → reject.
    pub(crate) const PLUS1PLUS2: &[u8] = &[
        0x53, 0x53, 0x26, 0xE6, 0x09, 0x46, 0x2D, 0xE5, 0x09, // stmt start, fn push, formals: a
        0x4C, 0x4F, 0x11, 0x53, 0x26, 0xE4, 0x09, 0xBD, 0x86, 0x41, 0x74, 0x00, 0x80, 0x01, 0x10,
        0x00, 0x00, 0xB9, 0xE5, 0x09, 0x86, 0x41, 0x74, 0x55, 0x86, 0x41, 0x74, 0x4C, 0x33, 0x86,
        0x41, 0x74, 0x01, 0x02, 0x33, 0x86, 0x41, 0x74, 0x02, 0x02, 0x41, 0x86, 0x41, 0x74, 0x3A,
        0xE7, 0x09, 0x54, 0x02, 0x29, 0xE7, 0x09, 0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00, 0x4F,
        0x02, 0x20, 0x00, 0x4F, 0x01, 0x03, 0x4D,
    ];
}
