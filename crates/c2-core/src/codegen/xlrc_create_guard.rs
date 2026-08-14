//! **W-XLR — the emitter for a two-stage create/attach guard whose four failure
//! paths converge on one returned status, in the port's FIRST helper-framed
//! function.**
//!
//! The reader's accept/refuse boundary and the source shape are on
//! [`c2_il::func::body::shapes::xlrc_create_guard`]; this file is the
//! thirty-eight body words and nothing else. Everything variable in them is
//! named in [`c2_il::XlrcCreateGuardFn`]: two callee names and **five**
//! immediates.
//!
//! ```text
//!    off  word       instruction               why it is this word
//!   ----  --------   -----------------------   ---------------------------------
//!   0x00  7d8802a6   mflr  r12                 FrameLayout{locals:4, out_slots:3,
//!   0x04  4bfffffd   bl    __savegprlr_26      saved_gprs:6}. THREE prologue
//!   0x08  9421ff70   stwu  r1,-144(r1)         words, not five: the helper does
//!                                              the six `std`s AND the LR spill,
//!                                              so there is no `stw r12,-8(r1)`.
//!                                              F = align16(max(80, 80+4)
//!                                                          + 8*7) = 144
//!
//!   0x0c  39600004   li    r11,K_INIT          ┐ the stack object. Its address
//!   0x10  7c7e1b78   mr    r30,r3              │ is taken, so it is WRITTEN to
//!   0x14  91610050   stw   r11,80(r1)          │ the frame and re-read three
//!   0x18  38610050   addi  r3,r1,80            │ times below — a folded local
//!   0x1c  7c9d2378   mr    r29,r4              │ would emit `li` there and drop
//!   0x20  7cbc2b78   mr    r28,r5              │ this store. 80 = locals_base()
//!   0x24  7cdb3378   mr    r27,r6              │ The four `mr`s INTERLEAVE with
//!   0x28  3b400000   li    r26,0               ┘ the store and the address, and
//!                                              that order is transcribed
//!
//!   0x2c  4bffffd5   bl    <create>     REL24  the first ordinary call
//!   0x30  7c7f1b79   mr.   r31,r3              **RECORD FORM** — the result is
//!                                              copied to its callee-saved home
//!                                              AND tested in one word, so this
//!                                              guard issues no compare at all
//!   0x34  40820024   bf    cr0.EQ -> Lelse     …branch when it is NOT null
//!
//!   0x38  81610050   lwz   r11,80(r1)          the RELOAD: the callee may have
//!                                              written the object
//!   0x3c  3f408007   lis   r26,HI(K_LO)        **HOISTED ABOVE THE BRANCH** —
//!                                              K_LO and K_HI share a high half,
//!                                              so one `lis` serves both arms and
//!                                              each arm is a single `ori`
//!   0x40  2b0b0004   cmplwi cr6,r11,K_BOUND    the body's ONLY cr6 test
//!   0x44  4098000c   bf    cr6.LT -> Lhi
//!   0x48  635a000e   ori   r26,r26,LO(K_LO)
//!   0x4c  48000040   b     Ljoin               intra-section `b` #1
//!  Lhi:
//!   0x50  635a10dd   ori   r26,r26,LO(K_HI)
//!   0x54  48000038   b     Ljoin               intra-section `b` #2
//!
//!  Lelse:
//!   0x58  7fc4f378   mr    r4,r30              the attach call's three
//!   0x5c  80a10050   lwz   r5,80(r1)           arguments: the parked formal,
//!   0x60  7fe3fb78   mr    r3,r31              a SECOND reload, and the parked
//!                                              create result
//!   0x64  4bffff9d   bl    <attach>     REL24
//!   0x68  28030000   cmplwi cr0,r3,0           an ORDINARY compare, on cr0:
//!                                              the value dies here, so there is
//!                                              nothing to copy and no record
//!                                              form to use
//!   0x6c  40820010   bf    cr0.EQ -> Lok
//!   0x70  3f408000   lis   r26,HI(K_FAIL)      its own pair — this arm is not
//!   0x74  635a4005   ori   r26,r26,LO(K_FAIL)  reachable from the one above
//!   0x78  48000014   b     Ljoin               intra-section `b` #3
//!  Lok:
//!   0x7c  81610050   lwz   r11,80(r1)          the THIRD reload
//!   0x80  917d0000   stw   r11,0(r29)          ┐ three stores through POINTER
//!   0x84  93fc0000   stw   r31,0(r28)          │ VALUES held in the parked
//!   0x88  907b0000   stw   r3,0(r27)           ┘ formals — no relocation, no
//!                                              `addi`, displacement 0
//!  Ljoin:
//!   0x8c  7f43d378   mr    r3,r26              the single returned status
//!   0x90  38210090   addi  r1,r1,144           ┐ the Class C epilogue: TWO
//!   0x94  4bffff6c   b     __restgprlr_26 REL24┘ words, and **no `blr` at all**
//! ```
//!
//! ## The three things this class is the first to need
//!
//! 1. **A prologue with a call in it.** Every framed body the port emitted
//!    before this one opens `mflr`/`stw`/`stwu` and has exactly one REL24 per
//!    IL-named callee. This one has **four** relocations for **two** IL callees:
//!    the helper pair is minted from `saved_gprs` and is invisible to the IL.
//! 2. **An epilogue that is a relocation.** `b __restgprlr_26` with `LK = 0` is
//!    the function's last word; `FrameLayout::epilogue` and its `blr` are not
//!    reachable from here.
//! 3. **Two symbols placed AFTER the `$T` label.** `docs/CODEGEN_FRAMED_CALLS.md`
//!    §2.3a's group is `.text+aux · fn · $M(end) · <callees> · $M(prologue) ·
//!    .pdata+aux · $T · __restgprlr_N · __savegprlr_N`, and the pair is in
//!    reverse first-reference order like everything else. They travel on
//!    `coff::Function::helper_externals` for that reason — putting them in
//!    `calls` alone would emit them in the callee region and every symbol index
//!    after `$M(end)` would move.
//!
//! **Zero words are chosen by a scheduler or a register allocator.** The
//! callee-saved assignment (r31 ← the create result, r30–r27 ← the four formals,
//! r26 ← the status) is *transcribed*: descending-from-r31 is the shipped rule
//! and it does **not** predict this order, because r31 goes to the value defined
//! last. Board **#1706** — anything the emitter cannot vary is refused by the
//! reader, and the reader pins the formal count at four.
//!
//! Every branch here is **self-relative** and therefore independent of where the
//! function lands in `.text`; only the four REL24 words encode their own
//! offsets, so they are the only ones that need `base_off`.

use crate::codegen::calls::encode_call_branch;
use crate::codegen::encode::{
    cr_bi, encode_addi, encode_addis, encode_cmplwi, encode_lwz,
    encode_mr, encode_mr_record, encode_ori, encode_stw, BO_FALSE, CR_BIT_EQ, CR_BIT_LT,
    CR_COMPARE,
};
use crate::codegen::frame::FrameLayout;
use crate::codegen::select::{out_of_class, ARG_REGS, RET_REG, SCRATCH_REG};
use crate::codegen::OptMode;
use crate::BackendError;
use c2_il::XlrcCreateGuardFn;
use crate::codegen::labels::Form;
use crate::codegen::reach;

/// The callee-saved register each formal is parked in, in declaration order:
/// `r3→r30`, `r4→r29`, `r5→r28`, `r6→r27`. Transcribed, not derived — see the
/// module doc.
const PARKED: [u8; 4] = [30, 29, 28, 27];

/// The callee-saved register holding the status accumulator.
const R_STATUS: u8 = 26;

/// The callee-saved register holding the create call's result.
const R_CLIENT: u8 = 31;

/// How many callee-saved GPRs this class parks: the four formals, the status and
/// the create result. Six is `__savegprlr_26`, and the whole reason this class
/// needs a frame emitter of its own.
const XLRC_SAVED_GPRS: u8 = 6;

/// The widest call this body makes takes three arguments. Below the ABI's 8-slot
/// floor, so it does not move the frame — carried anyway because
/// [`FrameLayout::size`] takes the maximum and a class that lied here would be
/// wrong the moment the floor changed.
const XLRC_OUT_SLOTS: u8 = 3;

/// The addressed local's size in bytes: one four-byte scalar, established
/// **positively** from `.sy`'s address-taken list by the reader.
const XLRC_LOCALS: u32 = 4;

/// The frame this class always builds.
pub fn xlrc_frame() -> FrameLayout {
    FrameLayout {
        locals: XLRC_LOCALS,
        out_slots: XLRC_OUT_SLOTS,
        saved_gprs: XLRC_SAVED_GPRS,
        saved_fprs: 0,
    }
}

/// The emitted body plus everything the obj writer needs from it.
pub struct XlrcCreateGuardBody {
    pub text: Vec<u8>,
    /// The four REL24 sites, in ascending `.text` order:
    /// `__savegprlr_N`, `<create>`, `<attach>`, `__restgprlr_N`.
    pub bl_offsets: [u32; 4],
    /// Prologue length in bytes — `$M(n)` and the `.pdata` `PrologLen`. **12**,
    /// i.e. three words, whatever `saved_gprs` is.
    pub prolog_len: u32,
}

/// Emit the thirty-eight words.
pub fn xlrc_create_guard_text(
    g: &XlrcCreateGuardFn,
    base_off: u32,
    mode: OptMode,
) -> Result<XlrcCreateGuardBody, BackendError> {
    // The mode is already gated in the parser (board #1638) — this is the second
    // lock, and it is the one that would fire if a future dispatch reached this
    // emitter from somewhere else.
    if mode != OptMode::O1 {
        return Err(out_of_class("xlrc-create-guard is /O1 only"));
    }
    if g.params.len() != PARKED.len() {
        return Err(out_of_class("xlrc-create-guard needs exactly four formals"));
    }
    let frame = xlrc_frame();
    let f = frame.size();
    let local = frame.locals_base() as i16;
    let f_imm = i16::try_from(f).map_err(|_| out_of_class("xlrc frame too large"))?;

    let lo16 = |k: i32| (k as u32 & 0xFFFF) as u16;
    let hi16 = |k: i32| ((k as u32 >> 16) as u16) as i16;

    let mut t: Vec<u8> = Vec::with_capacity(38 * 4);
    let w = |b: [u8; 4], t: &mut Vec<u8>| t.extend_from_slice(&b);

    // ---- the Class C prologue: three words, one of them a relocation --------
    let bl_save = base_off + 4;
    t.extend_from_slice(&frame.prologue_gpr_helper(base_off)?);
    let prolog_len = t.len() as u32;

    // ---- the entry block ----------------------------------------------------
    let k_init = i16::try_from(g.k_init)
        .map_err(|_| out_of_class("xlrc initial value wider than an li immediate"))?;
    w(encode_addi(SCRATCH_REG, 0, k_init), &mut t);
    w(encode_mr(PARKED[0], ARG_REGS[0]), &mut t);
    w(encode_stw(SCRATCH_REG, 1, local), &mut t);
    w(encode_addi(RET_REG, 1, local), &mut t);
    w(encode_mr(PARKED[1], ARG_REGS[1]), &mut t);
    w(encode_mr(PARKED[2], ARG_REGS[2]), &mut t);
    w(encode_mr(PARKED[3], ARG_REGS[3]), &mut t);
    w(encode_addi(R_STATUS, 0, 0), &mut t);
    let bl_create = base_off + t.len() as u32;
    w(encode_call_branch(bl_create), &mut t);

    // ---- `if (c == 0)`, tested by the record-form move ----------------------
    w(encode_mr_record(R_CLIENT, RET_REG), &mut t);
    let at_outer = t.len();
    w([0, 0, 0, 0], &mut t); // patched once `Lelse` is known

    // ---- the null arm: one hoisted `lis`, a cr6 test, two `ori`s -----------
    w(encode_lwz(SCRATCH_REG, 1, local), &mut t);
    w(encode_addis(R_STATUS, 0, hi16(g.k_lo)), &mut t);
    let k_bound = u16::try_from(g.k_bound)
        .map_err(|_| out_of_class("xlrc bound wider than a cmplwi immediate"))?;
    w(encode_cmplwi(CR_COMPARE, SCRATCH_REG, k_bound), &mut t);
    let at_inner = t.len();
    w([0, 0, 0, 0], &mut t); // patched once `Lhi` is known
    w(encode_ori(R_STATUS, R_STATUS, lo16(g.k_lo)), &mut t);
    let at_b1 = t.len();
    w([0, 0, 0, 0], &mut t); // patched once `Ljoin` is known
    let l_hi = t.len();
    w(encode_ori(R_STATUS, R_STATUS, lo16(g.k_hi)), &mut t);
    let at_b2 = t.len();
    w([0, 0, 0, 0], &mut t);

    // ---- the attach arm ----------------------------------------------------
    let l_else = t.len();
    w(encode_mr(ARG_REGS[1], PARKED[0]), &mut t);
    w(encode_lwz(ARG_REGS[2], 1, local), &mut t);
    w(encode_mr(ARG_REGS[0], R_CLIENT), &mut t);
    let bl_attach = base_off + t.len() as u32;
    w(encode_call_branch(bl_attach), &mut t);
    w(encode_cmplwi(0, RET_REG, 0), &mut t);
    let at_fail = t.len();
    w([0, 0, 0, 0], &mut t); // patched once `Lok` is known
    w(encode_addis(R_STATUS, 0, hi16(g.k_fail)), &mut t);
    w(encode_ori(R_STATUS, R_STATUS, lo16(g.k_fail)), &mut t);
    let at_b3 = t.len();
    w([0, 0, 0, 0], &mut t);

    // ---- the success arm: three stores through the parked pointers ---------
    let l_ok = t.len();
    w(encode_lwz(SCRATCH_REG, 1, local), &mut t);
    w(encode_stw(SCRATCH_REG, PARKED[1], 0), &mut t);
    w(encode_stw(R_CLIENT, PARKED[2], 0), &mut t);
    w(encode_stw(RET_REG, PARKED[3], 0), &mut t);

    // ---- the join, and the Class C epilogue --------------------------------
    let l_join = t.len();
    w(encode_mr(RET_REG, R_STATUS), &mut t);
    let bl_rest = base_off + t.len() as u32 + 4;
    t.extend_from_slice(&frame.epilogue_gpr_helper(base_off + t.len() as u32)?);
    debug_assert_eq!(f_imm as u32, f);

    // ---- the four forward branches, patched -------------------------------
    //
    // Each is `bf <bit>` — branch when the tested condition is FALSE — and the
    // three unconditional ones are plain intra-section `b`s. Every displacement
    // is `target − site`, self-relative, so none of them depends on `base_off`.
    let patch = |t: &mut Vec<u8>, at: usize, form: Form, disp: i32| -> Result<(), BackendError> {
        let word = reach::direct(form, disp, "xlrc branch")?;
        t[at..at + 4].copy_from_slice(&word);
        Ok(())
    };
    let bf = |bi: u8| Form::Bc { bo: BO_FALSE, bi };
    patch(&mut t, at_outer, bf(cr_bi(0, CR_BIT_EQ)), (l_else - at_outer) as i32)?;
    patch(
        &mut t,
        at_inner,
        bf(cr_bi(CR_COMPARE, CR_BIT_LT)),
        (l_hi - at_inner) as i32,
    )?;
    patch(&mut t, at_b1, Form::B, (l_join - at_b1) as i32)?;
    patch(&mut t, at_b2, Form::B, (l_join - at_b2) as i32)?;
    patch(&mut t, at_fail, bf(cr_bi(0, CR_BIT_EQ)), (l_ok - at_fail) as i32)?;
    patch(&mut t, at_b3, Form::B, (l_join - at_b3) as i32)?;

    Ok(XlrcCreateGuardBody {
        text: t,
        bl_offsets: [bl_save, bl_create, bl_attach, bl_rest],
        prolog_len,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `CXLrcImpl_CreateClientWithTransport` exactly as `xlrcimpl.cpp`'s IL
    /// decodes it.
    fn xlrcimpl() -> XlrcCreateGuardFn {
        XlrcCreateGuardFn {
            params: vec![0x09fc, 0x09fd, 0x09fe, 0x09ff],
            create: "?CreateClient@CXLrcImpl@@YAPAVCXLrcClient@@PAI@Z".to_string(),
            attach: "CXLrcClient_CreateTransport".to_string(),
            k_init: 4,
            k_bound: 4,
            k_lo: 0x8007_000Eu32 as i32,
            k_hi: 0x8007_10DDu32 as i32,
            k_fail: 0x8000_4005u32 as i32,
        }
    }

    /// **The whole body, against the reference obj** —
    /// `work/w-xlr/ref/xlrcimpl/dis.txt`, captured at the workload's own flags
    /// before this file existed. Every word, in order, including the four
    /// relocation placeholders.
    #[test]
    fn body_matches_the_reference_words() {
        let b = xlrc_create_guard_text(&xlrcimpl(), 0, OptMode::O1).unwrap();
        let want: [u32; 38] = [
            0x7d8802a6, 0x4bfffffd, 0x9421ff70, 0x39600004, 0x7c7e1b78, 0x91610050, 0x38610050,
            0x7c9d2378, 0x7cbc2b78, 0x7cdb3378, 0x3b400000, 0x4bffffd5, 0x7c7f1b79, 0x40820024,
            0x81610050, 0x3f408007, 0x2b0b0004, 0x4098000c, 0x635a000e, 0x48000040, 0x635a10dd,
            0x48000038, 0x7fc4f378, 0x80a10050, 0x7fe3fb78, 0x4bffff9d, 0x28030000, 0x40820010,
            0x3f408000, 0x635a4005, 0x48000014, 0x81610050, 0x917d0000, 0x93fc0000, 0x907b0000,
            0x7f43d378, 0x38210090, 0x4bffff6c,
        ];
        let got: Vec<u32> =
            b.text.chunks_exact(4).map(|c| u32::from_be_bytes(c.try_into().unwrap())).collect();
        assert_eq!(got, want, "the 38 words");
        assert_eq!(b.text.len(), 152);
    }

    /// The three offsets the obj writer reads out of the body, each pinned to a
    /// field of the reference obj rather than to this emitter's arithmetic.
    #[test]
    fn relocation_sites_and_prologue_length_match_the_reference() {
        let b = xlrc_create_guard_text(&xlrcimpl(), 0, OptMode::O1).unwrap();
        // `$M2589` has Value 0x0c and `.pdata` reads `40002603` — PrologLen 3
        // words, FunctionLen 0x26 = 38 words.
        assert_eq!(b.prolog_len, 0x0c);
        assert_eq!(b.text.len() as u32 / 4, 0x26);
        // REL24 at 0x04 (helper), 0x2c (create), 0x64 (attach), 0x94 (helper).
        assert_eq!(b.bl_offsets, [0x04, 0x2c, 0x64, 0x94]);
    }

    /// **The frame arithmetic, independently of the body.** 6 saved GPRs is the
    /// helper class, `N = 26`, and the size comes out of the published rule with
    /// nothing special-cased: `align16(max(80, 80+4) + 8*(1+6)) = 144`.
    #[test]
    fn the_frame_is_the_published_rule_at_six_saved_gprs() {
        let fr = xlrc_frame();
        assert!(fr.needs_gpr_helper());
        assert_eq!(fr.gpr_helper_n(), Some(26));
        assert_eq!(fr.save_gpr_helper_name().as_deref(), Some("__savegprlr_26"));
        assert_eq!(fr.rest_gpr_helper_name().as_deref(), Some("__restgprlr_26"));
        assert_eq!(fr.size(), 144);
        assert_eq!(fr.locals_base(), 80);
        assert_eq!(fr.out_of_class_ctx_gpr_helper(), None);
        // …and the ordinary gate still refuses it, which is what keeps every
        // other emitter out of this shape.
        assert_eq!(fr.out_of_class_ctx(), Some("frame-savegprlr-helper"));
    }

    /// The Class C prologue and epilogue in isolation, against the same obj.
    #[test]
    fn the_helper_prologue_and_epilogue_are_three_words_and_two() {
        let fr = xlrc_frame();
        assert_eq!(
            fr.prologue_gpr_helper(0).unwrap(),
            vec![0x7d, 0x88, 0x02, 0xa6, 0x4b, 0xff, 0xff, 0xfd, 0x94, 0x21, 0xff, 0x70]
        );
        // At `.text` offset 0x90 the epilogue's branch is at 0x94 and encodes
        // −0x94 with LK **clear** — an unlinked tail branch, and no `blr`.
        assert_eq!(
            fr.epilogue_gpr_helper(0x90).unwrap(),
            vec![0x38, 0x21, 0x00, 0x90, 0x4b, 0xff, 0xff, 0x6c]
        );
        // A layout that does NOT need the helper is refused by the same
        // predicate, in the other direction: Class C's three-word prologue is
        // wrong for a body whose saves are open-coded.
        let inline = FrameLayout { saved_gprs: 2, ..Default::default() };
        assert_eq!(
            inline.out_of_class_ctx_gpr_helper(),
            Some("frame-gpr-helper-class-without-a-helper")
        );
        assert!(inline.prologue_gpr_helper(0).is_err());
        assert!(inline.epilogue_gpr_helper(0).is_err());
    }

    /// `/Ox` is refused here as well as in the parser — board #1638's second
    /// lock. The class was measured at `/O1` and at nothing else.
    #[test]
    fn the_emitter_refuses_ox() {
        assert!(xlrc_create_guard_text(&xlrcimpl(), 0, OptMode::Ox).is_err());
    }

    /// The three unconditional `b`s all reach the SAME join word, and the two
    /// conditional ones reach the two arms. Asserted as decoded displacements
    /// rather than as bytes, because a transposition of two equal-looking
    /// branches is exactly the defect a byte compare of the whole body can hide
    /// when the immediates happen to coincide.
    #[test]
    fn every_branch_lands_where_the_block_plan_says() {
        let b = xlrc_create_guard_text(&xlrcimpl(), 0, OptMode::O1).unwrap();
        let word = |off: usize| u32::from_be_bytes(b.text[off..off + 4].try_into().unwrap());
        let bc_disp = |w: u32| ((w & 0xFFFC) as i16) as i32;
        let b_disp = |w: u32| {
            let d = w & 0x03FF_FFFC;
            if d & 0x0200_0000 != 0 { d as i32 - 0x0400_0000 } else { d as i32 }
        };
        assert_eq!(0x34 + bc_disp(word(0x34)), 0x58, "outer -> Lelse");
        assert_eq!(0x44 + bc_disp(word(0x44)), 0x50, "inner -> Lhi");
        assert_eq!(0x6c + bc_disp(word(0x6c)), 0x7c, "fail -> Lok");
        for site in [0x4c_i32, 0x54, 0x78] {
            assert_eq!(
                site + b_disp(word(site as usize)),
                0x8c,
                "intra-section b at {site:#x}"
            );
        }
    }

    /// A body offset by a non-zero `base_off` moves **only** the four REL24
    /// words: every conditional and intra-section branch is self-relative.
    #[test]
    fn only_the_relocated_words_depend_on_base_off() {
        let a = xlrc_create_guard_text(&xlrcimpl(), 0, OptMode::O1).unwrap();
        let c = xlrc_create_guard_text(&xlrcimpl(), 0x200, OptMode::O1).unwrap();
        let differing: Vec<usize> = a
            .text
            .chunks_exact(4)
            .zip(c.text.chunks_exact(4))
            .enumerate()
            .filter(|(_, (x, y))| x != y)
            .map(|(i, _)| i * 4)
            .collect();
        assert_eq!(differing, vec![0x04, 0x2c, 0x64, 0x94]);
        assert_eq!(c.bl_offsets, [0x204, 0x22c, 0x264, 0x294]);
    }

    /// The five immediates really are fields: varying each one moves exactly the
    /// word it belongs in and nothing else.
    #[test]
    fn the_five_immediates_are_fields() {
        let base = xlrc_create_guard_text(&xlrcimpl(), 0, OptMode::O1).unwrap().text;
        let moved = |g: XlrcCreateGuardFn| -> Vec<usize> {
            let t = xlrc_create_guard_text(&g, 0, OptMode::O1).unwrap().text;
            base.chunks_exact(4)
                .zip(t.chunks_exact(4))
                .enumerate()
                .filter(|(_, (x, y))| x != y)
                .map(|(i, _)| i * 4)
                .collect()
        };
        assert_eq!(moved(XlrcCreateGuardFn { k_init: 9, ..xlrcimpl() }), vec![0x0c]);
        assert_eq!(moved(XlrcCreateGuardFn { k_bound: 9, ..xlrcimpl() }), vec![0x40]);
        // `k_lo` owns the shared `lis` and its own `ori`; changing only its low
        // half leaves the `lis` alone, which is what makes the hoist legal.
        assert_eq!(
            moved(XlrcCreateGuardFn { k_lo: 0x8007_0001u32 as i32, ..xlrcimpl() }),
            vec![0x48]
        );
        assert_eq!(
            moved(XlrcCreateGuardFn { k_hi: 0x8007_0002u32 as i32, ..xlrcimpl() }),
            vec![0x50]
        );
        assert_eq!(
            moved(XlrcCreateGuardFn { k_fail: 0x8001_0003u32 as i32, ..xlrcimpl() }),
            vec![0x70, 0x74]
        );
    }

    /// A shape with the wrong formal count is refused rather than emitted short.
    #[test]
    fn a_wrong_formal_count_is_refused() {
        let g = XlrcCreateGuardFn { params: vec![1, 2, 3], ..xlrcimpl() };
        assert!(xlrc_create_guard_text(&g, 0, OptMode::O1).is_err());
    }
}
