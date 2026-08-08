//! **W-JSON — the UTF-16 → UTF-8 copy loop, in the port's FIRST framed body
//! with a back edge and its first FRAMELESS `__savegprlr_N` frame.**
//!
//! The reader's accept/refuse boundary and the source shape are on
//! [`c2_il::func::body::shapes::json_utf8_copy`]; this file is the
//! **seventy-six** body words and nothing else. Everything variable in them is
//! named in [`c2_il::JsonUtf8CopyFn`]: **four** values, two member offsets and
//! two wide status constants. Nothing else moves.
//!
//! ## The frame: a helper-saving LEAF
//!
//! ```text
//!   0x000  7d8802a6  mflr r12          TWO prologue words, not three: the body
//!   0x004  4bfffffd  bl __savegprlr_28 makes NO call, needs no outgoing
//!                                      parameter area and has no addressed
//!                                      local, so c2 allocates **no frame at
//!                                      all** — there is no `stwu`, and the
//!                                      helper's stores land below r1.
//!   …
//!   0x12c  4bfffed4  b __restgprlr_28  ONE epilogue word: no `addi r1,r1,F`
//!                                      either, and no `blr`.
//! ```
//!
//! `$M(prologue)` is therefore **8**, and the `.pdata` unwind word is
//! `0x40004C02` — `PrologLen 2`, `FuncLen 76` — which `coff::pdata_record`
//! produces from those two lengths with nothing special-cased.
//!
//! ## The seventy-six words
//!
//! ```text
//!    off  word       instruction              why it is this word
//!   ----  --------   ----------------------   --------------------------------
//!   0x008 38e00000   li    r7,0               THE ZERO. One register holds the
//!                                             literal 0 for the whole body and
//!                                             every later `= 0` is an `mr` off
//!                                             it — including the two NUL stores
//!   0x00c 2b050000   cmplwi cr6,r5,0          `!pSize`
//!   0x010 419a0110   bt    cr6.EQ -> Lfail
//!   0x014 2b040000   cmplwi cr6,r4,0          `!pBuffer`
//!   0x018 409a0010   bf    cr6.EQ -> Lelse
//!   0x01c 81650000   lwz   r11,0(r5)          `*pSize != 0`
//!   0x020 2b0b0000   cmplwi cr6,r11,0
//!   0x024 409a00fc   bf    cr6.EQ -> Lfail
//!  Lelse:
//!   0x028 81630004   lwz   r11,OFF_SIZE(r3)   `mBufferSize`
//!   0x02c 7ce83b78   mr    r8,r7              outputSize = 0
//!   0x030 7cff3b78   mr    r31,r7             index      = 0
//!   0x034 2b0b0000   cmplwi cr6,r11,0
//!   0x038 409900c8   bf    cr6.GT -> Lafter   `mBufferSize > 0`
//!   0x03c 7ce63b78   mr    r6,r7              offset = 0
//!   0x040 3bc00001   li    r30,1              **HOISTED OUT OF THE LOOP** — the
//!                                             `1` the two `rlwimi`s rotate to
//!                                             0x80, live across the back edge
//!                                             and therefore callee-saved
//!  Lloop:
//!   0x044 81630000   lwz   r11,OFF_BUF(r3)    `mBuffer`, reloaded every trip
//!   0x048 3bff0001   addi  r31,r31,1          index++
//!   0x04c 7d6b322e   lhzx  r11,r11,r6         `*(unsigned short *)((char *)mBuffer + offset)`
//!   0x050 38c60002   addi  r6,r6,2            offset += 2
//!   0x054 7d6a5b78   mr    r10,r11            the SECOND copy of wc, live only
//!                                             into the >0x7F arm's compare
//!   0x058 2b0b007f   cmplwi cr6,r11,127
//!   0x05c 41990020   bt    cr6.GT -> Lwide
//!   0x060 81450000   lwz   r10,0(r5)          `*pSize`
//!   0x064 39080001   addi  r8,r8,1            outputSize++
//!   0x068 7f085040   cmplw cr6,r8,r10
//!   0x06c 40980088   bf    cr6.LT -> Lcont
//!   0x070 556b067e   clrlwi r11,r11,25        `wc & 0x7F`
//!   0x074 b1640000   sth   r11,0(r4)
//!   0x078 48000034   b     Lnul               the ONE-byte arm joins the
//!                                             two-byte arm's trailing NUL
//!  Lwide:
//!   0x07c 81250000   lwz   r9,0(r5)           `maxSize = *pSize`
//!   0x080 2b0a07ff   cmplwi cr6,r10,2047
//!   0x084 41990030   bt    cr6.GT -> Lthree
//!   0x088 39080002   addi  r8,r8,2
//!   0x08c 7f084840   cmplw cr6,r8,r9
//!   0x090 40980064   bf    cr6.LT -> Lcont
//!   0x094 394000c0   li    r10,192
//!   0x098 7d695b78   mr    r9,r11
//!   0x09c 516ad6fe   rlwimi r10,r11,26,27,31  `0xC0 | ((wc >> 6) & 0x1F)`
//!   0x0a0 53c93832   rlwimi r9,r30,7,0,25     `0x80 | (wc & 0x3F)` — the 0x80
//!                                             comes from the HOISTED r30
//!   0x0a4 b1440000   sth   r10,0(r4)
//!   0x0a8 b5240002   sthu  r9,2(r4)
//!  Lnul:
//!   0x0ac b4e40002   sthu  r7,2(r4)           the shared trailing NUL
//!   0x0b0 48000044   b     Lcont
//!  Lthree:
//!   0x0b4 39080003   addi  r8,r8,3
//!   0x0b8 7f084840   cmplw cr6,r8,r9
//!   0x0bc 40980038   bf    cr6.LT -> Lcont
//!   0x0c0 392000e0   li    r9,224
//!   0x0c4 3ba00080   li    r29,128
//!   0x0c8 39440002   addi  r10,r4,2           a POINTER CHAIN, not folded: the
//!   0x0cc 5169a73e   rlwimi r9,r11,20,28,31   source writes `pBuffer++` twice,
//!   0x0d0 7d7c5b78   mr    r28,r11            and c2 spells the second bump
//!   0x0d4 517dd6be   rlwimi r29,r11,26,26,31  `r11 = r10 + 2` off the first
//!   0x0d8 b1240000   sth   r9,0(r4)           rather than `r4 + 4`
//!   0x0dc 396a0002   addi  r11,r10,2
//!   0x0e0 53dc3832   rlwimi r28,r30,7,0,25
//!   0x0e4 b3a40004   sth   r29,4(r4)          **index 2, not 1** — the source
//!                                             stores through `*(pBuffer + 1)`
//!   0x0e8 b78b0002   sthu  r28,2(r11)
//!   0x0ec 7d645b78   mr    r4,r11
//!   0x0f0 b0eb0000   sth   r7,0(r11)          this arm's NUL is its OWN word:
//!                                             it is at a different displacement
//!                                             from `Lnul`'s and cannot share it
//!  Lcont:
//!   0x0f4 81630004   lwz   r11,OFF_SIZE(r3)
//!   0x0f8 7f1f5840   cmplw cr6,r31,r11
//!   0x0fc 4198ff48   bt    cr6.LT -> Lloop    **THE BACK EDGE** — the only one
//!  Lafter:
//!   0x100 81650000   lwz   r11,0(r5)
//!   0x104 7f085840   cmplw cr6,r8,r11
//!   0x108 4198000c   bt    cr6.LT -> Lstore
//!   0x10c 3ce0803f   lis   r7,HI(K_SIZE_ERR)  **r7 STOPS BEING ZERO HERE** and
//!   0x110 60e70005   ori   r7,r7,LO(K_SIZE_ERR)  becomes the returned status,
//!                                             which is why the two NUL stores
//!                                             above are inside the loop and
//!                                             this is after it
//!  Lstore:
//!   0x114 39680001   addi  r11,r8,1           `*pSize = outputSize + 1`
//!   0x118 91650000   stw   r11,0(r5)
//!   0x11c 4800000c   b     Lret
//!  Lfail:
//!   0x120 3ce08007   lis   r7,HI(K_ARG_ERR)
//!   0x124 60e70057   ori   r7,r7,LO(K_ARG_ERR)
//!  Lret:
//!   0x128 7ce33b78   mr    r3,r7
//!   0x12c 4bfffed4   b     __restgprlr_28  REL24
//! ```
//!
//! **Zero words are chosen by a scheduler or a register allocator.** The whole
//! assignment — r7 the zero-then-status, r8 the running output size, r6 the byte
//! offset, r31 the loop index, r30 the hoisted `1`, r29/r28 the three-byte arm's
//! two continuation bytes, r9/r10/r11 the scratch — is **transcribed**, and the
//! reader refuses any body whose shape would move one of them. Board **#1706**.
//!
//! Every branch is **self-relative** and therefore independent of where the
//! function lands in `.text`; only the two REL24 words encode their own offsets,
//! so they are the only ones that need `base_off`.

use crate::codegen::encode::{
    cr_bi, encode_addi, encode_addis, encode_b_intra, encode_bc, encode_cmplw, encode_cmplwi,
    encode_lhzx, encode_lwz, encode_mr, encode_ori, encode_rlwimi, encode_rlwinm, encode_sth,
    encode_sthu, encode_stw, BO_FALSE, BO_TRUE, CR_BIT_EQ, CR_BIT_GT, CR_BIT_LT, CR_COMPARE,
};
use crate::codegen::frame::FrameLayout;
use crate::codegen::select::out_of_class;
use crate::codegen::OptMode;
use crate::BackendError;
use c2_il::JsonUtf8CopyFn;

/// `this`, the buffer and the size, in r3/r4/r5. r4 is **written** by the
/// three-byte arm, which is why it is not parked anywhere.
const R_THIS: u8 = 3;
const R_BUF: u8 = 4;
const R_SIZE: u8 = 5;
/// The byte offset into `mBuffer`, live across the back edge but **volatile** —
/// the body makes no call, so nothing can clobber it.
const R_OFF: u8 = 6;
/// The literal zero for the whole body, and the returned status after `Lafter`.
const R_ZERO: u8 = 7;
/// The running output size.
const R_OUT: u8 = 8;
const R_T9: u8 = 9;
const R_T10: u8 = 10;
const R_T11: u8 = 11;
/// The three-byte arm's third continuation byte.
const R_C3: u8 = 28;
/// The three-byte arm's second continuation byte.
const R_C2: u8 = 29;
/// The hoisted literal `1`, rotated to `0x80` by two `rlwimi`s.
const R_ONE: u8 = 30;
/// The loop index.
const R_IX: u8 = 31;

/// Four callee-saved GPRs — r28…r31 — which is `__savegprlr_28`, and the whole
/// reason this class needs a frame emitter of its own.
const JSON_SAVED_GPRS: u8 = 4;

/// The frame this class always builds: **no locals, no outgoing slots**, four
/// saved GPRs. [`FrameLayout::out_of_class_ctx_gpr_helper_leaf`] is what turns
/// that into a two-word prologue and a one-word epilogue.
pub fn json_frame() -> FrameLayout {
    FrameLayout { locals: 0, out_slots: 0, saved_gprs: JSON_SAVED_GPRS, saved_fprs: 0 }
}

/// The emitted body plus everything the obj writer needs from it.
pub struct JsonUtf8CopyBody {
    pub text: Vec<u8>,
    /// The two REL24 sites, in ascending `.text` order: `__savegprlr_28` then
    /// `__restgprlr_28`. **This class has no other call site at all.**
    pub bl_offsets: [u32; 2],
    /// Prologue length in bytes — `$M(n)` and the `.pdata` `PrologLen`. **8**,
    /// i.e. two words: there is no `stwu`.
    pub prolog_len: u32,
}

/// Emit the seventy-six words.
pub fn json_utf8_copy_text(
    g: &JsonUtf8CopyFn,
    base_off: u32,
    mode: OptMode,
) -> Result<JsonUtf8CopyBody, BackendError> {
    // The mode is already gated in the parser (board #1638) — this is the second
    // lock, and it is the one that would fire if a future dispatch reached this
    // emitter from somewhere else.
    if mode != OptMode::O1 {
        return Err(out_of_class("json-utf8-copy is /O1 only"));
    }
    let off_size = i16::try_from(g.off_size)
        .map_err(|_| out_of_class("json member offset wider than an lwz displacement"))?;
    let off_buf = i16::try_from(g.off_buffer)
        .map_err(|_| out_of_class("json member offset wider than an lwz displacement"))?;

    let lo16 = |k: i32| (k as u32 & 0xFFFF) as u16;
    let hi16 = |k: i32| ((k as u32 >> 16) as u16) as i16;

    let frame = json_frame();
    let mut t: Vec<u8> = Vec::with_capacity(76 * 4);
    let w = |b: [u8; 4], t: &mut Vec<u8>| t.extend_from_slice(&b);

    // ---- the frameless Class C prologue: two words, one of them a relocation
    let bl_save = base_off + 4;
    t.extend_from_slice(&frame.prologue_gpr_helper_leaf(base_off)?);
    let prolog_len = t.len() as u32;

    // ---- the parameter guards ----------------------------------------------
    w(encode_addi(R_ZERO, 0, 0), &mut t);
    w(encode_cmplwi(CR_COMPARE, R_SIZE, 0), &mut t);
    let at_fail0 = t.len();
    w([0, 0, 0, 0], &mut t);
    w(encode_cmplwi(CR_COMPARE, R_BUF, 0), &mut t);
    let at_else = t.len();
    w([0, 0, 0, 0], &mut t);
    w(encode_lwz(R_T11, R_SIZE, 0), &mut t);
    w(encode_cmplwi(CR_COMPARE, R_T11, 0), &mut t);
    let at_fail1 = t.len();
    w([0, 0, 0, 0], &mut t);

    // ---- Lelse: the loop preheader -----------------------------------------
    let l_else = t.len();
    w(encode_lwz(R_T11, R_THIS, off_size), &mut t);
    w(encode_mr(R_OUT, R_ZERO), &mut t);
    w(encode_mr(R_IX, R_ZERO), &mut t);
    w(encode_cmplwi(CR_COMPARE, R_T11, 0), &mut t);
    let at_after = t.len();
    w([0, 0, 0, 0], &mut t);
    w(encode_mr(R_OFF, R_ZERO), &mut t);
    w(encode_addi(R_ONE, 0, 1), &mut t);

    // ---- Lloop -------------------------------------------------------------
    let l_loop = t.len();
    w(encode_lwz(R_T11, R_THIS, off_buf), &mut t);
    w(encode_addi(R_IX, R_IX, 1), &mut t);
    w(encode_lhzx(R_T11, R_T11, R_OFF), &mut t);
    w(encode_addi(R_OFF, R_OFF, 2), &mut t);
    w(encode_mr(R_T10, R_T11), &mut t);
    w(encode_cmplwi(CR_COMPARE, R_T11, K_ASCII_MAX), &mut t);
    let at_wide = t.len();
    w([0, 0, 0, 0], &mut t);

    // ---- the one-byte arm --------------------------------------------------
    w(encode_lwz(R_T10, R_SIZE, 0), &mut t);
    w(encode_addi(R_OUT, R_OUT, 1), &mut t);
    w(encode_cmplw(CR_COMPARE, R_OUT, R_T10), &mut t);
    let at_cont0 = t.len();
    w([0, 0, 0, 0], &mut t);
    w(encode_rlwinm(R_T11, R_T11, 0, ASCII_MASK_MB, 31), &mut t);
    w(encode_sth(R_T11, R_BUF, 0), &mut t);
    let at_nul = t.len();
    w([0, 0, 0, 0], &mut t);

    // ---- Lwide: the two-byte arm -------------------------------------------
    let l_wide = t.len();
    w(encode_lwz(R_T9, R_SIZE, 0), &mut t);
    w(encode_cmplwi(CR_COMPARE, R_T10, K_TWO_MAX), &mut t);
    let at_three = t.len();
    w([0, 0, 0, 0], &mut t);
    w(encode_addi(R_OUT, R_OUT, 2), &mut t);
    w(encode_cmplw(CR_COMPARE, R_OUT, R_T9), &mut t);
    let at_cont1 = t.len();
    w([0, 0, 0, 0], &mut t);
    w(encode_addi(R_T10, 0, K_LEAD2), &mut t);
    w(encode_mr(R_T9, R_T11), &mut t);
    w(encode_rlwimi(R_T10, R_T11, 26, 27, 31), &mut t);
    w(encode_rlwimi(R_T9, R_ONE, 7, 0, 25), &mut t);
    w(encode_sth(R_T10, R_BUF, 0), &mut t);
    w(encode_sthu(R_T9, R_BUF, 2), &mut t);

    // ---- Lnul: the trailing NUL the one- and two-byte arms share -----------
    let l_nul = t.len();
    w(encode_sthu(R_ZERO, R_BUF, 2), &mut t);
    let at_cont2 = t.len();
    w([0, 0, 0, 0], &mut t);

    // ---- Lthree: the three-byte arm ----------------------------------------
    let l_three = t.len();
    w(encode_addi(R_OUT, R_OUT, 3), &mut t);
    w(encode_cmplw(CR_COMPARE, R_OUT, R_T9), &mut t);
    let at_cont3 = t.len();
    w([0, 0, 0, 0], &mut t);
    w(encode_addi(R_T9, 0, K_LEAD3), &mut t);
    w(encode_addi(R_C2, 0, K_CONT), &mut t);
    w(encode_addi(R_T10, R_BUF, 2), &mut t);
    w(encode_rlwimi(R_T9, R_T11, 20, 28, 31), &mut t);
    w(encode_mr(R_C3, R_T11), &mut t);
    w(encode_rlwimi(R_C2, R_T11, 26, 26, 31), &mut t);
    w(encode_sth(R_T9, R_BUF, 0), &mut t);
    w(encode_addi(R_T11, R_T10, 2), &mut t);
    w(encode_rlwimi(R_C3, R_ONE, 7, 0, 25), &mut t);
    w(encode_sth(R_C2, R_BUF, 4), &mut t);
    w(encode_sthu(R_C3, R_T11, 2), &mut t);
    w(encode_mr(R_BUF, R_T11), &mut t);
    w(encode_sth(R_ZERO, R_T11, 0), &mut t);

    // ---- Lcont: the loop's own test, and the BACK EDGE ---------------------
    let l_cont = t.len();
    w(encode_lwz(R_T11, R_THIS, off_size), &mut t);
    w(encode_cmplw(CR_COMPARE, R_IX, R_T11), &mut t);
    let at_back = t.len();
    let back = encode_bc(BO_TRUE, cr_bi(CR_COMPARE, CR_BIT_LT), l_loop as i32 - at_back as i32)
        .ok_or_else(|| out_of_class("json back edge out of range"))?;
    w(back, &mut t);

    // ---- Lafter ------------------------------------------------------------
    let l_after = t.len();
    w(encode_lwz(R_T11, R_SIZE, 0), &mut t);
    w(encode_cmplw(CR_COMPARE, R_OUT, R_T11), &mut t);
    let at_store = t.len();
    w([0, 0, 0, 0], &mut t);
    w(encode_addis(R_ZERO, 0, hi16(g.k_size_err)), &mut t);
    w(encode_ori(R_ZERO, R_ZERO, lo16(g.k_size_err)), &mut t);

    // ---- Lstore ------------------------------------------------------------
    let l_store = t.len();
    w(encode_addi(R_T11, R_OUT, 1), &mut t);
    w(encode_stw(R_T11, R_SIZE, 0), &mut t);
    let at_ret = t.len();
    w([0, 0, 0, 0], &mut t);

    // ---- Lfail -------------------------------------------------------------
    let l_fail = t.len();
    w(encode_addis(R_ZERO, 0, hi16(g.k_arg_err)), &mut t);
    w(encode_ori(R_ZERO, R_ZERO, lo16(g.k_arg_err)), &mut t);

    // ---- Lret, and the one-word epilogue ------------------------------------
    let l_ret = t.len();
    w(encode_mr(3, R_ZERO), &mut t);
    let bl_rest = base_off + t.len() as u32;
    t.extend_from_slice(&frame.epilogue_gpr_helper_leaf(base_off + t.len() as u32)?);

    // ---- the nine forward branches, patched --------------------------------
    //
    // Every displacement is `target − site`, self-relative, so none of them
    // depends on `base_off`.
    let patch = |t: &mut Vec<u8>, at: usize, word: Option<[u8; 4]>| -> Result<(), BackendError> {
        let word = word.ok_or_else(|| out_of_class("json branch out of range"))?;
        t[at..at + 4].copy_from_slice(&word);
        Ok(())
    };
    let bt = |bit: u8, target: usize, at: usize| {
        encode_bc(BO_TRUE, cr_bi(CR_COMPARE, bit), target as i32 - at as i32)
    };
    let bf = |bit: u8, target: usize, at: usize| {
        encode_bc(BO_FALSE, cr_bi(CR_COMPARE, bit), target as i32 - at as i32)
    };
    patch(&mut t, at_fail0, bt(CR_BIT_EQ, l_fail, at_fail0))?;
    patch(&mut t, at_else, bf(CR_BIT_EQ, l_else, at_else))?;
    patch(&mut t, at_fail1, bf(CR_BIT_EQ, l_fail, at_fail1))?;
    patch(&mut t, at_after, bf(CR_BIT_GT, l_after, at_after))?;
    patch(&mut t, at_wide, bt(CR_BIT_GT, l_wide, at_wide))?;
    patch(&mut t, at_cont0, bf(CR_BIT_LT, l_cont, at_cont0))?;
    patch(&mut t, at_nul, encode_b_intra(l_nul as i32 - at_nul as i32))?;
    patch(&mut t, at_three, bt(CR_BIT_GT, l_three, at_three))?;
    patch(&mut t, at_cont1, bf(CR_BIT_LT, l_cont, at_cont1))?;
    patch(&mut t, at_cont2, encode_b_intra(l_cont as i32 - at_cont2 as i32))?;
    patch(&mut t, at_cont3, bf(CR_BIT_LT, l_cont, at_cont3))?;
    patch(&mut t, at_store, bt(CR_BIT_LT, l_store, at_store))?;
    patch(&mut t, at_ret, encode_b_intra(l_ret as i32 - at_ret as i32))?;

    Ok(JsonUtf8CopyBody { text: t, bl_offsets: [bl_save, bl_rest], prolog_len })
}

/// `wc <= 0x7F` — the one-byte bound, in a `cmplwi` immediate.
///
/// **PINNED, not a field.** It is one of five constants that together are the
/// UTF-8 encoding: changing it without changing [`ASCII_MASK_MB`],
/// [`K_TWO_MAX`], the two `rlwimi` rotate/mask triples and the lead bytes gives
/// a body that is no longer this program, and the reader has no witness of any
/// other combination. Board **#1706** — anything the emitter cannot vary is
/// refused by the reader, and here the reader refuses every one of them.
const K_ASCII_MAX: u16 = 0x7F;

/// `clrlwi r11,r11,25` = `rlwinm r11,r11,0,25,31` — `wc & 0x7F`. PINNED with
/// [`K_ASCII_MAX`], whose mask it is.
const ASCII_MASK_MB: u8 = 25;

/// `wc <= 0x7FF` — the two-byte bound. PINNED.
const K_TWO_MAX: u16 = 0x7FF;

/// `0xC0`, the two-byte lead. PINNED.
const K_LEAD2: i16 = 0xC0;

/// `0xE0`, the three-byte lead. PINNED.
const K_LEAD3: i16 = 0xE0;

/// `0x80`, the continuation-byte mark, materialized as an `li` in the three-byte
/// arm and as the **hoisted `li r30,1` rotated by 7** everywhere else. PINNED —
/// and the two spellings are exactly why: which one c2 uses is not something
/// this emitter chooses.
const K_CONT: i16 = 0x80;

#[cfg(test)]
mod tests {
    use super::*;

    /// `?GetBuffer@JsonWriter@@QAAJPAGPAK@Z` exactly as `jsonwriter.cpp`'s IL
    /// decodes it.
    fn jsonwriter() -> JsonUtf8CopyFn {
        JsonUtf8CopyFn {
            params: vec![0x09f3, 0x09f0, 0x09f1],
            off_buffer: 0,
            off_size: 4,
            k_arg_err: 0x8007_0057u32 as i32,
            k_size_err: 0x803F_0005u32 as i32,
        }
    }

    /// The seventy-six reference words, `work/w-json/probe/ref.obj` disassembled
    /// at the workload's own flags before this file existed.
    const REF: [u32; 76] = [
        0x7d8802a6, 0x4bfffffd, 0x38e00000, 0x2b050000, 0x419a0110, 0x2b040000, 0x409a0010,
        0x81650000, 0x2b0b0000, 0x409a00fc, 0x81630004, 0x7ce83b78, 0x7cff3b78, 0x2b0b0000,
        0x409900c8, 0x7ce63b78, 0x3bc00001, 0x81630000, 0x3bff0001, 0x7d6b322e, 0x38c60002,
        0x7d6a5b78, 0x2b0b007f, 0x41990020, 0x81450000, 0x39080001, 0x7f085040, 0x40980088,
        0x556b067e, 0xb1640000, 0x48000034, 0x81250000, 0x2b0a07ff, 0x41990030, 0x39080002,
        0x7f084840, 0x40980064, 0x394000c0, 0x7d695b78, 0x516ad6fe, 0x53c93832, 0xb1440000,
        0xb5240002, 0xb4e40002, 0x48000044, 0x39080003, 0x7f084840, 0x40980038, 0x392000e0,
        0x3ba00080, 0x39440002, 0x5169a73e, 0x7d7c5b78, 0x517dd6be, 0xb1240000, 0x396a0002,
        0x53dc3832, 0xb3a40004, 0xb78b0002, 0x7d645b78, 0xb0eb0000, 0x81630004, 0x7f1f5840,
        0x4198ff48, 0x81650000, 0x7f085840, 0x4198000c, 0x3ce0803f, 0x60e70005, 0x39680001,
        0x91650000, 0x4800000c, 0x3ce08007, 0x60e70057, 0x7ce33b78, 0x4bfffed4,
    ];

    fn words(b: &[u8]) -> Vec<u32> {
        b.chunks_exact(4).map(|c| u32::from_be_bytes(c.try_into().unwrap())).collect()
    }

    /// **The whole body, against the reference obj** — every word, in order,
    /// including the two relocation placeholders and the one BACK EDGE.
    #[test]
    fn body_matches_the_reference_words() {
        let b = json_utf8_copy_text(&jsonwriter(), 0, OptMode::O1).unwrap();
        assert_eq!(words(&b.text), REF.to_vec(), "the 76 words");
        assert_eq!(b.text.len(), 304);
    }

    /// The two offsets the obj writer reads out of the body, each pinned to a
    /// field of the reference obj rather than to this emitter's arithmetic:
    /// `.pdata` says `PrologLen 2` and `$M2594` sits at `0x8`, and the two REL24
    /// records are at `0x4` and `0x12c`.
    #[test]
    fn relocation_sites_and_prologue_length_match_the_reference() {
        let b = json_utf8_copy_text(&jsonwriter(), 0, OptMode::O1).unwrap();
        assert_eq!(b.prolog_len, 8, "$M(prologue) — TWO words, there is no stwu");
        assert_eq!(b.bl_offsets, [4, 0x12c]);
        let fr = json_frame();
        assert_eq!(fr.save_gpr_helper_name(), Some("__savegprlr_28"));
        assert_eq!(fr.rest_gpr_helper_name(), Some("__restgprlr_28"));
    }

    /// The four free fields each move **exactly one instruction field** and
    /// nothing else — #1767's rule, asserted rather than asserted-about. Every
    /// other word is identical to the reference on every row.
    #[test]
    fn each_field_moves_exactly_the_words_it_names() {
        let base = words(&json_utf8_copy_text(&jsonwriter(), 0, OptMode::O1).unwrap().text);
        let moved = |g: JsonUtf8CopyFn| -> Vec<usize> {
            let w = words(&json_utf8_copy_text(&g, 0, OptMode::O1).unwrap().text);
            (0..76).filter(|&i| w[i] != base[i]).collect()
        };
        // `off_size` is read twice — the guard at 0x28 and the loop test at 0xf4.
        assert_eq!(moved(JsonUtf8CopyFn { off_size: 8, ..jsonwriter() }), vec![10, 61]);
        // `off_buffer` once, at the top of the loop.
        assert_eq!(moved(JsonUtf8CopyFn { off_buffer: 12, ..jsonwriter() }), vec![17]);
        // Each status constant moves its own `lis`+`ori` pair and nothing else.
        assert_eq!(moved(JsonUtf8CopyFn { k_arg_err: 0x8009_1234u32 as i32, ..jsonwriter() }),
                   vec![72, 73]);
        assert_eq!(moved(JsonUtf8CopyFn { k_size_err: 0x8004_5678u32 as i32, ..jsonwriter() }),
                   vec![67, 68]);
    }

    /// The mode gate's second lock (#1638). The parser asks first; this is the
    /// one that fires if a future dispatch reaches the emitter another way.
    #[test]
    fn the_emitter_refuses_every_mode_but_o1() {
        for m in [OptMode::Ox] {
            assert!(json_utf8_copy_text(&jsonwriter(), 0, m).is_err(), "{m:?}");
        }
    }
}
