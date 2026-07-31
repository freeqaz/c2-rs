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

mod bind;
mod body;
mod bundle;
mod census;
mod gl;
mod readers;
mod sy;

pub use self::body::{chain_form, Block, ChainForm, FP_SCRATCH};
pub use self::bundle::{
    is_empty_module, opt_word_mode, OptWordMode, OPT_WORD_O1, OPT_WORD_OX,
    OPT_WORD_SPECIAL_MEMBER,
};
pub use self::census::{FnCensus, FnVerdict, CENSUS_HEX_BACK, CENSUS_HEX_FWD};
pub use self::gl::{
    gl_symbol_conflicts, gl_symbol_index, label_counter, mangled_name, mangled_names, source_path,
};
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
    /// **Address of** a sub-object: pop a pointer, push the pointer `off` bytes
    /// further in — `return &s->m;`, `return &p->Base::m;`, `return s->arr;`.
    /// No memory is touched, which is the whole difference from
    /// [`IlOp::LoadInd`]: c2 emits one `addi rD, rBase, off`, and **nothing at
    /// all** when `off` is 0.
    ///
    /// Produced ONLY by [`try_parse_addr_leaf`], and only as the second and last
    /// op of a two-op stream `[Load(base), AddrOf { off }]`. It never appears in
    /// combination with arithmetic: an address that feeds an integer expression
    /// is a different construct (the address would have to be converted first)
    /// and no capture establishes its lowering.
    ///
    /// MEASURED (`work/bma/probes/p1.cpp`, `p2.cpp`, `p3.cpp`; the emitted words
    /// are read off the reference obj): `&s->b` at 4 is `addi r3,r3,4`, at 32764
    /// `addi r3,r3,32764`, at 32768 **`addis r3,r3,1 ; addi r3,r3,-32768`** —
    /// two instructions, so `off` is gated to a signed 16-bit displacement. The
    /// *pointee's* width is irrelevant: `char*`, `short*`, `int*`, `long long*`,
    /// `float*` and `double*` members all emit the same one `addi`.
    AddrOf { off: i32 },
    /// **Indirect store**: pop the value, pop the destination pointer, write
    /// `width` bytes `off` bytes in — `s->m = v;`, `p->Base::m = v;`,
    /// `s->arr[2] = v;`, `*p = v;`. IL production `32 <TYPE>` closed by the
    /// statement end `4B`, with the address built by the same designator +
    /// byte-offset-add run the load and address leaves use.
    ///
    /// Produced ONLY by [`try_parse_store_leaf`], and only as the **third and
    /// last** op of a three-op stream `[Load(base), Load(value)|Lit(k),
    /// StoreInd { off, width }]`. It never combines with arithmetic: a store
    /// whose value is computed puts the computation in the scratch register
    /// first, and a store that is one statement of several is a different body.
    ///
    /// MEASURED (`work/lf/probes/p1.cpp`, every word read off the reference obj
    /// at `/Ox /GS- /c`) — the width selects the opcode and nothing else does:
    ///
    /// ```text
    ///   width 1  ->  stb      width 2  ->  sth
    ///   width 4  ->  stw      width 8  ->  std (DS-form, off % 4 == 0)
    /// ```
    ///
    /// A **floating-point** value is not representable here; it is
    /// [`IlOp::StoreIndFp`], which carries a register rather than a token.
    StoreInd { off: i32, width: u8 },
    /// **Indirect store of a floating-point value** — `void f(S* s, float v)
    /// { s->f = v; }` — one `stfs`/`stfd` out of the FP argument file.
    ///
    /// MEASURED (`docs/CODEGEN_FP_ARGS.md` §3, every word read off a reference
    /// obj at `/O1 /GS- /c`); the width selects the opcode and the base register
    /// is the ordinary GPR argument:
    ///
    /// ```text
    ///   void s_f(S* s, float v)  { s->f = v; }        d0230004  stfs f1,4(r3)
    ///   void s_d(S* s, double v) { s->d = v; }        d8230008  stfd f1,8(r3)
    ///   void s_pf(float* p, float v) { *p = v; }      d0230000  stfs f1,0(r3)
    /// ```
    ///
    /// **`src` is a physical FP register and not a token**, which is the one
    /// design difference from [`IlOp::StoreInd`]. Everything else in this layer
    /// maps a token to a register by its index in [`IlFunction::params`], and for
    /// the FP file that mapping is *not* the index — it counts FP parameters
    /// alone and needs `.sy` to say which parameters those are. Resolving it in
    /// the parser, where `.sy` is in scope, keeps the FP numbering at exactly one
    /// site (`sy::fp_reg_of`) instead of giving codegen a second one to get wrong;
    /// `GAPS.md` §6 has two live wrong-bytes emits from that number existing in
    /// two places. The discriminating capture is
    /// `void s_two(S* s, float u, float v){ s->f = v; }` → `stfs f2,4(r3)`, and
    /// `void s_mix(S* s, float u, int k, float v){ s->f = v; }` → the same
    /// `stfs f2`, because the `int` advances the slot and not the FP file.
    StoreIndFp { off: i32, double: bool, src: u8 },
    /// **Indirect load of a floating-point value into the FP scratch register** —
    /// the value half of `d->f = s->f;`.
    ///
    /// The same IL production as [`IlOp::LoadInd`] (`<designator> 30 <TYPE>`), only
    /// with a `real`-class pointee, so it carries no register: the destination is
    /// always **f0**, which is what makes it different from [`IlOp::StoreIndFp`]'s
    /// `src`. MEASURED (`work/wsl/probe/p2.cpp`, read off the reference obj):
    ///
    /// ```text
    ///   void w_f(W* d, W* s) { d->f = s->f; }   c0040010 d0030010  lfs f0,16(r4) ; stfs f0,16(r3)
    ///   void w_g(W* d, W* s) { d->g = s->g; }   c8040018 d8030018  lfd f0,24(r4) ; stfd f0,24(r3)
    /// ```
    ///
    /// Produced ONLY by the store family's value position, and only as the third
    /// op of a four-op group `[Load(dbase), Load(sbase), LoadIndFp, StoreIndFp]`.
    /// It never reaches a return: an FP value that is *returned* goes to f1 and is
    /// [`super::body::shapes::leaf_float`]'s question, with its own captures.
    LoadIndFp { off: i32, double: bool },
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

/// **A single-argument floating-point tail call's argument marshalling** — the
/// whole of it.
///
/// `return g(x);` / `g(x);` where `x` is an FP formal is at most one instruction
/// plus the branch, and which instruction depends on two facts: where the value
/// is in the FP file, and whether the callee's formal is the narrower width.
/// [`IlFunction::params`] carries the first (the FP formals alone, in FP order —
/// entry `n` is `f(n+1)`), this record the second.
///
/// Set together with [`IlFunction::tail_call`], which is what makes the branch
/// and its REL24 come out of the shared tail-call path rather than a second copy.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FpTail {
    /// The argument formal's token; its index in [`IlFunction::params`] is its
    /// FP register number minus one.
    pub arg: u32,
    /// The callee's formal is `float` where the source is `double`, so the move
    /// is an `frsp f1,fS` — **fused**, not `fmr f1,fS` followed by
    /// `frsp f1,f1`. Captured: `float n2(double a, double b){ return g1f(b); }`
    /// is the single word `fc201018`.
    pub narrowing: bool,
}

/// One call of a [`CallSeq`], with its callee resolved and its argument setup in
/// whichever of the two forms the shared marshalling locator produced.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SeqCall {
    /// The callee's mangled name (from `.gl`), e.g. `?g1@@YAXXZ`.
    pub callee: String,
    /// The argument operand stream, computed into r3 — empty for a nullary call,
    /// `[Load(t)]` for a passthrough, `[Load, Lit, Add]` for `g(a + 1)`, `[Lit]`
    /// for `g(7)`. Mutually exclusive with [`Self::arg_sources`].
    pub arg_ops: Vec<IlOp>,
    /// A 2+-argument call's register permutation over the formals, exactly as
    /// [`IlFunction::arg_sources`] carries it for a multi-argument tail call.
    pub arg_sources: Option<Vec<usize>>,
    /// **WCL** — set when this call is a **chain link**: `p->a()->b(k)`'s outer
    /// call, whose receiver is the previous call's result and is therefore
    /// already in r3. Its explicit arguments start at argument slot **1** and
    /// are listed here in ascending slot order. `None` for every other call,
    /// including a chain's innermost one. See `c2_il::func::body::SlotArg`'s
    /// internal twin and `docs/rungs/2026-07-31-chain-link-arg.md`.
    pub link_args: Option<Vec<SlotArg>>,
}

/// **The argument slot a chain link's explicit arguments start at.** Its
/// receiver is the previous call's result, which a `bl` has already left in r3 =
/// argument slot 0, so the first explicit argument goes to r4.
///
/// One fact, so one constant, and it lives in *this* crate although only the
/// backend reads a register out of it: the IL parser uses it to bound the slot
/// list and the emitter uses it to pick the register, and those two agreeing is
/// the whole reason the census cannot claim a body the gate declines. A second
/// copy on the codegen side is the `docs/GAPS.md` §6 #9 shape exactly.
///
/// Measured at one and **only** one: no capture in this family separates "the
/// receiver occupies slot 0" from "the first explicit argument goes to r4", so a
/// per-call slot *number* would be a degree of freedom nothing has graded.
pub const LINK_FIRST_SLOT: usize = 1;

/// One explicit argument of a chain link ([`SeqCall::link_args`]) — the resolved
/// twin of `c2_il::func::body::SlotArg`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SlotArg {
    /// A formal, by index into [`IlFunction::params`]; it is in a callee-saved
    /// GPR by the time the link runs ([`CallSeq::saved`] says which).
    Formal(usize),
    /// A literal — `li r<slot>,k`, which costs no callee-saved register.
    Lit(i32),
}

/// What a [`CallSeq`] body does after its last call. See
/// `c2_il::func::body::SeqTail`, of which this is the resolved twin.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SeqTail {
    /// Nothing — the body returns void.
    Void,
    /// The last call's result is returned, plus `add_k` (`addi r3,r3,k`, elided
    /// when 0).
    CallValue { add_k: i32 },
    /// `return <literal>;` — one `li r3,k`.
    Lit(i32),
    /// **WCO** — the last call's pointer result is **read through**:
    /// `return p->a()->b()->m;` is one `lwz r3,off(r3)` after the last `bl`.
    ///
    /// Deliberately a sibling of [`SeqTail::CallValue`] and not a flag on it.
    /// `&p->a()->b()->m` is `addi r3,r3,off` — the identical designator with the
    /// `30` load absent — and `CallValue { add_k: off }` already spells that,
    /// including the offset-0 fold to no instruction. The load form does **not**
    /// fold at offset 0: `lwz r3,0(r3)` is emitted (measured, `c_off0` in
    /// `work/WCO/probe/p1.cpp`), which is the one place the two disagree.
    CallLoad { off: i32 },
    /// **WFL** — the same read-through whose member is **floating point**:
    /// `float f(O* p){ return p->a()->b()->m; }` is one `lfs f1,off(r3)`
    /// (`c0230004`), and a `double` member `lfd f1,off(r3)` (`c8230010`).
    ///
    /// A sibling of [`SeqTail::CallLoad`] rather than a width flag on it, for the
    /// reason `CallLoad` is a sibling of `CallValue`: the value lands in the
    /// **other register file**. Two consequences no integer tail has —
    /// [`IlFunction::touches_floating_point`] must be true of the body, which is
    /// what puts `_fltused` in the obj, and the lowering lives beside the FP
    /// leaf's register model rather than beside the call sequence's.
    ///
    /// `double` is the **loaded** width. `lfs` loads and converts in one
    /// instruction, so a `float` member returned as a `double` is byte-identical
    /// to the unpromoted body (measured) and the opcode follows the member; the
    /// reverse — a `double` member narrowed to a `float` result — is
    /// `lfd f0 ; frsp f1,f0`, two words into a scratch register, and the parser
    /// refuses it.
    CallLoadFp { off: i32, double: bool },
    /// **WCB/WCR — the two calls' results compared**, materialized to a 0/1 in
    /// r3: `return a->m() <rel> b->n();`.
    ///
    /// The **first** call's result is itself live across the second `bl`, so it
    /// takes a callee-saved register of its own — the one *after* the saved
    /// formals ([`CallSeq::saved`]), exactly as
    /// `docs/CODEGEN_FRAMED_CALLS.md` §3.1 records ("call results take the next
    /// descending register after the parameters"). That is the whole reason this
    /// tail exists as a variant rather than as a post-op: it changes the frame
    /// class, and [`CallSeq::saved_gprs`] is where that is spelled.
    ///
    /// `lhs_first` says whether the source's **left** operand is the call emitted
    /// first. It is not always true: c2 orders the two calls by the order c1xx
    /// NUMBERED their receivers — `this` last, although it is `params[0]` — and
    /// the spine's two operands are (the saved first result, r3), so the operand
    /// roles and the call order are two independent facts. See
    /// `docs/rungs/2026-07-31-cmp-two-calls.md`.
    Cmp { cmp: SeqCmp, lhs_first: bool },
    /// **WEC** — the body returns a **callee-saved formal**: one `mr r3, rSaved`
    /// after the last `bl`. `param` indexes [`IlFunction::params`], and that
    /// formal must be in [`CallSeq::saved`] (the parser puts it there; the
    /// emitter re-derives the register and refuses if it is not).
    ///
    /// Its one producer is the **empty constructor that delegates to one
    /// destructible base**: an MSVC constructor hands `this` back in r3, `this`
    /// is live across the base constructor's `bl`, and the whole body is
    /// therefore `mr r31,r3 ; bl ?B ; mr r3,r31` inside the shipped 1-saved-GPR
    /// frame. MEASURED at the workload's own `/O1 /Oi /EHsc`, 48 B, `F = 96`,
    /// **byte-identical across four source forms** (`work/WEC/probe/p2.obj`):
    /// a base with a destructor, a base without one, a constructor with an
    /// unused formal of its own, and a constructor that forwards its formal to
    /// the base. Only the last of those four is refused by the parser, and for a
    /// reason that has nothing to do with this tail.
    ///
    /// **It is not [`SeqTail::CallValue`] with `add_k: 0`.** That tail returns
    /// what the last `bl` left in r3; this one *overwrites* r3 with a value that
    /// predates the call. Folding it into `CallValue` would emit nothing at all
    /// here and return the base constructor's result — which happens to be the
    /// same pointer on this ABI, so the obj would still link and still run, and
    /// the four missing bytes would show up only as a length mismatch.
    SavedFormal { param: usize },
}

/// **Which comparison** a [`SeqTail::Cmp`] performs — and, for the order
/// relations only, the operand signedness.
///
/// The signedness lives *inside* the `Order` variant rather than beside the
/// relation because `==` does not read it: the `sub`/`cntlzw`/`rlwinm` zero fold
/// is byte-identical for `int` and `unsigned` operands, so a shared `signed`
/// field would be a fact carried where nothing consumes it — `docs/GAPS.md` §6's
/// recurring shape, and here it would also mean the shipped `==` production
/// acquiring an operand-type gate it has never needed and losing census to it.
///
/// The **result** type (`int` against `bool`) is deliberately absent, and that is
/// measured rather than assumed. `docs/CMP_PRODUCES_A_VALUE.md` reading 1 records
/// two of 24 literal-comparison cells where a `bool` result is two words longer;
/// over two call results the divergence is signed `>=`/`<=` (which this enum does
/// not admit) and **`>`, `<`, `==` and `!=` are byte-identical in `int`, `bool`
/// and `unsigned`**, in all four modes — `scripts/gt_cmp_rr.py`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SeqCmp {
    /// `==`: `sub r11,<rhs>,<lhs>` then the zero fold. Signedness-free.
    Eq,
    /// `>` (`greater`) or `<`, over two operands of the same signedness.
    ///
    /// One spine with the operand roles swapped, in both signednesses: `a < b`
    /// is `a > b` read right-to-left, exactly as the comparison *leaf*'s
    /// `Rel::Lt` arm is its `Rel::Gt` arm with two operand fields exchanged.
    Order { greater: bool, signed: bool },
}

/// **A framed many-call body** (#35 step 2): a sequence of statement-position
/// calls, with the tail the body ends on.
///
/// Class A ([`Self::saved`] empty) has no value live across any call, so nothing
/// is callee-saved and the frame is the shipped 96-byte one. Class B saves one or
/// two GPRs inline (`std`/`ld` at `-16(r1)` and `-24(r1)`, frame 96 or 112) for
/// the formals that have to survive a `bl`.
///
/// The whole body is `prologue · (setup_i · bl callee_i)* · tail · epilogue`, and
/// every `bl` is its own REL24 site — which is why `c2_core::coff::Function`
/// carries a *list* of calls rather than an `Option`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CallSeq {
    /// Every call in `.text` order; at least two, or one with a non-void tail
    /// (a lone statement call with nothing after it is tail-called instead).
    pub calls: Vec<SeqCall>,
    pub tail: SeqTail,
    /// **Class B**: the parameter indices c2 copies into callee-saved GPRs
    /// because their value has to survive a `bl`, taking `r31`, `r30`, … in
    /// this order. Empty for Class A, at most
    /// two entries — three or more is the `__savegprlr_N` helper class and the
    /// IL parser refuses it (`callseq-three-plus-saved`).
    pub saved: Vec<usize>,
}

impl CallSeq {
    /// **How many callee-saved GPRs the prologue must `std`** — which is not
    /// `saved.len()`.
    ///
    /// [`Self::saved`] counts the *formals* that have to survive a `bl`. A tail
    /// that consumes a call's result across a later call needs one more register,
    /// and c2 takes it from the same descending file immediately after the saved
    /// formals (`docs/CODEGEN_FRAMED_CALLS.md` §3.1). Keeping the two apart in
    /// the IL and summing them **here, once** is deliberate: `FrameLayout`,
    /// `call_seq_parts` and the `MAX_INLINE_SAVED_GPRS` gate all need the total,
    /// and three copies of `saved.len() + 1` is exactly the one-fact-two-
    /// implementations drift `docs/GAPS.md` §6 records.
    pub fn saved_gprs(&self) -> usize {
        self.saved.len() + usize::from(self.tail.saves_a_call_result())
    }
}

impl SeqTail {
    /// Whether this tail keeps an **earlier call's result** live across a later
    /// `bl`, which costs one callee-saved GPR beyond the saved formals.
    pub fn saves_a_call_result(self) -> bool {
        matches!(self, SeqTail::Cmp { .. })
    }

    /// **Extra compiler-label counter slots this tail consumes AHEAD of the
    /// function's own `$M`/`$M`/`$T` triple.**
    ///
    /// Measured seed-free and in-TU by `gt_label_stride.py`'s method, with the
    /// in-TU `a2` anchor control holding on every row
    /// (`scripts/gt_cmp_rr.py --stride`, `/Ox /Gy`, `/O1 /Gy` and packed `/Ox`):
    ///
    /// ```text
    ///                                          /Gy      packed
    ///   two calls, arithmetic tail            5    0    4    0
    ///   two calls, cmp `==`                   5    0    4    0
    ///   two calls, cmp UNSIGNED any relation  5    0    4    0
    ///   two calls, cmp SIGNED  `>` `<`        7    2    6    2
    ///                                       stride lead stride lead
    /// ```
    ///
    /// This is [`CompareLeaf::label_slots`]'s existing 1-or-3 table re-expressed
    /// as a surcharge, and it lands on the same **signed order relation** set —
    /// with the leaf's "`k == 0`" escape absent, because a register operand is
    /// not a zero literal. Placed ahead of the triple, the same way
    /// `docs/CODEGEN_FRAMED_CALLS.md` §4.4 records for the
    /// `__savegprlr_N`/`__restgprlr_N` pair (`docs/LABEL_COUNTER.md` §1.1).
    ///
    /// **The result type does NOT enter the stride** — `int` and `bool` give the
    /// same number on every row, which is why the counter cannot be used as a
    /// proxy for the spine.
    pub fn label_lead(self) -> u32 {
        match self {
            SeqTail::Cmp { cmp: SeqCmp::Order { signed: true, .. }, .. } => 2,
            _ => 0,
        }
    }
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

impl CompareLeaf {
    /// The census `ctx` of a comparison leaf that decodes cleanly but
    /// `c2_core::codegen::compare_leaf_text` would decline, or `None` when it is
    /// in class.
    ///
    /// The three clauses are pure functions of the decoded leaf, and they lived
    /// only in codegen — so `int f(unsigned a){ return a == 4294967295u; }`
    /// censused in class and the port refused it. Codegen keeps all three as
    /// backstops; this is the primary gate, so the census and the emitter agree
    /// (`docs/CODEGEN_W6_O1.md` has the byte evidence for each).
    pub fn out_of_class_ctx(&self) -> Option<&'static str> {
        // A zero literal takes the folded spines, which have no immediate at all.
        if self.k == 0 {
            return None;
        }
        // A wide literal needs `lis`+`ori` materialization and the extra temp slot
        // it consumes; not characterized.
        let Ok(k16) = i16::try_from(self.k) else {
            return Some("cmp-out-of-class-wide-lit");
        };
        // Only `==`/`!=` form `a - k` as `addi r11,a,-k`.
        if !matches!(self.rel, Rel::Eq | Rel::Ne) {
            return None;
        }
        // Against a large UNSIGNED literal c2 materializes the constant and
        // subtracts instead — one instruction more. The carry spines gate on raw
        // SIMM16 encodability and are unaffected, which is why this is not the
        // same predicate.
        if !self.signed && self.k < 0 {
            return Some("cmp-out-of-class-unsigned-wide-lit");
        }
        // `-(-32768)` does not fit the immediate.
        if k16.checked_neg().is_none() {
            return Some("cmp-out-of-class-lit-i16-min");
        }
        None
    }

    /// How many **compiler-label counter slots** this comparison leaf consumes.
    ///
    /// The counter is a per-TU running number that every function advances,
    /// whether or not it emits a label (`docs/OBJ_GY_SHAPES.md` §3.5/§3.6), so a
    /// framed function sharing a TU with a class whose stride is wrong gets `$M`
    /// numbers that are low by the error — six wrong bytes in an obj that still
    /// links. Every class the port emits consumes 1 **except** the comparison
    /// leaf, which is 1 for some relations and 3 for others.
    ///
    /// Measured over the whole 60-point grid of `<relation> × <literal 0, ±5,
    /// i16::MAX, i16::MIN, wide> × <signed, unsigned>`, each row compiled as
    /// `<leaf> ; int F(int a){return g(a)+1;}` with `F`'s first label
    /// differenced against the seed at `.gl+7` + 9 (so the stride is measured,
    /// not fitted against an unknown seed — `docs/OBJ_GY_SHAPES.md` §3.4's
    /// cautionary tale):
    ///
    /// ```text
    ///   ==, !=            1   every literal, both signednesses
    ///   unsigned operand  1   every relation, every literal
    ///   signed <  k == 0  1     signed >= k == 0  1
    ///   signed <  k != 0  3     signed >= k != 0  3
    ///   signed >  anything 3    signed <= anything 3
    /// ```
    ///
    /// The 1-block is exactly the set that lowers to a sign-bit extraction or a
    /// carry idiom with no label pair; the 3-block is the general relational
    /// spine. `OBJ_GY_SHAPES.md` §3.6 previously recorded only "`a==b` and `a<0`
    /// consume 1" and the gate keyed on "is this a comparison leaf" instead, so
    /// every comparison leaf was refused beside a framed function.
    pub fn label_slots(&self) -> u32 {
        match self.rel {
            Rel::Eq | Rel::Ne => 1,
            _ if !self.signed => 1,
            Rel::Lt | Rel::Ge if self.k == 0 => 1,
            _ => 3,
        }
    }
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
    /// If this function is a **Class A many-call body** (`void f(){ g1(); g2(); }`
    /// and friends), the call list and what follows them. Framed like
    /// [`Self::framed_call`] — same 96-byte frame, same `.pdata` record, same
    /// label stride — but with one REL24 site per call instead of one.
    /// [`Self::params`] carries the formals the first call's arguments index.
    pub call_seq: Option<CallSeq>,
    /// If this function is a **comparison leaf** (`return a <rel> k;`, W6), the
    /// decoded comparison. Mutually exclusive with the other body kinds.
    pub compare: Option<CompareLeaf>,
    /// If this function is a **W13a floating-point leaf**, whether it is double
    /// precision. Mutually exclusive with the other body kinds.
    pub float_leaf: Option<bool>,
    /// If this function is a **single-argument floating-point tail call**, its
    /// argument marshalling. Set together with [`Self::tail_call`], and then
    /// [`Self::params`] holds the FP formals **alone**, in FP-file order.
    ///
    /// Deliberately not folded into [`Self::float_leaf`], whose two readers
    /// ([`Self::label_slots`] and `float_leaf_text`) both want "this body is a
    /// W13 arithmetic chain" and neither wants "this body touches FP" — that
    /// conflation was the tenth, eleventh and twelfth wrong-bytes emits
    /// (`docs/CODEGEN_FP_ARGS.md` §4, §4.1). This shape's stride is 1 like any
    /// other tail call's; only [`Self::touches_floating_point`] is true of it.
    pub fp_tail: Option<FpTail>,
    /// A **multi-argument** floating-point tail call's argument permutation, W34.
    /// `Some(sources)` means `return g(x1, …, xn)` with `n >= 2` and every
    /// argument a bare FP formal: `sources[i]` is the index into [`Self::params`]
    /// — which then holds the FP formals **alone**, in FP-file order — of the
    /// value FP argument register `f(i+1)` wants. Set together with
    /// [`Self::tail_call`], and then [`Self::ops`] is empty.
    ///
    /// Separate from [`Self::arg_sources`] because that one indexes the **GPR**
    /// argument registers `r(3+i)`. A call that needed both would need the two
    /// files' interleaved schedule, which `docs/CODEGEN_FP_ARGS.md` §1.1 records
    /// as uncharacterized and the parser refuses.
    pub fp_arg_sources: Option<Vec<usize>>,
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
    /// **This body carries an EH-enabled `5C`/`5D`/`5E` object-goes-live marker
    /// and is on the CHEAP side of `docs/EH_RECORDS.md` §6's boundary** —
    /// `eh-bare`: one object live, one statement, nothing else. c2 emits an
    /// ordinary function for it: no `__CxxFrameHandler` prefix, no funclet, no
    /// EH `.rdata`, and the function symbol stays at `Value = 0` (against
    /// `Value = 8` on every EH function — §8.1, and the reason this flag says
    /// *cheap* rather than *EH*).
    ///
    /// **It is not free: it costs one label-counter slot.** MEASURED
    /// (`docs/EH_RECORDS.md` §8.5d for the framed row, and
    /// `scripts/gt_label_stride.py`'s `eh-bare-*` rows here for the two the port
    /// actually emits), seed-free and in-TU, with the anchor control holding on
    /// every row:
    ///
    /// ```text
    ///                                 /EHsc            no /EH
    ///                             extra  stride     extra  stride
    ///   eh-bare-dtor      leaf        -      2          -      1
    ///   eh-bare-dtor-led  leaf        -      2          -      1
    ///   eh-bare-dtor-adj  leaf        -      2          -      1
    ///   eh-bare-dtor-deleg leaf       -      2          -      1
    ///   eh-bare-ctor      framed      1      6          0      5
    ///   eh-bare-ctor-led  framed      1      6
    ///   eh-none-ctor-ctl  framed      0      5          0      5   <- CONTROL
    /// ```
    ///
    /// Four things that table settles and that nothing else here could:
    ///
    /// * **The `+1` is per FUNCTION, not per TU** — the `-led` rows charge it
    ///   again behind an `eh-bare` function that already paid it, in both the
    ///   leaf and the framed family. (`__CxxFrameHandler`'s own `+1` *is* per
    ///   TU, but an `eh-bare` function does not reference it at all.)
    /// * **It applies to a LEAF**, which `docs/EH_RECORDS.md` §8.5d's single
    ///   framed row could not have shown — and the three `empty-dtor-*` shapes
    ///   the port has emitted since W14/W15 are exactly that leaf. Before this
    ///   field they consumed 1, and a two-function TU of one such destructor
    ///   ahead of an ordinary framed call at the workload's own flags was a
    ///   **live `mismatch`** — `work/WEC/live/t1.cpp`, first divergence at file
    ///   offset 1039, both objs 1221 B: the `$M`/`$T` names, six wrong bytes in
    ///   an obj that still links.
    /// * **It is keyed on `/EHsc`, not on the shape**, and the IL says which:
    ///   `/EH…` clears bit `0x10` in both the `5C` statement trailer's flag and
    ///   the `5D`/`5E` count trailer's, so `(0x11, 0x31)` is the no-EH profile
    ///   and `(0x01, 0x21)` the workload's. This flag is set from **that byte**,
    ///   never from the compiler flags — the IL bundle does not record argv, and
    ///   a stride keyed on a flag the emitter cannot see is the failure this
    ///   whole three-valued counter exists to prevent.
    /// * **`eh-none-ctor-ctl` is the separating control**: the identical body
    ///   over a base with no destructor prints 5 at `/EHsc`. Without it the
    ///   `+1` could just as well have been a property of the constructor shape.
    ///
    /// Widened only from `docs/LABEL_COUNTER.md` §1.1's measured surcharge table
    /// — never from §2.1's retracted "one slot per TU-level external" reading,
    /// which would price this at **0** (an `eh-bare` function mints no external
    /// whatsoever: see the `minted` column, 1 for the leaf and 5 for the framed
    /// row, unchanged from their non-EH controls).
    ///
    /// # What this flag does NOT claim
    ///
    /// It is **not** a reading of `docs/EH_RECORDS.md` §6/§7's cheap/EH
    /// predicate, and it must not be widened by one. That predicate — *"one
    /// sub-object statement and nothing else"* — is refuted: the boundary is
    /// `maxState >= 1` over the distinct sets of live destructible objects
    /// observed at an outbound control transfer, and `int P(int a){ SE s;
    /// return a+1; }` has another statement beside the object and is still
    /// cheap. So §7's `eh-bare` count is a **lower** bound and its
    /// `eh-plus-stmt` an upper one.
    ///
    /// None of that reaches this field, because this field is not set from the
    /// axis. It is set by the two *grammars* that produce it — the generated
    /// empty destructor and the empty base-delegating constructor — from the
    /// trailer byte in their own bodies, and the `+1` is measured on exactly
    /// those four shapes. Both grammars require the count trailer to be `01`, so
    /// no body that reaches this flag can carry a second tracked object; the
    /// question "is the surcharge per function or per state" cannot arise inside
    /// the class, and is **NOT MEASURED** outside it.
    pub eh_bare: bool,
    /// **Names the IL references as an EH unwind action and the obj does NOT
    /// carry.** The base destructor a constructor would run if a later statement
    /// threw: it is in `.gl`, it is bound by a `26` push in the body, and — on
    /// the cheap side, where there is no funclet — c2 emits no `bl`, no
    /// relocation and **no symbol** for it. MEASURED: `work/WEC/probe/p2.obj` at
    /// `/O1 /Oi /EHsc` has one REL24 per constructor, to the base *constructor*,
    /// and no `??1B1@@QAA@XZ` anywhere in its symbol table.
    ///
    /// It exists for one consumer — the TU-level gate that refuses a bundle
    /// whose `.gl` has a name no record and no callee claims. That gate is right
    /// in general (an unclaimed name is usually a symbol the port would omit)
    /// and wrong here, so the exception is *named* rather than the gate
    /// loosened: this list accounts for the name and contributes nothing to
    /// [`Self::callees`], which is what the emitter reads.
    pub eh_unwind_callees: Vec<String>,
}

impl IlFunction {
    /// Everything a body shape does **not** discriminate on: provenance, and
    /// "no shape" for every parallel option.
    ///
    /// `shape_to_function`'s twelve arms each set the two or three fields they
    /// own and `None`/`false`/empty for the other nine — so before this
    /// existed, adding one field meant editing twelve arms, which is how a
    /// concurrent branch's new arm silently missed `call_seq`
    /// (`docs/ARCHITECTURE_SEAMS.md` §1, class 4, and the 231-line `bundle.rs`
    /// conflict). With `..IlFunction::base(…)` a new field is one edit here.
    ///
    /// **The trade-off, stated:** with struct-update syntax the compiler no
    /// longer forces every arm to consider a new field. That is acceptable
    /// *only* because for a shape-discriminant field the correct value in every
    /// arm that does not own it **is** the default — today's arms say so
    /// explicitly eleven times over. **If a future field ever has a non-default
    /// correct value for some shape, it must NOT go through `base()`**: give it
    /// to the arms that own it explicitly, or the default becomes a silent
    /// wrong answer that only the census/gate cross-check would catch.
    pub(crate) fn base(name: &str, src: &Option<String>) -> IlFunction {
        IlFunction {
            mangled_name: name.to_string(),
            source_path: src.clone(),
            params: Vec::new(),
            ops: Vec::new(),
            tail_call: None,
            framed_call: None,
            call_seq: None,
            compare: None,
            float_leaf: None,
            fp_tail: None,
            fp_arg_sources: None,
            arg_sources: None,
            empty_body: false,
            eh_bare: false,
            eh_unwind_callees: Vec::new(),
        }
    }

    /// How many **compiler-label counter slots** this function consumes, for the
    /// classes whose stride has been measured — `None` when it has not been.
    ///
    /// The counter is seeded from `.gl` and advanced once per function in `.text`
    /// order (`c2_core::coff::plan_labels`), so a framed function's `$M`/`$T`
    /// numbers depend on every function ahead of it, including ones that emit no
    /// label at all. A stride that is wrong by one is six wrong bytes in an obj
    /// that still links, which is why this is three-valued: an unmeasured class
    /// beside a framed function is refused, never guessed.
    ///
    /// **This is the PER-FUNCTION part of the stride only.** The TU-level
    /// surcharges — `_fltused`'s `+1` for the TU's first FP-touching function —
    /// are `c2_core::coff::plan_labels`'s, because they are questions about the
    /// whole function list and no per-function method can answer them.
    ///
    /// Measured strides (`docs/LABEL_COUNTER.md` §1, `docs/OBJ_GY_SHAPES.md`
    /// §3.6): every integer leaf, tail call, empty body, indirect load, address
    /// leaf **and constant-free floating-point leaf** consume **1**; a framed
    /// call **4** packed / **5** under `/Gy`; a comparison leaf 1 or 3 by
    /// relation ([`CompareLeaf::label_slots`]).
    ///
    /// **A floating-point leaf is NOT 2 here, and the older "2, or 4 with one
    /// pooled constant and 6 with two" reading of this list is retracted.** Those
    /// three numbers are whole-TU readings of a leaf that is itself the TU's
    /// first FP function: `leaf-float` measures 2 but `leaf-float-led` — the same
    /// leaf with `_fltused` already charged to a function ahead of it — measures
    /// **1**. What a pooled constant adds is **+2 per newly pooled
    /// `(bits,width)`**, which is again a per-TU question (`const1-dup-led`
    /// measures **0** for a constant an earlier function pooled), so a leaf that
    /// pools one is `None` here.
    ///
    /// **A Class A many-call body is the same 5 / 4.** Measured with a two-function
    /// TU of two-call bodies (`void f1(){g1();g2();} void f2(){g3();g1();}`): under
    /// `/Gy` `f1` is `$M2553/$M2554/$T2555` and `f2` `$M2558/$M2559/$T2560` — a
    /// stride of 5 against a `.gl+7` seed of 2538 (`2538 + 9 + 3·2 = 2553`); packed
    /// the same TU gives 2547 and 2551, a stride of 4. A leaf ahead of a two-call
    /// framed function still costs 1. So the call *count* does not enter the
    /// counter; framedness does.
    ///
    /// **The framed `/Gy` value is 5 only for a frame with no save/restore
    /// helper.** A framed function that uses the `__savegprlr_N`/`__restgprlr_N`
    /// pair consumes **two extra slots, allocated before its own `$M` pair**, so
    /// its stride is 7 and its first label is `cur + 2`
    /// (`docs/CODEGEN_FRAMED_CALLS.md` §4.4, seven witnesses). That is latent
    /// rather than live: `c2_core::codegen::FrameLayout` refuses every frame
    /// needing a helper, so the port emits only the no-helper class this 5
    /// describes. It becomes a wrong-bytes emit the moment #35 step 2 admits a
    /// framed function with ≥3 saved GPRs — the stride correction and the helper
    /// codegen have to land together.
    /// Whether this function's body **touches floating point** — which is what
    /// makes the obj carry the undefined external `_fltused`, not whether the
    /// function is a W13 float leaf.
    ///
    /// Those were one field until the FP store leaf separated them, and the
    /// separation is `docs/GAPS.md` §6's shape once more: `is_float` meant both
    /// "this body does FP arithmetic, so its label stride is 2" and "this
    /// translation unit needs the CRT's float-support hook", and every function
    /// that had ever set it satisfied both. `void f(S* s, float v){ s->f = v; }`
    /// satisfies only the second — it is a store leaf, stride 1 — and the port
    /// emitted an obj **one symbol short**, `Port=Mismatch @ offset 12` (the COFF
    /// header's `NumberOfSymbols`) on all fourteen positive cases at once.
    ///
    /// MEASURED, including the ordering: `_fltused` follows the first
    /// FP-*touching* function's symbol group, whatever kind it is. A TU of
    /// `int; fp-store; int; fp-store` puts it after the second function, and one
    /// of `fp-store; int; float-leaf` puts it after the **first**, ahead of the
    /// leaf (`docs/CODEGEN_FP_ARGS.md` §4).
    /// The **FP tail call** is the third producer, and it is the third kind of
    /// body that satisfies this and not "is a float leaf": `float f(float a,
    /// float b){ return g(b); }` is one `fmr` and a branch, stride 1, and its obj
    /// carries `_fltused` — verified, and placed by the same rule, immediately
    /// after the first FP-touching function's symbol group.
    /// **WFL is the fourth producer, and it is the first FRAMED one.** A
    /// `CallSeq` whose tail is [`SeqTail::CallLoadFp`] — `float f(O* p){ return
    /// p->a()->b()->m; }` — is a 40-byte framed body whose only FP is one `lfs`,
    /// and its obj carries `_fltused`. MEASURED (`work/WFL/probe/p1.obj`): the
    /// symbol lands immediately after the first FP-touching function's
    /// **complete** framed group — `.text` aux, the function, `$M`, `$M`, the
    /// `.pdata` aux, `$T` — which is the same "after the first FP-touching
    /// function's symbol group" rule, applied to a group that is now six symbols
    /// rather than three. The **label stride does not move**: `$M2646/$M2647/
    /// $T2648` then `$M2651/…` is the ordinary framed 5. FP costs a symbol here
    /// and not a label slot.
    ///
    /// This enumerates **shapes**, and W36's lesson is that the shape that gets
    /// missed is the one that is FP-touching without being FP-*shaped* — there,
    /// a callee's FP return type on an otherwise integer body. A `CallSeq` is
    /// integer-shaped in every field but its tail, so a producer written as
    /// "does the body have FP ops" would miss this one exactly the way that one
    /// was missed. The reader is `c2_core::coff::Function::is_float`, and the
    /// failure mode is an obj one symbol short — `Port=Mismatch @ offset 12`, the
    /// COFF header's `NumberOfSymbols`, on every positive case at once.
    pub fn touches_floating_point(&self) -> bool {
        self.float_leaf.is_some()
            || self.fp_tail.is_some()
            || self.fp_arg_sources.is_some()
            || self
                .call_seq
                .as_ref()
                .is_some_and(|s| matches!(s.tail, SeqTail::CallLoadFp { .. }))
            || self
                .ops
                .iter()
                .any(|o| matches!(o, IlOp::StoreIndFp { .. }))
    }

    /// True iff this function establishes a **stack frame** — it gets a `.pdata`
    /// record, a `$M`/`$M`/`$T` label triple, and the framed label stride.
    ///
    /// One predicate, asked by every TU-level gate that cares, so adding a framed
    /// shape cannot leave one of them behind. Both framed shapes are non-leaf
    /// calls whose result (or whose successor statement) outlives the `bl`.
    pub fn is_framed(&self) -> bool {
        self.framed_call.is_some() || self.call_seq.is_some()
    }

    /// **Label-counter slots this function takes BEFORE its own `$M` triple.**
    ///
    /// Zero for every class the port emitted before WCR. The two-call comparator
    /// with a **signed** `>`/`<` takes 2, measured on the grid
    /// [`SeqTail::label_lead`] tabulates. Split out from
    /// [`Self::label_slots`] because `c2_core::coff::plan_labels` needs the two
    /// numbers separately: the lead moves the function's own triple *and* every
    /// later function's, and the total moves only the later ones.
    /// **The `eh-bare` `+1` is a LEAD, and that is measured, not assumed.**
    /// `gt_label_stride.py`'s `eh-bare-ctor` row prints `extra 1` beside
    /// `stride 6`, i.e. the slot is taken before the function's own `$M` pair
    /// and moves this function's labels as well as every later one's — the same
    /// placement `docs/LABEL_COUNTER.md` §1.1 records for every other surcharge
    /// in the table. Moving it after the triple would leave every later
    /// function right and this one's `$M`/`$M`/`$T` low by one.
    ///
    /// A leaf has no triple, so lead and total are the same number for it; it
    /// is still added here rather than in [`Self::label_slots`]'s leaf arm so
    /// that `c2_core::coff::plan_labels` — which adds the lead before looking at
    /// the frame at all — charges both kinds through one path.
    pub fn label_lead(&self) -> u32 {
        self.call_seq.as_ref().map_or(0, |s| s.tail.label_lead()) + u32::from(self.eh_bare)
    }

    /// Every external this function calls, in **first-reference order** — which is
    /// the order the symbol table's per-function region is built from (reversed;
    /// `docs/OBJ_GY_SHAPES.md` §3.3). Duplicates are kept: a body may call the same
    /// callee twice and each site needs its own REL24, while the *symbol* is
    /// emitted once.
    pub fn callees(&self) -> impl Iterator<Item = &str> {
        self.tail_call
            .as_deref()
            .into_iter()
            .chain(self.framed_call.as_ref().map(|c| c.callee.as_str()))
            .chain(
                self.call_seq
                    .iter()
                    .flat_map(|s| s.calls.iter().map(|c| c.callee.as_str())),
            )
    }

    pub fn label_slots(&self, fn_level_linking: bool) -> Option<u32> {
        if self.framed_call.is_some() || self.call_seq.is_some() {
            return Some(self.label_lead() + if fn_level_linking { 5 } else { 4 });
        }
        if let Some(c) = &self.compare {
            return Some(self.label_lead() + c.label_slots());
        }
        // **A CONSTANT-FREE FP leaf consumes 1, like any other leaf.** This
        // used to return `None` for every float leaf, on the reading "a float
        // leaf is 2, or 4/6 with pooled constants". The 2 is `1 + the TU's
        // `_fltused` slot`, and that slot has belonged to the TU rather than to
        // the function since the eleven-row table below — `plan_labels` charges
        // it once per TU. So the `None` was refusing a class whose stride the
        // counter was *already* getting right, and it refused the whole TU:
        // `docs/CROSS_PRODUCT.md`'s 18-pair residue is every (FP leaf, framed
        // family) pair, all 18 through this one predicate.
        //
        // MEASURED seed-free and in-TU (`docs/LABEL_COUNTER.md` §1,
        // `scripts/gt_label_stride.py`, the in-TU anchor control holding on
        // every row):
        //
        // ```text
        //   leaf-float          float leaf, first FP function      stride 2
        //   leaf-float-led      float leaf, `_fltused` led         stride 1
        //   leaf-double-led     double leaf, `_fltused` led        stride 1
        //   leaf-float-c1-led   float leaf, ONE pooled constant    stride 3
        //   leaf-float-c2-led   float leaf, TWO pooled constants   stride 5
        // ```
        //
        // A leaf that POOLS a constant still refuses, and for **two**
        // independent reasons, either of which is sufficient on its own:
        //
        //  1. the surcharge is +2 per **newly** pooled `(bits,width)`
        //     (`LABEL_COUNTER.md` §1.1), and *newly* is a per-TU question no
        //     per-function method can answer — `const1-dup-led` measures **0**
        //     for a constant an earlier function in the TU already pooled. That
        //     is the same shape as the `_fltused` `+1` and belongs in
        //     `plan_labels`, not here;
        //  2. `c2_core::coff::emit_obj` does not know the `.rdata`/`.pdata`
        //     section order, because **no captured TU has both** — see its own
        //     `debug_assert!(pool.is_empty(), …)`. Admitting (1) alone would
        //     turn a refusal into a guessed section order.
        //
        // Widen this from §1.1's measured surcharge table and from nothing else:
        // the "one slot per TU-level external" story that once explained the
        // `+1` is REFUTED in both directions (§2.1), and it would have licensed
        // a pooled constant at +0.
        if self.float_leaf.is_some() && self.ops.iter().any(|o| matches!(o, IlOp::FpLit { .. })) {
            return None;
        }
        // **An FP-touching function consumes 1, like any other — the extra slot
        // belongs to the TRANSLATION UNIT, not to the function.**
        //
        // Master's rule here was "anything that touches floating point consumes
        // 2, the stride goes with the register file". That is right at **one** FP
        // function, which is all its capture had (one leaf ahead of one framed
        // function), and wrong from two on. Measured seed-free — two framed
        // functions in one TU, reading the *difference* between their labels, so
        // the `.gl` seed cancels and nothing depends on matching mangled-name
        // lengths (`/Ox /GS- /c`, `+1` on every row under `/Gy`):
        //
        // ```text
        //   fr1;                     fr2      delta 4     leaves 0
        //   fr1; int_store;          fr2      delta 5     leaves 1
        //   fr1; fp_store;           fr2      delta 6     leaves 2
        //   fr1; fp_store fp_store;  fr2      delta 7     leaves 3   <- not 4
        //   fr1; int_store fp_store; fr2      delta 7     leaves 3
        //   fr1; fp_store int_store; fr2      delta 7     leaves 3
        //   fr1; int_store int_store;fr2      delta 6     leaves 2
        //   fr1; fp_arith;           fr2      delta 6     leaves 2
        //   fr1; fp_arith fp_arith;  fr2      delta 7     leaves 3
        //   fr1; fp_store fp_arith;  fr2      delta 7     leaves 3
        //   fr1; fp_store x3;        fr2      delta 8     leaves 4   <- not 6
        // ```
        //
        // Eleven rows, one rule, zero residual: **every function consumes 1, plus
        // one extra slot for the TU if any function touches floating point.** The
        // extra slot is `_fltused`, and the two facts `is_float` carries (where
        // `_fltused` goes, and where the extra slot goes) are now the same fact
        // rather than two readers of one field.
        //
        // This comment used to *explain* the `+1` as "one slot per TU-level
        // external", unifying it with the `__savegprlr_N`/`__restgprlr_N` pair's
        // `+2`. **That explanation is refuted** (`docs/LABEL_COUNTER.md` §2.1,
        // measured seed-free against three in-TU anchors): a newly pooled FP
        // constant costs +2 and introduces no external at all, while a string
        // literal costs 0 and introduces one. Every number this method returns
        // survives — only the reason was wrong — but the reason is what would
        // license the next class, so widen this from `LABEL_COUNTER.md` §1.1's
        // measured surcharge table and from nothing else. The rule as stated
        // would have admitted a pooled constant at +0 and under-counted by 2.
        //
        // A per-function method cannot express a per-TU quantity, which is the
        // structural reason the old rule could not be stated correctly here:
        // the `+1` is applied by [`c2_core::coff::plan_labels`], which has the
        // whole function list.
        //
        // **The FP tail call was measured against the same rule rather than
        // assumed into it**, by the same seed-free two-framed-function
        // construction (`/O1 /GS- /c`, so `/Gy` and the framed stride is 5):
        //
        // ```text
        //   fr1;                     fr2      delta 5     leaves 0
        //   fr1; fp_tail;            fr2      delta 7     leaves 1
        //   fr1; fp_tail fp_tail;    fr2      delta 8     leaves 2
        // ```
        //
        // 5 + 1·leaves + 1 for the TU, with zero residual — the extra slot is
        // `_fltused` and it is charged once however many FP functions there are.
        //
        // **`+ label_lead()` is the `eh-bare` surcharge and it is not zero here.**
        // `docs/EH_RECORDS.md` §8.5d measured `eh-bare` at +1 on a *framed* row
        // only; the port's three `empty-dtor-*` shapes are `eh-bare` LEAVES, and
        // `scripts/gt_label_stride.py`'s `eh-bare-dtor{,-led,-adj,-deleg}` rows
        // print stride **2** at `/EHsc` against **1** without it — four shapes,
        // both modes, the anchor control holding on every row. Returning a bare
        // `1` here was a live wrong-bytes emit at the workload's own flags; see
        // [`Self::eh_bare`].
        Some(self.label_lead() + 1)
    }
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
    use super::sy::{Formals, SyView};

    /// These pinned segments are synthetic and have no `.sy` companion, so an
    /// empty local set is the honest input: nothing here is a local the parse may
    /// fold into the expression that reads it.
    ///
    /// Their formals, on the other hand, are all scalars *by construction* — the
    /// bytes are written here in this file — so the widths are stated rather than
    /// left undetermined. Undetermined would refuse every multi-parameter pinned
    /// body on `param-multi-reg` and the grammar tests would grade nothing;
    /// [`Formals::AllOneRegisterByConstruction`] is test-only and cannot appear in
    /// a release build.
    pub(crate) const NO_LOCALS: SyView<'static> =
        SyView { locals: &[], formals: Formals::AllOneRegisterByConstruction };

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


    // ---- the generated empty destructor -------------------------------------
    //
    // Both segments below are transcribed verbatim from live captures of
    // `struct Base { ~Base(); int b; }; struct Der : Base { ~Der(); int d; };
    //  Der::~Der() {}` — the SAME source, compiled twice — so the only bytes that
    // differ between them are the two trailer flags. The pre-body region is real:
    // `53 53 26 <??1Der> B9 <this> <ptr> 99 <ptr> 00 46` with an EMPTY formals list,
    // which is what makes `parse_params` yield `[this]`.

    /// `Der::~Der() {}` captured at the dc3 workload's own flags
    /// (`/nologo /c /GR /O1 /Oi /EHsc`). Trailers `5C … 01` / `5E 01 21`.
    pub(crate) const DTOR_DELEGATE: &[u8] = &[
        0x53, 0x53, 0x26, 0xF0, 0x09, // statement start, `??1Der@@QAA@XZ`
        0xB9, 0xFC, 0x09, 0xA6, 0x43, 0x81, 0x20, // `this`
        0x99, 0x86, 0x43, 0x8A, 0x20, 0x00, // its bind
        0x46, // formals marker, EMPTY list
        0x4C, 0x4F, 0x11, 0x53, // LO SS
        0x33, 0x86, 0x41, 0x74, 0x00, // LIT int 0
        0x26, 0xE4, 0x09, // `??1Base@@QAA@XZ`
        0x33, 0x86, 0x41, 0x74, 0x80, 0x41, 0x08, 0x00, 0x00, // LIT 2113
        0x40, 0x86, 0x43, 0x8E, 0x20, // intrinsic call, pointer result
        0x66, 0x02, 0x80, 0x20, 0x82, 0x20, // class-pair descriptor
        0x55, 0x86, 0x41, 0x74, // selector arg terminator
        0x33, 0x86, 0x41, 0x74, 0x00, 0x55, 0x86, 0x41, 0x74, // adjust offset 0
        0xB9, 0xFC, 0x09, 0xA6, 0x43, 0x81, 0x20, // the object pointer
        0x55, 0xA6, 0x43, 0x81, 0x20, //
        0x4C, // -> the adjusted receiver
        0x2C, 0xA6, 0x43, 0x84, 0x20, 0x00, // cv strip
        0x99, 0x86, 0x43, 0x85, 0x20, 0x00, // member bind
        0xBD, 0x82, 0x07, 0x03, 0x00, 0x80, 0x05, 0x10, 0x00, 0x00, // CALL void
        0x4C, // zero explicit arguments
        0x5C, 0x86, 0x41, 0x74, 0x01, // statement trailer (EH bit clear)
        0x4B, // statement end
        0x3A, 0xFD, 0x09, 0x54, 0x02, 0x29, 0xFD, 0x09, // return plumbing
        0x5E, 0x01, 0x21, // ONE sub-object (EH bit clear)
        0x4B, //
        0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00, // fn tail = segment end
    ];

    /// The same function captured at the fixtures' own profile (`/Ox /GS- /c`,
    /// **no** `/EH`). Byte-identical to [`DTOR_DELEGATE`] but for the two trailer
    /// flags, which gain bit `0x10`: `5C … 11` / `5E 01 31`. Both compile to the
    /// same four bytes (`b ??1Base@@QAA@XZ`), so both must be accepted — pinning
    /// only one would have refused either the whole workload or the whole fixture
    /// lane, depending on which was probed first.
    pub(crate) const DTOR_DELEGATE_NOEH: &[u8] = &[
        0x53, 0x53, 0x26, 0xF0, 0x09, 0xB9, 0xFC, 0x09, 0xA6, 0x43, 0x81, 0x20, 0x99, 0x86, 0x43,
        0x8A, 0x20, 0x00, 0x46, 0x4C, 0x4F, 0x11, 0x53, 0x33, 0x86, 0x41, 0x74, 0x00, 0x26, 0xE4,
        0x09, 0x33, 0x86, 0x41, 0x74, 0x80, 0x41, 0x08, 0x00, 0x00, 0x40, 0x86, 0x43, 0x8E, 0x20,
        0x66, 0x02, 0x80, 0x20, 0x82, 0x20, 0x55, 0x86, 0x41, 0x74, 0x33, 0x86, 0x41, 0x74, 0x00,
        0x55, 0x86, 0x41, 0x74, 0xB9, 0xFC, 0x09, 0xA6, 0x43, 0x81, 0x20, 0x55, 0xA6, 0x43, 0x81,
        0x20, 0x4C, 0x2C, 0xA6, 0x43, 0x84, 0x20, 0x00, 0x99, 0x86, 0x43, 0x85, 0x20, 0x00, 0xBD,
        0x82, 0x07, 0x03, 0x00, 0x80, 0x05, 0x10, 0x00, 0x00, 0x4C, //
        0x5C, 0x86, 0x41, 0x74, 0x11, // statement trailer (EH bit SET)
        0x4B, 0x3A, 0xFD, 0x09, 0x54, 0x02, 0x29, 0xFD, 0x09, //
        0x5E, 0x01, 0x31, // ONE sub-object (EH bit SET)
        0x4B, 0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00,
    ];

    // ---- the generated empty destructor, MEMBER form ------------------------
    //
    // The second generated destructor (`docs/IL_CALL_IN_EXPR.md` §14.3, §15): a
    // class with no destructible base and exactly one destructible **member**,
    // whose receiver is `this + k` through a plain `27` byte-offset add with no
    // class-layout intrinsic anywhere. Both segments below are transcribed verbatim
    // from one live capture of `work/rf/probes/p3.cpp` at the fixture profile
    // (`/Ox /GS- /c`, so the trailers read `5C … 11` / `5E 01 31`):
    //
    //   struct MemA    { ~MemA(); int a; };
    //   struct HasMem  { ~HasMem();  MemA m; };            // member at offset 0
    //   struct HasMem4 { ~HasMem4(); int pad; MemA m; };   // member at offset 4
    //   HasMem::~HasMem() {}
    //   HasMem4::~HasMem4() {}
    //
    // They differ from each other in exactly two places — the offset literal and
    // the per-TU token/type ids — and from `DTOR_DELEGATE` only in the receiver
    // designator, everything from the `2C` strip onward being the same skeleton.

    // ---- WEC: the empty constructor delegating to ONE base ------------------
    //
    // Both transcribed verbatim from live captures at the workload's own flags
    // (`/nologo /wd4355 /wd4164 /c /GR /O1 /Oi /EHsc`) of
    //
    //   struct B1 { B1(); ~B1(); int x; };  struct Ct1 : B1 { Ct1(); };  Ct1::Ct1(){}
    //   struct B0 { B0();        int x; };  struct Cn1 : B0 { Cn1(); };  Cn1::Cn1(){}
    //
    // — the SAME body over a base that has a destructor and one that does not.
    // Their `.text` is byte-identical (48 B, `F = 96`, `mr r31,r3 ; bl ; mr
    // r3,r31`, function symbol `Value = 0`); they differ by the unwind half of
    // the statement, the two EH trailers, and **one label-counter slot**.

    /// `Ct1::Ct1() {}` over a base **with** a destructor: `eh-bare`, carries the
    /// `26 <base dtor>` unwind action, `5C … 01` and `5D 01 21`.
    pub(crate) const CTOR_BASE_EH: &[u8] = &[
        0x53, 0x53, 0x26, 0xF0, 0x09, 0xB9, 0xFB, 0x09, 0xA6, 0x43, 0x81, 0x20,
        0x99, 0x86, 0x43, 0x8C, 0x20, 0x00, 0x46, 0x4C, 0x4F, 0x11, 0x53, 0x26,
        0xE4, 0x09, 0x33, 0x86, 0x41, 0x74, 0x80, 0x41, 0x08, 0x00, 0x00, 0x40,
        0x86, 0x43, 0x90, 0x20, 0x66, 0x02, 0x80, 0x20, 0x82, 0x20, 0x55, 0x86,
        0x41, 0x74, 0x33, 0x86, 0x41, 0x74, 0x00, 0x55, 0x86, 0x41, 0x74, 0xB9,
        0xFB, 0x09, 0xA6, 0x43, 0x81, 0x20, 0x55, 0xA6, 0x43, 0x81, 0x20, 0x4C,
        0x99, 0x86, 0x43, 0x86, 0x20, 0x00, 0xBD, 0xA6, 0x43, 0x84, 0x20, 0x00,
        0x80, 0x05, 0x10, 0x00, 0x00, 0x4C, 0x30, 0x86, 0x46, 0x82, 0x20, 0x26,
        0xE5, 0x09, 0x33, 0x86, 0x41, 0x74, 0x80, 0x41, 0x08, 0x00, 0x00, 0x40,
        0x86, 0x43, 0x90, 0x20, 0x66, 0x02, 0x80, 0x20, 0x82, 0x20, 0x55, 0x86,
        0x41, 0x74, 0x33, 0x86, 0x41, 0x74, 0x00, 0x55, 0x86, 0x41, 0x74, 0xB9,
        0xFB, 0x09, 0xA6, 0x43, 0x81, 0x20, 0x55, 0xA6, 0x43, 0x81, 0x20, 0x4C,
        0x2C, 0xA6, 0x43, 0x84, 0x20, 0x00, 0x99, 0x86, 0x43, 0x86, 0x20, 0x00,
        0xBD, 0x82, 0x07, 0x03, 0x00, 0x80, 0x06, 0x10, 0x00, 0x00, 0x4C, 0x5C,
        0x86, 0x46, 0x82, 0x20, 0x01, 0x4B, 0x3A, 0xFC, 0x09, 0x54, 0x02, 0x29,
        0xFC, 0x09, 0x5D, 0x01, 0x21, 0x4B, 0xB9, 0xFB, 0x09, 0xA6, 0x43, 0x81,
        0x20, 0x41, 0xA6, 0x43, 0x81, 0x20, 0x4F, 0x12, 0x47, 0x54, 0x01, 0x54,
        0x00,
    ];

    /// `Cn1::Cn1() {}` over a base with **no** destructor: `eh-none`, no unwind
    /// half, no `5C`, no `5D` — the statement is `… 4C 30 <OBJ> 4B`.
    pub(crate) const CTOR_BASE_NOEH: &[u8] = &[
        0x53, 0x53, 0x26, 0xEC, 0x09, 0xB9, 0xF3, 0x09, 0xA6, 0x43, 0x81, 0x20,
        0x99, 0x86, 0x43, 0x8B, 0x20, 0x00, 0x46, 0x4C, 0x4F, 0x11, 0x53, 0x26,
        0xE4, 0x09, 0x33, 0x86, 0x41, 0x74, 0x80, 0x41, 0x08, 0x00, 0x00, 0x40,
        0x86, 0x43, 0x8C, 0x20, 0x66, 0x02, 0x80, 0x20, 0x82, 0x20, 0x55, 0x86,
        0x41, 0x74, 0x33, 0x86, 0x41, 0x74, 0x00, 0x55, 0x86, 0x41, 0x74, 0xB9,
        0xF3, 0x09, 0xA6, 0x43, 0x81, 0x20, 0x55, 0xA6, 0x43, 0x81, 0x20, 0x4C,
        0x99, 0x86, 0x43, 0x8D, 0x20, 0x00, 0xBD, 0xA6, 0x43, 0x84, 0x20, 0x00,
        0x80, 0x05, 0x10, 0x00, 0x00, 0x4C, 0x30, 0x86, 0x46, 0x82, 0x20, 0x4B,
        0x3A, 0xF4, 0x09, 0x54, 0x02, 0x29, 0xF4, 0x09, 0xB9, 0xF3, 0x09, 0xA6,
        0x43, 0x81, 0x20, 0x41, 0xA6, 0x43, 0x81, 0x20, 0x4F, 0x12, 0x47, 0x54,
        0x01, 0x54, 0x00,
    ];

    /// `HasMem::~HasMem() {}` — the destructible member at offset **0**. The
    /// reference emits the four bytes `b ??1MemA@@QAA@XZ`.
    pub(crate) const DTOR_MEMBER_OFF0: &[u8] = &[
        0x53, 0x53, 0x26, 0xF0, 0x09, // statement start, `??1HasMem@@QAA@XZ`
        0xB9, 0x09, 0x0A, 0xA6, 0x43, 0x81, 0x20, // `this`
        0x99, 0x86, 0x43, 0x83, 0x20, 0x00, // its bind
        0x46, // formals marker, EMPTY list
        0x4C, 0x4F, 0x11, 0x53, // LO SS
        0x33, 0x86, 0x41, 0x74, 0x00, // LIT int 0 — the leading literal
        0x26, 0xE4, 0x09, // `??1MemA@@QAA@XZ`
        0xB9, 0x09, 0x0A, 0xA6, 0x43, 0x81, 0x20, // the object pointer, `this`
        0x33, 0x86, 0x41, 0x74, 0x00, // LIT int 0 — the member's byte offset
        0x27, 0xA6, 0x43, 0x8A, 0x20, // byte-offset add -> the member's address
        0x2C, 0xA6, 0x43, 0x8B, 0x20, 0x00, // cv strip
        0x99, 0x86, 0x43, 0x8C, 0x20, 0x00, // member bind
        0xBD, 0x82, 0x07, 0x03, 0x00, 0x80, 0x0C, 0x10, 0x00, 0x00, // CALL void
        0x4C, // zero explicit arguments
        0x5C, 0x86, 0x41, 0x74, 0x11, // statement trailer (EH bit set)
        0x4B, // statement end
        0x3A, 0x0A, 0x0A, 0x54, 0x02, 0x29, 0x0A, 0x0A, // return plumbing
        0x5E, 0x01, 0x31, // ONE sub-object (EH bit set)
        0x4B, //
        0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00, // fn tail = segment end
    ];

    /// `HasMem4::~HasMem4() {}` — the same member at offset **4**. The reference
    /// emits `addi r3,r3,4 ; b ??1MemA@@QAA@XZ`: the one instruction that is the
    /// whole codegen difference between the two.
    pub(crate) const DTOR_MEMBER_OFF4: &[u8] = &[
        0x53, 0x53, 0x26, 0xFC, 0x09, // statement start, `??1HasMem4@@QAA@XZ`
        0xB9, 0x0C, 0x0A, 0xA6, 0x43, 0x91, 0x20, // `this`
        0x99, 0x86, 0x43, 0x92, 0x20, 0x00, // its bind
        0x46, // formals marker, EMPTY list
        0x4C, 0x4F, 0x11, 0x53, // LO SS
        0x33, 0x86, 0x41, 0x74, 0x00, // LIT int 0 — the leading literal
        0x26, 0xE4, 0x09, // `??1MemA@@QAA@XZ`
        0xB9, 0x0C, 0x0A, 0xA6, 0x43, 0x91, 0x20, // the object pointer, `this`
        0x33, 0x86, 0x41, 0x74, 0x04, // LIT int 4 — the member's byte offset
        0x27, 0xA6, 0x43, 0x8A, 0x20, // byte-offset add -> the member's address
        0x2C, 0xA6, 0x43, 0x8B, 0x20, 0x00, // cv strip
        0x99, 0x86, 0x43, 0x8C, 0x20, 0x00, // member bind
        0xBD, 0x82, 0x07, 0x03, 0x00, 0x80, 0x0C, 0x10, 0x00, 0x00, // CALL void
        0x4C, // zero explicit arguments
        0x5C, 0x86, 0x41, 0x74, 0x11, // statement trailer (EH bit set)
        0x4B, // statement end
        0x3A, 0x0D, 0x0A, 0x54, 0x02, 0x29, 0x0D, 0x0A, // return plumbing
        0x5E, 0x01, 0x31, // ONE sub-object (EH bit set)
        0x4B, //
        0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00, // fn tail = segment end
    ];

    // ---- indirect-load leaf -------------------------------------------------
    //
    // Every byte below is transcribed from a live capture of
    // `fixtures/cpp/il_expr_deref.cpp` / `il_expr_member.cpp`
    // (`c2rs census <cpp> --keep-il <dir>`), not derived.

    /// `int ld_p(int* p) { return *p; }` — one formal, no offset add.
    // ---- the one-byte-unsigned value class (W26) -----------------------------
    //
    // Transcribed verbatim from a live capture of `work/lf/probes/p2.cpp` /
    // `p4.cpp` at the fixture profile (`/Ox /GS- /c`).

    /// `bool k_false() { return false; }` — a LITERAL of the class, returned as
    /// the class. Emits `38600000` (`li r3,0`) + `blr`, the same word the int
    /// literal leaf emits.
    pub(crate) const BOOL_LIT: &[u8] = &[
        0x53, 0x53, 0x26, 0xE2, 0x09, //
        0x46, // formals marker, empty list
        0x4C, 0x4F, 0x11, 0x53, // LO SS
        0x33, 0x82, 0x12, 0x30, 0x00, // LIT bool 0
        0x41, 0x82, 0x12, 0x30, // result type bool
        0x3A, 0xE3, 0x09, 0x54, 0x02, 0x29, 0xE3, 0x09, //
        0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00,
    ];

    /// `bool b_id(bool b) { return b; }` — the identity, a bare `blr`.
    pub(crate) const BOOL_ID: &[u8] = &[
        0x53, 0x53, 0x26, 0xE5, 0x09, //
        0x46, 0x2D, 0xE4, 0x09, // formals: b
        0x4C, 0x4F, 0x11, 0x53, //
        0xB9, 0xE4, 0x09, 0x82, 0x12, 0x30, // LOAD b (bool)
        0x41, 0x82, 0x12, 0x30, // result type bool
        0x3A, 0xE6, 0x09, 0x54, 0x02, 0x29, 0xE6, 0x09, //
        0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00,
    ];

    /// `unsigned u_from_b(bool b) { return b; }` — the conversion **out of** the
    /// class, which must refuse: the reference is `5463063e`
    /// (`rlwinm r3,r3,0,24,31`), a real mask, and it is spelled with the same
    /// `2C … 00` that is free between the two width-4 classes.
    pub(crate) const BOOL_WIDEN_NEG: &[u8] = &[
        0x53, 0x53, 0x26, 0xE7, 0x09, //
        0x46, 0x2D, 0xE6, 0x09, //
        0x4C, 0x4F, 0x11, 0x53, //
        0xB9, 0xE6, 0x09, 0x82, 0x12, 0x30, // LOAD b (bool)
        0x2C, 0x86, 0x41, 0x74, 0x00, // CONVERT to int
        0x41, 0x86, 0x41, 0x74, //
        0x3A, 0xE8, 0x09, 0x54, 0x02, 0x29, 0xE8, 0x09, //
        0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00,
    ];

    // ---- the store leaf (W25) ------------------------------------------------
    //
    // Every segment below is transcribed verbatim from a live capture of
    // `work/lf/probes/p1.cpp` at the fixture profile (`/Ox /GS- /c`), from the
    // `4F 1F` split point forward, with the per-function header region trimmed to
    // the `53 53 26 <fn>` prefix `free_fn` supplies.

    /// `void s_b(S* s, int v) { s->b = v; }` — the plain designator, a `27`
    /// byte-offset add of 4, an `int` value out of r4. Emits `90830004` (`stw
    /// r4,4(r3)`) + `blr`.
    pub(crate) const STORE_MEMBER: &[u8] = &[
        0x53, 0x53, 0x26, 0xFB, 0x09, // statement start, function-symbol push
        0x46, 0x2D, 0xFA, 0x09, 0x2D, 0xF9, 0x09, // formals (reverse order): v, s
        0x4C, 0x4F, 0x11, 0x53, // LO SS
        0xB9, 0xF9, 0x09, 0x86, 0x43, 0x81, 0x20, // LOAD s (S *)
        0x33, 0x86, 0x41, 0x74, 0x04, 0x27, 0x86, 0x43, 0xF4, 0x08, // + 4, retyped int*
        0xB9, 0xFA, 0x09, 0x86, 0x41, 0x74, // LOAD v (int)
        0x32, 0x86, 0x41, 0x74, // STORE int
        0x4B, // statement end
        0x3A, 0xFC, 0x09, 0x54, 0x02, 0x29, 0xFC, 0x09, // return plumbing (void)
        0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00, // fn tail = segment end
    ];

    /// `void s_c(S* s, char v) { s->c = v; }` — the same, at width 1 and offset
    /// 12. Emits `9883000c` (`stb r4,12(r3)`).
    pub(crate) const STORE_NARROW: &[u8] = &[
        0x53, 0x53, 0x26, 0x03, 0x0A, //
        0x46, 0x2D, 0x02, 0x0A, 0x2D, 0x01, 0x0A, // formals: v, s
        0x4C, 0x4F, 0x11, 0x53, //
        0xB9, 0x01, 0x0A, 0x86, 0x43, 0x81, 0x20, // LOAD s
        0x33, 0x86, 0x41, 0x74, 0x0C, 0x27, 0x82, 0x43, 0xF0, 0x08, // + 12, char*
        0xB9, 0x02, 0x0A, 0x82, 0x11, 0x70, // LOAD v (char)
        0x32, 0x82, 0x11, 0x70, // STORE char
        0x4B, //
        0x3A, 0x04, 0x0A, 0x54, 0x02, 0x29, 0x04, 0x0A, //
        0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00,
    ];

    /// `void s_k(S* s) { s->a = 7; }` — a LITERAL value. Emits
    /// `39600007 91630000` (`li r11,7 ; stw r11,0(r3)`).
    pub(crate) const STORE_LIT: &[u8] = &[
        0x53, 0x53, 0x26, 0x22, 0x0A, //
        0x46, 0x2D, 0x21, 0x0A, // formals: s
        0x4C, 0x4F, 0x11, 0x53, //
        0xB9, 0x21, 0x0A, 0x86, 0x43, 0x81, 0x20, // LOAD s
        0x33, 0x86, 0x41, 0x74, 0x00, 0x27, 0x86, 0x43, 0xF4, 0x08, // + 0, int*
        0x33, 0x86, 0x41, 0x74, 0x07, // LIT 7
        0x32, 0x86, 0x41, 0x74, // STORE int
        0x4B, //
        0x3A, 0x23, 0x0A, 0x54, 0x02, 0x29, 0x23, 0x0A, //
        0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00,
    ];

    /// `void t_sb1(Der* d, int v) { d->b1 = v; }` — the **intrinsic-2117**
    /// designator (`66 02` class-pair descriptor, member offset 4 + base offset
    /// 0). Emits the identical `90830004` the plain form does.
    pub(crate) const STORE_BASE_MEMBER: &[u8] = &[
        0x53, 0x53, 0x26, 0x63, 0x0A, //
        0x46, 0x2D, 0x62, 0x0A, 0x2D, 0x61, 0x0A, // formals: v, p
        0x4C, 0x4F, 0x11, 0x53, //
        0x33, 0x86, 0x41, 0x74, 0x80, 0x45, 0x08, 0x00, 0x00, // selector 2117
        0x40, 0x86, 0x43, 0xF4, 0x08, // intrinsic call -> int*
        0x66, 0x02, 0x8F, 0x20, 0x91, 0x20, // class-pair descriptor
        0x55, 0x86, 0x41, 0x74, //
        0x33, 0x86, 0x41, 0x74, 0x04, 0x55, 0x86, 0x41, 0x74, // member offset 4
        0x33, 0x86, 0x41, 0x74, 0x00, 0x55, 0x86, 0x41, 0x74, // base offset 0
        0xB9, 0x61, 0x0A, 0x86, 0x43, 0x97, 0x20, // LOAD p
        0x55, 0x86, 0x43, 0x97, 0x20, 0x4C, // apply
        0xB9, 0x62, 0x0A, 0x86, 0x41, 0x74, // LOAD v (int)
        0x32, 0x86, 0x41, 0x74, 0x4B, // STORE int, statement end
        0x3A, 0x64, 0x0A, 0x54, 0x02, 0x29, 0x64, 0x0A, //
        0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00,
    ];

    /// `void n_f(S* s, float v) { s->f = v; }` — the FLOAT value, which must
    /// refuse: the reference is `d0230014` (`stfs f1,20(r3)`), out of the FP
    /// argument file whose register number counts FP parameters alone.
    pub(crate) const STORE_FLOAT_NEG: &[u8] = &[
        0x53, 0x53, 0x26, 0x0F, 0x0A, //
        0x46, 0x2D, 0x0E, 0x0A, 0x2D, 0x0D, 0x0A, //
        0x4C, 0x4F, 0x11, 0x53, //
        0xB9, 0x0D, 0x0A, 0x86, 0x43, 0x81, 0x20, //
        0x33, 0x86, 0x41, 0x74, 0x14, 0x27, 0x86, 0x43, 0xC0, 0x08, // + 20, float*
        0xB9, 0x0E, 0x0A, 0x86, 0x45, 0x40, // LOAD v (float)
        0x32, 0x86, 0x45, 0x40, 0x4B, //
        0x3A, 0x10, 0x0A, 0x54, 0x02, 0x29, 0x10, 0x0A, //
        0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00,
    ];

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
    /// `int g(int,int); int f(int a,int b,int c){ return g(a,c); }` — a two-argument
    /// tail call that passes formals **0 and 2** of three.
    ///
    /// Transcribed verbatim from a live capture (`/Ox /GS- /c`, `c2rs capture`),
    /// and it is a **crash** fixture: the argument sources index the *formals*
    /// list, the permutation analysis in [`body::parse_segment`]'s multi-argument
    /// path indexed a `seen[]` sized by the *argument* count with one, and `2 >= 2`
    /// panicked `c2rs census` on two lines of ordinary C++. See the refusal at
    /// `call-arg-outer-formal`.
    pub(crate) const ARG2_OUTER_FORMAL: &[u8] = &[
        0x53, 0x53, 0x26, 0xE9, 0x09, // stmt start, fn push
        0x46, 0x2D, 0xE8, 0x09, 0x2D, 0xE7, 0x09, 0x2D, 0xE6, 0x09, // formals, REVERSED: c, b, a
        0x4C, 0x4F, 0x11, 0x53, // LO SS
        0x26, 0xE5, 0x09, 0xBD, 0x86, 0x41, 0x74, 0x00, 0x80, 0x01, 0x10, 0x00, 0x00, // 26 CALL
        0xB9, 0xE8, 0x09, 0x86, 0x41, 0x74, 0x55, 0x86, 0x41, 0x74, // push: c  (formal 2)
        0xB9, 0xE6, 0x09, 0x86, 0x41, 0x74, 0x55, 0x86, 0x41, 0x74, // push: a  (formal 0)
        0x4C, 0x41, 0x86, 0x41, 0x74, // call-end, result-type int
        0x3A, 0xEA, 0x09, 0x54, 0x02, 0x29, 0xEA, 0x09, // assign + return
        0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00, 0x4F, 0x02, 0x20, 0x00, 0x4F, 0x01, 0x03, 0x4D,
    ];

    // The **call-bound-to-a-local** form of the two shapes above
    // (`int z = g(…); return z;`, the `26 <dst>` … `32 <TYPE> 4B` … reload
    // production). It used to carry its own copy of the argument validation, and
    // the copy was missing a gate at each of these two points. Both transcribed
    // verbatim from live `/Ox /GS- /c` captures.

    // ---- Class A many-calls (#35 step 2, rung 1) -----------------------------
    //
    // All six transcribed verbatim from live `/O1 /GS- /c` captures, each beside
    // the `.text` the same source produced.

    /// `void g1(int); void g2(); void f(int a){ g1(a); g2(); }` — two statement
    /// calls, the formal dying at the first, nothing saved:
    ///
    /// ```text
    ///   mflr r12 ; stw r12,-8(r1) ; stwu r1,-96(r1)
    ///   bl ?g1 ; bl ?g2
    ///   addi r1,r1,96 ; lwz r12,-8(r1) ; mtlr r12 ; blr
    /// ```
    pub(crate) const SEQ_TWO_VOID: &[u8] = &[
        0x53, 0x53, 0x26, 0xE7, 0x09, // stmt start, fn push
        0x46, 0x2D, 0xE6, 0x09, // formals: a
        0x4C, 0x4F, 0x11, 0x53, // LO SS
        0x26, 0xE4, 0x09, 0xBD, 0x82, 0x07, 0x03, 0x00, 0x80, 0x01, 0x10, 0x00, 0x00, // g1
        0xB9, 0xE6, 0x09, 0x86, 0x41, 0x74, 0x55, 0x86, 0x41, 0x74, 0x4C, // arg a, call-end
        0x4B, // result discarded — a STATEMENT call
        0x26, 0xE5, 0x09, 0xBD, 0x82, 0x07, 0x03, 0x00, 0x80, 0x03, 0x10, 0x00, 0x00, // g2
        0x4C, 0x4B, // no arguments, discarded
        0x3A, 0xE8, 0x09, 0x54, 0x02, 0x29, 0xE8, 0x09, // void return plumbing
        0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00, 0x4F, 0x02, 0x20, 0x00, 0x4F, 0x01, 0x03, 0x4D,
    ];
    /// `void g1(int); int f(int a){ g1(a); return 5; }` — one statement call and a
    /// literal return: `bl ?g1 ; li r3,5 ; <epilogue>`. Framed on **one** call,
    /// which is why the class boundary is "is anything after the call", not "are
    /// there two calls".
    pub(crate) const SEQ_ONE_THEN_LIT: &[u8] = &[
        0x53, 0x53, 0x26, 0xE6, 0x09, // stmt start, fn push
        0x46, 0x2D, 0xE5, 0x09, // formals: a
        0x4C, 0x4F, 0x11, 0x53, // LO SS
        0x26, 0xE4, 0x09, 0xBD, 0x82, 0x07, 0x03, 0x00, 0x80, 0x01, 0x10, 0x00, 0x00, // g1
        0xB9, 0xE5, 0x09, 0x86, 0x41, 0x74, 0x55, 0x86, 0x41, 0x74, 0x4C, 0x4B, // arg a, stmt
        0x33, 0x86, 0x41, 0x74, 0x05, // LIT 5
        0x41, 0x86, 0x41, 0x74, // result-type int
        0x3A, 0xE7, 0x09, 0x54, 0x02, 0x29, 0xE7, 0x09, // return plumbing
        0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00, 0x4F, 0x02, 0x20, 0x00, 0x4F, 0x01, 0x03, 0x4D,
    ];
    /// `int g1(); int g2(); int f(){ g1(); return g2(); }` — the last call's value
    /// IS the result and it is **not** tail-called: `bl ?g1 ; bl ?g2 ;
    /// addi r1,r1,96 ; … ; blr`.
    pub(crate) const SEQ_CALL_VALUE: &[u8] = &[
        0x53, 0x53, 0x26, 0xE5, 0x09, 0x46, // stmt start, fn push, no formals
        0x4C, 0x4F, 0x11, 0x53, // LO SS
        0x26, 0xE3, 0x09, 0xBD, 0x86, 0x41, 0x74, 0x00, 0x80, 0x01, 0x10, 0x00, 0x00, // g1
        0x4C, 0x4B, // discarded
        0x26, 0xE4, 0x09, 0xBD, 0x86, 0x41, 0x74, 0x00, 0x80, 0x01, 0x10, 0x00, 0x00, // g2
        0x4C, // call-end, result CONSUMED
        0x41, 0x86, 0x41, 0x74, // result-type int
        0x3A, 0xE6, 0x09, 0x54, 0x02, 0x29, 0xE6, 0x09, // return plumbing
        0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00, 0x4F, 0x02, 0x20, 0x00, 0x4F, 0x01, 0x03, 0x4D,
    ];
    /// `void g1(); int g2(); int f(){ g1(); return g2() + 1; }` — the same, with
    /// the `addi r3,r3,1` post-op the single framed call already carried.
    pub(crate) const SEQ_CALL_VALUE_PLUSK: &[u8] = &[
        0x53, 0x53, 0x26, 0xE5, 0x09, 0x46, // stmt start, fn push, no formals
        0x4C, 0x4F, 0x11, 0x53, // LO SS
        0x26, 0xE3, 0x09, 0xBD, 0x82, 0x07, 0x03, 0x00, 0x80, 0x01, 0x10, 0x00, 0x00, // g1
        0x4C, 0x4B, // discarded
        0x26, 0xE4, 0x09, 0xBD, 0x86, 0x41, 0x74, 0x00, 0x80, 0x02, 0x10, 0x00, 0x00, // g2
        0x4C, // call-end, result CONSUMED
        0x33, 0x86, 0x41, 0x74, 0x01, 0x02, // post-op LIT 1 + ADD
        0x41, 0x86, 0x41, 0x74, // result-type int
        0x3A, 0xE6, 0x09, 0x54, 0x02, 0x29, 0xE6, 0x09, // return plumbing
        0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00, 0x4F, 0x02, 0x20, 0x00, 0x4F, 0x01, 0x03, 0x4D,
    ];
    /// `void g1(int); void g2(int); void f(int a,int b){ g1(a); g2(b); }` — the
    /// Class A **boundary**: `b` is read after the first call, so it must survive
    /// one, and c2 puts it in `r31` behind a `std`/`ld` pair (Class B, 5-word
    /// prologue, 11-word epilogue). Since 2026-07-30 this decodes as **Class B**
    /// with `saved = [1]` (`b` takes r31); it was the Class A boundary case.
    pub(crate) const SEQ_LIVE_ACROSS: &[u8] = &[
        0x53, 0x53, 0x26, 0xE9, 0x09, // stmt start, fn push
        0x46, 0x2D, 0xE8, 0x09, 0x2D, 0xE7, 0x09, // formals, REVERSED: b, a
        0x4C, 0x4F, 0x11, 0x53, // LO SS
        0x26, 0xE5, 0x09, 0xBD, 0x82, 0x07, 0x03, 0x00, 0x80, 0x01, 0x10, 0x00, 0x00, // g1
        0xB9, 0xE7, 0x09, 0x86, 0x41, 0x74, 0x55, 0x86, 0x41, 0x74, 0x4C, 0x4B, // arg a, stmt
        0x26, 0xE6, 0x09, 0xBD, 0x82, 0x07, 0x03, 0x00, 0x80, 0x01, 0x10, 0x00, 0x00, // g2
        0xB9, 0xE8, 0x09, 0x86, 0x41, 0x74, 0x55, 0x86, 0x41, 0x74, 0x4C, 0x4B, // arg b, stmt
        0x3A, 0xEA, 0x09, 0x54, 0x02, 0x29, 0xEA, 0x09, // void return plumbing
        0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00, 0x4F, 0x02, 0x20, 0x00, 0x4F, 0x01, 0x03, 0x4D,
    ];
    /// `void g1(int); void f(int a){ g1(a); }` — a **single** statement call with
    /// nothing after it. c2 tail-calls it: a bare `b ?g1`, 5 sections, no frame.
    /// Emitting the Class A frame here would be a mis-emit, so this is the control
    /// for the sequence production's entry condition.
    pub(crate) const SEQ_LONE_STMT_CALL: &[u8] = &[
        0x53, 0x53, 0x26, 0xE6, 0x09, // stmt start, fn push
        0x46, 0x2D, 0xE5, 0x09, // formals: a
        0x4C, 0x4F, 0x11, 0x53, // LO SS
        0x26, 0xE4, 0x09, 0xBD, 0x82, 0x07, 0x03, 0x00, 0x80, 0x01, 0x10, 0x00, 0x00, // g1
        0xB9, 0xE5, 0x09, 0x86, 0x41, 0x74, 0x55, 0x86, 0x41, 0x74, 0x4C, 0x4B, // arg a, stmt
        0x3A, 0xE7, 0x09, 0x54, 0x02, 0x29, 0xE7, 0x09, // void return plumbing
        0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00, 0x4F, 0x02, 0x20, 0x00, 0x4F, 0x01, 0x03, 0x4D,
    ];

    /// `int g(int); int f(int a,int b){ int z = g(b + a); return z; }` — a
    /// commutative argument expression in **non-canonical leaf order**.
    ///
    /// A **wrong-bytes emit**: c2 canonicalizes the leaves, so this compiles to
    /// the same `add r3,r3,r4 ; b ?g` as `g(a + b)`, and the port emitted
    /// `add r3,r4,r3`. `leaves_ascending` has gated the direct `return g(b + a);`
    /// form since the reassociation rule was measured; this one never asked.
    /// Fixture `il_call_bound_neg.cpp`.
    pub(crate) const BOUND_ARG_NONCANON: &[u8] = &[
        0x53, 0x53, 0x26, 0xE7, 0x09, // stmt start, fn push
        0x46, 0x2D, 0xE6, 0x09, 0x2D, 0xE5, 0x09, // formals, REVERSED: b, a
        0x4C, 0x4F, 0x11, 0x53, // LO SS
        0x26, 0xE9, 0x09, // 26 <dst> — the local `z`
        0x26, 0xE4, 0x09, 0xBD, 0x86, 0x41, 0x74, 0x00, 0x80, 0x01, 0x10, 0x00, 0x00, // 26 CALL
        0xB9, 0xE6, 0x09, 0x86, 0x41, 0x74, // LOAD b   <- source order, NOT ascending
        0xB9, 0xE5, 0x09, 0x86, 0x41, 0x74, // LOAD a
        0x02, // ADD
        0x55, 0x86, 0x41, 0x74, 0x4C, // 55 <int> 4C call-end
        0x32, 0x86, 0x41, 0x74, 0x4B, // store into z, discard
        0xB9, 0xE9, 0x09, 0x86, 0x41, 0x74, // reload z
        0x41, 0x86, 0x41, 0x74, // result-type int
        0x3A, 0xE8, 0x09, 0x54, 0x02, 0x29, 0xE8, 0x09, // assign + return
        0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00, 0x4F, 0x02, 0x20, 0x00, 0x4F, 0x01, 0x03, 0x4D,
    ];
    /// `int g(int); int f(int a,int b){ int z = g(a + b); return z; }` — the same
    /// body with the leaves in **canonical** order. The control: it is in class
    /// and byte-exact, and the pair is what separates "the gate refuses this
    /// production" from "the gate refuses this leaf order".
    pub(crate) const BOUND_ARG_CANON: &[u8] = &[
        0x53, 0x53, 0x26, 0xE7, 0x09, // stmt start, fn push
        0x46, 0x2D, 0xE6, 0x09, 0x2D, 0xE5, 0x09, // formals, REVERSED: b, a
        0x4C, 0x4F, 0x11, 0x53, // LO SS
        0x26, 0xE9, 0x09, // 26 <dst> — the local `z`
        0x26, 0xE4, 0x09, 0xBD, 0x86, 0x41, 0x74, 0x00, 0x80, 0x01, 0x10, 0x00, 0x00, // 26 CALL
        0xB9, 0xE5, 0x09, 0x86, 0x41, 0x74, // LOAD a
        0xB9, 0xE6, 0x09, 0x86, 0x41, 0x74, // LOAD b
        0x02, // ADD
        0x55, 0x86, 0x41, 0x74, 0x4C, // 55 <int> 4C call-end
        0x32, 0x86, 0x41, 0x74, 0x4B, // store into z, discard
        0xB9, 0xE9, 0x09, 0x86, 0x41, 0x74, // reload z
        0x41, 0x86, 0x41, 0x74, // result-type int
        0x3A, 0xE8, 0x09, 0x54, 0x02, 0x29, 0xE8, 0x09, // assign + return
        0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00, 0x4F, 0x02, 0x20, 0x00, 0x4F, 0x01, 0x03, 0x4D,
    ];
    /// `int g2(int,int); int f(int a,int b,int c){ int z = g2(a, c); return z; }` —
    /// [`ARG2_OUTER_FORMAL`] in the bound-to-a-local production.
    ///
    /// A **panic**: the permutation vector `[0, 2]` indexed a `seen[]` sized by the
    /// argument count. The direct form got the `call-arg-outer-formal` gate when
    /// that was found; this copy did not, and `c2rs census` died on it.
    pub(crate) const BOUND_ARG2_OUTER_FORMAL: &[u8] = &[
        0x53, 0x53, 0x26, 0xE9, 0x09, // stmt start, fn push
        0x46, 0x2D, 0xE8, 0x09, 0x2D, 0xE7, 0x09, 0x2D, 0xE6, 0x09, // formals, REVERSED: c, b, a
        0x4C, 0x4F, 0x11, 0x53, // LO SS
        0x26, 0xEB, 0x09, // 26 <dst> — the local `z`
        0x26, 0xE5, 0x09, 0xBD, 0x86, 0x41, 0x74, 0x00, 0x80, 0x01, 0x10, 0x00, 0x00, // 26 CALL
        0xB9, 0xE8, 0x09, 0x86, 0x41, 0x74, 0x55, 0x86, 0x41, 0x74, // push: c (formal 2)
        0xB9, 0xE6, 0x09, 0x86, 0x41, 0x74, 0x55, 0x86, 0x41, 0x74, // push: a (formal 0)
        0x4C, // call-end
        0x32, 0x86, 0x41, 0x74, 0x4B, // store into z, discard
        0xB9, 0xEB, 0x09, 0x86, 0x41, 0x74, // reload z
        0x41, 0x86, 0x41, 0x74, // result-type int
        0x3A, 0xEA, 0x09, 0x54, 0x02, 0x29, 0xEA, 0x09, // assign + return
        0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00, 0x4F, 0x02, 0x20, 0x00, 0x4F, 0x01, 0x03, 0x4D,
    ];
}

#[cfg(test)]
mod compare_leaf_gate_tests {
    use super::{CompareLeaf, Rel};

    fn leaf(rel: Rel, signed: bool, k: i32) -> CompareLeaf {
        CompareLeaf { param: 0x10, rel, signed, k }
    }

    /// The three clauses that used to live only in
    /// `c2_core::codegen::compare_leaf_text`. The unsigned one was in codegen
    /// *alone*, so `int f(unsigned a){ return a == 4294967295u; }` censused in
    /// class and the port refused it.
    #[test]
    fn the_difference_spine_gates_are_the_ones_the_census_now_applies() {
        // Accepted: a literal the `addi a,-k` immediate can carry.
        assert_eq!(leaf(Rel::Eq, false, 5).out_of_class_ctx(), None);
        assert_eq!(leaf(Rel::Ne, true, -32767).out_of_class_ctx(), None);
        // A large UNSIGNED literal (decoded as a negative i32) — c2 materializes
        // the constant and subtracts.
        assert_eq!(
            leaf(Rel::Eq, false, -1).out_of_class_ctx(),
            Some("cmp-out-of-class-unsigned-wide-lit")
        );
        // `-(-32768)` overflows the immediate.
        assert_eq!(
            leaf(Rel::Eq, true, -32768).out_of_class_ctx(),
            Some("cmp-out-of-class-lit-i16-min")
        );
        // Outside SIMM16 entirely: `lis`+`ori` and the temp slot it consumes.
        assert_eq!(
            leaf(Rel::Lt, true, 40000).out_of_class_ctx(),
            Some("cmp-out-of-class-wide-lit")
        );
        // The carry spines never negate the literal, so a large unsigned is a
        // legitimate `subfic` for them. Sharing one predicate here was a live
        // wrong-bytes emit; keeping them separate is the point.
        assert_eq!(leaf(Rel::Gt, false, -5).out_of_class_ctx(), None);
        // A zero literal takes the folded spines, which carry no immediate.
        assert_eq!(leaf(Rel::Eq, false, 0).out_of_class_ctx(), None);
    }
}
