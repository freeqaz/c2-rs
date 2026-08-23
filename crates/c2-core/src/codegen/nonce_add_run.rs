//! **W-XTEA3 — the two-element 64-bit member run whose addend is a
//! zero-extended 32-bit formal.** `?SetNonce@XTEABlockEncrypter@@QAAXPB_KI@Z`,
//! thirty-two bytes, one of the three bodies still blocking
//! `src/system/utl/EncryptXTEA.cpp` after `w-xtea2`.
//!
//! ```text
//!   0000  e9440000  ld     r10,<src+0>(r4)
//!   0004  78ab0020  clrldi r11,r5,32        the addend, zero-extended ONCE
//!   0008  7d4a5a14  add    r10,r10,r11
//!   000c  f9430000  std    r10,<dst+0>(r3)
//!   0010  e9440008  ld     r10,<src+8>(r4)
//!   0014  7d6a5a14  add    r11,r10,r11      r11's LAST use, so it is the target
//!   0018  f9630008  std    r11,<dst+8>(r3)
//!   001c  4e800020  blr
//! ```
//!
//! **The two scratch registers are not free and they are not symmetric.** The
//! recognizer's six cells (`work/w-xtea3/probe/nonce.cpp`,
//! [`c2_il::func::body::shapes::nonce_add_run`]) fix all four:
//!
//! * the `clrldi` is emitted **once**, before the first `add`, and the second
//!   statement reads its result — a per-statement lowering would be one word
//!   long and the obj would still link;
//! * the first `add` targets **r10** and the second **r11**, because r11 is live
//!   across the first statement and dead after the second. Cell `SetNonce1` —
//!   the same body with ONE element — reads `ld r11 · clrldi r10 ·
//!   add r11,r10,r11`, the two registers exchanged, which is why the run length
//!   is a constant of the class and not a parameter;
//! * cell `SetNonceU64` (a 64-bit addend) emits no `clrldi` at all and a third
//!   plan, so the zero-extension is minted by the addend's width, not by the
//!   store's;
//! * cell `EncOff` moves the destination offsets to 8 and 16 with the source at
//!   0 and 8, so the two bases are carried separately.
//!
//! This body takes **no relocation, defines no label and mints no external** —
//! the reference obj's `.text #7` has `nrel 0` — so it is a `Terminator::None`
//! and `plan_labels`' ordinary 1 for a non-framed function is already the charge
//! `work/w-xtea2/LABGRID.txt`'s `x-setnonce` row measures.

use crate::codegen::encode::{encode_add, encode_blr, encode_ld, encode_rldicl, encode_std};
use crate::codegen::select::{out_of_class, OptMode};
use crate::BackendError;
use c2_il::NonceAddRun;

/// The element stride. The same constant the recognizer fences on, restated here
/// because the emitter's two displacement pairs are what it means.
const ELEM: i32 = 8;

/// The scratch register the load lands in.
const R_LOAD: u8 = 10;
/// The scratch register the zero-extended addend lives in, across both
/// statements.
const R_ADDEND: u8 = 11;
/// The receiver's argument register.
const R_THIS: u8 = 3;
/// The source pointer's argument register.
const R_SRC: u8 = 4;
/// The addend's argument register.
const R_SHIFT: u8 = 5;
/// `clrldi rA,rS,32` is `rldicl rA,rS,0,32`.
const CLRLDI_32: u8 = 32;

/// The largest `ld`/`std` DS displacement: signed 16 bits with the low two bits
/// implied zero.
const DS_MAX: i32 = 0x7FF8;

/// The whole body, `blr` included. Nothing is left for the caller — this class
/// has no branch word that encodes its own `.text` offset.
pub fn nonce_add_run_text(n: &NonceAddRun, mode: OptMode) -> Result<Vec<u8>, BackendError> {
    // **The mode gate is asked here as well as in the parser** (board #1638), so
    // `select::function_gate` and both writers ask it in exactly one place. At
    // `/Ox` the same source emits `add r9,r10,r11` and `std r9` for its second
    // statement — two different registers in an obj that still links, which is
    // board #263's shape.
    if mode != OptMode::O1 {
        return Err(out_of_class(
            "the two-element 64-bit member run at `/Ox`: the second statement's \
             `add` targets r9 there and not r11, so the class's register plan is \
             a `/O1 /Oi` reading and holds at no other mode",
        ));
    }
    for off in [n.dst_off, n.src_off] {
        if !(0..=DS_MAX - ELEM).contains(&off) || off % 4 != 0 {
            return Err(out_of_class(
                "a 64-bit member run whose base is outside the `ld`/`std` DS \
                 displacement, or is not a multiple of four: the `addis`/`addi` \
                 pair a wider base needs has no cell here, and a misaligned one \
                 is not a DS form at all",
            ));
        }
    }
    let mut t = Vec::with_capacity(32);
    t.extend_from_slice(&encode_ld(R_LOAD, R_SRC, n.src_off as i16));
    t.extend_from_slice(&encode_rldicl(R_ADDEND, R_SHIFT, 0, CLRLDI_32));
    t.extend_from_slice(&encode_add(R_LOAD, R_LOAD, R_ADDEND));
    t.extend_from_slice(&encode_std(R_LOAD, R_THIS, n.dst_off as i16));
    t.extend_from_slice(&encode_ld(R_LOAD, R_SRC, (n.src_off + ELEM) as i16));
    // **The destination is r11 and not r10**, because this is r11's last use.
    t.extend_from_slice(&encode_add(R_ADDEND, R_LOAD, R_ADDEND));
    t.extend_from_slice(&encode_std(R_ADDEND, R_THIS, (n.dst_off + ELEM) as i16));
    t.extend_from_slice(&encode_blr());
    Ok(t)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `?SetNonce@XTEABlockEncrypter`'s own thirty-two bytes, word for word off
    /// `work/w-xtea3/ref/xtea.dump`.
    #[test]
    fn the_target_body_is_thirty_two_bytes() {
        let t = nonce_add_run_text(&NonceAddRun { dst_off: 0, src_off: 0 }, OptMode::O1).unwrap();
        assert_eq!(
            t,
            vec![
                0xe9, 0x44, 0x00, 0x00, // ld     r10,0(r4)
                0x78, 0xab, 0x00, 0x20, // clrldi r11,r5,32
                0x7d, 0x4a, 0x5a, 0x14, // add    r10,r10,r11
                0xf9, 0x43, 0x00, 0x00, // std    r10,0(r3)
                0xe9, 0x44, 0x00, 0x08, // ld     r10,8(r4)
                0x7d, 0x6a, 0x5a, 0x14, // add    r11,r10,r11
                0xf9, 0x63, 0x00, 0x08, // std    r11,8(r3)
                0x4e, 0x80, 0x00, 0x20, // blr
            ]
        );
    }

    /// Cell `EncOff`: the destination offsets move to 8 and 16 and the source
    /// stays at 0 and 8 — the two bases are independent.
    #[test]
    fn the_two_bases_move_independently() {
        let t = nonce_add_run_text(&NonceAddRun { dst_off: 8, src_off: 0 }, OptMode::O1).unwrap();
        assert_eq!(&t[12..16], &[0xf9, 0x43, 0x00, 0x08]); // std r10,8(r3)
        assert_eq!(&t[16..20], &[0xe9, 0x44, 0x00, 0x08]); // ld  r10,8(r4)
        assert_eq!(&t[24..28], &[0xf9, 0x63, 0x00, 0x10]); // std r11,16(r3)
    }

    /// The mode gate, which the parser also carries.
    #[test]
    fn ox_refuses_and_names_the_register_that_moves() {
        let e = nonce_add_run_text(&NonceAddRun { dst_off: 0, src_off: 0 }, OptMode::Ox)
            .unwrap_err();
        assert!(format!("{e:?}").contains("r9"), "{e:?}");
    }

    #[test]
    fn an_out_of_range_or_misaligned_base_refuses() {
        assert!(nonce_add_run_text(&NonceAddRun { dst_off: 0x7FF8, src_off: 0 }, OptMode::O1)
            .is_err());
        assert!(nonce_add_run_text(&NonceAddRun { dst_off: 2, src_off: 0 }, OptMode::O1).is_err());
        assert!(nonce_add_run_text(&NonceAddRun { dst_off: 0, src_off: -8 }, OptMode::O1).is_err());
    }

    /// The two `rldicl` cells this lane compiled, which are what separate the
    /// `SH[5]` bit from the `MB[5]` bit in [`encode_rldicl`].
    #[test]
    fn the_rldicl_encoder_reproduces_both_measured_words() {
        // `?SetNonce`'s   78ab0020  clrldi r11,r5,32   = rldicl r11,r5,0,32
        assert_eq!(encode_rldicl(11, 5, 0, 32), [0x78, 0xab, 0x00, 0x20]);
        // `?Encipher`'s   78890022  rldicl r9,r4,32,32 = srdi   r9,r4,32
        assert_eq!(encode_rldicl(9, 4, 32, 32), [0x78, 0x89, 0x00, 0x22]);
        // `?Encipher`'s   79430020  clrldi r3,r10,32   = rldicl r3,r10,0,32
        assert_eq!(encode_rldicl(3, 10, 0, 32), [0x79, 0x43, 0x00, 0x20]);
    }
}
