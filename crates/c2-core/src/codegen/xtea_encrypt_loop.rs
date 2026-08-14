//! **W-XTEA3 — the framed XTEA block loop.**
//! `?Encrypt@XTEABlockEncrypter@@QAAXPBUXTEABlock@@PAU2@@Z`, ninety-six bytes /
//! twenty-four words, the LAST blocked body of
//! `src/system/utl/EncryptXTEA.cpp`.
//!
//! ```text
//!   0000  7d8802a6  mflr   r12                ]
//!   0004  4bfffffd  bl     __savegprlr_26     ] the Class C prologue, three
//!   0008  9421ff70  stwu   r1,-144(r1)        ] words — $M(prologue) is 12
//!   000c  7c7c1b78  mr     r28,r3             `this`, live across the call
//!   0010  7c9f2378  mr     r31,r4             `in`
//!   0014  3b630010  addi   r27,r3,<key_off>   `key = mKey`
//!   0018  7f442850  sub    r26,r5,r4          `offset = out - in`
//!   001c  3bc3fff8  addi   r30,r3,<nonce-8>   the BIASED nonce base
//!   0020  3ba00002  li     r29,<trips>
//!  Lloop:
//!   0024  7f65db78  mr     r5,r27             arg 2
//!   0028  e89e0008  ld     r4,8(r30)          arg 1 — mNonce[i], off the bias
//!   002c  7f83e378  mr     r3,r28             arg 0
//!   0030  4bffffd1  bl     ?Encipher          REL24, a SAME-TU defined symbol
//!   0034  e97f0000  ld     r11,0(r31)         `*(unsigned long long *)in`
//!   0038  37bdffff  addic. r29,r29,-1         the counter AND the cr0 test
//!   003c  7c6b5a78  xor    r11,r3,r11
//!   0040  7d7af92a  stdx   r11,r26,r31        `*(offset + (char *)in) = …`
//!   0044  3bff0008  addi   r31,r31,8          `in += 8`
//!   0048  e97e0008  ld     r11,8(r30)
//!   004c  396b0001  addi   r11,r11,1
//!   0050  f97e0009  stdu   r11,8(r30)         `mNonce[i] += 1` AND the bias step
//!   0054  4082ffd0  bf     cr0.EQ -> Lloop
//!   0058  38210090  addi   r1,r1,144          ] the Class C epilogue, two words,
//!   005c  4bffffa4  b      __restgprlr_26     ] and there is no `blr` at all
//! ```
//!
//! **Three immediates move and nothing else does** — cells `Encrypt4` and
//! `EncOff::Encrypt` in `work/w-xtea3/probe/encrypt.cpp`:
//!
//! * `<key_off>` — `addi r27,r3,16` becomes `addi r27,r3,32`;
//! * `<nonce-8>` — `addi r30,r3,-8` becomes `addi r30,r3,8`, i.e. the member
//!   offset **minus one element**, because `stdu` post-increments the base and
//!   the first `ld` therefore reads `8(r30)`;
//! * `<trips>` — `li r29,2` becomes `li r29,4`.
//!
//! **Two words that are not what a general lowering would emit**, and they are
//! the reason this is a transcription rather than a composition:
//!
//! * `addic. r29,r29,-1` at 0x38 does the decrement AND sets cr0, and it is
//!   scheduled **between** the value load and the `xor` that consumes it — the
//!   loop's control lives inside the data-flow window;
//! * `stdu r11,8(r30)` at 0x50 is one word for two facts (`mNonce[i] += 1` and
//!   the induction step), which is `wb-loop`'s update-form pass — the pass
//!   `counted_accum_loop` declines by name (#1981).
//!
//! The back edge is `bf 2` off cr0 and **not** `bdnz`: the body makes a call, so
//! CTR is not available across it.

use crate::codegen::encode::{
    encode_addi, encode_addic_record, encode_ld, encode_mr, encode_stdu, encode_stdx,
    encode_subf, encode_xor,
};
use crate::codegen::frame::FrameLayout;
use crate::codegen::select::{out_of_class, OptMode};
use crate::BackendError;
use c2_il::XteaEncryptLoop;
use crate::codegen::labels::Form;
use crate::codegen::reach;

/// Callee-saved GPRs: r26–r31, so the helpers are `__savegprlr_26` /
/// `__restgprlr_26`. Six, which is what `FrameLayout`'s own rule turns into the
/// obj's 144-byte frame: `align16(80 + 8·(1+6)) = 144`.
const SAVED_GPRS: u8 = 6;
/// The widest call the body makes: `?Encipher(this, nonce, key)`. Floored at 8
/// by `FrameLayout`, so it does not reach the size — carried because it is what
/// the body actually does.
const OUT_SLOTS: u8 = 3;

/// The block element's byte stride.
const ELEM: i32 = 8;

const R_A0: u8 = 3;
const R_A1: u8 = 4;
const R_A2: u8 = 5;
const R_T11: u8 = 11;
/// `offset`, the destination bias.
const R_OFFSET: u8 = 26;
/// `key`.
const R_KEY: u8 = 27;
/// `this`.
const R_THIS: u8 = 28;
/// the trip counter.
const R_COUNT: u8 = 29;
/// the biased `mNonce` base, post-incremented by the `stdu`.
const R_NONCE: u8 = 30;
/// `in`.
const R_IN: u8 = 31;

/// `bf 2` — branch if cr0.EQ is false. `BO = 4` (branch if the bit is clear),
/// `BI = 2` (cr0's EQ bit).
const BO_FALSE: u8 = 4;
const BI_CR0_EQ: u8 = 2;

/// The frame this class allocates.
pub fn xtea_frame() -> FrameLayout {
    FrameLayout { locals: 0, out_slots: OUT_SLOTS, saved_gprs: SAVED_GPRS, saved_fprs: 0 }
}

/// The emitted body plus everything the obj writer needs from it.
#[derive(Debug)]
pub struct XteaEncryptLoopBody {
    pub text: Vec<u8>,
    /// The three REL24 sites in ascending `.text` order: `__savegprlr_26`,
    /// `?Encipher`, `__restgprlr_26`.
    pub bl_offsets: [u32; 3],
    /// Prologue length in bytes — `$M(n)` and the `.pdata` `PrologLen`. **12**.
    pub prolog_len: u32,
}

/// Emit the twenty-four words. `base_off` is the function's `.text` offset,
/// which the two frame-helper branch words encode.
pub fn xtea_encrypt_loop_text(
    x: &XteaEncryptLoop,
    base_off: u32,
    mode: OptMode,
) -> Result<XteaEncryptLoopBody, BackendError> {
    // **The mode gate is asked here as well as in the parser** (board #1638). At
    // `/Ox` c2 inlines `?Encipher` into this body.
    if mode != OptMode::O1 {
        return Err(out_of_class(
            "the framed XTEA block loop at `/Ox`: c2 inlines the same-TU callee \
             there, which `work/w-xtea2/LABGRID.txt` reads as a label stride of \
             41 with the callee defined against 8 without it",
        ));
    }
    if !(1..=0x7FFF).contains(&x.trips) {
        return Err(out_of_class(
            "an XTEA block count outside the `li` immediate: the `lis`/`ori` \
             pair a wider trip count needs has no cell here",
        ));
    }
    let key_off = i16::try_from(x.key_off)
        .map_err(|_| out_of_class("an XTEA key member offset wider than an `addi` immediate"))?;
    // **The nonce base is emitted BIASED by one element.** The loop's `stdu`
    // post-increments it, so the first `ld` reads `8(r30)` and the biased value
    // is what has to fit — cell `EncOff::Encrypt` is `addi r30,r3,8` for a
    // member at 16.
    let nonce_biased = i16::try_from(x.nonce_off - ELEM).map_err(|_| {
        out_of_class("an XTEA nonce member offset wider than an `addi` immediate")
    })?;

    let frame = xtea_frame();
    let mut t: Vec<u8> = Vec::with_capacity(96);

    // ---- the Class C prologue: three words, one of them a relocation --------
    let bl_save = base_off + 4;
    t.extend_from_slice(&frame.prologue_gpr_helper(base_off)?);
    let prolog_len = t.len() as u32;

    // ---- the loop's invariants ---------------------------------------------
    t.extend_from_slice(&encode_mr(R_THIS, R_A0));
    t.extend_from_slice(&encode_mr(R_IN, R_A1));
    t.extend_from_slice(&encode_addi(R_KEY, R_A0, key_off));
    // `sub rD,rA,rB` is `subf rD,rB,rA` — the operand order is the whole reason
    // this is `encode_subf(R_OFFSET, R_A1, R_A2)` and not the other way round.
    t.extend_from_slice(&encode_subf(R_OFFSET, R_A1, R_A2));
    t.extend_from_slice(&encode_addi(R_NONCE, R_A0, nonce_biased));
    t.extend_from_slice(&encode_addi(R_COUNT, 0, x.trips as i16));

    // ---- the loop ----------------------------------------------------------
    let loop_top = t.len();
    t.extend_from_slice(&encode_mr(R_A2, R_KEY));
    t.extend_from_slice(&encode_ld(R_A1, R_NONCE, ELEM as i16));
    t.extend_from_slice(&encode_mr(R_A0, R_THIS));
    let bl_call = base_off + t.len() as u32;
    t.extend_from_slice(&crate::codegen::calls::encode_call_branch(bl_call));
    t.extend_from_slice(&encode_ld(R_T11, R_IN, 0));
    t.extend_from_slice(&encode_addic_record(R_COUNT, R_COUNT, -1));
    t.extend_from_slice(&encode_xor(R_T11, R_A0, R_T11));
    t.extend_from_slice(&encode_stdx(R_T11, R_OFFSET, R_IN));
    t.extend_from_slice(&encode_addi(R_IN, R_IN, ELEM as i16));
    t.extend_from_slice(&encode_ld(R_T11, R_NONCE, ELEM as i16));
    t.extend_from_slice(&encode_addi(R_T11, R_T11, 1));
    t.extend_from_slice(&encode_stdu(R_T11, R_NONCE, ELEM as i16));
    let back = loop_top as i32 - t.len() as i32;
    t.extend_from_slice(&reach::direct(
        Form::Bc { bo: BO_FALSE, bi: BI_CR0_EQ },
        back,
        "an XTEA block loop back edge",
    )?);

    // ---- the Class C epilogue: two words, and no `blr` ----------------------
    let epi = base_off + t.len() as u32;
    t.extend_from_slice(&frame.epilogue_gpr_helper(epi)?);
    let bl_rest = epi + 4;

    debug_assert_eq!(t.len(), 96);
    Ok(XteaEncryptLoopBody { text: t, bl_offsets: [bl_save, bl_call, bl_rest], prolog_len })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `?Encrypt@XTEABlockEncrypter`'s own 96 bytes, word for word off
    /// `work/w-xtea3/ref/xtea.dump`.
    const TARGET: &[u8] = &[
        0x7d, 0x88, 0x02, 0xa6, 0x4b, 0xff, 0xff, 0xfd, 0x94, 0x21, 0xff, 0x70, 0x7c, 0x7c, 0x1b,
        0x78, 0x7c, 0x9f, 0x23, 0x78, 0x3b, 0x63, 0x00, 0x10, 0x7f, 0x44, 0x28, 0x50, 0x3b, 0xc3,
        0xff, 0xf8, 0x3b, 0xa0, 0x00, 0x02, 0x7f, 0x65, 0xdb, 0x78, 0xe8, 0x9e, 0x00, 0x08, 0x7f,
        0x83, 0xe3, 0x78, 0x4b, 0xff, 0xff, 0xd1, 0xe9, 0x7f, 0x00, 0x00, 0x37, 0xbd, 0xff, 0xff,
        0x7c, 0x6b, 0x5a, 0x78, 0x7d, 0x7a, 0xf9, 0x2a, 0x3b, 0xff, 0x00, 0x08, 0xe9, 0x7e, 0x00,
        0x08, 0x39, 0x6b, 0x00, 0x01, 0xf9, 0x7e, 0x00, 0x09, 0x40, 0x82, 0xff, 0xd0, 0x38, 0x21,
        0x00, 0x90, 0x4b, 0xff, 0xff, 0xa4,
    ];

    fn target() -> XteaEncryptLoop {
        XteaEncryptLoop {
            callee: "?Encipher@XTEABlockEncrypter@@AAA_K_KPAI@Z".to_string(),
            key_off: 16,
            nonce_off: 0,
            trips: 2,
        }
    }

    #[test]
    fn the_target_body_is_ninety_six_bytes() {
        let b = xtea_encrypt_loop_text(&target(), 0, OptMode::O1).unwrap();
        assert_eq!(b.text, TARGET);
        assert_eq!(b.prolog_len, 12);
        assert_eq!(b.bl_offsets, [4, 0x30, 0x5c]);
    }

    /// The frame arithmetic, from `FrameLayout`'s own rule and not fitted:
    /// `align16(80 + 8·7) = 144`, and `32 − 6 = 26`.
    #[test]
    fn the_frame_is_one_hundred_and_forty_four_bytes_over_six_saved_gprs() {
        let f = xtea_frame();
        assert_eq!(f.size(), 144);
        assert_eq!(f.save_gpr_helper_name(), Some("__savegprlr_26"));
        assert_eq!(f.rest_gpr_helper_name(), Some("__restgprlr_26"));
    }

    /// Cell `Encrypt4`: the trip count reaches exactly the `li r29` immediate.
    #[test]
    fn the_trip_count_reaches_exactly_one_word() {
        let b = xtea_encrypt_loop_text(
            &XteaEncryptLoop { trips: 4, ..target() },
            0,
            OptMode::O1,
        )
        .unwrap();
        assert_eq!(&b.text[32..36], &[0x3b, 0xa0, 0x00, 0x04]);
        assert_eq!(&b.text[..32], &TARGET[..32]);
        assert_eq!(&b.text[36..], &TARGET[36..]);
    }

    /// Cell `EncOff::Encrypt`: the two member offsets reach exactly two `addi`
    /// immediates, and the nonce one is emitted BIASED by an element.
    #[test]
    fn the_two_member_offsets_reach_exactly_two_words_and_the_nonce_is_biased() {
        let b = xtea_encrypt_loop_text(
            &XteaEncryptLoop { key_off: 32, nonce_off: 16, ..target() },
            0,
            OptMode::O1,
        )
        .unwrap();
        assert_eq!(&b.text[20..24], &[0x3b, 0x63, 0x00, 0x20]); // addi r27,r3,32
        assert_eq!(&b.text[28..32], &[0x3b, 0xc3, 0x00, 0x08]); // addi r30,r3,8
        assert_eq!(&b.text[..20], &TARGET[..20]);
        assert_eq!(&b.text[24..28], &TARGET[24..28]);
        assert_eq!(&b.text[32..], &TARGET[32..]);
    }

    /// **THREE branch words encode their own `.text` offset, not two**, and the
    /// third is the one a reading of "the frame helpers move" would miss: the
    /// `bl ?Encipher` at 0x30 is a REL24 too. Every other word is
    /// offset-independent.
    #[test]
    fn exactly_three_words_move_with_the_function() {
        let b = xtea_encrypt_loop_text(&target(), 0x100, OptMode::O1).unwrap();
        assert_eq!(b.bl_offsets, [0x104, 0x130, 0x15c]);
        let moved: Vec<usize> =
            (0..24).filter(|i| b.text[i * 4..i * 4 + 4] != TARGET[i * 4..i * 4 + 4]).collect();
        assert_eq!(moved, vec![1, 12, 23], "the two frame helpers AND the callee");
    }

    #[test]
    fn ox_refuses_and_names_the_inliner() {
        let e = xtea_encrypt_loop_text(&target(), 0, OptMode::Ox).unwrap_err();
        assert!(format!("{e:?}").contains("inlines"), "{e:?}");
    }

    /// The three encoders this class is the first caller of, each pinned to the
    /// word the reference obj carries.
    #[test]
    fn the_three_new_encoders_reproduce_their_measured_words() {
        assert_eq!(encode_addic_record(29, 29, -1), [0x37, 0xbd, 0xff, 0xff]);
        assert_eq!(encode_stdx(11, 26, 31), [0x7d, 0x7a, 0xf9, 0x2a]);
        assert_eq!(encode_stdu(11, 30, 8), [0xf9, 0x7e, 0x00, 0x09]);
    }
}
