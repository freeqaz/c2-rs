//! **W-MMIO3 — the emitter for the guarded close chain**, i.e.
//! `src/xdk/nuispeech/mmio.cpp`'s `mmioClose` and the last 124 of that TU's 380
//! bytes.
//!
//! The reader's accept/refuse boundary and the source shape are on
//! [`c2_il::func::body::shapes::close_call_chain`]; this file is the **thirty-one
//! words** and nothing else. Everything variable in them is named in
//! [`c2_il::CloseCallChain`].
//!
//! # The thirty-one words, transcribed from `.text #14`
//!
//! ```text
//!    off  word       instruction             why it is this word
//!   ----  --------   ---------------------   -----------------------------------
//!   0x00  7d8802a6   mflr  r12               FrameLayout{locals:0, out_slots:4,
//!   0x04  9181fff8   stw   r12,-8(r1)        saved_gprs:1}: 96 bytes. The widest
//!   0x08  fbe1fff0   std   r31,-16(r1)       call takes four arguments, which is
//!   0x0c  9421ffa0   stwu  r1,-96(r1)        under the ABI floor of eight and so
//!                                            does not enter the size
//!   0x10  7c7f1b78   mr    r31,r3            THE r31 PARK: `p` is live across the
//!                                            `bctrl` and across an EXTERNAL `bl`,
//!                                            both of which have the whole volatile
//!                                            set as their footprint, so no volatile
//!                                            qualifies (M-RULE)
//!   0x14  7c852378   mr    r5,r4             THE r5 PARK: `u` crosses ONE call whose
//!                                            callee this TU defines and whose
//!                                            footprint is {r3}, and r5 is the
//!                                            register its next consumer — the
//!                                            `bctrl`'s third argument — wants.
//!                                            "Coalescing beats allocation"
//!   0x18  2b030000   cmplwi cr6,r3,0         the guard reads r3 ITSELF: the park
//!   0x1c  409a000c   bf    26,-> next        copied it, it did not move it
//!   0x20  38600005   li    r3,<K>            the arm, IN SOURCE ORDER
//!   0x24  48000044   b     -> epilogue
//!   0x28  38800000   li    r4,<L1>           call 1's second argument
//!   0x2c  7fe3fb78   mr    r3,r31            its first, back out of the park
//!   0x30  4bffffd1   bl    <g>       REL24   #1
//!   0x34  28030000   cmplwi cr0,r3,0         THE RESULT COMPARE, on cr0 — every
//!   0x38  40820030   bf    2,-> epilogue     guard in this seam reads cr6, and
//!                                            the branch sense is INVERTED: the arm
//!                                            returns a value already in r3, so it
//!                                            costs no instruction and folds into
//!                                            the branch to the epilogue
//!   0x3c  817f0008   lwz   r11,<off>(r31)    the function pointer
//!   0x40  38c00000   li    r6,<A3>           the indirect call's arguments, and
//!   0x44  38800004   li    r4,<A1>           r5 is ALREADY the parked `u`
//!   0x48  7fe3fb78   mr    r3,r31
//!   0x4c  7d6903a6   mtctr r11
//!   0x50  4e800421   bctrl                   the class's one new word
//!   0x54  28030000   cmplwi cr0,r3,0         the second result compare
//!   0x58  40820010   bf    2,-> epilogue
//!   0x5c  7fe3fb78   mr    r3,r31            the void call's one argument
//!   0x60  4bffffa1   bl    <k>       REL24   #2
//!   0x64  38600000   li    r3,0
//!   0x68  38210060   addi  r1,r1,96          the MATERIALISED COMMON EPILOGUE:
//!   0x6c  8181fff8   lwz   r12,-8(r1)        one block, reached from FOUR places
//!   0x70  7d8803a6   mtlr  r12
//!   0x74  ebe1fff0   ld    r31,-16(r1)
//!   0x78  4e800020   blr
//! ```
//!
//! # The word that is NOT there, and it is the point
//!
//! Between `0x58` and `0x5c` the source has `h(p, 0, 0, 0);` — a call, spelled
//! out in the `.ex` stream statement by statement — and the obj has **no
//! branch, no relocation and no symbol** for it. That is `w-ifn` #2351's D2
//! purity rule, and the emitter's part in it is to emit nothing: the fact that
//! licenses the omission is established at `c2_il::IlBundle::functions`, over
//! the callee's own segment, before this function is ever called.
//!
//! # Three things that are measurements and not choices
//!
//! * **The result compares are on `cr0` and the guard's is on `cr6`, in one
//!   body.** `28030000` against `2b030000` — the same instruction, four bits
//!   apart, and it is the difference between a compare against a formal and a
//!   compare against a call's return value.
//! * **The two parks are two different rules and neither is a default.**
//!   `WB_CHOOSER_FINDINGS` §2.3 M-RULE and its first sub-rule; `w-ifn`'s
//!   `probe/park.cpp` is the second instrument and moves in the predicted
//!   direction (external callee ⇒ r30 and a 112-byte frame). **Sub-rule #1882
//!   — `r11` when the value does not cross a call, `r10` when it does — is
//!   about which volatile a MOVE picks and does not apply to either park
//!   here**, which is stated because it is the rule a reader would reach for.
//! * **Every branch except the two `bl` is self-relative**, so only those two
//!   need `base_off`.

use crate::codegen::calls::encode_call_branch;
use crate::codegen::encode::{
    cr_bi, encode_addi, encode_bctrl, encode_cmplwi, encode_lwz,
    encode_mr, encode_mtctr, BO_FALSE, CR_BIT_EQ,
};
use crate::codegen::frame::FrameLayout;
use crate::codegen::select::{fits_i16, out_of_class};
use crate::codegen::OptMode;
use crate::BackendError;
use c2_il::CloseCallChain;
use crate::codegen::block_ir::{BlockOrder, BodyLayout, Terminator};

/// The condition-register field the GUARD reads — `2b030000` is
/// `cmplwi cr6,r3,0`, the same field [`super::guard_ret_chain`] measures.
const GUARD_CRF: u8 = 6;

/// The condition-register field the two RESULT compares read. **`cmplwi` with
/// no explicit field is `cr0`**, and that is not a spelling difference: the
/// guard's word is `2b030000` and these are `28030000`.
const RESULT_CRF: u8 = 0;

/// The callee-saved register the pointer formal is parked in. The only saved
/// GPR, which is what makes the frame `saved_gprs: 1`.
const PARK_REG: u8 = 31;

/// The VOLATILE the second formal is parked in — the register the indirect
/// call's third argument wants. See the module doc; this is the constant
/// M-RULE's "coalescing beats allocation" sub-rule produces here and it is not
/// "the next free volatile".
const VOL_PARK_REG: u8 = 5;

/// The scratch the function pointer is loaded into before `mtctr`.
const FNPTR_REG: u8 = 11;

/// `li rD,k` — `addi rD,0,k`. The same two-line helper every framed class here
/// carries.
fn encode_li(rd: u8, k: i16) -> [u8; 4] {
    encode_addi(rd, 0, k)
}

/// This class's emitted body: the bytes plus the offsets the writers need.
pub struct CloseCallChainBody {
    pub text: Vec<u8>,
    /// Absolute `.text` offsets of the two `bl` (already including `base_off`),
    /// **in offset order**, which is also the order
    /// [`c2_il::IlFunction::callees`] lists their names in.
    pub bl_offsets: [u32; 2],
    /// Prologue length in bytes: the `$M(n)` label's value and the `.pdata`
    /// record's `PrologLen`.
    pub prolog_len: u32,
}

/// The frame this class builds. `out_slots` is the widest call's argument
/// count — the indirect call's four — which is under the ABI floor of eight and
/// therefore does not enter the size; it is written as 4 rather than 8 so the
/// layout says what the body does.
pub fn frame_for() -> FrameLayout {
    FrameLayout {
        locals: 0,
        out_slots: 4,
        saved_gprs: 1,
        saved_fprs: 0,
    }
}

/// Emit the body.
///
/// `base_off` is the function's own offset within `.text` — zero under `/Gy`,
/// where each function is its own COMDAT. It reaches only the two `bl`.
pub fn close_call_chain_text(
    c: &CloseCallChain,
    base_off: u32,
    mode: OptMode,
) -> Result<CloseCallChainBody, BackendError> {
    // **`/O1` only.** The reader asks this first, before any body byte is read
    // (board #1638); this is the emitter's own copy, kept for the reason every
    // framed class here keeps its own: the two must not be able to disagree
    // silently, and `select_function` is what `function_gate` runs.
    if mode != OptMode::O1 {
        return Err(out_of_class(
            "a close chain with a materialised common epilogue at /Ox or /O2: \
             the shared block is reached from four places and tail-duplicates \
             on a threshold this port has not fitted (board row X-b)",
        ));
    }
    if c.params.len() != 2 {
        return Err(out_of_class(
            "close-call-chain with other than two formals: a different arity \
             moves every argument register and the class has one witness",
        ));
    }
    let (Some(k), Some(l1), Some(a1), Some(a3)) = (
        i16::try_from(c.guard_ret).ok(),
        i16::try_from(c.call1_arg1).ok(),
        i16::try_from(c.icall_arg1).ok(),
        i16::try_from(c.icall_arg3).ok(),
    ) else {
        return Err(out_of_class(
            "close-call-chain literal wider than simm16: every one of them is a `li`",
        ));
    };
    if !fits_i16(c.fnptr_off) || c.fnptr_off < 0 {
        return Err(out_of_class(
            "close-call-chain function-pointer member offset outside a `lwz` displacement",
        ));
    }

    let frame = frame_for();
    let prologue = frame.prologue()?;
    let prolog_len = prologue.len() as u32;

    // **The body is seven blocks in `BodyLayout`** — `CFG_SHAPE.md` §6.2 item A,
    // board **#3124**. The four branches this class emits are `LabelMap`'s (item
    // B, #290): the guard's literal `12` and the three `epi_branches` patches
    // into a flat `Vec` are gone, and with them the only places this file could
    // disagree with its own layout.
    let mut l = BodyLayout::new(BlockOrder::IlStatement);
    let b_entry = l.declare("entry");
    let b_guard_arm = l.declare("guard-arm");
    let b_call1 = l.declare("call-1");
    let b_indirect = l.declare("indirect-call");
    let b_void = l.declare("void-call");
    let b_tail = l.declare("tail");
    let b_epi = l.declare("epilogue");

    // ---- the two parks, then the guard on cr6, reading r3 itself -------------
    let mut run = prologue;
    run.extend_from_slice(&encode_mr(PARK_REG, 3));
    run.extend_from_slice(&encode_mr(VOL_PARK_REG, 4));
    run.extend_from_slice(&encode_cmplwi(GUARD_CRF, 3, 0));
    l.place(
        b_entry,
        run,
        Terminator::Bc { bo: BO_FALSE, bi: cr_bi(GUARD_CRF, CR_BIT_EQ), taken: b_call1 },
    )?;
    // The guard's arm: one word, and its whole body is the jump to the epilogue.
    l.place(b_guard_arm, encode_li(3, k).to_vec(), Terminator::B { target: b_epi })?;

    // ---- call 1, then the first early return ---------------------------------
    //
    // The `bl` is a zero placeholder: its word encodes its own `.text` offset
    // (§3.3, #191), which is the layout's answer. `BL0_IN_CALL1` is where it
    // sits in THIS BLOCK'S own run.
    const BL0_IN_CALL1: u32 = 8;
    let mut run = Vec::new();
    run.extend_from_slice(&encode_li(4, l1));
    run.extend_from_slice(&encode_mr(3, PARK_REG));
    debug_assert_eq!(run.len() as u32, BL0_IN_CALL1);
    run.extend_from_slice(&[0; 4]);
    // The compare is on cr0 and the branch sense is INVERTED against every
    // guard in this seam: `bf 2` is taken when the result is NON-zero, i.e. on
    // the arm, because the arm's value is already in r3 and its whole body is
    // the jump to the epilogue. The arm has no words of its own at all, so the
    // branch names the epilogue directly.
    run.extend_from_slice(&encode_cmplwi(RESULT_CRF, 3, 0));
    l.place(
        b_call1,
        run,
        Terminator::Bc { bo: BO_FALSE, bi: cr_bi(RESULT_CRF, CR_BIT_EQ), taken: b_epi },
    )?;

    // ---- the indirect call ---------------------------------------------------
    let mut run = Vec::new();
    run.extend_from_slice(&encode_lwz(FNPTR_REG, PARK_REG, c.fnptr_off as i16));
    run.extend_from_slice(&encode_li(6, a3));
    run.extend_from_slice(&encode_li(4, a1));
    run.extend_from_slice(&encode_mr(3, PARK_REG));
    run.extend_from_slice(&encode_mtctr(FNPTR_REG));
    run.extend_from_slice(&encode_bctrl());
    run.extend_from_slice(&encode_cmplwi(RESULT_CRF, 3, 0));
    l.place(
        b_indirect,
        run,
        Terminator::Bc { bo: BO_FALSE, bi: cr_bi(RESULT_CRF, CR_BIT_EQ), taken: b_epi },
    )?;

    // ---- the ELIDED call emits NOTHING ---------------------------------------
    //
    // `h(p, 0, 0, 0)` is a statement in the `.ex` stream and there is no word
    // for it here. `w-ifn` #2351's D2, and the fact that licenses it is
    // established over the callee's own segment at
    // `c2_il::IlBundle::functions`.

    // ---- the void call to an external ----------------------------------------
    const BL1_IN_VOID: u32 = 4;
    let mut run = Vec::new();
    run.extend_from_slice(&encode_mr(3, PARK_REG));
    debug_assert_eq!(run.len() as u32, BL1_IN_VOID);
    run.extend_from_slice(&[0; 4]);
    l.place(b_void, run, Terminator::FallThrough)?;

    l.place(b_tail, encode_li(3, 0).to_vec(), Terminator::FallThrough)?;

    // ---- the materialised common epilogue ------------------------------------
    l.place(b_epi, frame.epilogue_run()?, Terminator::Blr)?;

    let body = l.finish()?;
    // ---- the two positions this class publishes, both the layout's -----------
    let at0 = body.at(b_call1, BL0_IN_CALL1)?;
    let at1 = body.at(b_void, BL1_IN_VOID)?;
    let bl_offsets = [base_off + at0, base_off + at1];
    let mut t = body.text;
    for (at, off) in [(at0, bl_offsets[0]), (at1, bl_offsets[1])] {
        t[at as usize..at as usize + 4].copy_from_slice(&encode_call_branch(off));
    }

    debug_assert_eq!(t.len() % 4, 0, "a body is a whole number of words");
    Ok(CloseCallChainBody {
        text: t,
        bl_offsets,
        prolog_len,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **The whole of `mmio.cpp`'s `.text #14`, byte for byte.**
    ///
    /// Transcribed out of `work/w-ifn/ref/mmio.dump.txt` — the reference obj
    /// produced by real `c2.dll` under wibo at the workload's own flags — and
    /// asserted as one block rather than word by word, so a change that moves
    /// a word cannot be absorbed by moving an assertion. The relocation sites
    /// are asserted beside it because a body with the right bytes and the wrong
    /// REL24 offsets links to the wrong function.
    #[test]
    fn the_text_is_byte_identical_to_the_reference_obj() {
        let c = CloseCallChain {
            params: vec![1, 2],
            guard_ret: 5,
            call1: "mmioFlush".to_string(),
            call1_arg1: 0,
            fnptr_off: 8,
            icall_arg1: 4,
            icall_arg3: 0,
            elided: "mmioSetBuffer".to_string(),
            void_call: "?FreeHandle@@YAXPAX@Z".to_string(),
        };
        let body = close_call_chain_text(&c, 0, OptMode::O1).expect("in class");
        #[rustfmt::skip]
        let want: Vec<u8> = [
            0x7d8802a6u32, 0x9181fff8, 0xfbe1fff0, 0x9421ffa0,
            0x7c7f1b78, 0x7c852378,
            0x2b030000, 0x409a000c, 0x38600005, 0x48000044,
            0x38800000, 0x7fe3fb78, 0x4bffffd1,
            0x28030000, 0x40820030,
            0x817f0008, 0x38c00000, 0x38800004, 0x7fe3fb78, 0x7d6903a6, 0x4e800421,
            0x28030000, 0x40820010,
            0x7fe3fb78, 0x4bffffa1,
            0x38600000,
            0x38210060, 0x8181fff8, 0x7d8803a6, 0xebe1fff0, 0x4e800020,
        ]
        .iter()
        .flat_map(|w| w.to_be_bytes())
        .collect();
        assert_eq!(body.text.len(), 124, "thirty-one words");
        // The two `bl` words encode their own displacement from `base_off`, and
        // the reference's are relative to the WHOLE `.text` of a packed obj,
        // not to this COMDAT. Compare everything else exactly and the two `bl`
        // by their opcode field alone.
        for (i, (got, exp)) in body.text.chunks(4).zip(want.chunks(4)).enumerate() {
            let off = i * 4;
            if off == 0x30 || off == 0x60 {
                assert_eq!(got[0] & 0xFC, 0x48, "word at {off:#x} is a `bl`");
                continue;
            }
            assert_eq!(got, exp, "word at {off:#x}");
        }
        assert_eq!(body.bl_offsets, [0x30, 0x60], "the two REL24 sites");
        assert_eq!(body.prolog_len, 0x10, "$M(n) and the .pdata PrologLen");
    }

    /// **The mode gate is the emitter's own copy of the parser's** (board
    /// #1638). If this ever passes at `/Ox` the two have drifted.
    #[test]
    fn it_refuses_outside_o1() {
        let c = CloseCallChain {
            params: vec![1, 2],
            guard_ret: 5,
            call1: "g".to_string(),
            call1_arg1: 0,
            fnptr_off: 8,
            icall_arg1: 4,
            icall_arg3: 0,
            elided: "h".to_string(),
            void_call: "k".to_string(),
        };
        assert!(close_call_chain_text(&c, 0, OptMode::Ox).is_err());
    }

    /// `bctrl` is one word with no operands, and it is the only new instruction
    /// this class needed. Asserted against the reference obj's own word rather
    /// than against a manual.
    #[test]
    fn bctrl_is_the_reference_word() {
        assert_eq!(encode_bctrl(), [0x4e, 0x80, 0x04, 0x21]);
    }

    /// **The frame is 96 bytes with one saved GPR**, which is what makes the
    /// prologue four words and `$M(n)` land at `0x10`. A frame that rounded to
    /// 112 would move every word after the `stwu` and the `.pdata` record with
    /// them.
    #[test]
    fn the_frame_is_ninety_six() {
        assert_eq!(frame_for().size(), 96);
    }
}
