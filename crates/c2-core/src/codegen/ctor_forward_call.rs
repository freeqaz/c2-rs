//! **W-BIQUAD — the emitter for the constructor that is nothing but a forwarded
//! member call.**
//!
//! The reader's accept/refuse boundary and the source shape are on
//! [`c2_il::func::body::shapes::ctor_forward_call`]; this file is the nine words
//! and nothing else. The one thing that varies in them is the callee's name.
//!
//! ```text
//!   ??0Biquad@DSP@@QAA@PAM@Z                 .text COMDAT, 0x24 B, nrel 1
//!
//!    off  word       instruction         why it is this word
//!   ----  --------   -----------------   ------------------------------------
//!   0x00  7d8802a6   mflr r12            the shipped Class A 96-byte frame,
//!   0x04  9181fff8   stw  r12,-8(r1)     built by `FrameLayout` so this class
//!   0x08  9421ffa0   stwu r1,-96(r1)     cannot disagree with W10/W11 about it
//!   0x0c  7c6a1b78   mr   r10,r3         THE PARK. `this` is live across the
//!                                        `bl` (a constructor's result is
//!                                        `this`) and the callee does not write
//!                                        r10, so M-RULE takes a VOLATILE and
//!                                        the frame saves no GPR.
//!   0x10  4bfffff1   bl   <callee>       REL24. `disp = -(own .text offset)`.
//!   0x14  38210060   addi r1,r1,96       …and NO `mr r3,r10`: the callee does
//!   0x18  8181fff8   lwz  r12,-8(r1)     not write r3 either, so the parked
//!   0x1c  7d8803a6   mtlr r12            value never has to come back.
//!   0x20  4e800020   blr
//! ```
//!
//! ## The park and the missing restore are ONE fact, and it is about the CALLEE
//!
//! `WB_CHOOSER_FINDINGS` §2.3's M-RULE: a value live across a call goes in the
//! lowest-cost register that is neither an argument register at that call nor in
//! the callee's register footprint — and for a direct call to a same-TU function
//! that itself calls nothing of unknown footprint, the footprint is the **exact**
//! set of registers that callee writes, not the whole volatile set. §2.6 counts
//! 14 volatile-side sites against 9 callee-saved, and names this very
//! constructor as one of them.
//!
//! This lane compiled both sides rather than quoting them
//! (`work/w-biquad/probe/park_{extern,local}.cpp`):
//!
//! ```text
//!   callee UNDEFINED EXTERNAL   mflr · stw · std r31,-16(r1) · stwu
//!                               · mr r31,r3 · bl · mr r3,r31 · addi · lwz
//!                               · mtlr · ld r31 · blr            48 bytes
//!   callee SAME-TU and SMALL    c2 inlines it: 12 bytes, no call, no frame
//! ```
//!
//! So *"which register"* and *"is there a restore"* are both answers about the
//! callee, and neither is readable from this function's own IL. That is why
//! [`ctor_forward_call_text`] does not decide it: [`crate::comdat`] does, where
//! the callee's own lowering exists, and it admits exactly the classes whose GPR
//! footprint this port has stated (today: one).
//!
//! **A port that guessed would be right about eight of the nine words**, which
//! is why the refusal is where it is.

use crate::BackendError;
use crate::codegen::calls::encode_call_branch;
use crate::codegen::encode::encode_mr;
use crate::codegen::frame::FrameLayout;

/// The volatile the park takes. `WB_CHOOSER_FINDINGS` §2.3: **r11 when the value
/// does not cross a call, r10 when it does** — 6 witnesses for the first and 8
/// for the second, and `this` here crosses one.
const PARK_REG: u8 = 10;
/// `this` arrives in r3 and the park copies out of it.
const THIS_REG: u8 = 3;

/// A forwarding constructor's emitted body, with the offsets the caller needs.
pub struct CtorForwardBody {
    pub text: Vec<u8>,
    /// Absolute `.text` offset of the `bl <callee>` — the REL24 site.
    pub bl_offset: u32,
    /// Prologue length in bytes, which becomes the `$M(n)` label and the
    /// `.pdata` `PrologLen`.
    pub prolog_len: u32,
}

/// Emit the `.text` for a **W-BIQUAD forwarding constructor**.
///
/// `base_off` is the function's start within the `.text` section it lands in —
/// 0 under `/Gy`, its packed offset otherwise — because the `bl` displacement
/// follows MSVC's `disp = −(own .text offset)` convention. Threaded rather than
/// hardcoded to 0, which is the live wrong-bytes emit
/// [`crate::codegen::framed_call_text`]'s doc records for the framed class.
pub fn ctor_forward_call_text(base_off: u32) -> Result<CtorForwardBody, BackendError> {
    let frame = FrameLayout::default();
    let prologue = frame.prologue()?;
    let epilogue = frame.epilogue()?;
    let prolog_len = prologue.len() as u32;
    let mut text: Vec<u8> = Vec::with_capacity(prologue.len() + 8 + epilogue.len());
    text.extend_from_slice(&prologue);
    // The park, between the prologue and the `bl` — the same slot
    // `Selected::Framed`'s argument setup occupies, and it is NOT an argument
    // setup: every argument is already in the register the call wants (that is
    // the reader's own gate), so nothing here marshals anything.
    text.extend_from_slice(&encode_mr(PARK_REG, THIS_REG));
    let bl_offset = base_off + text.len() as u32;
    text.extend_from_slice(&encode_call_branch(bl_offset));
    // **No post-call word.** `framed_call_text` emits `addi r3,r3,k` here for
    // its `return g(x) + k` shape; this body has nothing between the `bl` and
    // the epilogue, and emitting `addi r3,r3,0` would be ten words where c2 has
    // nine.
    text.extend_from_slice(&epilogue);
    Ok(CtorForwardBody { text, bl_offset, prolog_len })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn words(t: &[u8]) -> Vec<u32> {
        t.chunks(4).map(|w| u32::from_be_bytes(w.try_into().unwrap())).collect()
    }

    /// **The nine words, against `work/w-biquad/real.obj`.**
    /// `??0Biquad@DSP@@QAA@PAM@Z` is 36 bytes and its `bl` is at 0x10 with
    /// `disp = −0x10` and LK set.
    #[test]
    fn the_nine_words_are_the_reference_obj() {
        let b = ctor_forward_call_text(0).expect("in class");
        assert_eq!(b.text.len(), 36, "the reference `.text` COMDAT is 36 bytes");
        assert_eq!(b.prolog_len, 12);
        assert_eq!(b.bl_offset, 0x10);
        assert_eq!(
            words(&b.text),
            vec![
                0x7d8802a6, 0x9181fff8, 0x9421ffa0, // the 96-byte frame
                0x7c6a1b78,                         // mr r10,r3 — THE PARK
                0x4bfffff1,                         // bl <callee>, REL24
                0x38210060, 0x8181fff8, 0x7d8803a6, 0x4e800020,
            ]
        );
    }

    /// **There is no post-call word.** [`crate::codegen::framed_call_text`]
    /// emits `addi r3,r3,k` between the `bl` and the epilogue for its
    /// `return g(x) + k` shape; this class has nothing there, and `addi r3,r3,0`
    /// would be ten words where c2 has nine. Asserted positively — the word
    /// after the `bl` is the epilogue's first — rather than as an absence.
    #[test]
    fn nothing_sits_between_the_call_and_the_epilogue() {
        let b = ctor_forward_call_text(0).expect("in class");
        let w = words(&b.text);
        let bl = (b.bl_offset / 4) as usize;
        assert_eq!(w[bl + 1], 0x38210060, "`addi r1,r1,96`, the epilogue");
    }

    /// **`base_off` is threaded, not hardcoded.** The `bl` displacement follows
    /// MSVC's `disp = −(own .text offset)` convention, and a class built at a
    /// hardcoded 0 is a live wrong-bytes emit for any function that is not first
    /// in a packed `.text` — the defect `framed_call_text`'s own doc records
    /// having shipped. This class is refused on the packed path today, so
    /// nothing exercises the threading; it is asserted here rather than left to
    /// be discovered by whatever lifts that refusal.
    #[test]
    fn the_call_displacement_follows_the_function() {
        for base in [0u32, 8, 0x24, 0x100] {
            let b = ctor_forward_call_text(base).expect("in class");
            assert_eq!(b.bl_offset, base + 0x10);
            let word = words(&b.text)[4];
            assert_eq!(word & 1, 1, "LK is set");
            let disp = ((word & 0x03ff_fffc) as i32) << 6 >> 6;
            assert_eq!(disp, -((base as i32) + 0x10));
        }
    }
}
