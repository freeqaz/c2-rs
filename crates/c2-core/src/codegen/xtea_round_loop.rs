//! **W-XTEA3 — the XTEA round loop.** `?Encipher@XTEABlockEncrypter@@AAA_K_KPAI@Z`,
//! one hundred and sixteen bytes / twenty-nine words.
//!
//! ```text
//!   -- prologue --------------------------------------------------------------
//!   0000  39000004  li     r8,<trips>          the ONE word the trip count reaches
//!   0004  548a003e  slwi   r10,r4,0            v1 = low 32 of the nonce
//!   0008  78890022  rldicl r9,r4,32,32         v2 = high 32
//!   000c  39600000  li     r11,0               sum = 0
//!   0010  7d0903a6  mtctr  r8
//!   -- the round, twenty words, SOFTWARE-PIPELINED ---------------------------
//!   0014  5568173a  rlwinm r8,r11,2,28,29      (sum & 3) * 4
//!   0018  55262036  slwi   r6,r9,4
//!   001c  5527d97e  srwi   r7,r9,5
//!   0020  7ce73278  xor    r7,r7,r6
//!   0024  7d08282e  lwzx   r8,r8,r5            key[sum & 3]
//!   0028  7ce74a14  add    r7,r7,r9
//!   002c  7d085a14  add    r8,r8,r11
//!   0030  3d6b9e37  addis  r11,r11,<delta hi>  the round constant, HALF of it,
//!   0034  7ce84278  xor    r8,r7,r8            …split around this `xor`…
//!   0038  396b79b9  addi   r11,r11,<delta lo>  …and finished here
//!   003c  7d485214  add    r10,r8,r10          v1 += …
//!   0040  5567bf3a  rlwinm r7,r11,23,28,29     ((sum >> 11) & 3) * 4
//!   0044  5548d97e  srwi   r8,r10,5
//!   0048  55462036  slwi   r6,r10,4
//!   004c  7d083278  xor    r8,r8,r6
//!   0050  7ce7282e  lwzx   r7,r7,r5
//!   0054  7d085214  add    r8,r8,r10
//!   0058  7ce75a14  add    r7,r7,r11
//!   005c  7d083a78  xor    r8,r8,r7
//!   0060  7d284a14  add    r9,r8,r9            v2 += …
//!   0064  4200ffb0  bdnz   -80
//!   -- the returned pair ------------------------------------------------------
//!   0068  79430020  clrldi r3,r10,32           the LOW half
//!   006c  7923000e  rldimi r3,r9,32,0          the HIGH half spliced in
//!   0070  4e800020  blr
//! ```
//!
//! **This is a TRANSCRIPTION and the module says so.** The `addis`/`addi` pair
//! at 0x30/0x38 is split around an `xor` that does not depend on it, and the
//! second half-round's index word at 0x40 is hoisted above the first half's last
//! use of r11. Nothing in this port derives that order. What makes the class
//! honest is the recognizer's fences
//! ([`c2_il::func::body::shapes::xtea_round_loop`]): everything that is not one
//! of the two measured parameters is required to be exactly what the four
//! compiled cells carry.
//!
//! **The two parameters, and the four words they reach:**
//!
//! * the trip count — cell `Encipher8` moves `li r8,4` to `li r8,8` and nothing
//!   else in 116 bytes;
//! * the returned halves' order — cell `EncipherSwap` exchanges exactly the
//!   register fields of the last two words.
//!
//! The round constant is carried rather than fixed, and it reaches the two
//! immediates of the split pair. Its low half must be below `0x8000`: above
//! that the `addis` immediate takes the borrow adjustment, which no cell here
//! witnesses and which the recognizer refuses.
//!
//! No relocation, no pooled constant, no label symbol — the reference obj's
//! `.text #8` reads `nrel 0`. So this is a [`super::select::Terminator::None`],
//! and the whole of its label story is
//! [`c2_il::IlFunction::label_lead`]'s `+2`.

use crate::codegen::encode::{
    mop_add, mop_addi, mop_addis, mop_bdnz, mop_blr, mop_lwzx, mop_mtctr,
    mop_rldicl, mop_rldimi, mop_rlwinm, mop_xor,
};
use crate::codegen::mop::{ops_to_bytes, MachineOp, Ops};
use crate::codegen::select::{out_of_class, OptMode};
use crate::BackendError;
use c2_il::XteaRoundLoop;

/// The round constant the four cells carry, as an `i32`. `0x9E3779B9` does not
/// fit a signed 32-bit literal, so it is written once here rather than cast at
/// every use.
pub const DELTA: i32 = 0x9E37_79B9u32 as i32;

/// The nonce's argument register.
const R_NONCE: u8 = 4;
/// The key pointer's argument register.
const R_KEY: u8 = 5;
/// The returned value's register.
const R_RET: u8 = 3;
/// `v1` — the low half.
const R_V1: u8 = 10;
/// `v2` — the high half.
const R_V2: u8 = 9;
/// `sum` — the round accumulator.
const R_SUM: u8 = 11;
/// The three scratch registers the round uses, in the roles the words give them.
const R_T6: u8 = 6;
const R_T7: u8 = 7;
const R_T8: u8 = 8;

/// The loop's own displacement: twenty words back from the `bdnz`.
const BACK_EDGE: i32 = -80;

/// `slwi rA,rS,k` is `rlwinm rA,rS,k,0,31-k`; `srwi rA,rS,k` is
/// `rlwinm rA,rS,32-k,k,31`. Written out rather than shared with
/// [`mop_rlwinm`]'s callers because the two forms differ only in their field
/// arithmetic and a single helper for both is how a mask ends up one bit wide.
///
/// **S1c (i): both build a [`MachineOp`] rather than a word.** They are the
/// file's only obstruction to an op stream and the conversion is exactly the
/// return type — the field arithmetic, which is the part the doc above warns
/// about, is untouched.
fn slwi(ra: u8, rs: u8, k: u8) -> MachineOp {
    mop_rlwinm(ra, rs, k, 0, 31 - k)
}
fn srwi(ra: u8, rs: u8, k: u8) -> MachineOp {
    mop_rlwinm(ra, rs, 32 - k, k, 31)
}

/// The whole body, `blr` included. Nothing is left for the caller: this class
/// has no branch word that encodes its own `.text` offset — the `bdnz` is
/// self-relative and takes no relocation.
pub fn xtea_round_loop_text(x: &XteaRoundLoop, mode: OptMode) -> Result<Vec<u8>, BackendError> {
    Ok(ops_to_bytes(&xtea_round_loop_ops(x, mode)?))
}

/// **S1c (i): the same twenty-nine words as an op stream**, reachable by a
/// caller.
///
/// The header calls this body a TRANSCRIPTION of c2's software-pipelined
/// schedule — the `addis`/`addi` pair split around an unrelated `xor`, the
/// second half-round's index word hoisted above the first half's last use of
/// r11. That schedule is the content of the class, and an op stream is the form
/// in which it is *readable*: the goal's permuter searches orderings, and it
/// can only search an ordering it can see.
pub fn xtea_round_loop_ops(x: &XteaRoundLoop, mode: OptMode) -> Result<Ops, BackendError> {
    // **The mode gate is asked here as well as in the parser** (board #1638). At
    // `/Ox` this source is 1,352 bytes with a `__savegprlr_28` frame, six
    // relocations and the loop fully unrolled.
    if mode != OptMode::O1 {
        return Err(out_of_class(
            "the XTEA round loop at `/Ox`: the same source is 1,352 bytes there \
             with a `__savegprlr_28` frame and six relocations, and the loop is \
             fully unrolled",
        ));
    }
    if !(1..=0x7FFF).contains(&x.trips) {
        return Err(out_of_class(
            "an XTEA round count outside the `li` immediate: the `lis`/`ori` \
             pair a wider trip count needs has no cell here",
        ));
    }
    let lo = (x.delta & 0xFFFF) as u16;
    if lo >= 0x8000 {
        return Err(out_of_class(
            "an XTEA round constant whose low half is at or above 0x8000: the \
             `addis` immediate then takes the borrow adjustment the `addi`'s \
             sign extension forces, and no cell here witnesses it",
        ));
    }
    let hi = (x.delta >> 16) as i16;

    // The two registers the returned pair reads. Cell `EncipherSwap` is the only
    // thing that moves them, and it moves exactly these two fields.
    let (ret_lo, ret_hi) = if x.swapped { (R_V2, R_V1) } else { (R_V1, R_V2) };

    let mut t: Ops = Vec::with_capacity(29);
    // -- prologue ----------------------------------------------------------
    t.push(mop_addi(R_T8, 0, x.trips as i16)); // li r8,<trips>
    t.push(slwi(R_V1, R_NONCE, 0)); // slwi r10,r4,0
    t.push(mop_rldicl(R_V2, R_NONCE, 32, 32)); // rldicl r9,r4,32,32
    t.push(mop_addi(R_SUM, 0, 0)); // li r11,0
    t.push(mop_mtctr(R_T8));
    // -- the round ---------------------------------------------------------
    // `(sum & 3) * 4`: rotate left 2 with a mask of bits 28..29 does the AND and
    // the scale in one word.
    t.push(mop_rlwinm(R_T8, R_SUM, 2, 28, 29));
    t.push(slwi(R_T6, R_V2, 4));
    t.push(srwi(R_T7, R_V2, 5));
    t.push(mop_xor(R_T7, R_T7, R_T6));
    t.push(mop_lwzx(R_T8, R_T8, R_KEY));
    t.push(mop_add(R_T7, R_T7, R_V2));
    t.push(mop_add(R_T8, R_T8, R_SUM));
    t.push(mop_addis(R_SUM, R_SUM, hi));
    t.push(mop_xor(R_T8, R_T7, R_T8));
    t.push(mop_addi(R_SUM, R_SUM, lo as i16));
    t.push(mop_add(R_V1, R_T8, R_V1));
    // `((sum >> 11) & 3) * 4`: `32 - 11 + 2 == 23`, and the same 28..29 mask.
    t.push(mop_rlwinm(R_T7, R_SUM, 23, 28, 29));
    t.push(srwi(R_T8, R_V1, 5));
    t.push(slwi(R_T6, R_V1, 4));
    t.push(mop_xor(R_T8, R_T8, R_T6));
    t.push(mop_lwzx(R_T7, R_T7, R_KEY));
    t.push(mop_add(R_T8, R_T8, R_V1));
    t.push(mop_add(R_T7, R_T7, R_SUM));
    t.push(mop_xor(R_T8, R_T8, R_T7));
    t.push(mop_add(R_V2, R_T8, R_V2));
    t.push(mop_bdnz(BACK_EDGE).ok_or_else(|| {
        out_of_class("an XTEA round loop whose back edge does not fit a `bdnz`")
    })?);
    // -- the returned pair -------------------------------------------------
    t.push(mop_rldicl(R_RET, ret_lo, 0, 32));
    t.push(mop_rldimi(R_RET, ret_hi, 32, 0));
    t.push(mop_blr());
    debug_assert_eq!(t.len(), 29, "the class's body length is a constant");
    Ok(t)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `?Encipher@XTEABlockEncrypter`'s own 116 bytes, word for word off
    /// `work/w-xtea3/ref/xtea.dump`.
    const TARGET: &[u8] = &[
        0x39, 0x00, 0x00, 0x04, 0x54, 0x8a, 0x00, 0x3e, 0x78, 0x89, 0x00, 0x22, 0x39, 0x60, 0x00,
        0x00, 0x7d, 0x09, 0x03, 0xa6, 0x55, 0x68, 0x17, 0x3a, 0x55, 0x26, 0x20, 0x36, 0x55, 0x27,
        0xd9, 0x7e, 0x7c, 0xe7, 0x32, 0x78, 0x7d, 0x08, 0x28, 0x2e, 0x7c, 0xe7, 0x4a, 0x14, 0x7d,
        0x08, 0x5a, 0x14, 0x3d, 0x6b, 0x9e, 0x37, 0x7c, 0xe8, 0x42, 0x78, 0x39, 0x6b, 0x79, 0xb9,
        0x7d, 0x48, 0x52, 0x14, 0x55, 0x67, 0xbf, 0x3a, 0x55, 0x48, 0xd9, 0x7e, 0x55, 0x46, 0x20,
        0x36, 0x7d, 0x08, 0x32, 0x78, 0x7c, 0xe7, 0x28, 0x2e, 0x7d, 0x08, 0x52, 0x14, 0x7c, 0xe7,
        0x5a, 0x14, 0x7d, 0x08, 0x3a, 0x78, 0x7d, 0x28, 0x4a, 0x14, 0x42, 0x00, 0xff, 0xb0, 0x79,
        0x43, 0x00, 0x20, 0x79, 0x23, 0x00, 0x0e, 0x4e, 0x80, 0x00, 0x20,
    ];

    #[test]
    fn the_target_body_is_one_hundred_and_sixteen_bytes() {
        let t = xtea_round_loop_text(
            &XteaRoundLoop { trips: 4, delta: DELTA, swapped: false },
            OptMode::O1,
        )
        .unwrap();
        assert_eq!(t, TARGET);
    }

    /// Cell `Encipher8`: the trip count reaches the `li r8` immediate and
    /// **nothing else in 116 bytes**.
    #[test]
    fn the_trip_count_reaches_exactly_one_word() {
        let t = xtea_round_loop_text(
            &XteaRoundLoop { trips: 8, delta: DELTA, swapped: false },
            OptMode::O1,
        )
        .unwrap();
        assert_eq!(&t[0..4], &[0x39, 0x00, 0x00, 0x08]);
        assert_eq!(&t[4..], &TARGET[4..]);
    }

    /// Cell `EncipherSwap`: the returned halves exchange exactly the two
    /// register fields of the last two words.
    #[test]
    fn swapping_the_returned_halves_moves_exactly_two_words() {
        let t = xtea_round_loop_text(
            &XteaRoundLoop { trips: 4, delta: DELTA, swapped: true },
            OptMode::O1,
        )
        .unwrap();
        // The `bdnz` ends at 104; the two words the swap reaches are the two
        // after it, and the `blr` behind them does not move.
        assert_eq!(&t[..104], &TARGET[..104]);
        assert_eq!(&t[104..108], &[0x79, 0x23, 0x00, 0x20]); // clrldi r3,r9,32
        assert_eq!(&t[108..112], &[0x79, 0x43, 0x00, 0x0e]); // rldimi r3,r10,32,0
        assert_eq!(&t[112..116], &TARGET[112..116]); // blr
    }

    #[test]
    fn ox_refuses_and_names_the_frame_it_would_need() {
        let e = xtea_round_loop_text(
            &XteaRoundLoop { trips: 4, delta: DELTA, swapped: false },
            OptMode::Ox,
        )
        .unwrap_err();
        assert!(format!("{e:?}").contains("__savegprlr_28"), "{e:?}");
    }

    #[test]
    fn an_out_of_range_trip_count_or_a_borrowing_constant_refuses() {
        let o = OptMode::O1;
        assert!(xtea_round_loop_text(
            &XteaRoundLoop { trips: 0, delta: DELTA, swapped: false }, o
        )
        .is_err());
        // A low half at or above 0x8000 needs the borrow adjustment.
        assert!(xtea_round_loop_text(
            &XteaRoundLoop { trips: 4, delta: 0x9E37_8000u32 as i32, swapped: false }, o
        )
        .is_err());
    }

    /// `rldimi`'s own measured word, which is what separates its extended
    /// opcode from `rldicl`'s.
    ///
    /// **Still asked of the ENCODER, not of the twin, after S1c (i) moved the
    /// body to `mop_rldimi`.** `encode_rldimi` is `mop_rldimi(..).word()`, so
    /// the two cannot disagree — but this assertion's subject is the *word*,
    /// and rewriting it as `mop_rldimi(..).word()` would restate the definition
    /// instead of checking the measurement. The import is explicit for the same
    /// reason the emitter's is not.
    #[test]
    fn the_rldimi_encoder_reproduces_the_measured_word() {
        use crate::codegen::encode::encode_rldimi;
        assert_eq!(encode_rldimi(3, 9, 32, 0), [0x79, 0x23, 0x00, 0x0e]);
        assert_eq!(encode_rldimi(3, 10, 32, 0), [0x79, 0x43, 0x00, 0x0e]);
    }
}
