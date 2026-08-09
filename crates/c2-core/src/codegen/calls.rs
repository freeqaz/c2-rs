//! The frame/call complex: framed bodies, call sequences, tail calls, and
//! argument permutation. **One module on purpose.**
//!
//! This is the serial spine's home (Class B — values live across calls —
//! Class C helpers, multi-call accumulators, and the EH records after them).
//! Its parts are genuinely coupled to each other and to `coff.rs`'s symbol and
//! label order; pretending they are independent would invite two agents to
//! guess allocator order concurrently, which is the one thing the doctrine
//! forbids (`docs/ARCHITECTURE_SEAMS.md` §7, §9.8). One agent owns this file.

use c2_il::{IlFunction, IlOp};
use crate::BackendError;
use crate::codegen::cond_tail::branch_sense;
use crate::codegen::encode::{
    cr_bi, encode_addi, encode_addis, encode_blr, encode_cmplwi,
    encode_cmpwi, encode_mr, BO_FALSE, BO_TRUE, CR_COMPARE,
};
use crate::codegen::frame::FrameLayout;
use crate::codegen::labels::{Form, LabelMap};
use c2_il::LINK_FIRST_SLOT;
use crate::codegen::select::{ARG_REGS, OptMode, RET_REG, SCRATCH_REG, out_of_class};
use crate::codegen::straightline::select_text;

/// Encode an unconditional relative branch `b` (primary opcode 18, AA=0, LK=0)
/// used for a **tail call**, to be paired with a REL24 relocation.
///
/// MSVC's PPC convention stores the displacement field as `−(the instruction's
/// own byte offset within .text)`, so the pre-link value references `.text`
/// offset 0; the linker then patches the 24-bit field from the target symbol.
/// Verified: a tail-call `b` at offset 0 → `0x48000000`; at offset 8 →
/// `0x4BFFFFF8` (displacement −8).
pub fn encode_tail_branch(text_offset: u32) -> [u8; 4] {
    let disp = -(text_offset as i32);
    let word: u32 = 0x4800_0000 | (disp as u32 & 0x03FF_FFFC);
    word.to_be_bytes()
}

/// **W-R1c — the `??__E` dynamic-initializer thunk's `.text$yc` body**, built
/// from the decoded slots rather than transcribed.
///
/// The reference payload, byte-identical across `fixtures/cpp/il_dyninit_static.cpp`,
/// `TomCryptLicense.cpp` and `ZlibLicense.cpp` (`docs/OBJ_DYNINIT_SHAPE.md`
/// §3.3, §7.2):
///
/// ```text
///   3d 60 00 00   lis   r11, 0      <- REFHI(string)   slot 1
///   3d 40 00 00   lis   r10, 0      <- REFHI(object)   slot 0
///   38 8b 00 00   addi  r4, r11, 0  <- REFLO(string)   slot 1 -> r4
///   38 6a 00 00   addi  r3, r10, 0  <- REFLO(object)   slot 0 -> r3
///   38 a0 00 00   li    r5, k                          slot 2 -> r5
///   4b ff ff ec   b     -0x14       <- REL24(ctor)
/// ```
///
/// **The schedule is three blocks, not one descending walk**, and that is the
/// part worth stating because it is *not* what `permute_args_parts` does: every
/// `lis` first (scratch registers descending from r11, symbols in **reverse
/// slot** order), then every address `addi` (destinations descending), then
/// every `li`. `docs/IL_CALL_IN_EXPR.md` §17.3's `g(&gA, 7, "cc")` is the
/// witness that the `li` really does come last — it emits
/// `lis r11,cc · lis r10,gA · addi r5,r11 · addi r3,r10 · li r4,7 · b`, where a
/// merged descending walk would have put `li r4,7` between the two `addi`s.
///
/// Fenced to exactly the two-symbol/one-literal shape this lane measured. The
/// general two-symbol tail call stays declined (w-r1 rung, "found and not
/// taken" item 2) precisely because that emission order is not the one this port
/// ships elsewhere.
///
/// Returns the text plus the offsets the caller relocates: `(text, hi/lo per
/// slot, branch offset)`.
pub struct DynInitBody {
    pub text: Vec<u8>,
    /// REFHI/REFLO site pair for the **object** (slot 0).
    pub object_hi: u32,
    pub object_lo: u32,
    /// REFHI/REFLO site pair for the **literal** (slot 1).
    pub literal_hi: u32,
    pub literal_lo: u32,
    /// REL24 site of the tail branch to the constructor.
    pub branch: u32,
}

/// Build [`DynInitBody`] for `??__E<obj>` calling `ctor(&obj, literal, k)`.
///
/// `None` when `k` does not fit the `li` immediate — the only input that can
/// vary in the measured class, and a value that does not fit is a different
/// instruction, not a wider one.
pub fn dyninit_thunk_text(k: i32) -> Option<DynInitBody> {
    /// `li`'s signed 16-bit immediate, the same bound `c2-il`'s slot parser
    /// applies before it will call a literal a slot at all.
    const LI_IMM: std::ops::RangeInclusive<i32> = -0x8000..=0x7FFF;
    if !LI_IMM.contains(&k) {
        return None;
    }
    /// `lis rD, 0` — `addis rD, r0, 0`, primary opcode 15.
    fn lis(d: u32) -> [u8; 4] {
        (0x3C00_0000u32 | (d << 21)).to_be_bytes()
    }
    /// `addi rD, rA, 0` — primary opcode 14, the low half of an address.
    fn addi(d: u32, a: u32) -> [u8; 4] {
        (0x3800_0000u32 | (d << 21) | (a << 16)).to_be_bytes()
    }
    /// `li rD, k` — `addi rD, r0, k`.
    fn li(d: u32, k: i32) -> [u8; 4] {
        (0x3800_0000u32 | (d << 21) | (k as u32 & 0xFFFF)).to_be_bytes()
    }
    let mut text: Vec<u8> = Vec::with_capacity(0x18);
    // The `lis` block: scratch registers descending from r11, taken in REVERSE
    // slot order, so the literal (slot 1) gets r11 and the object (slot 0) r10.
    let literal_hi = text.len() as u32;
    text.extend_from_slice(&lis(11));
    let object_hi = text.len() as u32;
    text.extend_from_slice(&lis(10));
    // The address `addi` block: destinations descending (r4 then r3), each
    // reading the scratch its own `lis` wrote.
    let literal_lo = text.len() as u32;
    text.extend_from_slice(&addi(4, 11));
    let object_lo = text.len() as u32;
    text.extend_from_slice(&addi(3, 10));
    // The `li` block, last.
    text.extend_from_slice(&li(5, k));
    let branch = text.len() as u32;
    text.extend_from_slice(&encode_tail_branch(branch));
    Some(DynInitBody {
        text,
        object_hi,
        object_lo,
        literal_hi,
        literal_lo,
        branch,
    })
}

/// Encode a **linking** relative branch `bl` (primary opcode 18, AA=0, **LK=1**)
/// used for a non-leaf `bl <callee>` inside a framed function, paired with a
/// REL24 relocation. Same MSVC displacement convention as [`encode_tail_branch`]
/// (`disp = −(own .text offset)`) plus the link bit. Verified: `bl` at offset
/// 0xC → `0x4BFFFFF5` (disp −0xC, LK=1).
pub fn encode_call_branch(text_offset: u32) -> [u8; 4] {
    let disp = -(text_offset as i32);
    let word: u32 = 0x4800_0000 | (disp as u32 & 0x03FF_FFFC) | 1;
    word.to_be_bytes()
}

/// A framed non-leaf call's emitted body: the bytes, and the `.text` offsets the
/// caller needs — the REL24 site of the `bl` and the prologue length that
/// becomes the `$M(n)` label and the `.pdata` `PrologLen`.
pub struct FramedBody {
    pub text: Vec<u8>,
    /// Absolute `.text` offset of the `bl <callee>` (already includes
    /// `base_off`): the REL24 relocation site.
    pub bl_offset: u32,
    /// Prologue length in bytes, relative to the function start.
    pub prolog_len: u32,
}

/// Emit the `.text` for a **framed non-leaf call** `return g(<formal>) + k`
/// (W4b2).
///
/// ```text
/// 7d8802a6  mflr r12                prologue: save LR
/// 9181fff8  stw  r12,-8(r1)
/// 9421ffa0  stwu r1,-96(r1)         allocate the 96-byte frame
/// [7c832378  or  r3,rN,rN]          argument setup — ONLY when the argument is
///                                   not already the formal in r3
/// 4bfffff5  bl   <callee>           REL24 reloc site
/// 3863kkkk  addi r3,r3,k            the post-call op (+k)
/// 38210060  addi r1,r1,96           epilogue: free frame
/// 8181fff8  lwz  r12,-8(r1)         restore LR
/// 7d8803a6  mtlr r12
/// 4e800020  blr
/// ```
///
/// **The argument-setup word was missing and that was a live wrong-bytes emit.**
/// This function used to emit one byte-constant 0x24-byte body, on the tacit
/// assumption that the call's argument is always the formal already in r3. The
/// parser only ever required the argument to be *a* formal, so
/// `int f(int a,int b){ return g(b) + 1; }` emitted 9 words where c2 emits 10 —
/// `or r3,r4,r4` at `.text+0xC` — with the `.pdata` `FuncLen` and both `$M`
/// labels wrong to match. 37 of 47 probes around the accepted class mismatched,
/// including every member function (`this` occupies r3, so a one-parameter
/// member's argument is in r4) and every free function with a leading `float`,
/// `double`, `long long`, pointer or 8-byte aggregate parameter. The sweep never
/// separated it because every generated framed case is `int F(int a){ return
/// g(a) + 1; }` — one parameter, in r3. `docs/GAPS.md` §6: a corpus holding only
/// the safe half of a pair cannot see the dangerous half.
///
/// The setup is computed by the caller through [`select_text`], the *same*
/// locator the integer tail call uses for the same job, so the formal → argument
/// register mapping has one implementation and not two.
///
/// `k` must fit the signed-16-bit `addi` immediate (the IL parser guarantees
/// this before constructing the [`c2_il::FramedCall`]).
///
/// `base_off` is the function's start within the `.text` section it lands in —
/// 0 under `/Gy` (its own COMDAT) and its packed offset otherwise. It exists
/// because the `bl` displacement follows MSVC's `disp = −(own .text offset)`
/// convention, so a framed function that is **not first** in a packed `.text`
/// needs a different branch word: `?f` at 0x08 with the `bl` at 0x14 gets
/// `4BFFFFED`, not the `4BFFFFF5` this function emitted unconditionally. That
/// was unreachable while a framed TU was gated to one function and became a
/// live wrong-bytes emit the moment the gate came off.
pub fn framed_call_text(
    setup: &[u8],
    add_k: i32,
    base_off: u32,
    frame: FrameLayout,
) -> Result<FramedBody, BackendError> {
    let k = add_k as i16; // range-checked upstream (c2_il::func::parse_segment)
    let prologue = frame.prologue()?;
    let epilogue = frame.epilogue()?;
    let prolog_len = prologue.len() as u32;
    let bl_offset = base_off + prolog_len + setup.len() as u32;
    let mut text = Vec::with_capacity(prologue.len() + setup.len() + 8 + epilogue.len());
    text.extend_from_slice(&prologue);
    text.extend_from_slice(setup);
    // Call (LK=1); the REL24 reloc at `bl_offset` patches the target.
    text.extend_from_slice(&encode_call_branch(bl_offset));
    // Post-call op.
    text.extend_from_slice(&encode_addi(RET_REG, RET_REG, k)); // addi r3,r3,k
    text.extend_from_slice(&epilogue);
    Ok(FramedBody { text, bl_offset, prolog_len })
}

/// A **Class A many-call body**'s emitted `.text`, with one REL24 site per call.
pub struct SeqBody {
    pub text: Vec<u8>,
    /// Absolute `.text` offsets of the `bl <callee>` words (already including
    /// `base_off`), in call order — one relocation site each.
    pub bl_offsets: Vec<u32>,
    pub prolog_len: u32,
}

/// Emit the `.text` for a **Class A many-call body** (`docs/GAPS.md` #35 step 2,
/// rung 1): a framed function whose body is a sequence of calls with nothing live
/// across any of them.
///
/// ```text
/// 7d8802a6  mflr r12                prologue — the shipped Class A three words
/// 9181fff8  stw  r12,-8(r1)
/// 9421ffa0  stwu r1,-96(r1)
/// <setup 0>                         per call: the argument marshalling …
/// 4bfffff5  bl   <callee 0>         … then the LINKING branch. REL24 site.
/// <setup 1>
/// 4bfffff1  bl   <callee 1>
/// <tail>                            nothing, `addi r3,r3,k`, or `li r3,k`
/// 38210060  addi r1,r1,96           epilogue
/// 8181fff8  lwz  r12,-8(r1)
/// 7d8803a6  mtlr r12
/// 4e800020  blr
/// ```
///
/// Byte evidence, one probe per row, all at `/O1 /GS- /c` (and the `.text` is
/// identical at `/Ox` and `/O2` — `docs/CODEGEN_FRAMED_CALLS.md`'s mode note):
///
/// ```text
///   void f(){ g1(); g2(); }             36 B  bl bl
///   void f(){ g1(); g2(); g3(); g4(); } 44 B  bl bl bl bl
///   void f(int a){ g1(a); g2(); }       36 B  a is already in r3
///   void f(){ g1(1); g2(2); }           44 B  li r3,1 · bl · li r3,2 · bl
///   void f(int a,int b){ g2(a,b); h(); }36 B  identity permutation, no moves
///   void f(int a,int b){ g2(b,a); h(); }48 B  mr r11,r4 · mr r4,r3 · mr r3,r11
///   void f(int a){ g1(a+1); g2(); }     40 B  addi r3,r3,1
///   int  f(int a){ g1(a); return 5; }   36 B  li r3,5 after the last bl
///   int  f(){ g1(); return g2(); }      36 B  no post-op at all
///   int  f(){ g1(); return g2()+1; }    40 B  addi r3,r3,1
/// ```
///
/// **The last call is a `bl`, never a `b`.** Every row above ends
/// `bl <callee> … addi r1,r1,96 … blr`: c2's tail-call transform is off once the
/// function is framed. The one shape that *is* tail-called is a lone statement
/// call with nothing after it (`void f(int a){ g(a); }` → a bare `b ?g`, five
/// sections, no `.pdata`), and the IL parser routes that to the tail-call
/// production so it can never reach here.
///
/// `setups[i]` is call `i`'s argument marshalling, already computed by the caller
/// through the *same* [`select_text`] / [`permute_args_text`] locators every other
/// call shape uses; `tail` is the post-call word(s), empty for a void body.
/// `base_off` is the function's start within its `.text` section — 0 under `/Gy`,
/// its packed offset otherwise — because the `bl` displacement follows MSVC's
/// `disp = −(own .text offset)` convention.
/// **W10 — the emission plan for a [`c2_il::SeqGuard`]**: the compare word and
/// the branch's `(BO, BI)`, resolved once so [`call_seq_text`] only has to place
/// them.
///
/// Built by [`seq_guard_emit`], which is the *only* place the compare register
/// is chosen — and it chooses the formal's **home** argument register, because
/// this class admits no entry-block move and therefore has no post-hoist
/// location to resolve. (`cond_tail`'s `plan_cond_pair` does have one, and the
/// separating cell is `?mmioGetInfo`: `mr r11,r3 ; cmplwi cr6,r11,0`.
///
/// **This paragraph used to end "a guarded sequence with a park is refused in
/// the IL parser, so the two rules cannot be confused here", and that was
/// FALSE.** Lane `w-clear` gridded it: the IL parser accepts a guarded EARLY
/// RETURN in front of a permuted call and the emitter lowered it with the
/// *unguarded* cycle, producing wrong bytes in **30 of 54** cells. The refusal
/// the sentence asserted now exists — in [`call_seq_parts`], with the grid
/// behind it — but it is a W11 `early` refusal, not a W10 `guard` one, and the
/// two are different fields.)
pub struct SeqGuardEmit {
    /// `cmpwi cr6,rA,k` or `cmplwi cr6,rA,k`, already encoded.
    cmp: [u8; 4],
    /// The conditional branch's `BO` and `BI`, the **negation** of the IL
    /// relation because the IL's `38` is brFALSE.
    bo: u8,
    bi: u8,
}

/// Resolve a [`c2_il::SeqGuard`] into the two words its emission needs.
///
/// The compare instruction comes from the operand's signedness and from nothing
/// else — the relational opcodes are sign-agnostic (`docs/CFG_SHAPE.md` §3.2) —
/// and the branch sense from [`crate::codegen::cond_tail::branch_sense`], the
/// **same** table W9 graded against the real `c2` across all six relations and
/// both signednesses. Sharing it is the point: a second copy would be a second
/// place for the `bt`/`bf` split to be wrong, and that split had no oracle
/// witness at all until W9.
pub fn seq_guard_emit(guard: &c2_il::SeqGuard) -> Result<SeqGuardEmit, BackendError> {
    let ra = *ARG_REGS.get(guard.cmp_param).ok_or_else(|| {
        out_of_class("a guarded sequence comparing a stack-homed formal")
    })?;
    let cmp = if guard.signed {
        let k = i16::try_from(guard.k)
            .map_err(|_| out_of_class("a signed guard literal wider than `cmpwi`'s immediate"))?;
        encode_cmpwi(CR_COMPARE, ra, k)
    } else {
        let k = u16::try_from(guard.k)
            .map_err(|_| out_of_class("an unsigned guard literal wider than `cmplwi`'s immediate"))?;
        encode_cmplwi(CR_COMPARE, ra, k)
    };
    let (bo, bit) = branch_sense(guard.rel);
    Ok(SeqGuardEmit { cmp, bo, bi: cr_bi(CR_COMPARE, bit) })
}

/// **W11 — the emission plan for one [`c2_il::SeqEarlyReturn`]**: the compare
/// word, the branch's `(BO, BI)`, and the literal the arm materializes.
///
/// Built by [`seq_early_emit`], which is the only place the compare register and
/// the branch sense are chosen for this class.
pub struct SeqEarlyEmit {
    /// `cmpwi cr6,rA,k` or `cmplwi cr6,rA,k`, already encoded.
    cmp: [u8; 4],
    bo: u8,
    bi: u8,
    /// **W-SMALL — the short-circuit `&&`'s further `(cmp, bo, bi)` triples**, in
    /// source order after the one above. Empty for a single-test guard.
    ///
    /// Each emits `cmp ; bc` at the **same** label with the **same** sense, so
    /// this is one `extend` per conjunct and no new block. That is the measured
    /// shape and not a simplification: `int P(int a,int b){ if (a != 0 && b != 0)
    /// return 5; v0(); return 0; }` at `/Ox` emits `2f030000 419a0020 2f040000
    /// 419a0018` — the two `bc` words differ only in displacement, both forward,
    /// both naming the arm's skip target, whose predecessor count is therefore 2.
    /// That is exactly the multi-reference case [`LabelMap`] was built for, and
    /// the compiler-label counter charges **+0** for it
    /// (`docs/rungs/2026-08-04-w-label.md` §2.3, `ho-and`/`ho-or` at
    /// `stride 5 / extra 0`), so `coff::plan_labels` is untouched.
    and_conds: Vec<([u8; 4], u8, u8)>,
    /// The returned literal, or `None` for `return;` — which is what decides
    /// both the branch's sense and its target.
    value: Option<i32>,
}

/// Resolve a [`c2_il::SeqEarlyReturn`] into the words its emission needs.
///
/// The compare instruction comes from the operand's signedness alone, and the
/// branch sense from [`crate::codegen::cond_tail::branch_sense`] — the same
/// table W9 graded against the real `c2` across all six relations and both
/// signednesses, shared rather than copied for the reason W10's guard shares it.
///
/// **Then one rule on top of that table, and it is measured, not tidy.**
/// `branch_sense` returns the *negation* of the source relation, because the
/// branch is normally the edge that steps **past** the arm. A **void** arm is
/// empty — there is no block to step past — so c2 deletes it and points the
/// branch straight at the epilogue with the relation **itself**:
///
/// ```text
///   int  f(int a){ if(a!=0) return 5; v0(); return 0; }  ->  bt 26  (negated)
///   void g(int a){ if(a!=0) return;   v0(); v1();      }  ->  bf 26  (NOT negated)
/// ```
///
/// `work/w-conv/p/probe1.cpp::e4`, `probe2.cpp::rv`, `probe3.cpp::w1`/`w2`, at
/// `/O1` and `/Ox` — and the void form is byte-identical between the two modes,
/// which is its own control on the mode split [`call_seq_text`] implements. It
/// is `work/w-cross/PREREG.md` §1's **empty-arm inversion**, in the smallest
/// body that has it.
pub fn seq_early_emit(e: &c2_il::SeqEarlyReturn) -> Result<SeqEarlyEmit, BackendError> {
    seq_early_emit_remapped(e, &SeqPark::default(), 0)
}

/// [`seq_early_emit`], but reading the compare register out of the
/// **entry-block park** rather than out of the formal's home.
///
/// Board #275: once a park has run, the formal a guard tests may no longer be
/// in its home register — `?mmioGetInfo` compares `r11` for `a0` and `r3` for
/// `a1`, and `h3_n4_p2013_g120` compares `r4`, `r3` and `r11` for `a1`, `a2`
/// and `a0` in that order. Measured on every cell of grids 1–3: **the guard
/// reads whatever register currently holds its formal.** Resolving that here,
/// out of the same [`SeqPark`] that produced the moves, is what stops a compare
/// from naming a register the entry block did not write.
pub fn seq_early_emit_remapped(
    e: &c2_il::SeqEarlyReturn,
    park: &SeqPark,
    ix: usize,
) -> Result<SeqEarlyEmit, BackendError> {
    let home = *ARG_REGS.get(e.cmp_param).ok_or_else(|| {
        out_of_class("a guarded early return comparing a stack-homed formal")
    })?;
    let ra = park.reg_of(e.cmp_param, home, ix);
    if !park.entry.is_empty() && !e.and_conds.is_empty() {
        // A `&&` conjunct reads a SECOND formal, and no cell of grids 1–3
        // crossed a conjunct with a park. Refused rather than remapped.
        return Err(out_of_class(
            "a short-circuit `&&` conjunct beside an entry-block park: the \
             conjunct's own scrutinee is a second formal and the composition is \
             ungraded",
        ));
    }
    let cmp = if e.signed {
        let k = i16::try_from(e.k).map_err(|_| {
            out_of_class("a signed early-return guard literal wider than `cmpwi`'s immediate")
        })?;
        encode_cmpwi(CR_COMPARE, ra, k)
    } else {
        let k = u16::try_from(e.k).map_err(|_| {
            out_of_class("an unsigned early-return guard literal wider than `cmplwi`'s immediate")
        })?;
        encode_cmplwi(CR_COMPARE, ra, k)
    };
    let (bo, bit) = branch_sense(e.rel);
    // The empty-arm inversion: flip the BO and keep the CR bit, which is exactly
    // "use the relation rather than its negation" over `branch_sense`'s table.
    let bo = if e.value.is_some() {
        bo
    } else if bo == BO_TRUE {
        BO_FALSE
    } else {
        BO_TRUE
    };
    // **W-SMALL — the `&&` conjuncts, resolved through the SAME rules.** Sharing
    // the compare encoding and `branch_sense` rather than restating them is what
    // keeps a conjunct from acquiring a different signedness or sense rule than
    // the conjunct in the fields above; the empty-arm inversion is applied to
    // every one of them, because the arm they all skip is the same arm.
    let mut and_conds = Vec::with_capacity(e.and_conds.len());
    for &(cmp_param, rel, signed, k) in &e.and_conds {
        let ra = *ARG_REGS.get(cmp_param).ok_or_else(|| {
            out_of_class("a short-circuit `&&` conjunct comparing a stack-homed formal")
        })?;
        let c = if signed {
            let k = i16::try_from(k).map_err(|_| {
                out_of_class("a signed `&&` conjunct literal wider than `cmpwi`'s immediate")
            })?;
            encode_cmpwi(CR_COMPARE, ra, k)
        } else {
            let k = u16::try_from(k).map_err(|_| {
                out_of_class("an unsigned `&&` conjunct literal wider than `cmplwi`'s immediate")
            })?;
            encode_cmplwi(CR_COMPARE, ra, k)
        };
        let (cbo, cbit) = branch_sense(rel);
        let cbo = if e.value.is_some() {
            cbo
        } else if cbo == BO_TRUE {
            BO_FALSE
        } else {
            BO_TRUE
        };
        and_conds.push((c, cbo, cr_bi(CR_COMPARE, cbit)));
    }
    Ok(SeqEarlyEmit { cmp, bo, bi: cr_bi(CR_COMPARE, bit), and_conds, value: e.value })
}

/// **Board #275 — the ENTRY-BLOCK PARK**, as the split between what a guarded
/// permuted call emits *before* its guards and what it leaves at the call.
///
/// Built by [`seq_entry_park`], which is the only place the anchor is chosen.
/// [`SeqEarlyEmit`] resolves its compare register through [`SeqPark::reg_of`],
/// so the guards and the moves cannot disagree about where a formal lives.
#[derive(Debug, Default, Clone)]
pub struct SeqPark {
    /// The words between the prologue and the first early return: `mr r11,rA`
    /// and then the ascending prefix of the chain. Empty when there is no park.
    pub entry: Vec<u8>,
    /// **Which register the FIRST guard compares**, for every formal whose home
    /// the entry block overwrote. Parameter index → register.
    pub first: Vec<(usize, u8)>,
    /// **Which register every LATER guard compares** — the last register the
    /// entry block wrote the value into — for every formal that moved at all.
    pub later: Vec<(usize, u8)>,
}

impl SeqPark {
    /// The register holding formal `pi` at guard number `ix`.
    ///
    /// **The two maps are different, and that is measured, not defensive.**
    /// Over all 1,654 guards of grids 1–3:
    ///
    /// ```text
    ///   guard 0     its formal's HOME, unless the entry block overwrote the
    ///               home — then wherever the value went
    ///   guard 1..n  the LAST register the entry block wrote the value into,
    ///               even when the home still holds a live copy
    /// ```
    ///
    /// The separating pair is `gtgt_n4_p0312_g3` against `g2ord_n4_p0312_g23`:
    /// identical entry blocks (`mr r11,r6` and nothing else) and the same
    /// formal tested, and c2 compares **r6** in the first and **r11** in the
    /// second — the only difference being that in the second it is not the
    /// first guard. Both are correct code; the choice is not forced, and a
    /// single map gets one of them wrong. Scored: **0 of 1,654** wrong.
    pub fn reg_of(&self, pi: usize, home: u8, ix: usize) -> u8 {
        let m = if ix == 0 { &self.first } else { &self.later };
        m.iter()
            .find(|(p, _)| *p == pi)
            .map(|(_, r)| *r)
            .unwrap_or(home)
    }
}

/// **The park's rule, measured over 886 cells against the real `c2.dll`** —
/// grids 1–3 in `work/w-mmio/probe{,2,3}/`, at the dc3 workload's own flags and
/// cwd, and re-checked byte-for-byte at `/Ox` and `/O2` on 30 of them.
///
/// Board **#1414** publishes this rule as *"break the cycle by saving the
/// LOWEST slot's home into r11, then hoist the maximal prefix whose destination
/// register is strictly increasing"*. **The first half is wrong**, and lane
/// `w-clear`'s five cells could not see it because in all five the guard's
/// formal and the cycle minimum were the same register `r3`. Measured over a
/// population that separates them, `R-MIN` scores **394 of 832**.
///
/// The rule that holds — and the reason it holds:
///
/// 1. **The call site emits DESCENDING by destination** ([`moves_descending`],
///    the rule this emitter already implements for the unguarded case), and the
///    **entry block emits ASCENDING**. A chain can therefore be laid out at all
///    only when its destination sequence is **unimodal**; the split falls at
///    the peak.
/// 2. **The anchor is the guard's own scrutinee** when the chain rooted there
///    is unimodal. That is the case this function admits, and it is 496 of 496
///    over the three grids — fitted on grid 1, confirmed unchanged on grids 2
///    and 3, which were generated and committed before they were compiled.
/// 3. When the first guard cannot anchor, c2 scans on to later guards and, past
///    that, falls back to the cycle minimum. That clause was refuted twice —
///    once by grid 2 and once by grid 3 — and each replacement was fitted to
///    the grid that refuted it. **It is deliberately NOT implemented**, and the
///    parser refuses its population by name. Board #260's warning is the
///    reason: a clause that has been re-fitted at every new population is a
///    clause whose next population will re-fit it again.
///
/// ```text
///   f(p,q){ if(!p) return 5; g(q,p); }
///     ENTRY   mr r11,r3 · mr r3,r4      the park, ASCENDING
///             cmplwi cr6,r11,0 · bf …   the guard reads the PARKED register
///     CALL    mr r4,r11                 the remainder, DESCENDING
/// ```
///
/// Returns `(entry, call, remap)`. `sources[d]` is the formal in slot `d`;
/// `scrutinee` is the first early return's `cmp_param`.
pub fn seq_entry_park(
    sources: &[usize],
    scrutinee: usize,
) -> Result<(Vec<u8>, Vec<u8>, Vec<(usize, u8)>, Vec<(usize, u8)>), BackendError> {
    let n = sources.len();
    if n > ARG_REGS.len() {
        return Err(out_of_class("a parked permutation past the register slots"));
    }
    // The single non-trivial cycle. Multi-cycle and length > 3 are refused
    // upstream by `c2_il` (`call-arg-multicycle`, `call-arg-long-cycle`) and by
    // `permute_args_parts`; this walk is the backstop that says so by name.
    let mut seen = vec![false; n];
    let mut cycles: Vec<Vec<usize>> = Vec::new();
    for start in 0..n {
        if seen[start] || sources[start] == start {
            seen[start] = true;
            continue;
        }
        let (mut cycle, mut at) = (Vec::new(), start);
        while !seen[at] {
            seen[at] = true;
            cycle.push(at);
            if sources[at] >= n {
                return Err(out_of_class("a parked permutation sources outside its slots"));
            }
            at = sources[at];
        }
        cycles.push(cycle);
    }
    if cycles.len() != 1 {
        return Err(out_of_class(
            "a parked permutation with other than one non-trivial cycle",
        ));
    }
    let cycle = &cycles[0];
    if cycle.len() > 3 {
        return Err(out_of_class(
            "a parked permutation with a cycle longer than three: past three c2 \
             hoists a second save into r10 and the order it picks is not \
             characterized",
        ));
    }
    // **Clause 2, and ONLY clause 2.** The anchor is the guard's scrutinee or
    // this shape is out of class — see the doc above for why clause 3 is not
    // here.
    if !cycle.contains(&scrutinee) {
        return Err(out_of_class(
            "a guarded permuted call whose scrutinee is not in the permutation's \
             cycle: c2 then anchors at a later guard or at the cycle minimum, and \
             that clause has been re-fitted at every population that measured it",
        ));
    }
    let chain = park_chain(sources, scrutinee);
    let dests: Vec<u8> = chain.iter().map(|&(d, _)| d).collect();
    let mut peak = 0usize;
    while peak + 1 < dests.len() && dests[peak] < dests[peak + 1] {
        peak += 1;
    }
    let mut down = peak;
    while down + 1 < dests.len() && dests[down] > dests[down + 1] {
        down += 1;
    }
    if down != dests.len() - 1 {
        return Err(out_of_class(
            "a guarded permuted call whose chain is not unimodal: the entry block \
             ascends and the call site descends, so this chain has no layout and \
             c2 anchors elsewhere",
        ));
    }
    // `peak` is the index of the highest destination; everything strictly before
    // it is hoisted, and it and everything after stay at the call.
    let anchor_reg = ARG_REGS[scrutinee];
    let mut entry = encode_mr(SCRATCH_REG, anchor_reg).to_vec();
    for &(d, s) in &chain[..peak] {
        entry.extend_from_slice(&encode_mr(d, s));
    }
    let mut call = Vec::new();
    for &(d, s) in &chain[peak..] {
        call.extend_from_slice(&encode_mr(d, s));
    }
    // Where every formal lives at the guards, by simulating the entry block —
    // and the two maps `SeqPark::reg_of` documents, which differ only at guard
    // zero and differ there in 178 measured cells.
    let mut at: Vec<u8> = (0..n).map(|i| ARG_REGS[i]).collect();
    let mut clobbered: Vec<u8> = vec![SCRATCH_REG];
    at[scrutinee] = SCRATCH_REG;
    for &(d, s) in &chain[..peak] {
        if let Some(pi) = (0..n).find(|&i| at[i] == s) {
            at[pi] = d;
        }
        clobbered.push(d);
    }
    let later: Vec<(usize, u8)> = (0..n)
        .filter(|&i| at[i] != ARG_REGS[i])
        .map(|i| (i, at[i]))
        .collect();
    let first: Vec<(usize, u8)> = later
        .iter()
        .copied()
        .filter(|&(i, _)| clobbered.contains(&ARG_REGS[i]))
        .collect();
    Ok((entry, call, first, later))
}

/// The move chain rooted at `anchor`: `anchor <- s(anchor)`, then that slot's
/// own source, and finally the read-back from [`SCRATCH_REG`]. Its order is
/// forced by the dependencies and is shared by the entry block and the call.
fn park_chain(sources: &[usize], anchor: usize) -> Vec<(u8, u8)> {
    let mut moves = Vec::new();
    let mut cur = anchor;
    loop {
        let nxt = sources[cur];
        if nxt == anchor {
            moves.push((ARG_REGS[cur], SCRATCH_REG));
            return moves;
        }
        moves.push((ARG_REGS[cur], ARG_REGS[nxt]));
        cur = nxt;
    }
}

/// **W-MEMCPY — merge a literal argument into the park's CALL-SITE remainder.**
///
/// [`seq_entry_park`] returns the entry block and the call-site moves and is
/// deliberately left byte-for-byte alone: it landed with 496 of 496 cells
/// behind it and this lane re-checks it rather than re-opening it. What is new
/// is that the call-site run may now also carry a `li`.
///
/// **The rule, measured over GRID-L (`work/w-memcpy/probeL`, 747 cells through
/// the real `c2.dll` at the dc3 workload's own flags):**
///
/// ```text
///   R-DESC   the literal takes its place by DESCENDING DESTINATION, the same
///            walk `moves_descending` and `lit_slots_text` already use
///
///     guarded, <= 1 call-site move, <= 1 literal   416 / 416   <- this class
///     every graded cell                            379 / 403
/// ```
///
/// and **the literal is never hoisted into the entry block — 0 of 579** guarded
/// cells. The park saves a register's *value*; a literal has none to save.
///
/// `?mmioGetInfo`'s own three words are the smallest instance:
///
/// ```text
///   ENTRY   mr r11,r3 · mr r3,r4
///   CALL    li r5,72  · mr r4,r11        72 -> r5, and r5 > r4
/// ```
///
/// **The fence is the parser's** (`callseq-multiarg-lit-*`); this is the
/// backstop, board #139, and it refuses rather than guessing so that a body the
/// parser lets through by mistake comes out as a `codegen-gap` and not as
/// bytes. Outside the fence the rule that fits is one fitted to the cells that
/// refuted its predecessor — board #260, and `w-mmio` §3 declined its own third
/// fit on the same ground.
fn park_call_with_literals(
    call: &[u8],
    slots: &[c2_il::SlotArg],
) -> Result<Vec<u8>, BackendError> {
    let lits: Vec<(u8, i32)> = slots
        .iter()
        .enumerate()
        .filter_map(|(slot, a)| match a {
            c2_il::SlotArg::Lit(k) => Some((slot, *k)),
            _ => None,
        })
        .map(|(slot, k)| {
            ARG_REGS
                .get(slot)
                .copied()
                .map(|r| (r, k))
                .ok_or_else(|| out_of_class("a literal in a stack-homed argument slot"))
        })
        .collect::<Result<_, _>>()?;
    if lits.is_empty() {
        return Ok(call.to_vec());
    }
    if lits.len() > 1 {
        return Err(out_of_class(
            "two literals beside a park: (g1, 1 move, 2 literals) is 29 of 32 for \
             the descending rule and the rule that fits the other 3 was fitted to \
             them; out of class",
        ));
    }
    // The park's call-site remainder, as destination registers. Every word
    // `seq_entry_park` puts here is an `mr rD,rS`, so `D` is bits 16..21 —
    // decoded rather than re-derived, so there is one walk of the chain and not
    // two.
    if call.len() % 4 != 0 {
        return Err(out_of_class("a park remainder that is not a whole number of words"));
    }
    let dests: Vec<u8> = call
        .chunks_exact(4)
        .map(|w| ((u32::from_be_bytes([w[0], w[1], w[2], w[3]]) >> 16) & 31) as u8)
        .collect();
    if dests.len() > 1 {
        return Err(out_of_class(
            "a literal beside a park that leaves two or more moves at the call: \
             (g1, 2 moves, 1 literal) is 72 of 76 and (g1, 3 moves, 1 literal) is \
             4 of 4, a non-monotone pair, so the boundary is the largest \
             UNANIMOUS cell of the grid's axes; out of class",
        ));
    }
    let (lit_reg, k) = lits[0];
    if dests.iter().any(|&d| d == lit_reg) {
        return Err(out_of_class(
            "a literal slot the park also writes: the slot list claims one value \
             and the permutation another",
        ));
    }
    let li = encode_addi(lit_reg, 0, k as i16);
    let mut out = Vec::with_capacity(call.len() + 4);
    // DESCENDING destination, and with one move and one literal that is the
    // whole of the merge.
    if dests.first().is_some_and(|&d| d > lit_reg) {
        out.extend_from_slice(call);
        out.extend_from_slice(&li);
    } else {
        out.extend_from_slice(&li);
        out.extend_from_slice(call);
    }
    Ok(out)
}

pub fn call_seq_text(
    setups: &[Vec<u8>],
    tail: &[u8],
    base_off: u32,
    frame: FrameLayout,
    park: &[u8],
    guard: Option<&SeqGuardEmit>,
    early: &[SeqEarlyEmit],
    mode: OptMode,
) -> Result<SeqBody, BackendError> {
    if setups.is_empty() {
        return Err(out_of_class("a call sequence with no calls"));
    }
    if guard.is_some() {
        // The guarded call is `setups[0]` and the join's first call is
        // `setups[1]`. The IL parser guarantees the second, and this is the
        // backstop — an out-of-range index below would be a silent
        // wrong-displacement emit rather than a panic.
        if setups.len() < 2 {
            return Err(out_of_class(
                "a guarded call sequence with no unguarded call after the guard:                  that shape is fold band 2 (`bclr`) plus a tail call, not a frame",
            ));
        }
    }
    if !early.is_empty() && guard.is_some() {
        // Two block plans in one body. c2 composes them
        // (`work/w-conv/p/probe3.cpp::x6`) and so could this emitter, but the IL
        // parser refuses the combination and this is the backstop: an emitter
        // that silently interleaved them would be laying out blocks nothing has
        // graded.
        return Err(out_of_class(
            "a guarded call and a guarded early return in one body: two block \
             plans, one production each, and the combination is ungraded",
        ));
    }
    if !park.is_empty() && (guard.is_some() || early.is_empty()) {
        // The park is the ENTRY BLOCK of a guarded-early-return body and has no
        // meaning without one. A W10 `guard` beside it is refused two lines
        // below anyway; saying so here as well keeps the park from being laid
        // out into a block plan nothing has graded.
        return Err(out_of_class(
            "an entry-block park without a guarded early return in front of it",
        ));
    }
    let prologue = frame.prologue()?;
    let epilogue = frame.epilogue()?;
    let prolog_len = prologue.len() as u32;
    let mut text = prologue;
    // ---- board #275: the ENTRY-BLOCK PARK, ahead of the early returns -------
    //
    // `mr r11,rA` and the ascending prefix of the permutation's chain, emitted
    // between the prologue and the first compare. The guards then read the
    // register each formal has *landed in*, which `seq_early_emit_remapped`
    // resolves through the same [`SeqPark`] this text came from — one place, so
    // a compare cannot name a register the moves above it did not write.
    text.extend_from_slice(park);
    // ---- W11: the guarded early returns, ahead of everything ----------------
    //
    // Each is `cmp ; bc ; <arm>`, and the arm is where the two optimization
    // modes part company:
    //
    //   /O1        li r3,K ; b -> EPILOGUE      the epilogue is SHARED
    //   /Ox, /O2   li r3,K ; <the whole epilogue>   it is DUPLICATED
    //
    // That is board row X-b's mode split, and it **refutes** `docs/OPT_MODE.md`
    // and this crate's own `OptMode` doc as they stood: the modes differ in
    // block structure, not only in a register field. It is not W10's declined
    // cost model, though — the duplicated block here is the epilogue, whose
    // length is a constant of the frame class, and `/Ox` copies it in every
    // measured cell (guard counts 1–3, six relations, both signednesses,
    // trailing-call counts 1–4, scrutinee at formals 0–3). Two layouts, one per
    // mode, ≥ 8 witnesses each.
    //
    // A **void** arm is empty in both modes, so it emits no `li`, no `b` and no
    // duplicate — and its branch goes straight to the epilogue with the sense
    // `seq_early_emit` already inverted.
    // **Every branch below goes through the label map** ([`LabelMap`]), which is
    // `docs/CFG_SHAPE.md` §6.2 item B. W11 resolved these against one implicit
    // target — the epilogue's identity was carried by the *shape of a tuple* —
    // and the four lines it replaced said of themselves "there is no fixup list
    // and no label map". Every byte this emits is the byte W11 emitted; what the
    // map adds is that a second target can now exist, that a **backward**
    // reference is refused by name (`labels.rs`: ≥ +1 on the compiler-label
    // counter in 11 of 11 measured cells, against `plan_labels`'s 0), and that
    // the two same-opcode encodings of board #191 cannot be confused because
    // `Form` has no variant for the relocated one.
    let mut labels = LabelMap::new();
    let epi = labels.mint("epilogue");
    for e in early {
        text.extend_from_slice(&e.cmp);
        match e.value {
            Some(k) => {
                // A value arm's branch steps PAST the arm, to whatever comes
                // next — the following guard's compare, or the sequence.
                let after_arm = labels.mint("after-early-arm");
                labels.reference(&mut text, after_arm, Form::Bc { bo: e.bo, bi: e.bi });
                // **W-SMALL — `&&`: one more `cmp ; bc` per conjunct, at the SAME
                // label.** This is the ≥ 2-reference case `LabelMap` exists for
                // and the first shape in the port that actually uses it; every
                // reference is forward, so `labels.rs` invariant 4 holds.
                for (c, cbo, cbi) in &e.and_conds {
                    text.extend_from_slice(c);
                    labels.reference(&mut text, after_arm, Form::Bc { bo: *cbo, bi: *cbi });
                }
                let k = i16::try_from(k).map_err(|_| {
                    out_of_class("an early return's literal is wider than `li`")
                })?;
                text.extend_from_slice(&encode_addi(RET_REG, 0, k));
                match mode {
                    OptMode::O1 => labels.reference(&mut text, epi, Form::B),
                    OptMode::Ox => text.extend_from_slice(&epilogue),
                }
                labels.define(after_arm, &text)?;
            }
            None => {
                // A **void** arm is empty in both modes — c2 deletes the block
                // and points the guard's own `bc` straight at the epilogue, with
                // the relation itself where the value form emits its negation
                // (`seq_early_emit` has already done the inversion). There is no
                // arm to step past, so there is no second label.
                //
                // **W-SMALL — and a void arm with `&&` conjuncts is NOT this
                // shape.** The IL parser refuses it
                // (`c2_il::…::try_parse_early_return_seq`, with the disassembly);
                // this is the backstop, because emitting one `bc` per conjunct at
                // the epilogue computes `||` where the source says `&&` and would
                // be a wrong-bytes obj that still links. It was exactly that for
                // 12 cells of this lane's grid before the oracle caught it.
                if !e.and_conds.is_empty() {
                    return Err(out_of_class(
                        "a VOID early-return arm guarded by a short-circuit `&&`: \
                         c2 sends every conjunct but the last to the SEQUENCE with \
                         the negated sense and only the last to the epilogue, which \
                         mints a third label whose counter cost is unmeasured",
                    ));
                }
                labels.reference(&mut text, epi, Form::Bc { bo: e.bo, bi: e.bi });
            }
        }
    }
    // **The guard sits between the prologue and the sequence** — measured, with
    // the guarded call's own setup staying INSIDE the guarded block:
    // `work/w-cross/p/probe2.cpp::s1` (`if(a!=0) a1(b); v1();`) emits
    // `cmpwi cr6,r3,0 ; bt 26,+12 ; mr r3,r4 ; bl ?a1 ; bl ?v1`. An emitter that
    // hoisted the `mr` above the branch would be four bytes right and one
    // instruction wrong.
    let join = guard.map(|g| {
        text.extend_from_slice(&g.cmp);
        let l = labels.mint("guarded-call-join");
        labels.reference(&mut text, l, Form::Bc { bo: g.bo, bi: g.bi });
        l
    });
    let mut bl_offsets = Vec::with_capacity(setups.len());
    for (i, setup) in setups.iter().enumerate() {
        text.extend_from_slice(setup);
        let off = base_off + text.len() as u32;
        text.extend_from_slice(&encode_call_branch(off));
        bl_offsets.push(off);
        // **The join is the word after the FIRST call.** The guard's branch
        // skips the then-block — which is the guarded call's own setup plus its
        // `bl`, the setup staying inside the block — and lands on the join's
        // first instruction. Binding the label here rather than computing a
        // displacement at the end is the whole difference: the offset is stated
        // where it is known instead of recovered from a length the emitter
        // happens to still be holding.
        if i == 0 {
            if let Some(j) = join {
                labels.define(j, &text)?;
            }
        }
    }
    text.extend_from_slice(tail);
    // ---- the label map, resolved -------------------------------------------
    //
    // The epilogue's offset is only knowable here, after every guard, every
    // arm, every call and the tail. Two branch kinds land on it and they are
    // **the same opcode with different encodings** (board #191): the arms'
    // intra-section `b`, which carries its true displacement and takes no
    // relocation, and — for a void arm — the guard's own `bc`. Neither is the
    // `encode_tail_branch` beside them, which stores −(own offset) and takes a
    // REL24. That is the discriminator `docs/CFG_SHAPE.md` §3.3 warns a single
    // "patch the branch" path corrupts, and it is why `Form` carries the first
    // two and has no variant for the third.
    labels.define(epi, &text)?;
    labels.resolve(&mut text)?;
    text.extend_from_slice(&epilogue);
    Ok(SeqBody { text, bl_offsets, prolog_len })
}

/// The parameter index a single-`Load` argument stream names, for a Class B call
/// that has to read it out of a callee-saved register instead of an argument one.
fn seq_load_param_index(params: &[u32], ops: &[IlOp]) -> Result<usize, BackendError> {
    match ops {
        [IlOp::Load(t)] => params
            .iter()
            .position(|q| q == t)
            .ok_or_else(|| out_of_class("a call argument that is not a formal")),
        _ => Err(out_of_class("expected a single passthrough argument")),
    }
}

/// Lower one call's argument setup, or a `Lit`-only tail, through
/// [`select_text`] — the locator the integer tail call and the single framed call
/// already use — and drop its trailing `blr`.
///
/// A synthetic [`IlFunction`] is the input because `select_text` reads exactly two
/// of its fields (`params` and `ops`). Copying the selection logic instead is how
/// the argument-register move went missing from `framed_call_text`
/// (`docs/ROADMAP.md` §6g item 1): the one-implementation rule is the whole point.
fn ops_setup_text(
    params: &[u32],
    ops: &[IlOp],
    mode: OptMode,
) -> Result<Vec<u8>, BackendError> {
    if ops.is_empty() {
        return Ok(Vec::new());
    }
    let synth = IlFunction {
        // A SYNTHETIC function, never a `.gl` record — so "unasked" is the only
        // honest value, and it is also the one no consumer acts on.
        inlinable: None,
        alloc_init_or_fail: None,
        osf_handle_guard: None,
        guard_ret_chain: None,
        memcpy_tail: None,
        nonce_add_run: None,
        xtea_round_loop: None,
        xtea_encrypt_loop: None,
        fp_store_diamond: None,
        ctor_forward_call: None,
        xlrc_create_guard: None,
        json_utf8_copy: None,
        pool_free_list: None,
        pool_ctor_chain: None,
        mangled_name: String::new(),
        source_path: None,
        params: params.to_vec(),
        ops: ops.to_vec(),
        tail_call: None,
        framed_call: None,
        call_seq: None,
        cond_pair: None,
        compare: None,
        cmp_shift_or: None,
        if_call_join: None,
        ptr_walk_loop: None,
        static_scan_loop: None,
            counted_accum_loop: None,
        guard_chain_shared_tail: None,
        data_def: None,
        ptr_walk_chain_loop: None,
        div_mod_leaf: None,
        float_leaf: None,
        fp_tail: None,
        fp_arg_sources: None,
        arg_sources: None,
        data_syms: Vec::new(),
        fn_addr_sym: None,
        empty_body: false,
        // A synthetic operand-stream carrier, never a function: `select_text`
        // reads `params` and `ops` and nothing else, and the label counter never
        // sees this value.
        eh_bare: false,
        eh_unwind_callees: Vec::new(),
            float_walk_loop: None,
    };
    let mut t = select_text(&synth, mode)?;
    let blr = encode_blr();
    debug_assert!(t.ends_with(&blr), "select_text always terminates in blr");
    t.truncate(t.len() - blr.len());
    Ok(t)
}

/// The callee-saved GPR file, in allocation order: `r31` first, then `r30`.
/// Only two, because at three saved GPRs c2 switches to `__savegprlr_29` and the
/// whole prologue/epilogue/label shape changes ([`FrameLayout::needs_gpr_helper`],
/// `docs/CODEGEN_FRAMED_CALLS.md` §2.3) — the IL parser refuses that class, and
/// this array is the second lock on it.
const SAVED_GPRS: [u8; 2] = [31, 30];

/// Emit a set of **non-conflicting** register-to-register moves, highest
/// destination first.
///
/// This is §3.2's rule for a marshalling with no permutation to break, and it is
/// what a Class B call after the first always needs: its sources are callee-saved
/// registers (`r31`/`r30`) and its destinations are argument registers
/// (`r3`…`r10`), two disjoint sets, so no move can clobber another's source and
/// [`permute_args_text`]'s cycle machinery has nothing to do. Captured:
///
/// ```text
///   void f(int a,int b,int c){ v1(a); g2(c,b); }
///     mr r31,r4 ; mr r30,r5 ; bl ?v1 ; mr r4,r31 ; mr r3,r30 ; bl ?g2
/// ```
///
/// — slot 0 wants `c` (in r30) and slot 1 wants `b` (in r31), and the r4 move is
/// emitted before the r3 one, which is the same descending order the leaf
/// selector and the argument permutation already use.
fn moves_descending(moves: &[(u8, u8)]) -> Vec<u8> {
    let mut m: Vec<(u8, u8)> = moves.iter().copied().filter(|(d, s)| d != s).collect();
    m.sort_by(|a, b| b.0.cmp(&a.0));
    let mut w = Vec::with_capacity(4 * m.len());
    for (dst, src) in m {
        w.extend_from_slice(&encode_mr(dst, src));
    }
    w
}

/// **WCL — a CHAIN LINK's argument setup**: the marshalling for a call whose
/// slot 0 is already filled, because its receiver is the previous call's result
/// and a `bl` has just left it in r3.
///
/// It disagrees with [`moves_descending`] — the rule every other call in this
/// file uses — on the one thing they both have an opinion about, and the
/// disagreement is **measured on both sides of the same probe TU**
/// (`work/WCL/probe/p2.cpp`, `p3.cpp`, `/O1 /GS- /c`):
///
/// ```text
///   a call whose list starts at slot 0 — DESCENDING destination
///     void f(int a,int b,int c){ v1(a); g2(c,b); }   mr r4,r31 ; mr r3,r30
///     void f(int a,int b,int c){ v1(a); g2(b,c); }   mr r4,r30 ; mr r3,r31
///     void f(int a,int b){ v1(a); g3(a,b,5); }       li r5,5 ; mr r4,r30 ; mr r3,r31
///     int  f(O* p,int j){ v1(j); return p->gia(j); } mr r4,r30 ; mr r3,r31
///
///   a chain link, whose list starts at slot 1 — ASCENDING destination
///     int f(O* p,int j,int k){ return p->Next()->gia2(j,k); }
///       … mr r31,r4 ; mr r30,r5 ; bl ?Next ; mr r4,r31 ; mr r5,r30 ; bl ?gia2
///     int f(O* p,int j,int k){ return p->Next()->gia2(k,j); }
///       …                        bl ?Next ; mr r4,r30 ; mr r5,r31
///     int f(O* p,int j,int k){ return p->Next()->gia3(j,5,k); }
///       …                        bl ?Next ; mr r4,r31 ; li r5,5 ; mr r6,r30
/// ```
///
/// The fourth free-function row is the one that matters, and it is why this is
/// not "member calls go the other way": it **is** a member call, its `this` is
/// saved, its slot 0 therefore needs an instruction, and it comes out
/// descending with everything else. The axis is whether the argument list
/// starts at slot 0, not whether there is a receiver.
///
/// Literals interleave in the same order rather than being grouped, in both
/// families — `li r5,5` sits between the two moves in the third row above and
/// between them again in the last. That is the reason this walks the slots once
/// instead of emitting the moves and then the constants.
///
/// No cycle machinery: the sources are the callee-saved file (`r31`/`r30`) and
/// the destinations are argument registers, two disjoint sets, so nothing can
/// clobber anything and a value wanted **twice** is simply written twice
/// (`p->Next()->gia2(j,j)` → `mr r4,r31 ; mr r5,r31`, captured).
fn link_setup_text(
    link: &[c2_il::SlotArg],
    saved_reg: impl Fn(usize) -> Option<u8>,
) -> Result<Vec<u8>, BackendError> {
    let mut w = Vec::with_capacity(4 * link.len());
    for (i, a) in link.iter().enumerate() {
        // Ascending slot order IS the emission order; the loop is the rule.
        let dst = *ARG_REGS.get(LINK_FIRST_SLOT + i).ok_or_else(|| {
            out_of_class("a chain link's argument past the eight register slots")
        })?;
        match a {
            c2_il::SlotArg::Formal(pi) => {
                let src = saved_reg(*pi).ok_or_else(|| {
                    out_of_class("a chain link reads a formal that is not callee-saved")
                })?;
                w.extend_from_slice(&encode_mr(dst, src));
            }
            // `li rD,k` is `addi rD,0,k` — the same encoder the leaf selector's
            // bare constant goes through, at a register it cannot name.
            c2_il::SlotArg::Lit(k) => {
                let k = i16::try_from(*k)
                    .map_err(|_| out_of_class("a chain link's literal wider than an addi immediate"))?;
                w.extend_from_slice(&encode_addi(dst, 0, k));
            }
            // WR1: never produced for a chain link — `c2_il`'s parser puts a
            // symbol address only in a *tail* call's slot list, because the
            // address would have to survive the previous `bl` and nothing
            // captures where c2 keeps it. The backstop, not the gate.
            c2_il::SlotArg::SymAddr => {
                return Err(out_of_class(
                    "a data symbol's address in a chain link's argument list; \
                     out of class",
                ))
            }
            // **W42** — a `(formal >> k) & m` slot. Produced ONLY by the
            // conditional-tail-pair parser, whose own emitter
            // (`codegen::cond_tail`) is the only consumer with a measured
            // schedule for it. Reaching any other call shape means a parser
            // widened past its witness; refused by name so it comes out as a gap.
            c2_il::SlotArg::ShiftMask { .. } => {
                return Err(out_of_class(
                    "a shift-and-mask (W42) argument outside the conditional                      tail pair; out of class",
                ))
            }
        }
    }
    Ok(w)
}

/// The per-call argument setups and the post-call tail bytes of a framed
/// many-call body, in one place so the packed and `/Gy` emitters share them.
///
/// **Class B** (`seq.saved` non-empty) differs from Class A in two places and
/// nowhere else: the first call's setup gains one `mr rSaved, rArg` per saved
/// formal, and every later call marshals **out of** the saved registers instead of
/// the argument ones. The frame, the prologue and the epilogue are
/// [`FrameLayout`]'s job and the caller sets `saved_gprs` from the same list.
///
/// Byte evidence for the whole shape, `/O1 /GS- /c`:
///
/// ```text
///   void f(int a,int b){ v1(a); v2(b); }                52 B, F = 96
///     mflr r12 ; stw r12,-8(r1) ; std r31,-16(r1) ; stwu r1,-96(r1)
///     mr r31,r4 ; bl ?v1 ; mr r3,r31 ; bl ?v2
///     addi r1,r1,96 ; lwz r12,-8(r1) ; mtlr r12 ; ld r31,-16(r1) ; blr
///
///   void f(int a,int b,int c){ v1(a); v2(b); v3(c); }   72 B, F = 112
///     … std r30,-24(r1) ; std r31,-16(r1) ; stwu r1,-112(r1)
///     mr r31,r4 ; mr r30,r5 ; bl ?v1 ; mr r3,r31 ; bl ?v2 ; mr r3,r30 ; bl ?v3
///     … ld r30,-24(r1) ; ld r31,-16(r1) ; blr
/// ```
///
/// The save moves are emitted **after** the first call's own setup, which the IL
/// parser keeps empty for this class ([`c2_il`]'s `plan_saved_gprs` records the
/// measured exception and why it is refused rather than modeled).
pub fn call_seq_parts(
    params: &[u32],
    seq: &c2_il::CallSeq,
    mode: OptMode,
) -> Result<(Vec<Vec<u8>>, Vec<u8>, SeqPark), BackendError> {
    let mut park = SeqPark::default();
    // The TOTAL, not `seq.saved.len()`: a tail that keeps an earlier call's
    // result alive takes a register from the same file, so a body with two saved
    // formals and such a tail is already the helper class.
    if seq.saved_gprs() > SAVED_GPRS.len() {
        return Err(out_of_class(
            "three or more callee-saved GPRs: that is the `__savegprlr_N` helper \
             class, with a second REL24 site, a tail-branch epilogue and a \
             different /Gy label stride",
        ));
    }
    // Where each saved formal lives once it is saved: parameter index -> register.
    let saved_reg = |pi: usize| -> Option<u8> {
        seq.saved.iter().position(|&s| s == pi).map(|k| SAVED_GPRS[k])
    };

    // **Board #844 — the composition seam.** A sequence carrying a store run is
    // the one shape whose first call's setup is not an argument marshalling at
    // all: it is the whole scheduled run with the callee-saved copy spliced
    // through it. Asked HERE, ahead of the marshalling arms below, and it
    // RETURNS — the arms below cannot see the run and cannot half-emit it.
    //
    // The gate is the parser's own, restated (`store_run_call::gate_composition`)
    // so the census and the emitter cannot disagree silently, and it is asked
    // BEFORE the splice so a body outside the class comes out as a refusal
    // rather than as bytes.
    if let Some(prefix) = &seq.store_run {
        super::store_run_call::gate_composition(seq)?;
        let src = saved_reg(0).ok_or_else(|| {
            out_of_class("a store-run composition whose receiver is not callee-saved")
        })?;
        let setup = super::store_run_call::store_run_prefix_text(params, prefix, src)?;
        // The tail is `SeqTail::SavedFormal { param: 0 }` by the gate above, and
        // it goes through the same `match seq.tail` every other sequence uses —
        // no second lowering of `mr r3,r31`.
        let tail = moves_descending(&[(RET_REG, src)]);
        return Ok((vec![setup], tail, park));
    }
    // **The first call's RESULT**, when the tail consumes it across a later `bl`.
    // `docs/CODEGEN_FRAMED_CALLS.md` §3.1: "call results take the next descending
    // register after the parameters" — so it is `SAVED_GPRS[seq.saved.len()]`,
    // never a fixed r30. Captured at both widths: `bool f(const U* p){ return
    // p->m() == p->n(); }` saves one formal and takes r30 for the result, and the
    // hypothetical zero-formal form would take r31.
    let result_reg: Option<u8> = seq
        .tail
        .saves_a_call_result()
        .then(|| SAVED_GPRS.get(seq.saved.len()).copied())
        .flatten();

    let mut setups = Vec::with_capacity(seq.calls.len());
    for (i, c) in seq.calls.iter().enumerate() {
        // **WCL** — a chain link is its own arm, and it is asked FIRST because
        // the two arms below both key on Class A/Class B, and a link's lowering
        // is the same in either: its arguments come out of the callee-saved file
        // when there are any and out of nowhere when there are not. The Class A
        // chain with a literal link argument (`p->Next()->gia(7)` → `li r4,7`)
        // is exactly the case that would take the wrong arm otherwise, and it
        // would take it silently, into `li r3,7`.
        let mut setup = if let Some(link) = &c.link_args {
            if i == 0 {
                // A link is never the first call: its receiver is a previous
                // call's result. The IL parser builds it that way; this is the
                // backstop, because the arm below would then emit no saves.
                return Err(out_of_class("a chain link in the first call position"));
            }
            link_setup_text(link, saved_reg)?
        } else if i == 0 || seq.saved.is_empty() {
            // The first call still reads its arguments out of the argument
            // registers — nothing has been clobbered yet — so it goes through the
            // same locators every other call shape uses. Class A takes this arm
            // for every call.
            //
            // **The save moves interleave with it, and where is measured.** A save
            // whose source register this marshalling OVERWRITES is hoisted in
            // front of the whole marshalling; one whose source it leaves alone is
            // emitted after it. Both halves in one capture,
            // `void f(int a,int b,int c,int d){ g2(a,d); v1(b); v2(c); }`:
            //
            // ```text
            //   mr r31,r4    b — r4 is about to be overwritten, so HOISTED
            //   mr r4,r6     the marshalling (slot 1 <- d)
            //   mr r30,r5    c — r5 is untouched, so it TRAILS
            //   bl ?g2
            // ```
            //
            // The hoist clears the **whole** marshalling, not just the instruction
            // that would clobber: `void f(int a,int b,int c,int d,int e){
            // g3(a,d,e); v1(b); }` is `mr r31,r4 ; mr r5,r7 ; mr r4,r6`, which
            // refutes the "save as late as possible" reading that fits the row
            // above. Within each group the moves go highest destination first,
            // which for the saves is the same as parameter order.
            let (text, writes) = match (&c.arg_slots, c.arg_ops.as_slice()) {
                // A non-identity permutation beside a save is refused by the IL
                // parser: c2 breaks the cycle with the **callee-saved register**
                // rather than r11 there, which is a different algorithm and not
                // this interleaving at all. This is the backstop.
                (Some(slots), _) => {
                    // **W-MEMCPY — a framed sequence call's slots are NO LONGER
                    // all formals.** This comment used to read *"a framed
                    // sequence call's slots are all formals by construction —
                    // `c2_il`'s `seq_call_arg_sources` is what refuses a
                    // literal on the way in"*, and lane `w-memcpy` widened that
                    // locator over GRID-L's 747 cells. `permute_args_parts`
                    // already took a `&[SlotArg]` and already dispatches a list
                    // carrying a literal to `lit_slots_text`, so the *unparked*
                    // half needed no change at all — only `formal_slots`, the
                    // adapter that threw the literal away, had to go.
                    //
                    // **The two branches below now key on `permuted` rather
                    // than on "the setup text is non-empty", and that is not
                    // cosmetic**: a list with a literal and no permutation has
                    // non-empty setup and must NOT take the park branch, where
                    // `seq_entry_park` would be handed an identity permutation.
                    // 144 of GRID-L's cells are exactly that shape and every
                    // one of them is a bare `li` at the call site.
                    let sources = c2_il::slot_sources(slots);
                    let permuted = sources.iter().enumerate().any(|(d, &s)| s != d);
                    let has_lit =
                        slots.iter().any(|a| matches!(a, c2_il::SlotArg::Lit(_)));
                    if !seq.saved.is_empty() && (permuted || has_lit) {
                        return Err(out_of_class(
                            "a permuted or literal-carrying first call beside a callee-saved copy: c2                              breaks the cycle through the callee-saved register                              instead of r11, which is not characterized",
                        ));
                    }
                    // **W-CLEAR / board #275 — a permuted call behind a GUARDED
                    // EARLY RETURN is c2's ENTRY-BLOCK PARK, and this emitter
                    // does not have it.** Refused rather than emitted, because
                    // the bytes it would emit are WRONG, not merely absent.
                    //
                    // [`SeqGuardEmit`]'s doc says a guarded sequence with a park
                    // "is refused in the IL parser, so the two rules cannot be
                    // confused here". **That was untrue and untested.** Lane
                    // `w-clear` gridded guard-count × permutation × call-count at
                    // the dc3 workload's own flags —
                    // `work/w-clear/probe/grid.txt`, 54 cells — and read **30
                    // `Port=Mismatch`**: every cell with ≥ 1 early return and a
                    // non-identity permutation. `mismatch 0` is this project's
                    // sole correctness criterion and this shape broke it.
                    //
                    // What c2 does, measured on those cells:
                    //
                    // ```text
                    //   f(p,q){ g(q,p); }            mr r11,r4 · mr r4,r3 · mr r3,r11   <- the port MATCHES this
                    //   f(p,q){ if(!p) …; g(q,p); }  mr r11,r3 · mr r3,r4               <- ENTRY BLOCK, before the compare
                    //                                cmplwi cr6,r11,0 · bf 26 · …
                    //                                mr r4,r11                          <- and only the remainder here
                    // ```
                    //
                    // The unguarded row is already byte-exact, so this is not a
                    // missing encoder: it is a *different cycle break* plus a
                    // *split across two blocks*, chosen by a rule the rung doc
                    // states and no fixture had ever exercised.
                    //
                    // **W-MMIO — and this is where the park is now BUILT.**
                    // [`seq_entry_park`] carries the rule and the population it
                    // was measured over; what stays refused here is every shape
                    // it declines — a scrutinee outside the cycle, a chain that
                    // is not unimodal, a cycle past three — plus the two the
                    // grid never crossed: a park beside a callee-saved copy
                    // (already refused above) and a park in any call but the
                    // first.
                    if !seq.early.is_empty() && permuted {
                        if i != 0 {
                            return Err(out_of_class(
                                "a permuted call after the first behind a guarded \
                                 early return: the park is an ENTRY-BLOCK move and \
                                 there is no capture of one belonging to a later call",
                            ));
                        }
                        let (entry, call, first, later) =
                            seq_entry_park(&sources, seq.early[0].cmp_param)?;
                        park = SeqPark { entry, first, later };
                        // The write set the callee-saved interleaving reads is
                        // the CALL's, not the whole permutation's — but a park
                        // beside a save is refused two arms up, so it is empty
                        // by construction and stated rather than derived.
                        (park_call_with_literals(&call, slots)?, Vec::new())
                    } else {
                        permute_args_parts(slots)?
                    }
                }
                (None, []) => (Vec::new(), Vec::new()),
                // A single passthrough or literal argument selects to `mr r3,rN`
                // or `li r3,k` — one word writing r3, or nothing at all when the
                // value is already there.
                (None, [IlOp::Load(_)]) | (None, [IlOp::Lit(_)]) => {
                    let t = ops_setup_text(params, &c.arg_ops, mode)?;
                    let w = if t.is_empty() { Vec::new() } else { vec![RET_REG] };
                    (t, w)
                }
                // Anything computed. Under `/Ox` a chain intermediate goes to a
                // fresh **descending** register, which is the same file the saves
                // live in, so the write set is not `{r3}` and the interleaving is
                // not the measured one. The IL parser refuses this while anything
                // is saved; this is the backstop.
                (None, _) if !seq.saved.is_empty() => {
                    return Err(out_of_class(
                        "a computed first-call argument beside a callee-saved copy: \
                         the marshalling's write set reaches the callee-saved file \
                         and the interleaving is not characterized",
                    ))
                }
                (None, ops) => (ops_setup_text(params, ops, mode)?, Vec::new()),
            };
            let mut hoisted = Vec::new();
            let mut trailing = Vec::new();
            for (k, &pi) in seq.saved.iter().enumerate() {
                let src = *ARG_REGS.get(pi).ok_or_else(|| {
                    out_of_class("a stack-homed formal cannot be copied to a callee-saved GPR")
                })?;
                let mv = (SAVED_GPRS[k], src);
                if writes.contains(&src) {
                    hoisted.push(mv);
                } else {
                    trailing.push(mv);
                }
            }
            let mut w = moves_descending(&hoisted);
            w.extend_from_slice(&text);
            w.extend_from_slice(&moves_descending(&trailing));
            w
        } else {
            // A later call in Class B: every formal it reads is in a callee-saved
            // register by construction (that is what put it there), and a literal
            // argument is the same `li r3,k` the leaf selector emits.
            match (&c.arg_slots, c.arg_ops.as_slice()) {
                (Some(slots), _) => {
                    let mut moves = Vec::with_capacity(slots.len());
                    for (slot, a) in slots.iter().enumerate() {
                        // **A literal in a call after the first is refused, and
                        // it is an UNMEASURED shape rather than a measured
                        // refusal** — GRID-L put a literal in the first call of
                        // every one of its 747 cells and in a later one never.
                        // The IL parser refuses the same shape by name
                        // (`callseq-multiarg-lit-later-call`); this is the
                        // backstop, board #139.
                        let &c2_il::SlotArg::Formal(pi) = a else {
                            return Err(out_of_class(
                                "a call after the first carries a slot that is not \
                                 a formal: no capture puts one there",
                            ));
                        };
                        let (Some(src), Some(&dst)) = (saved_reg(pi), ARG_REGS.get(slot)) else {
                            return Err(out_of_class(
                                "a call after the first reads a value that is not \
                                 in a callee-saved register",
                            ));
                        };
                        moves.push((dst, src));
                    }
                    moves_descending(&moves)
                }
                (None, [c2_il::IlOp::Load(_)]) => {
                    // `params` and the operand agree by construction; the parser
                    // resolved the token to a formal index before accepting.
                    let pi = seq_load_param_index(params, &c.arg_ops)?;
                    let src = saved_reg(pi).ok_or_else(|| {
                        out_of_class("a call after the first reads an unsaved formal")
                    })?;
                    moves_descending(&[(RET_REG, src)])
                }
                (None, ops) => ops_setup_text(params, ops, mode)?,
            }
        };
        // **Save the first call's result** before the second call's setup can
        // clobber r3. Measured, every capture of the shape:
        //
        // ```text
        //   mr r30,r3      the first result -> its callee-saved register
        //   mr r3,r31      the second call's receiver -> r3
        //   bl ?n
        // ```
        //
        // Emitted as a prefix rather than folded into `moves_descending`'s set:
        // the two are not one non-conflicting group — `mr r3,r31` reads r31 and
        // `mr r30,r3` reads r3, so their order is a data dependence, not a
        // sort key. (Descending order happens to agree here, and relying on that
        // coincidence is how a later widening with a lower-numbered save register
        // would silently invert them.)
        if i == 1 {
            if let Some(rr) = result_reg {
                let mut w = encode_mr(rr, RET_REG).to_vec();
                w.extend_from_slice(&setup);
                setup = w;
            }
        }
        setups.push(std::mem::take(&mut setup));
    }
    let tail = match seq.tail {
        c2_il::SeqTail::Void => Vec::new(),
        // The result is already in r3; `+0` folds away exactly as it does for the
        // single framed call.
        c2_il::SeqTail::CallValue { add_k: 0 } => Vec::new(),
        // **W-FLTRET — the same fold, in the other register file.** The callee
        // left a `float`/`double` in `f1` and the caller's return reads `f1`, so
        // there is nothing to emit and nothing to elide: this arm is `Vec::new()`
        // because the instruction stream genuinely IS the integer one, measured
        // word for word off c2's `/FAsc` listing (`work/w-fltret/probe/v3.cod`).
        //
        // The whole difference is a TU-level symbol. `coff::Function::is_float`
        // carries it, fed by `c2_il::IlFunction::touches_floating_point`, which
        // matches this tail; a body that reached here without it emits an obj one
        // symbol short — `Port=Mismatch @ offset 12`, the COFF header's
        // `NumberOfSymbols`, on every positive case at once.
        c2_il::SeqTail::CallValueFp => Vec::new(),
        c2_il::SeqTail::CallValue { add_k } => {
            let k = i16::try_from(add_k)
                .map_err(|_| out_of_class("call-sequence post-op wider than an addi immediate"))?;
            encode_addi(RET_REG, RET_REG, k).to_vec()
        }
        // `return <literal>;` — the same `li r3,k` a bare-literal leaf emits, so it
        // goes through the same selector rather than a second encoder.
        c2_il::SeqTail::Lit(k) => ops_setup_text(params, &[IlOp::Lit(k)], mode)?,
        // **WCO — `return p->a()->b()->m;`**: the last call left a pointer in r3
        // and the body reads through it. One `lwz r3,off(r3)`, in exactly the
        // position the `CallValue` post-op's `addi` occupies.
        //
        // **It does NOT fold at offset 0**, and that is the one rule this tail
        // does not share with its sibling: `CallValue { add_k: 0 }` above emits
        // nothing because `r3 + 0` is already the answer, while `*(r3 + 0)` is a
        // memory read that has to happen. MEASURED, two functions one line apart
        // in `work/WCO/probe/p1.cpp` at `/O1 /GS- /c` — `c_off0` is a 40-byte
        // body ending `lwz r3,0(r3)` and `c_addr0` is a 36-byte body with no
        // instruction between the `bl` and the epilogue.
        //
        // The width is 4 by construction (the IL parser admits only a 4-byte
        // integer or a 4-byte pointer here and names every other width); a
        // narrow or 8-byte member is `lbz`/`lhz`/`ld` and a `float` one is
        // `lfs` into f1, which is a different register file.
        c2_il::SeqTail::CallLoad { off } => {
            let d = i16::try_from(off).map_err(|_| {
                out_of_class("call-sequence tail load offset exceeds a 16-bit displacement")
            })?;
            crate::codegen::encode::encode_lwz(RET_REG, RET_REG, d).to_vec()
        }
        // **WFL — the same read-through whose member is floating point**: one
        // `lfs`/`lfd` into **f1**, the other register file. Delegated to
        // [`crate::codegen::leaf::float::chain_result_fp_load_text`] rather than
        // encoded here, beside the FP leaf's register model and for the same
        // reason `fp_tail_call_text` lives there: what the instruction decides is
        // that the destination is f1 and not r3. The base register is this
        // sequence's contribution — the last `bl` left the pointer in r3.
        //
        // The TU-level half of this tail is `_fltused`, produced by
        // `c2_il::IlFunction::touches_floating_point` and consumed by
        // `coff::Function::is_float`; a body that reached here without it emits
        // an obj one symbol short.
        c2_il::SeqTail::CallLoadFp { off, double } => {
            crate::codegen::leaf::float::chain_result_fp_load_text(RET_REG, off, double)?
        }
        // **WCB/WCR — `return a->m() <rel> b->n();`**, the register-register
        // comparison spines (`docs/CMP_PRODUCES_A_VALUE.md` reading 4). All three
        // — `==`, signed order, unsigned order — live in
        // [`crate::codegen::leaf::compare::cmp_of_two_call_results`] beside the
        // *leaf* comparison spines they share their temp-allocation rule with,
        // rather than here beside the call sequence they share a frame with.
        //
        // The operand roles are **not** the emission order: the first call's
        // result is in `result_reg` and the second's is still in r3, and which of
        // those is the source's left operand is `lhs_first`. c2 orders the calls
        // by the order c1xx NUMBERED their receivers (`this` last, whatever
        // register it is in) and keeps the spine's operands in source order, so
        // both facts are needed and neither implies the other.
        c2_il::SeqTail::Cmp { cmp, lhs_first } => {
            crate::codegen::leaf::compare::cmp_of_two_call_results(cmp, lhs_first, result_reg, mode)?
        }
        // **WEC — `return <a callee-saved formal>;`**: one `mr r3, rSaved`, in
        // exactly the position `CallValue`'s `addi` post-op occupies.
        //
        // The register is re-derived from `seq.saved` through the same
        // `saved_reg` closure every other consumer here uses, and a formal that
        // is not saved is an error rather than a guessed register: this tail
        // exists precisely because the value predates the last `bl`, so reading
        // it out of an argument register would be reading a clobbered one.
        c2_il::SeqTail::SavedFormal { param } => {
            let src = saved_reg(param).ok_or_else(|| {
                out_of_class("the tail returns a formal that is not callee-saved")
            })?;
            moves_descending(&[(RET_REG, src)])
        }
    };
    Ok((setups, tail, park))
}

/// Emit the `.text` for an **integer tail call** `return g(<arg>)` (and the
/// identity-fold `g(a) + 0`): the single call argument computed into r3 by the
/// leaf arithmetic selector, then a `b <callee>` tail branch (paired with a
/// REL24 relocation the caller registers at the returned offset). Returns the
/// `.text` bytes and the branch's byte offset within `.text` (= `base_off` +
/// the arg-setup length; the reloc site).
///
/// The argument setup reuses [`select_text`] (params → r3, the exact same
/// instruction class as a leaf `return <arg>`) and drops its trailing `blr`:
/// the argument value stays live in r3 across the tail branch. A passthrough
/// `g(a)` selects to just `blr` → an empty prefix → a bare `b g` at `base_off`,
/// byte-identical to the void tail call; `g(a + 1)` selects `addi r3,r3,1 ; blr`
/// → the prefix `addi r3,r3,1` and the branch at `base_off + 4`. Non-commutative
/// arg-setup (`k - a`, shifts) is rejected inside `select_text` (fail closed),
/// honoring the CLAUDE.md correctness boundary; `a + k`/`a - k` fold to `addi`.
pub fn int_tail_call_text(
    func: &IlFunction,
    base_off: u32,
    mode: OptMode,
) -> Result<(Vec<u8>, u32), BackendError> {
    let mut text = select_text(func, mode)?; // arg-setup + trailing blr
    let blr = encode_blr();
    debug_assert!(
        text.ends_with(&blr),
        "select_text always terminates in blr; the arg-setup is everything before it"
    );
    text.truncate(text.len() - blr.len()); // drop blr; the value stays live in r3
    let branch_off = base_off + text.len() as u32;
    text.extend_from_slice(&encode_tail_branch(branch_off));
    Ok((text, branch_off))
}

/// Lower an **argument permutation** for a multi-argument tail call: emit the
/// register moves that put each argument register's wanted value in place, with
/// `r11` as the single break temp.
///
/// `sources[i]` is the parameter index whose value argument slot `i` wants, so
/// slot `i` must end up holding what `ARG_REGS[sources[i]]` holds on entry.
/// `sources` being the identity is the passthrough case and emits nothing.
///
/// The rule, from the captures in `fixtures/cpp/il_call_multi.cpp`: decompose the
/// permutation into cycles; for each cycle save the source of its **lowest**
/// destination into the temp, then assign along the cycle in the order forced by
/// clobbering, filling that lowest destination from the temp last.
///
/// Only a **single** non-trivial cycle is accepted. Two disjoint cycles do both
/// saves up front (r11 then r10) and then have several clobber-free orders to
/// choose between, and the one capture available does not determine which — see
/// `rev4` in that fixture. A repeated argument is also refused: `dup3` emits a
/// *dead* `mr r11,r4`, which no live-value-driven solver would produce.
pub fn permute_args_text(sources: &[c2_il::SlotArg]) -> Result<Vec<u8>, BackendError> {
    permute_args_parts(sources).map(|(text, _)| text)
}

/// A slot list of nothing but formals, for the tests that spell a permutation
/// as bare indices.
///
/// **It used to have a production caller** — a framed sequence call's
/// `arg_sources` was an `Option<Vec<usize>>` and this adapter widened it on the
/// way into [`permute_args_parts`], throwing any literal away because there
/// could not be one. Lane `w-memcpy` gave `SeqCall` a slot list, so the adapter
/// has no production caller left and is `cfg(test)` rather than deleted: a
/// permutation written as `[1, 0]` is a great deal easier to read in a test
/// than as two `SlotArg::Formal`s.
#[cfg(test)]
fn formal_slots(sources: &[usize]) -> Vec<c2_il::SlotArg> {
    sources.iter().map(|&i| c2_il::SlotArg::Formal(i)).collect()
}

/// **WLA — the literal argument slots of a tail call**, `li r<3+i>,k` each.
///
/// Reached only when every non-literal slot is the formal already sitting in its
/// own argument register (`c2_il`'s `lit_arg_tail_call` is the primary gate;
/// the `in_place` check below is the backstop), so there is no move to schedule
/// against and the whole setup is the constants.
///
/// **The order is DESCENDING destination**, the same rule
/// [`moves_descending`] uses and the opposite of a chain link's. Captured,
/// `work/WLA/probe/p1.cpp` at `/O1 /GS- /c`:
///
/// ```text
///   void f(int a,int b) { g3(a, b, 7); }   38a00007  li 5,7
///   void f(int a)       { g3(a, 5, 6); }   38a00006  li 5,6
///                                          38800005  li 4,5     <- 6 before 5
/// ```
/// **WLB — `g2(b, 7)`**: two argument slots, the literal in slot 1 and slot 0
/// wanting a formal that is **not** in r3.
///
/// The order is not a fixed one, and that is the whole content of this function.
/// c2's default is [`moves_descending`]'s — highest destination first, which
/// puts the `li` in front — and it **hoists** the move ahead of the `li` exactly
/// when the `li`'s destination register is the one holding the value the move
/// needs. Both cells, one probe TU apart (`work/WLA/probe/p2.cpp`, `/O1 /GS- /c`):
///
/// ```text
///   void f(int a,int b)       { g2(b, 7); }   7c832378 mr r3,r4 · 38800007 li 4,7
///   void f(int a,int b,int c) { g2(c, 7); }   38800007 li 4,7   · 7ca32b78 mr r3,r5
/// ```
///
/// — the same hoist/trail rule [`call_seq_parts`] applies to the callee-saved
/// copies, which is why it is *recognised* here rather than discovered.
///
/// **Two slots only, and the reason is measured.** At three the same probe has
/// `g3(c,b,7)` (`mr r3,r5 ; li r5,7`) and `g3(b,c,7)`
/// (`mr r3,r4 ; mr r4,r5 ; li r5,7`) following the hoist, and `g3(c,a,7)` —
/// one formal moving up while another moves down — going
/// `mr r11,r5 ; mr r4,r3 ; li r5,7 ; mr r3,r11`, with the `li` *inside* the
/// break walk. Any rule fitted to the first two mis-emits the third. The IL
/// parser is the gate (`call-arg-lit-permuted`); this is the backstop.
fn one_moved_formal_text(slots: &[c2_il::SlotArg]) -> Result<(Vec<u8>, Vec<u8>), BackendError> {
    let [c2_il::SlotArg::Formal(pi), c2_il::SlotArg::Lit(k)] = slots else {
        return Err(out_of_class(
            "a literal argument beside a formal that has to move, in a list that \
             is not the captured two-slot one: at three slots c2 breaks through \
             r11 and emits the `li` inside the walk, which is not characterized",
        ));
    };
    let src = *ARG_REGS.get(*pi).ok_or_else(|| {
        out_of_class("a call argument sources a stack-homed parameter")
    })?;
    let k = i16::try_from(*k)
        .map_err(|_| out_of_class("a literal argument wider than an addi immediate"))?;
    let mv = encode_mr(ARG_REGS[0], src);
    let li = encode_addi(ARG_REGS[1], 0, k);
    let mut w = Vec::with_capacity(8);
    // The hoist, stated as the dependence it is: the `li` writes slot 1's
    // register, and the move reads it.
    if src == ARG_REGS[1] {
        w.extend_from_slice(&mv);
        w.extend_from_slice(&li);
    } else {
        w.extend_from_slice(&li);
        w.extend_from_slice(&mv);
    }
    Ok((w, vec![ARG_REGS[0], ARG_REGS[1]]))
}

/// **WR1 — the argument setup of a tail call one of whose slots is a NAMED DATA
/// SYMBOL's address.**
///
/// ```text
///   lis  r11,sym@ha          <- ALWAYS the function's first word (REFHI + PAIR)
///   <the FIRST word of the descending non-address walk, if there is one>
///   addi rD,r11,sym@l        <- the address (REFLO + PAIR), in SECOND place
///   <the REST of the descending walk>
///   b    <callee>
/// ```
///
/// Byte evidence, read off the reference obj (`work/wr1/probes/p2.cpp`, `/Ox
/// /GS- /c`); the relocation quad is `c2_core::coff`'s and the offsets are
/// the `lis`'s own and the `lis`'s + 4:
///
/// ```text
///   void a5()            { g1("ee"); }      3d600000 · 386b0000        · b
///   void a1(S* s)        { s->m1("aa"); }   3d600000 · 388b0000        · b
///   void a8(int j,int k) { g4(j,k,"hh"); }  3d600000 · 38ab0000        · b
///   void a9(a..g)        { g8(a..g,"ii"); } 3d600000 · 394b0000        · b
///   void c1()            { g2("jj", 7); }   3d600000 · 38800007 li r4,7
///                                                    · 386b0000        · b
/// ```
///
/// **The `lis` is hoisted to the top, and the `addi` is emitted after EXACTLY ONE
/// word of the descending non-address walk** — second place, whatever slot the
/// address belongs to, and first when the walk is empty. WR1's discriminating
/// pair is still the reason the position is not the descending walk's own:
///
/// ```text
///   void f()     { gsp(&gI, 7);   }   lis r11 · 38800007 li r4,7 · 386b0000 addi r3
///   void f(S* s) { s->m3(7, &gI); }   lis r11 · 38800007 li r4,7 · 38ab0000 addi r5
/// ```
///
/// — the same instruction order with the symbol at slot 0 and at slot 2, so
/// **descending and address-second agree on the first and disagree on the
/// second**. Six sweep cases mismatched at obj offset 541 on the descending
/// reading (`scripts/sweep.d/53-data-symbol-addr.py`).
///
/// **W-ADJUST — the rule this replaces was "the `addi` goes LAST", and that was
/// wrong from two setup words up.** At one word the two readings are the same
/// sequence, and WR1's whole corpus — fixture and sweep — had at most one word
/// beside the address, so nothing could tell them apart. The differential found
/// it on the first three-word case:
///
/// ```text
///   void b3() { gs3(&gI, 3, 4); }     lis r11 · li r5,4 · addi r3 · li r4,3
///                       address-last:  lis r11 · li r5,4 · li r4,3 · addi r3   WRONG
/// ```
///
/// `b3` is a **pure WR1 shape** — a data symbol as a call argument, no receiver
/// anywhere — and it was in class and mis-emitting on mainline before this rung
/// existed; the rung only supplied the arity that reaches it. Eleven cells now
/// pin the rule, every word read off c2's own `.cod` listing at `/O1 /Oi /EHsc`
/// (`work/wadjust/probe/q1.cpp`, `q3.cpp`), and they cover the address at slot 0
/// and at a middle slot, walks of length 0 through 4, literals and in-place
/// formals in the walk, and both a free and a member caller:
///
/// ```text
///   gs2(&gI,7)          li r4,7          · addi r3
///   gs3(&gI,b,7)        li r5,7          · addi r3
///   gs3(&gI,3,4)        li r5,4          · addi r3 · li r4,3
///   gs4(&gI,3,4,5)      li r6,5          · addi r3 · li r5,4 · li r4,3
///   gDbg.four(3,4,5,6)  li r7,6          · addi r3 · li r6,5 · li r5,4 · li r4,3
///   gs3b(3,&gI,4)       li r5,4          · addi r4 · li r3,3
///   gs4b(3,&gI,4,5)     li r6,5          · addi r4 · li r5,4 · li r3,3
///   Fwd::m2(k)          li r6,8          · addi r3 · li r5,7
///   s->m3(7,&gI)        li r4,7          · addi r5                     (WR1 c4)
///   s->so(&gI)          (empty walk)     · addi r4                     (WR1 a1)
///   gso(&gI)            (empty walk)     · addi r3                     (WR1 a3)
/// ```
///
/// The scratch is **r11** in every witness. It becomes r10 only in the shape
/// this class refuses — two formals shifting, where c2 pre-saves into r11 first
/// (`a4`, `docs/IL_CALL_IN_EXPR.md` §17.3 (d)) — so the register is not a free
/// variable here, it is the one the captured cells use.
///
/// The gate is `c2_il`'s `sym_addr_tail_call`; the checks below are the backstop
/// (`docs/GAPS.md` §6 #9 — one fact, and the second copy is the one that drifts,
/// so this one only ever *refuses*).
fn sym_slots_text(slots: &[c2_il::SlotArg]) -> Result<(Vec<u8>, Vec<u8>), BackendError> {
    if slots.iter().filter(|a| matches!(a, c2_il::SlotArg::SymAddr)).count() != 1 {
        return Err(out_of_class(
            "two or more data-symbol addresses in one call: c2 materializes only \
             the first through a relocation pair and derives the rest by .rdata \
             pool-offset difference; out of class",
        ));
    }
    let in_place = slots.iter().enumerate().all(|(i, a)| match a {
        c2_il::SlotArg::SymAddr | c2_il::SlotArg::Lit(_) => true,
        c2_il::SlotArg::Formal(pi) => *pi == i,
        // **W42** — never produced for this shape; `false` routes it to the
        // refusal below rather than to a schedule that cannot express it.
        c2_il::SlotArg::ShiftMask { .. } => false,
    });
    if !in_place {
        return Err(out_of_class(
            "a data symbol's address beside a formal that has to move: at two \
             shifting formals c2 pre-saves into r11 and moves the `lis` to r10, \
             which one probe does not separate from the one-move schedule; out \
             of class",
        ));
    }
    if slots.len() > ARG_REGS.len() {
        return Err(out_of_class(
            "a data symbol's address past the eight register slots; out of class",
        ));
    }
    // The hoisted high half. Its `.text` offset is 0 within this body, which is
    // what lets the caller register REFHI/REFLO at the function's own start
    // without codegen threading an offset back — and `crate::PortC2` checks the
    // first word against this encoding rather than assuming it.
    let mut w = Vec::with_capacity(4 * (slots.len() + 1));
    w.extend_from_slice(&encode_addis(SCRATCH_REG, 0, 0));
    let mut writes = vec![SCRATCH_REG];
    // The non-address slots, descending destination — the same walk
    // [`lit_slots_text`] makes, and the address is not part of it.
    let mut sym_dst: Option<u8> = None;
    let mut walk: Vec<(u8, [u8; 4])> = Vec::with_capacity(slots.len());
    for (i, a) in slots.iter().enumerate().rev() {
        let dst = *ARG_REGS.get(i).ok_or_else(|| {
            out_of_class("a call argument past the eight register slots")
        })?;
        match a {
            c2_il::SlotArg::Formal(_) => continue,
            c2_il::SlotArg::Lit(k) => {
                let k = i16::try_from(*k)
                    .map_err(|_| out_of_class("a literal argument wider than an addi immediate"))?;
                walk.push((dst, encode_addi(dst, 0, k)));
            }
            c2_il::SlotArg::SymAddr => {
                sym_dst = Some(dst);
            }
            // **W42** — unreachable: the `in_place` gate above already refused
            // it. Stated as a refusal, not an `unreachable!`, because the CLI
            // must degrade cleanly.
            c2_il::SlotArg::ShiftMask { .. } => {
                return Err(out_of_class(
                    "a shift-and-mask (W42) argument beside a data symbol's \
                     address; out of class",
                ))
            }
        }
    }
    let dst = sym_dst.ok_or_else(|| out_of_class("no data-symbol slot after the count said one"))?;
    // …and the low half **immediately after the FIRST word of that walk** — see
    // the rule stated above. `sym@l` is 0 before the linker patches it, exactly
    // as the pooled-FP-constant `lfs`'s displacement is.
    let emit = |reg: u8, word: [u8; 4], w: &mut Vec<u8>, writes: &mut Vec<u8>| {
        w.extend_from_slice(&word);
        writes.push(reg);
    };
    let mut it = walk.into_iter();
    if let Some((r, word)) = it.next() {
        emit(r, word, &mut w, &mut writes);
    }
    emit(dst, encode_addi(dst, SCRATCH_REG, 0), &mut w, &mut writes);
    for (r, word) in it {
        emit(r, word, &mut w, &mut writes);
    }
    Ok((w, writes))
}

/// **W-VSNPRNC — the formals in order with ONE LITERAL SPLICED IN.**
///
/// `vsnprnc.cpp`'s `vsprintf_s`: four formals forwarded to a five-argument
/// callee with a `0` in slot 3, so slot 4 wants the formal sitting one register
/// low and everything below slot 3 is already home.
///
/// ```text
///   mr r7,r6      the formals AT AND ABOVE the literal's slot, each one up,
///   li r6,0       emitted DESCENDING; then the literal, into the register the
///   b callee      last of those moves just read
/// ```
///
/// The accept/refuse boundary is `c2_il`'s `lit_insert_at`, which carries
/// GRID-L's eighteen graded cells and — deliberately — the statement of what
/// that grid does **not** settle. This is the backstop, and it re-derives the
/// slot list rather than trusting it, so the two cannot drift.
///
/// `Ok(None)` means "not this shape", so the caller can fall through to the WLB
/// cell. The two are disjoint by construction: WLB's `[Formal(1), Lit]` drops a
/// formal and this requires every formal, in order.
fn lit_insert_shift_text(
    slots: &[c2_il::SlotArg],
) -> Result<Option<(Vec<u8>, Vec<u8>)>, BackendError> {
    // The literal's slot, and the check that the rest is the identity with a
    // hole in it. Written here as well as in the parser for the reason every
    // backstop in this file is: `select_function` is what `function_gate` runs,
    // and a clause that lives on one side only is a clause the two can disagree
    // about silently (board #1638).
    let mut lit: Option<(usize, i32)> = None;
    for (i, a) in slots.iter().enumerate() {
        match a {
            c2_il::SlotArg::Lit(k) => {
                if lit.is_some() {
                    return Ok(None);
                }
                lit = Some((i, *k));
            }
            c2_il::SlotArg::Formal(ix) => {
                let want = if lit.is_some() { i - 1 } else { i };
                if *ix != want {
                    return Ok(None);
                }
            }
            _ => return Ok(None),
        }
    }
    let Some((j, k)) = lit else { return Ok(None) };
    if slots.len() > ARG_REGS.len() {
        return Err(out_of_class(
            "an inserted literal past the eight register slots: the rest are \
             stack-homed; out of class",
        ));
    }
    let k = i16::try_from(k)
        .map_err(|_| out_of_class("a literal argument wider than an addi immediate"))?;
    let mut w = Vec::with_capacity(4 * (slots.len() - j));
    let mut writes = Vec::new();
    // Descending destination, from the top slot down to the one just above the
    // literal. Each move reads the register the next one writes, so this order
    // is the only clobber-free one; the `li` then lands in the register the last
    // move read.
    for dst in (j + 1..slots.len()).rev() {
        w.extend_from_slice(&encode_mr(ARG_REGS[dst], ARG_REGS[dst - 1]));
        writes.push(ARG_REGS[dst]);
    }
    w.extend_from_slice(&encode_addi(ARG_REGS[j], 0, k));
    writes.push(ARG_REGS[j]);
    Ok(Some((w, writes)))
}

fn lit_slots_text(slots: &[c2_il::SlotArg]) -> Result<(Vec<u8>, Vec<u8>), BackendError> {
    let in_place = slots.iter().enumerate().all(|(i, a)| match a {
        c2_il::SlotArg::Lit(_) => true,
        c2_il::SlotArg::Formal(pi) => *pi == i,
        // Unreachable: `permute_args_parts` dispatches a symbol-bearing list to
        // [`sym_slots_text`] before this one is reached.
        c2_il::SlotArg::SymAddr => false,
        // **W42** — likewise never produced here; see `link_setup_text`.
        c2_il::SlotArg::ShiftMask { .. } => false,
    });
    if !in_place {
        // **W-VSNPRNC — the inserted literal**, asked ahead of the WLB two-slot
        // cell because the two are disjoint (see `lit_insert_shift_text`) and
        // this one is the wider list. A list that is neither still lands on
        // `one_moved_formal_text`'s refusal, unchanged.
        if let Some(w) = lit_insert_shift_text(slots)? {
            return Ok(w);
        }
        return one_moved_formal_text(slots);
    }
    let mut w = Vec::new();
    let mut writes = Vec::new();
    // Descending destination: walk the slots from the top.
    for (i, a) in slots.iter().enumerate().rev() {
        let c2_il::SlotArg::Lit(k) = a else { continue };
        let dst = *ARG_REGS.get(i).ok_or_else(|| {
            out_of_class("a literal argument past the eight register slots")
        })?;
        let k = i16::try_from(*k)
            .map_err(|_| out_of_class("a literal argument wider than an addi immediate"))?;
        // `li rD,k` is `addi rD,0,k` — the same encoder the leaf selector's bare
        // constant goes through, at a register it cannot name.
        w.extend_from_slice(&encode_addi(dst, 0, k));
        writes.push(dst);
    }
    Ok((w, writes))
}

/// [`permute_args_text`] plus **the registers its moves write**, which Class B
/// needs in order to decide whether a callee-saved copy has to be hoisted in
/// front of the marshalling (`c2_il`'s `plan_saved_gprs`). It is one function
/// returning two views of one cycle decomposition rather than a second walk of
/// the same permutation: a write set derived independently would be the "two
/// implementations of one rule" shape `docs/GAPS.md` §6 #9 records, and this one
/// cannot drift from the bytes because it is computed beside them.
fn permute_args_parts(slots: &[c2_il::SlotArg]) -> Result<(Vec<u8>, Vec<u8>), BackendError> {
    // **WLA** — a list carrying a literal is asked FIRST, because everything
    // below reads a slot as a formal index and a `Lit` has none. The two forms
    // do not mix in class (`c2_il`'s `lit_arg_tail_call` admits a literal only
    // beside formals that are already in place), so this is a dispatch and not a
    // precedence.
    // **WR1** — a list carrying a data symbol's address is asked FIRST, ahead of
    // the literal path, because the symbol's `lis` is hoisted in front of the
    // whole setup and the literals then take their ordinary descending place
    // beside it. `lit_slots_text` has never seen one and must not be handed it.
    if slots.iter().any(|a| matches!(a, c2_il::SlotArg::SymAddr)) {
        return sym_slots_text(slots);
    }
    if slots.iter().any(|a| matches!(a, c2_il::SlotArg::Lit(_))) {
        return lit_slots_text(slots);
    }
    let mut sources = Vec::with_capacity(slots.len());
    for a in slots {
        match a {
            c2_il::SlotArg::Formal(i) => sources.push(*i),
            // Unreachable by the dispatch above, stated as a refusal rather than
            // an `unreachable!` because the CLI must degrade cleanly.
            c2_il::SlotArg::Lit(_) => {
                return Err(out_of_class("a literal argument reached the permutation walk"))
            }
            c2_il::SlotArg::SymAddr => {
                return Err(out_of_class("a data symbol's address reached the permutation walk"))
            }
            // **W42** — a `(formal >> k) & m` slot. Produced ONLY by the
            // conditional-tail-pair parser, whose own emitter
            // (`codegen::cond_tail`) is the only consumer with a measured
            // schedule for it. Reaching any other call shape means a parser
            // widened past its witness; refused by name so it comes out as a gap.
            c2_il::SlotArg::ShiftMask { .. } => {
                return Err(out_of_class(
                    "a shift-and-mask (W42) argument reached the permutation walk; outside the conditional                      tail pair; out of class",
                ))
            }
        }
    }
    let sources = &sources[..];
    if sources.len() > ARG_REGS.len() {
        return Err(out_of_class(
            "more arguments than the eight register slots: the rest are \
             stack-homed; out of class",
        ));
    }
    // A value wanted by two slots is a duplicate, which emits a dead move.
    for (i, s) in sources.iter().enumerate() {
        if sources[..i].contains(s) {
            return Err(out_of_class(
                "an argument value is passed twice: c2 emits a dead `mr` through \
                 the temp for this shape; out of class",
            ));
        }
        if *s >= ARG_REGS.len() {
            return Err(out_of_class("argument sources a stack-homed parameter"));
        }
    }

    // Cycle-decompose. `sources[i] == i` is a fixed point and needs no move.
    let n = sources.len();
    let mut seen = vec![false; n];
    let mut cycles: Vec<Vec<usize>> = Vec::new();
    for start in 0..n {
        if seen[start] || sources[start] == start {
            seen[start] = true;
            continue;
        }
        // Walk destination -> its source, collecting the cycle.
        let mut cycle = Vec::new();
        let mut at = start;
        while !seen[at] {
            seen[at] = true;
            cycle.push(at);
            at = sources[at];
            // A source outside `sources`' own index range cannot close a cycle;
            // that is a permutation of a larger set than we were given.
            if at >= n {
                return Err(out_of_class(
                    "argument permutation references a slot outside the argument \
                     list; out of class",
                ));
            }
        }
        cycles.push(cycle);
    }

    if cycles.is_empty() {
        return Ok((Vec::new(), Vec::new())); // passthrough
    }
    if cycles.len() > 1 {
        return Err(out_of_class(
            "argument permutation has two or more disjoint cycles: c2 hoists both \
             saves (r11 then r10) and then has several clobber-free orders to pick \
             from, which one capture does not determine; out of class",
        ));
    }

    // **Only up to a three-element cycle.** Past three, c2 abandons the minimal
    // single-temp walk below: it hoists a *second* save into r10 and writes the
    // destinations in a different order. `return a4(c,d,b,a)` is
    //
    //   mr r11,r5 ; mr r10,r6 ; mr r6,r3 ; mr r5,r4 ; mr r4,r10 ; mr r3,r11
    //
    // — six moves and two temps against this function's five and one. Measured
    // over complete grids, not sampled: all 24 four-argument permutations and all
    // 84 single cycles of length 2–5 in a five-argument call give 0 mismatches at
    // lengths 2 and 3, 10 of 30 at length 4 and 16 of 24 at length 5. It was a
    // **live wrong-bytes emit on mainline** and no fixture reached it, because
    // `il_call_perm.cpp` and `il_call_multi.cpp` between them hold no cycle longer
    // than three. The order c2 picks past three is not characterized, so the
    // boundary is the measured edge rather than a fit.
    //
    // The primary gate is `c2_il`'s (`call-arg-long-cycle`), so the census and the
    // emitter agree; this is the backstop.
    if cycles[0].len() > 3 {
        return Err(out_of_class(
            "argument permutation has a cycle longer than three: past three c2 \
             hoists a second save into r10 and reorders the writes, and which \
             order it picks is not characterized; out of class",
        ));
    }
    // One cycle. Its lowest destination is filled from the temp, last.
    let cycle = &cycles[0];
    let lowest = *cycle.iter().min().expect("non-empty cycle");
    let reg = |slot: usize| ARG_REGS[slot];
    let mut t = Vec::new();
    let mut writes = vec![SCRATCH_REG];
    t.extend_from_slice(&encode_mr(SCRATCH_REG, reg(sources[lowest])));
    // Walk backwards from `lowest`: each step writes a destination whose old
    // value has already been consumed. This is the unique clobber-free order,
    // and it runs in whichever direction the cycle happens to go — which is why
    // `rot3` emits r4-then-r5 and `rot3b` emits r5-then-r4.
    let mut dst = sources[lowest];
    while dst != lowest {
        t.extend_from_slice(&encode_mr(reg(dst), reg(sources[dst])));
        writes.push(reg(dst));
        dst = sources[dst];
    }
    t.extend_from_slice(&encode_mr(reg(lowest), SCRATCH_REG));
    writes.push(reg(lowest));
    Ok((t, writes))
}

#[cfg(test)]
mod tests {
    // The single `mod tests` this was split out of opened with
    // `use super::*;`; the glob keeps that reach.
    #[allow(unused_imports)]
    use super::*;

    /// **The generated thunk body must be the reference payload, byte for
    /// byte.** `docs/OBJ_DYNINIT_SHAPE.md` §3.3/§7.2: this exact 0x18 is what
    /// `fixtures/cpp/il_dyninit_static.cpp`, `TomCryptLicense.cpp` and
    /// `ZlibLicense.cpp` all carry.
    ///
    /// The point of *generating* it rather than transcribing it is that the
    /// generator can be wrong in a way a constant cannot; this is the assertion
    /// that makes that trade safe.
    /// **W-MEMCPY — the park's call-site remainder with a literal merged into
    /// it is `?mmioGetInfo`'s own two words**, and they are asserted against
    /// the bytes real `c2` emitted rather than against the rule that produced
    /// them (`frontier_bytes::C2_MMIOGETINFO_TEXT` `0x34..0x3c`).
    ///
    /// `li r5,72 ; mr r4,r11` — the literal FIRST, because `r5 > r4`. The
    /// opposite order is what `R-LITLAST` predicts, and that rival scores 219
    /// of 403 over GRID-L.
    #[test]
    fn the_park_remainder_merges_a_literal_by_descending_destination() {
        use crate::codegen::frontier_bytes::C2_MMIOGETINFO_TEXT;
        let slots = vec![
            c2_il::SlotArg::Formal(1),
            c2_il::SlotArg::Formal(0),
            c2_il::SlotArg::Lit(72),
        ];
        // What `seq_entry_park` leaves at the call for `[1, 0, 2]` anchored at
        // formal 0: the single `mr r4,r11`.
        let (_entry, call, _first, _later) =
            seq_entry_park(&[1, 0, 2], 0).expect("the mmioGetInfo park is in class");
        assert_eq!(call, encode_mr(4, SCRATCH_REG).to_vec());
        let merged = park_call_with_literals(&call, &slots).expect("one move, one literal");
        assert_eq!(merged, C2_MMIOGETINFO_TEXT[0x34..0x3c].to_vec());
    }

    /// The other direction of the same walk: a literal BELOW the move goes
    /// after it. GRID-L separates this from "literals first" at 390 cells.
    #[test]
    fn a_literal_below_the_park_remainder_goes_after_it() {
        let slots = vec![
            c2_il::SlotArg::Lit(72),
            c2_il::SlotArg::Formal(2),
            c2_il::SlotArg::Formal(1),
        ];
        let call = encode_mr(5, SCRATCH_REG).to_vec();
        let merged = park_call_with_literals(&call, &slots).expect("one move, one literal");
        let mut want = call.clone();
        want.extend_from_slice(&encode_addi(3, 0, 72));
        assert_eq!(merged, want);
    }

    /// **Two moves at the call beside a literal is REFUSED, and refusing it is
    /// the finding.** `(g1, 2 moves, 1 literal)` is 72 of 76 for the
    /// descending rule and `(g1, 3 moves, 1 literal)` is 4 of 4 — non-monotone,
    /// so the boundary is the largest UNANIMOUS cell of the grid's own axes.
    #[test]
    fn two_park_moves_beside_a_literal_are_out_of_class() {
        let slots = vec![
            c2_il::SlotArg::Formal(2),
            c2_il::SlotArg::Formal(0),
            c2_il::SlotArg::Formal(1),
            c2_il::SlotArg::Lit(72),
        ];
        let mut call = encode_mr(5, 4).to_vec();
        call.extend_from_slice(&encode_mr(4, SCRATCH_REG));
        assert!(park_call_with_literals(&call, &slots).is_err());
    }

    #[test]
    fn the_dyninit_thunk_body_is_the_measured_payload() {
        let b = dyninit_thunk_text(0).expect("k = 0 is the measured cell");
        assert_eq!(
            b.text,
            vec![
                0x3d, 0x60, 0x00, 0x00, // lis  r11, 0      REFHI(string)
                0x3d, 0x40, 0x00, 0x00, // lis  r10, 0      REFHI(object)
                0x38, 0x8b, 0x00, 0x00, // addi r4, r11, 0  REFLO(string)
                0x38, 0x6a, 0x00, 0x00, // addi r3, r10, 0  REFLO(object)
                0x38, 0xa0, 0x00, 0x00, // li   r5, 0
                0x4b, 0xff, 0xff, 0xec, // b    -0x14       REL24(ctor)
            ]
        );
        // The relocation sites §3.2 lists, and the ordering fact behind them:
        // the HI block comes first as a block, then the LO block — the two are
        // NOT adjacent per symbol.
        assert_eq!((b.literal_hi, b.object_hi), (0x00, 0x04));
        assert_eq!((b.literal_lo, b.object_lo), (0x08, 0x0c));
        assert_eq!(b.branch, 0x14);
        assert!(b.literal_hi < b.object_hi && b.object_hi < b.literal_lo);
    }

    /// The branch word is `4b ff ff ec` and **not** the listing's `48 00 00 00`
    /// (`docs/OBJ_DYNINIT_SHAPE.md` §6): MSVC encodes the displacement as
    /// −(the branch's own section offset).
    #[test]
    fn the_tail_branch_displacement_is_negative_its_own_offset() {
        let b = dyninit_thunk_text(0).unwrap();
        assert_eq!(&b.text[0x14..], &encode_tail_branch(0x14));
        assert_eq!(&b.text[0x14..], &[0x4b, 0xff, 0xff, 0xec]);
    }

    /// The literal slot is emitted from the value that was decoded, not from a
    /// constant — and a value outside `li`'s immediate refuses rather than being
    /// truncated into a plausible-looking wrong instruction.
    #[test]
    fn the_literal_slot_tracks_the_decoded_value_and_refuses_outside_li() {
        assert_eq!(&dyninit_thunk_text(7).unwrap().text[0x10..0x14], &[0x38, 0xa0, 0x00, 0x07]);
        assert_eq!(
            &dyninit_thunk_text(-1).unwrap().text[0x10..0x14],
            &[0x38, 0xa0, 0xff, 0xff]
        );
        assert_eq!(&dyninit_thunk_text(0x7fff).unwrap().text[0x10..0x14], &[0x38, 0xa0, 0x7f, 0xff]);
        assert!(dyninit_thunk_text(0x8000).is_none());
        assert!(dyninit_thunk_text(-0x8001).is_none());
    }
    #[allow(unused_imports)]
    use crate::codegen::*;
    #[allow(unused_imports)]
    use c2_il::{IlFunction, IlOp};
    #[allow(unused_imports)]
    use crate::codegen::testutil::*;

    /// **W-CLEAR / board #275 — a permuted call BEHIND A GUARDED EARLY RETURN
    /// refuses, and the same permutation WITHOUT the guard still emits.**
    ///
    /// The pair is the whole point. `w-clear` measured 54 cells of
    /// guard-count × permutation × call-count against real `c2.dll` at the dc3
    /// workload's own flags and found **30 `Port=Mismatch`** — every cell with
    /// at least one early return and a non-identity permutation, and **zero**
    /// with no early return. So a blanket refusal on permutations would give up
    /// twelve cells that are byte-exact today, and a blanket acceptance is the
    /// wrong-bytes emit this test exists to prevent. Both halves are asserted.
    ///
    /// The identity permutation is asserted too, in the guarded arm: it produces
    /// no moves at all, so it must keep emitting — six of the 54 cells are that
    /// row and all six matched.
    #[test]
    fn a_permuted_call_behind_a_guarded_early_return_refuses_but_the_unguarded_one_emits() {
        let seq = |sources: Vec<usize>, guarded: bool| c2_il::CallSeq {
            early: if guarded {
                vec![c2_il::SeqEarlyReturn {
                    and_conds: Vec::new(),
                    cmp_param: 0,
                    rel: c2_il::Rel::Eq,
                    signed: false,
                    k: 0,
                    value: Some(5),
                }]
            } else {
                Vec::new()
            },
            calls: vec![c2_il::SeqCall {
                callee: "?g@@YAXPAX0@Z".into(),
                arg_ops: Vec::new(),
                arg_slots: Some(formal_slots(&sources)),
                link_args: None,
            }],
            tail: c2_il::SeqTail::Lit(0),
            saved: Vec::new(),
            guard: None,
            store_run: None,
        };
        let params = [9, 9, 9];

        // ---- the swap, UNGUARDED: `mr r11,r4 · mr r4,r3 · mr r3,r11` --------
        // The bytes real c2 emits for `f(p,q){ g(q,p); return 0; }`, which the
        // port already reproduces (`work/w-clear/probe/n1.cpp`, `match`).
        let (setups, _, _) = call_seq_parts(&params[..2], &seq(vec![1, 0], false), OptMode::O1)
            .expect("the unguarded permutation is in class and must stay in class");
        assert_eq!(
            setups[0],
            vec![
                0x7c, 0x8b, 0x23, 0x78, // mr r11,r4
                0x7c, 0x64, 0x1b, 0x78, // mr r4,r3
                0x7d, 0x63, 0x5b, 0x78, // mr r3,r11
            ],
            "the unguarded cycle break is the shipped, byte-exact lowering"
        );

        // ---- the SAME permutation behind one early return: THE PARK ---------
        //
        // **W-MMIO.** `w-clear` refused this whole shape after reading 30
        // `Port=Mismatch`; lane `w-mmio` measured it over 886 cells and it now
        // emits. The scrutinee is formal 0, which is the cycle minimum, so the
        // chain `r3<-r4 · r4<-r11` ascends and the split falls after the first
        // move. **These are `?mmioGetInfo`'s own two entry-block words** —
        // `frontier_bytes::C2_MMIOGETINFO_TEXT[0x0c..0x14]`.
        let (setups, _, park) = call_seq_parts(&params[..2], &seq(vec![1, 0], true), OptMode::O1)
            .expect("the park is in class: the scrutinee is in the cycle and unimodal");
        assert_eq!(
            park.entry,
            vec![
                0x7c, 0x6b, 0x1b, 0x78, // mr r11,r3   <- the PARK
                0x7c, 0x83, 0x23, 0x78, // mr r3,r4    <- the ascending prefix
            ],
            "the entry block is the park plus the ascending prefix"
        );
        assert_eq!(
            setups[0],
            encode_mr(4, 11).to_vec(),
            "only the cycle-closing move is left at the call"
        );
        // And it is a DIFFERENT cycle break from the unguarded one above: that
        // saves r4, this saves r3.
        assert_ne!(park.entry[..4], setups[0][..4]);
        // The guard reads the PARKED register, because its home was overwritten.
        assert_eq!(park.reg_of(0, ARG_REGS[0], 0), SCRATCH_REG);
        // …and a formal whose home the entry block did NOT touch is read at
        // home by the first guard and at its new location by a later one.
        assert_eq!(park.reg_of(1, ARG_REGS[1], 1), ARG_REGS[0]);

        // ---- three-argument cycles, both rotations ---------------------------
        //
        // Scrutinee 0 is the cycle minimum in all four, so all four are in
        // class; the split differs between them and that is the descent clause.
        for (sources, entry_words, call_words) in [
            (vec![1, 2, 0], 3, 1),  // dests r3,r4,r5 — ascending, hoist two
            (vec![2, 0, 1], 2, 2),  // dests r3,r5,r4 — the DESCENT at r4
            (vec![2, 1, 0], 2, 1),
            // `[0,2,1]` is NOT here: it moves formals 1 and 2 only, so
            // scrutinee 0 sits outside its cycle and it is refused below.
        ] {
            let (setups, _, park) = call_seq_parts(&params, &seq(sources.clone(), true), OptMode::O1)
                .unwrap_or_else(|e| panic!("{sources:?} behind a guard is in class: {e:?}"));
            assert_eq!(park.entry.len(), entry_words * 4, "{sources:?} entry block");
            assert_eq!(setups[0].len(), call_words * 4, "{sources:?} call site");
            assert!(
                call_seq_parts(&params, &seq(sources.clone(), false), OptMode::O1).is_ok(),
                "{sources:?} with NO guard must still emit — it is byte-exact today"
            );
        }

        // ---- and what stays REFUSED, by name --------------------------------
        //
        // A scrutinee outside the cycle: c2 then anchors at a later guard or at
        // the cycle minimum, and that clause was re-fitted by every population
        // that measured it (grid 2 refuted grid 1's, grid 3 refuted grid 2's).
        // `[0,2,1]` moves formals 1 and 2 only, so scrutinee 0 is outside it.
        let e = seq_entry_park(&[0, 2, 1], 0)
            .expect_err("a scrutinee outside the cycle is out of class");
        assert!(format!("{e:?}").contains("not in the permutation"), "{e:?}");
        // A chain that dips and rises again has no ascending|descending layout.
        let e = seq_entry_park(&[2, 0, 1], 1)
            .expect_err("a non-unimodal chain is out of class");
        assert!(format!("{e:?}").contains("unimodal"), "{e:?}");
        // …and the SAME permutation anchored at the minimum is in class, so the
        // refusal above is the chain's shape and not the permutation's.
        assert!(seq_entry_park(&[2, 0, 1], 0).is_ok());

        // ---- and the IDENTITY keeps emitting behind the guard ---------------
        // No moves, nothing to park, and all six measured cells matched.
        let (setups, _, _) = call_seq_parts(&params, &seq(vec![0, 1, 2], true), OptMode::O1)
            .expect("the identity permutation behind a guard is byte-exact today");
        assert!(setups[0].is_empty(), "the identity permutation emits no moves");
    }

    /// **BOARD #1414 IS REFUTED HERE, in the smallest cell that does it.**
    ///
    /// #1414 publishes the park's cycle break as *"saving the LOWEST slot's
    /// home into r11"*. `g(a2,a0,a1)` guarded on **`a2`** breaks that: c2 parks
    /// **r5**, the guard's own scrutinee, and leaves the whole chain at the
    /// call. Lane `w-clear`'s five cells could not see it because in all five
    /// the guard's formal and the cycle minimum were the same register `r3`;
    /// scored over the 832 in-class cells of `work/w-mmio/probe{,2,3}/`, the
    /// minimum rule gets **394** and this one gets **832**.
    ///
    /// Both anchors are asserted from the SAME permutation, so what this test
    /// pins is the guard's effect and not the permutation's.
    #[test]
    fn the_park_anchors_at_the_guards_scrutinee_and_not_at_the_cycle_minimum() {
        // `g(a2,a0,a1)`: slot 0 <- a2, slot 1 <- a0, slot 2 <- a1.
        let sources = [2usize, 0, 1];

        // Guarded on a0 — which IS the cycle minimum, so the two rules agree.
        let (entry, call, ..) = seq_entry_park(&sources, 0).expect("in class");
        assert_eq!(entry, [encode_mr(11, 3), encode_mr(3, 5)].concat());
        assert_eq!(call, [encode_mr(5, 4), encode_mr(4, 11)].concat());

        // Guarded on a2 — the cell #1414 gets wrong. The park is r5, NOT r3.
        let (entry, call, ..) = seq_entry_park(&sources, 2).expect("in class");
        assert_eq!(entry, encode_mr(11, 5).to_vec(), "the anchor is the SCRUTINEE");
        assert_ne!(entry, encode_mr(11, 3).to_vec(), "#1414 predicts r3 here");
        assert_eq!(
            call,
            [encode_mr(5, 4), encode_mr(4, 3), encode_mr(3, 11)].concat(),
            "nothing is hoisted past the park: the chain descends throughout"
        );
    }

    /// **The split: the ENTRY block ascends, the CALL SITE descends — so a
    /// chain that dips and rises again has no layout at all.**
    ///
    /// The two rotations of one three-cycle put the split in different places,
    /// which is the descent clause; and the same permutation anchored where its
    /// chain is not unimodal is refused rather than guessed. #1414 had one cell
    /// for the first half and none for the second.
    #[test]
    fn the_entry_block_ascends_the_call_site_descends_and_a_dip_has_no_layout() {
        // `g(a1,a2,a0)` — the chain writes r3, r4, r5: ascending, hoist two.
        let (entry, call, ..) = seq_entry_park(&[1, 2, 0], 0).expect("in class");
        assert_eq!(entry.len(), 12, "park + two hoisted moves");
        assert_eq!(call, encode_mr(5, 11).to_vec(), "only the closer is left");

        // `g(a2,a0,a1)` — the chain writes r3, r5, r4: the descent at r4 moves
        // the split one instruction earlier.
        let (entry, call, ..) = seq_entry_park(&[2, 0, 1], 0).expect("in class");
        assert_eq!(entry.len(), 8, "park + ONE hoisted move");
        assert_eq!(call.len(), 8);

        // …and the call site is strictly DESCENDING by destination in both,
        // which is `moves_descending`'s rule arriving from the other side.
        for k in (4..call.len()).step_by(4) {
            let prev = u32::from_be_bytes(call[k - 4..k].try_into().unwrap());
            let cur = u32::from_be_bytes(call[k..k + 4].try_into().unwrap());
            assert!((prev >> 16) & 31 > (cur >> 16) & 31, "descending");
        }

        // The same permutation anchored at a slot whose chain dips and rises.
        let e = seq_entry_park(&[2, 0, 1], 1).expect_err("no layout");
        assert!(format!("{e:?}").contains("unimodal"), "{e:?}");
    }

    /// **The guards' compare register needs TWO maps, and the pair that shows
    /// it differs by nothing a cost model would notice.**
    ///
    /// `gtgt_n4_p0312_g3` and `g2ord_n4_p0312_g23` have byte-identical entry
    /// blocks — `mr r11,r6` and nothing else — and test the same formal, and
    /// real `c2` compares **r6** in the first and **r11** in the second. Both
    /// are correct code. A single map gets 327 of 1,654 measured guards wrong;
    /// this one gets 0.
    #[test]
    fn the_first_guard_reads_a_live_home_and_a_later_guard_reads_where_it_went() {
        // `g(a0,a3,a1,a2)` anchored on a3: the entry block is the park alone.
        let (entry, _, first, later) = seq_entry_park(&[0, 3, 1, 2], 3).expect("in class");
        assert_eq!(entry, encode_mr(11, 6).to_vec());
        let park = SeqPark { entry, first, later };
        assert_eq!(park.reg_of(3, ARG_REGS[3], 0), ARG_REGS[3], "guard 0 reads r6");
        assert_eq!(park.reg_of(3, ARG_REGS[3], 1), SCRATCH_REG, "a later guard reads r11");

        // …and where the entry block DID overwrite the home, both maps agree,
        // because reading the home would be wrong code rather than a choice.
        let (entry, _, first, later) = seq_entry_park(&[1, 0], 0).expect("in class");
        let park = SeqPark { entry, first, later };
        assert_eq!(park.reg_of(0, ARG_REGS[0], 0), SCRATCH_REG);
        assert_eq!(park.reg_of(0, ARG_REGS[0], 1), SCRATCH_REG);
        // The formal that MOVED is read at its new home by a later guard even
        // though its own home register still holds a live copy — this is
        // `?mmioGetInfo`'s second guard, `cmplwi cr6,r3,0` at 0x24.
        assert_eq!(park.reg_of(1, ARG_REGS[1], 1), ARG_REGS[0]);
    }

    /// **The park this emitter builds is `?mmioGetInfo`'s own, byte for byte.**
    ///
    /// `frontier_bytes::C2_MMIOGETINFO_TEXT` is a transcription of what real
    /// `c2` emitted for the frontier's head (#502); these are the three words
    /// of it that board #275 is about, and they are now *derived* rather than
    /// read off. The function still does not emit — its `li r5,72` is
    /// `callseq-multiarg-lit` — so this is the seam between the two, asserted.
    #[test]
    fn the_park_reproduces_mmiogetinfos_own_entry_block_and_call_remainder() {
        let (entry, call, ..) = seq_entry_park(&[1, 0], 0).expect("in class");
        let t = crate::codegen::frontier_bytes::C2_MMIOGETINFO_TEXT;
        assert_eq!(entry, t[0x0c..0x14].to_vec(), "the entry block, 0x0c..0x14");
        assert_eq!(call, t[0x38..0x3c].to_vec(), "the remainder at the call, 0x38");
        // And it is NOT the unguarded break, which saves r4 where this saves r3.
        assert_ne!(&entry[..4], &encode_mr(11, 4)[..]);
    }

    /// **WCO — the chain-result designator, and the one place its two forms
    /// disagree.**
    ///
    /// `p->a()->b()->m` is `lwz r3,off(r3)` and `&p->a()->b()->m` is
    /// `addi r3,r3,off`. At `off == 0` the add folds to nothing and the load
    /// does NOT — measured, `c_off0` (40 B) against `c_addr0` (36 B) in
    /// `work/WCO/probe/p1.cpp` at `/O1 /GS- /c`. Both directions are pinned,
    /// because the tempting simplification is to give `CallLoad` the same
    /// `add_k: 0` fold its sibling has.
    #[test]
    fn the_chain_tail_load_does_not_fold_at_offset_zero_but_the_add_does() {
        let seq = |tail| c2_il::CallSeq {
            early: Vec::new(),
            calls: vec![
                c2_il::SeqCall { callee: "?a@@YAPAUM@@XZ".into(), arg_ops: vec![IlOp::Load(9)], arg_slots: None, link_args: None },
                c2_il::SeqCall { callee: "?b@@YAPAUM@@XZ".into(), arg_ops: Vec::new(), arg_slots: None, link_args: Some(Vec::new()) },
            ],
            tail,
            saved: Vec::new(),
            guard: None,
            store_run: None,
        };
        let tail_of = |t| {
            call_seq_parts(&[9], &seq(t), OptMode::O1).expect("in class").1
        };
        // `lwz r3,4(r3)` / `lwz r3,0(r3)` — the load is emitted at both offsets.
        assert_eq!(tail_of(c2_il::SeqTail::CallLoad { off: 4 }), vec![0x80, 0x63, 0x00, 0x04]);
        assert_eq!(tail_of(c2_il::SeqTail::CallLoad { off: 0 }), vec![0x80, 0x63, 0x00, 0x00]);
        // `addi r3,r3,4` — and NOTHING at 0.
        assert_eq!(tail_of(c2_il::SeqTail::CallValue { add_k: 4 }), vec![0x38, 0x63, 0x00, 0x04]);
        assert_eq!(tail_of(c2_il::SeqTail::CallValue { add_k: 0 }), Vec::<u8>::new());
        // **W-FLTRET — the FP value tail emits nothing, and that is a
        // MEASUREMENT and not an omission.** c2's own `/FAsc` listing for
        // `float f(O*o){ o->Poll(); return o->Level(); }` ends `bl ?Level ; addi
        // r1,r1,96 ; …` — no `fmr`, no `frsp`, the callee's `f1` IS the result
        // (`work/w-fltret/probe/v1.cod`). The difference between this tail and
        // the integer one above it is one obj symbol, `_fltused`, and no
        // instruction at all.
        assert_eq!(tail_of(c2_il::SeqTail::CallValueFp), Vec::<u8>::new());
        // A negative displacement is representable and is not a fold either.
        assert_eq!(tail_of(c2_il::SeqTail::CallLoad { off: -4 }), vec![0x80, 0x63, 0xFF, 0xFC]);
        // Past the signed-16-bit displacement it refuses rather than truncating.
        // The IL parser gates this; the second lock is here.
        assert!(call_seq_parts(&[9], &seq(c2_il::SeqTail::CallLoad { off: 0x8000 }), OptMode::O1).is_err());
    }

    /// **WFL — the same designator step in the other register file.**
    ///
    /// The words are read off the reference obj (`work/WFL/probe/p1.cpp`,
    /// `/O1 /GS- /c`) and the destination is `f1`, not r3 — which is the whole
    /// reason this is a variant and not a width flag on `CallLoad`. It does not
    /// fold at 0 either, for the same reason its integer sibling does not.
    #[test]
    fn the_chain_tail_fp_load_is_an_lfs_or_an_lfd_into_f1() {
        let seq = |tail| c2_il::CallSeq {
            early: Vec::new(),
            calls: vec![
                c2_il::SeqCall { callee: "?a@@YAPAUM@@XZ".into(), arg_ops: vec![IlOp::Load(9)], arg_slots: None, link_args: None },
                c2_il::SeqCall { callee: "?b@@YAPAUM@@XZ".into(), arg_ops: Vec::new(), arg_slots: None, link_args: Some(Vec::new()) },
            ],
            tail,
            saved: Vec::new(),
            guard: None,
            store_run: None,
        };
        let tail_of = |t| call_seq_parts(&[9], &seq(t), OptMode::O1).expect("in class").1;
        // `lfs f1,4(r3)` = c0230004 and `lfd f1,16(r3)` = c8230010 — the two
        // cells `c_f` and `c_d` compile to, one primary opcode apart.
        assert_eq!(
            tail_of(c2_il::SeqTail::CallLoadFp { off: 4, double: false }),
            vec![0xC0, 0x23, 0x00, 0x04]
        );
        assert_eq!(
            tail_of(c2_il::SeqTail::CallLoadFp { off: 16, double: true }),
            vec![0xC8, 0x23, 0x00, 0x10]
        );
        // No fold at 0: `*(r3 + 0)` is a memory read that has to happen.
        assert_eq!(
            tail_of(c2_il::SeqTail::CallLoadFp { off: 0, double: false }),
            vec![0xC0, 0x23, 0x00, 0x00]
        );
        // `lfd` is D-form, NOT the DS-form the integer `ld` is, so an 8-byte
        // load at a displacement that is not a multiple of 4 encodes fine and
        // needs none of the alignment gate the load leaf carries.
        assert_eq!(
            tail_of(c2_il::SeqTail::CallLoadFp { off: 6, double: true }),
            vec![0xC8, 0x23, 0x00, 0x06]
        );
        // Past the signed-16-bit displacement it refuses rather than truncating.
        assert!(
            call_seq_parts(&[9], &seq(c2_il::SeqTail::CallLoadFp { off: 0x8000, double: false }), OptMode::O1)
                .is_err()
        );
    }

    #[test]
    fn encode_tail_branch_stores_negative_self_offset() {
        // Tail-call `b` displacement = −(own .text offset): offset 0 → 0x48000000,
        // offset 8 → 0x4BFFFFF8 (the REL24 reloc patches the target).
        assert_eq!(encode_tail_branch(0), [0x48, 0x00, 0x00, 0x00]);
        assert_eq!(encode_tail_branch(8), [0x4B, 0xFF, 0xFF, 0xF8]);
    }

    /// **W10 — the framed × branching cell, against the reference obj's own
    /// bytes.**
    ///
    /// `void g1(int a){ if (a != 0) v0(); v1(); }`, 44 B, transcribed from the
    /// obj `cl.exe` 16.00.11886.00 emits for `fixtures/cpp/w10_guarded_seq.cpp`
    /// — **identical at `/O1` and at `/Ox`**, which is the property the `else`
    /// form turned out not to have.
    ///
    /// This is the assertion with a byte behind it. The two below it check the
    /// displacement's *dependence* on the setup, which this one cannot: `g1`'s
    /// guarded arm has an empty setup, so a `bc` computed from the wrong length
    /// would still come out `+8` here.
    #[test]
    fn a_guarded_call_sequence_matches_the_reference_bytes() {
        let guard = seq_guard_emit(&c2_il::SeqGuard {
            cmp_param: 0,
            rel: c2_il::Rel::Ne,
            signed: true,
            k: 0,
        })
        .expect("in class");
        let body = call_seq_text(
            &[Vec::new(), Vec::new()],
            &[],
            0,
            FrameLayout::default(),
            &[],
            Some(&guard),
            &[],
            OptMode::Ox,
        )
        .expect("in class");
        #[rustfmt::skip]
        let want: Vec<u8> = vec![
            0x7d, 0x88, 0x02, 0xa6, // mflr  r12
            0x91, 0x81, 0xff, 0xf8, // stw   r12,-8(r1)
            0x94, 0x21, 0xff, 0xa0, // stwu  r1,-96(r1)
            0x2f, 0x03, 0x00, 0x00, // cmpwi cr6,r3,0    <- signed: `a` is an int
            0x41, 0x9a, 0x00, 0x08, // bt    26,+8       <- NEGATION of `!=`
            0x4b, 0xff, 0xff, 0xed, // bl    ?v0         REL24 @0x14
            0x4b, 0xff, 0xff, 0xe9, // bl    ?v1         REL24 @0x18
            0x38, 0x21, 0x00, 0x60, // addi  r1,r1,96
            0x81, 0x81, 0xff, 0xf8, // lwz   r12,-8(r1)
            0x7d, 0x88, 0x03, 0xa6, // mtlr  r12
            0x4e, 0x80, 0x00, 0x20, // blr
        ];
        assert_eq!(body.text, want);
        assert_eq!(body.text.len(), 44);
        assert_eq!(body.bl_offsets, vec![0x14, 0x18]);
        assert_eq!(body.prolog_len, 12);
    }

    /// **The `bc`'s displacement is measured over the guarded call's SETUP, and
    /// the setup stays INSIDE the guarded block.**
    ///
    /// `void g2(int a,int b){ if (a != 0) a1(b); v1(); }` is 48 B and the
    /// reference emits `cmpwi cr6,r3,0 ; bt 26,+12 ; mr r3,r4 ; bl ?a1 ; bl ?v1`
    /// — the `mr` **after** the branch, and the branch four bytes longer than
    /// `g1`'s to step over it. An emitter that hoisted the setup above the
    /// compare would produce the same 48 bytes in a different order and the
    /// same `+8` displacement `g1` has, so this cell is what separates the two
    /// readings. (Hoisting is what c2 does the moment the arm needs a scratch
    /// park — `work/w-cross/p/probe2.cpp::s4` — which is why an arm here takes
    /// at most one argument.)
    #[test]
    fn the_guarded_branch_steps_over_the_arms_setup() {
        let guard = seq_guard_emit(&c2_il::SeqGuard {
            cmp_param: 0,
            rel: c2_il::Rel::Ne,
            signed: true,
            k: 0,
        })
        .unwrap();
        // `mr r3,r4` — the guarded call's whole setup.
        let setup = encode_mr(3, 4).to_vec();
        let body =
            call_seq_text(
                &[setup, Vec::new()],
                &[],
                0,
                FrameLayout::default(),
                &[],
                Some(&guard),
                &[],
                OptMode::Ox,
            )
                .expect("in class");
        assert_eq!(&body.text[12..16], &[0x2f, 0x03, 0x00, 0x00], "cmpwi cr6,r3,0");
        assert_eq!(&body.text[16..20], &[0x41, 0x9a, 0x00, 0x0c], "bt 26,+12");
        assert_eq!(&body.text[20..24], &encode_mr(3, 4), "the setup, AFTER the branch");
        assert_eq!(body.bl_offsets, vec![0x18, 0x1c]);
        assert_eq!(body.text.len(), 48);
    }

    /// The compare reads the scrutinee's **home** argument register, and its
    /// signedness picks the instruction. `w10_guarded_seq.cpp::g5` compares the
    /// SECOND formal, so the word is `2f04……` and not `2f03……` — the cell that
    /// separates "the compare reads the scrutinee" from "the compare reads r3".
    #[test]
    fn the_guard_compares_the_scrutinees_home_register() {
        let mk = |cmp_param, signed, k| {
            seq_guard_emit(&c2_il::SeqGuard { cmp_param, rel: c2_il::Rel::Ne, signed, k })
                .map(|g| {
                    call_seq_text(
                        &[Vec::new(), Vec::new()],
                        &[],
                        0,
                        FrameLayout::default(),
                        &[],
                        Some(&g),
                        &[],
                        OptMode::Ox,
                    )
                    .unwrap()
                    .text[12..16]
                        .to_vec()
                })
        };
        assert_eq!(mk(0, true, 0).unwrap(), vec![0x2f, 0x03, 0x00, 0x00]); // cmpwi  cr6,r3,0
        assert_eq!(mk(1, true, 0).unwrap(), vec![0x2f, 0x04, 0x00, 0x00]); // cmpwi  cr6,r4,0
        assert_eq!(mk(2, false, 7).unwrap(), vec![0x2b, 0x05, 0x00, 0x07]); // cmplwi cr6,r5,7
        // Past the immediate field it refuses rather than truncating.
        assert!(mk(0, true, 0x8000).is_err());
        assert!(mk(0, false, 0x1_0000).is_err());
    }

    /// **A guard with nothing after it must not reach the emitter.** The IL
    /// parser refuses it (`callseq-guard-no-trailing-call`) because that shape
    /// is fold band 2 plus a tail call, not a frame — 16 B and no `.pdata`. This
    /// is the backstop, and it is a refusal rather than a panic because an
    /// out-of-range index would otherwise be a silent wrong-displacement emit.
    #[test]
    fn a_guard_with_no_call_after_it_refuses_in_the_emitter_too() {
        let guard = seq_guard_emit(&c2_il::SeqGuard {
            cmp_param: 0,
            rel: c2_il::Rel::Ne,
            signed: true,
            k: 0,
        })
        .unwrap();
        assert!(
            call_seq_text(
                &[Vec::new()],
                &[],
                0,
                FrameLayout::default(),
                &[],
                Some(&guard),
                &[],
                OptMode::Ox,
            )
            .is_err()
        );
        // …and the same sequence with no guard is the shipped Class A body.
        assert!(
            call_seq_text(
                &[Vec::new(), Vec::new()],
                &[],
                0,
                FrameLayout::default(),
                &[],
                None,
                &[],
                OptMode::Ox,
            )
            .is_ok()
        );
    }

    #[test]
    fn encode_call_branch_sets_link_bit() {
        // `bl` at offset 0xC → disp −0xC, LK=1 → 0x4BFFFFF5 (reference `bl g`).
        assert_eq!(encode_call_branch(0x0C), [0x4B, 0xFF, 0xFF, 0xF5]);
    }

    #[test]
    fn framed_call_text_matches_reference_body() {
        let plain = FrameLayout::default();
        // `int f(int a){ return g(a) + 1; }` — the verified 0x24-byte body. `a` is
        // already in r3, so the argument setup is empty.
        let b = framed_call_text(&[], 1, 0, plain).unwrap();
        assert_eq!(
            b.text,
            vec![
                0x7D, 0x88, 0x02, 0xA6, // mflr r12
                0x91, 0x81, 0xFF, 0xF8, // stw  r12,-8(r1)
                0x94, 0x21, 0xFF, 0xA0, // stwu r1,-96(r1)
                0x4B, 0xFF, 0xFF, 0xF5, // bl   g (REL24 @ 0xC)
                0x38, 0x63, 0x00, 0x01, // addi r3,r3,1
                0x38, 0x21, 0x00, 0x60, // addi r1,r1,96
                0x81, 0x81, 0xFF, 0xF8, // lwz  r12,-8(r1)
                0x7D, 0x88, 0x03, 0xA6, // mtlr r12
                0x4E, 0x80, 0x00, 0x20, // blr
            ]
        );
        assert_eq!((b.bl_offset, b.prolog_len), (0x0C, 0x0C));
        // `+ 2` differs only in the addi immediate.
        assert_eq!(framed_call_text(&[], 2, 0, plain).unwrap().text[19], 0x02);
        // Placed at 0x08 in a packed `.text` (a leaf ahead of it), the `bl` is at
        // 0x14 and its displacement follows: `4BFFFFED`, not `4BFFFFF5`. Every
        // other byte of the body is unchanged.
        let at8 = framed_call_text(&[], 1, 0x08, plain).unwrap();
        assert_eq!(&at8.text[12..16], &[0x4B, 0xFF, 0xFF, 0xED]);
        assert_eq!(&at8.text[..12], &b.text[..12]);
        assert_eq!(&at8.text[16..], &b.text[16..]);
        assert_eq!(at8.bl_offset, 0x14);
    }

    /// The argument-setup word, against the reference obj it was missing from.
    ///
    /// `int f(int a,int b){ return g(b) + 1; }` at `/Ox /GS- /c`, `.text` = 40
    /// bytes (10 words), `.pdata` `40000a03` (FuncLen 10, PrologLen 3), REL24 at
    /// 0x10. The port emitted the 9-word body with no `or` and every downstream
    /// field to match — a live wrong-bytes emit on mainline, and the reason
    /// [`Selected::Framed`] carries a setup at all.
    #[test]
    fn framed_call_moves_a_non_first_formal_into_r3() {
        let setup = encode_mr(RET_REG, 4);
        let b = framed_call_text(&setup, 1, 0, FrameLayout::default()).unwrap();
        assert_eq!(b.text.len(), 0x28);
        assert_eq!(&b.text[12..16], &[0x7C, 0x83, 0x23, 0x78]); // or r3,r4,r4
        assert_eq!(&b.text[16..20], &[0x4B, 0xFF, 0xFF, 0xF1]); // bl, disp −0x10
        assert_eq!((b.bl_offset, b.prolog_len), (0x10, 0x0C));
        // …and from r5, the three-formal case (`or r3,r5,r5` = 7ca32b78).
        let b5 = framed_call_text(&encode_mr(RET_REG, 5), 1, 0, FrameLayout::default()).unwrap();
        assert_eq!(&b5.text[12..16], &[0x7C, 0xA3, 0x2B, 0x78]);
    }

    #[test]
    fn int_tail_call_passthrough_is_bare_branch() {
        // `return g(a)` — a is already in r3, so no arg setup: a bare `b g` at
        // offset 0 (`48000000`), reloc site 0 — byte-identical to the void
        // tail call. Verified against the live obj (.text=48000000, REL24 @0x0).
        let f = func_with(vec![0xE309], vec![IlOp::Load(0xE309)]);
        let (text, reloc) = int_tail_call_text(&f, 0, OptMode::Ox).unwrap();
        assert_eq!(text, vec![0x48, 0x00, 0x00, 0x00]);
        assert_eq!(reloc, 0);
    }

    #[test]
    fn int_tail_call_arg_setup_prepends_addi() {
        // `return g(a + 1)` — the argument a+1 computed into r3 (`addi r3,r3,1`),
        // then `b g` at offset 4 (`4bfffffc`, disp −4), reloc site 0x4. Verified
        // against the live obj (.text=386300014bfffffc, REL24 @0x4).
        let f = func_with(
            vec![0xE309],
            vec![IlOp::Load(0xE309), IlOp::Lit(1), IlOp::Add],
        );
        let (text, reloc) = int_tail_call_text(&f, 0, OptMode::Ox).unwrap();
        assert_eq!(
            text,
            vec![
                0x38, 0x63, 0x00, 0x01, // addi r3,r3,1
                0x4B, 0xFF, 0xFF, 0xFC, // b g (disp −4)
            ]
        );
        assert_eq!(reloc, 4);
    }

    // ---- multi-argument tail-call permutations ------------------------------

    #[test]
    fn argument_permutations_match_the_reference() {
        // Captured from `int g3(int,int,int)` call sites; `sources[i]` is the
        // parameter index that argument slot i wants.
        // Passthrough: the parameters are already placed, so no moves at all.
        assert!(permute_args_text(&formal_slots(&[0, 1])).unwrap().is_empty());
        assert!(permute_args_text(&formal_slots(&[0, 1, 2])).unwrap().is_empty());

        // g3(b,a,c) — swap r3/r4, r5 untouched.
        assert_eq!(
            permute_args_text(&formal_slots(&[1, 0, 2])).unwrap(),
            vec![
                0x7C, 0x8B, 0x23, 0x78, // mr r11,r4
                0x7C, 0x64, 0x1B, 0x78, // mr r4,r3
                0x7D, 0x63, 0x5B, 0x78, // mr r3,r11
            ]
        );
        // g3(a,c,b) — swap r4/r5. The temp still takes the source of the cycle's
        // LOWEST destination (r4's), not r3's, which is not in the cycle at all.
        assert_eq!(
            permute_args_text(&formal_slots(&[0, 2, 1])).unwrap(),
            vec![
                0x7C, 0xAB, 0x2B, 0x78, // mr r11,r5
                0x7C, 0x85, 0x23, 0x78, // mr r5,r4
                0x7D, 0x64, 0x5B, 0x78, // mr r4,r11
            ]
        );
        // g3(c,b,a) — swap r3/r5, r4 untouched.
        assert_eq!(
            permute_args_text(&formal_slots(&[2, 1, 0])).unwrap(),
            vec![
                0x7C, 0xAB, 0x2B, 0x78, // mr r11,r5
                0x7C, 0x65, 0x1B, 0x78, // mr r5,r3
                0x7D, 0x63, 0x5B, 0x78, // mr r3,r11
            ]
        );
        // The two 3-cycles run in OPPOSITE directions, so their middle moves come
        // out in opposite orders. Fixing either order as "the rule" mis-emits the
        // other, which is why both are pinned.
        // g3(b,c,a): r3<-r4, r4<-r5, r5<-r3.
        assert_eq!(
            permute_args_text(&formal_slots(&[1, 2, 0])).unwrap(),
            vec![
                0x7C, 0x8B, 0x23, 0x78, // mr r11,r4
                0x7C, 0xA4, 0x2B, 0x78, // mr r4,r5
                0x7C, 0x65, 0x1B, 0x78, // mr r5,r3
                0x7D, 0x63, 0x5B, 0x78, // mr r3,r11
            ]
        );
        // g3(c,a,b): r3<-r5, r4<-r3, r5<-r4.
        assert_eq!(
            permute_args_text(&formal_slots(&[2, 0, 1])).unwrap(),
            vec![
                0x7C, 0xAB, 0x2B, 0x78, // mr r11,r5
                0x7C, 0x85, 0x23, 0x78, // mr r5,r4
                0x7C, 0x64, 0x1B, 0x78, // mr r4,r3
                0x7D, 0x63, 0x5B, 0x78, // mr r3,r11
            ]
        );
    }

    // ---- WCL: a chain link's marshalling runs the OTHER way ------------------

    /// The two orders, side by side, from the two halves of one probe TU.
    ///
    /// This is the whole rung in one assertion. Both bodies are Class B with the
    /// same two saved formals in the same two registers; they differ only in
    /// whether argument slot 0 belongs to the call or to the `bl` in front of it,
    /// and c2 emits the moves in opposite orders because of it. A single
    /// `moves_descending` for both — which is what "reuse the shipped rule, it is
    /// the same marshalling" would have produced — is byte-wrong on every chain
    /// link that carries two or more arguments.
    #[test]
    fn a_chain_links_moves_ascend_where_every_other_calls_descend() {
        let saved = |pi: usize| [Some(31u8), Some(30u8)].get(pi.wrapping_sub(1)).copied().flatten();
        // `int f(O* p,int j,int k){ return p->Next()->gia2(j,k); }`
        //   … bl ?Next ; mr r4,r31 ; mr r5,r30 ; bl ?gia2
        assert_eq!(
            link_setup_text(&[c2_il::SlotArg::Formal(1), c2_il::SlotArg::Formal(2)], saved).unwrap(),
            vec![
                0x7F, 0xE4, 0xFB, 0x78, // mr r4,r31
                0x7F, 0xC5, 0xF3, 0x78, // mr r5,r30
            ]
        );
        // …and the same slots wanting the other values: still ascending, so the
        // ORDER is not a function of which register the value is in.
        // `gia2(k,j)` → `mr r4,r30 ; mr r5,r31`.
        assert_eq!(
            link_setup_text(&[c2_il::SlotArg::Formal(2), c2_il::SlotArg::Formal(1)], saved).unwrap(),
            vec![
                0x7F, 0xC4, 0xF3, 0x78, // mr r4,r30
                0x7F, 0xE5, 0xFB, 0x78, // mr r5,r31
            ]
        );
        // The shipped rule for a list that starts at slot 0 — captured from
        // `void f(int a,int b,int c){ v1(a); g2(c,b); }` — goes the other way,
        // and it still does.
        assert_eq!(
            moves_descending(&[(RET_REG, 30), (4, 31)]),
            vec![
                0x7F, 0xE4, 0xFB, 0x78, // mr r4,r31
                0x7F, 0xC3, 0xF3, 0x78, // mr r3,r30
            ]
        );
    }

    /// A literal link argument is `li r<slot>,k` **in slot order**, interleaved
    /// with the moves rather than grouped before or after them. Captured:
    /// `int f(O* p,int j,int k){ return p->Next()->gia3(j,5,k); }` is
    /// `mr r4,r31 ; li r5,5 ; mr r6,r30`.
    #[test]
    fn a_chain_links_literals_interleave_in_slot_order() {
        let saved = |pi: usize| [Some(31u8), Some(30u8)].get(pi.wrapping_sub(1)).copied().flatten();
        assert_eq!(
            link_setup_text(
                &[
                    c2_il::SlotArg::Formal(1),
                    c2_il::SlotArg::Lit(5),
                    c2_il::SlotArg::Formal(2)
                ],
                saved
            )
            .unwrap(),
            vec![
                0x7F, 0xE4, 0xFB, 0x78, // mr r4,r31
                0x38, 0xA0, 0x00, 0x05, // li r5,5
                0x7F, 0xC6, 0xF3, 0x78, // mr r6,r30
            ]
        );
        // A value wanted TWICE is two ordinary moves — the sources are the
        // callee-saved file and the destinations the argument one, so there is no
        // cycle to break and no dead `mr r11`.
        // `p->Next()->gia2(j,j)` → `mr r4,r31 ; mr r5,r31`.
        assert_eq!(
            link_setup_text(&[c2_il::SlotArg::Formal(1), c2_il::SlotArg::Formal(1)], saved).unwrap(),
            vec![
                0x7F, 0xE4, 0xFB, 0x78, // mr r4,r31
                0x7F, 0xE5, 0xFB, 0x78, // mr r5,r31
            ]
        );
        // Slot 0 is the receiver, so seven explicit arguments fill r4..r10 and an
        // eighth would be stack-homed. The IL parser draws the same bound
        // (`mcall-chain-link-arg-overflow`); this is the backstop.
        let seven = vec![c2_il::SlotArg::Lit(1); 7];
        assert_eq!(link_setup_text(&seven, saved).unwrap().len(), 28);
        let eight = vec![c2_il::SlotArg::Lit(1); 8];
        assert!(link_setup_text(&eight, saved).is_err());
    }

    /// **WLA — a literal argument is one `li` and no move, and the order of two
    /// of them is DESCENDING**, which is the opposite of a chain link's.
    ///
    /// Both halves are read off `work/WLA/probe/p1.cpp` at `/O1 /GS- /c`. The
    /// single-literal rows agree with either order, so the two-literal one is
    /// the whole assertion: emitting them ascending is byte-wrong on every call
    /// that carries more than one constant.
    #[test]
    fn the_spliced_literal_matches_grid_l_at_every_arity_it_graded() {
        use c2_il::SlotArg::{Formal, Lit};
        // GRID-L's four families, transcribed from the reference objs. `mr rD,rS`
        // is `7c 00 03 78 | S<<21 | D<<16 | S<<11`; the constants below are the
        // words `scripts/gt_dump.py` printed.
        //
        // l1_n4 — `vsnprnc.cpp::vsprintf_s` itself: one move, `li` into the
        // register that move just read.
        assert_eq!(
            permute_args_text(&[Formal(0), Formal(1), Formal(2), Lit(0), Formal(3)]).unwrap(),
            vec![
                0x7C, 0xC7, 0x33, 0x78, // mr r7,r6
                0x38, 0xC0, 0x00, 0x00, // li r6,0
            ]
        );
        // l2_n4 — TWO moves, descending, then the `li`.
        assert_eq!(
            permute_args_text(&[Formal(0), Formal(1), Lit(0), Formal(2), Formal(3)]).unwrap(),
            vec![
                0x7C, 0xC7, 0x33, 0x78, // mr r7,r6
                0x7C, 0xA6, 0x2B, 0x78, // mr r6,r5
                0x38, 0xA0, 0x00, 0x00, // li r5,0
            ]
        );
        // l4_n3 — the literal FIRST: every formal shifts and the `li` lands in
        // r3 last. The far edge, and the cell where a rule that emitted the
        // literal first would pass three arguments wrong.
        assert_eq!(
            permute_args_text(&[Lit(0), Formal(0), Formal(1), Formal(2)]).unwrap(),
            vec![
                0x7C, 0xA6, 0x2B, 0x78, // mr r6,r5
                0x7C, 0x85, 0x23, 0x78, // mr r5,r4
                0x7C, 0x64, 0x1B, 0x78, // mr r4,r3
                0x38, 0x60, 0x00, 0x00, // li r3,0
            ]
        );
    }

    /// **The splice class and the shipped WLB two-slot cell are DISJOINT**, and
    /// the fixture `wvsnprnc_lit_insert_tail_neg.cpp::n2` is this in C++.
    ///
    /// `[Formal(2), Lit]` DROPS formals 0 and 1. No insertion produces that, so
    /// it must still reach `one_moved_formal_text` — which emits the hoist —
    /// rather than the splice path. Asserted on the bytes, because both paths
    /// would produce *something* and only one of them is right.
    #[test]
    fn a_dropped_formal_is_not_an_insertion() {
        use c2_il::SlotArg::{Formal, Lit};
        // `void f(int a,int b,int c){ g2(c, 7); }` — the `li` FIRST (descending),
        // because r4 does not hold the value the move needs.
        assert_eq!(
            permute_args_text(&[Formal(2), Lit(7)]).unwrap(),
            vec![
                0x38, 0x80, 0x00, 0x07, // li r4,7
                0x7C, 0xA3, 0x2B, 0x78, // mr r3,r5
            ]
        );
        // And the WLB hoist cell, where it does.
        assert_eq!(
            permute_args_text(&[Formal(1), Lit(7)]).unwrap(),
            vec![
                0x7C, 0x83, 0x23, 0x78, // mr r3,r4
                0x38, 0x80, 0x00, 0x07, // li r4,7
            ]
        );
    }

    #[test]
    fn a_literal_call_argument_is_one_li_and_two_of_them_descend() {
        use c2_il::SlotArg::{Formal, Lit};
        // `void f(int a,int b){ g3(a, b, 7); }` -> `li 5,7` (38a00007).
        assert_eq!(
            permute_args_text(&[Formal(0), Formal(1), Lit(7)]).unwrap(),
            vec![0x38, 0xA0, 0x00, 0x07]
        );
        // `void f(int a){ g2(a, 5); }` -> `li 4,5` (38800005).
        assert_eq!(
            permute_args_text(&[Formal(0), Lit(5)]).unwrap(),
            vec![0x38, 0x80, 0x00, 0x05]
        );
        // `void f(int a){ g3(a, 5, 6); }` -> `li 5,6` THEN `li 4,5`.
        assert_eq!(
            permute_args_text(&[Formal(0), Lit(5), Lit(6)]).unwrap(),
            vec![
                0x38, 0xA0, 0x00, 0x06, // li 5,6
                0x38, 0x80, 0x00, 0x05, // li 4,5
            ]
        );
        // Negative and zero immediates are the same encoder.
        assert_eq!(
            permute_args_text(&[Formal(0), Formal(1), Lit(-1)]).unwrap(),
            vec![0x38, 0xA0, 0xFF, 0xFF]
        );
        // **SUPERSEDED 2026-08-09 by lane `w-vsnprnc`.** This read
        //
        //     assert!(permute_args_text(&[Formal(0), Lit(7), Formal(1)]).is_err());
        //
        // with the reason *"`g3(a,7,b)` is `mr r5,r4 ; li r4,7` and the same
        // list over a real cycle is not characterized at all"* — a refusal that
        // named the right bytes and declined to emit them for want of a grid.
        // GRID-L is that grid: eighteen cells, and this list is its `l1_n2`,
        // graded `match` against the real `c2.dll`. The assertion is inverted
        // rather than deleted so the supersession sits where the claim did.
        assert_eq!(
            permute_args_text(&[Formal(0), Lit(7), Formal(1)]).unwrap(),
            vec![
                0x7C, 0x85, 0x23, 0x78, // mr r5,r4
                0x38, 0x80, 0x00, 0x07, // li r4,7
            ]
        );
        // **And the cell that is NOT an insertion still refuses.** `g3(b,a,7)`
        // reorders two formals; no splice produces it, and the WLB fence's own
        // measured counterexample `g3(c,a,7)` is refused for the same reason.
        assert!(permute_args_text(&[Formal(1), Formal(0), Lit(7)]).is_err());
        assert!(permute_args_text(&[Formal(2), Formal(0), Lit(7)]).is_err());
        // Past the eight register slots an argument is stack-homed.
        let nine = [
            Formal(0), Formal(1), Formal(2), Formal(3), Formal(4), Formal(5),
            Formal(6), Formal(7), Lit(1),
        ];
        assert!(permute_args_text(&nine).is_err());
        // …and a literal wider than the `addi` immediate. The IL parser draws
        // the same bound (`call-arg-lit-wide`); this is the backstop.
        assert!(permute_args_text(&[Formal(0), Lit(70000)]).is_err());
    }

    /// **WLB — the moved formal beside the literal, and the hoist that is not a
    /// sort order.**
    ///
    /// The two cells differ only in whether the `li`'s destination register is
    /// the one the move reads, and c2 emits them in opposite orders because of
    /// it. Both read off `work/WLA/probe/p2.cpp` at `/O1 /GS- /c`. A single
    /// fixed order — which is what "reuse `moves_descending`, it is the same
    /// marshalling" would have produced — is byte-wrong on one of them.
    #[test]
    fn a_moved_formal_hoists_past_the_literal_only_when_the_li_clobbers_it() {
        use c2_il::SlotArg::{Formal, Lit};
        // `void f(int a,int b){ g2(b, 7); }` — b is in r4, which is where the
        // `li` goes, so the move is HOISTED: mr r3,r4 ; li r4,7.
        assert_eq!(
            permute_args_text(&[Formal(1), Lit(7)]).unwrap(),
            vec![
                0x7C, 0x83, 0x23, 0x78, // mr r3,r4
                0x38, 0x80, 0x00, 0x07, // li r4,7
            ]
        );
        // `void f(int a,int b,int c){ g2(c, 7); }` — c is in r5, which the `li`
        // does not touch, so the default descending order stands:
        // li r4,7 ; mr r3,r5.
        assert_eq!(
            permute_args_text(&[Formal(2), Lit(7)]).unwrap(),
            vec![
                0x38, 0x80, 0x00, 0x07, // li r4,7
                0x7C, 0xA3, 0x2B, 0x78, // mr r3,r5
            ]
        );
        // The source walks to r10 and the order does not change again.
        assert_eq!(
            &permute_args_text(&[Formal(7), Lit(7)]).unwrap()[4..],
            &[0x7D, 0x43, 0x53, 0x78] // mr r3,r10
        );
        // THREE slots is a different lowering — `g3(c,a,7)` breaks through r11
        // and emits the `li` inside the walk — and all of it stays refused.
        assert!(permute_args_text(&[Formal(2), Formal(1), Lit(7)]).is_err());
        assert!(permute_args_text(&[Formal(2), Formal(0), Lit(7)]).is_err());
        // A literal in slot 0 beside a moved formal is a different register
        // pair and is not this cell.
        assert!(permute_args_text(&[Lit(7), Formal(2)]).is_err());
        // …and the immediate bound still applies.
        assert!(permute_args_text(&[Formal(1), Lit(70000)]).is_err());
    }

    /// **W-ADJUST — the address `addi` is SECOND, and this is the portable pin
    /// for it.**
    ///
    /// §9.10's standing rule, applied to the rule rather than to the file: WR1
    /// established an ordering rule for this emitter, added **no** `#[test]` for
    /// it, and the differential could only catch it where some fixture happened
    /// to arrange the discriminating case. It did not — WR1's fixture and its
    /// generated sweep both had at most ONE setup word beside the address, which
    /// is exactly the arity at which "address last" and "address second" are the
    /// same sequence. The first three-word case mis-emitted.
    ///
    /// Every expectation below is a word read off c2's own `.cod` listing
    /// (`work/wadjust/probe/q1.cpp`, `q3.cpp`, `/O1 /Oi /EHsc`), and the file
    /// that grades them against a real obj is `fixtures/cpp/wr1_sym_addr.cpp`.
    /// This test is what holds when the toolchain is absent.
    #[test]
    fn the_data_address_addi_is_emitted_second_not_last() {
        use c2_il::SlotArg::{Formal, Lit, SymAddr};
        let lis = [0x3Du8, 0x60, 0x00, 0x00]; // lis r11,0
        let words = |slots: &[c2_il::SlotArg]| -> Vec<[u8; 4]> {
            sym_slots_text(slots)
                .unwrap()
                .0
                .chunks_exact(4)
                .map(|c| [c[0], c[1], c[2], c[3]])
                .collect()
        };
        // An EMPTY walk: the `addi` follows the `lis` directly. `gso(&gI)`.
        assert_eq!(words(&[SymAddr]), vec![lis, [0x38, 0x6B, 0x00, 0x00]]);
        // `s->so(&gI)` — one in-place formal emits nothing, so the walk is still
        // empty and the address takes r4.
        assert_eq!(
            words(&[Formal(0), SymAddr]),
            vec![lis, [0x38, 0x8B, 0x00, 0x00]]
        );
        // ONE word: `gs2(&gI,7)` — the arity at which last and second agree, and
        // the only arity WR1 ever saw.
        assert_eq!(
            words(&[SymAddr, Lit(7)]),
            vec![lis, [0x38, 0x80, 0x00, 0x07], [0x38, 0x6B, 0x00, 0x00]]
        );
        // TWO words — **the discriminating arity**. `gs3(&gI,3,4)`:
        //   li r5,4 · addi r3 · li r4,3      (measured)
        //   li r5,4 · li r4,3 · addi r3      (the rule this replaces)
        assert_eq!(
            words(&[SymAddr, Lit(3), Lit(4)]),
            vec![
                lis,
                [0x38, 0xA0, 0x00, 0x04], // li r5,4
                [0x38, 0x6B, 0x00, 0x00], // addi r3,r11,0   <- SECOND
                [0x38, 0x80, 0x00, 0x03], // li r4,3
            ]
        );
        // THREE words, `gs4(&gI,3,4,5)` — the rest of the walk keeps descending.
        assert_eq!(
            words(&[SymAddr, Lit(3), Lit(4), Lit(5)]),
            vec![
                lis,
                [0x38, 0xC0, 0x00, 0x05], // li r6,5
                [0x38, 0x6B, 0x00, 0x00], // addi r3,r11,0
                [0x38, 0xA0, 0x00, 0x04], // li r5,4
                [0x38, 0x80, 0x00, 0x03], // li r4,3
            ]
        );
        // The address at a MIDDLE slot, `gs4b(3,&gI,4,5)`: it is still second,
        // which is what separates this rule from the descending walk WR1 refuted.
        assert_eq!(
            words(&[Lit(3), SymAddr, Lit(4), Lit(5)]),
            vec![
                lis,
                [0x38, 0xC0, 0x00, 0x05], // li r6,5
                [0x38, 0x8B, 0x00, 0x00], // addi r4,r11,0
                [0x38, 0xA0, 0x00, 0x04], // li r5,4
                [0x38, 0x60, 0x00, 0x03], // li r3,3
            ]
        );
        // …and WR1's own `c4`, `s->m3(7,&gI)`: one word, address at slot 2.
        assert_eq!(
            words(&[Formal(0), Lit(7), SymAddr]),
            vec![lis, [0x38, 0x80, 0x00, 0x07], [0x38, 0xAB, 0x00, 0x00]]
        );
        // An in-place formal inside the walk emits nothing and does not count as
        // the one word the address follows: `gs3(&gI,b,7)` is
        // `li r5,7 · addi r3`, not `addi r3 · li r5,7`.
        assert_eq!(
            words(&[SymAddr, Formal(1), Lit(7)]),
            vec![lis, [0x38, 0xA0, 0x00, 0x07], [0x38, 0x6B, 0x00, 0x00]]
        );
    }

    /// The refusals `sym_slots_text` owns, stated positively so a widening cannot
    /// delete one by accident. Each is a shape whose emission is measured to be
    /// something else entirely (`docs/IL_CALL_IN_EXPR.md` §17.2/§17.3).
    #[test]
    fn the_data_address_setup_refuses_the_shapes_it_has_no_capture_for() {
        use c2_il::SlotArg::{Formal, Lit, SymAddr};
        // Two symbols in one call: c2 derives the second by pool-offset
        // difference off the first's `addi`.
        assert!(sym_slots_text(&[SymAddr, SymAddr]).is_err());
        // A formal that has to move beside the address.
        assert!(sym_slots_text(&[SymAddr, Formal(0)]).is_err());
        assert!(sym_slots_text(&[Formal(1), SymAddr]).is_err());
        // Past the eight register slots.
        assert!(sym_slots_text(&[
            SymAddr,
            Formal(1),
            Formal(2),
            Formal(3),
            Formal(4),
            Formal(5),
            Formal(6),
            Formal(7),
            Formal(8)
        ])
        .is_err());
        // A literal wider than the `li` immediate.
        assert!(sym_slots_text(&[SymAddr, Lit(70000)]).is_err());
    }

    // -----------------------------------------------------------------------
    // W-SLOTARG / board #149 — the COMPUTED address (`base + k`, `f(&t->s.k)`).
    //
    // §9.17.5 priced the construct at **+356 emitted functions** and named the
    // single blocker: `tail_call_shape` has no `SlotArg` for a computed address.
    // The variant is trivial; its **position in the permutation walk** is not,
    // and §9.13.1's ALARM is that a wrong position ships GREEN — the 878-TU
    // differential reads 6 match / 0 mismatch under a sink that provably
    // mis-emits, because no byte-exact TU carries the shape.
    //
    // Three capture grids (`scripts/slotarg_grid{1,2,3}.py`, **754 cells** of
    // c2's own `.cod` listing) were taken. Two facts came out, and they are what
    // these two tests exist to keep:
    //
    //   1. the computed address is NOT scheduled like a data-symbol address, so
    //      `sym_slots_text` must never be reused for it;
    //   2. the schedule is **not established** — a rule fitted to 360 in-sample
    //      cells mispredicted 98 of 394 out-of-sample cells, every one of them
    //      an r11 pre-save it did not expect. So the port must REFUSE the shape,
    //      which today it does by having no variant to refuse.
    // -----------------------------------------------------------------------

    /// **The two address constructs take different positions, and one capture
    /// grid cannot tell them apart.**
    ///
    /// For the same arrangement — the address at slot 0 with two literals above
    /// it — c2 places a *data-symbol* address at walk index **1** and a
    /// *computed* address at walk index **2** (last). Wiring the off-add through
    /// [`sym_slots_text`] would therefore emit the right instructions in the
    /// wrong order, which is §9.13.1 verbatim.
    ///
    /// Captured, `/O1 /Oi /EHsc /GS- /c`, `scripts/slotarg_grid1.py`:
    ///
    /// ```text
    ///   f3_0(&t->k, 11, 12)   li r5,12 · li r4,11 · addi r3,r3,8   <- LAST
    ///   gs3(&gI, 3, 4)        li r5,4  · addi r3,r11,0 · li r4,3   <- SECOND
    /// ```
    #[test]
    fn a_computed_address_is_not_scheduled_like_a_data_symbol_address() {
        use c2_il::SlotArg::{Lit, SymAddr};
        // Where `sym_slots_text` puts the address, among the words that follow
        // the hoisted `lis`. The address is the one `addi` sourced from r11.
        let sym_index = |slots: &[c2_il::SlotArg]| -> usize {
            let w = sym_slots_text(slots).unwrap().0;
            let words: Vec<[u8; 4]> =
                w.chunks_exact(4).map(|c| [c[0], c[1], c[2], c[3]]).collect();
            words[1..]
                .iter()
                .position(|w| w[1] & 0x1F == SCRATCH_REG)
                .expect("the address addi is sourced from the scratch")
        };

        // The address at slot 0 with a two-word walk above it.
        assert_eq!(sym_index(&[SymAddr, Lit(3), Lit(4)]), 1);
        // c2's CAPTURED computed-address schedule for the same arrangement puts
        // it at 2 — `li r5,12 · li r4,11 · addi r3,r3,8`. The grids are the
        // evidence; this constant is what makes the divergence portable.
        const COMPUTED_ADDRESS_INDEX_AT_SLOT_0_WALK_2: usize = 2;
        assert_ne!(
            sym_index(&[SymAddr, Lit(3), Lit(4)]),
            COMPUTED_ADDRESS_INDEX_AT_SLOT_0_WALK_2,
            "the data-symbol schedule must not be reused for a computed address"
        );

        // …and it is not an artefact of one arity. Three literals above the
        // address: the symbol stays at 1, the computed address goes to 3.
        assert_eq!(sym_index(&[SymAddr, Lit(3), Lit(4), Lit(5)]), 1);
        const COMPUTED_ADDRESS_INDEX_AT_SLOT_0_WALK_3: usize = 3;
        assert_ne!(
            sym_index(&[SymAddr, Lit(3), Lit(4), Lit(5)]),
            COMPUTED_ADDRESS_INDEX_AT_SLOT_0_WALK_3,
        );

        // The GREEN CONTROL (§9.12's pin): the symbol schedule itself is
        // untouched by this lane and still reads SECOND at the discriminating
        // arity. A mutation that reddened every cell would identify nothing.
        assert_eq!(sym_index(&[SymAddr, Lit(7)]), 1);
        assert_eq!(sym_index(&[Lit(3), SymAddr, Lit(4), Lit(5)]), 1);
    }

    /// **The computed-address schedule is NOT established, so there is no slot
    /// variant to mis-order.**
    ///
    /// The `match` below is exhaustive on purpose: adding a `SlotArg` variant
    /// for `base + k` (board #149) stops it compiling, and whoever adds it has
    /// to read this. What they need to know before choosing an ordering:
    ///
    /// * **WR1's address-last rule mis-emits 654 of the 728 captured cells that
    ///   have a walk (89.8 %)** — it is not a safe default.
    /// * A rule fitted to grids 1–2 (360 cells, agreeing on all 360) mispredicts
    ///   **98 of 394** grid-3 cells. Every single miss is an **r11 pre-save**
    ///   that the fitted rule did not expect, and the axis that produces them is
    ///   the base formal's own register position — which grids 1–2 could not
    ///   vary, because they always parked the base at the lowest slot.
    /// * That is the same shape [`sym_slots_text`] already refuses by name
    ///   ("at two shifting formals c2 pre-saves into r11 … which one probe does
    ///   not separate"). Here it fires at **one** shifting formal.
    ///
    /// Witness (`scripts/slotarg_grid3.py`, `w_32764_3a_0_m1`), where the base
    /// formal sits in r4 rather than r3:
    ///
    /// ```text
    ///   predicted   addi r3,r4,32764 · li r5,12 · li r4,11
    ///   c2 emits    mr r11,r4 · li r5,12 · li r4,11 · addi r3,r11,32764
    /// ```
    #[test]
    fn the_computed_address_schedule_is_not_established_and_has_no_slot_variant() {
        fn every_slot_source_is_accounted_for(a: c2_il::SlotArg) -> &'static str {
            match a {
                c2_il::SlotArg::Formal(_) => "a formal, already in a register",
                c2_il::SlotArg::Lit(_) => "a literal, one `li`",
                c2_il::SlotArg::SymAddr => "a data symbol's address, `lis`+`addi`",
                // **W42** — a shift-and-mask of a formal, one `rlwinm`. It is
                // NOT a computed *address*: it lives only inside a conditional
                // tail arm (`codegen::cond_tail`), and every other call shape
                // refuses it by name. Listing it here keeps this test what it
                // is — an enumeration that fails to compile when the slot
                // vocabulary grows — rather than letting a new variant slip
                // through a wildcard.
                c2_il::SlotArg::ShiftMask { .. } => "W42: `(formal >> k) & m`, one `rlwinm`",
                // No arm for a COMPUTED address: the port refuses the shape by
                // being unable to represent it, which is honest, and is why
                // `c2rs gap` reports these 356 emitted functions as blocked
                // rather than emitting them wrongly.
            }
        }
        assert_eq!(
            every_slot_source_is_accounted_for(c2_il::SlotArg::SymAddr),
            "a data symbol's address, `lis`+`addi`"
        );
        assert_eq!(
            every_slot_source_is_accounted_for(c2_il::SlotArg::Lit(1)),
            "a literal, one `li`"
        );
        assert_eq!(
            every_slot_source_is_accounted_for(c2_il::SlotArg::Formal(0)),
            "a formal, already in a register"
        );

        // …and the permutation walk really has no path that accepts one: every
        // slot list `permute_args_parts` can be handed is built from the three
        // arms above, so there is no arrangement that reaches codegen with a
        // computed address and gets it wrong. Asserted rather than asserted-in-
        // prose, because "the port cannot mis-emit this" is the whole claim.
        for slots in [
            vec![c2_il::SlotArg::SymAddr, c2_il::SlotArg::Lit(3)],
            vec![c2_il::SlotArg::Formal(0), c2_il::SlotArg::Lit(3)],
            vec![c2_il::SlotArg::Formal(0), c2_il::SlotArg::Formal(1)],
        ] {
            let parts = permute_args_parts(&slots);
            assert!(
                parts.is_ok(),
                "the three admitted slot sources still lower; only the computed \
                 address is missing, and it is missing by construction"
            );
        }
    }

    #[test]
    fn argument_permutations_refuse_the_uncharacterized_shapes() {
        // Two disjoint cycles (`g4(d,c,b,a)`): c2 hoists both saves into r11 and
        // r10 and then picks one of several clobber-free orders. One capture does
        // not determine which.
        assert!(permute_args_text(&formal_slots(&[3, 2, 1, 0])).is_err());
        // A repeated argument (`g3(b,a,b)`) emits a dead `mr r11,r4` that no
        // live-value-driven solver would produce.
        assert!(permute_args_text(&formal_slots(&[1, 0, 1])).is_err());
        // More arguments than register slots.
        assert!(permute_args_text(&formal_slots(&[0, 1, 2, 3, 4, 5, 6, 7, 8])).is_err());
    }

    // -----------------------------------------------------------------------
    // #137 — the PORTABLE pin for WR1's address-last rule.
    //
    // [`sym_slots_text`]'s doc comment states the rule and names the
    // discriminating pair. Until now nothing in `cargo test` asserted it: with
    // the `addi` moved back into the descending walk, `cargo test --workspace`
    // read **571 passed / 0 failed** with the toolchain resolving *and* on the
    // portable lane, and only `scripts/gate.sh` went red (10 of 12 lanes,
    // `c2rs diff fixtures/cpp/wr1_sym_addr.cpp` → Mismatch @ obj offset 821).
    // `docs/ROADMAP.md` §9.12.
    // -----------------------------------------------------------------------

    /// The words the two rules argue about, named once so the two tests below
    /// cannot drift: `lis r11,0`, `li rD,k` (`addi rD,0,k`) and the low half
    /// `addi rD,r11,0`. Three distinct encodings — in particular the `li` and
    /// the low-half `addi` differ in their RA field (0 against 11), which is
    /// what makes an order assertion over them meaningful.
    fn sym_words(lit_slot: usize, lit: i16, sym_slot: usize) -> (Vec<u8>, Vec<u8>, Vec<u8>) {
        (
            encode_addis(SCRATCH_REG, 0, 0).to_vec(),
            encode_addi(ARG_REGS[lit_slot], 0, lit).to_vec(),
            encode_addi(ARG_REGS[sym_slot], SCRATCH_REG, 0).to_vec(),
        )
    }

    /// **#137 rule 1 — the address `addi` is emitted LAST**, after every other
    /// slot's setup, whatever slot it belongs to.
    ///
    /// This is the arrangement that says so, and the only one that can:
    /// `void c4(S* s){ s->m3(7, &gI); }` puts the literal at slot **1** and the
    /// symbol at slot **2**, so the literal's slot is strictly *lower*. A
    /// descending-destination walk emits the higher slot first and would produce
    /// `lis · addi r5 · li r4`; c2 produces `lis · li r4,7 · addi r5,r11,0`
    /// (MEASURED, `work/wr1/probes/p2.cpp` at `/Ox /GS- /c`).
    ///
    /// **The arrangement is the whole test.** WR1's own hand fixture had three
    /// literal cases and all three put the symbol at slot 0, where the two rules
    /// agree — see the companion test below, which is that agreeing case and is
    /// the control saying which of the two arrangements does the work.
    #[test]
    fn the_address_addi_is_emitted_last_even_when_a_literal_sits_at_a_lower_slot() {
        use c2_il::SlotArg::{Formal, Lit, SymAddr};
        let slots = vec![Formal(0), Lit(7), SymAddr];

        // (a) The fixture property, pinned FIRST and phrased over the INPUT
        // rather than over the rule under test: without a literal at a strictly
        // lower slot than the symbol this body cannot tell the two rules apart,
        // and every assertion below would pass under either.
        let lit_slot = slots.iter().position(|a| matches!(a, Lit(_))).expect("a literal slot");
        let sym_slot = slots.iter().position(|a| matches!(a, SymAddr)).expect("a symbol slot");
        assert!(
            lit_slot < sym_slot,
            "(a) this arrangement does not discriminate: the literal must sit at \
             a strictly LOWER slot than the symbol, got literal at {lit_slot} and \
             symbol at {sym_slot}"
        );

        let (lis, li, addi) = sym_words(lit_slot, 7, sym_slot);
        let (w, _writes) = sym_slots_text(&slots).expect("in class");

        // (b) Three words and no more — pinned before anything is read by index.
        assert_eq!(
            w.len(),
            12,
            "(b) `s->m3(7, &gI)` setup is 3 words (lis, li, addi), got {} bytes",
            w.len()
        );

        // (c) The rule, positionally: the LOWER slot's `li` precedes the HIGHER
        // slot's address `addi`. Stated over positions rather than over the whole
        // byte string so that a future re-encoding still leaves a red on the
        // ORDER rather than on an unrelated field.
        let words: Vec<&[u8]> = w.chunks_exact(4).collect();
        let li_at = words.iter().position(|x| *x == li.as_slice());
        let addi_at = words.iter().position(|x| *x == addi.as_slice());
        assert_eq!(
            (li_at, addi_at),
            (Some(1), Some(2)),
            "(c) the address `addi` must come LAST: expected `li` at word 1 and \
             the low-half `addi` at word 2, got li_at={li_at:?} addi_at={addi_at:?} \
             in {words:02x?} — a descending walk would put the symbol's slot 2 \
             ahead of the literal's slot 1"
        );

        // (d) The `lis` is hoisted to the body's first word — the other half of
        // the pair, and what `PortC2::data_refs_of` derives `hi_off` from.
        assert_eq!(
            words[0],
            lis.as_slice(),
            "(d) the `lis rS,sym@ha` must be the body's FIRST word: {words:02x?}"
        );

        // (e) …and the whole string, so a reordering that somehow satisfies (c)
        // and (d) separately still goes red.
        let mut want = lis.clone();
        want.extend_from_slice(&li);
        want.extend_from_slice(&addi);
        assert_eq!(
            w, want,
            "(e) `s->m3(7, &gI)` must be `lis r11 · li r4,7 · addi r5,r11,0`"
        );
    }

    /// **The control for the test above — this one AGREES under both rules.**
    ///
    /// `void c1(){ gsp(&gI, 7); }` puts the symbol at slot **0** and the literal
    /// at slot 1, and a descending walk reaches slot 0 last anyway. So this test
    /// must stay **green** under the descending mutation while its neighbour
    /// goes red; if it went red too, the neighbour's red would not identify
    /// *which* arrangement discriminates and the pair would be worth nothing.
    ///
    /// This is the shape WR1's fixture had three copies of (`c1`, `c2`, `c3`) —
    /// a fixture that grew by three cases without gaining one bit of evidence.
    #[test]
    fn the_symbol_at_slot_zero_does_not_discriminate_the_two_rules() {
        use c2_il::SlotArg::{Lit, SymAddr};
        let slots = vec![SymAddr, Lit(7)];

        // (a) The fixture property again, and the opposite one: the symbol is at
        // the LOWEST slot, which is exactly where the two rules agree.
        let lit_slot = slots.iter().position(|a| matches!(a, Lit(_))).expect("a literal slot");
        let sym_slot = slots.iter().position(|a| matches!(a, SymAddr)).expect("a symbol slot");
        assert_eq!(
            (sym_slot, lit_slot),
            (0, 1),
            "(f) the control needs the symbol at slot 0 and a literal above it, \
             got symbol at {sym_slot} and literal at {lit_slot}"
        );

        let (lis, li, addi) = sym_words(lit_slot, 7, sym_slot);
        let (w, _writes) = sym_slots_text(&slots).expect("in class");
        let mut want = lis;
        want.extend_from_slice(&li);
        want.extend_from_slice(&addi);
        assert_eq!(
            w, want,
            "(g) `gsp(&gI, 7)` must be `lis r11 · li r4,7 · addi r3,r11,0` — the \
             arrangement on which descending and address-last AGREE"
        );
    }
}
