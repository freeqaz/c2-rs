//! PPC word encoders — one fact per function, no dependencies.
//!
//! Every `encode_*` here turns operands into the four big-endian bytes of one
//! instruction, and nothing else: no `IlFunction`, no gate, no allocation
//! policy. That is why they live together in one alphabetizable file rather
//! than beside the lowerings that call them.
//!
//! > ⚠ **STRUCK 2026-08-26, board #3640 — the paragraph below is a dated
//! > record of this file BEFORE read R2 landed, and it is superseded by
//! > § "2026-08-22, lane `w-s1`" twenty-odd lines down.** It describes R2 as
//! > future work; R2 landed the same day it was written, and the 85 opcode
//! > literals it is about are gone. Two lanes read this one `//!` block to
//! > opposite conclusions (`#3634` vs decision 16) because the correction was
//! > appended *below* and the original was never struck — which is
//! > `DOC_CONVENTIONS.md` §2's own named failure mode: **an amend-beside that
//! > does not strike its original is a second claim, not a correction.** What
//! > survives of it is stated in the superseding section: the *choice of
//! > opcode and operand role* is still black-box; the *bits* are read.
//! >
//! > ~~**This file is a black-box re-derivation of two tables c2 states
//! > plainly, and the read is priced — comment only, nothing here changes.**
//! > Added 2026-08-22 under read-before-probe
//! > (`docs/WHITEBOX_LEVERAGE_2026-08-21.md` §1;
//! > `docs/whitebox/READ_PLAN_2026-08-21.md` §2/§3). Every word below was
//! > recovered from captured objs; c2 composes the same words from a base-word
//! > table at `0x10c3a578` and an encode-form table at `0x10c39b18`,
//! > dispatched through a 111-entry jump table at `0x10bfae2d` with **79
//! > distinct arm targets**, all inside `FUN_10bf9f15`'s 3,861 bytes. Read
//! > **R2** (2–4 d) dumps both tables and reads the 79 arms, yielding
//! > `encode(tuple) → u32` as a **total function** — the same content this
//! > file accumulates one captured fact at a time. R2 is also the read that
//! > specs **I2**, the general-lowering row priced at 1.5–4.5
//! > engineer-months (`docs/STEP5_PRICING_2026-08-21.md` §2).~~
//!
//! **The bound, so this is not overread**: a complete encoder is not a
//! complete emit seam — relocations are **not** in R2's scope (0 cells read at
//! that seam), and ~~2 of the 111 entries are read today~~ — **that clause is
//! struck with the paragraph above, 2026-08-26, board #3640**: it counted the
//! jump-table entries read *before* R2, and R2 read all 79 distinct arms. The
//! port transcribes **27** of them, covering **35** form numbers (counted in
//! `super::mop::plan` on this tree; `DISCLOSURE.md` `W-MOP-3` is the row).
//! Relocations are still 0. The per-function evidence notes below stay as
//! written; they are what the port is graded on.
//!
//! The file also exists to make one specific defect impossible. Two branches
//! once landed two `encode_std`s 2,000 lines apart in the old single-file
//! `codegen.rs` and git flagged nothing (`docs/ARCHITECTURE_SEAMS.md` §1,
//! class 4). In one file a duplicate encoder is a compile error, in the same
//! file, immediately.
//!
//! ---
//!
//! # 2026-08-22, lane `w-s1` — **THE BLACK-BOX RE-DERIVATION IS RETIRED, AND
//! THE READ TABLE IS THE PORT'S ONLY SOURCE OF A PRIMARY OPCODE ~~FULL STOP~~
//! FOR EVERY INSTRUCTION IN THIS FILE.**
//!
//! *(The qualifier is a correction, 2026-08-26, board **#3638**. As written the
//! headline was true of `encode.rs` and false of the crate: `calls.rs` and
//! `frame.rs` held **eight** word productions sourcing primaries 14, 15, 18,
//! 31, 32 and 36 from their own literals — board **#3637**. Lane `w-mopfold`
//! folded all eight, and three instructions the port emits still have no row
//! (`bl`, `mfspr`, `stwux`), enumerated and re-checked every test run by
//! `super::word_seam`. The scope word is the whole repair: the sentence was
//! never wrong about this file.)*
//!
//! Every function below used to carry its own copy of a primary opcode and an
//! extended opcode as literals. Read **R2** dumped the two tables those
//! literals were re-deriving, so they are gone: each function is now a
//! **[`super::mop::MachineOp`] constructor** naming c2's own opcode number, and
//! [`super::mop::encode_op`] is the one place a word is composed.
//!
//! **What it did NOT retire, per `#3640`'s adjudication.** `w-s1` moved where
//! the *bits* come from; it did not move **which opcode a lowering should name
//! and which operand role each argument plays.** That choice is still
//! black-box — recovered from captured objs, function by function, and recorded
//! in the per-function evidence notes below. So `#3634` ("the black-box
//! re-derivation was retired") and decision 16 ("this file is a black-box
//! re-derivation") are both right about different objects, and quoting either
//! as *"nothing in `encode.rs` is black-box-derived any more"* is the over-read
//! neither of them made.
//!
//! **What that buys, concretely.** The bit layouts stopped being 85 independent
//! facts. `xo31`, `fp_a_form`, `fp_primary` and `encode_logical_x`'s `xo`
//! parameter are deleted: each was a *positional* helper whose argument order
//! silently meant different fields for different callers — `xo31`'s first
//! argument was the destination for `subfc` (form 49) and the **source** for
//! `extsb` (form 38), which is precisely the class of confusion
//! `encode_logical_x`'s own doc warns about, living inside the helper meant to
//! prevent it.
//!
//! **What it does NOT change**: every signature, every returned word, and every
//! per-function evidence note below. The originals are kept verbatim in
//! `mod incumbent` at the bottom of this file and cross-checked against the
//! general composition over each function's whole operand domain — that test,
//! not the gate, is what makes the required-zero byte delta provable in the
//! portable lane.

use super::mop::{op, MachineOp};

/// Encode `add rD, rA, rB` (rD = rA + rB): primary opcode 31, XO 266, OE=0,
/// Rc=0. Returns the 4-byte big-endian instruction word.
///
/// `word = (31<<26) | (rd<<21) | (ra<<16) | (rb<<11) | (266<<1)`.
pub fn encode_add(rd: u8, ra: u8, rb: u8) -> [u8; 4] {
    mop_add(rd, ra, rb).word()
}

/// The [`MachineOp`] form of [`encode_add`] — the value S1c's op streams
/// carry, before any word exists.
///
/// **The evidence note for this opcode and its operand roles is on
/// [`encode_add`] directly above, and stays there.** This function adds
/// nothing to it but the absence of the final `.word()`.
#[inline(always)]
pub fn mop_add(rd: u8, ra: u8, rb: u8) -> MachineOp {
    MachineOp::new(op::ADD).s(rd).d0(ra).d1(rb)
}

/// Encode `mullw rD, rA, rB` (rD = rA * rB): primary opcode 31, XO 235, OE=0,
/// Rc=0. Commutative in rA/rB (like `add`), so operand order is match-neutral.
pub fn encode_mullw(rd: u8, ra: u8, rb: u8) -> [u8; 4] {
    mop_mullw(rd, ra, rb).word()
}

/// The [`MachineOp`] form of [`encode_mullw`] — the value S1c's op streams
/// carry, before any word exists.
///
/// **The evidence note for this opcode and its operand roles is on
/// [`encode_mullw`] directly above, and stays there.** This function adds
/// nothing to it but the absence of the final `.word()`.
#[inline(always)]
pub fn mop_mullw(rd: u8, ra: u8, rb: u8) -> MachineOp {
    MachineOp::new(op::MULLW).s(rd).d0(ra).d1(rb)
}

/// Encode `subf rD, rA, rB`: primary opcode 31, XO 40, OE=0, Rc=0.
///
/// **Non-commutative — operand order is load-bearing.** `subf` computes
/// `rD = rB - rA` (the *first* register operand is the subtrahend). To realize
/// a source `lhs - rhs`, the caller must pass `ra = rhs` (subtrahend) and
/// `rb = lhs` (minuend). Swapping `ra`/`rb` silently negates the result — a
/// corruption invisible to `fuzzy%` (it is a valid `subf`, just the wrong one),
/// exactly the non-commutative hazard the CLAUDE.md correctness boundary names.
/// This encoder is deliberately separate from `encode_add` and its single
/// caller ([`select_text`]'s `Sub` arm) documents the mapping at the call site.
pub fn encode_subf(rd: u8, ra: u8, rb: u8) -> [u8; 4] {
    mop_subf(rd, ra, rb).word()
}

/// The [`MachineOp`] form of [`encode_subf`] — the value S1c's op streams
/// carry, before any word exists.
///
/// **The evidence note for this opcode and its operand roles is on
/// [`encode_subf`] directly above, and stays there.** This function adds
/// nothing to it but the absence of the final `.word()`.
#[inline(always)]
pub fn mop_subf(rd: u8, ra: u8, rb: u8) -> MachineOp {
    MachineOp::new(op::SUBF).s(rd).d0(ra).d1(rb)
}

/// Encode an **X-form logical / shift** `op rA, rS, rB` — the register-register
/// bitwise and shift family, `lane w-build`.
///
/// **The field order is NOT the one [`encode_add`] uses, and that is the whole
/// reason this is a separate encoder rather than a parameter on that one.** The
/// D-form arithmetic instructions put the *destination* in the RT field at bits
/// 6–10; every instruction below puts the destination in the **RA** field at
/// bits 11–15 and its *source* in RS at 6–10. Encoding `and` through
/// `encode_add`'s layout produces a valid `and` with the destination and the
/// left operand exchanged — bytes that assemble, disassemble and fuzz-match, and
/// compute the wrong thing whenever the two differ.
///
/// Every `xo` below is read off a **transcribed capture**, never inferred:
/// `work/w-build/probe/bits.cod` and `bits2.cod`, at the workload's own
/// `/GR /O1 /Oi /EHsc`. The sixteen captured words are reproduced verbatim by
/// this encoder in [`the_logical_xforms_reproduce_their_captured_words`].
///
/// ```text
///   and  r3,r3,r4    7c632038      xo  28     and  r3,r11,r5   7d632838
///   or   r3,r3,r4    7c632378      xo 444     or   r3,r11,r10  7d635378
///   xor  r3,r3,r4    7c632278      xo 316     xor  r3,r11,r5   7d632a78
///   slw  r3,r3,r4    7c632030      xo  24     slw  r3,r11,r5   7d632830
///   srw  r3,r3,r4    7c632430      xo 536
///   sraw r3,r3,r4    7c632630      xo 792     sraw r11,r3,r4   7c6b2630
/// ```
///
/// `ra` is the DESTINATION, `rs` the left operand, `rb` the right one. The
/// shifts are **non-commutative** in exactly the way [`encode_subf`] warns about
/// — `rs` is the value shifted and `rb` the amount — so the three arguments are
/// named for their roles rather than for their field letters.
fn encode_logical_x(op: super::mop::C2Op, ra_dest: u8, rs_lhs: u8, rb_rhs: u8) -> [u8; 4] {
    mop_logical_x(op, ra_dest, rs_lhs, rb_rhs).word()
}

/// The [`MachineOp`] form of [`encode_logical_x`] — the value S1c's op streams
/// carry, before any word exists.
///
/// **The evidence note for this opcode and its operand roles is on
/// [`encode_logical_x`] directly above, and stays there.** This function adds
/// nothing to it but the absence of the final `.word()`.
#[inline(always)]
pub fn mop_logical_x(op: super::mop::C2Op, ra_dest: u8, rs_lhs: u8, rb_rhs: u8) -> MachineOp {
    MachineOp::new(op).s(ra_dest).d0(rs_lhs).d1(rb_rhs)
}

/// `and rA, rS, rB` — XO 28. Commutative; captured `7c632038`.
pub fn encode_and(dest: u8, lhs: u8, rhs: u8) -> [u8; 4] {
    encode_logical_x(op::AND, dest, lhs, rhs)
}

/// The [`MachineOp`] form of [`encode_and`] — the value S1c's op streams
/// carry, before any word exists.
///
/// **The evidence note for this opcode and its operand roles is on
/// [`encode_and`] directly above, and stays there.** This function adds
/// nothing to it but the absence of the final `.word()`.
#[inline(always)]
pub fn mop_and(dest: u8, lhs: u8, rhs: u8) -> MachineOp {
    mop_logical_x(op::AND, dest, lhs, rhs)
}

/// `or rA, rS, rB` — XO 444. Commutative; captured `7c632378`.
pub fn encode_or(dest: u8, lhs: u8, rhs: u8) -> [u8; 4] {
    encode_logical_x(op::OR, dest, lhs, rhs)
}

/// The [`MachineOp`] form of [`encode_or`] — the value S1c's op streams
/// carry, before any word exists.
///
/// **The evidence note for this opcode and its operand roles is on
/// [`encode_or`] directly above, and stays there.** This function adds
/// nothing to it but the absence of the final `.word()`.
#[inline(always)]
pub fn mop_or(dest: u8, lhs: u8, rhs: u8) -> MachineOp {
    mop_logical_x(op::OR, dest, lhs, rhs)
}

/// `xor rA, rS, rB` — XO 316. Commutative; captured `7c632278`.
pub fn encode_xor(dest: u8, lhs: u8, rhs: u8) -> [u8; 4] {
    encode_logical_x(op::XOR, dest, lhs, rhs)
}

/// The [`MachineOp`] form of [`encode_xor`] — the value S1c's op streams
/// carry, before any word exists.
///
/// **The evidence note for this opcode and its operand roles is on
/// [`encode_xor`] directly above, and stays there.** This function adds
/// nothing to it but the absence of the final `.word()`.
#[inline(always)]
pub fn mop_xor(dest: u8, lhs: u8, rhs: u8) -> MachineOp {
    mop_logical_x(op::XOR, dest, lhs, rhs)
}

/// `slw rA, rS, rB` — XO 24. **Non-commutative**: `lhs` is shifted by `rhs`.
///
/// One instruction for both signednesses, and that is measured rather than
/// assumed: `int f(int a,int b){return a<<b;}` and the all-`unsigned` spelling
/// both emit `7c632030`, as does the mixed `int f(int a,unsigned b)`.
pub fn encode_slw(dest: u8, lhs: u8, rhs: u8) -> [u8; 4] {
    encode_logical_x(op::SLW, dest, lhs, rhs)
}

/// The [`MachineOp`] form of [`encode_slw`] — the value S1c's op streams
/// carry, before any word exists.
///
/// **The evidence note for this opcode and its operand roles is on
/// [`encode_slw`] directly above, and stays there.** This function adds
/// nothing to it but the absence of the final `.word()`.
#[inline(always)]
pub fn mop_slw(dest: u8, lhs: u8, rhs: u8) -> MachineOp {
    mop_logical_x(op::SLW, dest, lhs, rhs)
}

/// `srw rA, rS, rB` — XO 536, the **logical** right shift. Captured `7c632430`.
pub fn encode_srw(dest: u8, lhs: u8, rhs: u8) -> [u8; 4] {
    encode_logical_x(op::SRW, dest, lhs, rhs)
}

/// The [`MachineOp`] form of [`encode_srw`] — the value S1c's op streams
/// carry, before any word exists.
///
/// **The evidence note for this opcode and its operand roles is on
/// [`encode_srw`] directly above, and stays there.** This function adds
/// nothing to it but the absence of the final `.word()`.
#[inline(always)]
pub fn mop_srw(dest: u8, lhs: u8, rhs: u8) -> MachineOp {
    mop_logical_x(op::SRW, dest, lhs, rhs)
}

/// `sraw rA, rS, rB` — XO 792, the **arithmetic** right shift. Captured
/// `7c632630`.
///
/// **`sraw` and `srw` differ by one bit of the operand TYPE and by nothing in
/// the IL opcode**, which is the trap this family carries and the reason
/// `parse_expr` refuses a mixed-signedness right shift outright. Probed:
/// `int f(int a, unsigned b){return a>>b;}` is `sraw` and
/// `unsigned f(unsigned a, int b){return a>>b;}` is `srw` — **only the LEFT
/// operand decides**, and both spellings carry the identical IL byte `0A`.
pub fn encode_sraw(dest: u8, lhs: u8, rhs: u8) -> [u8; 4] {
    encode_logical_x(op::SRAW, dest, lhs, rhs)
}

/// The [`MachineOp`] form of [`encode_sraw`] — the value S1c's op streams
/// carry, before any word exists.
///
/// **The evidence note for this opcode and its operand roles is on
/// [`encode_sraw`] directly above, and stays there.** This function adds
/// nothing to it but the absence of the final `.word()`.
#[inline(always)]
pub fn mop_sraw(dest: u8, lhs: u8, rhs: u8) -> MachineOp {
    mop_logical_x(op::SRAW, dest, lhs, rhs)
}

/// Encode `addi rD, rA, SI` (rD = rA + sign-extended SI): primary opcode 14.
/// `SI` is a 16-bit signed immediate. Note `addi` special-cases `rA = 0` to
/// mean the literal 0 (not the contents of r0), so `addi rD, 0, k` is the
/// canonical `li rD, k`. Used for `reg ± small-constant` and constant loads.
pub fn encode_addi(rd: u8, ra: u8, si: i16) -> [u8; 4] {
    mop_addi(rd, ra, si).word()
}

/// The [`MachineOp`] form of [`encode_addi`] — the value S1c's op streams
/// carry, before any word exists.
///
/// **The evidence note for this opcode and its operand roles is on
/// [`encode_addi`] directly above, and stays there.** This function adds
/// nothing to it but the absence of the final `.word()`.
#[inline(always)]
pub fn mop_addi(rd: u8, ra: u8, si: i16) -> MachineOp {
    MachineOp::new(op::ADDI).s(rd).d0(ra).disp(si as i32)
}

/// Encode `addis rD, rA, SI` (rD = rA + (SI << 16)): primary opcode 15. The
/// high half of a wide constant / immediate (with rA=0 for the `lis` idiom).
pub fn encode_addis(rd: u8, ra: u8, si: i16) -> [u8; 4] {
    mop_addis(rd, ra, si).word()
}

/// The [`MachineOp`] form of [`encode_addis`] — the value S1c's op streams
/// carry, before any word exists.
///
/// **The evidence note for this opcode and its operand roles is on
/// [`encode_addis`] directly above, and stays there.** This function adds
/// nothing to it but the absence of the final `.word()`.
#[inline(always)]
pub fn mop_addis(rd: u8, ra: u8, si: i16) -> MachineOp {
    MachineOp::new(op::ADDIS).s(rd).d0(ra).disp(si as i32)
}

/// Encode `ori rA, rS, UI` (rA = rS | UI): primary opcode 24. The low half of
/// a wide **constant load** (`lis`+`ori`); `UI` is a zero-extended 16-bit field.
pub fn encode_ori(ra: u8, rs: u8, ui: u16) -> [u8; 4] {
    mop_ori(ra, rs, ui).word()
}

/// The [`MachineOp`] form of [`encode_ori`] — the value S1c's op streams
/// carry, before any word exists.
///
/// **The evidence note for this opcode and its operand roles is on
/// [`encode_ori`] directly above, and stays there.** This function adds
/// nothing to it but the absence of the final `.word()`.
#[inline(always)]
pub fn mop_ori(ra: u8, rs: u8, ui: u16) -> MachineOp {
    MachineOp::new(op::ORI).s(ra).d0(rs).imm_d1(ui as u32)
}

/// `blr` — branch to link register (function return). `bclr` with BO=20
/// ("always"), BI=0, LK=0 → the fixed word `0x4E800020`.
pub fn encode_blr() -> [u8; 4] {
    mop_blr().word()
}

/// The [`MachineOp`] form of [`encode_blr`] — the value S1c's op streams
/// carry, before any word exists.
///
/// **The evidence note for this opcode and its operand roles is on
/// [`encode_blr`] directly above, and stays there.** This function adds
/// nothing to it but the absence of the final `.word()`.
#[inline(always)]
pub fn mop_blr() -> MachineOp {
    MachineOp::new(op::BLR)
}

/// `bclr BO,BI` — a **conditional return**: branch to the link register when the
/// CR bit says so, opcode 19 XO 16, `LK = 0`.
///
/// This is w-rotate's **P2** in one word (`docs/rungs/2026-08-05-w-rotate.md`
/// §3, 46 of 46): a rotation guard branches to the block the loop falls out to,
/// and it **folds to `bclr` exactly when that block is a bare `blr`** — so the
/// guard carries no displacement at all and cannot go stale when the body's
/// length changes. It is the reason a variable-length loop body needs no
/// forward fixup for its entry test.
///
/// Captured: `4d820020` = `bclr 12,2` (branch-if-cr0.EQ to LR), every
/// `TWO`-regime cell of `work/w-varloop/probe.py`. [`encode_blr`] is this word
/// at `BO = `[`BO_ALWAYS`]`, BI = 0`, and the two agree by construction — there
/// is a test.
pub fn encode_bclr(bo: u8, bi: u8) -> [u8; 4] {
    mop_bclr(bo, bi).word()
}

/// The [`MachineOp`] form of [`encode_bclr`] — the value S1c's op streams
/// carry, before any word exists.
///
/// **The evidence note for this opcode and its operand roles is on
/// [`encode_bclr`] directly above, and stays there.** This function adds
/// nothing to it but the absence of the final `.word()`.
#[inline(always)]
pub fn mop_bclr(bo: u8, bi: u8) -> MachineOp {
    MachineOp::new(op::BCLR).s(bo).d0(bi)
}

/// `mtctr rS` — `mtspr 9,rS`: opcode 31, XO 467, with the SPR number carried in
/// a **split five-and-five field**, low half first.
///
/// The split is the part worth spelling out, because writing `9 << 11` produces
/// a legal-looking `mtspr` naming SPR 288 and the assembler in your head does
/// not catch it: the field at bits 11..20 is `(spr & 0x1F) << 5 | (spr >> 5)`,
/// so SPR 9 is `0x120` and not `0x009`.
///
/// Captured, not derived: `7d6903a6` = `mtctr r11` in every converted cell of
/// `work/w-bdnz/probe/L3.obj`, and `7c8903a6` = `mtctr r4` in `L1.obj`. Both are
/// reproduced by the assertion in this module's tests.
///
/// **This is `wb-loop`'s pass 2 in one word.** It is minted by
/// `p2\ppc\lower.c`'s per-loop converter and nothing else creates it, which is
/// why `/d2QXnobdnz` removes all 29 `bdnz` **and** 29 of the 31 `mtctr` from a
/// 36-cell obj — the two survivors being a `bctrl` and a `bctr`, i.e. genuine
/// indirect branches (`WB_LOOP_FINDINGS.md` §7.7). That counterfactual is the
/// black-box evidence this encoder rests on; no address out of `c2.dll` is used
/// here or anywhere in the class.
pub fn encode_mtctr(rs: u8) -> [u8; 4] {
    mop_mtctr(rs).word()
}

/// The [`MachineOp`] form of [`encode_mtctr`] — the value S1c's op streams
/// carry, before any word exists.
///
/// **The evidence note for this opcode and its operand roles is on
/// [`encode_mtctr`] directly above, and stays there.** This function adds
/// nothing to it but the absence of the final `.word()`.
#[inline(always)]
pub fn mop_mtctr(rs: u8) -> MachineOp {
    // PROV[S] PowerPC ISA — `SPR` 9 is `CTR`. Not from c2. The split-field spelling beside it (`(spr & 0x1F) << 5 | spr >> 5`) is the ISA's, too.
    const SPR_CTR: u32 = 9;
    MachineOp::new(op::MTSPR)
        .s(rs)
        .imm_d1(SPR_CTR & 0x1F)
        .imm_d2(SPR_CTR >> 5)
}

/// `mtlr rS` — `mtspr 8,rS`: the epilogue's link-register restore.
///
/// The evidence and the split-field trap are on [`mop_mtlr`] below, because
/// **this pair is the one place in the file where the `mop_*` half came first**:
/// lane `w-mopfold` needed a `const fn` for `frame::FRAME_MTLR_R12`, and the
/// `encode_*` twin exists so `mtlr` is shaped like every other instruction here
/// rather than being the file's one lone `mop_*`. It has no caller today; that
/// is the honest state of an encoder whose only consumer is a `const`.
pub fn encode_mtlr(rs: u8) -> [u8; 4] {
    mop_mtlr(rs).word()
}

/// `mflr rD` — `mfspr rD,8`: the prologue's link-register save.
///
/// The same shape as [`encode_mtlr`] and for the same reason — lane
/// `w-encarms` needed the `const fn` [`mop_mflr`] for `frame::FRAME_MFLR_R12`,
/// and this twin exists so `mflr` is shaped like every other instruction in the
/// file. It has no caller today either; `S1c` (i) asks for one twin per
/// encoder, and the honest state of an encoder whose only consumer is a `const`
/// is that it has none.
pub fn encode_mflr(rd: u8) -> [u8; 4] {
    mop_mflr(rd).word()
}

/// `mtlr rS` — `mtspr 8,rS`: the epilogue's link-register restore, the same
/// opcode and the same split field as [`mop_mtctr`] one SPR number over.
///
/// **Added by lane `w-mopfold` so `frame.rs` stops spelling this instruction as
/// the literal `0x7D88_03A6`** (board **#3637**: `FRAME_MTLR_R12` was one of
/// eight words the port composed by a second rule). `mtspr`'s base word carries
/// the SPR field at **zero** and c2's form-62 arm does the five-and-five split
/// itself (`mop::plan`'s arm `10bfa7a3`, `P_ENCODE.md` §8.1 residual 5), which
/// is precisely why a baked full word cannot show which half is which and this
/// constructor can.
///
/// `const fn` because its one caller is a `const`. Evidence: `7d8803a6` is
/// `mtlr r12` in every framed epilogue this project has captured, and
/// `word_seam`'s inventory pins the historical literal against it.
/// `mflr rD` — `mfspr rD,8`, the **mirror** of [`mop_mtlr`].
///
/// **Added by lane `w-encarms` (wave 18) so `frame.rs` stops spelling this
/// instruction as the literal `0x7D88_02A6`.** `w-mopfold` folded four of
/// `frame.rs`'s six fixed words and priced this one as a *refusal*, because
/// `mop::OPCODES` had no `mfspr` row and `mop::plan` no form-54 arm — a missing
/// transcription rather than a disagreement, and `word_seam::EXCEPTIONS`
/// carried it armed. Both halves exist now: the row (`0x00e6`, base
/// `7c0002a6`, form **54**) and the arm `10bfa76a`, read at that address by
/// this lane. `DISCLOSURE.md` `W-ENCARMS-1`.
///
/// The SPR split is the same low-half-first shape as `mtspr`'s and it is
/// c2's arm, not this port's convention, that does it — which is exactly why
/// the baked `0x7D88_02A6` could not show which half was which.
#[inline(always)]
pub const fn mop_mflr(rd: u8) -> MachineOp {
    // PROV[S] PowerPC ISA — `SPR` 8 is `LR`. Not from c2; the same fact `mop_mtlr` uses one instruction over.
    const SPR_LR: u32 = 8;
    MachineOp::new(op::MFSPR)
        .s(rd)
        .imm_d1(SPR_LR & 0x1F)
        .imm_d2(SPR_LR >> 5)
}

#[inline(always)]
pub const fn mop_mtlr(rs: u8) -> MachineOp {
    // PROV[S] PowerPC ISA — `SPR` 8 is `LR`. Not from c2; the neighbouring `SPR_CTR` is the same fact one number over.
    const SPR_LR: u32 = 8;
    MachineOp::new(op::MTSPR)
        .s(rs)
        .imm_d1(SPR_LR & 0x1F)
        .imm_d2(SPR_LR >> 5)
}

/// **W-MMIO3 — `bctrl`**: branch to CTR, unconditional, and set LR.
///
/// `4e800421`, one word with no operands at all. `XL`-form:
/// opcode 19, `BO = 20` (branch always, CR ignored, CTR not decremented),
/// `BI = 0`, `BH = 0`, extended opcode **528** (`bcctr`), `LK = 1`.
///
/// **Captured, not derived.** `src/xdk/nuispeech/mmio.cpp`'s reference obj at
/// the workload's own flags carries it at `.text #14 + 0x50`, immediately after
/// the `mtctr r11` [`encode_mtctr`] already emits, and
/// `WB_LOOP_FINDINGS.md` §7.7's `/d2QXnobdnz` counterfactual names the same
/// word from the other side: of a 36-cell obj's 31 `mtctr`, the two that
/// SURVIVE the switch are the ones feeding a `bctrl` and a `bctr`, i.e. the
/// genuine indirect branches that `p2\ppc\lower.c`'s loop converter did not
/// mint. This is the `bctrl` of that pair.
///
/// **`LK = 1` is the whole difference from `bctr`** and it is what makes the
/// caller framed: the callee's `blr` returns here, so LR must be saved, which
/// is why every user of this word is a `.pdata`-bearing function.
pub fn encode_bctrl() -> [u8; 4] {
    mop_bctrl().word()
}

/// The [`MachineOp`] form of [`encode_bctrl`] — the value S1c's op streams
/// carry, before any word exists.
///
/// **The evidence note for this opcode and its operand roles is on
/// [`encode_bctrl`] directly above, and stays there.** This function adds
/// nothing to it but the absence of the final `.word()`.
#[inline(always)]
pub fn mop_bctrl() -> MachineOp {
    MachineOp::new(op::BCTRL)
}

/// `BO` for **"decrement CTR, then branch if CTR is still non-zero"** — the
/// `bdnz` form. Bit 2 of `BO` clears ("decrement the counter"), bit 1 clears
/// ("branch if the counter is non-zero") and bit 0 sets ("ignore the CR"),
/// giving `10000` = 16.
///
/// It is a **named constant beside [`BO_TRUE`]/[`BO_FALSE`]** rather than a
/// literal for the reason [`CR_COMPARE`]'s doc gives one level over: `BO = 16`
/// ignores `BI` entirely, so a `bdnz` that borrowed a CR bit from a nearby
/// compare would still assemble and still branch correctly — and would differ
/// from c2's word in bits nobody would think to look at. `BI` is **0** in every
/// captured `bdnz`.
/// PROV[S] PowerPC ISA — `BO` 16 is decrement-CTR-and-branch-if-nonzero. Its DOC is `[O]` and worth reading: `BI` is 0 in every captured `bdnz`, and because `BO = 16` ignores `BI`, a wrong `BI` would still assemble and still branch correctly while differing from c2's word.
pub const BO_DNZ: u8 = 16;

/// `bdnz <target>` — decrement CTR and branch back while it is non-zero.
///
/// Captured: `4200fffc` = `bdnz .-4` (the one-instruction body of every cell in
/// `work/w-bdnz/probe/L3.obj`) and `4200fff8` = `bdnz .-8` (`L1.obj`, a two-word
/// body). `disp` is self-relative and the reach is [`BC_MAX_DISP`]'s, so this is
/// [`encode_bc`] at [`BO_DNZ`] with `BI = 0` and it returns `None` out of range
/// for the same reason: a truncated `BD` is a legal-looking branch to the wrong
/// place.
pub fn encode_bdnz(disp: i32) -> Option<[u8; 4]> {
    Some(mop_bdnz(disp)?.word())
}

/// The [`MachineOp`] form of [`encode_bdnz`] — the value S1c's op streams
/// carry, before any word exists.
///
/// **The evidence note for this opcode and its operand roles is on
/// [`encode_bdnz`] directly above, and stays there.** This function adds
/// nothing to it but the absence of the final `.word()`.
#[inline(always)]
pub fn mop_bdnz(disp: i32) -> Option<MachineOp> {
    if disp % 4 != 0 || !(-BC_MAX_DISP - 4..=BC_MAX_DISP).contains(&disp) {
        return None;
    }
    Some(MachineOp::new(op::BDNZ).disp(disp))
}

/// `lwz rD, D(rA)` — load a 32-bit word: primary opcode 32.
///
/// The constants are transcribed from raw captures rather than derived:
/// `int f(int* p){return *p;}` is `80630000`, `int f(int a,int* p){return *p;}`
/// is `80640000`, `s->d` (offset 16) is `80630010`, `p[-1]` is `8063fffc` and
/// `p[8000]` is `80637d00`. See `docs/IL_EXPR_LAYER.md` §3.
pub fn encode_lwz(rd: u8, ra: u8, d: i16) -> [u8; 4] {
    mop_lwz(rd, ra, d).word()
}

/// The [`MachineOp`] form of [`encode_lwz`] — the value S1c's op streams
/// carry, before any word exists.
///
/// **The evidence note for this opcode and its operand roles is on
/// [`encode_lwz`] directly above, and stays there.** This function adds
/// nothing to it but the absence of the final `.word()`.
#[inline(always)]
pub const fn mop_lwz(rd: u8, ra: u8, d: i16) -> MachineOp {
    MachineOp::new(op::LWZ).s(rd).d0(ra).disp(d as i32)
}

/// `lbz rD, D(rA)` — load a zero-extended byte: primary opcode 34. Transcribed
/// from captures: `char f(char* p){return *p;}` is `88630000`, `s->c` at offset 4
/// is `88630004`, and the r11 target an `extsb` consumes is `89630000`.
pub fn encode_lbz(rd: u8, ra: u8, d: i16) -> [u8; 4] {
    mop_lbz(rd, ra, d).word()
}

/// The [`MachineOp`] form of [`encode_lbz`] — the value S1c's op streams
/// carry, before any word exists.
///
/// **The evidence note for this opcode and its operand roles is on
/// [`encode_lbz`] directly above, and stays there.** This function adds
/// nothing to it but the absence of the final `.word()`.
#[inline(always)]
pub fn mop_lbz(rd: u8, ra: u8, d: i16) -> MachineOp {
    MachineOp::new(op::LBZ).s(rd).d0(ra).disp(d as i32)
}

/// `lhz rD, D(rA)` — load a zero-extended halfword: primary opcode 40.
/// Captured: `short f(short* p){return *p;}` is `a0630000` (**never `lha`** —
/// see [`indirect_load_text`]), `s->h` at offset 6 is `a0630006`.
pub fn encode_lhz(rd: u8, ra: u8, d: i16) -> [u8; 4] {
    mop_lhz(rd, ra, d).word()
}

/// The [`MachineOp`] form of [`encode_lhz`] — the value S1c's op streams
/// carry, before any word exists.
///
/// **The evidence note for this opcode and its operand roles is on
/// [`encode_lhz`] directly above, and stays there.** This function adds
/// nothing to it but the absence of the final `.word()`.
#[inline(always)]
pub fn mop_lhz(rd: u8, ra: u8, d: i16) -> MachineOp {
    MachineOp::new(op::LHZ).s(rd).d0(ra).disp(d as i32)
}

/// `ld rD, DS(rA)` — load a doubleword: primary opcode 58, **DS-form**. The low
/// two bits of the 16-bit field are the form selector (0 for `ld`), so the
/// displacement is only representable when it is a multiple of 4; callers gate on
/// that rather than letting it round. Captured: `long long f(long long* p)` is
/// `e8630000`, `s->q` at offset 16 is `e8630010`.
pub fn encode_ld(rd: u8, ra: u8, ds: i16) -> [u8; 4] {
    mop_ld(rd, ra, ds).word()
}

/// The [`MachineOp`] form of [`encode_ld`] — the value S1c's op streams
/// carry, before any word exists.
///
/// **The evidence note for this opcode and its operand roles is on
/// [`encode_ld`] directly above, and stays there.** This function adds
/// nothing to it but the absence of the final `.word()`.
#[inline(always)]
pub fn mop_ld(rd: u8, ra: u8, ds: i16) -> MachineOp {
    MachineOp::new(op::LD).s(rd).d0(ra).disp(ds as i32)
}

/// `extsb rA, rS` — sign-extend byte: opcode 31, XO 954. Captured as
/// `7d630774` = `extsb r3,r11` (the r11-then-r3 rule; see
/// [`indirect_load_text`]).
pub fn encode_extsb(ra: u8, rs: u8) -> [u8; 4] {
    mop_extsb(ra, rs).word()
}

/// The [`MachineOp`] form of [`encode_extsb`] — the value S1c's op streams
/// carry, before any word exists.
///
/// **The evidence note for this opcode and its operand roles is on
/// [`encode_extsb`] directly above, and stays there.** This function adds
/// nothing to it but the absence of the final `.word()`.
#[inline(always)]
pub fn mop_extsb(ra: u8, rs: u8) -> MachineOp {
    MachineOp::new(op::EXTSB).s(ra).d0(rs)
}

/// `extsb. rA, rS` — the **record form** of the byte sign-extension, opcode 31
/// XO 954 with `Rc = 1`. It writes **cr0** as a side effect, which is how `c2`
/// closes a **signed**-element sentinel walk: the character the next iteration
/// tests is widened and tested in one instruction, with no `cmplwi` at the
/// bottom of the body at all.
///
/// Captured: `7d6b0775` = `extsb. r11,r11` and `7d2b0775` = `extsb. r11,r9`
/// (`work/w-varloop/probe.py`, every `const char*` cell).
///
/// The signed sibling of [`encode_mr_record`], and a separate function rather
/// than an `rc: bool` on [`encode_extsb`] for the same reason that one is: the
/// two differ in whether a branch may read cr0 after them, and board #188 is
/// what this project already paid for confusing that.
pub fn encode_extsb_record(ra: u8, rs: u8) -> [u8; 4] {
    mop_extsb_record(ra, rs).word()
}

/// The [`MachineOp`] form of [`encode_extsb_record`] — the value S1c's op streams
/// carry, before any word exists.
///
/// **The evidence note for this opcode and its operand roles is on
/// [`encode_extsb_record`] directly above, and stays there.** This function adds
/// nothing to it but the absence of the final `.word()`.
#[inline(always)]
pub fn mop_extsb_record(ra: u8, rs: u8) -> MachineOp {
    MachineOp::new(op::EXTSB_RC).s(ra).d0(rs)
}

/// `extsh rA, rS` — sign-extend halfword: opcode 31, XO 922. Captured as
/// `7d630734` = `extsh r3,r11`. Emitted by no shape the port accepts today: the
/// one construct that produces it (`int f(short* p){return *p;}` under `/Ox`) is
/// refused because the same source is one `lha` under `/O1`, and this path has no
/// mode parameter. Kept, with its pinning test, because the *encoder* is measured
/// and the missing piece is the mode plumbing, not the word.
pub fn encode_extsh(ra: u8, rs: u8) -> [u8; 4] {
    mop_extsh(ra, rs).word()
}

/// The [`MachineOp`] form of [`encode_extsh`] — the value S1c's op streams
/// carry, before any word exists.
///
/// **The evidence note for this opcode and its operand roles is on
/// [`encode_extsh`] directly above, and stays there.** This function adds
/// nothing to it but the absence of the final `.word()`.
#[inline(always)]
pub fn mop_extsh(ra: u8, rs: u8) -> MachineOp {
    MachineOp::new(op::EXTSH).s(ra).d0(rs)
}

/// `stw rS, D(rA)` — store a 32-bit word: primary opcode 36.
///
/// Transcribed from captures (`work/lf/probes/p1.cpp`), not derived:
/// `void f(S* s,int v){ s->a = v; }` is `90830000`, `s->b` (offset 4) is
/// `90830004`, `s->arr[2]` (offset 48) is `90830030`, and
/// `void f(int x,S* s,int v){ s->b = v; }` is `90a40004` — value r5, base r4.
pub fn encode_stw(rs: u8, ra: u8, d: i16) -> [u8; 4] {
    mop_stw(rs, ra, d).word()
}

/// The [`MachineOp`] form of [`encode_stw`] — the value S1c's op streams
/// carry, before any word exists.
///
/// **The evidence note for this opcode and its operand roles is on
/// [`encode_stw`] directly above, and stays there.** This function adds
/// nothing to it but the absence of the final `.word()`.
#[inline(always)]
pub const fn mop_stw(rs: u8, ra: u8, d: i16) -> MachineOp {
    MachineOp::new(op::STW).d0(rs).s(ra).disp(d as i32)
}

/// `stb rS, D(rA)` — store a byte: primary opcode 38. Captured: a `char` member
/// at offset 12 is `9883000c`, an `unsigned char` at 16 is `98830010`, a `bool`
/// at 56 is `98830038`, and the literal form's `stb r11` is `99630000`.
pub fn encode_stb(rs: u8, ra: u8, d: i16) -> [u8; 4] {
    mop_stb(rs, ra, d).word()
}

/// The [`MachineOp`] form of [`encode_stb`] — the value S1c's op streams
/// carry, before any word exists.
///
/// **The evidence note for this opcode and its operand roles is on
/// [`encode_stb`] directly above, and stays there.** This function adds
/// nothing to it but the absence of the final `.word()`.
#[inline(always)]
pub fn mop_stb(rs: u8, ra: u8, d: i16) -> MachineOp {
    MachineOp::new(op::STB).d0(rs).s(ra).disp(d as i32)
}

/// `sth rS, D(rA)` — store a halfword: primary opcode 44. Captured: a `short`
/// member at offset 14 is `b083000e`.
pub fn encode_sth(rs: u8, ra: u8, d: i16) -> [u8; 4] {
    mop_sth(rs, ra, d).word()
}

/// The [`MachineOp`] form of [`encode_sth`] — the value S1c's op streams
/// carry, before any word exists.
///
/// **The evidence note for this opcode and its operand roles is on
/// [`encode_sth`] directly above, and stays there.** This function adds
/// nothing to it but the absence of the final `.word()`.
#[inline(always)]
pub fn mop_sth(rs: u8, ra: u8, d: i16) -> MachineOp {
    MachineOp::new(op::STH).d0(rs).s(ra).disp(d as i32)
}

/// `sthu rS, D(rA)` — store a halfword **with update**: primary opcode 45, and
/// `rA` is written back to the effective address.
///
/// Pinned to a byte real `c2` emitted rather than to a manual's bit layout:
/// `sthu r9,2(r4)` at `+0xa8` of `?GetBuffer@JsonWriter@@QAAJPAGPAK@Z` is
/// `b5240002` (`work/w-json/probe/ref.obj`). It is one bit away from
/// [`encode_sth`] — primary 45 against 44 — and that bit is a pointer bump the
/// caller then must not emit itself, which is why the two are separate
/// functions and the test names both words.
pub fn encode_sthu(rs: u8, ra: u8, d: i16) -> [u8; 4] {
    mop_sthu(rs, ra, d).word()
}

/// The [`MachineOp`] form of [`encode_sthu`] — the value S1c's op streams
/// carry, before any word exists.
///
/// **The evidence note for this opcode and its operand roles is on
/// [`encode_sthu`] directly above, and stays there.** This function adds
/// nothing to it but the absence of the final `.word()`.
#[inline(always)]
pub fn mop_sthu(rs: u8, ra: u8, d: i16) -> MachineOp {
    MachineOp::new(op::STHU).d0(rs).s(ra).disp(d as i32)
}

/// `lhzx rD, rA, rB` — indexed zero-extending halfword load: primary 31,
/// extended 279.
///
/// Pinned to a byte real `c2` emitted: `lhzx r11,r11,r6` at `+0x4c` of
/// `?GetBuffer@JsonWriter@@QAAJPAGPAK@Z` is `7d6b322e`. The neighbouring
/// [`encode_lwzx`] is extended **23** and [`encode_lhz`] is a different form
/// entirely, so this is a third cell and not a parameterization of either.
pub fn encode_lhzx(rd: u8, ra: u8, rb: u8) -> [u8; 4] {
    mop_lhzx(rd, ra, rb).word()
}

/// The [`MachineOp`] form of [`encode_lhzx`] — the value S1c's op streams
/// carry, before any word exists.
///
/// **The evidence note for this opcode and its operand roles is on
/// [`encode_lhzx`] directly above, and stays there.** This function adds
/// nothing to it but the absence of the final `.word()`.
#[inline(always)]
pub fn mop_lhzx(rd: u8, ra: u8, rb: u8) -> MachineOp {
    MachineOp::new(op::LHZX).s(rd).d0(ra).d1(rb)
}

// ---- W6: comparison → boolean materialization encoders ---------------------
//
// c2 materializes integer comparisons **branchlessly** — it emits no
// `cmpw`/`cmplw` at all for a `return a <rel> k` leaf, but instead carry-bit and
// bit-extraction idioms (see docs/CODEGEN_W6_COMPARE.md, where every word below
// is matched against a live capture). Several of these are non-commutative and
// their operand order is load-bearing exactly like [`encode_subf`]'s.

/// `addic rD, rA, SIMM` (rD = rA + SIMM, **setting CA**): primary opcode 12.
/// The carry-out is the point: `addic rD,rX,-1` sets CA iff `rX != 0`.
pub fn encode_addic(rd: u8, ra: u8, si: i16) -> [u8; 4] {
    mop_addic(rd, ra, si).word()
}

/// The [`MachineOp`] form of [`encode_addic`] — the value S1c's op streams
/// carry, before any word exists.
///
/// **The evidence note for this opcode and its operand roles is on
/// [`encode_addic`] directly above, and stays there.** This function adds
/// nothing to it but the absence of the final `.word()`.
#[inline(always)]
pub fn mop_addic(rd: u8, ra: u8, si: i16) -> MachineOp {
    MachineOp::new(op::ADDIC).s(rd).d0(ra).disp(si as i32)
}

/// `subfic rD, rA, SIMM` (rD = SIMM − rA, setting CA): primary opcode 8.
/// **Non-commutative**: the immediate is the minuend, the register the
/// subtrahend. CA is set iff `rA <= SIMM` unsigned.
pub fn encode_subfic(rd: u8, ra: u8, si: i16) -> [u8; 4] {
    mop_subfic(rd, ra, si).word()
}

/// The [`MachineOp`] form of [`encode_subfic`] — the value S1c's op streams
/// carry, before any word exists.
///
/// **The evidence note for this opcode and its operand roles is on
/// [`encode_subfic`] directly above, and stays there.** This function adds
/// nothing to it but the absence of the final `.word()`.
#[inline(always)]
pub fn mop_subfic(rd: u8, ra: u8, si: i16) -> MachineOp {
    MachineOp::new(op::SUBFIC).s(rd).d0(ra).disp(si as i32)
}

/// `subfc rD, rA, rB` (rD = rB − rA, setting CA): opcode 31, XO 8.
/// **Non-commutative — same reversed mapping as [`encode_subf`]**: to realize
/// `lhs − rhs` pass `ra = rhs`, `rb = lhs`.
pub fn encode_subfc(rd: u8, ra: u8, rb: u8) -> [u8; 4] {
    mop_subfc(rd, ra, rb).word()
}

/// The [`MachineOp`] form of [`encode_subfc`] — the value S1c's op streams
/// carry, before any word exists.
///
/// **The evidence note for this opcode and its operand roles is on
/// [`encode_subfc`] directly above, and stays there.** This function adds
/// nothing to it but the absence of the final `.word()`.
#[inline(always)]
pub fn mop_subfc(rd: u8, ra: u8, rb: u8) -> MachineOp {
    MachineOp::new(op::SUBFC).s(rd).d0(ra).d1(rb)
}

/// `subfe rD, rA, rB` (rD = ¬rA + rB + CA): opcode 31, XO 136.
/// **Non-commutative.** With `rA == rB` the register terms cancel to −1, so the
/// result is `CA − 1` — the don't-care-source idiom (§3.5 of the W6 doc), where
/// the source register number is still byte-visible and must be reproduced.
pub fn encode_subfe(rd: u8, ra: u8, rb: u8) -> [u8; 4] {
    mop_subfe(rd, ra, rb).word()
}

/// The [`MachineOp`] form of [`encode_subfe`] — the value S1c's op streams
/// carry, before any word exists.
///
/// **The evidence note for this opcode and its operand roles is on
/// [`encode_subfe`] directly above, and stays there.** This function adds
/// nothing to it but the absence of the final `.word()`.
#[inline(always)]
pub fn mop_subfe(rd: u8, ra: u8, rb: u8) -> MachineOp {
    MachineOp::new(op::SUBFE).s(rd).d0(ra).d1(rb)
}

/// `addze rD, rA` (rD = rA + CA): opcode 31, XO 202.
pub fn encode_addze(rd: u8, ra: u8) -> [u8; 4] {
    mop_addze(rd, ra).word()
}

/// The [`MachineOp`] form of [`encode_addze`] — the value S1c's op streams
/// carry, before any word exists.
///
/// **The evidence note for this opcode and its operand roles is on
/// [`encode_addze`] directly above, and stays there.** This function adds
/// nothing to it but the absence of the final `.word()`.
#[inline(always)]
pub fn mop_addze(rd: u8, ra: u8) -> MachineOp {
    MachineOp::new(op::ADDZE).s(rd).d0(ra)
}

/// `adde rD, rA, rB` (rD = rA + rB + CA): opcode 31, XO 138. The two-sided
/// counterpart of [`encode_addze`], used by the signed `>=`/`<=` spines to add
/// the two sign terms and the borrow in one instruction.
pub fn encode_adde(rd: u8, ra: u8, rb: u8) -> [u8; 4] {
    mop_adde(rd, ra, rb).word()
}

/// The [`MachineOp`] form of [`encode_adde`] — the value S1c's op streams
/// carry, before any word exists.
///
/// **The evidence note for this opcode and its operand roles is on
/// [`encode_adde`] directly above, and stays there.** This function adds
/// nothing to it but the absence of the final `.word()`.
#[inline(always)]
pub fn mop_adde(rd: u8, ra: u8, rb: u8) -> MachineOp {
    MachineOp::new(op::ADDE).s(rd).d0(ra).d1(rb)
}

/// `subfze rD, rA` (rD = ~rA + CA): opcode 31, XO 200. Against a preloaded
/// `rA = -1` this is exactly "materialize CA", which is how the unsigned
/// `>=`/`<=` spines turn a borrow into a 0/1 boolean.
pub fn encode_subfze(rd: u8, ra: u8) -> [u8; 4] {
    mop_subfze(rd, ra).word()
}

/// The [`MachineOp`] form of [`encode_subfze`] — the value S1c's op streams
/// carry, before any word exists.
///
/// **The evidence note for this opcode and its operand roles is on
/// [`encode_subfze`] directly above, and stays there.** This function adds
/// nothing to it but the absence of the final `.word()`.
#[inline(always)]
pub fn mop_subfze(rd: u8, ra: u8) -> MachineOp {
    MachineOp::new(op::SUBFZE).s(rd).d0(ra)
}

/// `srawi rA, rS, SH` (arithmetic shift right immediate, setting CA): opcode 31,
/// XO 824. At `SH = 31` this broadcasts the sign bit, giving 0 or −1 — the
/// signed relational spines' "sign of the operand" term. Note this is *not*
/// [`encode_srwi31`], which yields 0 or 1 via `rlwinm`; the signed `>=`/`<=`
/// spines use one of each and the pair is not interchangeable.
pub fn encode_srawi(ra: u8, rs: u8, sh: u8) -> [u8; 4] {
    mop_srawi(ra, rs, sh).word()
}

/// The [`MachineOp`] form of [`encode_srawi`] — the value S1c's op streams
/// carry, before any word exists.
///
/// **The evidence note for this opcode and its operand roles is on
/// [`encode_srawi`] directly above, and stays there.** This function adds
/// nothing to it but the absence of the final `.word()`.
#[inline(always)]
pub fn mop_srawi(ra: u8, rs: u8, sh: u8) -> MachineOp {
    MachineOp::new(op::SRAWI).s(ra).d0(rs).d1(sh)
}

/// `neg rD, rA` (rD = −rA): opcode 31, XO 104.
pub fn encode_neg(rd: u8, ra: u8) -> [u8; 4] {
    mop_neg(rd, ra).word()
}

/// The [`MachineOp`] form of [`encode_neg`] — the value S1c's op streams
/// carry, before any word exists.
///
/// **The evidence note for this opcode and its operand roles is on
/// [`encode_neg`] directly above, and stays there.** This function adds
/// nothing to it but the absence of the final `.word()`.
#[inline(always)]
pub fn mop_neg(rd: u8, ra: u8) -> MachineOp {
    MachineOp::new(op::NEG).s(rd).d0(ra)
}

/// `andc rA, rS, rB` (rA = rS & ¬rB): opcode 31, XO 60. Not symmetric in
/// rS/rB — the complement applies to rB only.
pub fn encode_andc(ra: u8, rs: u8, rb: u8) -> [u8; 4] {
    mop_andc(ra, rs, rb).word()
}

/// The [`MachineOp`] form of [`encode_andc`] — the value S1c's op streams
/// carry, before any word exists.
///
/// **The evidence note for this opcode and its operand roles is on
/// [`encode_andc`] directly above, and stays there.** This function adds
/// nothing to it but the absence of the final `.word()`.
#[inline(always)]
pub fn mop_andc(ra: u8, rs: u8, rb: u8) -> MachineOp {
    MachineOp::new(op::ANDC).s(ra).d0(rs).d1(rb)
}

/// `orc rA, rS, rB` (rA = rS | ¬rB): opcode 31, XO 412. Not symmetric.
pub fn encode_orc(ra: u8, rs: u8, rb: u8) -> [u8; 4] {
    mop_orc(ra, rs, rb).word()
}

/// The [`MachineOp`] form of [`encode_orc`] — the value S1c's op streams
/// carry, before any word exists.
///
/// **The evidence note for this opcode and its operand roles is on
/// [`encode_orc`] directly above, and stays there.** This function adds
/// nothing to it but the absence of the final `.word()`.
#[inline(always)]
pub fn mop_orc(ra: u8, rs: u8, rb: u8) -> MachineOp {
    MachineOp::new(op::ORC).s(ra).d0(rs).d1(rb)
}

/// `eqv rA, rS, rB` (rA = ¬(rS ^ rB)): opcode 31, XO 284. Logically symmetric,
/// but c2's emitted rS/rB order is reproduced rather than chosen.
pub fn encode_eqv(ra: u8, rs: u8, rb: u8) -> [u8; 4] {
    mop_eqv(ra, rs, rb).word()
}

/// The [`MachineOp`] form of [`encode_eqv`] — the value S1c's op streams
/// carry, before any word exists.
///
/// **The evidence note for this opcode and its operand roles is on
/// [`encode_eqv`] directly above, and stays there.** This function adds
/// nothing to it but the absence of the final `.word()`.
#[inline(always)]
pub fn mop_eqv(ra: u8, rs: u8, rb: u8) -> MachineOp {
    MachineOp::new(op::EQV).s(ra).d0(rs).d1(rb)
}

/// `cntlzw rA, rS` (count leading zero bits): opcode 31, XO 26. Yields exactly
/// 32 iff rS is zero — the basis of the `== 0` idiom.
pub fn encode_cntlzw(ra: u8, rs: u8) -> [u8; 4] {
    mop_cntlzw(ra, rs).word()
}

/// The [`MachineOp`] form of [`encode_cntlzw`] — the value S1c's op streams
/// carry, before any word exists.
///
/// **The evidence note for this opcode and its operand roles is on
/// [`encode_cntlzw`] directly above, and stays there.** This function adds
/// nothing to it but the absence of the final `.word()`.
#[inline(always)]
pub fn mop_cntlzw(ra: u8, rs: u8) -> MachineOp {
    MachineOp::new(op::CNTLZW).s(ra).d0(rs)
}

/// `xori rA, rS, UIMM` (rA = rS ^ UIMM): primary opcode 26.
pub fn encode_xori(ra: u8, rs: u8, ui: u16) -> [u8; 4] {
    mop_xori(ra, rs, ui).word()
}

/// The [`MachineOp`] form of [`encode_xori`] — the value S1c's op streams
/// carry, before any word exists.
///
/// **The evidence note for this opcode and its operand roles is on
/// [`encode_xori`] directly above, and stays there.** This function adds
/// nothing to it but the absence of the final `.word()`.
#[inline(always)]
pub fn mop_xori(ra: u8, rs: u8, ui: u16) -> MachineOp {
    MachineOp::new(op::XORI).s(ra).d0(rs).imm_d1(ui as u32)
}

/// `rlwinm rA, rS, SH, MB, ME` — rotate left word immediate then AND with mask:
/// primary opcode 21, Rc=0. The workhorse of bit extraction here.
pub fn encode_rlwinm(ra: u8, rs: u8, sh: u8, mb: u8, me: u8) -> [u8; 4] {
    mop_rlwinm(ra, rs, sh, mb, me).word()
}

/// The [`MachineOp`] form of [`encode_rlwinm`] — the value S1c's op streams
/// carry, before any word exists.
///
/// **The evidence note for this opcode and its operand roles is on
/// [`encode_rlwinm`] directly above, and stays there.** This function adds
/// nothing to it but the absence of the final `.word()`.
#[inline(always)]
pub fn mop_rlwinm(ra: u8, rs: u8, sh: u8, mb: u8, me: u8) -> MachineOp {
    MachineOp::new(op::RLWINM).s(ra).d0(rs).d1(sh).d2(mb).d3(me)
}

/// `rlwimi rA, rS, SH, MB, ME` — rotate left word immediate then mask
/// **INSERT**: primary opcode 20, Rc=0. Unlike [`encode_rlwinm`] this reads
/// `rA` as well as writing it — the bits outside `MB..ME` survive, which is the
/// whole point and the reason W43 can fold a shift and an OR into one word.
pub fn encode_rlwimi(ra: u8, rs: u8, sh: u8, mb: u8, me: u8) -> [u8; 4] {
    mop_rlwimi(ra, rs, sh, mb, me).word()
}

/// The [`MachineOp`] form of [`encode_rlwimi`] — the value S1c's op streams
/// carry, before any word exists.
///
/// **The evidence note for this opcode and its operand roles is on
/// [`encode_rlwimi`] directly above, and stays there.** This function adds
/// nothing to it but the absence of the final `.word()`.
#[inline(always)]
pub fn mop_rlwimi(ra: u8, rs: u8, sh: u8, mb: u8, me: u8) -> MachineOp {
    MachineOp::new(op::RLWIMI).s(ra).d0(rs).d1(sh).d2(mb).d3(me)
}

/// `rldicl rA, rS, SH, MB` — **rotate left DOUBLEWORD immediate then clear
/// left**: primary opcode 30, extended opcode 0, Rc=0. The first 64-bit
/// rotate/mask encoder in this file — board **#2344** recorded that there was
/// none anywhere in `c2-core`.
///
/// The two immediate fields are **split**, which is the whole reason this is not
/// `encode_rlwinm` with a wider mask:
///
/// * `SH` is six bits: `SH[4:0]` at bits 16..21 and `SH[5]` alone at **bit 30**;
/// * `MB` is six bits stored **low-bit-first**: `MB[4:0]` at bits 21..26 and
///   `MB[5]` at bit 26 — i.e. the field is `(MB & 0x1F) << 1 | (MB >> 5)`.
///
/// Both are read off the target obj rather than off a manual:
/// `?SetNonce@XTEABlockEncrypter`'s `78ab0020` decodes as `rA=11, rS=5, SH=0,
/// MB=32` (`clrldi r11,r5,32`, the zero-extension of a 32-bit value) and
/// `?Encipher@`'s `78890022` as `rA=9, rS=4, SH=32, MB=32` (`srdi r9,r4,32`) —
/// so the two cells separate the `SH[5]` bit from the `MB[5]` bit, which a
/// single witness could not.
pub fn encode_rldicl(ra: u8, rs: u8, sh: u8, mb: u8) -> [u8; 4] {
    mop_rldicl(ra, rs, sh, mb).word()
}

/// The [`MachineOp`] form of [`encode_rldicl`] — the value S1c's op streams
/// carry, before any word exists.
///
/// **The evidence note for this opcode and its operand roles is on
/// [`encode_rldicl`] directly above, and stays there.** This function adds
/// nothing to it but the absence of the final `.word()`.
#[inline(always)]
pub fn mop_rldicl(ra: u8, rs: u8, sh: u8, mb: u8) -> MachineOp {
    MachineOp::new(op::RLDICL).s(ra).d0(rs).imm_d1(sh as u32).imm_d2(mb as u32)
}

/// `rldimi rA, rS, SH, MB` — **rotate left DOUBLEWORD immediate then mask
/// INSERT**: primary opcode 30, extended opcode 3, Rc=0. Unlike
/// [`encode_rldicl`] this reads `rA` as well as writing it — the bits outside
/// the mask survive, which is what lets one word splice a 32-bit value into the
/// high half of a register that already holds the low half.
///
/// The two split immediate fields are [`encode_rldicl`]'s, and the only
/// difference is the extended opcode. Read off `?Encipher@XTEABlockEncrypter`'s
/// `7923000e`, which decodes as `rA=3, rS=9, SH=32, MB=0`.
pub fn encode_rldimi(ra: u8, rs: u8, sh: u8, mb: u8) -> [u8; 4] {
    mop_rldimi(ra, rs, sh, mb).word()
}

/// The [`MachineOp`] form of [`encode_rldimi`] — the value S1c's op streams
/// carry, before any word exists.
///
/// **The evidence note for this opcode and its operand roles is on
/// [`encode_rldimi`] directly above, and stays there.** This function adds
/// nothing to it but the absence of the final `.word()`.
#[inline(always)]
pub fn mop_rldimi(ra: u8, rs: u8, sh: u8, mb: u8) -> MachineOp {
    MachineOp::new(op::RLDIMI).s(ra).d0(rs).imm_d1(sh as u32).imm_d2(mb as u32)
}

/// `stdu rS, DS(rA)` — **store doubleword with update**: primary opcode 62,
/// XO = 1, and the displacement is a **DS** field (14 bits, the low two implied
/// zero). One word for two facts — the store and the base's post-increment — and
/// board **#2567** recorded it as missing.
///
/// Read off `?Encrypt@XTEABlockEncrypter`'s `f97e0009`: `rS=11, rA=30, DS=8`.
pub fn encode_stdu(rs: u8, ra: u8, ds: i16) -> [u8; 4] {
    mop_stdu(rs, ra, ds).word()
}

/// The [`MachineOp`] form of [`encode_stdu`] — the value S1c's op streams
/// carry, before any word exists.
///
/// **The evidence note for this opcode and its operand roles is on
/// [`encode_stdu`] directly above, and stays there.** This function adds
/// nothing to it but the absence of the final `.word()`.
#[inline(always)]
pub fn mop_stdu(rs: u8, ra: u8, ds: i16) -> MachineOp {
    debug_assert_eq!(ds & 3, 0, "a DS displacement's low two bits are implied zero");
    MachineOp::new(op::STDU).d0(rs).s(ra).disp(ds as i32)
}

/// `stdx rS, rA, rB` — **store doubleword indexed**: primary opcode 31,
/// XO = 149, Rc = 0. Board **#2567** recorded it as missing.
///
/// Read off `?Encrypt@XTEABlockEncrypter`'s `7d7af92a`: `rS=11, rA=26, rB=31`.
pub fn encode_stdx(rs: u8, ra: u8, rb: u8) -> [u8; 4] {
    mop_stdx(rs, ra, rb).word()
}

/// The [`MachineOp`] form of [`encode_stdx`] — the value S1c's op streams
/// carry, before any word exists.
///
/// **The evidence note for this opcode and its operand roles is on
/// [`encode_stdx`] directly above, and stays there.** This function adds
/// nothing to it but the absence of the final `.word()`.
#[inline(always)]
pub fn mop_stdx(rs: u8, ra: u8, rb: u8) -> MachineOp {
    MachineOp::new(op::STDX).d0(rs).s(ra).d1(rb)
}

/// `addic. rD, rA, SI` — **add immediate carrying, RECORD form**: primary
/// opcode **13**, against [`encode_addic`]'s 12. One word for two facts, the
/// decrement and the cr0 test, and board **#2567** recorded the record form as
/// missing while the plain one was present.
///
/// Read off `?Encrypt@XTEABlockEncrypter`'s `37bdffff`: `rD=29, rA=29, SI=-1`.
pub fn encode_addic_record(rd: u8, ra: u8, si: i16) -> [u8; 4] {
    mop_addic_record(rd, ra, si).word()
}

/// The [`MachineOp`] form of [`encode_addic_record`] — the value S1c's op streams
/// carry, before any word exists.
///
/// **The evidence note for this opcode and its operand roles is on
/// [`encode_addic_record`] directly above, and stays there.** This function adds
/// nothing to it but the absence of the final `.word()`.
#[inline(always)]
pub fn mop_addic_record(rd: u8, ra: u8, si: i16) -> MachineOp {
    MachineOp::new(op::ADDIC_RC).s(rd).d0(ra).disp(si as i32)
}

/// `srwi rA, rS, 31` — extract the sign bit. The `rlwinm rA,rS,1,31,31` form.
pub fn encode_srwi31(ra: u8, rs: u8) -> [u8; 4] {
    mop_srwi31(ra, rs).word()
}

/// The [`MachineOp`] form of [`encode_srwi31`] — the value S1c's op streams
/// carry, before any word exists.
///
/// **The evidence note for this opcode and its operand roles is on
/// [`encode_srwi31`] directly above, and stays there.** This function adds
/// nothing to it but the absence of the final `.word()`.
#[inline(always)]
pub fn mop_srwi31(ra: u8, rs: u8) -> MachineOp {
    mop_rlwinm(ra, rs, 1, 31, 31)
}

/// `clrlwi rA, rS, 31` — keep only bit 31. The `rlwinm rA,rS,0,31,31` form.
pub fn encode_clrlwi31(ra: u8, rs: u8) -> [u8; 4] {
    mop_clrlwi31(ra, rs).word()
}

/// The [`MachineOp`] form of [`encode_clrlwi31`] — the value S1c's op streams
/// carry, before any word exists.
///
/// **The evidence note for this opcode and its operand roles is on
/// [`encode_clrlwi31`] directly above, and stays there.** This function adds
/// nothing to it but the absence of the final `.word()`.
#[inline(always)]
pub fn mop_clrlwi31(ra: u8, rs: u8) -> MachineOp {
    mop_rlwinm(ra, rs, 0, 31, 31)
}

/// `rlwinm. rA, rS, SH, MB, ME` — the **record form** of [`encode_rlwinm`]:
/// primary opcode 21 with `Rc = 1`, so the masked result sets `cr0` and **no
/// compare instruction is issued at all**.
///
/// That last clause is the whole reason this encoder exists as its own name
/// rather than as an `rc: bool` parameter on [`encode_rlwinm`]. A caller that
/// reaches for the record form is making a *control-flow* decision — the branch
/// below it reads `cr0` — where a caller of the non-record form is computing a
/// value. Two names keep the two decisions apart at the call site.
///
/// **Pinned to real `c2` output, not derived from a manual.**
/// `clrlwi. r10,r10,31` at offset `0x48` of `_free_osfhnd` is `554a07ff`
/// (`work/w-osfinfo/ref/osfinfo/dis.txt`, the workload's own
/// `/O1 /Oi /EHsc /GR`), and `codegen::osf_handle_guard` asserts that word in
/// place against the whole 152-byte function.
pub fn encode_rlwinm_record(ra: u8, rs: u8, sh: u8, mb: u8, me: u8) -> [u8; 4] {
    mop_rlwinm_record(ra, rs, sh, mb, me).word()
}

/// The [`MachineOp`] form of [`encode_rlwinm_record`] — the value S1c's op streams
/// carry, before any word exists.
///
/// **The evidence note for this opcode and its operand roles is on
/// [`encode_rlwinm_record`] directly above, and stays there.** This function adds
/// nothing to it but the absence of the final `.word()`.
#[inline(always)]
pub fn mop_rlwinm_record(ra: u8, rs: u8, sh: u8, mb: u8, me: u8) -> MachineOp {
    MachineOp::new(op::RLWINM_RC).s(ra).d0(rs).d1(sh).d2(mb).d3(me)
}

/// `clrlwi. rA, rS, N` — keep the low `32 − N` bits **and set cr0**. The
/// `rlwinm. rA,rS,0,N,31` form.
///
/// `N = 31` is the `& 1` test `_free_osfhnd` uses on its `osfile` byte; the
/// parameter is open because the reader derives it from the mask literal the IL
/// carries (`mask + 1` a power of two ⇒ `N = 32 − log2(mask + 1)`), and a class
/// that hardcoded 31 would have a field it could not vary.
pub fn encode_clrlwi_record(ra: u8, rs: u8, n: u8) -> [u8; 4] {
    mop_clrlwi_record(ra, rs, n).word()
}

/// The [`MachineOp`] form of [`encode_clrlwi_record`] — the value S1c's op streams
/// carry, before any word exists.
///
/// **The evidence note for this opcode and its operand roles is on
/// [`encode_clrlwi_record`] directly above, and stays there.** This function adds
/// nothing to it but the absence of the final `.word()`.
#[inline(always)]
pub fn mop_clrlwi_record(ra: u8, rs: u8, n: u8) -> MachineOp {
    mop_rlwinm_record(ra, rs, 0, n, 31)
}



/// `fadds`/`fadd` — XO 21. Commutative.
pub fn encode_fadd(double: bool, fd: u8, fa: u8, fb: u8) -> [u8; 4] {
    mop_fadd(double, fd, fa, fb).word()
}

/// The [`MachineOp`] form of [`encode_fadd`] — the value S1c's op streams
/// carry, before any word exists.
///
/// **The evidence note for this opcode and its operand roles is on
/// [`encode_fadd`] directly above, and stays there.** This function adds
/// nothing to it but the absence of the final `.word()`.
#[inline(always)]
pub fn mop_fadd(double: bool, fd: u8, fa: u8, fb: u8) -> MachineOp {
    MachineOp::new(if double { op::FADD } else { op::FADDS }).s(fd).d0(fa).d1(fb)
}

/// `fsubs`/`fsub` — XO 20. **`fD = fA − fB`**, i.e. the operands are in source
/// order, unlike the integer [`encode_subf`]. Swapping them negates the result.
pub fn encode_fsub(double: bool, fd: u8, fa: u8, fb: u8) -> [u8; 4] {
    mop_fsub(double, fd, fa, fb).word()
}

/// The [`MachineOp`] form of [`encode_fsub`] — the value S1c's op streams
/// carry, before any word exists.
///
/// **The evidence note for this opcode and its operand roles is on
/// [`encode_fsub`] directly above, and stays there.** This function adds
/// nothing to it but the absence of the final `.word()`.
#[inline(always)]
pub fn mop_fsub(double: bool, fd: u8, fa: u8, fb: u8) -> MachineOp {
    MachineOp::new(if double { op::FSUB } else { op::FSUBS }).s(fd).d0(fa).d1(fb)
}

/// `fmuls`/`fmul` — XO 25, with the multiplier in the **C** field.
pub fn encode_fmul(double: bool, fd: u8, fa: u8, fc: u8) -> [u8; 4] {
    mop_fmul(double, fd, fa, fc).word()
}

/// The [`MachineOp`] form of [`encode_fmul`] — the value S1c's op streams
/// carry, before any word exists.
///
/// **The evidence note for this opcode and its operand roles is on
/// [`encode_fmul`] directly above, and stays there.** This function adds
/// nothing to it but the absence of the final `.word()`.
#[inline(always)]
pub fn mop_fmul(double: bool, fd: u8, fa: u8, fc: u8) -> MachineOp {
    MachineOp::new(if double { op::FMUL } else { op::FMULS }).s(fd).d0(fa).d1(fc)
}

/// `fmadds`/`fmadd` — the fused multiply-add, `fD = fA*fC + fB`. XO 29.
///
/// **The parameter order is c2's slot order, which is the MNEMONIC's order and
/// not the bit layout's**: `fc` is the second multiplicand and lands in the `C`
/// field at bit 6; `fb` is the ADDEND and lands in the `B` field at bit 11.
/// Read at arm `0x10bfa49a` (`mop::plan`'s form-24 arm, DISCLOSURE
/// `W-FMADD-1`), not inferred. Passing the addend as `fc` yields a word that
/// disassembles cleanly and computes `fA*fB + fC`.
///
/// c2 **always** contracts a `*` feeding a `+`/`-`; there is no mode in which
/// it emits `fmuls`+`fadds` for `a*b+c` (`docs/CODEGEN_W13_FLOAT.md` §3.3), so
/// this is not an optimisation the port may decline to take — declining is a
/// wrong emit.
pub fn encode_fmadd(double: bool, fd: u8, fa: u8, fc: u8, fb: u8) -> [u8; 4] {
    mop_fmadd(double, fd, fa, fc, fb).word()
}

/// The [`MachineOp`] form of [`encode_fmadd`] — the value S1c's op streams
/// carry, before any word exists.
#[inline(always)]
pub fn mop_fmadd(double: bool, fd: u8, fa: u8, fc: u8, fb: u8) -> MachineOp {
    MachineOp::new(if double { op::FMADD } else { op::FMADDS })
        .s(fd)
        .d0(fa)
        .d1(fc)
        .d2(fb)
}

/// `fmsubs`/`fmsub` — `fD = fA*fC − fB`. XO 28. Same slot order as
/// [`encode_fmadd`]; see its note.
pub fn encode_fmsub(double: bool, fd: u8, fa: u8, fc: u8, fb: u8) -> [u8; 4] {
    mop_fmsub(double, fd, fa, fc, fb).word()
}

/// The [`MachineOp`] form of [`encode_fmsub`].
#[inline(always)]
pub fn mop_fmsub(double: bool, fd: u8, fa: u8, fc: u8, fb: u8) -> MachineOp {
    MachineOp::new(if double { op::FMSUB } else { op::FMSUBS })
        .s(fd)
        .d0(fa)
        .d1(fc)
        .d2(fb)
}

/// `fnmsubs`/`fnmsub` — `fD = −(fA*fC − fB)` = **`fB − fA*fC`**. XO 30.
///
/// The negated form is what makes `c - a*b` **one** instruction with no `fneg`
/// (`docs/CODEGEN_W13_FLOAT.md` §3.3), and it is why the port must distinguish
/// which side of the `-` the product was on: product-on-the-left is
/// [`encode_fmsub`], product-on-the-right is this. Emitting `fmsub` for the
/// right-hand case negates the result.
pub fn encode_fnmsub(double: bool, fd: u8, fa: u8, fc: u8, fb: u8) -> [u8; 4] {
    mop_fnmsub(double, fd, fa, fc, fb).word()
}

/// The [`MachineOp`] form of [`encode_fnmsub`].
#[inline(always)]
pub fn mop_fnmsub(double: bool, fd: u8, fa: u8, fc: u8, fb: u8) -> MachineOp {
    MachineOp::new(if double { op::FNMSUB } else { op::FNMSUBS })
        .s(fd)
        .d0(fa)
        .d1(fc)
        .d2(fb)
}

/// `fdivs`/`fdiv` — XO 18.
pub fn encode_fdiv(double: bool, fd: u8, fa: u8, fb: u8) -> [u8; 4] {
    mop_fdiv(double, fd, fa, fb).word()
}

/// The [`MachineOp`] form of [`encode_fdiv`] — the value S1c's op streams
/// carry, before any word exists.
///
/// **The evidence note for this opcode and its operand roles is on
/// [`encode_fdiv`] directly above, and stays there.** This function adds
/// nothing to it but the absence of the final `.word()`.
#[inline(always)]
pub fn mop_fdiv(double: bool, fd: u8, fa: u8, fb: u8) -> MachineOp {
    MachineOp::new(if double { op::FDIV } else { op::FDIVS }).s(fd).d0(fa).d1(fb)
}

/// `lfsx fD, rA, rB` — load float single, **X-form indexed**: primary 31,
/// XO 535. The effective address is `rA + rB` with no displacement field at all.
///
/// **W-BLOCKIR.** This is the word the base-difference strength reduction needs
/// (`docs/whitebox/WB_LOOP_FINDINGS.md` §4.3): with two arrays walked by one
/// pointer, the array that is *not* the walker is reached at a preheader-computed
/// difference, and an X-form is the only load that can take two registers. It is
/// also — stated here because it is the reason the update form is not free — why
/// the walker cannot fold its `addi` into this access: `lfsx` has no
/// displacement to fold into.
///
/// Captured as `7c0a5c2e` = `lfsx f0, r10, r11`
/// (`work/w-blockir/ref/ipp.dis.txt`, `?Add_InPlace@IPP@@YAXIPBMPAM@Z` +0x14).
pub fn encode_lfsx(fd: u8, ra: u8, rb: u8) -> [u8; 4] {
    mop_lfsx(fd, ra, rb).word()
}

/// The [`MachineOp`] form of [`encode_lfsx`] — the value S1c's op streams
/// carry, before any word exists.
///
/// **The evidence note for this opcode and its operand roles is on
/// [`encode_lfsx`] directly above, and stays there.** This function adds
/// nothing to it but the absence of the final `.word()`.
#[inline(always)]
pub fn mop_lfsx(fd: u8, ra: u8, rb: u8) -> MachineOp {
    MachineOp::new(op::LFSX).s(fd).d0(ra).d1(rb)
}

/// `stfsx fS, rA, rB` — store float single, X-form indexed: primary 31, XO 663.
///
/// The store side of the same rule. Captured as `7c095d2e` =
/// `stfsx f0, r9, r11` (`work/w-blockir/ref/ipp.dis.txt`,
/// `?Mul@IPP@@YAXIPBM0PAM@Z` +0x24).
pub fn encode_stfsx(fs: u8, ra: u8, rb: u8) -> [u8; 4] {
    mop_stfsx(fs, ra, rb).word()
}

/// The [`MachineOp`] form of [`encode_stfsx`] — the value S1c's op streams
/// carry, before any word exists.
///
/// **The evidence note for this opcode and its operand roles is on
/// [`encode_stfsx`] directly above, and stays there.** This function adds
/// nothing to it but the absence of the final `.word()`.
#[inline(always)]
pub fn mop_stfsx(fs: u8, ra: u8, rb: u8) -> MachineOp {
    MachineOp::new(op::STFSX).d0(fs).s(ra).d1(rb)
}

/// `stfsu fS, d(rA)` — store float single **with update**: primary **53**,
/// D-form, and `rA` is written back with the effective address.
///
/// One word doing a store and the induction step, which is why c2 reaches for it
/// — and it can only be reached when *every* access on the walking pointer is
/// D-form, because the write-back moves the base out from under any X-form
/// access sharing it. That is `WB_LOOP_FINDINGS.md` §4.3's *"the base-difference
/// trick is what kills the update form"*, and it is why this encoder appears in
/// exactly one arm of `super::float_walk_loop` (the single-array shape) and
/// nowhere else.
///
/// **This is NOT `wb-loop`'s declined pass 3.** That pass is a *general* rule for
/// choosing the update form, over which four rivals were gridded and none
/// elected (`w-bdnz` board #1981). This is one word in one transcribed shape
/// with one graded witness pair (`?MulConstant_InPlace@IPP@@YAXIPAMM@Z` and
/// `probe/walk.cpp`'s `c12`, byte-identical at 36 B).
///
/// Captured as `d40b0004` = `stfsu f0, 4(r11)`
/// (`work/w-blockir/ref/ipp.dis.txt`, `?MulConstant_InPlace` +0x18).
pub fn encode_stfsu(fs: u8, ra: u8, d: i16) -> [u8; 4] {
    mop_stfsu(fs, ra, d).word()
}

/// The [`MachineOp`] form of [`encode_stfsu`] — the value S1c's op streams
/// carry, before any word exists.
///
/// **The evidence note for this opcode and its operand roles is on
/// [`encode_stfsu`] directly above, and stays there.** This function adds
/// nothing to it but the absence of the final `.word()`.
#[inline(always)]
pub fn mop_stfsu(fs: u8, ra: u8, d: i16) -> MachineOp {
    MachineOp::new(op::STFSU).d0(fs).s(ra).disp(d as i32)
}

/// `stfs fS, d(rA)` / `stfd fS, d(rA)` — store a floating-point register.
///
/// Primary **52** single, **54** double, both plain D-form. Note the asymmetry
/// with the integer family: `std` is DS-form and cannot encode a displacement
/// that is not a multiple of 4, while `stfd` owns all sixteen bits — so the
/// alignment gate `try_parse_store_leaf` applies to a `width == 8` integer store
/// deliberately has no counterpart on the FP path. Verified: `d0230004` is
/// `stfs f1,4(r3)` and `d8230008` is `stfd f1,8(r3)`
/// (`docs/CODEGEN_FP_ARGS.md` §3).
pub fn encode_stfs(double: bool, fs: u8, ra: u8, d: i16) -> [u8; 4] {
    mop_stfs(double, fs, ra, d).word()
}

/// The [`MachineOp`] form of [`encode_stfs`] — the value S1c's op streams
/// carry, before any word exists.
///
/// **The evidence note for this opcode and its operand roles is on
/// [`encode_stfs`] directly above, and stays there.** This function adds
/// nothing to it but the absence of the final `.word()`.
#[inline(always)]
pub fn mop_stfs(double: bool, fs: u8, ra: u8, d: i16) -> MachineOp {
    MachineOp::new(if double { op::STFD } else { op::STFS }).d0(fs).s(ra).disp(d as i32)
}

/// `fmr fD, fB` — the FP register move: X-form, primary **63**, XO 72.
///
/// **Primary 63 whatever the operand width.** There is no `fmrs`: the
/// single-precision A-form ops use primary 59, but a register move is a bit copy
/// and the FPRs hold double internally, so the same encoding serves `float` and
/// `double`. Captured both ways — `float t2(float a,float b){ return g1f(b); }`
/// and its `double` twin both emit `fc201090`, `fmr f1,f2`
/// (`docs/CODEGEN_FP_ARGS.md` §1) — which is why this takes no `double` flag and
/// the A-form encoders above do.
pub fn encode_fmr(fd: u8, fb: u8) -> [u8; 4] {
    mop_fmr(fd, fb).word()
}

/// The [`MachineOp`] form of [`encode_fmr`] — the value S1c's op streams
/// carry, before any word exists.
///
/// **The evidence note for this opcode and its operand roles is on
/// [`encode_fmr`] directly above, and stays there.** This function adds
/// nothing to it but the absence of the final `.word()`.
#[inline(always)]
pub fn mop_fmr(fd: u8, fb: u8) -> MachineOp {
    MachineOp::new(op::FMR).s(fd).d0(fb)
}

/// `frsp fD, fB` — round to single precision: X-form, primary 63, XO 12.
///
/// The `double` → `float` narrowing, and it is a **real instruction** where the
/// widening `float` → `double` is nothing at all. Captured as the pair, which is
/// the only way to establish that the asymmetry is c2's and not the C standard's:
/// `double wid(float a){ return gd1(a); }` is a bare `b`, while
/// `float nar(double a){ return gf1(a); }` is `fc200818 ; b` —
/// `frsp f1,f1` (`docs/CODEGEN_FP_ARGS.md` §2).
pub fn encode_frsp(fd: u8, fb: u8) -> [u8; 4] {
    mop_frsp(fd, fb).word()
}

/// The [`MachineOp`] form of [`encode_frsp`] — the value S1c's op streams
/// carry, before any word exists.
///
/// **The evidence note for this opcode and its operand roles is on
/// [`encode_frsp`] directly above, and stays there.** This function adds
/// nothing to it but the absence of the final `.word()`.
#[inline(always)]
pub fn mop_frsp(fd: u8, fb: u8) -> MachineOp {
    MachineOp::new(op::FRSP).s(fd).d0(fb)
}

/// `lfs fD, d(rA)` — load float single: primary opcode 48. The `lfd` (double)
/// form is primary 50. Both are D-form with a signed 16-bit displacement, which
/// the REFLO relocation rewrites, so `d` is emitted as 0.
pub fn encode_lfs(double: bool, fd: u8, ra: u8, d: i16) -> [u8; 4] {
    mop_lfs(double, fd, ra, d).word()
}

/// The [`MachineOp`] form of [`encode_lfs`] — the value S1c's op streams
/// carry, before any word exists.
///
/// **The evidence note for this opcode and its operand roles is on
/// [`encode_lfs`] directly above, and stays there.** This function adds
/// nothing to it but the absence of the final `.word()`.
#[inline(always)]
pub fn mop_lfs(double: bool, fd: u8, ra: u8, d: i16) -> MachineOp {
    MachineOp::new(if double { op::LFD } else { op::LFS }).s(fd).d0(ra).disp(d as i32)
}


/// `std rS, DS(rA)` — store doubleword, primary opcode 62, DS-form (the low two
/// bits select the form, so the displacement must be a multiple of 4). Captured
/// as `fbe1fff0` = `std r31,-16(r1)` in every callee-saved GPR prologue.
pub fn encode_std(rs: u8, ra: u8, ds: i16) -> [u8; 4] {
    mop_std(rs, ra, ds).word()
}

/// The [`MachineOp`] form of [`encode_std`] — the value S1c's op streams
/// carry, before any word exists.
///
/// **The evidence note for this opcode and its operand roles is on
/// [`encode_std`] directly above, and stays there.** This function adds
/// nothing to it but the absence of the final `.word()`.
#[inline(always)]
pub fn mop_std(rs: u8, ra: u8, ds: i16) -> MachineOp {
    MachineOp::new(op::STD).d0(rs).s(ra).disp(ds as i32)
}

/// `ld rD, DS(rA)` with a **GPR** destination — the epilogue's reload. Same
/// encoder as [`encode_ld`]; named separately only where the frame code reads
/// better for it. Captured as `ebe1fff0` = `ld r31,-16(r1)`.
pub(crate) fn encode_ldr(rd: u8, ra: u8, ds: i16) -> [u8; 4] {
    mop_ldr(rd, ra, ds).word()
}

/// The [`MachineOp`] form of [`encode_ldr`] — the value S1c's op streams
/// carry, before any word exists.
///
/// **The evidence note for this opcode and its operand roles is on
/// [`encode_ldr`] directly above, and stays there.** This function adds
/// nothing to it but the absence of the final `.word()`.
#[inline(always)]
pub(crate) fn mop_ldr(rd: u8, ra: u8, ds: i16) -> MachineOp {
    mop_ld(rd, ra, ds)
}

/// `stfd frS, d(rA)` — store float double, primary opcode 54 (D-form, so any
/// 16-bit displacement). Captured as `dbe1fff0` = `stfd f31,-16(r1)`.
pub fn encode_stfd(frs: u8, ra: u8, d: i16) -> [u8; 4] {
    mop_stfd(frs, ra, d).word()
}

/// The [`MachineOp`] form of [`encode_stfd`] — the value S1c's op streams
/// carry, before any word exists.
///
/// **The evidence note for this opcode and its operand roles is on
/// [`encode_stfd`] directly above, and stays there.** This function adds
/// nothing to it but the absence of the final `.word()`.
#[inline(always)]
pub fn mop_stfd(frs: u8, ra: u8, d: i16) -> MachineOp {
    MachineOp::new(op::STFD).d0(frs).s(ra).disp(d as i32)
}

/// `lfd frD, d(rA)` — load float double, primary opcode 50. Captured as
/// `cbe1fff0` = `lfd f31,-16(r1)`.
pub fn encode_lfd(frd: u8, ra: u8, d: i16) -> [u8; 4] {
    mop_lfd(frd, ra, d).word()
}

/// The [`MachineOp`] form of [`encode_lfd`] — the value S1c's op streams
/// carry, before any word exists.
///
/// **The evidence note for this opcode and its operand roles is on
/// [`encode_lfd`] directly above, and stays there.** This function adds
/// nothing to it but the absence of the final `.word()`.
#[inline(always)]
pub fn mop_lfd(frd: u8, ra: u8, d: i16) -> MachineOp {
    MachineOp::new(op::LFD).s(frd).d0(ra).disp(d as i32)
}

/// `stwu rS, d(rA)` — store word with update, primary opcode 37: the frame
/// allocation. Captured as `9421ffa0` = `stwu r1,-96(r1)`.
pub fn encode_stwu(rs: u8, ra: u8, d: i16) -> [u8; 4] {
    mop_stwu(rs, ra, d).word()
}

/// The [`MachineOp`] form of [`encode_stwu`] — the value S1c's op streams
/// carry, before any word exists.
///
/// **The evidence note for this opcode and its operand roles is on
/// [`encode_stwu`] directly above, and stays there.** This function adds
/// nothing to it but the absence of the final `.word()`.
#[inline(always)]
pub fn mop_stwu(rs: u8, ra: u8, d: i16) -> MachineOp {
    MachineOp::new(op::STWU).d0(rs).s(ra).disp(d as i32)
}

/// `mr rA, rS` — the `or rA, rS, rS` idiom c2 uses for a register-to-register
/// move (opcode 31, XO 444).
pub fn encode_mr(ra: u8, rs: u8) -> [u8; 4] {
    mop_mr(ra, rs).word()
}

/// The [`MachineOp`] form of [`encode_mr`] — the value S1c's op streams
/// carry, before any word exists.
///
/// **The evidence note for this opcode and its operand roles is on
/// [`encode_mr`] directly above, and stays there.** This function adds
/// nothing to it but the absence of the final `.word()`.
#[inline(always)]
pub fn mop_mr(ra: u8, rs: u8) -> MachineOp {
    MachineOp::new(op::MR).s(ra).d0(rs)
}

/// `mr. rA, rS` — the **record form** of the `or` move, opcode 31 XO 444 with
/// `Rc = 1`. It writes **cr0** as a side effect and is how `c2` closes a
/// sentinel loop: the value the next iteration needs is copied and tested in one
/// instruction, so no `cmplwi` is issued at the bottom of the body at all.
///
/// Captured: `7d4b5379` = `mr. r11,r10` (`?HashString@@YAHPBDH@Z` + 0x20).
///
/// Deliberately its own function rather than a `rc: bool` on [`encode_mr`]: the
/// two differ in *which condition register the branch after them reads* — cr0
/// here against [`CR_COMPARE`]'s cr6 — and that is board #188's defect, which
/// this port has already paid for once.
pub fn encode_mr_record(ra: u8, rs: u8) -> [u8; 4] {
    mop_mr_record(ra, rs).word()
}

/// The [`MachineOp`] form of [`encode_mr_record`] — the value S1c's op streams
/// carry, before any word exists.
///
/// **The evidence note for this opcode and its operand roles is on
/// [`encode_mr_record`] directly above, and stays there.** This function adds
/// nothing to it but the absence of the final `.word()`.
#[inline(always)]
pub fn mop_mr_record(ra: u8, rs: u8) -> MachineOp {
    MachineOp::new(op::OR_RC).s(ra).d0(rs).d1(rs)
}

/// `mulli rD, rA, SIMM` — primary opcode 7. The whole of `a * k` for the
/// literals `codegen::ptr_walk_loop` admits; see
/// `c2_il::func::body::shapes::ptr_walk_loop::is_mulli_literal` for the 38-cell
/// grid that says which those are and what `c2` emits instead for the rest.
///
/// Captured: `1d0a007f` = `mulli r8,r10,127`.
pub fn encode_mulli(rd: u8, ra: u8, simm: i16) -> [u8; 4] {
    mop_mulli(rd, ra, simm).word()
}

/// The [`MachineOp`] form of [`encode_mulli`] — the value S1c's op streams
/// carry, before any word exists.
///
/// **The evidence note for this opcode and its operand roles is on
/// [`encode_mulli`] directly above, and stays there.** This function adds
/// nothing to it but the absence of the final `.word()`.
#[inline(always)]
pub fn mop_mulli(rd: u8, ra: u8, simm: i16) -> MachineOp {
    MachineOp::new(op::MULLI).s(rd).d0(ra).disp(simm as i32)
}

/// `lbzu rD, d(rA)` — load byte and zero with **update**, primary opcode 35.
/// `rA` is written back with the effective address, so the pointer induction is
/// folded into the addressing mode and the loop body carries no separate
/// increment.
///
/// Captured: `8d490001` = `lbzu r10,1(r9)`.
pub fn encode_lbzu(rd: u8, ra: u8, d: i16) -> [u8; 4] {
    mop_lbzu(rd, ra, d).word()
}

/// The [`MachineOp`] form of [`encode_lbzu`] — the value S1c's op streams
/// carry, before any word exists.
///
/// **The evidence note for this opcode and its operand roles is on
/// [`encode_lbzu`] directly above, and stays there.** This function adds
/// nothing to it but the absence of the final `.word()`.
#[inline(always)]
pub fn mop_lbzu(rd: u8, ra: u8, d: i16) -> MachineOp {
    MachineOp::new(op::LBZU).s(rd).d0(ra).disp(d as i32)
}

/// `divw rD, rA, rB` — signed word divide, opcode 31 XO 491.
/// Captured: `7ce823d6` = `divw r7,r8,r4`.
pub fn encode_divw(rd: u8, ra: u8, rb: u8) -> [u8; 4] {
    mop_divw(rd, ra, rb).word()
}

/// The [`MachineOp`] form of [`encode_divw`] — the value S1c's op streams
/// carry, before any word exists.
///
/// **The evidence note for this opcode and its operand roles is on
/// [`encode_divw`] directly above, and stays there.** This function adds
/// nothing to it but the absence of the final `.word()`.
#[inline(always)]
pub fn mop_divw(rd: u8, ra: u8, rb: u8) -> MachineOp {
    MachineOp::new(op::DIVW).s(rd).d0(ra).d1(rb)
}

/// `divwu rD, rA, rB` — **unsigned** word divide, opcode 31 XO 459.
/// Captured: `7c632396` = `divwu r3,r3,r4` (`work/w-divmod/twigrid.py`, row
/// `u-div-var`, byte-identical at `/O1` and `/Ox`).
pub fn encode_divwu(rd: u8, ra: u8, rb: u8) -> [u8; 4] {
    mop_divwu(rd, ra, rb).word()
}

/// The [`MachineOp`] form of [`encode_divwu`] — the value S1c's op streams
/// carry, before any word exists.
///
/// **The evidence note for this opcode and its operand roles is on
/// [`encode_divwu`] directly above, and stays there.** This function adds
/// nothing to it but the absence of the final `.word()`.
#[inline(always)]
pub fn mop_divwu(rd: u8, ra: u8, rb: u8) -> MachineOp {
    MachineOp::new(op::DIVWU).s(rd).d0(ra).d1(rb)
}

/// `twi TO, rA, SIMM` — **trap word immediate**, primary opcode 3. The
/// architectural `TO` bits, MSB first, are
/// `[a<b signed, a>b signed, a=b, a<b unsigned, a>b unsigned]`.
///
/// `c2` emits exactly two of them for a signed integer `/` or `%` by a
/// **non-constant** divisor, and they are the two guards the C++ standard makes
/// undefined. Both were read off the encoding rather than paraphrased:
///
/// * **`twi 6, rD, 0`** — `TO = 0b00110` = *equal* ∪ *unsigned less-than*.
///   `rD <u 0` is unsatisfiable, so the instruction traps exactly when the
///   **divisor is zero**. Captured `0cc40000` = `twi 6,r4,0`.
/// * **`twi 5, rX, -1`** — `TO = 0b00101` = *equal* ∪ *unsigned greater-than*.
///   `rX >u 0xFFFFFFFF` is unsatisfiable, so it traps exactly when `rX == -1`,
///   and `rX` is `andc(divisor, rotlwi(dividend,1) - 1)`:
///   `rotlwi(n,1) - 1` is `0` **iff** `n == INT_MIN` (`0x80000000` rotates to
///   `1`), and `andc(d, 0)` is `d`, so `rX == -1` **iff**
///   `dividend == INT_MIN && divisor == -1`. That is the `INT_MIN / -1`
///   overflow guard, and the three-instruction predicate ahead of it is its
///   whole computation. Captured `0ca6ffff` = `twi 5,r6,-1`.
///
/// A **non-zero constant** divisor emits neither — `c2` decides both guards
/// statically (`work/w-hash/divgrid.py`, rows `s-mod-k7`/`s-div-k7`;
/// `work/w-divmod/twigrid.py` re-runs it over **24** literal cells covering both
/// signs, both signednesses, `INT_MIN`, `INT_MAX`, the `simm16` cliff, and the
/// same values reached through a `const` local, a namespace-scope `const` and an
/// enumerator) — and an **unsigned** divide emits only the first (`u-div-var`,
/// `u-mod-var`), because the overflow case cannot arise.
///
/// **There is a THIRD `TO`, and it is not a guard.** A divisor that is a
/// compile-time **zero** emits no division at all and a bare
///
/// * **`twi 7, r0, 0`** — `TO = 0b00111` = *equal* ∪ *unsigned less-than* ∪
///   *unsigned greater-than*, which is a tautology over the unsigned order, so
///   the instruction traps **unconditionally**. Captured `0ce00000`, and the
///   operand register is **`r0`** — not the dividend, not the divisor, because
///   the trap does not read anything.
///
/// Seven cells produce it and they are all the same value by different routes
/// (`a%0`, `a/0`, `a%0u`, `a/0u`, a `const int k=0`, a namespace-scope `const`,
/// an enumerator). `TO = 7` is *not* emitted for any other divisor, and the
/// grid observed no fourth value across 161 cells. None of this is shipped —
/// `div_mod_leaf` refuses every constant divisor — but the `TO` axis is
/// recorded here so a later rung does not rediscover it as an anomaly.
pub fn encode_twi(to: u8, ra: u8, simm: i16) -> [u8; 4] {
    mop_twi(to, ra, simm).word()
}

/// The [`MachineOp`] form of [`encode_twi`] — the value S1c's op streams
/// carry, before any word exists.
///
/// **The evidence note for this opcode and its operand roles is on
/// [`encode_twi`] directly above, and stays there.** This function adds
/// nothing to it but the absence of the final `.word()`.
#[inline(always)]
pub fn mop_twi(to: u8, ra: u8, simm: i16) -> MachineOp {
    MachineOp::new(op::TWI).s(to).d0(ra).disp(simm as i32)
}

// ---- W8: the conditional-branch family ------------------------------------
//
// `docs/CFG_SHAPE.md` §3.1 tabulates four forms; three of them are here and the
// fourth (the external `b`/`bl`) is [`super::calls::encode_tail_branch`]'s,
// because it encodes a section offset rather than a displacement and takes a
// relocation. **They are the same opcode.** An emitter that treats every `b`
// alike corrupts one of the two (§3.3, board #191), which is why the two
// encoders are deliberately not merged.

/// The condition-register field an explicit compare feeding a branch writes.
///
/// **cr6, and it is REUSED rather than allocated** — `?b_ifn` writes cr6 three
/// times in one body, each branch consuming its own before the next is issued
/// (`docs/CFG_SHAPE.md` §3.2). It is a named constant and not a literal `6`
/// because the *other* producer is different: a record-form instruction such as
/// `addic.` writes **cr0**, and c2 branches on cr0 there without an intervening
/// compare. A lowering that hard-codes `BI = 4*6 + bit` emits `409a…` where the
/// obj has `4082…` for every decrement-and-test loop — board #188, and the
/// reason this constant exists to be *passed in* the day a record-form producer
/// is admitted.
/// PROV[O] board #188 — WHICH CR field c2 uses is c2's choice, not the ISA's: `cr6` for an explicit compare, but `addic.` writes `cr0` and c2 branches on `cr0` there with no intervening compare. A lowering that hard-codes `4*6 + bit` emits `409a…` where the obj has `4082…`.
pub const CR_COMPARE: u8 = 6;

/// `BO` for "branch if the CR bit is SET".
/// PROV[S] PowerPC ISA — `BO` 12 is branch-if-CR-bit-set.
pub const BO_TRUE: u8 = 12;
/// `BO` for "branch if the CR bit is CLEAR".
/// PROV[S] PowerPC ISA — `BO` 4 is branch-if-CR-bit-clear.
pub const BO_FALSE: u8 = 4;
/// `BO` for "branch always" — what makes `bclr` a plain `blr`.
/// PROV[S] PowerPC ISA — `BO` 20 is branch-always, which is what makes `bclr` a plain `blr`.
pub const BO_ALWAYS: u8 = 20;

/// The bit within a CR field, by relation: LT=0, GT=1, EQ=2, SO=3.
/// PROV[S] PowerPC ISA — the CR field bit order is LT=0, GT=1, EQ=2, SO=3.
pub const CR_BIT_LT: u8 = 0;
// PROV[S] PowerPC ISA — CR field bit order.
pub const CR_BIT_GT: u8 = 1;
// PROV[S] PowerPC ISA — CR field bit order.
pub const CR_BIT_EQ: u8 = 2;

/// `BI` = `4*crf + bit`.
pub fn cr_bi(crf: u8, bit: u8) -> u8 {
    4 * (crf & 7) + (bit & 3)
}

/// **The architectural reach of a `bc`**: `BD` is a signed 14-bit field scaled
/// by 4, so ±32764 bytes.
///
/// Measured, not assumed. `docs/CFG_SHAPE.md` §3.3.1 swept the displacement and
/// found c2 emitting a direct `bne` at **+32628** and the two-instruction
/// expansion — invert the condition, branch over an unconditional `b` — at
/// **+34148**. The switch is at the limit with **no slack**: c2 uses the full
/// field before expanding.
/// PROV[O] `docs/CFG_SHAPE.md` §3.3.1 — the ISA gives the 14-bit scaled field; what is MEASURED is that c2 uses the full field with no slack, direct `bne` at +32628 and the two-instruction expansion at +34148. The number is architectural, the CHOICE to switch exactly here is c2's and is graded.
pub const BC_MAX_DISP: i32 = 32764;

/// Encode `bc BO,BI,<target>` — primary opcode 16, `AA=0`, `LK=0`.
///
/// `disp` is **self-relative**: `target_offset − branch_offset`, not relative
/// to the section start (`docs/CFG_SHAPE.md` §3.3). It carries **no
/// relocation**; `pa.cpp`'s seven code sections all report `nrel = 0` despite
/// six of them containing a branch.
///
/// Returns `None` past [`BC_MAX_DISP`], where the expansion is required. The
/// caller must not truncate: a truncated `BD` is a legal-looking branch to the
/// wrong place, which is the fuzzy-invisible failure class
/// `docs/CODEGEN_PPC_MVP.md` warns about.
pub fn encode_bc(bo: u8, bi: u8, disp: i32) -> Option<[u8; 4]> {
    Some(mop_bc(bo, bi, disp)?.word())
}

/// The [`MachineOp`] form of [`encode_bc`] — the value S1c's op streams
/// carry, before any word exists.
///
/// **The evidence note for this opcode and its operand roles is on
/// [`encode_bc`] directly above, and stays there.** This function adds
/// nothing to it but the absence of the final `.word()`.
#[inline(always)]
pub fn mop_bc(bo: u8, bi: u8, disp: i32) -> Option<MachineOp> {
    if disp % 4 != 0 || !(-BC_MAX_DISP - 4..=BC_MAX_DISP).contains(&disp) {
        return None;
    }
    Some(MachineOp::new(op::BC).s(bo).d0(bi).disp(disp))
}

/// The largest displacement an unconditional `b` reaches: `LI` is a signed
/// 24-bit field scaled by 4.
/// PROV[S] PowerPC ISA — `LI` is a signed 24-bit field scaled by 4. No c2-side measurement pins the switch point here the way `CFG_SHAPE.md` §3.3.1 does for `BC_MAX_DISP`; the reach is the architecture's.
pub const B_MAX_DISP: i32 = 0x01FF_FFFC;

/// Encode an **intra-section** unconditional branch `b` — primary opcode 18,
/// `AA=0`, `LK=0` — carrying its **true self-relative displacement** and taking
/// **no relocation**.
///
/// **This is board #191, and it is the same opcode as [`encode_tail_branch`].**
/// The two are different encodings of one instruction and the discriminator is
/// *where the target lives*, not what the branch is:
///
/// ```text
///   48000008   intra-section: LI is the real displacement, nrel = 0
///   4bffffec   external:      LI is −(own .text offset), plus a REL24
/// ```
///
/// A fixup pass that treats every `b` alike corrupts one of the two
/// (`docs/CFG_SHAPE.md` §3.3), which is why they are two functions here rather
/// than one with a flag.
///
/// **It has been written once before and deleted.** W10 built it for the `else`
/// arm's join branch, found that arm's block layout to be mode-dependent on a
/// threshold that is a c2 cost model, and removed the encoder rather than ship a
/// code path the oracle had never graded (w-frame row **F-c**). It comes back
/// with W11's guarded early return, whose `b` targets the **epilogue** — a block
/// that exists in both modes and whose length is a constant of the frame class,
/// so there is no threshold to fit.
///
/// `disp` is `target_offset − branch_offset`. Returns `None` for a misaligned or
/// out-of-range displacement rather than truncating: a truncated `LI` is a
/// legal-looking branch to the wrong place.
pub fn encode_b_intra(disp: i32) -> Option<[u8; 4]> {
    Some(mop_b_intra(disp)?.word())
}

/// The [`MachineOp`] form of [`encode_b_intra`] — the value S1c's op streams
/// carry, before any word exists.
///
/// **The evidence note for this opcode and its operand roles is on
/// [`encode_b_intra`] directly above, and stays there.** This function adds
/// nothing to it but the absence of the final `.word()`.
#[inline(always)]
pub fn mop_b_intra(disp: i32) -> Option<MachineOp> {
    if disp % 4 != 0 || !(-B_MAX_DISP - 4..=B_MAX_DISP).contains(&disp) {
        return None;
    }
    Some(MachineOp::new(op::B).disp(disp))
}

/// Encode `cmpwi crf,rA,SIMM` — the **signed** immediate compare, opcode 11.
pub fn encode_cmpwi(crf: u8, ra: u8, simm: i16) -> [u8; 4] {
    mop_cmpwi(crf, ra, simm).word()
}

/// The [`MachineOp`] form of [`encode_cmpwi`] — the value S1c's op streams
/// carry, before any word exists.
///
/// **The evidence note for this opcode and its operand roles is on
/// [`encode_cmpwi`] directly above, and stays there.** This function adds
/// nothing to it but the absence of the final `.word()`.
#[inline(always)]
pub fn mop_cmpwi(crf: u8, ra: u8, simm: i16) -> MachineOp {
    MachineOp::new(op::CMPI).s(crf).d0(ra).disp(simm as i32)
}

/// Encode `cmplwi crf,rA,UIMM` — the **unsigned** immediate compare, opcode 10.
///
/// Which of the two a body gets comes from the shared operand TYPE triple at the
/// comparison and from nothing else: the relational opcodes are sign-agnostic,
/// and a pointer null-check is therefore an *unsigned* compare
/// (`docs/CFG_SHAPE.md` §3.2 — `?MemFree` and both `Pool.cpp` functions emit
/// `cmplwi`).
pub fn encode_cmplwi(crf: u8, ra: u8, uimm: u16) -> [u8; 4] {
    mop_cmplwi(crf, ra, uimm).word()
}

/// The [`MachineOp`] form of [`encode_cmplwi`] — the value S1c's op streams
/// carry, before any word exists.
///
/// **The evidence note for this opcode and its operand roles is on
/// [`encode_cmplwi`] directly above, and stays there.** This function adds
/// nothing to it but the absence of the final `.word()`.
#[inline(always)]
pub fn mop_cmplwi(crf: u8, ra: u8, uimm: u16) -> MachineOp {
    MachineOp::new(op::CMPLI).s(crf).d0(ra).disp(uimm as i32)
}

/// Encode `cmpw crf,rA,rB` — the **signed register-register** word compare,
/// X-form: primary opcode 31, extended 0, `L = 0`.
///
/// [`encode_cmpwi`] is its immediate sibling and existed first because every
/// comparison the port had lowered until now put a literal on one side. The
/// register-register form is what a loop test against a *loaded* value needs,
/// and board **#1105** names its absence as the first of `Primes.cpp`'s
/// refusals.
///
/// **Pinned to real `c2` output, not derived from a manual.** `cmpw cr6,r10,r3`
/// at offset `0x14` of `?NextHashPrime@@YAHH@Z` is `7f0a1800`
/// (`work/w-loop/Primes_b.obj`, `/O1 /Oi /EHsc`, the workload's own flags), and
/// `codegen::frontier_bytes` (`cfg(test)`) asserts that word in place against the whole
/// 64-byte function.
///
/// **This encoder has no accept-path caller and that is deliberate.** Nothing in
/// [`super::select`] reaches it; the port still returns `NotImplemented` on
/// every body that would need it. It is an ISA transcription in a file of ISA
/// transcriptions, graded by a byte c2 really emitted — which is the distinction
/// board **#278** drew when it *deleted* `bss_deferred_layout`: that item's tests
/// asserted a **layout rule** that had been superseded, where these assert a
/// **fixed instruction encoding** that cannot be.
pub fn encode_cmpw(crf: u8, ra: u8, rb: u8) -> [u8; 4] {
    mop_cmpw(crf, ra, rb).word()
}

/// The [`MachineOp`] form of [`encode_cmpw`] — the value S1c's op streams
/// carry, before any word exists.
///
/// **The evidence note for this opcode and its operand roles is on
/// [`encode_cmpw`] directly above, and stays there.** This function adds
/// nothing to it but the absence of the final `.word()`.
#[inline(always)]
pub fn mop_cmpw(crf: u8, ra: u8, rb: u8) -> MachineOp {
    MachineOp::new(op::CMP).s(crf).d0(ra).d1(rb)
}

/// Encode `cmplw crf,rA,rB` — the **unsigned** register-register compare:
/// X-form, primary opcode 31, extended **32**, where [`encode_cmpw`]'s signed
/// form is extended 0.
///
/// **This encoder did not exist, and two published rung tables disagreed about
/// whether it did.** `w-extdata` §2 priced `osfinfo`'s missing encoders at two;
/// `w-undname` §5 corrected that to one on the ground that "`encode_cmplw`
/// already exists". It does not — what exists is [`encode_cmpw`] (extended 0,
/// signed) and [`encode_cmplwi`] (primary opcode 10, immediate), and neither
/// produces this word. The original count of two was right. Recorded here
/// rather than only in a rung because the next lane to read that table will
/// read this file too.
///
/// **Pinned to real `c2` output, not derived from a manual.**
/// `cmplw cr6,r3,r11` at offset `0x1c` of `_free_osfhnd` is `7f035840`
/// (`work/w-osfinfo/ref/osfinfo/dis.txt`, the workload's own
/// `/O1 /Oi /EHsc /GR`), and `codegen::osf_handle_guard` asserts that word in
/// place against the whole 152-byte function.
///
/// The signed/unsigned split is **not** cosmetic here: `_free_osfhnd` tests
/// `fh >= 0` with the signed immediate form two words earlier and
/// `fh < _nhandle` with this one, in the same body, on the same operand. A
/// class that used one form for both emits the right program with one wrong
/// word and every branch still resolving — `docs/GAPS.md` §6.
pub fn encode_cmplw(crf: u8, ra: u8, rb: u8) -> [u8; 4] {
    mop_cmplw(crf, ra, rb).word()
}

/// The [`MachineOp`] form of [`encode_cmplw`] — the value S1c's op streams
/// carry, before any word exists.
///
/// **The evidence note for this opcode and its operand roles is on
/// [`encode_cmplw`] directly above, and stays there.** This function adds
/// nothing to it but the absence of the final `.word()`.
#[inline(always)]
pub fn mop_cmplw(crf: u8, ra: u8, rb: u8) -> MachineOp {
    MachineOp::new(op::CMPL).s(crf).d0(ra).d1(rb)
}

/// Encode `lwzx rD,rA,rB` — load word, **indexed**: X-form, primary opcode 31,
/// extended 23.
///
/// The scaled-index addressing mode `base[i]`: c2 emits `slwi rT,rI,2` (an
/// [`encode_rlwinm`] the port already has) and then this. It is the second and
/// last instruction in `Primes.cpp`'s 64 bytes with no encoder — see
/// `codegen::frontier_bytes` (`cfg(test)`) for the count that statement comes from.
///
/// **Pinned to real `c2` output**: `lwzx r10,r10,r9` at `0x24` is `7d4a482e`
/// and `lwzx r3,r11,r9` at `0x38` is `7c6b482e`, both from
/// `work/w-loop/Primes_b.obj`. Two distinct cells, so the `rD` and `rA` fields
/// are separated by the pins rather than only by the formula.
///
/// Same accept-path caveat as [`encode_cmpw`], for the same reason.
pub fn encode_lwzx(rd: u8, ra: u8, rb: u8) -> [u8; 4] {
    mop_lwzx(rd, ra, rb).word()
}

/// The [`MachineOp`] form of [`encode_lwzx`] — the value S1c's op streams
/// carry, before any word exists.
///
/// **The evidence note for this opcode and its operand roles is on
/// [`encode_lwzx`] directly above, and stays there.** This function adds
/// nothing to it but the absence of the final `.word()`.
#[inline(always)]
pub fn mop_lwzx(rd: u8, ra: u8, rb: u8) -> MachineOp {
    MachineOp::new(op::LWZX).s(rd).d0(ra).d1(rb)
}

#[cfg(test)]
mod tests {
    // The single `mod tests` this was split out of opened with
    // `use super::*;`; the glob keeps that reach.
    #[allow(unused_imports)]
    use super::*;
    #[allow(unused_imports)]
    use crate::codegen::*;
    #[allow(unused_imports)]
    use c2_il::{IlFunction, IlOp};
    #[allow(unused_imports)]
    use crate::codegen::testutil::*;
    #[test]
    fn encode_add_matches_reference_words() {
        assert_eq!(encode_add(11, 3, 4), [0x7D, 0x63, 0x22, 0x14]);
        assert_eq!(encode_add(3, 11, 5), [0x7C, 0x6B, 0x2A, 0x14]);
    }

    #[test]
    fn encode_blr_is_fixed() {
        assert_eq!(encode_blr(), [0x4E, 0x80, 0x00, 0x20]);
    }

    #[test]
    fn encode_lwz_matches_reference_words() {
        // Transcribed from the reference obj of fixtures/cpp/il_expr_deref.cpp and
        // il_expr_member.cpp — not derived from the encoding rule.
        assert_eq!(encode_lwz(3, 3, 0), [0x80, 0x63, 0x00, 0x00]); // *p          , p in r3
        assert_eq!(encode_lwz(3, 4, 0), [0x80, 0x64, 0x00, 0x00]); // *p          , p in r4
        assert_eq!(encode_lwz(3, 5, 0), [0x80, 0x65, 0x00, 0x00]); // *p          , p in r5
        assert_eq!(encode_lwz(3, 3, 4), [0x80, 0x63, 0x00, 0x04]); // s->b
        assert_eq!(encode_lwz(3, 3, 16), [0x80, 0x63, 0x00, 0x10]); // s->d
        assert_eq!(encode_lwz(3, 3, 12), [0x80, 0x63, 0x00, 0x0C]); // p[3]
        assert_eq!(encode_lwz(3, 3, -4), [0x80, 0x63, 0xFF, 0xFC]); // p[-1]
        assert_eq!(encode_lwz(3, 3, 32000), [0x80, 0x63, 0x7D, 0x00]); // p[8000]
        assert_eq!(encode_lwz(3, 4, 8), [0x80, 0x64, 0x00, 0x08]); // int f(int a,S* s){return s->c;}
    }

    #[test]
    fn narrow_load_encoders_match_reference_words() {
        // Every word transcribed from a reference obj of
        // `fixtures/cpp/w12_narrow_getters.cpp` (and the probe TUs behind
        // `docs/IL_LOAD_TYPES.md` §3) — not derived from the encoding rule.
        //
        // `char f(char* p){return *p;}`            88630000  lbz r3,0(r3)
        assert_eq!(encode_lbz(3, 3, 0), [0x88, 0x63, 0x00, 0x00]);
        // `int f(char* p){return *p;}`             89630000  lbz r11,0(r3)
        assert_eq!(encode_lbz(11, 3, 0), [0x89, 0x63, 0x00, 0x00]);
        // `int f(int a,char* p){return *p;}`       89640000  lbz r11,0(r4)
        assert_eq!(encode_lbz(11, 4, 0), [0x89, 0x64, 0x00, 0x00]);
        // `s->c` at 4 / `s->u` at 8 / `p[3]`       88630004 / 88630008 / 88630003
        assert_eq!(encode_lbz(3, 3, 4), [0x88, 0x63, 0x00, 0x04]);
        assert_eq!(encode_lbz(3, 3, 8), [0x88, 0x63, 0x00, 0x08]);
        assert_eq!(encode_lbz(3, 3, 3), [0x88, 0x63, 0x00, 0x03]);
        assert_eq!(encode_lbz(11, 3, 4), [0x89, 0x63, 0x00, 0x04]);
        // `short f(short* p){return *p;}`          a0630000  lhz r3,0(r3)
        assert_eq!(encode_lhz(3, 3, 0), [0xA0, 0x63, 0x00, 0x00]);
        // `s->h` at 6 / `p[2]` at 4 / `t_uh` at 6  a0630006 / a0630004
        assert_eq!(encode_lhz(3, 3, 6), [0xA0, 0x63, 0x00, 0x06]);
        assert_eq!(encode_lhz(3, 3, 4), [0xA0, 0x63, 0x00, 0x04]);
        // `int f(short* p){return *p;}` under /Ox  a1630000  lhz r11,0(r3)
        assert_eq!(encode_lhz(11, 3, 0), [0xA1, 0x63, 0x00, 0x00]);
        // `long long f(long long* p){return *p;}`  e8630000  ld r3,0(r3)
        assert_eq!(encode_ld(3, 3, 0), [0xE8, 0x63, 0x00, 0x00]);
        // `s->q` at 16 / `t_q` at 8 / `p[2]` at 16 e8630010 / e8630008
        assert_eq!(encode_ld(3, 3, 16), [0xE8, 0x63, 0x00, 0x10]);
        assert_eq!(encode_ld(3, 3, 8), [0xE8, 0x63, 0x00, 0x08]);
        // DS-form: the low two bits are the form's, never the displacement's. A
        // caller must gate `off % 4`; if one ever did not, the word it would get is
        // the truncated one, not a rounded-up address.
        assert_eq!(encode_ld(3, 3, -8), [0xE8, 0x63, 0xFF, 0xF8]);
        assert_eq!(encode_ld(3, 3, 3), [0xE8, 0x63, 0x00, 0x00]);
        // `extsb r3,r11` / `extsh r3,r11` — rS in bits 21..25, rA in 16..20, so the
        // operand order in the mnemonic is the reverse of the field order.
        assert_eq!(encode_extsb(3, 11), [0x7D, 0x63, 0x07, 0x74]);
        assert_eq!(encode_extsh(3, 11), [0x7D, 0x63, 0x07, 0x34]);
        // `extsb r11,r11` (`*p + 1`, the refused arithmetic form) and `extsb r3,r3`
        // (`int f(char a)`, the refused widen-param rung) — both captured, both
        // distinct words, so the register fields are pinned in each direction.
        assert_eq!(encode_extsb(11, 11), [0x7D, 0x6B, 0x07, 0x74]);
        assert_eq!(encode_extsb(3, 3), [0x7C, 0x63, 0x07, 0x74]);
    }

    #[test]
    fn encode_mullw_matches_reference_words() {
        // a*b*c → mullw r11,r3,r4 ; mullw r3,r11,r5
        assert_eq!(encode_mullw(11, 3, 4), [0x7D, 0x63, 0x21, 0xD6]);
        assert_eq!(encode_mullw(3, 11, 5), [0x7C, 0x6B, 0x29, 0xD6]);
    }

    /// `bclr` and `extsb.` against the words real `c2` emits for a signed
    /// sentinel walk (`work/w-varloop/probe.py`, every TWO-regime cell).
    ///
    /// **`blr` is `bclr` at `BO_ALWAYS`, `BI = 0`** — asserted rather than
    /// asserted-in-prose, because [`encode_blr`] is a hard-coded constant and
    /// [`encode_bclr`] is computed, and two spellings of one instruction that
    /// nothing compares are two chances to be wrong about it.
    #[test]
    fn encode_bclr_and_extsb_record_match_reference_words() {
        assert_eq!(encode_bclr(BO_TRUE, cr_bi(0, CR_BIT_EQ)), [0x4D, 0x82, 0x00, 0x20]);
        assert_eq!(encode_bclr(BO_ALWAYS, 0), encode_blr());
        // `extsb. r11,r11` (the entry test) and `extsb. r11,r9` (the record
        // form) — the two spellings the loop emits, and the Rc bit is the whole
        // difference from `encode_extsb`.
        assert_eq!(encode_extsb_record(11, 11), [0x7D, 0x6B, 0x07, 0x75]);
        assert_eq!(encode_extsb_record(11, 9), [0x7D, 0x2B, 0x07, 0x75]);
        assert_eq!(u32::from_be_bytes(encode_extsb(11, 11)) | 1,
                   u32::from_be_bytes(encode_extsb_record(11, 11)));
        // The record form writes cr0 and the plain form does not: a branch may
        // read the CR after one and not the other (board #188).
        assert_eq!(u32::from_be_bytes(encode_extsb(11, 11)) & 1, 0);
    }

    #[test]
    fn encode_subf_matches_reference_words() {
        // a-b-c → subf r11,r4,r3 ; subf r3,r5,r11 (rA = subtrahend).
        assert_eq!(encode_subf(11, 4, 3), [0x7D, 0x64, 0x18, 0x50]);
        assert_eq!(encode_subf(3, 5, 11), [0x7C, 0x65, 0x58, 0x50]);
    }

    #[test]
    fn encode_addi_matches_reference_words() {
        assert_eq!(encode_addi(3, 3, 5), [0x38, 0x63, 0x00, 0x05]); // a+5
        assert_eq!(encode_addi(3, 3, -5), [0x38, 0x63, 0xFF, 0xFB]); // a-5
        assert_eq!(encode_addi(3, 0, 42), [0x38, 0x60, 0x00, 0x2A]); // li r3,42
    }


    #[test]
    fn w8_branch_words_match_the_reference_obj() {
        // `?MemFree@NUISPEECH@@YAXPAX0K@Z`, docs/CFG_SHAPE.md §4.1/§3.1's worked
        // example: BO=4 (branch-if-clear), BI=4*6+2=26 (cr6's EQ bit), BD=+16.
        assert_eq!(cr_bi(CR_COMPARE, CR_BIT_EQ), 26);
        assert_eq!(encode_bc(BO_FALSE, 26, 16), Some([0x40, 0x9A, 0x00, 0x10]));
        // `?MemAlloc`, same body one word shorter.
        assert_eq!(encode_bc(BO_FALSE, 26, 12), Some([0x40, 0x9A, 0x00, 0x0C]));
        // §3.4's `?b_ifelse`/`?d_early` rows, the other sense.
        assert_eq!(encode_bc(BO_TRUE, 26, 8), Some([0x41, 0x9A, 0x00, 0x08]));
        assert_eq!(encode_bc(BO_TRUE, 26, 12), Some([0x41, 0x9A, 0x00, 0x0C]));
        // §3.7a's `?c_forcall` back edge: BO=12, BI=24 (LT), BD=-20.
        assert_eq!(encode_bc(BO_TRUE, 24, -20), Some([0x41, 0x98, 0xFF, 0xEC]));
    }

    #[test]
    fn a_branch_past_the_field_refuses_rather_than_truncating() {
        // §3.3.1 bracketed the switch between +32628 (direct) and +34148
        // (expanded), i.e. at the architectural limit with no slack. A
        // truncated `BD` is a legal-looking branch to the wrong place, so the
        // encoder returns None and the caller refuses.
        assert!(encode_bc(BO_FALSE, 26, 32628).is_some());
        assert!(encode_bc(BO_FALSE, 26, BC_MAX_DISP).is_some());
        assert!(encode_bc(BO_FALSE, 26, BC_MAX_DISP + 4).is_none());
        assert!(encode_bc(BO_FALSE, 26, 34148).is_none());
        // Not word-aligned: not a branch target at all.
        assert!(encode_bc(BO_FALSE, 26, 6).is_none());
    }

    #[test]
    fn w8_compare_words_match_the_reference_obj() {
        // §3.2's witness rows.
        assert_eq!(encode_cmplwi(CR_COMPARE, 3, 0), [0x2B, 0x03, 0x00, 0x00]); // ?MemFree
        assert_eq!(encode_cmplwi(CR_COMPARE, 11, 0), [0x2B, 0x0B, 0x00, 0x00]); // ?mmioGetInfo
        assert_eq!(encode_cmpwi(CR_COMPARE, 3, 0), [0x2F, 0x03, 0x00, 0x00]); // ?b_ifn
        assert_eq!(encode_cmpwi(CR_COMPARE, 3, 7), [0x2F, 0x03, 0x00, 0x07]); // ?d_switch
        assert_eq!(encode_cmpwi(CR_COMPARE, 31, 0), [0x2F, 0x1F, 0x00, 0x00]); // ?d_cont
    }

    /// **W-OSFINFO — `cmplw` against the byte real `c2` emitted**, plus the
    /// separation from the three compare encoders that already existed.
    ///
    /// The separation is the point rather than the value: two published rung
    /// tables disagreed about whether this encoder existed, and the reason the
    /// wrong one was believable is that `cmpw`, `cmplwi` and `cmplw` are one
    /// letter apart in the name and one field apart in the word.
    #[test]
    fn w_osfinfo_cmplw_matches_the_reference_obj_and_is_none_of_its_neighbours() {
        // `_free_osfhnd` +0x1c, `work/w-osfinfo/ref/osfinfo/dis.txt`.
        assert_eq!(encode_cmplw(CR_COMPARE, 3, 11), [0x7F, 0x03, 0x58, 0x40]);
        // …and the signed register form two words of the ISA away, which this
        // body does NOT use here — extended 0 against extended 32.
        assert_eq!(encode_cmpw(CR_COMPARE, 3, 11), [0x7F, 0x03, 0x58, 0x00]);
        assert_ne!(encode_cmplw(CR_COMPARE, 3, 11), encode_cmpw(CR_COMPARE, 3, 11));
        // …and the immediate form, which is a different primary opcode.
        assert_ne!(encode_cmplw(CR_COMPARE, 3, 11), encode_cmplwi(CR_COMPARE, 3, 11));
        // The `rB` field is separated from `rA` by a second pin.
        assert_eq!(encode_cmplw(CR_COMPARE, 11, 3), [0x7F, 0x0B, 0x18, 0x40]);
    }

    /// **W-JSON — `lhzx` against the byte real `c2` emitted**, and the
    /// separation from the two loads it is one field away from.
    ///
    /// `encode_lwzx` beside it is extended **23** where this is **279**, and
    /// `encode_lhz` is a different primary opcode entirely. The separation is
    /// the point rather than the value: an indexed halfword load that read a
    /// word would be a program that runs and reads two code units at once.
    #[test]
    fn w_json_lhzx_matches_the_reference_obj_and_is_none_of_its_neighbours() {
        // `?GetBuffer@JsonWriter@@QAAJPAGPAK@Z` +0x4c, `work/w-json/probe/ref.obj`.
        assert_eq!(encode_lhzx(11, 11, 6), [0x7D, 0x6B, 0x32, 0x2E]);
        assert_ne!(encode_lhzx(11, 11, 6), encode_lwzx(11, 11, 6));
        assert_ne!(encode_lhzx(11, 11, 6), encode_lhz(11, 11, 6));
        // The three register fields are separated from each other by a second pin.
        assert_eq!(encode_lhzx(9, 4, 3), [0x7D, 0x24, 0x1A, 0x2E]);
    }

    /// **W-BLOCKIR — the three FP forms the array-walk loop needs**, each
    /// against the byte real `c2` emitted, and each separated from the
    /// neighbours it is one field or one primary opcode away from.
    ///
    /// The separation is the point rather than the value. `lfsx` and `stfsx`
    /// differ by their extended opcode alone (535 against 663) and a load
    /// written where a store belongs is a program that runs; `stfsu` and `stfs`
    /// differ by their **primary** opcode alone (53 against 52) and the wrong
    /// one silently drops the induction step, which reads as an infinite loop
    /// rather than as a wrong byte.
    #[test]
    fn w_blockir_fp_walk_forms_match_the_reference_obj_and_are_none_of_their_neighbours() {
        // `?Add_InPlace@IPP@@YAXIPBMPAM@Z` +0x14, work/w-blockir/ref/ipp.dis.txt.
        assert_eq!(encode_lfsx(0, 10, 11), [0x7C, 0x0A, 0x5C, 0x2E]);
        // `?Mul@IPP@@YAXIPBM0PAM@Z` +0x24, same dump.
        assert_eq!(encode_stfsx(0, 9, 11), [0x7C, 0x09, 0x5D, 0x2E]);
        // `?MulConstant_InPlace@IPP@@YAXIPAMM@Z` +0x18, same dump.
        assert_eq!(encode_stfsu(0, 11, 4), [0xD4, 0x0B, 0x00, 0x04]);
        // One extended opcode apart, and the two are not the same word.
        assert_ne!(encode_lfsx(0, 9, 11), encode_stfsx(0, 9, 11));
        // One PRIMARY opcode apart: `stfs f0,4(r11)` is `d00b0004`.
        assert_eq!(encode_stfs(false, 0, 11, 4), [0xD0, 0x0B, 0x00, 0x04]);
        assert_ne!(encode_stfsu(0, 11, 4), encode_stfs(false, 0, 11, 4));
        // …and the D-form load the walker itself uses, which is a third word.
        assert_eq!(encode_lfs(false, 13, 11, 0), [0xC1, 0xAB, 0x00, 0x00]);
        assert_ne!(encode_lfs(false, 0, 11, 0), encode_lfsx(0, 0, 11));
        // The register fields are separated from each other by a second pin:
        // `lfsx f13,r10,r11` is `?Mul_InPlace`'s `-=` sibling's word.
        assert_eq!(encode_lfsx(13, 10, 11), [0x7D, 0xAA, 0x5C, 0x2E]);
        // A negative displacement on the update form is the pre-bias's own
        // sibling and must sign-extend rather than wrap into the base field.
        assert_eq!(encode_stfsu(0, 11, -4), [0xD4, 0x0B, 0xFF, 0xFC]);
    }

    /// **W-JSON — `sthu` against the byte real `c2` emitted**, and the ONE-BIT
    /// separation from `sth`.
    ///
    /// Primary 45 against 44. That bit is a pointer bump the caller must then
    /// not emit itself, so confusing the two is either a lost increment or a
    /// doubled one — in an obj that links.
    #[test]
    fn w_json_sthu_matches_the_reference_obj_and_is_one_bit_from_sth() {
        // `?GetBuffer@JsonWriter@@QAAJPAGPAK@Z` +0xa8, `work/w-json/probe/ref.obj`.
        assert_eq!(encode_sthu(9, 4, 2), [0xB5, 0x24, 0x00, 0x02]);
        assert_eq!(encode_sth(9, 4, 2), [0xB1, 0x24, 0x00, 0x02]);
        assert_ne!(encode_sthu(9, 4, 2), encode_sth(9, 4, 2));
        // …and the body's other two `sthu` sites, which separate `rS` and `rA`.
        assert_eq!(encode_sthu(7, 4, 2), [0xB4, 0xE4, 0x00, 0x02]);
        assert_eq!(encode_sthu(28, 11, 2), [0xB7, 0x8B, 0x00, 0x02]);
    }

    /// **W-OSFINFO — the record form of `rlwinm` against the byte real `c2`
    /// emitted**, and the one-bit separation from the non-record form.
    ///
    /// A dropped `Rc` bit is the failure this pins: the masked value would be
    /// identical, `cr0` would hold whatever the last instruction left there, and
    /// the `bt 2` below it would branch on a stale bit. The program is wrong and
    /// the obj still links.
    #[test]
    fn w_osfinfo_clrlwi_record_matches_the_reference_obj() {
        // `_free_osfhnd` +0x48, `work/w-osfinfo/ref/osfinfo/dis.txt`.
        assert_eq!(encode_clrlwi_record(10, 10, 31), [0x55, 0x4A, 0x07, 0xFF]);
        // The non-record form of the same mask differs in exactly the Rc bit.
        assert_eq!(encode_clrlwi31(10, 10), [0x55, 0x4A, 0x07, 0xFE]);
        assert_eq!(encode_clrlwi_record(10, 10, 31)[3] & 1, 1);
        assert_eq!(encode_clrlwi31(10, 10)[3] & 1, 0);
        // The mask width is a parameter, not a constant: `& 3` is `clrlwi. ,30`.
        assert_eq!(encode_clrlwi_record(10, 10, 30), encode_rlwinm_record(10, 10, 0, 30, 31));
        // …and the *non*-record `rlwinm` this body also emits, so the two
        // spellings of the same instruction family are pinned side by side:
        // `clrlwi r11,r3,27` (+0x34) and `slwi r9,r11,2` (+0x2c).
        assert_eq!(encode_rlwinm(11, 3, 0, 27, 31), [0x54, 0x6B, 0x06, 0xFE]);
        assert_eq!(encode_rlwinm(9, 11, 2, 0, 29), [0x55, 0x69, 0x10, 0x3A]);
    }

    /// Every word below is TRANSCRIBED from `work/w-build/probe/bits.cod` /
    /// `bits2.cod` — c2's own `/FAsc` listing at the workload's flags — and is
    /// checked against the encoder rather than against the table that produced
    /// it. That is the discipline `expr_opcode_name`'s header states: a value
    /// derived from the thing it validates checks nothing.
    ///
    /// The sixteen rows deliberately include every one that distinguishes the
    /// **RA-destination** X-form layout from [`encode_add`]'s RT-destination
    /// one — `and r11,r3,r4` and `and r3,r11,r5` differ in exactly the two
    /// fields that would swap.
    #[test]
    fn the_logical_xforms_reproduce_their_captured_words() {
        let w = |b: [u8; 4]| u32::from_be_bytes(b);
        // and — `a & b`, and the three-address chain `a & b & c`.
        assert_eq!(w(encode_and(3, 3, 4)), 0x7c63_2038);
        assert_eq!(w(encode_and(11, 3, 4)), 0x7c6b_2038);
        assert_eq!(w(encode_and(3, 11, 5)), 0x7d63_2838);
        assert_eq!(w(encode_and(10, 5, 6)), 0x7caa_3038);
        assert_eq!(w(encode_and(3, 11, 10)), 0x7d63_5038);
        // or
        assert_eq!(w(encode_or(3, 3, 4)), 0x7c63_2378);
        assert_eq!(w(encode_or(11, 3, 4)), 0x7c6b_2378);
        assert_eq!(w(encode_or(3, 11, 5)), 0x7d63_2b78);
        assert_eq!(w(encode_or(3, 11, 10)), 0x7d63_5378);
        // xor
        assert_eq!(w(encode_xor(3, 3, 4)), 0x7c63_2278);
        assert_eq!(w(encode_xor(3, 11, 5)), 0x7d63_2a78);
        // slw / srw / sraw
        assert_eq!(w(encode_slw(3, 3, 4)), 0x7c63_2030);
        assert_eq!(w(encode_slw(3, 11, 5)), 0x7d63_2830);
        assert_eq!(w(encode_srw(3, 3, 4)), 0x7c63_2430);
        assert_eq!(w(encode_sraw(3, 3, 4)), 0x7c63_2630);
        assert_eq!(w(encode_sraw(11, 3, 4)), 0x7c6b_2630);
    }

    /// The layout hazard, stated as a test rather than as a comment.
    ///
    /// `and r11, r3, r4` and `and r3, r11, r4` are DIFFERENT instructions, and
    /// an encoder that used [`encode_add`]'s RT-destination field order would
    /// produce the second when asked for the first. The bytes are valid either
    /// way, so nothing downstream — not `fuzzy%`, not a disassembler — would
    /// flag it; only a byte compare against c2 would, and only on a body where
    /// the destination and the left operand happen to differ.
    #[test]
    fn the_logical_destination_field_is_ra_and_not_rt() {
        assert_ne!(encode_and(11, 3, 4), encode_and(3, 11, 4));
        // What the WRONG layout would have produced for `and r11,r3,r4`, spelled
        // out: RT=11 at bits 6-10, RA=3 at 11-15 — which is `and r3,r11,r4`.
        let wrong = (31u32 << 26) | (11 << 21) | (3 << 16) | (4 << 11) | (28 << 1);
        assert_eq!(wrong.to_be_bytes(), encode_and(3, 11, 4));
        assert_ne!(wrong.to_be_bytes(), encode_and(11, 3, 4));
        // …and the captured byte says which one c2 emits for `a & b & c`'s first
        // instruction: `7c6b2038`, this encoder's `encode_and(11, 3, 4)`.
        assert_eq!(u32::from_be_bytes(encode_and(11, 3, 4)), 0x7c6b_2038);
    }

    /// `sraw` and `srw` are one IL opcode apart from each other by NOTHING —
    /// the distinction lives in the operand type — so the two encoders must not
    /// be interchangeable and the test says so with the captured pair.
    #[test]
    fn the_two_right_shifts_are_different_instructions() {
        assert_ne!(encode_sraw(3, 3, 4), encode_srw(3, 3, 4));
        assert_eq!(u32::from_be_bytes(encode_sraw(3, 3, 4)), 0x7c63_2630); // int
        assert_eq!(u32::from_be_bytes(encode_srw(3, 3, 4)), 0x7c63_2430); // unsigned
    }
}

// ---------------------------------------------------------------------------
// S1's `#[cfg(test)]` cross-check: the incumbent encoders, kept VERBATIM.
// ---------------------------------------------------------------------------

/// **The 81 hand-written encoders this file shipped until 2026-08-22, byte for
/// byte, as a test-only oracle.**
///
/// Lane `w-s1` (Phase 0 slice **S1**) re-expressed the live path through
/// [`super::super::mop`]'s general composition. The construct-rung criterion is
/// a **required-zero byte delta**, and board **#3346** is the condition that
/// makes such a delta *evidence* rather than a tautology: the re-expression has
/// to be on a path the gate exercises, with no incumbent left to fall back to.
/// It is — [`select_function`](super::super::select::select_function) reaches
/// every one of these through the new composition and there is no second route.
///
/// So the incumbent survives **only here**, and only as the thing the new path
/// is graded against. Nothing outside `#[cfg(test)]` can call it, which is the
/// half board #3336 found missing on `ir0`: a re-expression whose old path is
/// still reachable has not been graded, it has been *duplicated*.
///
/// **Why verbatim and not "cleaned up".** These bodies are the accumulated
/// black-box record — every primary opcode and every XO in them was transcribed
/// from a captured obj one fact at a time, and the per-function evidence notes
/// above cite the captures. Re-deriving them from the same read table the new
/// path uses would make the cross-check compare c2's table to itself and pass
/// by construction. The whole value of this module is that it is an
/// **independent** derivation.
#[cfg(test)]
#[allow(clippy::identity_op, clippy::erasing_op)]
mod incumbent {
    use super::{B_MAX_DISP, BC_MAX_DISP, BO_DNZ};

    pub(super) fn inc_add(rd: u8, ra: u8, rb: u8) -> [u8; 4] {
        let word: u32 = (31 << 26)
            | ((rd as u32 & 0x1F) << 21)
            | ((ra as u32 & 0x1F) << 16)
            | ((rb as u32 & 0x1F) << 11)
            | (266 << 1);
        word.to_be_bytes()
    }

    pub(super) fn inc_mullw(rd: u8, ra: u8, rb: u8) -> [u8; 4] {
        let word: u32 = (31 << 26)
            | ((rd as u32 & 0x1F) << 21)
            | ((ra as u32 & 0x1F) << 16)
            | ((rb as u32 & 0x1F) << 11)
            | (235 << 1);
        word.to_be_bytes()
    }

    pub(super) fn inc_subf(rd: u8, ra: u8, rb: u8) -> [u8; 4] {
        let word: u32 = (31 << 26)
            | ((rd as u32 & 0x1F) << 21)
            | ((ra as u32 & 0x1F) << 16)
            | ((rb as u32 & 0x1F) << 11)
            | (40 << 1);
        word.to_be_bytes()
    }

    pub(super) fn inc_logical_x(xo: u32, ra_dest: u8, rs_lhs: u8, rb_rhs: u8) -> [u8; 4] {
        let word: u32 = (31 << 26)
            | ((rs_lhs as u32 & 0x1F) << 21)
            | ((ra_dest as u32 & 0x1F) << 16)
            | ((rb_rhs as u32 & 0x1F) << 11)
            | (xo << 1);
        word.to_be_bytes()
    }

    pub(super) fn inc_and(dest: u8, lhs: u8, rhs: u8) -> [u8; 4] {
        inc_logical_x(28, dest, lhs, rhs)
    }

    pub(super) fn inc_or(dest: u8, lhs: u8, rhs: u8) -> [u8; 4] {
        inc_logical_x(444, dest, lhs, rhs)
    }

    pub(super) fn inc_xor(dest: u8, lhs: u8, rhs: u8) -> [u8; 4] {
        inc_logical_x(316, dest, lhs, rhs)
    }

    pub(super) fn inc_slw(dest: u8, lhs: u8, rhs: u8) -> [u8; 4] {
        inc_logical_x(24, dest, lhs, rhs)
    }

    pub(super) fn inc_srw(dest: u8, lhs: u8, rhs: u8) -> [u8; 4] {
        inc_logical_x(536, dest, lhs, rhs)
    }

    pub(super) fn inc_sraw(dest: u8, lhs: u8, rhs: u8) -> [u8; 4] {
        inc_logical_x(792, dest, lhs, rhs)
    }

    pub(super) fn inc_addi(rd: u8, ra: u8, si: i16) -> [u8; 4] {
        let word: u32 =
            (14 << 26) | ((rd as u32 & 0x1F) << 21) | ((ra as u32 & 0x1F) << 16) | (si as u16 as u32);
        word.to_be_bytes()
    }

    pub(super) fn inc_addis(rd: u8, ra: u8, si: i16) -> [u8; 4] {
        let word: u32 =
            (15 << 26) | ((rd as u32 & 0x1F) << 21) | ((ra as u32 & 0x1F) << 16) | (si as u16 as u32);
        word.to_be_bytes()
    }

    pub(super) fn inc_ori(ra: u8, rs: u8, ui: u16) -> [u8; 4] {
        let word: u32 =
            (24 << 26) | ((rs as u32 & 0x1F) << 21) | ((ra as u32 & 0x1F) << 16) | (ui as u32);
        word.to_be_bytes()
    }

    pub(super) fn inc_blr() -> [u8; 4] {
        0x4E80_0020u32.to_be_bytes()
    }

    pub(super) fn inc_bclr(bo: u8, bi: u8) -> [u8; 4] {
        let word: u32 =
            (19 << 26) | ((bo as u32 & 0x1F) << 21) | ((bi as u32 & 0x1F) << 16) | (16 << 1);
        word.to_be_bytes()
    }

    pub(super) fn inc_mtctr(rs: u8) -> [u8; 4] {
        const SPR_CTR: u32 = 9;
        let spr_field = ((SPR_CTR & 0x1F) << 5) | ((SPR_CTR >> 5) & 0x1F);
        let word: u32 =
            (31 << 26) | ((rs as u32 & 0x1F) << 21) | (spr_field << 11) | (467 << 1);
        word.to_be_bytes()
    }

    pub(super) fn inc_bctrl() -> [u8; 4] {
        const BO_ALWAYS: u32 = 20;
        const XO_BCCTR: u32 = 528;
        let word: u32 = (19 << 26) | (BO_ALWAYS << 21) | (XO_BCCTR << 1) | 1;
        word.to_be_bytes()
    }

    pub(super) fn inc_bdnz(disp: i32) -> Option<[u8; 4]> {
        inc_bc(BO_DNZ, 0, disp)
    }

    pub(super) fn inc_lwz(rd: u8, ra: u8, d: i16) -> [u8; 4] {
        let word: u32 =
            (32 << 26) | ((rd as u32 & 0x1F) << 21) | ((ra as u32 & 0x1F) << 16) | (d as u16 as u32);
        word.to_be_bytes()
    }

    pub(super) fn inc_lbz(rd: u8, ra: u8, d: i16) -> [u8; 4] {
        let word: u32 =
            (34 << 26) | ((rd as u32 & 0x1F) << 21) | ((ra as u32 & 0x1F) << 16) | (d as u16 as u32);
        word.to_be_bytes()
    }

    pub(super) fn inc_lhz(rd: u8, ra: u8, d: i16) -> [u8; 4] {
        let word: u32 =
            (40 << 26) | ((rd as u32 & 0x1F) << 21) | ((ra as u32 & 0x1F) << 16) | (d as u16 as u32);
        word.to_be_bytes()
    }

    pub(super) fn inc_ld(rd: u8, ra: u8, ds: i16) -> [u8; 4] {
        let word: u32 = (58 << 26)
            | ((rd as u32 & 0x1F) << 21)
            | ((ra as u32 & 0x1F) << 16)
            | ((ds as u16 as u32) & 0xFFFC);
        word.to_be_bytes()
    }

    pub(super) fn inc_extsb(ra: u8, rs: u8) -> [u8; 4] {
        xo31(rs, ra, 0, 954)
    }

    pub(super) fn inc_extsb_record(ra: u8, rs: u8) -> [u8; 4] {
        let mut w = u32::from_be_bytes(xo31(rs, ra, 0, 954));
        w |= 1;
        w.to_be_bytes()
    }

    pub(super) fn inc_extsh(ra: u8, rs: u8) -> [u8; 4] {
        xo31(rs, ra, 0, 922)
    }

    pub(super) fn inc_stw(rs: u8, ra: u8, d: i16) -> [u8; 4] {
        let word: u32 =
            (36 << 26) | ((rs as u32 & 0x1F) << 21) | ((ra as u32 & 0x1F) << 16) | (d as u16 as u32);
        word.to_be_bytes()
    }

    pub(super) fn inc_stb(rs: u8, ra: u8, d: i16) -> [u8; 4] {
        let word: u32 =
            (38 << 26) | ((rs as u32 & 0x1F) << 21) | ((ra as u32 & 0x1F) << 16) | (d as u16 as u32);
        word.to_be_bytes()
    }

    pub(super) fn inc_sth(rs: u8, ra: u8, d: i16) -> [u8; 4] {
        let word: u32 =
            (44 << 26) | ((rs as u32 & 0x1F) << 21) | ((ra as u32 & 0x1F) << 16) | (d as u16 as u32);
        word.to_be_bytes()
    }

    pub(super) fn inc_sthu(rs: u8, ra: u8, d: i16) -> [u8; 4] {
        let word: u32 =
            (45 << 26) | ((rs as u32 & 0x1F) << 21) | ((ra as u32 & 0x1F) << 16) | (d as u16 as u32);
        word.to_be_bytes()
    }

    pub(super) fn inc_lhzx(rd: u8, ra: u8, rb: u8) -> [u8; 4] {
        let word: u32 = (31 << 26)
            | ((rd as u32 & 0x1F) << 21)
            | ((ra as u32 & 0x1F) << 16)
            | ((rb as u32 & 0x1F) << 11)
            | (279 << 1);
        word.to_be_bytes()
    }

    pub(super) fn inc_addic(rd: u8, ra: u8, si: i16) -> [u8; 4] {
        let word: u32 =
            (12 << 26) | ((rd as u32 & 0x1F) << 21) | ((ra as u32 & 0x1F) << 16) | (si as u16 as u32);
        word.to_be_bytes()
    }

    pub(super) fn inc_subfic(rd: u8, ra: u8, si: i16) -> [u8; 4] {
        let word: u32 =
            (8 << 26) | ((rd as u32 & 0x1F) << 21) | ((ra as u32 & 0x1F) << 16) | (si as u16 as u32);
        word.to_be_bytes()
    }

    pub(super) fn inc_subfc(rd: u8, ra: u8, rb: u8) -> [u8; 4] {
        xo31(rd, ra, rb, 8)
    }

    pub(super) fn inc_subfe(rd: u8, ra: u8, rb: u8) -> [u8; 4] {
        xo31(rd, ra, rb, 136)
    }

    pub(super) fn inc_addze(rd: u8, ra: u8) -> [u8; 4] {
        xo31(rd, ra, 0, 202)
    }

    pub(super) fn inc_adde(rd: u8, ra: u8, rb: u8) -> [u8; 4] {
        xo31(rd, ra, rb, 138)
    }

    pub(super) fn inc_subfze(rd: u8, ra: u8) -> [u8; 4] {
        xo31(rd, ra, 0, 200)
    }

    pub(super) fn inc_srawi(ra: u8, rs: u8, sh: u8) -> [u8; 4] {
        let word: u32 = (31 << 26)
            | ((rs as u32 & 0x1F) << 21)
            | ((ra as u32 & 0x1F) << 16)
            | ((sh as u32 & 0x1F) << 11)
            | (824 << 1);
        word.to_be_bytes()
    }

    pub(super) fn inc_neg(rd: u8, ra: u8) -> [u8; 4] {
        xo31(rd, ra, 0, 104)
    }

    pub(super) fn inc_andc(ra: u8, rs: u8, rb: u8) -> [u8; 4] {
        xo31(rs, ra, rb, 60)
    }

    pub(super) fn inc_orc(ra: u8, rs: u8, rb: u8) -> [u8; 4] {
        xo31(rs, ra, rb, 412)
    }

    pub(super) fn inc_eqv(ra: u8, rs: u8, rb: u8) -> [u8; 4] {
        xo31(rs, ra, rb, 284)
    }

    pub(super) fn inc_cntlzw(ra: u8, rs: u8) -> [u8; 4] {
        xo31(rs, ra, 0, 26)
    }

    pub(super) fn inc_xori(ra: u8, rs: u8, ui: u16) -> [u8; 4] {
        let word: u32 =
            (26 << 26) | ((rs as u32 & 0x1F) << 21) | ((ra as u32 & 0x1F) << 16) | (ui as u32);
        word.to_be_bytes()
    }

    pub(super) fn inc_rlwinm(ra: u8, rs: u8, sh: u8, mb: u8, me: u8) -> [u8; 4] {
        let word: u32 = (21 << 26)
            | ((rs as u32 & 0x1F) << 21)
            | ((ra as u32 & 0x1F) << 16)
            | ((sh as u32 & 0x1F) << 11)
            | ((mb as u32 & 0x1F) << 6)
            | ((me as u32 & 0x1F) << 1);
        word.to_be_bytes()
    }

    pub(super) fn inc_rlwimi(ra: u8, rs: u8, sh: u8, mb: u8, me: u8) -> [u8; 4] {
        let word: u32 = (20 << 26)
            | ((rs as u32 & 0x1F) << 21)
            | ((ra as u32 & 0x1F) << 16)
            | ((sh as u32 & 0x1F) << 11)
            | ((mb as u32 & 0x1F) << 6)
            | ((me as u32 & 0x1F) << 1);
        word.to_be_bytes()
    }

    pub(super) fn inc_rldicl(ra: u8, rs: u8, sh: u8, mb: u8) -> [u8; 4] {
        let sh = sh as u32 & 0x3F;
        let mb = mb as u32 & 0x3F;
        let word: u32 = (30 << 26)
            | ((rs as u32 & 0x1F) << 21)
            | ((ra as u32 & 0x1F) << 16)
            | ((sh & 0x1F) << 11)
            | (((mb & 0x1F) << 1 | (mb >> 5)) << 5)
            | ((sh >> 5) << 1);
        word.to_be_bytes()
    }

    pub(super) fn inc_rldimi(ra: u8, rs: u8, sh: u8, mb: u8) -> [u8; 4] {
        let sh = sh as u32 & 0x3F;
        let mb = mb as u32 & 0x3F;
        let word: u32 = (30 << 26)
            | ((rs as u32 & 0x1F) << 21)
            | ((ra as u32 & 0x1F) << 16)
            | ((sh & 0x1F) << 11)
            | (((mb & 0x1F) << 1 | (mb >> 5)) << 5)
            | (3 << 2)
            | ((sh >> 5) << 1);
        word.to_be_bytes()
    }

    pub(super) fn inc_stdu(rs: u8, ra: u8, ds: i16) -> [u8; 4] {
        debug_assert_eq!(ds & 3, 0, "a DS displacement's low two bits are implied zero");
        let word: u32 = (62 << 26)
            | ((rs as u32 & 0x1F) << 21)
            | ((ra as u32 & 0x1F) << 16)
            | ((ds as u16 as u32) & 0xFFFC)
            | 1;
        word.to_be_bytes()
    }

    pub(super) fn inc_stdx(rs: u8, ra: u8, rb: u8) -> [u8; 4] {
        let word: u32 = (31 << 26)
            | ((rs as u32 & 0x1F) << 21)
            | ((ra as u32 & 0x1F) << 16)
            | ((rb as u32 & 0x1F) << 11)
            | (149 << 1);
        word.to_be_bytes()
    }

    pub(super) fn inc_addic_record(rd: u8, ra: u8, si: i16) -> [u8; 4] {
        let word: u32 =
            (13 << 26) | ((rd as u32 & 0x1F) << 21) | ((ra as u32 & 0x1F) << 16) | (si as u16 as u32);
        word.to_be_bytes()
    }

    pub(super) fn inc_rlwinm_record(ra: u8, rs: u8, sh: u8, mb: u8, me: u8) -> [u8; 4] {
        let mut w = inc_rlwinm(ra, rs, sh, mb, me);
        w[3] |= 1;
        w
    }

    pub(super) fn fp_primary(double: bool) -> u32 {
        if double {
            63
        } else {
            59
        }
    }

    pub(super) fn fp_a_form(double: bool, fd: u8, fa: u8, fb: u8, fc: u8, xo: u32) -> [u8; 4] {
        let word: u32 = (fp_primary(double) << 26)
            | ((fd as u32 & 0x1F) << 21)
            | ((fa as u32 & 0x1F) << 16)
            | ((fb as u32 & 0x1F) << 11)
            | ((fc as u32 & 0x1F) << 6)
            | (xo << 1);
        word.to_be_bytes()
    }

    pub(super) fn inc_fadd(double: bool, fd: u8, fa: u8, fb: u8) -> [u8; 4] {
        fp_a_form(double, fd, fa, fb, 0, 21)
    }

    pub(super) fn inc_fsub(double: bool, fd: u8, fa: u8, fb: u8) -> [u8; 4] {
        fp_a_form(double, fd, fa, fb, 0, 20)
    }

    pub(super) fn inc_fmul(double: bool, fd: u8, fa: u8, fc: u8) -> [u8; 4] {
        fp_a_form(double, fd, fa, 0, fc, 25)
    }

    pub(super) fn inc_fdiv(double: bool, fd: u8, fa: u8, fb: u8) -> [u8; 4] {
        fp_a_form(double, fd, fa, fb, 0, 18)
    }

    pub(super) fn inc_lfsx(fd: u8, ra: u8, rb: u8) -> [u8; 4] {
        xo31(fd, ra, rb, 535)
    }

    pub(super) fn inc_stfsx(fs: u8, ra: u8, rb: u8) -> [u8; 4] {
        xo31(fs, ra, rb, 663)
    }

    pub(super) fn inc_stfsu(fs: u8, ra: u8, d: i16) -> [u8; 4] {
        let word: u32 = (53u32 << 26)
            | ((fs as u32 & 0x1F) << 21)
            | ((ra as u32 & 0x1F) << 16)
            | (d as u16 as u32);
        word.to_be_bytes()
    }

    pub(super) fn inc_stfs(double: bool, fs: u8, ra: u8, d: i16) -> [u8; 4] {
        let primary: u32 = if double { 54 } else { 52 };
        let word: u32 = (primary << 26)
            | ((fs as u32 & 0x1F) << 21)
            | ((ra as u32 & 0x1F) << 16)
            | (d as u16 as u32);
        word.to_be_bytes()
    }

    pub(super) fn inc_fmr(fd: u8, fb: u8) -> [u8; 4] {
        let word: u32 =
            (63u32 << 26) | ((fd as u32 & 0x1F) << 21) | ((fb as u32 & 0x1F) << 11) | (72u32 << 1);
        word.to_be_bytes()
    }

    pub(super) fn inc_frsp(fd: u8, fb: u8) -> [u8; 4] {
        let word: u32 =
            (63u32 << 26) | ((fd as u32 & 0x1F) << 21) | ((fb as u32 & 0x1F) << 11) | (12u32 << 1);
        word.to_be_bytes()
    }

    pub(super) fn inc_lfs(double: bool, fd: u8, ra: u8, d: i16) -> [u8; 4] {
        let primary: u32 = if double { 50 } else { 48 };
        let word: u32 = (primary << 26)
            | ((fd as u32 & 0x1F) << 21)
            | ((ra as u32 & 0x1F) << 16)
            | (d as u16 as u32);
        word.to_be_bytes()
    }

    pub(super) fn xo31(first: u8, second: u8, rb: u8, xo: u32) -> [u8; 4] {
        let word: u32 = (31 << 26)
            | ((first as u32 & 0x1F) << 21)
            | ((second as u32 & 0x1F) << 16)
            | ((rb as u32 & 0x1F) << 11)
            | (xo << 1);
        word.to_be_bytes()
    }

    pub(super) fn inc_std(rs: u8, ra: u8, ds: i16) -> [u8; 4] {
        let word: u32 =
            (62 << 26) | ((rs as u32 & 0x1F) << 21) | ((ra as u32 & 0x1F) << 16) | ((ds as u16 as u32) & 0xFFFC);
        word.to_be_bytes()
    }

    pub(super) fn inc_stfd(frs: u8, ra: u8, d: i16) -> [u8; 4] {
        let word: u32 =
            (54 << 26) | ((frs as u32 & 0x1F) << 21) | ((ra as u32 & 0x1F) << 16) | (d as u16 as u32);
        word.to_be_bytes()
    }

    pub(super) fn inc_lfd(frd: u8, ra: u8, d: i16) -> [u8; 4] {
        let word: u32 =
            (50 << 26) | ((frd as u32 & 0x1F) << 21) | ((ra as u32 & 0x1F) << 16) | (d as u16 as u32);
        word.to_be_bytes()
    }

    pub(super) fn inc_stwu(rs: u8, ra: u8, d: i16) -> [u8; 4] {
        let word: u32 =
            (37 << 26) | ((rs as u32 & 0x1F) << 21) | ((ra as u32 & 0x1F) << 16) | (d as u16 as u32);
        word.to_be_bytes()
    }

    pub(super) fn inc_mr(ra: u8, rs: u8) -> [u8; 4] {
        xo31(rs, ra, rs, 444)
    }

    pub(super) fn inc_mr_record(ra: u8, rs: u8) -> [u8; 4] {
        let mut w = u32::from_be_bytes(xo31(rs, ra, rs, 444));
        w |= 1;
        w.to_be_bytes()
    }

    pub(super) fn inc_mulli(rd: u8, ra: u8, simm: i16) -> [u8; 4] {
        let word: u32 =
            (7 << 26) | ((rd as u32 & 0x1F) << 21) | ((ra as u32 & 0x1F) << 16) | (simm as u16 as u32);
        word.to_be_bytes()
    }

    pub(super) fn inc_lbzu(rd: u8, ra: u8, d: i16) -> [u8; 4] {
        let word: u32 =
            (35 << 26) | ((rd as u32 & 0x1F) << 21) | ((ra as u32 & 0x1F) << 16) | (d as u16 as u32);
        word.to_be_bytes()
    }

    pub(super) fn inc_divw(rd: u8, ra: u8, rb: u8) -> [u8; 4] {
        xo31(rd, ra, rb, 491)
    }

    pub(super) fn inc_divwu(rd: u8, ra: u8, rb: u8) -> [u8; 4] {
        xo31(rd, ra, rb, 459)
    }

    pub(super) fn inc_twi(to: u8, ra: u8, simm: i16) -> [u8; 4] {
        let word: u32 =
            (3 << 26) | ((to as u32 & 0x1F) << 21) | ((ra as u32 & 0x1F) << 16) | (simm as u16 as u32);
        word.to_be_bytes()
    }

    pub(super) fn inc_bc(bo: u8, bi: u8, disp: i32) -> Option<[u8; 4]> {
        if disp % 4 != 0 || !(-BC_MAX_DISP - 4..=BC_MAX_DISP).contains(&disp) {
            return None;
        }
        let word: u32 = 0x4000_0000
            | ((bo as u32 & 0x1F) << 21)
            | ((bi as u32 & 0x1F) << 16)
            | (disp as u32 & 0xFFFC);
        Some(word.to_be_bytes())
    }

    pub(super) fn inc_b_intra(disp: i32) -> Option<[u8; 4]> {
        if disp % 4 != 0 || !(-B_MAX_DISP - 4..=B_MAX_DISP).contains(&disp) {
            return None;
        }
        let word: u32 = 0x4800_0000 | (disp as u32 & 0x03FF_FFFC);
        Some(word.to_be_bytes())
    }

    pub(super) fn inc_cmpwi(crf: u8, ra: u8, simm: i16) -> [u8; 4] {
        let word: u32 = (11 << 26)
            | ((crf as u32 & 7) << 23)
            | ((ra as u32 & 0x1F) << 16)
            | (simm as u16 as u32);
        word.to_be_bytes()
    }

    pub(super) fn inc_cmplwi(crf: u8, ra: u8, uimm: u16) -> [u8; 4] {
        let word: u32 = (10 << 26)
            | ((crf as u32 & 7) << 23)
            | ((ra as u32 & 0x1F) << 16)
            | (uimm as u32);
        word.to_be_bytes()
    }

    pub(super) fn inc_cmpw(crf: u8, ra: u8, rb: u8) -> [u8; 4] {
        let word: u32 = (31 << 26)
            | ((crf as u32 & 7) << 23)
            | ((ra as u32 & 0x1F) << 16)
            | ((rb as u32 & 0x1F) << 11);
        word.to_be_bytes()
    }

    pub(super) fn inc_cmplw(crf: u8, ra: u8, rb: u8) -> [u8; 4] {
        let word: u32 = (31 << 26)
            | ((crf as u32 & 7) << 23)
            | ((ra as u32 & 0x1F) << 16)
            | ((rb as u32 & 0x1F) << 11)
            | (32 << 1);
        word.to_be_bytes()
    }

    pub(super) fn inc_lwzx(rd: u8, ra: u8, rb: u8) -> [u8; 4] {
        let word: u32 = (31 << 26)
            | ((rd as u32 & 0x1F) << 21)
            | ((ra as u32 & 0x1F) << 16)
            | ((rb as u32 & 0x1F) << 11)
            | (23 << 1);
        word.to_be_bytes()
    }
}

#[cfg(test)]
mod cross_check {
    use super::*;

    /// The register numbers a 5-bit field must round-trip, plus the ones that
    /// separate a 5-bit field from a 4-bit one.
    ///
    /// **Board #3379's own lesson, applied.** Its purpose-built 46-word probe
    /// could not distinguish a 4-bit `RB` from a 5-bit one *because no word in
    /// it used a register >= 16* — "a control is only capable of failing on the
    /// population you ran it on." So this sweep is the whole 0..32, not a
    /// sample, and the displacement list carries both signs and both boundary
    /// magnitudes.
    const REGS: [u8; 32] = [
        0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23,
        24, 25, 26, 27, 28, 29, 30, 31,
    ];
    const DISPS: [i16; 12] = [
        0, 1, 2, 4, -4, 8, -8, 0x7FFF, -0x8000, 0x7FFC, -0x7FFC, 0x1234,
    ];

    /// Every three-register encoder, over the **whole** 32x32x32 domain.
    ///
    /// 32,768 cells each. This is the assertion that makes S1's required-zero
    /// byte delta provable in the PORTABLE lane rather than merely observed by
    /// a toolchain gate: a toolchain-gated fixture pins nothing where there is
    /// no toolchain, and the gate only exercises the operand combinations its
    /// fixtures happen to contain.
    #[test]
    fn every_three_register_encoder_reproduces_its_incumbent_word() {
        for &a in REGS.iter() {
            for &b in REGS.iter() {
                for &c in REGS.iter() {
                    assert_eq!(encode_add(a, b, c), incumbent::inc_add(a, b, c), "add {a} {b} {c}");
                    assert_eq!(encode_mullw(a, b, c), incumbent::inc_mullw(a, b, c));
                    assert_eq!(encode_subf(a, b, c), incumbent::inc_subf(a, b, c));
                    assert_eq!(encode_subfc(a, b, c), incumbent::inc_subfc(a, b, c));
                    assert_eq!(encode_subfe(a, b, c), incumbent::inc_subfe(a, b, c));
                    assert_eq!(encode_adde(a, b, c), incumbent::inc_adde(a, b, c));
                    assert_eq!(encode_divw(a, b, c), incumbent::inc_divw(a, b, c));
                    assert_eq!(encode_divwu(a, b, c), incumbent::inc_divwu(a, b, c));
                    assert_eq!(encode_and(a, b, c), incumbent::inc_and(a, b, c), "and {a} {b} {c}");
                    assert_eq!(encode_or(a, b, c), incumbent::inc_or(a, b, c));
                    assert_eq!(encode_xor(a, b, c), incumbent::inc_xor(a, b, c));
                    assert_eq!(encode_slw(a, b, c), incumbent::inc_slw(a, b, c));
                    assert_eq!(encode_srw(a, b, c), incumbent::inc_srw(a, b, c));
                    assert_eq!(encode_sraw(a, b, c), incumbent::inc_sraw(a, b, c));
                    assert_eq!(encode_andc(a, b, c), incumbent::inc_andc(a, b, c));
                    assert_eq!(encode_orc(a, b, c), incumbent::inc_orc(a, b, c));
                    assert_eq!(encode_eqv(a, b, c), incumbent::inc_eqv(a, b, c));
                    assert_eq!(encode_lhzx(a, b, c), incumbent::inc_lhzx(a, b, c));
                    assert_eq!(encode_lwzx(a, b, c), incumbent::inc_lwzx(a, b, c));
                    assert_eq!(encode_lfsx(a, b, c), incumbent::inc_lfsx(a, b, c));
                    assert_eq!(encode_stdx(a, b, c), incumbent::inc_stdx(a, b, c));
                    assert_eq!(encode_stfsx(a, b, c), incumbent::inc_stfsx(a, b, c));
                    assert_eq!(encode_srawi(a, b, c), incumbent::inc_srawi(a, b, c));
                }
            }
        }
    }

    /// Every two-register encoder, whole 32x32 domain.
    #[test]
    fn every_two_register_encoder_reproduces_its_incumbent_word() {
        for &a in REGS.iter() {
            for &b in REGS.iter() {
                assert_eq!(encode_addze(a, b), incumbent::inc_addze(a, b));
                assert_eq!(encode_subfze(a, b), incumbent::inc_subfze(a, b));
                assert_eq!(encode_neg(a, b), incumbent::inc_neg(a, b));
                assert_eq!(encode_extsb(a, b), incumbent::inc_extsb(a, b));
                assert_eq!(encode_extsb_record(a, b), incumbent::inc_extsb_record(a, b));
                assert_eq!(encode_extsh(a, b), incumbent::inc_extsh(a, b));
                assert_eq!(encode_cntlzw(a, b), incumbent::inc_cntlzw(a, b));
                assert_eq!(encode_fmr(a, b), incumbent::inc_fmr(a, b));
                assert_eq!(encode_frsp(a, b), incumbent::inc_frsp(a, b));
                assert_eq!(encode_mr(a, b), incumbent::inc_mr(a, b), "mr {a} {b}");
                assert_eq!(encode_mr_record(a, b), incumbent::inc_mr_record(a, b));
                assert_eq!(encode_bclr(a, b), incumbent::inc_bclr(a, b));
            }
        }
        for &r in REGS.iter() {
            assert_eq!(encode_mtctr(r), incumbent::inc_mtctr(r), "mtctr {r}");
        }
        assert_eq!(encode_blr(), incumbent::inc_blr());
        assert_eq!(encode_bctrl(), incumbent::inc_bctrl());
    }

    /// Every `(register, register, displacement)` encoder, over 32x32x12.
    ///
    /// The displacement list deliberately includes values that are **not**
    /// multiples of 4, which the DS-form encoders (`ld`, `std`, `stdu`) round
    /// down. The incumbent rounds by masking `& 0xFFFC` and the general layer
    /// by an arithmetic `>> 2`; those two functions agree on every multiple of
    /// 4 and this test is what says they also agree everywhere else, including
    /// on negatives, where a logical and an arithmetic shift part company.
    #[test]
    fn every_displacement_encoder_reproduces_its_incumbent_word() {
        for &a in REGS.iter() {
            for &b in REGS.iter() {
                for &d in DISPS.iter() {
                    assert_eq!(encode_addi(a, b, d), incumbent::inc_addi(a, b, d));
                    assert_eq!(encode_addis(a, b, d), incumbent::inc_addis(a, b, d));
                    assert_eq!(encode_addic(a, b, d), incumbent::inc_addic(a, b, d));
                    assert_eq!(encode_addic_record(a, b, d), incumbent::inc_addic_record(a, b, d));
                    assert_eq!(encode_subfic(a, b, d), incumbent::inc_subfic(a, b, d));
                    assert_eq!(encode_mulli(a, b, d), incumbent::inc_mulli(a, b, d));
                    assert_eq!(encode_lwz(a, b, d), incumbent::inc_lwz(a, b, d));
                    assert_eq!(encode_lbz(a, b, d), incumbent::inc_lbz(a, b, d));
                    assert_eq!(encode_lbzu(a, b, d), incumbent::inc_lbzu(a, b, d));
                    assert_eq!(encode_lhz(a, b, d), incumbent::inc_lhz(a, b, d));
                    assert_eq!(encode_ld(a, b, d), incumbent::inc_ld(a, b, d), "ld {a} {b} {d}");
                    assert_eq!(encode_ldr(a, b, d), incumbent::inc_ld(a, b, d));
                    assert_eq!(encode_stw(a, b, d), incumbent::inc_stw(a, b, d));
                    assert_eq!(encode_stwu(a, b, d), incumbent::inc_stwu(a, b, d));
                    assert_eq!(encode_stb(a, b, d), incumbent::inc_stb(a, b, d));
                    assert_eq!(encode_sth(a, b, d), incumbent::inc_sth(a, b, d));
                    assert_eq!(encode_sthu(a, b, d), incumbent::inc_sthu(a, b, d));
                    assert_eq!(encode_std(a, b, d), incumbent::inc_std(a, b, d), "std {a} {b} {d}");
                    assert_eq!(encode_stfd(a, b, d), incumbent::inc_stfd(a, b, d));
                    assert_eq!(encode_lfd(a, b, d), incumbent::inc_lfd(a, b, d));
                    assert_eq!(encode_stfsu(a, b, d), incumbent::inc_stfsu(a, b, d));
                    assert_eq!(encode_twi(a, b, d), incumbent::inc_twi(a, b, d));
                    assert_eq!(encode_cmpwi(a, b, d), incumbent::inc_cmpwi(a, b, d));
                    assert_eq!(
                        encode_cmplwi(a, b, d as u16),
                        incumbent::inc_cmplwi(a, b, d as u16)
                    );
                    assert_eq!(encode_ori(a, b, d as u16), incumbent::inc_ori(a, b, d as u16));
                    assert_eq!(encode_xori(a, b, d as u16), incumbent::inc_xori(a, b, d as u16));
                    assert_eq!(encode_cmpw(a, b, (d as u8) & 0x1F), incumbent::inc_cmpw(a, b, (d as u8) & 0x1F));
                    assert_eq!(encode_cmplw(a, b, (d as u8) & 0x1F), incumbent::inc_cmplw(a, b, (d as u8) & 0x1F));
                    for &dbl in [false, true].iter() {
                        assert_eq!(encode_stfs(dbl, a, b, d), incumbent::inc_stfs(dbl, a, b, d));
                        assert_eq!(encode_lfs(dbl, a, b, d), incumbent::inc_lfs(dbl, a, b, d));
                    }
                    // `stdu` debug-asserts an aligned DS field, so only feed it
                    // aligned displacements — the assertion is part of the
                    // incumbent contract and reproducing it is the point.
                    if d % 4 == 0 {
                        assert_eq!(encode_stdu(a, b, d), incumbent::inc_stdu(a, b, d));
                    }
                }
            }
        }
    }

    /// The rotate family, over the whole `SH x MB x ME` immediate domain for a
    /// fixed register pair, and the whole register domain at a fixed mask.
    #[test]
    fn every_rotate_encoder_reproduces_its_incumbent_word() {
        for sh in 0u8..32 {
            for mb in 0u8..32 {
                for me in 0u8..32 {
                    assert_eq!(
                        encode_rlwinm(3, 11, sh, mb, me),
                        incumbent::inc_rlwinm(3, 11, sh, mb, me)
                    );
                    assert_eq!(
                        encode_rlwimi(3, 11, sh, mb, me),
                        incumbent::inc_rlwimi(3, 11, sh, mb, me)
                    );
                    assert_eq!(
                        encode_rlwinm_record(3, 11, sh, mb, me),
                        incumbent::inc_rlwinm_record(3, 11, sh, mb, me)
                    );
                }
            }
        }
        for &a in REGS.iter() {
            for &b in REGS.iter() {
                assert_eq!(encode_rlwinm(a, b, 1, 31, 31), incumbent::inc_rlwinm(a, b, 1, 31, 31));
                assert_eq!(encode_srwi31(a, b), incumbent::inc_rlwinm(a, b, 1, 31, 31));
                assert_eq!(encode_clrlwi31(a, b), incumbent::inc_rlwinm(a, b, 0, 31, 31));
                for n in 0u8..32 {
                    assert_eq!(
                        encode_clrlwi_record(a, b, n),
                        incumbent::inc_rlwinm_record(a, b, 0, n, 31)
                    );
                }
                // The 64-bit rotates: both immediate fields are SIX bits and
                // are split in two DIFFERENT shapes, so the sweep has to reach
                // 32..64 on each or the split bit is never exercised.
                for sh in 0u8..64 {
                    for mb in [0u8, 1, 31, 32, 33, 63].iter().copied() {
                        assert_eq!(
                            encode_rldicl(a, b, sh, mb),
                            incumbent::inc_rldicl(a, b, sh, mb),
                            "rldicl {a} {b} {sh} {mb}"
                        );
                        assert_eq!(
                            encode_rldimi(a, b, sh, mb),
                            incumbent::inc_rldimi(a, b, sh, mb)
                        );
                    }
                }
            }
        }
    }

    /// The A-form floating-point encoders, whole register domain, both
    /// precisions. `fmul`'s multiplier lives in the **C** field and the others'
    /// second source in **B**; a general layer that placed them alike would
    /// pass every single-precision test and multiply by the wrong register.
    #[test]
    fn every_fp_encoder_reproduces_its_incumbent_word() {
        for &a in REGS.iter() {
            for &b in REGS.iter() {
                for &c in REGS.iter() {
                    for &dbl in [false, true].iter() {
                        assert_eq!(encode_fadd(dbl, a, b, c), incumbent::inc_fadd(dbl, a, b, c));
                        assert_eq!(encode_fsub(dbl, a, b, c), incumbent::inc_fsub(dbl, a, b, c));
                        assert_eq!(
                            encode_fmul(dbl, a, b, c),
                            incumbent::inc_fmul(dbl, a, b, c),
                            "fmul dbl={dbl} {a} {b} {c}"
                        );
                        assert_eq!(encode_fdiv(dbl, a, b, c), incumbent::inc_fdiv(dbl, a, b, c));
                    }
                }
            }
        }
    }

    /// The branch encoders, over every legal displacement AND across both
    /// range boundaries — the refusal is part of the contract and a general
    /// layer that widened it would be a wrong emit, not a wider class.
    #[test]
    fn every_branch_encoder_reproduces_its_incumbent_word_and_its_refusal() {
        let mut d = -BC_MAX_DISP - 8;
        while d <= BC_MAX_DISP + 8 {
            assert_eq!(encode_bc(12, 2, d), incumbent::inc_bc(12, 2, d), "bc {d}");
            assert_eq!(encode_bdnz(d), incumbent::inc_bdnz(d), "bdnz {d}");
            d += 4;
        }
        // …and the unaligned displacements, which must refuse on both sides.
        for d in [-6i32, -2, 1, 2, 3, 6, 4097].iter().copied() {
            assert_eq!(encode_bc(12, 2, d), incumbent::inc_bc(12, 2, d));
            assert_eq!(encode_b_intra(d), incumbent::inc_b_intra(d));
            assert!(encode_bc(12, 2, d).is_none());
        }
        for d in [
            0i32, 4, -4, 0x1000, -0x1000, B_MAX_DISP, -B_MAX_DISP, B_MAX_DISP - 4,
            -B_MAX_DISP - 4, B_MAX_DISP + 4, -B_MAX_DISP - 8,
        ]
        .iter()
        .copied()
        {
            assert_eq!(encode_b_intra(d), incumbent::inc_b_intra(d), "b {d}");
        }
        for bo in 0u8..32 {
            for bi in 0u8..32 {
                for d in [0i32, 4, -4, 0x7FFC, -0x8000].iter().copied() {
                    assert_eq!(encode_bc(bo, bi, d), incumbent::inc_bc(bo, bi, d));
                }
            }
        }
    }
}
