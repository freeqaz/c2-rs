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
mod diag;
mod gl;
mod glalias;
mod ininit;
mod inlit;
mod readers;
mod sy;

pub use self::bind::{gl_body_record_names, EmitBinding};
pub use self::body::{chain_form, Block, ChainForm, FP_SCRATCH};
pub use self::bundle::{
    DataObject, DataTu,
    DynInitTu,
    GlDataRow, InAliasReport,
    is_empty_module, opt_word_mode, OptWordMode, OPT_WORD_O1, OPT_WORD_OX,
    OPT_WORD_SPECIAL_MEMBER,
};
/// The body-start locator, crate-visible so `codec` calls the ONE rule instead
/// of keeping a second copy of it (ROADMAP §10.12, §10.14).
pub(crate) use self::bundle::{body_start, body_start_is_bare, ops_start};
pub use self::census::{
    cflow_residue_admit_set, FnCensus, FnVerdict, CENSUS_HEX_BACK, CENSUS_HEX_FWD,
};
pub use self::diag::{cause, DecodeCauses};
pub use self::ininit::{InInitReport, InInitResidue, InSymbolRef};
pub use self::gl::{
    gl_symbol_conflicts, gl_symbol_index, label_counter, mangled_name, mangled_names, source_path,
};
pub use self::glalias::{
    gl_alias_table, gl_alias_table_shifted, GlAliasStats, GlAliasTable,
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
    /// **WR1 — the address of a NAMED DATA SYMBOL**, by its `.gl` operand token
    /// (`26 <tok>`, optionally followed by one array-to-pointer `2C`).
    ///
    /// Two instructions and a relocation quad, and nothing else:
    /// `lis r11,sym@ha` + `addi rD,r11,sym@l`, carrying `REFHI+PAIR` at the `lis`
    /// and `REFLO+PAIR` at the `addi` — byte-for-byte the shape
    /// [`c2_core::coff`] already emits for a pooled FP constant. MEASURED, every
    /// word read off the reference obj (`work/wr1/probes/p1.cpp`, `p2.cpp`):
    ///
    /// ```text
    ///   void f(S* s){ s->so(&gI); }   3d600000 lis r11,0 · 388b0000 addi r4,r11,0 · b ?so
    ///   void f(){ gso(&gI); }         3d600000 lis r11,0 · 386b0000 addi r3,r11,0 · b ?gso
    /// ```
    ///
    /// **The addend is never folded into the relocation** (`docs/IL_CALL_IN_EXPR.md`
    /// §17.2 item 1): `&gT.b` is `lis ; addi r11,r11,0 ; addi r3,r11,4`, a third
    /// instruction, so an offset run on the designator is refused rather than
    /// added to the `addi`'s displacement.
    ///
    /// The token is resolved to a mangled name through the same `.gl` symbol index
    /// a callee goes through, and the same way: an unresolvable token refuses.
    /// **A string literal's token is not in that index** — `gl.rs`'s
    /// `NAME_SEPARATORS` excludes the `25` separator that introduces a `??_C@…`
    /// record — so `f("hi")` refuses here, which is deliberate: a literal needs a
    /// `.rdata` pool ( `/Ox` ) or a `??_C@…` COMDAT ( `/O1` ) that this port does
    /// not emit.
    ///
    /// Produced ONLY by [`super::body::shapes::calls::eat_call_args`], and only as
    /// the whole of one call argument's operand stream.
    SymAddr(u32),
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
    /// **Board #1199 — THE BIND CARRIER.** The token `tok`, which denotes
    /// `base + off`: a C++ reference (or pointer) local bound to a formal's
    /// interior, `auto& listHead = mListHead;`, standing where an ordinary
    /// [`IlOp::Load`] of a formal stands.
    ///
    /// # Why this is an OP and not a field
    ///
    /// `w-bind` named this row and was precise about what it is *not*. #844's
    /// carrier ([`CallSeq::store_run`]) holds *a run and a call* and has no field
    /// binding a token to `formal + offset`; the obvious repair — a
    /// `binds: Vec<RefBind>` beside the ops — puts the fact in a **second
    /// container**, and a second container is a thing a consumer can hold the run
    /// without. That is board #232's mechanism and #844's own, one layer out:
    /// `IlFunction::ops` and `CallSeq::store_run` are two homes for a run, so a
    /// bindings list beside them is a fact that can be dropped by whichever home
    /// the selector picks.
    ///
    /// Inside the op stream there is nothing beside the ops to drop. `w-seam2`
    /// found the same shape of answer for #844 — *"an ordering fix would leave
    /// two settable fields and one winner; carrying the run inside the sequence
    /// makes the race unspellable"* — and this is that sentence applied to the
    /// binding.
    ///
    /// # The state that must be unspellable, and why it is
    ///
    /// Board **#1128**: `src/xdk/nuispeech/xboxheap.cpp`'s constructor written
    /// *with* the bind and *without* it emit **different bodies**, four words
    /// apart. The wrong state is the reader handing the emitter the *other*
    /// spelling's op stream — resolving the bound local to the formal it hangs
    /// off, so the run's may-alias analysis sees one base symbol where c2 sees
    /// two.
    ///
    /// A store's **base symbol** (what [`crate::func::body::shapes`]' consumers
    /// and `c2_core::codegen::schedule::Stmt::base` key aliasing on) and its
    /// **base register plus displacement** are two derivations of **one**
    /// `BoundAddr` value: the symbol is `tok`, the address is `base` and
    /// `off + <the store's own offset>`. They cannot disagree, because they come
    /// from the same value. To emit the direct spelling's words the op stream
    /// would have to hold `Load(base)` where this stands — and the reader never
    /// substitutes, so it cannot.
    ///
    /// # `off` is NEVER pre-added into the store's displacement
    ///
    /// The offset *inside* the bound object stays on the [`IlOp::StoreInd`]; the
    /// offset *of* the bound object stays here. The sum is formed at exactly one
    /// site, in the emitter, so the binding cannot be discharged twice. Summing
    /// in the reader is what would make the two source spellings' op streams
    /// identical, and #1128 measured that their bodies are not.
    ///
    /// # A consumer that does not know this variant REFUSES
    ///
    /// Every op-stream walk under `crates/c2-core` is an exact slice pattern over
    /// [`IlOp::Load`] / [`IlOp::Lit`] / [`IlOp::StoreInd`] and friends. `BoundAddr`
    /// matches none of them, so an unwidened consumer falls to its own
    /// `out_of_class` — a gap, never a shorter body.
    ///
    /// Produced ONLY by [`crate::func::bundle::shape_to_function`]'s
    /// [`crate::func::body::BodyShape::StoreRunBind`] arm, which discharges the
    /// reader's `RefBind` list into the op stream and then builds the carriers
    /// that already exist. `RefBind` itself never crosses into `c2-core`.
    BoundAddr { tok: u32, base: u32, off: i32 },
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
    /// Pop rhs then lhs, push `lhs & rhs` (IL opcode `0x0B`, commutative).
    ///
    /// **Register-register only.** The whole bitwise/shift family below is
    /// admitted at exactly one operand form, and the immediate form is refused
    /// with a measured reason rather than an assumed one — `lane w-build`
    /// probed the immediate axis of `&` alone and found **four** distinct
    /// lowerings across five cells at the workload's own flags:
    ///
    /// ```text
    ///   a & 1        5463 07fe   clrlwi r3,r3,31          a contiguous mask -> rlwinm
    ///   a & 0xFF00   5463 042e   rlwinm r3,r3,0,16,23     "
    ///   a & -2       5463 003c   clrrwi r3,r3,1           "
    ///   a & 5        7063 0005   andi.  r3,r3,5           NOT contiguous, fits 16 bits
    ///   a & 0x12345  3d80 0001   lis    r12,1             neither: materialize, and
    ///                618c 2345   ori    r12,r12,0x2345    in **r12**, not the r11
    ///                7c63 6038   and    r3,r3,r12         every other shape uses
    /// ```
    ///
    /// So the selector is a predicate over the immediate's **value** — is the
    /// mask contiguous, does it fit sixteen bits — not over its type; one of
    /// its three branches sets `CR0` (`andi.` is record-form only, there is no
    /// plain `andi`); and one of them uses a *different scratch register*. That
    /// is a cross product this rung does not grid, so `Imm` on any of these ops
    /// is `out_of_class` in `select_text` — the same refusal [`IlOp::Mul`]
    /// already carries for its non-register forms, and for the same reason.
    And,
    /// Pop rhs then lhs, push `lhs | rhs` (IL opcode `0x0C`, commutative).
    ///
    /// Immediates refused: `a | 1` is `ori`, `a | 0x10000` is `oris`, and
    /// `a | 0x12345` is **both** (`oris` then `ori`, two instructions). Three
    /// shapes across one axis; see [`IlOp::And`].
    Or,
    /// Pop rhs then lhs, push `lhs ^ rhs` (IL opcode `0x0D`, commutative).
    /// Immediates refused, same three shapes as [`IlOp::Or`] with
    /// `xori`/`xoris`.
    Xor,
    /// Pop rhs then lhs, push `lhs << rhs` (IL opcode `0x09`, NON-commutative).
    ///
    /// **One instruction for both signednesses**, measured and not assumed:
    /// `int<<int`, `unsigned<<unsigned` and `int<<unsigned` all emit `slw`,
    /// `7c632030`. That is why there is one `Shl` and two `Shr`s.
    Shl,
    /// Pop rhs then lhs, push `lhs >> rhs` **arithmetically** — `sraw` (IL
    /// opcode `0x0A` over a SIGNED left operand).
    ///
    /// [`IlOp::ShrS`] and [`IlOp::ShrU`] are the same IL byte. The IL does not
    /// distinguish them at all: the difference is one nibble of the *operand
    /// TYPE* (`86 41` signed, `86 42` unsigned), and `ValueClass::Int4` — the
    /// class the expression parser tracks — deliberately collapses the two,
    /// because every other modeled operator is identical across them. So the
    /// parser reads the signedness separately for this operator, and refuses a
    /// mixed-signedness expression outright under `expr-shr-mixed-sign`.
    ///
    /// **Only the LEFT operand decides**, probed both ways round:
    /// `int f(int a, unsigned b){return a>>b;}` is `sraw` and
    /// `unsigned f(unsigned a, int b){return a>>b;}` is `srw`.
    ShrS,
    /// Pop rhs then lhs, push `lhs >> rhs` **logically** — `srw` (IL opcode
    /// `0x0A` over an UNSIGNED left operand). See [`IlOp::ShrS`].
    ShrU,
}

impl IlOp {
    /// True for the binary integer operators a **serial accumulator chain** can
    /// carry — the ones that pop two operands and push one, and that the
    /// straight-line selector lowers into one register-register instruction.
    ///
    /// One predicate rather than a `matches!` repeated at each site: the
    /// stack-depth simulation in `chain_form`, the pointer-arithmetic guard and
    /// the `bool`-arithmetic guard in `parse_expr`, and the selector's own
    /// binary arm must agree about the set exactly, or the census claims bodies
    /// the port refuses. `ROADMAP.md` §6d records what a disagreement between
    /// two copies of one predicate cost the last time.
    #[must_use]
    pub fn is_binary_int(self) -> bool {
        matches!(
            self,
            IlOp::Add
                | IlOp::Sub
                | IlOp::Mul
                | IlOp::And
                | IlOp::Or
                | IlOp::Xor
                | IlOp::Shl
                | IlOp::ShrS
                | IlOp::ShrU
        )
    }

    /// True for the binary operators the **bitwise/shift** rung added — the
    /// subset of [`IlOp::is_binary_int`] that `lane w-build` shipped.
    ///
    /// Kept apart from the arithmetic three because the guards differ: `+`/`-`
    /// scale over a pointer and these do not (they refuse a pointer outright
    /// instead), and the depth-2 tree selector accepts the arithmetic three and
    /// **not** these — see `try_select_depth2_tree`.
    #[must_use]
    pub fn is_bitwise_or_shift(self) -> bool {
        matches!(
            self,
            IlOp::And | IlOp::Or | IlOp::Xor | IlOp::Shl | IlOp::ShrS | IlOp::ShrU
        )
    }
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

// ---- W8: the two-arm conditional tail call --------------------------------

/// **One arm of a [`CondTailPair`]** — a tail call, resolved.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CondArm {
    /// The external callee's mangled name (from `.gl`).
    pub callee: String,
    /// What each argument slot wants; slot `j` is register `r(3+j)`. Only
    /// [`SlotArg::Formal`] and [`SlotArg::Lit`] occur — a data symbol's address
    /// inside an arm has no capture (its `lis` is hoisted ahead of the *whole*
    /// setup, and where that lands relative to a branch is unmeasured).
    pub slots: Vec<SlotArg>,
}

/// **W8 — the two-arm conditional tail call.** `docs/CFG_SHAPE.md` §4's minimal
/// instance, and the first shape in this crate whose lowering emits a branch.
///
/// ```cpp
///   void MemFree(void *v1, void *v2, unsigned long ul) {
///       if (v1 == nullptr) { XMemFree(v2, ul); return; }
///       RtlFreeHeap(v1, 0, v2);
///   }
/// ```
///
/// `then_arm` is the arm taken when the relation **holds**. The IL's `38` is
/// brFALSE, so the emitted `bc` carries the *negation* and jumps to `else_arm`;
/// the then-arm is the fall-through (`CFG_SHAPE.md` §3.4, and §1 prediction A3,
/// which scored RIGHT across ten cells).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CondTailPair {
    /// Which formal the condition compares, by index into
    /// [`IlFunction::params`].
    pub cmp_param: usize,
    /// The relation as written in the source. The emitted branch is its
    /// negation.
    pub rel: Rel,
    /// Whether the comparison is signed — `cmpwi` rather than `cmplwi`. From
    /// the operand TYPE triple and from nothing else; the relational opcodes are
    /// sign-agnostic (`docs/CFG_SHAPE.md` §3.2).
    pub signed: bool,
    /// The literal the formal is compared against; fits the 16-bit immediate.
    pub k: i32,
    pub then_arm: CondArm,
    pub else_arm: CondArm,
}

/// One instruction of a [`CondPlan`] block. Blocks are emitted in **descending
/// destination register** order, which is `moves_descending`'s incumbent rule
/// and which literals interleave into rather than being grouped after.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CondStep {
    /// `mr <dst>,<src>`.
    Move { dst: u8, src: u8 },
    /// `li <dst>,<k>`.
    Li { dst: u8, k: i32 },
    /// **W42** — `rlwinm <dst>,<src>,<sh>,<mb>,<me>`, the folded
    /// [`SlotArg::ShiftMask`]. `dst == src` in every cell measured; see
    /// [`plan_cond_pair`]'s rule 1b for why the out-of-place form is refused
    /// rather than emitted.
    Rlwinm { dst: u8, src: u8, sh: u8, mb: u8, me: u8 },
}

impl CondStep {
    fn dst(self) -> u8 {
        match self {
            CondStep::Move { dst, .. }
            | CondStep::Li { dst, .. }
            | CondStep::Rlwinm { dst, .. } => dst,
        }
    }
}

/// **The register schedule of a [`CondTailPair`]** — where every value is at
/// every point, as three blocks.
///
/// This is a *class predicate as much as an emitter input*: a body whose values
/// this cannot schedule must not census as in class, so the parser runs
/// [`plan_cond_pair`] as its last gate and the emitter runs the same function.
/// One decision procedure, the same discipline `CompareLeaf::out_of_class_ctx`
/// applies one shape down.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CondPlan {
    /// The entry block: the shuffles that must happen before the compare,
    /// because both successors need the value and at least one of them clobbers
    /// where it lives.
    pub entry: Vec<CondStep>,
    /// The register the `cmpwi`/`cmplwi` reads — the compared formal's location
    /// **after** the entry block, which is not always its home register
    /// (`?mmioGetInfo` parks its compared value in r11 and compares r11).
    pub cmp_reg: u8,
    /// The fall-through block (the relation holds).
    pub then_steps: Vec<CondStep>,
    /// The branch target block (the relation does not hold).
    pub else_steps: Vec<CondStep>,
}

/// The scratch register a value both arms need is parked in. Measured at r11 in
/// `?MemFree`, `?MemAlloc`, `?MemSize` and `?mmioGetInfo`; **only** at r11.
pub const COND_PARK_REG: u8 = 11;

/// **The register schedule, and the class boundary it draws.**
///
/// Derived from the three `xboxmem.cpp` functions plus `?mmioGetInfo`, and
/// stated as three rules:
///
/// 1. a formal **both arms want in the same register** has its move hoisted into
///    the entry block (`?MemAlloc`'s `mr r4,r5` — the then-arm passes `attrs` and
///    the else-arm passes `(attrs>>27)&8`, both in r4);
/// 2. a formal **both arms want in different registers**, whose home register is
///    clobbered by both arms, is **parked in r11** in the entry block
///    (`?MemFree`'s `mr r11,r4`: `v2` goes to r3 in one arm and r5 in the other);
/// 3. everything else stays in its arm.
///
/// Within every block the order is **descending destination register**.
///
/// **What is fitted and what is tested.** `docs/CFG_SHAPE.md` §8.1 **B3** says
/// plainly that the discriminator "needed on both paths" *fits* `?MemFree` and
/// `?MemAlloc` and is *tested by* neither, and this function does not pretend
/// otherwise. What it does instead is **verify** rather than trust: the schedule
/// it produces is simulated register by register and a schedule that does not
/// deliver every slot its value is refused, not emitted. So a shape the rules
/// mis-handle comes out as a refusal, which is a gap, instead of as a plausible
/// wrong branch, which is a mis-emit.
///
/// Returns `None` — a refusal — for anything outside that.
pub fn plan_cond_pair(
    n_params: usize,
    cmp_param: usize,
    then_slots: &[SlotArg],
    else_slots: &[SlotArg],
) -> Option<CondPlan> {
    /// The argument registers, by slot. Past the eighth an argument is
    /// stack-homed and its setup is a store, not a move.
    fn arg_reg(slot: usize) -> Option<u8> {
        (slot < 8).then(|| 3 + slot as u8)
    }
    let home = |i: usize| -> Option<u8> { arg_reg(i) };

    if cmp_param >= n_params {
        return None;
    }
    if then_slots.len() > 8 || else_slots.len() > 8 {
        return None;
    }
    // A data symbol's address inside an arm is out of class; so is a slot
    // sourcing a formal this function does not have.
    for s in then_slots.iter().chain(else_slots) {
        match s {
            SlotArg::Formal(f) if *f < n_params => {}
            SlotArg::ShiftMask { formal, .. } if *formal < n_params => {}
            SlotArg::Lit(_) => {}
            _ => return None,
        }
    }
    // Where each arm wants each formal. A formal wanted **twice** by one arm is
    // the shape `tail_call_shape` refuses under `call-arg-duplicated` — c2 emits
    // a dead `mr` through the temp, which no live-value-driven schedule
    // produces.
    //
    // A [`SlotArg::ShiftMask`] is a use of its source formal and is counted
    // here as one: `?MemAlloc`'s then-arm passes `attrs` and its else-arm passes
    // `(attrs>>27)&8`, **both in r4**, and it is exactly that agreement that
    // makes c2 hoist `mr r4,r5` into the entry block (rule 1).
    let want = |slots: &[SlotArg], i: usize| -> Option<Option<usize>> {
        let mut found = None;
        for (j, a) in slots.iter().enumerate() {
            let uses = match a {
                SlotArg::Formal(f) => *f == i,
                SlotArg::ShiftMask { formal, .. } => *formal == i,
                _ => false,
            };
            if uses {
                if found.is_some() {
                    return None;
                }
                found = Some(j);
            }
        }
        Some(found)
    };

    // `loc[i]` is where formal `i` lives once the entry block has run.
    let mut loc: Vec<u8> = (0..n_params).map(|i| home(i).unwrap_or(0)).collect();
    for i in 0..n_params {
        if home(i).is_none() {
            return None;
        }
    }
    let mut entry: Vec<CondStep> = Vec::new();
    let mut then_steps: Vec<CondStep> = Vec::new();
    let mut else_steps: Vec<CondStep> = Vec::new();
    let mut parks = 0usize;

    // Which argument registers each arm writes. Rule 2's precondition — "the
    // home register is clobbered by both arms" — is asked against these.
    let writes = |slots: &[SlotArg]| -> Vec<u8> {
        (0..slots.len()).filter_map(arg_reg).collect()
    };
    let then_writes = writes(then_slots);
    let else_writes = writes(else_slots);

    for i in 0..n_params {
        let h = home(i)?;
        let a = want(then_slots, i)?;
        let b = want(else_slots, i)?;
        match (a, b) {
            (Some(ja), Some(jb)) if ja == jb => {
                // Rule 1 — one destination, both arms: hoist the move.
                let d = arg_reg(ja)?;
                if d != h {
                    entry.push(CondStep::Move { dst: d, src: h });
                }
                loc[i] = d;
            }
            (Some(_), Some(_)) => {
                // Rule 2 — two destinations: park, but only where the witnesses
                // are. A home register neither arm overwrites needs no park at
                // all and each arm could simply read it; that shape has no
                // capture, so it is refused rather than guessed.
                if !(then_writes.contains(&h) && else_writes.contains(&h)) {
                    return None;
                }
                parks += 1;
                if parks > 1 {
                    // A second park would descend to r10 on a register model
                    // `docs/CODEGEN_W6_COMPARE.md` §6 records as demonstrably
                    // richer than a descending counter and NOT characterized.
                    return None;
                }
                entry.push(CondStep::Move { dst: COND_PARK_REG, src: h });
                loc[i] = COND_PARK_REG;
            }
            _ => {}
        }
    }

    // Rule 3 — the arm-local steps, from each formal's post-entry location.
    let arm = |slots: &[SlotArg], steps: &mut Vec<CondStep>| -> Option<()> {
        for (j, s) in slots.iter().enumerate() {
            let d = arg_reg(j)?;
            match s {
                SlotArg::Formal(f) => {
                    let src = *loc.get(*f)?;
                    if src != d {
                        steps.push(CondStep::Move { dst: d, src });
                    }
                }
                SlotArg::Lit(k) => {
                    if !(-0x8000..=0x7FFF).contains(k) {
                        return None;
                    }
                    steps.push(CondStep::Li { dst: d, k: *k });
                }
                SlotArg::ShiftMask { formal, sh, mb, me } => {
                    // **Rule 1b — the computed step is IN PLACE, or it is not
                    // this class.** `?MemAlloc` emits `rlwinm r4,r4,5,28,28`,
                    // reading the value rule 1 already hoisted into the
                    // destination register. The neighbouring cell where the
                    // then-arm does NOT want the same formal (`work/w-tu1/p/
                    // ma.cpp` `q2`) emits `mr r10,r5` at entry and then
                    // `rlwinm r4,r10,…` — a scratch this planner does not model
                    // and whose register `docs/CODEGEN_W6_COMPARE.md` §6 records
                    // as uncharacterized. Refused rather than guessed.
                    let src = *loc.get(*formal)?;
                    if src != d {
                        return None;
                    }
                    steps.push(CondStep::Rlwinm { dst: d, src, sh: *sh, mb: *mb, me: *me });
                }
                SlotArg::SymAddr => return None,
            }
        }
        Some(())
    };
    arm(then_slots, &mut then_steps)?;
    arm(else_slots, &mut else_steps)?;

    // Descending destination, in every block.
    let desc = |v: &mut Vec<CondStep>| v.sort_by(|x, y| y.dst().cmp(&x.dst()));
    desc(&mut entry);
    desc(&mut then_steps);
    desc(&mut else_steps);

    let plan = CondPlan {
        entry,
        cmp_reg: loc[cmp_param],
        then_steps,
        else_steps,
    };
    // **Verify, do not trust.** Simulate the schedule and refuse one that does
    // not deliver. See the doc comment: the placement rules are fitted, the
    // check is not.
    plan.verify(n_params, cmp_param, then_slots, else_slots)?;
    Some(plan)
}

/// An abstract register value, for [`CondPlan::verify`]. Formals and literals
/// are different value *spaces* on purpose: a slot must not be able to satisfy
/// itself with the right number of the wrong kind.
#[derive(Clone, Copy, PartialEq, Eq)]
enum CondVal {
    Formal(usize),
    Lit(i32),
    /// **W42** — a formal that has had one `rlwinm` applied. Its own space, so
    /// a slot wanting the raw formal cannot be satisfied by the shifted one or
    /// the other way round.
    Shifted(usize, u8, u8, u8),
}

impl CondPlan {
    /// Simulate the schedule and check it delivers. `None` in a register is
    /// "undefined", and reading one refuses.
    fn verify(
        &self,
        n_params: usize,
        cmp_param: usize,
        then_slots: &[SlotArg],
        else_slots: &[SlotArg],
    ) -> Option<()> {
        // r0..r11 is enough: every destination in class is an argument register
        // or [`COND_PARK_REG`].
        const NREG: usize = COND_PARK_REG as usize + 1;
        let mut regs: [Option<CondVal>; NREG] = [None; NREG];
        for i in 0..n_params {
            *regs.get_mut(3 + i)? = Some(CondVal::Formal(i));
        }
        fn run(regs: &mut [Option<CondVal>], steps: &[CondStep]) -> Option<()> {
            for s in steps {
                match *s {
                    CondStep::Move { dst, src } => {
                        let v = (*regs.get(src as usize)?)?;
                        *regs.get_mut(dst as usize)? = Some(v);
                    }
                    CondStep::Li { dst, k } => {
                        *regs.get_mut(dst as usize)? = Some(CondVal::Lit(k));
                    }
                    CondStep::Rlwinm { dst, src, sh, mb, me } => {
                        // The source must hold the RAW formal: a second fold on
                        // an already-folded value is a different expression and
                        // has no capture.
                        let CondVal::Formal(f) = (*regs.get(src as usize)?)? else {
                            return None;
                        };
                        *regs.get_mut(dst as usize)? = Some(CondVal::Shifted(f, sh, mb, me));
                    }
                }
            }
            Some(())
        }
        run(&mut regs, &self.entry)?;
        // The compare reads its register after the entry block, and it must hold
        // the formal it names.
        if (*regs.get(self.cmp_reg as usize)?)? != CondVal::Formal(cmp_param) {
            return None;
        }
        for (steps, slots) in [(&self.then_steps, then_slots), (&self.else_steps, else_slots)] {
            let mut r = regs;
            run(&mut r, steps)?;
            for (j, s) in slots.iter().enumerate() {
                let want = match s {
                    SlotArg::Formal(f) => CondVal::Formal(*f),
                    SlotArg::Lit(k) => CondVal::Lit(*k),
                    SlotArg::ShiftMask { formal, sh, mb, me } => {
                        CondVal::Shifted(*formal, *sh, *mb, *me)
                    }
                    SlotArg::SymAddr => return None,
                };
                if (*r.get(3 + j)?)? != want {
                    return None;
                }
            }
        }
        Some(())
    }
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
    /// for `g(7)`. Mutually exclusive with [`Self::arg_slots`].
    pub arg_ops: Vec<IlOp>,
    /// A 2+-argument call's argument slots, in slot order.
    ///
    /// **This was `Option<Vec<usize>>` — a bare permutation over the formals —
    /// until lane `w-memcpy`**, and the field was renamed rather than widened
    /// in place so that every one of its thirty-odd sites had to be visited by
    /// the compiler. A `Vec<usize>` and a `Vec<SlotArg>` of formals carry the
    /// same information, so a same-named widening would have compiled at the
    /// sites that map a slot to a register and been silently wrong at the ones
    /// that count moves.
    ///
    /// A [`SlotArg::Lit`] here is `callseq-multiarg-lit`, fenced to the class
    /// GRID-L measured (`work/w-memcpy/probeL`, 747 cells): a guarded early
    /// return, Class A, one call, at most one literal slot and at most one move
    /// left at the call site after the park. Everything wider is refused by
    /// name in `body::shapes::calls` — R-DESC, the rule that holds there at
    /// 416 of 416, is only 379 of 403 over the whole grid and its misses are
    /// the two-call and callee-saved drivers.
    pub arg_slots: Option<Vec<SlotArg>>,
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

/// The formal index each slot of `slots` is filled from, with a **literal slot
/// reading as a fixed point** — the value is already where it belongs, because
/// it is about to be materialized there by a `li`.
///
/// This is the view the argument permutation's own rules take: a cycle walk, a
/// `park_in_class` check and `c2_core::codegen::calls::seq_entry_park` all want
/// a permutation, and a literal slot participates in no cycle.
///
/// **It lives in this crate although only the backend walks the registers**,
/// for the reason [`LINK_FIRST_SLOT`] does: the IL parser uses it to decide
/// what is in class and the emitter uses it to lay out the moves, and those two
/// agreeing is the whole reason the census cannot claim a body the gate
/// declines. A second copy on the codegen side is `docs/GAPS.md` §6 #9 exactly
/// — one rule, two implementations, and the corpus only ever exercising the
/// correct one.
///
/// A [`SlotArg::SymAddr`] or [`SlotArg::ShiftMask`] also reads as a fixed
/// point; neither reaches a call shape that walks a permutation, and both are
/// refused by name where they could.
pub fn slot_sources(slots: &[SlotArg]) -> Vec<usize> {
    slots
        .iter()
        .enumerate()
        .map(|(slot, a)| match a {
            SlotArg::Formal(ix) => *ix,
            _ => slot,
        })
        .collect()
}

/// One explicit argument of a chain link ([`SeqCall::link_args`]) — the resolved
/// twin of `c2_il::func::body::SlotArg`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SlotArg {
    /// A formal, by index into [`IlFunction::params`]; it is in a callee-saved
    /// GPR by the time the link runs ([`CallSeq::saved`] says which).
    Formal(usize),
    /// A literal — `li r<slot>,k`, which costs no callee-saved register.
    Lit(i32),
    /// **WR1 — the address of the body's one named data symbol**, materialized
    /// into this slot by `lis r11,sym@ha ; addi r<slot>,r11,sym@l`.
    ///
    /// A unit variant with the name carried on [`IlFunction::data_sym`] rather
    /// than here, because the class admits **exactly one** such symbol per body
    /// and [`SlotArg`] is `Copy`. Two or more is `docs/IL_CALL_IN_EXPR.md` §17.3
    /// (a)/(b) — c2 materializes only the first through a relocation pair and
    /// derives the rest by `.rdata` pool-offset difference, which needs a
    /// whole-TU layout decision, and which symbol anchors is a fitted hypothesis
    /// with no mechanism. Refused, not modeled.
    SymAddr,
    /// **W42 — `(formal >> k) & m`, folded to one `rlwinm`.** The only
    /// *computed* slot in the vocabulary, and it is admitted by
    /// [`super::body::shapes::cond_tail`] alone; every other shape refuses it by
    /// name.
    ///
    /// `sh`/`mb`/`me` are the `rlwinm` fields, already derived — the fold is
    /// done at parse time so the census and the emitter cannot disagree about
    /// which instruction this is. See [`shift_mask_rlwinm`] for the derivation
    /// and for the two cells that bound it.
    ShiftMask {
        /// The source formal, by index into [`IlFunction::params`].
        formal: usize,
        /// `rlwinm`'s `SH`, i.e. the LEFT rotate — `32 - k` for a right shift
        /// of `k`.
        sh: u8,
        /// `rlwinm`'s `MB`, big-endian bit number of the mask's first bit.
        mb: u8,
        /// `rlwinm`'s `ME`, big-endian bit number of the mask's last bit.
        me: u8,
    },
}

/// **The `(x >> k) & m` fold, and the whole of what it is allowed to be.**
///
/// Returns `Some(None)` when the expression is provably **zero** — the mask
/// keeps no bit the shift left behind — and `Some(Some((sh, mb, me)))` for the
/// `rlwinm` that computes it. `None` is a refusal.
///
/// ```text
///   eff = m & ((1 << (32 - k)) - 1)     the bits the shift actually delivers
///   eff == 0            -> the value is the literal 0
///   eff not contiguous  -> REFUSED (no single rlwinm computes it)
///   otherwise           -> rlwinm rD,rS,32-k,31-hi,31-lo
/// ```
///
/// **Measured, not fitted: 70 cells through real `c2` at the workload's own
/// `/O1 /Oi /EHsc /GR` profile** — `k ∈ {1,4,8,16,24,27,31}` × `m ∈ {1, 2, 3,
/// 8, 12, 15, 255, 0x10, 0x10000, 0xFFFFFFF0}`, the whole cross product, every
/// one of them agreeing with the three lines above and **none** disagreeing
/// (`work/w-tu1/p/grid_sm.cpp`, re-runnable through `work/w-tu1/p/gradeo1.sh`).
/// Six of the seventy are the `eff == 0` collapse and c2 emits `li rD,0` for
/// each — with the *layout* reverting to the un-hoisted `?MemFree` shape,
/// because a literal is not a use of the formal. **That collapse is the
/// neighbouring-cell trap this fold has to survive**: `(at>>16)&0x10000` sits
/// between `(at>>16)&0x10` and `(at>>16)&0xFFFFFFF0`, both ordinary `rlwinm`s,
/// and reading it as one would emit a wrong instruction and a wrong block
/// layout at once.
///
/// The disassembler prints `rlwinm rA,rS,32-k,k,31` as `srwi rA,rS,k`, which is
/// the same word; six grid cells land there and are not exceptions.
pub fn shift_mask_rlwinm(k: u32, m: u32) -> Option<Option<(u8, u8, u8)>> {
    if k == 0 || k >= 32 {
        // `>> 0` is the identity (no instruction at all, a different slot kind)
        // and `>= 32` is undefined behaviour the front end may have folded.
        return None;
    }
    let eff = m & (((1u64 << (32 - k)) - 1) as u32);
    if eff == 0 {
        return Some(None);
    }
    let lo = eff.trailing_zeros();
    let hi = 31 - eff.leading_zeros();
    // Contiguity: one run of ones, no holes. `rlwinm`'s MB..ME with MB <= ME
    // expresses exactly that and nothing else.
    let run = (((1u64 << (hi - lo + 1)) - 1) << lo) as u32;
    if eff != run {
        return None;
    }
    Some(Some(((32 - k) as u8, (31 - hi) as u8, (31 - lo) as u8)))
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
/// **W10 — the guard on a [`CallSeq`]: the FRAMED × BRANCHING cell.**
///
/// The backend's view of [`crate::func::body::SeqGuardShape`]. The guarded call
/// is `calls[0]` and the join's first call is `calls[1]`.
///
/// This is the field that made the port's frame class and its branch class
/// intersect. Before it, `work/w-frame/RANKING.md` §4 measured 28 framed
/// functions and 2 branching ones emitted byte-exact, with **zero** in both
/// sets, and 10 of the 17 FRONTIER TUs needing the product.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SeqGuard {
    /// Index into [`CallSeq`]'s owning function's `params` of the compared
    /// formal. Its home argument register is what the compare reads: this
    /// production admits no entry-block move, so there is no post-hoist
    /// location to resolve.
    pub cmp_param: usize,
    /// The **source** relation; the emitted branch is its negation, because the
    /// IL's `38` is brFALSE (`docs/CFG_SHAPE.md` §1 prediction A3).
    pub rel: Rel,
    /// `cmpwi` when true, `cmplwi` when false.
    pub signed: bool,
    /// The comparison literal, inside the 16-bit immediate field.
    pub k: i32,
}

/// **W11 — one guarded EARLY RETURN ahead of a [`CallSeq`].**
///
/// The backend's view of [`crate::func::body::SeqEarlyReturnShape`]. A `CallSeq`
/// carries a `Vec` of these, in source order, all ahead of `calls[0]`.
///
/// This is the field that gives the port an **intra-section unconditional `b`**
/// (board #191) and its first real label→offset map — the two mechanisms
/// `work/w-conv/PREREG.md` §2 measures as wanted by 10 and 14 of the 17 FRONTIER
/// TUs respectively, and which nothing in the port had ever emitted.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SeqEarlyReturn {
    /// **W-SMALL — the short-circuit `&&`'s further conditions**, in source
    /// order, each `(cmp_param, rel, signed, k)` read exactly as the four fields
    /// below are. Empty for a plain single-test guard.
    ///
    /// Every one emits one more `cmp ; bc` at the **same** target with the
    /// **same** sense as the first — `419a` in both words, measured — so the
    /// consumer loops rather than branching on the count. `||` is a different
    /// shape and is refused in the parser, not here.
    pub and_conds: Vec<(usize, Rel, bool, i32)>,
    /// Index into the owning function's `params` of the compared formal, read in
    /// its home argument register.
    pub cmp_param: usize,
    /// The **source** relation. Whether the emitted branch negates it depends on
    /// `value`: a value arm is a real block so the branch steps past it and
    /// carries the negation; a void arm is empty, so c2 deletes the block and
    /// points the branch at the epilogue with the relation itself. See
    /// [`crate::func::body::SeqEarlyReturnShape`] for the measurement.
    pub rel: Rel,
    /// `cmpwi` when true, `cmplwi` when false.
    pub signed: bool,
    /// The comparison literal, inside the 16-bit immediate field.
    pub k: i32,
    /// The returned literal, or `None` for `return;`. Every exit value in one
    /// body is distinct, including [`SeqTail::Lit`]'s — the IL parser refuses
    /// the rest, because c2 merges arms that produce the same value.
    pub value: Option<i32>,
}

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
    /// **W10** — `Some` when the first call (and, with an `else` arm, the
    /// second) is guarded by a conditional branch. `None` for every sequence
    /// the Class A/B rungs shipped.
    pub guard: Option<SeqGuard>,
    /// **W11** — the guarded early returns written ahead of the sequence, in
    /// source order. Empty for every sequence the earlier rungs shipped, and
    /// never non-empty at the same time as `guard`: the two emit different
    /// blocks and the IL parser refuses the combination rather than
    /// interleaving two block plans.
    pub early: Vec<SeqEarlyReturn>,
    /// **Board #844 — THE COMPOSITION CARRIER.** The store run this sequence
    /// emits *before* its call, and the one fact about the call the run's
    /// schedule depends on. `None` for every sequence every earlier rung
    /// shipped, so no existing body changes shape by this field existing.
    ///
    /// # Why the run lives HERE and not in [`IlFunction::ops`]
    ///
    /// This is the whole of #844, and it is a *model* defect rather than a
    /// missing emitter. `IlFunction` carried an op stream **or** a call, and
    /// `c2_core::codegen::select_function` tries them in a **fixed order**:
    /// `call_seq` at position 2, `store_leaf_text` at position 10. A function
    /// carrying both therefore emitted the call sequence and **silently dropped
    /// the store run** — a complete, plausible, wrong body, which is board
    /// **#232**'s exact mechanism, and #232 was live for **255 commits** while
    /// the workload scan read `mismatch 0`.
    ///
    /// `w-f23` measured that and refused the composition in
    /// [`crate::func::bundle::shape_to_function`] rather than build the racing
    /// version. The repair is not "try the composition earlier" — an ordering
    /// fix leaves two fields that can both be set and one that wins. It is that
    /// **the composition is one carrier**: `ops` stays empty for this shape, the
    /// run is a field of the sequence it belongs to, and there is nothing for a
    /// dispatch order to get wrong. `IlFunction::store_run_is_carried_alone`
    /// asserts the invariant and `select_function` refuses a violation by name.
    ///
    /// # The class this is admitted on
    ///
    /// Exactly [`crate::func::body::shapes::try_parse_store_run_call`]'s: one
    /// call, an **empty** argument setup (board #1129 — every slot already holds
    /// the formal that occupies it, so the run's base register is never
    /// written), the **constructor** tail (board #869/#1131 — the `void`,
    /// `return <call>` and discarded-`int` forms are frame words 0 and
    /// tail-call *behind* the run), and `saved = [0]` because `this` is the one
    /// value live across the one call. The emitter restates each of those as a
    /// backstop rather than trusting the parser (`codegen::store_run_call`).
    pub store_run: Option<StoreRunPrefix>,
}

/// **Board #844** — the store run a [`CallSeq`] emits *before* its call, and the
/// one fact about the call that the run's schedule turns out to depend on.
///
/// # Why `live_args` is here, and why it is not derivable
///
/// The composition is admitted only when the call's argument setup is **empty**
/// (board #1129: every slot `i` already holds `params[i]`), so [`SeqCall`] has
/// nothing in `arg_ops` and the emitter cannot see how many arguments the call
/// takes. That looked like a fact nobody needed — and then it turned out to
/// decide the run's **order**:
///
/// ```text
///   void P::lf(unsigned a, unsigned b) { m0=0; m1=b; m2=a; }         the LEAF
///       li 11,0 ; stw 5,4(3) ; stw 4,8(3) ; stw 11,0(3) ; blr
///
///   P::P(unsigned a, unsigned b) { m0=0; m1=b; m2=a; Alloc(a); }     FRAMED
///       li 11,0 ; stw 4,8(3) ; stw 5,4(3) ; mr 31,3 ; stw 11,0(3) ; bl
///
///   P::P(unsigned a, unsigned b) { m0=0; m1=b; m2=a; Reset(); }      FRAMED,
///       li 11,0 ; stw 5,4(3) ; stw 4,8(3) ; mr 31,3 ; stw 11,0(3) ; bl  nullary
/// ```
///
/// **The two unproduced stores swap — and only when the call passes `a`.** `a`
/// is live until the `bl`; `b` dies at its own store. With a nullary callee
/// nothing is kept alive and the run is the leaf's, word for word. So *"the leaf
/// schedule transfers unchanged into a framed body"* — board **#866** over 96
/// cells, and 34 more in `w-seam2`'s GRID S — is **true only while no store
/// reads a value the call keeps alive**, and this field is what lets the emitter
/// tell. `work/w-seam2/grid3/` is the twelve-cell probe that separated it and
/// `work/w-seam2/grid2/` is where it fired first, on seven cells at once.
///
/// `live_args` counts the argument slots **including the receiver at 0**, and
/// the receiver is exempt: `this` is the store base and is copied to `r31`
/// regardless, and storing it transfers on every measured cell (`p6`, `p11`, and
/// every `w3` cell of GRID S). It is the slots `>= 1` that break the run.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StoreRunPrefix {
    /// The run's op stream, exactly as [`crate::func::body::BodyShape::StoreRun`]
    /// carries it: one `[Load(base), Load(formal) | Lit(k), StoreInd { off,
    /// width }]` group per statement, in source order.
    pub ops: Vec<IlOp>,
    /// How many argument slots the call occupies, **receiver included**. Slot
    /// `i` holds `params[i]` by the production's own gate, so this is exactly
    /// "which formals are still live at the `bl`".
    pub live_args: usize,
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

/// **W43 — `return ((unsigned)(P != 0) << SH) | C;`**, the comparison leaf's
/// `!=`-against-zero fold with a constant ORed into a field the shift leaves
/// empty. `?GetXAllocAttributes@NUISPEECH@@YAKH@Z` from
/// `src/xdk/nuispeech/xboxmem.cpp`, and the first body in the port whose
/// selection depends on a property of a **literal's bit pattern**.
///
/// ```text
///   addic  r11,r3,-1              the `!= 0` fold, unchanged from W6
///   lis    r10,C>>16              the constant, high half only
///   subfe  rS,r11,r3              rS = r11 at /O1, r9 at /Ox
///   rlwimi r10,rS,SH,0,31-SH      the OR and the shift, in ONE instruction
///   mr     r3,r10
///   blr
/// ```
///
/// The `/O1` register is **not a new rule**: it is `docs/CODEGEN_W6_O1.md`'s
/// incumbent — *a temp whose defining instruction makes the last use of the
/// value in r11 is written to r11 instead of taking a fresh descending number* —
/// which `compare_leaf_text` already applies across a 108-cell matrix. Here the
/// `subfe` is the last use of the `addic` result, so it takes r11; at `/Ox` the
/// descending counter has already spent r11 and r10 and hands out r9.
/// **The pointer-walk accumulate loop** — the port's first body class with a
/// back edge, and `src/system/math/Sort.cpp`'s whole content.
///
/// ```c
///   int P(const char *str, int i) {
///       int ret = <acc_init>;
///       for (unsigned char *u = (unsigned char *)str; *u != 0; u++)
///           ret = (*u + ret * <mul_k>) % i;
///       return ret;
///   }
/// ```
///
/// The recognizer
/// ([`crate::func::body::shapes::ptr_walk_loop::try_parse_ptr_walk_loop`])
/// carries the whole accept/refuse boundary, including the two facts that are
/// *not* visible in this struct because they are required literally:
/// `params.len() == 2` with the pointer at slot 0 and the divisor at slot 1, and
/// a stride of exactly 1. Both re-plan `c2`'s register assignment when varied,
/// with the measured counterexamples named in the recognizer's module docs.
///
/// There is no label field and no callee: the emitted body takes **no
/// relocation, mints no symbol and defines no label** — every branch in it is
/// self-relative. That is why it reaches codegen as an ordinary
/// `Selected::Plain` and needed no new obj shape.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PtrWalkModLoop {
    /// The two formals, in register order: `params[0]` is the walked pointer's
    /// source (r3), `params[1]` the modulo's divisor (r4).
    pub params: Vec<u32>,
    /// The accumulator's initial literal, inside `simm16` — one `li`.
    pub acc_init: i32,
    /// The multiplier, restricted to the `mulli`-eligible **positive** literals
    /// (`ptr_walk_loop::is_mulli_literal` plus `> 0`; see there for the 38-cell
    /// grid and for why the sign is a second clause and not part of the first).
    pub mul_k: i32,
}

/// **W-CFG1 — the two-armed `if`/`else` whose arms are CALLS.** The port's first
/// `cflow-if-n` body class.
///
/// ```c
///   const Node* f(Blend b, void *clip, float t) {
///       const Node* n = <acc_init>;
///       if (b >= <k1>) {
///           if (b == <k1>) { n = <acc_init>; }   // the DEAD arm
///           else {
///               if (b >= <k2>) n = <callee_hi>(clip, t);
///               else           n = <callee_lo>(clip, t);
///           }
///       }
///       return n;
///   }
/// ```
///
/// The recognizer
/// ([`crate::func::body::shapes::if_call_join::try_parse_if_call_join`]) carries
/// the whole accept/refuse boundary, including the facts that are *not* visible
/// in this struct because they are required literally: three formals of kinds
/// `(int-like, ptr4, float)` in that order, both arms calling with the **same**
/// two arguments, the dead arm storing the **same** literal the entry stored,
/// and both scope-close depths of every block pinned. Each one re-plans c2's
/// register assignment or its block layout when varied.
///
/// Unlike [`PtrWalkModLoop`] this body **is framed**: it gets a `.pdata` record,
/// a `$M`/`$M`/`$T` triple and two REL24 sites, so it reaches codegen as its own
/// [`Selected`](../../c2_core/codegen/enum.Selected.html) variant and not as a
/// `Plain`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IfCallJoin {
    /// The three formals in **argument-register order**: `params[0]` the
    /// scrutinee (r3, evicted to r10 by the entry park), `params[1]` the pointer
    /// the arms share (r4, hoisted to r3 above every branch), `params[2]` the
    /// `float` (fr1, untouched — it emits no setup word at all).
    pub params: Vec<u32>,
    /// The outer and middle tests' shared literal, inside `simm16` — one `cmpwi`
    /// serves both, which is the fact that makes this a transcription.
    pub k1: i32,
    /// The inner test's literal, inside `simm16`.
    pub k2: i32,
    /// The accumulator's initial literal — stored twice in the IL (entry block
    /// and dead arm) and emitted once, which is why c2's middle block is empty.
    pub acc_init: i32,
    /// The `.gl` token of the callee taken when `s >= k2`. Resolved to a name in
    /// [`crate::IlBundle`].
    pub callee_hi_tok: u32,
    /// The `.gl` token of the callee taken when `s < k2`.
    pub callee_lo_tok: u32,
}

/// [`IfCallJoin`] with its two callee tokens **resolved** through the `.gl`
/// symbol index — the form the emitter consumes.
///
/// A separate type rather than a mutated field for the reason
/// `docs/GAPS.md` §6 gives: a token and a symbol name are two facts, and one
/// field holding either is where they silently disagree.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IfCallJoinFn {
    /// The three formals, exactly as [`IfCallJoin::params`].
    pub params: Vec<u32>,
    /// The shared literal of the outer and middle tests.
    pub k1: i32,
    /// The inner test's literal.
    pub k2: i32,
    /// The accumulator's initial literal.
    pub acc_init: i32,
    /// The callee taken when `s >= k2` — the FIRST `bl` in block order, so the
    /// first REL24 site and the earlier symbol in the per-function region.
    pub callee_hi: String,
    /// The callee taken when `s < k2`.
    pub callee_lo: String,
}

/// The right-hand operand of one accumulate-chain step.
///
/// Two cases and no third: the chain either folds in a literal or folds in the
/// walked character. That is the whole operand vocabulary
/// [`PtrWalkChainLoop`]'s recognizer admits, and it is what makes the emitter's
/// `pv` — the last step reading the character — an **IL** fact rather than
/// something read back out of emitted registers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChainRhs {
    /// An immediate. Its admissible range depends on the opcode and is checked
    /// by the recognizer, not here (`addi`/`mulli` take `simm16`, `xori`/`ori`
    /// take `uimm16`).
    Lit(i32),
    /// The walked character — `c` in `int c = *s;`.
    Char,
}

/// The operator of one accumulate-chain step.
///
/// **Four, and the omissions are measured rather than cautious.** Each admitted
/// kind selects to exactly one instruction in both operand shapes, and every
/// omitted one was excluded by a captured counterexample:
///
/// ```text
///   r = r & K   ->  andi. rD,rS,K   -- WRITES cr0.  c2 then demotes the record
///                                      form to a plain `extsb` and adds an
///                                      explicit `cmpwi r11,0` before the back
///                                      edge: a DIFFERENT block, one word longer
///   r = r - c   ->  subf            -- non-commutative; its operand roles come
///                                      from instruction selection, so S5 does
///                                      not speak for it (w-sched2 §6.5, seven
///                                      refused cells)
///   r = r << K  ->  rlwinm          -- reassociates and folds the length axis
///                                      (w-sched2 §11.4, unchased)
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChainOpKind {
    /// `add` / `addi`.
    Add,
    /// `xor` / `xori`.
    Xor,
    /// `or` / `ori`.
    Or,
    /// `mullw` / `mulli`.
    Mul,
}

/// One step of the accumulate chain: `r = r <kind> <rhs>`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ChainOp {
    pub kind: ChainOpKind,
    pub rhs: ChainRhs,
}

/// **The pointer-walk accumulate loop, body-parameterized** — the port's first
/// lowering whose emitted body has no fixed length.
///
/// ```c
///   int P(const char* s) {
///       int r = <acc_init>;
///       while (*s) { int c = *s; r = <ops[0]>; r = <ops[1]>; … s++; }
///       return r;
///   }
/// ```
///
/// # Why this is a different shape from [`PtrWalkModLoop`] and not a widening
///
/// `PtrWalkModLoop` carries three scalars and **no operation list at all**; its
/// recognizer consumes the accumulate with literal `eat_byte(0x04)` calls at
/// fixed cursor positions and its emitter hand-writes twenty words behind a
/// `debug_assert_eq!(t.len(), 80)`. It is a transcription of one function, says
/// so in its own module doc, and is left exactly as it is — it additionally
/// carries the signed `%` spine, which belongs to lane `w-divmod` and is
/// treated here as a black box.
///
/// This shape carries `ops`, and everything about the emitted body — its
/// length, the induction load's slot, the record form's slot, every register
/// field and the back edge's displacement — is **computed from that list** by
/// the rules `docs/rungs/2026-08-05-w-sched2.md` measured and
/// `docs/rungs/2026-08-05-w-rotate.md` §3 completed.
///
/// # The fields are the emitter's whole input
///
/// The recognizer
/// ([`crate::func::body::shapes::ptr_walk_chain_loop::try_parse_ptr_walk_chain_loop`])
/// carries the accept/refuse boundary, including the facts not visible here
/// because they are required literally: **exactly one formal**, the walked
/// pointer, at slot 0; a stride of exactly 1; the loop test a bare truth test
/// on the dereference; and at least one chain step reading the character.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PtrWalkChainLoop {
    /// The one formal: the walked pointer's source, arriving in r3.
    pub params: Vec<u32>,
    /// The accumulator's initial literal, inside `simm16` — one `li`.
    pub acc_init: i32,
    /// The element's signedness, which decides the **record form** and the
    /// entry test. Measured, both regimes
    /// (`work/w-varloop/probe.py --sig 'const unsigned char* s'`):
    ///
    /// ```text
    ///   signed             record `extsb. CHAR,LD`   entry test `extsb. CHAR,CHAR`
    ///   unsigned, TWO      record `mr.    CHAR,LD`   entry test `cmplwi cr0,CHAR,0`
    ///   unsigned, SAME     record `cmplwi cr0,CHAR,0`   -- no entry test at all
    /// ```
    ///
    /// The third row is a fact w-sched2's reconstruction never had to derive:
    /// it copied the record form's opcode out of `c2`'s own bytes. An emitter
    /// must choose it, and in the SAME regime `mr. CHAR,CHAR` would be the
    /// redundant move `c2` declines to emit.
    pub elem_unsigned: bool,
    /// **The operation list** — the accumulate in data-dependence order, one
    /// entry per source statement. Never empty.
    pub ops: Vec<ChainOp>,
}

impl PtrWalkChainLoop {
    /// `M` — the producer count. One IL step is one producer here **by
    /// construction**, which is how this shape stays clear of board #644: a
    /// producer split across a `lis`/`ori` pair is exactly what the
    /// recognizer's literal-range checks refuse, so `M == N` always holds and
    /// the two units w-sched2 §5 had to distinguish coincide.
    pub fn producers(&self) -> usize {
        self.ops.len()
    }

    /// `pv` — the index of the **last** chain step reading the character's
    /// value. `None` when no step reads it, which the recognizer refuses.
    ///
    /// w-sched2 computes this from emitted bytes with a liveness flag, because
    /// a physical register can be re-read after the value in it has died. Here
    /// it is a plain IL fact: `c` is assigned once and never reassigned, so
    /// every `Char` operand is a read of the live value.
    pub fn pv(&self) -> Option<usize> {
        self.ops.iter().rposition(|o| o.rhs == ChainRhs::Char)
    }

    /// **S3m** — the register regime, at 84 of 84 held out
    /// (`docs/rungs/2026-08-05-w-sched2.md` §3.3). `true` is the SAME regime:
    /// the induction load reuses the character's register, so the loop runs on
    /// one register and is entered by jumping *into* the record form.
    ///
    /// Both inputs are IL facts, which is the finding that made a lowering
    /// reachable at all — w-rotate had left the entry form unevaluable from IL.
    pub fn regime_same(&self) -> bool {
        self.pv() == Some(0) && self.producers() >= 4
    }
}

/// **The integer divide / modulo leaf** — `return a / b;` / `return a % b;`
/// over exactly two formals, signed or unsigned.
///
/// Four bodies, three to nine words, and **no free fields at all**: the two
/// booleans below select one of four constant schedules per optimization mode
/// and everything in each of them — every register, both trap `TO` fields, the
/// operand order of the closing `subf` — is a constant of the class. The
/// register plan and the `TO` fields were read off real `c2.dll`'s own words
/// (`work/w-divmod/twigrid.py --dis`), not paraphrased from a mnemonic table.
///
/// The recognizer
/// ([`crate::func::body::shapes::div_mod_leaf::try_parse_div_mod_leaf`]) holds
/// the whole accept/refuse boundary, including the facts that are not visible
/// here because they are required literally: `params.len() == 2`, the dividend
/// at slot 0, the divisor at slot 1, both operands loaded straight from
/// formals, and the type triple identical in all three positions. Each of those
/// has a measured counterexample named in that module's docs.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DivModLeaf {
    /// The two formals in register order: `params[0]` is the dividend (r3),
    /// `params[1]` the divisor (r4).
    pub params: Vec<u32>,
    /// `%` when true, `/` when false — IL bytes `06` and `05`. They are
    /// **different lengths**, not a flag on one body: signed `%` is nine words
    /// and signed `/` is seven, because `/` needs neither the `mullw` nor the
    /// `subf`.
    pub is_mod: bool,
    /// Signedness, which selects `divw`+two traps or `divwu`+**one**. The
    /// `INT_MIN / -1` overflow cannot arise unsigned, so `c2` emits no
    /// predicate and no `twi 5` at all — `unsigned / unsigned` is three words.
    pub signed: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CmpShiftOr {
    /// The compared formal's IL token. It is the function's ONLY formal and
    /// occupies r3.
    pub param: u32,
    /// Whether the compared operand is signed. `!= 0` emits the same two words
    /// either way (`CompareLeaf`'s `(Rel::Ne, _)` arm); carried so the census
    /// key and the type gate agree.
    pub signed: bool,
    /// The left shift, 1..=31.
    pub sh: u8,
    /// The ORed constant. Low 16 bits zero, and `sh > msb(C)` — see
    /// [`shift_or_rlwimi`].
    pub c: u32,
}

/// **The `((x != 0) << SH) | C` -> `lis` + `rlwimi` selection, and exactly how
/// narrow it is.** Returns the `rlwimi` mask `(mb, me)`, or `None` — a refusal.
///
/// c2 has (at least) three lowerings for this expression and the choice is a
/// function of `C`'s bit pattern, not of `SH` alone:
///
/// ```text
///   slwi rT,rS,SH ; oris r3,rT,C>>16      C's low half is 0 and SH <= msb(C)
///   slwi rT,rS,SH ; ori  r3,rT,C          C's high half is 0
///   lis rT,C>>16 ; rlwimi rT,rS,SH,0,31-SH ; mr r3,rT
/// ```
///
/// **Measured: 288 cells** — `C ∈ {0x80000000, 0x40000000, 0x249b0000,
/// 0x10000000, 0x08000000, 0x00030000, 0x00010000, 0x0000ffff, 0x00000004}` ×
/// `SH ∈ 0..=31`, compiled by real `c2` at the workload's own `/O1 /Oi /EHsc
/// /GR` (`work/w-tu1/p/grid_kc.cpp`). The `rlwimi` region is **not** simply
/// "SH >= 32 - lz(C)": two rows of that grid disagree with every clean rule this
/// lane could state — `C = 0x80000000` takes `rlwimi` for SH 1..30 and something
/// else entirely at 31, and `C = 0x00030000` crosses one column early.
///
/// So this predicate does **not** claim the boundary. It claims a **region
/// strictly inside it**, `C_low16 == 0 && SH > msb(C)`, on which all 288 cells
/// agree with the three-instruction form and **none** disagrees — including both
/// anomalous rows, which the predicate excludes rather than explains
/// (`0x80000000` has `msb = 31`, so the region is empty there). A further 62
/// cells of that region were compared **word for word**, not just by mnemonic,
/// and 57 of the 62 matched exactly; the 5 that did not are the
/// parameter-position axis, which is why the class requires the compared formal
/// to be the function's only parameter — see `w43_cmp_shift_or_neg.cpp`.
///
/// Inside the region `31 - SH < lz(C)`, so the mask is always `0 .. 31-SH` and
/// the `min` that a general rule would need never binds.
pub fn shift_or_rlwimi(sh: u8, c: u32) -> Option<(u8, u8)> {
    if sh == 0 || sh > 31 || c == 0 {
        return None;
    }
    if c & 0xFFFF != 0 {
        // `lis` alone does not materialize it. c2 reaches for `ori`/`li` and a
        // two-instruction form on some of those cells and `rlwimi` on others;
        // the grid does not separate them. Refused.
        return None;
    }
    let msb = 31 - c.leading_zeros();
    if u32::from(sh) <= msb {
        // At or below `C`'s top set bit c2 emits `slwi` + `oris`, two
        // instructions and no `mr`. A different body, not a different register.
        return None;
    }
    Some((0, 31 - sh))
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
    /// If this function is a **W8 two-arm conditional tail call**, the decoded
    /// branch and both arms. Mutually exclusive with the other body kinds, and
    /// the only one of them whose lowering emits a `bc`.
    pub cond_pair: Option<CondTailPair>,
    /// If this function is a **comparison leaf** (`return a <rel> k;`, W6), the
    /// decoded comparison. Mutually exclusive with the other body kinds.
    pub compare: Option<CompareLeaf>,
    /// **W43** — if this function is `return ((unsigned)(P != 0) << SH) | C;`.
    /// Mutually exclusive with the other body kinds, [`Self::compare`]
    /// included: the two share a spine but not a body.
    pub cmp_shift_or: Option<CmpShiftOr>,
    /// If this function is the **pointer-walk accumulate loop**, its parameters.
    /// Mutually exclusive with the other body kinds, and the only one whose
    /// lowering emits a **backward** branch.
    ///
    /// It makes [`Self::label_slots`] return `None`: `w-loop` measured that a
    /// leaf loop charges the compiler-label counter `+1..+4` — never 1 — and
    /// which of the four is a function of the *source* loop shape, not of the
    /// emitted bytes (three source shapes emit the identical 24 bytes and charge
    /// `+1`, `+3`, `+1`). So the charge is real, is not derivable from anything
    /// this struct carries, and is **unobservable in a TU with no framed
    /// function**, which is the only kind of TU this shape may appear in. Board
    /// **#746**.
    pub ptr_walk_loop: Option<PtrWalkModLoop>,
    /// If this function is the **`if`/`else`-with-a-join whose arms are calls**,
    /// its formals, its two literals and its two resolved callees. See
    /// [`IfCallJoinFn`].
    ///
    /// Mutually exclusive with every other body kind — exactly one parser
    /// production sets it. Unlike [`Self::ptr_walk_loop`] it **is framed**, so
    /// [`Self::label_slots`] answers with a real number rather than refusing.
    pub if_call_join: Option<IfCallJoinFn>,
    /// If this function is the **body-parameterized pointer-walk loop**, its
    /// one formal and its accumulate **operation list**. See
    /// [`PtrWalkChainLoop`].
    ///
    /// Mutually exclusive with [`Self::ptr_walk_loop`]: the two recognizers
    /// accept disjoint grammars (a rotated `for` against a top-test `while`),
    /// and exactly one parser production sets each field.
    pub ptr_walk_chain_loop: Option<PtrWalkChainLoop>,
    /// If this function is the **integer divide/modulo leaf**, its two formals
    /// and the operator. See [`DivModLeaf`].
    ///
    /// Unlike [`Self::ptr_walk_loop`] this body is a **single basic block** and
    /// emits no branch at all, so it takes no compiler-label slot and
    /// [`Self::label_slots`] treats it like every other leaf.
    pub div_mod_leaf: Option<DivModLeaf>,
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
    /// A **multi-argument** tail call's argument slots. `Some(slots)` means this
    /// is `return g(a1, …, an)` with `n >= 2` and every argument either a bare
    /// parameter or (WLA) a literal: `slots[i]` is what argument slot `i`
    /// (register `r(3+i)`) wants — [`SlotArg::Formal`] indexes [`Self::params`],
    /// [`SlotArg::Lit`] is one `li r(3+i),k`. Set together with
    /// [`Self::tail_call`], and then [`Self::ops`] is empty — the slot list, not
    /// an operand stream, is the whole argument setup.
    ///
    /// The one-argument case keeps using `ops` instead, because it can carry a
    /// computed argument (`g(a + 1)`) that this form cannot express.
    pub arg_sources: Option<Vec<SlotArg>>,
    /// **WR1 — the one named data symbol whose address this body materializes**,
    /// already resolved through `.gl` to its mangled name (`?gI@@3HA`).
    ///
    /// `Some` exactly when [`Self::arg_sources`] contains a [`SlotArg::SymAddr`].
    /// The emitter turns it into an **undefined-external DATA** symbol
    /// (`Type` 0x0000, not the 0x0020 a callee carries) plus a REFHI/PAIR/REFLO/PAIR
    /// quad; the TU-level accounting in [`IlBundle::functions`] counts it as
    /// referenced so the unclaimed-`.gl`-name gate does not refuse the TU for a
    /// symbol the obj legitimately carries.
    ///
    /// **A defined or static global never reaches here.** It puts a `.data`/`.bss`
    /// section in the middle of the section table, so the `.gl` linkage byte
    /// refuses it (`docs/IL_CALL_IN_EXPR.md` §17.2 item 7,
    /// `gl::gl_extern_data_names`) and the name stays unaccounted, which refuses
    /// the TU as well. Two gates, because the failure is a wrong section count and
    /// not a wrong instruction.
    pub data_sym: Option<String>,
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
            cond_pair: None,
            compare: None,
            cmp_shift_or: None,
            ptr_walk_loop: None,
            if_call_join: None,
            ptr_walk_chain_loop: None,
            div_mod_leaf: None,
            float_leaf: None,
            fp_tail: None,
            fp_arg_sources: None,
            arg_sources: None,
            data_sym: None,
            empty_body: false,
            eh_bare: false,
            eh_unwind_callees: Vec::new(),
        }
    }

    /// **Board #844's invariant: a carried store run is carried ALONE.**
    ///
    /// True when [`CallSeq::store_run`] is non-empty and [`Self::ops`] is also
    /// non-empty — i.e. the function is spelling the composition **twice**, once
    /// in the carrier and once in the field `c2_core::codegen::select_function`
    /// reaches through a different arm. That is the state board #232 was: two
    /// fields, one dispatch order, and whichever the order reaches first wins
    /// while the other is silently dropped.
    ///
    /// No production sets both — `shape_to_function`'s `StoreRunCall` arm puts
    /// the run in the carrier and leaves `ops` at `base()`'s empty default — so
    /// this is a **backstop**, and it is here rather than in the emitter because
    /// the invariant is a property of the model. `select_function` consults it
    /// and refuses by name; refusing is the whole point, because the alternative
    /// to refusing is picking a winner, and picking a winner is the defect.
    pub fn store_run_carried_twice(&self) -> bool {
        !self.ops.is_empty()
            && self
                .call_seq
                .as_ref()
                .is_some_and(|s| s.store_run.is_some())
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
            // **W-CFG1 is FP-touching without being FP-SHAPED** — exactly the
            // miss this doc-comment warns about. Its `float` formal travels in
            // fr1 and emits no instruction at all, so a predicate written as
            // "does the body have FP ops" answers no; the obj still carries
            // `_fltused` (`src/system/negate_test.cpp` symbol 21), and without
            // it every positive case is one symbol short at COFF offset 12.
            // The recognizer requires the third formal to be a 4-byte real, so
            // this is a structural fact of the class and not a body scan.
            || self.if_call_join.is_some()
    }

    /// True iff this function establishes a **stack frame** — it gets a `.pdata`
    /// record, a `$M`/`$M`/`$T` label triple, and the framed label stride.
    ///
    /// One predicate, asked by every TU-level gate that cares, so adding a framed
    /// shape cannot leave one of them behind. Both framed shapes are non-leaf
    /// calls whose result (or whose successor statement) outlives the `bl`.
    pub fn is_framed(&self) -> bool {
        self.framed_call.is_some() || self.call_seq.is_some() || self.if_call_join.is_some()
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
        self.call_seq.as_ref().map_or(0, |s| s.tail.label_lead())
            + u32::from(self.eh_bare)
            // **W-CFG1 charges ONE slot before its own triple**, so a framed
            // function of this class strides 6 under `/Gy` where every other
            // framed class strides 5. Measured seed-free and in-TU on
            // `src/system/negate_test.cpp`, whose two functions are this class
            // and nothing else: `$M2581`/`$M2582`/`$T2583` then
            // `$M2587`/`$M2588`/`$T2589` — a difference of **6**, with the
            // `_fltused` slot charged once for the TU and therefore cancelling
            // out of the difference entirely. `scripts/gt_label_stride.py`'s
            // anchor control is the shipped 5 on a `Seq` body in the same TU
            // (`fixtures/cpp/wcfg1_join_then_seq.cpp`).
            + u32::from(self.if_call_join.is_some())
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
            // W8: both arms, in BLOCK order — the then-arm's `b` is emitted
            // first, so its REL24 site is the lower offset and its symbol the
            // earlier one in the per-function region.
            .chain(
                self.cond_pair
                    .iter()
                    .flat_map(|c| [c.then_arm.callee.as_str(), c.else_arm.callee.as_str()]),
            )
            // W-CFG1: both arms, in BLOCK order — the `s >= k2` arm's `bl` is at
            // the lower `.text` offset, so its symbol is the earlier one in the
            // per-function region for the same reason W8's then-arm is.
            .chain(
                self.if_call_join
                    .iter()
                    .flat_map(|c| [c.callee_hi.as_str(), c.callee_lo.as_str()]),
            )
    }

    pub fn label_slots(&self, fn_level_linking: bool) -> Option<u32> {
        if self.framed_call.is_some() || self.call_seq.is_some() || self.if_call_join.is_some() {
            return Some(self.label_lead() + if fn_level_linking { 5 } else { 4 });
        }
        // **The pointer-walk loop refuses, and the refusal is the measurement,
        // not caution.** `w-loop` read the *leaf* loop stride seed-free over 17
        // probes with a 5/5 anchor control on every row: `while` +2, `do/while`
        // +1, `for` +2, `for(;;)`+`break` +3, nested +4, `Sort.cpp`'s own
        // pointer-walk shape **+3** — against `leaf-none` = 1. So a loop leaf is
        // never 1, and *which* of the four it is cannot be read off the emitted
        // bytes: `do/while`, `for(;;)`+`break` and a backward `goto` emit the
        // **identical 24 bytes** and charge +1, +3, +1.
        //
        // `Some(4)` for the +3 measured here would therefore be a rule fitted on
        // one source spelling of a class whose members are indistinguishable at
        // the only place this port can look. `None` refuses instead, and the
        // three-valued gate in [`crate::IlBundle::functions`] turns that into:
        // **a TU pairing this shape with a framed function is rejected; a TU
        // with no framed function is admitted**, which is exactly the boundary
        // `w-loop` §5.1 measured (34 of 34 leaf-only TUs mint zero labels, 28 of
        // them carrying a backward branch; control 17 of 17). Board **#746**,
        // and `fixtures/cpp/whash_loop_then_framed.cpp` is board **#747** — the
        // two-function TU of mixed frame class neither `expr_sweep.sh` nor
        // `mode_cross.sh` can generate.
        // **MUST-FAIL MUTATION, verified.** Replacing this `None` with
        // `Some(1)` — the ordinary leaf charge — turns
        // `fixtures/cpp/whash_loop_then_framed.cpp` from `NotImplemented` into a
        // live `mismatch` against real `c2.dll`, while its separating control
        // `fixtures/cpp/whash_ptr_walk_loop.cpp` (the identical loop with no
        // framed function beside it) stays `match`. Real `c2` mints
        // `$M2564`/`$M2565`/`$T2566` for the framed `?z9`; the mutated port
        // charges the loop 1 where `c2` charges 4, so the triple lands three
        // low — six wrong bytes in an obj that still links, board #263's shape.
        // Neither `expr_sweep.sh` nor `mode_cross.sh` can generate that TU
        // (board #747), so the fixture is the only thing that grades it.
        if self.ptr_walk_loop.is_some() {
            return None;
        }
        // **The body-parameterized loop refuses for exactly the same reason**,
        // and the reason is unchanged by the body's length: `w-loop` measured
        // that *which* of the four loop charges applies cannot be read off the
        // emitted bytes, and a `while` charging +2 emits words indistinguishable
        // from a `do/while` charging +1. The chain's length does not enter that
        // argument at all, so a variable-length body inherits the same `None`.
        //
        // **MUST-FAIL MUTATION, verified** — the same shape as the one above:
        // replacing this `None` with `Some(1)` turns
        // `fixtures/cpp/wvl_chain_then_framed.cpp` from `NotImplemented` into a
        // live `mismatch` against real `c2.dll`, while its separating control
        // `fixtures/cpp/wvl_chain3.cpp` (the identical loop with no framed
        // function beside it) stays `match`.
        if self.ptr_walk_chain_loop.is_some() {
            return None;
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
        SyView { locals: &[], ptr_locals: &[], formals: Formals::AllOneRegisterByConstruction };

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
