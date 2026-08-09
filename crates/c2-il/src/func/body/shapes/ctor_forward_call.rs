//! **W-BIQUAD — the constructor that is NOTHING BUT a forwarded member call**,
//! and the smaller half of `src/system/synth_xbox/Biquad.cpp`.
//!
//! ```cpp
//!   Biquad::Biquad(float *flts) { SetCoefficients(flts); }
//! ```
//!
//! ```text
//!   ??0Biquad@DSP@@QAA@PAM@Z                 .text COMDAT, 0x24 B, nrel 1
//!     0000  7d8802a6  mflr r12          the shipped Class A 96-byte frame
//!     0004  9181fff8  stw  r12,-8(r1)
//!     0008  9421ffa0  stwu r1,-96(r1)
//!     000c  7c6a1b78  mr   r10,r3       THE PARK — and it is never read again
//!     0010  4bfffff1  bl   ?SetCoefficients…    REL24, a callee THIS OBJ DEFINES
//!     0014  38210060  addi r1,r1,96
//!     0018  8181fff8  lwz  r12,-8(r1)
//!     001c  7d8803a6  mtlr r12
//!     0020  4e800020  blr
//! ```
//!
//! ## The park is M-RULE's volatile branch, and it is not a free choice
//!
//! `this` is live across the `bl` — a constructor's result is `this` — so it has
//! to go somewhere the callee does not write. `WB_CHOOSER_FINDINGS` §2.3's
//! M-RULE: for a direct call to a function defined **in the same TU** that
//! itself calls nothing of unknown footprint, c2 uses that callee's **exact**
//! register footprint; for anything else it uses the whole volatile set. Both
//! sides are compiled cells of this lane's, not quotations:
//!
//! ```text
//!   work/w-biquad/probe/park_extern.cpp   the callee is an undefined external
//!     mflr · stw · std r31,-16(r1) · stwu · mr r31,r3 · bl · mr r3,r31
//!       · addi · lwz · mtlr · ld r31 · blr                       48 bytes
//!
//!   work/w-biquad/probe/park_local.cpp    the callee is defined here and SMALL
//!     c2 INLINES it — 12 bytes, no call, no frame, no park at all
//! ```
//!
//! So the volatile park has exactly one shape in reach: **a same-TU callee big
//! enough that c2 keeps the call**. That is why this class is not "a constructor
//! forwarding a call" — it is that, *and* a statement about the callee, and the
//! statement is checked one layer down where the callee's lowering is available
//! ([`c2_core::comdat`], through `c2_core::codegen::fp_store_diamond::GPR_FOOTPRINT`).
//! The reader admits the SHAPE and the emitter refuses every callee whose
//! footprint is not established — which is the project's standing preference for
//! a refusal that names its construct over one hidden in a parse failure.
//!
//! **And the restore is absent, which is the second half of the same fact.**
//! The external cell emits `mr r3,r31` after its `bl`; this body does not,
//! because c2 knows the callee leaves r3 alone. A port that emitted the park
//! *and* a restore would be nine words where c2 has nine and wrong in one of
//! them.
//!
//! ## What this production is, in one sentence
//!
//! It is [`super::leaf_store::try_parse_store_run_call`] with an **empty run**.
//! Every gate on the call — the receiver already in slot 0, every argument slot
//! already holding the formal that occupies it, the discarded result, the
//! constructor tail — is [`super::leaf_store::run_call_tail`]'s, asked through
//! that function rather than restated here. `shapes/mod.rs`'s header calls a
//! recognizer that re-parses a shared fact "visibly reinventing it"; the store
//! run's own file already split this tail out for exactly this reason ("two
//! productions end a run on a call, and a second copy of the four locators …
//! is where those four drift apart").
//!
//! The one thing this file adds is the mode gate and the formal count.

use super::super::expr::{parse_formals, BODY_SCOPE_DEPTH};
use super::super::{blk, BodyShape, Block};
use super::leaf_store::run_call_tail;
use super::params::parse_params;
use crate::func::bundle::{opt_word_at, opt_word_mode, OptWordMode};


/// **The recognizer.** `start` is the first byte after the body's own `53`; `lo`
/// is the `4C 4F 11` body marker.
///
/// Non-committal in the sense every sibling production here is: it works on its
/// own cursor and returns `Err`/`None` without side effects, so a body that
/// declines still reports its dispatch arm's blocker and no census key moves.
pub(crate) fn try_parse_ctor_forward_call(
    seg: &[u8],
    start: usize,
    lo: usize,
) -> Result<BodyShape, Block> {
    // **THE MODE GATE LIVES HERE, IN THE PARSER** (board #1638). The emitted
    // frame and the park are both `/O1` readings; at `/Ox` c2's inline budget is
    // a different constant entirely (`WB_INLINE_FINDINGS` F1/F2 measure the
    // ceilings moving with the favour-speed bit), so a body admitted at `/Ox`
    // could be one c2 inlined away. Asked before any body byte is read.
    if opt_word_mode(opt_word_at(seg)) != Some(OptWordMode::O1) {
        return Err(blk(seg, start, "ctorfwd-not-o1"));
    }
    let params = parse_params(seg, lo)?;
    // `this` plus at least one explicit formal, and `parse_params` rather than
    // `parse_formals` so `this` is counted — the park is `mr r10,r3` and a
    // reader that lost `this` would park the wrong register.
    if params.len() < 2 || parse_formals(seg, lo)?.len() + 1 != params.len() {
        return Err(blk(seg, start, "ctorfwd-formals-not-this-plus-n"));
    }
    // The whole call, its four locators and every gate on it —
    // `run_call_tail`'s, over an EMPTY run.
    let (callee_tok, live_args) =
        run_call_tail(seg, start, lo, BODY_SCOPE_DEPTH, &[], &params)
            .ok_or_else(|| blk(seg, start, "ctorfwd-not-a-forwarded-ctor-call"))?;
    Ok(BodyShape::CtorForwardCall { params, callee_tok, live_args })
}
