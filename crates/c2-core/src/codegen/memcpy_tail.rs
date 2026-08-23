//! **W-XTEA2 — the whole-body `memcpy` lowered as a TAIL BRANCH.**
//! `?SetKey@XTEABlockEncrypter@@QAAXPBE@Z`, twelve bytes, one of the four
//! blocked bodies of `src/system/utl/EncryptXTEA.cpp`.
//!
//! ```text
//!   0000  38630010  addi r3,r3,<dst_off>     omitted entirely when it is 0
//!   0004  38a00010  li   r5,<len>
//!   0008  4bfffff8  b    memcpy              REL24, appended by the caller
//! ```
//!
//! Two words, and both are witnessed against real `c2.dll` on this lane's own
//! cells (`work/w-xtea2/probe/mcpytail.cpp`, `/O1 /Oi`):
//!
//! ```text
//!   off16   memcpy(k, uc, 0x10)   addi r3,r3,16 · li r5,16 · b memcpy
//!   len8    memcpy(k, uc, 0x8)    addi r3,r3,16 · li r5,8  · b memcpy
//!   off0    memcpy(n, uc, 0x10)                   li r5,16 · b memcpy
//!   freefn  memcpy(d, s, 0x10)                    li r5,16 · b memcpy
//! ```
//!
//! **The source is free and the order is fixed.** The copy's second argument is
//! already in r4 because it is the function's second argument register — the
//! recognizer has checked exactly that — so no `mr` is emitted, and the
//! destination's `addi` precedes the length's `li` on both cells that have one.
//!
//! **The third word is not here**, for [`super::select::Terminator::TailCall`]'s
//! reason: a branch word encodes its own `.text` offset, so only the caller —
//! which knows where the function lands — can finish it. What *is* different
//! from an ordinary tail call is the callee's NAME: `memcpy` arrives in the IL
//! as intrinsic selector 172 with no `.gl` record at all, so it is minted in
//! `c2_core::comdat` from [`MEMCPY_NAME`] rather than read out of the IL. That
//! is `w-ifn`'s arrangement, taken whole.
//!
//! # Where the minted symbol goes, and why it is NOT `w-ifn`'s answer
//!
//! `CEILING.md` §11's NC-1 item 7 records `memcpy`'s symbol landing **after the
//! first user's `$T` label**, on `coff::Function::helper_externals`. Every
//! witness behind that sentence is a FRAMED user. This class's user is a LEAF
//! and has no `$T`, and both this lane's obj readings put `memcpy` in the
//! **callee region** instead:
//!
//! ```text
//!   work/w-xtea2/ref/xtea.dump     [16] ?SetKey…  [17] memcpy  [18] .text
//!   probe/mcpytail.obj             [13] ?off16…   [14] memcpy  [15] .text
//! ```
//!
//! and the probe's three later users mint **no second symbol**, which is the
//! same dedup `w-ifn`'s `sub2` cell shows on its side. So the placement is a
//! fact about the user's frame class, not about the name, and the two live side
//! by side in `comdat.rs`: this class puts the name only in `Function::calls`
//! and leaves `helper_externals` empty.

use crate::codegen::encode::encode_addi;
use crate::codegen::select::{fits_i16, out_of_class, OptMode};
use crate::BackendError;
use c2_il::MemcpyTail;

/// The minted callee. Not read from the IL — see the module header — and
/// deliberately the SAME constant `w-ifn` mints, imported rather than respelled
/// so the two classes cannot disagree about the spelling of one symbol.
pub use crate::codegen::guard_ret_chain::MEMCPY_NAME;

/// `memcpy`'s destination argument register.
const DST_REG: u8 = 3;
/// `memcpy`'s length argument register. The source is r4 and is never written.
const LEN_REG: u8 = 5;

/// The bytes **before** the tail branch. Empty is not a possible answer: the
/// `li` is always emitted, so the caller's branch offset is 4 or 8.
pub fn memcpy_tail_text(m: &MemcpyTail, mode: OptMode) -> Result<Vec<u8>, BackendError> {
    // **The mode gate is asked here as well as in the parser** (board #1638), so
    // that `select::function_gate` and both writers ask it in exactly one place.
    // Every cell behind the two words is `/O1 /Oi`; at `/Ox` the `/Oi` expansion
    // threshold is a different constant and a copy this class admits could be
    // one c2 expanded inline instead of branching to.
    if mode != OptMode::O1 {
        return Err(out_of_class(
            "the whole-body `memcpy` tail branch at `/Ox`: every cell behind its \
             two words is `/O1 /Oi`, and `/Oi`'s inline-expansion threshold is a \
             different constant there",
        ));
    }
    if !fits_i16(m.dst_off) || m.dst_off < 0 {
        return Err(out_of_class(
            "a `memcpy` destination past the `addi` immediate: the `lis`/`ori` \
             pair a wider member offset needs has no cell here",
        ));
    }
    if !fits_i16(m.len) || m.len <= 0 {
        return Err(out_of_class(
            "a `memcpy` length outside the `li` immediate: the materialisation \
             a wider length needs has no cell here",
        ));
    }
    let mut text = Vec::with_capacity(8);
    // **Zero emits nothing** — cells `off0` and `freefn`, both 8 bytes with no
    // `addi` at all. A port that emitted `addi r3,r3,0` would be one word long
    // and every relocation would still resolve.
    if m.dst_off != 0 {
        text.extend_from_slice(&encode_addi(DST_REG, DST_REG, m.dst_off as i16));
    }
    // `li rD,k` IS `addi rD,r0,k` — one encoder, not two. `38a00010` in the
    // reference obj reads `li 5,16` and `addi 5,0,16` interchangeably.
    text.extend_from_slice(&encode_addi(LEN_REG, 0, m.len as i16));
    Ok(text)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `?SetKey@XTEABlockEncrypter`'s own two words, byte for byte off
    /// `work/w-xtea2/ref/xtea.dump`.
    #[test]
    fn the_target_body_is_addi_then_li() {
        let t = memcpy_tail_text(&MemcpyTail { dst_off: 16, len: 16 }, OptMode::O1).unwrap();
        assert_eq!(t, vec![0x38, 0x63, 0x00, 0x10, 0x38, 0xa0, 0x00, 0x10]);
    }

    /// Cell `len8`: only the `li`'s immediate moves.
    #[test]
    fn the_length_is_the_li_immediate_and_nothing_else_moves() {
        let t = memcpy_tail_text(&MemcpyTail { dst_off: 16, len: 8 }, OptMode::O1).unwrap();
        assert_eq!(t, vec![0x38, 0x63, 0x00, 0x10, 0x38, 0xa0, 0x00, 0x08]);
    }

    /// Cells `off0` and `freefn`: a zero offset emits NO `addi`.
    #[test]
    fn a_zero_destination_offset_emits_no_addi() {
        let t = memcpy_tail_text(&MemcpyTail { dst_off: 0, len: 16 }, OptMode::O1).unwrap();
        assert_eq!(t, vec![0x38, 0xa0, 0x00, 0x10]);
    }

    /// The mode gate, which the parser also carries.
    #[test]
    fn ox_refuses_and_names_the_threshold() {
        let e = memcpy_tail_text(&MemcpyTail { dst_off: 16, len: 16 }, OptMode::Ox).unwrap_err();
        assert!(format!("{e:?}").contains("/Ox"), "{e:?}");
    }

    #[test]
    fn an_out_of_range_offset_or_length_refuses() {
        assert!(memcpy_tail_text(&MemcpyTail { dst_off: 0x8000, len: 16 }, OptMode::O1).is_err());
        assert!(memcpy_tail_text(&MemcpyTail { dst_off: -8, len: 16 }, OptMode::O1).is_err());
        assert!(memcpy_tail_text(&MemcpyTail { dst_off: 0, len: 0 }, OptMode::O1).is_err());
        assert!(memcpy_tail_text(&MemcpyTail { dst_off: 0, len: 0x8000 }, OptMode::O1).is_err());
    }
}
