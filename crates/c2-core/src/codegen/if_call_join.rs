//! **W-CFG1 — the emitter for the two-armed `if`/`else` whose arms are CALLS.**
//!
//! The reader's accept/refuse boundary and the source shape are on
//! [`c2_il::func::body::shapes::if_call_join`]; this file is the twenty words and
//! nothing else. Everything variable in them is named in
//! [`c2_il::IfCallJoinFn`]: two compare literals, two callees, and the
//! accumulator's initial value.
//!
//! # ✘ Correction, 2026-08-15, board **#3168** — **the park's stated rationale
//! was refuted by this file's own listing, and by this file's own `PARK_REG`
//! doc, and the bytes were right the whole time**
//!
//! Filed by `w-itemf-price` (docs-only, so it could not apply it) and applied
//! here by `w-json`, which owns this directory. **It is a prose change and
//! moves no byte** — that is why it survived: `gate.sh` grades bytes,
//! `board_audit.sh` grades anchors, and a wrong *reason* attached to a right
//! *word* is invisible to both.
//!
//! The comment at `0x0c` read *"the scrutinee is read by three tests that
//! straddle two `bl`s, so it cannot stay in a **volatile argument register**"*.
//! Three things are wrong with it and **the file contained all three refutations
//! already**:
//!
//! 1. **The word it emits is `mr r10,r3`, and `select::ARG_REGS` is
//!    `[3, 4, 5, 6, 7, 8, 9, 10]`.** r10 **is** a volatile argument register, so
//!    the rule as stated forbids the register the line beneath it picks.
//! 2. **The range does not straddle a call.** Both `cmpwi cr6,r10` are at `0x18`
//!    and `0x24`; both `bl` are at `0x2c` and `0x34`. The listing four lines
//!    below the claim says so.
//! 3. **[`PARK_REG`]'s own doc contradicts it in the same file** — *"Both are
//!    volatile — nothing here is callee-saved"* — which `#3168` does not name and
//!    which is the sharpest of the three, because the two comments disagree
//!    about the same register sixty lines apart.
//!
//! **What actually gives r10, with no false premise**: r3 is clobbered by the
//! very next word (`mr r3,r4`, the hoist), r11 is taken by the accumulator home
//! at `0x14`, and the selector walking `r11, r10, …` returns r10.
//!
//! This is `#3171`'s disease at module scope: **the prose and the code quantify
//! over different sets, and nothing in this repo compares them.** `#3165` is the
//! same thing at document scope, in the section written to support the item it
//! refutes.
//!
//! ```text
//!    off  word       instruction              why it is this word
//!   ----  --------   ----------------------   -----------------------------------
//!   0x00  7d8802a6   mflr  r12                the shipped Class A 96-byte frame,
//!   0x04  9181fff8   stw   r12,-8(r1)         built by `FrameLayout` so this
//!   0x08  9421ffa0   stwu  r1,-96(r1)         class cannot disagree with W10/W11
//!   0x0c  7c6a1b78   mr    r10,r3             THE PARK: r3 is wanted by the very
//!                                             next word, and r11 is taken by the
//!                                             result home at 0x14 — see the
//!                                             correction below on what this
//!                                             comment used to claim
//!   0x10  7c832378   mr    r3,r4              THE HOIST: both arms call with the
//!                                             same argument, so its setup is in
//!                                             the ENTRY block, above every branch
//!   0x14  3960KKKK   li    r11,<acc_init>     the result HOME
//!   0x18  2f0aKKKK   cmpwi cr6,r10,<k1>       ONE compare ...
//!   0x1c  41980020   bt    24,+0x20  -> 0x3c  ... read at LT ...
//!   0x20  419a001c   bt    26,+0x1c  -> 0x3c  ... and at EQ. Both exit to the
//!                                             SAME block, which is what deleting
//!                                             the empty middle arm buys
//!   0x24  2f0aKKKK   cmpwi cr6,r10,<k2>
//!   0x28  4198000c   bt    24,+0x0c  -> 0x34
//!   0x2c  4bxxxxxx   bl    <callee_hi>        REL24
//!   0x30  48000008   b     +8        -> 0x38  the intra-section `b` to a JOIN
//!   0x34  4bxxxxxx   bl    <callee_lo>        REL24
//!   0x38  7c6b1b78   mr    r11,r3             the join: the arms' shared result
//!                                             into the home ...
//!   0x3c  7d635b78   mr    r3,r11             ... and straight back out
//!   0x40  38210060   addi  r1,r1,96
//!   0x44  8181fff8   lwz   r12,-8(r1)
//!   0x48  7d8803a6   mtlr  r12
//!   0x4c  4e800020   blr
//! ```
//!
//! **The last pair is not a peephole opportunity, it is the class.** `mr r11,r3`
//! followed immediately by `mr r3,r11` is `mr r3,r3` — semantically nothing, one
//! word each way, and every register allocator would collapse it. c2 does not,
//! because the two words belong to two different blocks: `0x38` is the join both
//! arms reach and `0x3c` is the exit the two entry guards reach *without passing
//! through it*. Collapsing them is a wrong-bytes emit that still links, which is
//! board **#1400**'s finding on `Primes.cpp` — *"the optimization a codegen lane
//! would reach for by reflex is the defect"* — arriving on a second TU. The
//! mutation test below asserts it.
//!
//! **The two `bt`s share one compare and that is asserted, not implied.** A guard
//! emitter that emitted a compare per test would produce a 21-word body: the
//! right program, four bytes long, every later displacement wrong. The shipped
//! `seq_guard_emit` does exactly that, which is why this class does not route
//! through it.
//!
//! Every branch here is **self-relative** and therefore independent of where the
//! function lands in `.text`; only the two `bl`s encode their own offset, so they
//! are the only words that need `base_off`.

use crate::codegen::calls::encode_call_branch;
use crate::codegen::encode::{
    cr_bi, encode_blr, encode_cmpwi, encode_mr, BO_TRUE, CR_BIT_EQ,
    CR_BIT_LT, CR_COMPARE,
};
use crate::codegen::frame::FrameLayout;
use crate::codegen::select::{fits_i16, out_of_class, ARG_REGS, RET_REG, SCRATCH_REG};
use crate::codegen::OptMode;
use crate::BackendError;
use c2_il::IfCallJoinFn;
use crate::codegen::block_ir::{BlockOrder, BodyLayout, Terminator};

/// The register the scrutinee is parked in for the whole body.
///
/// r10 and not r11: r11 is the result home and is live across both `bl`s too, so
/// the two cannot share. Both are volatile — nothing here is callee-saved, which
/// is why the frame stays the plain 96-byte Class A one with no GPR save area.
///
/// **This paragraph is the one that was right** (board **#3168**): r10 is
/// volatile *and* is `select::ARG_REGS[7]`, and the module header used to give
/// "it cannot stay in a volatile argument register" as the reason for choosing
/// it. See the header's 2026-08-15 correction. The scrutinee's live range ends
/// at `0x24`, above both `bl`s, so it never straddles a call at all.
const PARK_REG: u8 = 10;

/// This class's emitted body: the bytes plus the two offsets the writers need.
pub struct IfCallJoinBody {
    pub text: Vec<u8>,
    /// Absolute `.text` offsets of the two `bl` words, in **block order** —
    /// `callee_hi` first. The caller zips them against
    /// [`c2_il::IlFunction::callees`], which yields the same order for the same
    /// reason.
    pub bl_offsets: [u32; 2],
    /// Prologue length in bytes, relative to the function start: the `$M(n)`
    /// label's value and the `.pdata` record's `PrologLen`.
    pub prolog_len: u32,
}

/// Emit the twenty words.
///
/// `base_off` is the function's own offset within `.text` — zero under `/Gy`,
/// where each function is its own COMDAT, and the packed section offset
/// otherwise. It reaches only the two `bl` words.
pub fn if_call_join_text(
    j: &IfCallJoinFn,
    base_off: u32,
    mode: OptMode,
) -> Result<IfCallJoinBody, BackendError> {
    // **`/O1` only, and the refusal is a measurement.** `/Ox` and `/O2`
    // tail-duplicate a join block rather than sharing it behind a `b`, on a
    // threshold W10 bracketed with one cell either side and did not fit
    // (board row X-b; `fixtures/cpp/w10_guarded_seq.cpp`'s header carries the
    // two layouts). This body's `b` at 0x30 targets a join, so it is exactly the
    // shape that moves. `codegen::ptr_walk_loop` carries the same clause.
    if mode != OptMode::O1 {
        return Err(out_of_class(
            "if/else-with-a-join at /Ox or /O2: the join block tail-duplicates on \
             a threshold this port has not fitted (board row X-b)",
        ));
    }
    // The literals are range-checked here as well as in the reader, because this
    // is where the truncation would happen: `cmpwi`'s immediate is one signed
    // 16-bit field and `li`'s is another.
    if !fits_i16(j.k1) || !fits_i16(j.k2) || !fits_i16(j.acc_init) {
        return Err(out_of_class("if/else-with-a-join: a literal outside simm16"));
    }
    if j.params.len() != 3 {
        return Err(out_of_class("if/else-with-a-join: not three formals"));
    }

    let frame = FrameLayout::default();
    let prologue = frame.prologue()?;
    let prolog_len = prologue.len() as u32;

    // **The body is seven blocks in `BodyLayout`** — `CFG_SHAPE.md` §6.2 item A,
    // board **#3124**, and this class is the one that motivated the row.
    //
    // Every displacement here used to be an entry in a table of WORD INDICES
    // INTO THE WHOLE BODY — `I_BT_LT = 7`, `I_EXIT = 15`, and four branches
    // spelled as `(I_EXIT - I_BT_LT) * 4`. That table was correct and it was
    // eight constants of the *body*: nothing could insert a word anywhere in
    // this function without silently invalidating all eight, and the two `bl`
    // offsets carried `- 3` to convert an index back past the prologue. The
    // shape they encode is the placement order, and it is now written once, as
    // the placement order.
    let mut l = BodyLayout::new(BlockOrder::IlStatement);
    let b_entry = l.declare("entry");
    let b_eq = l.declare("eq-guard");
    let b_inner = l.declare("inner-test");
    let b_hi = l.declare("hi-arm");
    let b_lo = l.declare("lo-arm");
    let b_join = l.declare("join");
    let b_exit = l.declare("exit");

    // The entry block. `ARG_REGS[0]` and `ARG_REGS[1]` rather than literal 3 and
    // 4 so this class reads the ABI from the same place every other one does.
    let mut run = prologue;
    run.extend_from_slice(&encode_mr(PARK_REG, ARG_REGS[0]));
    run.extend_from_slice(&encode_mr(RET_REG, ARG_REGS[1]));
    run.extend_from_slice(&encode_li(SCRATCH_REG, j.acc_init as i16));
    // The two guards that share one compare. Both name the SAME successor — the
    // `mr r3,r11` at the body's exit — because the middle arm is empty, and the
    // second guard's block is therefore a compare-less block whose only content
    // is its branch. That is what "they share one compare" *is*, and it now
    // shows in the IR: `b_eq`'s run is empty and its condition source reads
    // `NotInThisBlock`.
    run.extend_from_slice(&encode_cmpwi(CR_COMPARE, PARK_REG, j.k1 as i16));
    l.place(
        b_entry,
        run,
        Terminator::Bc { bo: BO_TRUE, bi: cr_bi(CR_COMPARE, CR_BIT_LT), taken: b_exit },
    )?;
    l.place(
        b_eq,
        Vec::new(),
        Terminator::Bc { bo: BO_TRUE, bi: cr_bi(CR_COMPARE, CR_BIT_EQ), taken: b_exit },
    )?;

    // The inner test and its arms.
    l.place(
        b_inner,
        encode_cmpwi(CR_COMPARE, PARK_REG, j.k2 as i16).to_vec(),
        Terminator::Bc { bo: BO_TRUE, bi: cr_bi(CR_COMPARE, CR_BIT_LT), taken: b_lo },
    )?;
    // Each arm is one `bl`, and each `bl` is a zero placeholder: the word
    // encodes its own `.text` offset (§3.3, #191), which is the layout's answer.
    // Both sit at word 0 of their own block, which is why the `- 3` that used to
    // convert a body word index back past the prologue is gone.
    const BL_AT_ARM_HEAD: u32 = 0;
    l.place(b_hi, vec![0; 4], Terminator::B { target: b_join })?;
    l.place(b_lo, vec![0; 4], Terminator::FallThrough)?;

    // The join, then the exit. Two words that compose to nothing and are two
    // blocks — see the module header.
    l.place(b_join, encode_mr(SCRATCH_REG, RET_REG).to_vec(), Terminator::FallThrough)?;
    let mut run = encode_mr(RET_REG, SCRATCH_REG).to_vec();
    run.extend_from_slice(&frame.epilogue_run()?);
    l.place(b_exit, run, Terminator::Blr)?;

    let body = l.finish()?;
    let at_hi = body.at(b_hi, BL_AT_ARM_HEAD)?;
    let at_lo = body.at(b_lo, BL_AT_ARM_HEAD)?;
    let bl_hi = base_off + at_hi;
    let bl_lo = base_off + at_lo;
    let mut t = body.text;
    t[at_hi as usize..at_hi as usize + 4].copy_from_slice(&encode_call_branch(bl_hi));
    t[at_lo as usize..at_lo as usize + 4].copy_from_slice(&encode_call_branch(bl_lo));

    debug_assert_eq!(t.len(), 80, "the class is twenty words by construction");
    debug_assert_eq!(t[t.len() - 4..], encode_blr()[..]);
    Ok(IfCallJoinBody { text: t, bl_offsets: [bl_hi, bl_lo], prolog_len })
}

/// `li rD,K` — `addi rD,0,K`.
fn encode_li(rd: u8, k: i16) -> [u8; 4] {
    crate::codegen::encode::encode_addi(rd, 0, k)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codegen::encode::encode_bc;

    /// The twenty words real `c2.dll` emits for
    /// `?FindNodeA@@YAPBUCharGraphNode@@W4PlayBlend@@PAXM@Z`, at the dc3
    /// workload's own flags, with the two `bl` targets left at their reference
    /// displacements (`base_off = 0`, i.e. the `/Gy` COMDAT layout the workload
    /// produces).
    ///
    /// Reproduce: `c2rs compile src/system/negate_test.cpp --keep-obj … ;
    /// scripts/gt_dump.py`. The obj is not committed (CLAUDE.md forbids objs);
    /// these eighty bytes are, because a table of bytes with no producer beside
    /// it is the thing this project cannot check.
    const C2_TEXT: [u8; 80] = [
        0x7d, 0x88, 0x02, 0xa6, 0x91, 0x81, 0xff, 0xf8, 0x94, 0x21, 0xff, 0xa0, //
        0x7c, 0x6a, 0x1b, 0x78, 0x7c, 0x83, 0x23, 0x78, 0x39, 0x60, 0x00, 0x00, //
        0x2f, 0x0a, 0x00, 0x01, 0x41, 0x98, 0x00, 0x20, 0x41, 0x9a, 0x00, 0x1c, //
        0x2f, 0x0a, 0x00, 0x02, 0x41, 0x98, 0x00, 0x0c, 0x4b, 0xff, 0xff, 0xd5, //
        0x48, 0x00, 0x00, 0x08, 0x4b, 0xff, 0xff, 0xcd, 0x7c, 0x6b, 0x1b, 0x78, //
        0x7d, 0x63, 0x5b, 0x78, 0x38, 0x21, 0x00, 0x60, 0x81, 0x81, 0xff, 0xf8, //
        0x7d, 0x88, 0x03, 0xa6, 0x4e, 0x80, 0x00, 0x20, //
    ];

    fn negate_test_a() -> IfCallJoinFn {
        IfCallJoinFn {
            params: vec![0xee09, 0xef09, 0xf009],
            k1: 1,
            k2: 2,
            acc_init: 0,
            callee_hi: "?FindLast@@YAPBUCharGraphNode@@PAXM@Z".into(),
            callee_lo: "?FindFirst@@YAPBUCharGraphNode@@PAXM@Z".into(),
        }
    }

    /// **The whole claim, as one compare**: this crate's encoders rebuild every
    /// one of the eighty bytes real `c2` emitted for a FRONTIER function.
    #[test]
    fn the_body_is_the_bytes_c2_emitted_for_negate_test() {
        let b = if_call_join_text(&negate_test_a(), 0, OptMode::O1).unwrap();
        assert_eq!(b.text, C2_TEXT);
        assert_eq!(b.prolog_len, 0x0c);
        assert_eq!(b.bl_offsets, [0x2c, 0x34]);
    }

    /// The `.pdata` and `$M` inputs, pinned separately from the bytes: the
    /// reference obj's `$M2581` is at `0xc` and `$M2582` at `0x50`, i.e.
    /// prologue length and function length, and its `.pdata` word is
    /// `0x40001403` — 0x14 words long, 3 words of prologue.
    #[test]
    fn the_frame_offsets_are_the_ones_the_pdata_record_needs() {
        let b = if_call_join_text(&negate_test_a(), 0, OptMode::O1).unwrap();
        assert_eq!((b.prolog_len / 4, b.text.len() as u32 / 4), (3, 0x14));
    }

    /// **`base_off` reaches the two `bl` words and NOTHING else.** Packed, the
    /// function does not start at 0 and every `bl` encodes its own offset; every
    /// branch beside them is self-relative and must be byte-identical.
    #[test]
    fn only_the_two_call_words_move_with_the_function() {
        let a = if_call_join_text(&negate_test_a(), 0, OptMode::O1).unwrap();
        let b = if_call_join_text(&negate_test_a(), 0x40, OptMode::O1).unwrap();
        assert_eq!(b.bl_offsets, [0x6c, 0x74]);
        for (i, (x, y)) in a.text.chunks(4).zip(b.text.chunks(4)).enumerate() {
            if i == 11 || i == 13 {
                assert_ne!(x, y, "word {i} is a `bl` and must carry its own offset");
            } else {
                assert_eq!(x, y, "word {i} is self-relative and must not move");
            }
        }
    }

    /// **The mutation the module header names**: collapsing the join's
    /// `mr r11,r3 ; mr r3,r11` into nothing — or into the single `mr r3,r3` it
    /// is equivalent to — is a different obj. Asserted as an inequality so a
    /// future peephole cannot land silently.
    #[test]
    fn the_join_round_trip_is_not_removable() {
        let b = if_call_join_text(&negate_test_a(), 0, OptMode::O1).unwrap();
        assert_eq!(&b.text[56..60], &encode_mr(SCRATCH_REG, RET_REG));
        assert_eq!(&b.text[60..64], &encode_mr(RET_REG, SCRATCH_REG));
        let mut collapsed = b.text.clone();
        collapsed.drain(56..64);
        assert_ne!(collapsed.len(), C2_TEXT.len());
    }

    /// **The two guards share ONE compare.** A second `cmpwi` between them is
    /// the emitter defect this class exists to avoid, and it is one word, so it
    /// would also move every displacement after it.
    #[test]
    fn one_compare_serves_both_entry_guards() {
        let b = if_call_join_text(&negate_test_a(), 0, OptMode::O1).unwrap();
        assert_eq!(&b.text[24..28], &encode_cmpwi(CR_COMPARE, PARK_REG, 1));
        assert_eq!(&b.text[28..32], &encode_bc(BO_TRUE, cr_bi(6, CR_BIT_LT), 32).unwrap());
        assert_eq!(&b.text[32..36], &encode_bc(BO_TRUE, cr_bi(6, CR_BIT_EQ), 28).unwrap());
        // …and the words the two `bt`s name are the same word.
        assert_eq!(32 + 32, 36 + 28);
    }

    /// The two literals are the only two immediate fields, and they land in the
    /// two `cmpwi` words and nowhere else.
    #[test]
    fn the_literals_are_two_immediate_fields() {
        let mut j = negate_test_a();
        j.k1 = 0x1234;
        j.k2 = -3;
        let b = if_call_join_text(&j, 0, OptMode::O1).unwrap();
        assert_eq!(&b.text[24..28], &encode_cmpwi(CR_COMPARE, PARK_REG, 0x1234));
        assert_eq!(&b.text[36..40], &encode_cmpwi(CR_COMPARE, PARK_REG, -3));
        // Everything else is the reference body.
        for (i, (x, y)) in b.text.chunks(4).zip(C2_TEXT.chunks(4)).enumerate() {
            if i != 6 && i != 9 {
                assert_eq!(x, y, "word {i} must not depend on the literals");
            }
        }
    }

    /// `/Ox` and `/O2` refuse. The join block tail-duplicates there and this
    /// port has not fitted the threshold.
    #[test]
    fn ox_refuses() {
        assert!(if_call_join_text(&negate_test_a(), 0, OptMode::Ox).is_err());
    }

    /// A literal outside `simm16` refuses rather than truncating.
    #[test]
    fn a_wide_literal_refuses() {
        let mut j = negate_test_a();
        j.k1 = 0x1_0000;
        assert!(if_call_join_text(&j, 0, OptMode::O1).is_err());
    }
}
