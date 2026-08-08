//! **W-OSFINFO — the emitter for a range-and-flag guarded table lookup whose
//! two failure statements are TAIL-MERGED with its success statement.**
//!
//! The reader's accept/refuse boundary and the source shape are on
//! [`c2_il::func::body::shapes::osf_handle_guard`]; this file is the
//! thirty-one body words and nothing else. Everything variable in them is named
//! in [`c2_il::OsfHandleGuardFn`]: four names and **ten** immediates, plus two
//! constants the reader pins rather than carries (§ "the two pinned values").
//!
//! ```text
//!    off  word       instruction               why it is this word
//!   ----  --------   -----------------------   ---------------------------------
//!   0x00  7d8802a6   mflr  r12                 FrameLayout::default() with
//!   0x04  9181fff8   stw   r12,-8(r1)          saved_gprs 0 and out_slots 0:
//!   0x08  9421ffa0   stwu  r1,-96(r1)          96 bytes, byte for byte what the
//!                                              shipped FrameLayout emits. Both
//!                                              callees take no arguments, so
//!                                              the parameter area is the
//!                                              8-slot minimum and nothing in
//!                                              this body is addressed
//!
//!   0x0c  2f030000   cmpwi cr6,r3,0            `fh >= 0`   — SIGNED, IMMEDIATE
//!   0x10  41980058   bt    24,-> Lerr          …branch if LT, i.e. if `< 0`
//!   0x14  3d600000   lis   r11,0        REFHI  ┐ <limit>. The low half is a
//!   0x18  816b0000   lwz   r11,0(r11)   REFLO  ┘ **`lwz` DISPLACEMENT**, not an
//!                                              `addi`: the value is loaded, not
//!                                              the address taken. `data_refs_of`
//!                                              had no form for this
//!   0x1c  7f035840   cmplw cr6,r3,r11          `fh < *<limit>` — UNSIGNED,
//!                                              REGISTER. A different compare
//!                                              form from 0x0c, on the same
//!                                              operand, four words apart
//!   0x20  40980048   bf    24,-> Lerr          …branch if NOT LT
//!
//!   0x24  7c6b2e70   srawi r11,r3,K_SHIFT      ┐ the two-level table walk:
//!   0x28  3d400000   lis   r10,0        REFHI  │ <table>[fh >> K_SHIFT] is a
//!   0x2c  5569103a   slwi  r9,r11,2            │ POINTER (scale 4, a `slwi`),
//!   0x30  394a0000   addi  r10,r10,0    REFLO  │ and (fh & K_MASK) * K_ELEM is
//!   0x34  546b06fe   clrlwi r11,r3,32-N        │ a byte offset into whatever it
//!   0x38  1d6b0048   mulli r11,r11,K_ELEM      │ points at. TWO multiplies, TWO
//!   0x3c  7d49502e   lwzx  r10,r9,r10          │ different instructions
//!   0x40  7d6a5a14   add   r11,r10,r11         ┘
//!                                              **The high half at 0x28 is in
//!                                              r10, NOT the scratch**, and it is
//!                                              open across two unrelated words.
//!                                              `data_refs_of`'s walk was keyed
//!                                              on `addis r11,0,0` and could not
//!                                              see this quad at all
//!
//!   0x44  894b0004   lbz   r10,OFF_FILE(r11)   a BYTE field
//!   0x48  554a07ff   clrlwi. r10,r10,32-M      `& K_BIT`, RECORD form: sets cr0
//!                                              with no compare instruction
//!   0x4c  4182001c   bt    2,-> Lerr           …reading cr0, where the other
//!                                              three guards read cr6
//!   0x50  814b0000   lwz   r10,0(r11)          OFF_HND, pinned to 0 — see below
//!   0x54  2f0affff   cmpwi cr6,r10,K_INVALID
//!   0x58  419a0010   bt    26,-> Lerr
//!
//!   0x5c  3940ffff   li    r10,K_INVALID       ┐ the success arm, which does NOT
//!   0x60  38600000   li    r3,K_OK             │ contain its own store
//!   0x64  48000020   b     Ljoin               ┘
//!  Lerr:
//!   0x68  4bxxxxxx   bl    <errno>      REL24  the error block, reached FOUR
//!   0x6c  39600009   li    r11,K_ERRNO         ways and SUNK to the end
//!   0x70  91630000   stw   r11,0(r3)
//!   0x74  4bxxxxxx   bl    <doserrno>   REL24
//!   0x78  7c6b1b78   mr    r11,r3
//!   0x7c  39400000   li    r10,K_DOSERRNO
//!   0x80  3860ffff   li    r3,K_FAIL
//!  Ljoin:
//!   0x84  914b0000   stw   r10,0(r11)          **THE TAIL MERGE**
//!   0x88  38210060   addi  r1,r1,96            the epilogue, also free
//!   …     …          lwz r12 / mtlr r12 / blr
//! ```
//!
//! ## The tail merge, which is the whole reason this is one class
//!
//! `0x84` is **one word serving two structurally unrelated statements**. On the
//! success path r11 is the table entry and r10 is `K_INVALID`, so it is
//! `entry->OFF_HND = K_INVALID`. On the error path r11 is `<doserrno>`'s return
//! and r10 is `K_DOSERRNO`, so it is `*<doserrno>() = K_DOSERRNO`. c2 allocated
//! the address to r11 and the value to r10 on **both** paths so the two stores
//! could be the same word, and then reached the shared word from the success arm
//! through the body's one intra-section `b`.
//!
//! An emitter that lowered the two statements separately writes 33 words instead
//! of 31, gets every displacement after `0x5c` wrong, and **links**. This is
//! `docs/GAPS.md` §6's shape, and it is why the reader pins `OFF_HND` (below)
//! rather than carrying it: the merge is only legal at zero.
//!
//! ## The two pinned values
//!
//! * **`OFF_HND` must be 0.** At any other offset the success path's store is
//!   `stw r10,OFF_HND(r11)` and the error path's is `stw r10,0(r11)`; they are
//!   two words and the merge above does not exist. Refused in the reader.
//! * **`K_SCALE` must be 4.** The body multiplies twice — by 4 with a `slwi` and
//!   by `K_ELEM` with a `mulli` — and *which* instruction c2 picks is a chooser
//!   over "is the constant a power of two". With one witness of each form there
//!   is nothing to fit, so the scale is pinned to the witnessed 4 and `K_ELEM`
//!   is refused if it IS a power of two. Board **#1706**: anything the emitter
//!   cannot vary must be refused by the READER.
//!
//! **Zero words are chosen by a scheduler or a register allocator**, which is
//! what PREREG **D1** registered.
//!
//! Every branch here is **self-relative** and therefore independent of where the
//! function lands in `.text`; only the two `bl`s encode their own offsets, so
//! they are the only words that need `base_off`.

use crate::codegen::calls::encode_call_branch;
use crate::codegen::encode::{
    cr_bi, encode_add, encode_addi, encode_addis, encode_b_intra, encode_bc, encode_clrlwi_record,
    encode_cmplw, encode_cmpwi, encode_lbz, encode_lwz, encode_lwzx, encode_mr, encode_mulli,
    encode_rlwinm, encode_srawi, encode_stw, BO_FALSE, BO_TRUE, CR_BIT_EQ, CR_BIT_LT, CR_COMPARE,
};
use crate::codegen::frame::FrameLayout;
use crate::codegen::select::{fits_i16, out_of_class, ARG_REGS, RET_REG, SCRATCH_REG};
use crate::codegen::OptMode;
use crate::BackendError;
use c2_il::OsfHandleGuardFn;

/// The volatile register holding the loaded VALUE throughout — the table entry's
/// flag byte, then its handle word, then the value the merged store writes.
/// Also the register the second REFHI/REFLO quad lands in, which is the fact
/// `data_refs_of`'s old scratch-keyed walk could not represent.
const VAL_REG: u8 = 10;
/// The volatile register holding the scaled table INDEX for the one `lwzx`.
const INDEX_REG: u8 = 9;
/// `cr0`, which the flag test reads because a record-form `rlwinm` writes there
/// and nowhere else. The other three guards read [`CR_COMPARE`] (cr6).
const CR_FLAG: u8 = 0;
/// The scale of the outer table's elements, in bytes. **Pinned, not carried** —
/// see the module doc's "the two pinned values".
pub const K_SCALE: i32 = 4;

/// `li rD,k` — `addi rD,0,k`. The same two-line helper
/// [`super::alloc_init_or_fail`] and [`super::guard_chain_shared_tail`] carry,
/// and for the same reason: `encode_addi` with `ra = 0` is `li`, and spelling
/// that at every call site hides it.
fn encode_li(rd: u8, k: i16) -> [u8; 4] {
    encode_addi(rd, 0, k)
}

/// `clrlwi rA,rS,32−n` for a mask of `2^n − 1`, i.e. "keep the low `n` bits".
///
/// Returns `None` when `mask + 1` is not a power of two — the mask is a field
/// and this is the only form the class has words for.
fn clrlwi_mb(mask: i32) -> Option<u8> {
    if mask <= 0 {
        return None;
    }
    let m = mask as u32;
    if !(m + 1).is_power_of_two() {
        return None;
    }
    let n = (m + 1).trailing_zeros();
    // `n == 32` cannot occur: `m + 1` would have overflowed `i32`'s positive
    // range long before. `n == 0` cannot either — `mask > 0`.
    Some(32u8.saturating_sub(n as u8))
}

/// This class's emitted body: the bytes plus the offsets the writers need.
pub struct OsfHandleGuardBody {
    pub text: Vec<u8>,
    /// Absolute `.text` offsets of the two `bl` words, in block order —
    /// `<errno>` first, then `<doserrno>`.
    pub bl_offsets: [u32; 2],
    /// Prologue length in bytes: the `$M(n)` label's value and the `.pdata`
    /// record's `PrologLen`.
    pub prolog_len: u32,
}

/// Emit the thirty-one body words.
///
/// `base_off` is the function's own offset within `.text` — zero under `/Gy`,
/// where each function is its own COMDAT. It reaches only the two `bl`s.
pub fn osf_handle_guard_text(
    g: &OsfHandleGuardFn,
    base_off: u32,
    mode: OptMode,
) -> Result<OsfHandleGuardBody, BackendError> {
    // **`/O1` only.** The reader asks this first, before any body byte is read
    // (board #1638); this is the emitter's own copy, kept for the reason its two
    // siblings keep theirs: the two must not be able to disagree silently, and
    // `select_function` is what `function_gate` runs.
    if mode != OptMode::O1 {
        return Err(out_of_class(
            "a range-and-flag guarded table lookup at /Ox or /O2: the error \
             block is reached from four places and its duplication threshold \
             has not been fitted",
        ));
    }
    // Range-checked here as well as in the reader, because this is where a
    // truncation would happen: each of these lands in one signed 16-bit field.
    for k in [
        g.k_elem,
        g.off_file,
        g.k_invalid,
        g.k_ok,
        g.k_errno,
        g.k_doserrno,
        g.k_fail,
    ] {
        if !fits_i16(k) {
            return Err(out_of_class(
                "a guarded table lookup whose literal or member offset is outside simm16",
            ));
        }
    }
    // The shift is a `srawi` field: 0..=31, and a shift of 0 is a different
    // program (the index and the offset would name the same bits).
    if !(1..=31).contains(&g.k_shift) {
        return Err(out_of_class(
            "a guarded table lookup whose index shift is not 1..=31",
        ));
    }
    let mb_mask = clrlwi_mb(g.k_mask).ok_or_else(|| {
        out_of_class(
            "a guarded table lookup whose element mask is not `2^n - 1`: the \
             class has a `clrlwi` and no other masking word",
        )
    })?;
    let mb_bit = clrlwi_mb(g.k_bit).ok_or_else(|| {
        out_of_class(
            "a guarded table lookup whose flag mask is not `2^n - 1`: the class \
             has a record-form `clrlwi` and no other masking word",
        )
    })?;
    // The two pinned values. Both are also refused in the reader; this is the
    // emitter's copy, for the same reason the mode gate has one.
    if g.off_hnd != 0 {
        return Err(out_of_class(
            "a guarded table lookup whose handle member is not at offset 0: the \
             success store and the error store share ONE word and that is only \
             legal at zero",
        ));
    }
    if g.k_elem <= 0 || (g.k_elem as u32).is_power_of_two() {
        return Err(out_of_class(
            "a guarded table lookup whose element size is a power of two: c2 \
             emits a `slwi` there and this class has a `mulli`, and with one \
             witness of each form the chooser has not been fitted",
        ));
    }

    // `out_slots` 0 — both callees take no arguments — and `saved_gprs` 0: every
    // value that outlives a `bl` is recomputed rather than parked. The shipped
    // `FrameLayout` then emits 96 bytes, which is the reference's `stwu`.
    let frame = FrameLayout::default();
    let prologue = frame.prologue()?;
    let epilogue = frame.epilogue()?;
    let prolog_len = prologue.len() as u32;

    let mut t = prologue;
    let fh = ARG_REGS[0];

    // ---- guard 1: `fh >= 0` — signed, immediate, branch on LT --------------
    t.extend_from_slice(&encode_cmpwi(CR_COMPARE, fh, 0));
    let low_site = t.len() as u32;
    t.extend_from_slice(&[0; 4]);

    // ---- guard 2: `fh < *<limit>` — unsigned, register ---------------------
    //
    // The high half is the scratch and the low half is the `lwz` that loads the
    // limit's VALUE. Nothing takes the limit's address, so there is no `addi`
    // here at all — the relocation rides the load's displacement field.
    t.extend_from_slice(&encode_addis(SCRATCH_REG, 0, 0));
    t.extend_from_slice(&encode_lwz(SCRATCH_REG, SCRATCH_REG, 0));
    t.extend_from_slice(&encode_cmplw(CR_COMPARE, fh, SCRATCH_REG));
    let high_site = t.len() as u32;
    t.extend_from_slice(&[0; 4]);

    // ---- the two-level table walk ------------------------------------------
    //
    // Written out in this order rather than as "address arithmetic with the high
    // half hoisted N", because the second quad's two words are three apart with
    // two unrelated instructions between them and the first quad's are adjacent:
    // there is no hoist rule here, only a transcription.
    t.extend_from_slice(&encode_srawi(SCRATCH_REG, fh, g.k_shift as u8));
    t.extend_from_slice(&encode_addis(VAL_REG, 0, 0));
    // `slwi rD,rS,2` is `rlwinm rD,rS,2,0,29`. `K_SCALE` is pinned to 4, so the
    // shift amount is 2 and the mask end is 31 − 2.
    let scale_sh = K_SCALE.trailing_zeros() as u8;
    t.extend_from_slice(&encode_rlwinm(INDEX_REG, SCRATCH_REG, scale_sh, 0, 31 - scale_sh));
    t.extend_from_slice(&encode_addi(VAL_REG, VAL_REG, 0));
    t.extend_from_slice(&encode_rlwinm(SCRATCH_REG, fh, 0, mb_mask, 31));
    t.extend_from_slice(&encode_mulli(SCRATCH_REG, SCRATCH_REG, g.k_elem as i16));
    t.extend_from_slice(&encode_lwzx(VAL_REG, INDEX_REG, VAL_REG));
    t.extend_from_slice(&encode_add(SCRATCH_REG, VAL_REG, SCRATCH_REG));

    // ---- guard 3: the flag byte, on cr0 through a record-form mask ---------
    t.extend_from_slice(&encode_lbz(VAL_REG, SCRATCH_REG, g.off_file as i16));
    t.extend_from_slice(&encode_clrlwi_record(VAL_REG, VAL_REG, mb_bit));
    let flag_site = t.len() as u32;
    t.extend_from_slice(&[0; 4]);

    // ---- guard 4: the handle word is not already invalid -------------------
    t.extend_from_slice(&encode_lwz(VAL_REG, SCRATCH_REG, g.off_hnd as i16));
    t.extend_from_slice(&encode_cmpwi(CR_COMPARE, VAL_REG, g.k_invalid as i16));
    let live_site = t.len() as u32;
    t.extend_from_slice(&[0; 4]);

    // ---- the success arm: set up the merged store's two registers ----------
    t.extend_from_slice(&encode_li(VAL_REG, g.k_invalid as i16));
    t.extend_from_slice(&encode_li(RET_REG, g.k_ok as i16));
    let join_site = t.len() as u32;
    t.extend_from_slice(&[0; 4]);

    // ---- the error block, SUNK here and reached FOUR ways ------------------
    let l_err = t.len() as u32;
    let bl0 = base_off + t.len() as u32;
    t.extend_from_slice(&encode_call_branch(bl0));
    t.extend_from_slice(&encode_li(SCRATCH_REG, g.k_errno as i16));
    t.extend_from_slice(&encode_stw(SCRATCH_REG, RET_REG, 0));
    let bl1 = base_off + t.len() as u32;
    t.extend_from_slice(&encode_call_branch(bl1));
    t.extend_from_slice(&encode_mr(SCRATCH_REG, RET_REG));
    t.extend_from_slice(&encode_li(VAL_REG, g.k_doserrno as i16));
    t.extend_from_slice(&encode_li(RET_REG, g.k_fail as i16));

    // ---- the merged store ---------------------------------------------------
    let l_join = t.len() as u32;
    t.extend_from_slice(&encode_stw(VAL_REG, SCRATCH_REG, 0));

    t.extend_from_slice(&epilogue);

    // ---- the five displacements --------------------------------------------
    //
    // Every one is computed from the block layout above rather than written as a
    // constant, so a change to any block's length moves them all together. A
    // hardcoded `+0x58` would keep linking and stop being right.
    let mut patch = |site: u32, word: Option<[u8; 4]>| -> Result<(), BackendError> {
        let w = word.ok_or_else(|| {
            out_of_class("a guarded-table-lookup branch outside its displacement field")
        })?;
        t[site as usize..site as usize + 4].copy_from_slice(&w);
        Ok(())
    };
    let bi_lt = cr_bi(CR_COMPARE, CR_BIT_LT);
    let bi_eq = cr_bi(CR_COMPARE, CR_BIT_EQ);
    let bi_flag = cr_bi(CR_FLAG, CR_BIT_EQ);
    patch(low_site, encode_bc(BO_TRUE, bi_lt, l_err as i32 - low_site as i32))?;
    patch(high_site, encode_bc(BO_FALSE, bi_lt, l_err as i32 - high_site as i32))?;
    patch(flag_site, encode_bc(BO_TRUE, bi_flag, l_err as i32 - flag_site as i32))?;
    patch(live_site, encode_bc(BO_TRUE, bi_eq, l_err as i32 - live_site as i32))?;
    patch(join_site, encode_b_intra(l_join as i32 - join_site as i32))?;

    Ok(OsfHandleGuardBody { text: t, bl_offsets: [bl0, bl1], prolog_len })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codegen::encode::{encode_cmpw, encode_clrlwi31};

    /// `_free_osfhnd`'s parse, as the reader produces it from the workload's own
    /// IL. Tokens are the capture's; the names are the obj's.
    fn osfinfo() -> OsfHandleGuardFn {
        OsfHandleGuardFn {
            params: vec![0x9f7],
            limit: "_nhandle".into(),
            table: "__pioinfo".into(),
            errno: "_errno".into(),
            doserrno: "__doserrno".into(),
            k_shift: 5,
            k_mask: 31,
            k_elem: 72,
            off_file: 4,
            k_bit: 1,
            off_hnd: 0,
            k_invalid: -1,
            k_ok: 0,
            k_errno: 9,
            k_doserrno: 0,
            k_fail: -1,
        }
    }

    /// **The thirty-eight words, against the real obj.**
    ///
    /// Transcribed from `work/w-osfinfo/ref/osfinfo/dis.txt`. The two `bl`s
    /// carry `-104` and `-116` because the function is its own COMDAT under
    /// `/Gy` and the placeholder displacement is `-(offset of the branch word)`.
    const REF_TEXT: [u32; 38] = [
        0x7d8802a6, 0x9181fff8, 0x9421ffa0,
        0x2f030000, 0x41980058,
        0x3d600000, 0x816b0000, 0x7f035840, 0x40980048,
        0x7c6b2e70, 0x3d400000, 0x5569103a, 0x394a0000,
        0x546b06fe, 0x1d6b0048, 0x7d49502e, 0x7d6a5a14,
        0x894b0004, 0x554a07ff, 0x4182001c,
        0x814b0000, 0x2f0affff, 0x419a0010,
        0x3940ffff, 0x38600000, 0x48000020,
        0x4bffff99, 0x39600009, 0x91630000,
        0x4bffff8d, 0x7c6b1b78, 0x39400000, 0x3860ffff,
        0x914b0000,
        0x38210060, 0x8181fff8, 0x7d8803a6, 0x4e800020,
    ];

    fn ref_bytes() -> Vec<u8> {
        REF_TEXT.iter().flat_map(|w| w.to_be_bytes()).collect()
    }

    #[test]
    fn the_body_is_the_reference_obj_word_for_word() {
        let b = osf_handle_guard_text(&osfinfo(), 0, OptMode::O1).expect("in class");
        let want = ref_bytes();
        assert_eq!(b.text.len(), want.len(), "152 bytes");
        for (i, (g, w)) in b.text.chunks_exact(4).zip(want.chunks_exact(4)).enumerate() {
            assert_eq!(g, w, "word {i} at .text+{:#04x}", i * 4);
        }
        assert_eq!(b.prolog_len, 0x0c, "the $M(n) label and the .pdata PrologLen");
        assert_eq!(b.bl_offsets, [0x68, 0x74]);
    }

    /// **The two entry guards use DIFFERENT compare forms on the SAME operand**,
    /// four words apart, and nothing in the source says so: `fh >= 0` is signed
    /// and immediate, `fh < *<limit>` is unsigned and register-register. A class
    /// that reached for one form throughout emits the right program with one
    /// wrong word and every branch still resolving.
    #[test]
    fn the_two_entry_guards_do_not_share_a_compare_form() {
        let b = osf_handle_guard_text(&osfinfo(), 0, OptMode::O1).expect("in class");
        assert_eq!(&b.text[0x0c..0x10], &encode_cmpwi(CR_COMPARE, 3, 0));
        assert_eq!(&b.text[0x1c..0x20], &encode_cmplw(CR_COMPARE, 3, SCRATCH_REG));
        // the signed register form is the rival, and it is one field away
        assert_ne!(&b.text[0x1c..0x20], &encode_cmpw(CR_COMPARE, 3, SCRATCH_REG));
    }

    /// **The flag test reads cr0 and the other three guards read cr6**, and no
    /// compare instruction is issued for it at all — the record-form `rlwinm`
    /// sets cr0 itself. A non-record mask leaves cr0 holding whatever the last
    /// instruction left there and the `bt` branches on a stale bit.
    #[test]
    fn the_flag_test_is_a_record_form_mask_reading_cr0() {
        let b = osf_handle_guard_text(&osfinfo(), 0, OptMode::O1).expect("in class");
        assert_eq!(&b.text[0x48..0x4c], &encode_clrlwi_record(VAL_REG, VAL_REG, 31));
        assert_ne!(&b.text[0x48..0x4c], &encode_clrlwi31(VAL_REG, VAL_REG));
        // …and the branch below it names cr0's EQ bit, not cr6's.
        assert_eq!(
            &b.text[0x4c..0x50],
            &encode_bc(BO_TRUE, cr_bi(CR_FLAG, CR_BIT_EQ), 0x68 - 0x4c).expect("in range")
        );
        assert_eq!(cr_bi(CR_FLAG, CR_BIT_EQ), 2);
        assert_eq!(cr_bi(CR_COMPARE, CR_BIT_EQ), 26);
    }

    /// **THE TAIL MERGE.** One `stw` at the end serves both the success
    /// statement and the error statement; the success arm reaches it through the
    /// body's one intra-section `b` and contributes no store of its own.
    ///
    /// The rival — lowering the two statements separately — is 33 words, gets
    /// every displacement after 0x5c wrong, and links.
    #[test]
    fn the_success_and_error_stores_are_one_word() {
        let b = osf_handle_guard_text(&osfinfo(), 0, OptMode::O1).expect("in class");
        // exactly ONE `stw rS,0(r11)` in the whole body, and it is the last
        // instruction before the epilogue
        let merged = encode_stw(VAL_REG, SCRATCH_REG, 0);
        let n = b.text.chunks_exact(4).filter(|w| *w == merged).count();
        assert_eq!(n, 1, "the merged store appears once");
        assert_eq!(&b.text[0x84..0x88], &merged);
        // the success arm's last word is the `b` to it, not a store
        assert_eq!(&b.text[0x64..0x68], &encode_b_intra(0x84 - 0x64).expect("in range"));
    }

    /// **All four guards branch to ONE block**, and the block is sunk below the
    /// success arm. Two of the four read the LT bit — one true, one false — and
    /// that polarity is the relation, not the operand order.
    #[test]
    fn four_guards_name_one_sunk_error_block() {
        let b = osf_handle_guard_text(&osfinfo(), 0, OptMode::O1).expect("in class");
        let sites = [(0x10u32, BO_TRUE, 24u8), (0x20, BO_FALSE, 24), (0x4c, BO_TRUE, 2), (0x58, BO_TRUE, 26)];
        for (site, bo, bi) in sites {
            let want = encode_bc(bo, bi, 0x68 - site as i32).expect("in range");
            assert_eq!(&b.text[site as usize..site as usize + 4], &want, "guard at {site:#04x}");
        }
    }

    /// Every immediate is a field, and moving one moves exactly the words that
    /// carry it. **Ten fields** — the guard against a class that hardcoded one.
    ///
    /// `off_hnd` and the scale are deliberately NOT in this list: they are
    /// pinned, and the two tests below assert that they are refused rather than
    /// varied.
    #[test]
    fn the_ten_immediates_are_fields() {
        let base = osf_handle_guard_text(&osfinfo(), 0, OptMode::O1).expect("in class");
        let mut g = osfinfo();
        g.k_shift = 6;
        g.k_mask = 63;
        g.k_elem = 40;
        g.off_file = 8;
        g.k_bit = 3;
        g.k_invalid = -2;
        g.k_ok = 1;
        g.k_errno = 22;
        g.k_doserrno = 5;
        g.k_fail = -3;
        let moved = osf_handle_guard_text(&g, 0, OptMode::O1).expect("in class");
        assert_eq!(moved.text.len(), base.text.len(), "the same block plan");
        let differing = base
            .text
            .chunks_exact(4)
            .zip(moved.text.chunks_exact(4))
            .filter(|(x, y)| x != y)
            .count();
        // 0x24 0x34 0x38 0x44 0x48 0x54 0x5c 0x60 0x6c 0x7c 0x80 — eleven words,
        // because `k_invalid` reaches both the `cmpwi` and the `li`.
        assert_eq!(differing, 11);
    }

    /// **`off_hnd != 0` is REFUSED, not emitted.** The merge is only legal at
    /// zero and the emitter has one word for both stores.
    #[test]
    fn a_handle_member_away_from_zero_refuses() {
        let mut g = osfinfo();
        g.off_hnd = 4;
        assert!(osf_handle_guard_text(&g, 0, OptMode::O1).is_err());
    }

    /// **A power-of-two element size is REFUSED**, because c2 emits a `slwi`
    /// there and this class has a `mulli`. The chooser is not fitted and is not
    /// guessed.
    #[test]
    fn a_power_of_two_element_size_refuses() {
        for k in [8, 16, 64] {
            let mut g = osfinfo();
            g.k_elem = k;
            assert!(osf_handle_guard_text(&g, 0, OptMode::O1).is_err(), "{k}");
        }
        // …and 72, the witness, is accepted.
        assert!(osf_handle_guard_text(&osfinfo(), 0, OptMode::O1).is_ok());
    }

    /// A mask that is not `2^n − 1` has no `clrlwi` and is refused rather than
    /// approximated.
    #[test]
    fn a_non_contiguous_mask_refuses() {
        for m in [5, 6, 0, -1] {
            let mut g = osfinfo();
            g.k_mask = m;
            assert!(osf_handle_guard_text(&g, 0, OptMode::O1).is_err(), "{m}");
        }
        assert_eq!(clrlwi_mb(31), Some(27));
        assert_eq!(clrlwi_mb(1), Some(31));
        assert_eq!(clrlwi_mb(3), Some(30));
        assert_eq!(clrlwi_mb(6), None);
    }

    /// `/Ox` (which is also `/O2`) refuses in the emitter as well as in the
    /// reader — one fact, two locators, deliberately, so they cannot disagree
    /// silently.
    #[test]
    fn a_mode_other_than_o1_refuses() {
        for m in [OptMode::Ox] {
            assert!(osf_handle_guard_text(&osfinfo(), 0, m).is_err(), "{m:?}");
        }
    }
}
