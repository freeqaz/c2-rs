//! The frame layout: sizes, thresholds, prologue and epilogue.
//!
//! Owned by the serial spine lane together with `calls.rs` and `coff.rs`
//! (`docs/ARCHITECTURE_SEAMS.md` §7). Every constant here is a *captured*
//! threshold, not a derived one — see `docs/CODEGEN_FRAMED_CALLS.md` §1.2 —
//! and the tests at the bottom are the cross-check against the witnesses.

use crate::BackendError;
use crate::codegen::encode::{
    encode_addi,
    encode_blr,
    encode_ld,
    encode_ldr,
    encode_lfd,
    encode_std,
    encode_stfd,
    encode_stwu,
    mop_lwz,
    mop_mtlr,
    mop_stw,
};
use crate::codegen::mop;
use crate::codegen::select::out_of_class;

// ---------------------------------------------------------------------------
// The X360 stack frame — a model, not a constant
// ---------------------------------------------------------------------------

/// Fixed head of every MSVC X360 stack frame, in bytes: 16 bytes of linkage
/// (the back chain at `0(r1)` and one reserved doubleword) plus a 64-byte
/// outgoing-parameter home area — 8 slots, the ABI floor. Measured: every local
/// this project has captured is addressed at `80(r1)` or above, and the frame of
/// a body with no locals and no saved registers is `align16(80 + 8) = 96`.
///
/// It is a *floor on the parameter area*, not a floor on the frame: a function
/// whose widest call passes more than eight arguments pushes the locals up
/// (`FrameLayout::locals_base`).
pub const FRAME_HEAD: u32 = 80;

/// The ABI floor on the outgoing-parameter home area, in 8-byte slots.
pub const FRAME_MIN_OUT_SLOTS: u32 = 8;

/// The largest `saved_gprs + saved_fprs` for which the frame-size rule is exact.
/// Past it the allocator spills to slots the rule does not model and the frame
/// grows by an unmeasured amount (39 of 480 designed compiles, all at
/// `nSaved ≥ 18` — `docs/CODEGEN_FRAMED_CALLS.md` §1.3). Refused, not guessed.
pub const FRAME_MAX_SAVED_NO_SPILL: u8 = 17;

/// The page the prologue's stack probes step by, and the unit of the
/// `_RtlCheckStack12` threshold. Measured: the probes are `ld r12,-4096(r1)`,
/// `ld r12,-8192(r1)`, … .
pub const FRAME_PAGE: u32 = 4096;

// ---------------------------------------------------------------------------
// The frame's six fixed instructions — four composed, two still literal
// ---------------------------------------------------------------------------
//
// **2026-08-26, lane `w-mopfold` — four of these six stopped being literals.**
// Board **#3637** found eleven live instruction-word productions outside
// `mop::encode_op`, of which these four were duplicates: the port's own READ
// base-word table already composed the identical 32 bits, by a second rule.
// They agreed to the bit, which is exactly why nothing caught them — a byte
// compare cannot see a *concurring* second producer, and `mismatch 0` stays
// silent right up until the day the two rules disagree, at which point the
// defect is indistinguishable from a lowering bug.
//
// `mop::const_word` evaluates `mop::encode_op`'s composition at compile time,
// so these are still `const u32` with the same values and the same consumers
// (`prologue`, `epilogue`, and `coff::ehscope::plan_text`); not one emitted
// byte moved. The historical literals are pinned in `word_seam`'s inventory,
// which is where a regression in this direction goes red.
//
// **The two that did NOT fold are `FRAME_MFLR_R12` and `FRAME_STWUX`, and the
// reason is a missing table row rather than a disagreement** — see their own
// notes below. `word_seam` carries them as inventoried exceptions and *arms*
// the refusal: the day `mfspr` or `stwux` is transcribed into `mop::OPCODES`,
// the seam test turns red on these two and asks for this fold to finish.

/// `stw r12,-8(r1)` — spill the just-`mflr`'d link register into the caller's
/// frame. The LR slot is the topmost doubleword of *this* function's frame
/// (`F-8(r1)` after the `stwu`), which is why it is written before the frame is
/// allocated and read back after it is freed.
pub(crate) const FRAME_LR_STORE: u32 = mop::const_word(mop_stw(12, 1, -8));

/// `lwz r12,-8(r1)` — the matching reload.
pub(crate) const FRAME_LR_LOAD: u32 = mop::const_word(mop_lwz(12, 1, -8));

/// `mflr r12` (`mfspr r12,8`).
///
/// **Still a literal, and this one is NOT a duplicate.** `mop::OPCODES` has no
/// `mfspr` row and `mop::plan` has no arm for its form, so nothing in this port
/// can compose this word — there is no second rule here to disagree with.
/// c2 does carry the row (`docs/whitebox/ref/ENCODE_OPCODES.txt`: opcode
/// `0x00e6`, base `7c0002a6`, **form 54**, arm `10bfa76a`), so finishing the
/// fold is a *transcription*, not a derivation — but a transcription is an
/// **adoption**, it moves `DISCLOSURE.md`'s `W-MOP-2`/`W-MOP-3` counts, and
/// form 54's field placement is not among the 27 arms lane `w-read-r2` read.
/// Priced and declined by `w-mopfold`; see `word_seam`'s inventory row.
pub(crate) const FRAME_MFLR_R12: u32 = 0x7D88_02A6;

/// `mtlr r12` (`mtspr 8,r12`).
pub(crate) const FRAME_MTLR_R12: u32 = mop::const_word(mop_mtlr(12));

/// `stwux r1,r1,r12` — opcode 31, XO 183. The variable-size frame allocation
/// c2 emits immediately after `bl _RtlCheckStack12`, which takes `−F` in r12.
/// Captured, and pinned by a test, for a shape [`FrameLayout`] refuses: keeping
/// the measured word beside the threshold that gates it is what stops the next
/// implementer from guessing it.
///
/// **Still a literal, and not a duplicate either** — same reason as
/// [`FRAME_MFLR_R12`], one step closer to foldable: c2's row is opcode
/// `0x017f`, base `7c00016e`, **form 61**, and this port *already* has a field
/// plan for form 61 (it emits `stdx`). So `stwux` needs one transcribed row and
/// no new arm, which makes it the cheapest of the three refusals — and still an
/// adoption this lane may not make.
pub const FRAME_STWUX: u32 = 0x7C21_616E;

/// `lwz r1,0(r1)` — deallocate through the back chain, used when `+F` does not
/// fit an `addi` immediate.
const FRAME_BACKCHAIN: u32 = mop::const_word(mop_lwz(1, 1, 0));


/// `__savegprlr_N` indexed by `N`, for every `N` a layout this emitter can build
/// can produce: `N = 32 − saved_gprs` with `saved_gprs` in `3..=18`, so
/// `N ∈ 14..=29`. Empty strings outside that window — the two accessors return
/// `None` for them because [`FrameLayout::gpr_helper_n`] never yields one.
///
/// A table rather than `format!` so the names are `&'static str`: they are
/// symbol-table entries in `coff::Function`, whose lists borrow.
const SAVE_GPR_HELPERS: [&str; 30] = [
    /* 0 */ "",
    /* 1 */ "",
    /* 2 */ "",
    /* 3 */ "",
    /* 4 */ "",
    /* 5 */ "",
    /* 6 */ "",
    /* 7 */ "",
    /* 8 */ "",
    /* 9 */ "",
    /* 10 */ "",
    /* 11 */ "",
    /* 12 */ "",
    /* 13 */ "",
    /* 14 */ "__savegprlr_14",
    /* 15 */ "__savegprlr_15",
    /* 16 */ "__savegprlr_16",
    /* 17 */ "__savegprlr_17",
    /* 18 */ "__savegprlr_18",
    /* 19 */ "__savegprlr_19",
    /* 20 */ "__savegprlr_20",
    /* 21 */ "__savegprlr_21",
    /* 22 */ "__savegprlr_22",
    /* 23 */ "__savegprlr_23",
    /* 24 */ "__savegprlr_24",
    /* 25 */ "__savegprlr_25",
    /* 26 */ "__savegprlr_26",
    /* 27 */ "__savegprlr_27",
    /* 28 */ "__savegprlr_28",
    /* 29 */ "__savegprlr_29",
];

/// `__restgprlr_N`, the epilogue half of the pair above.
const REST_GPR_HELPERS: [&str; 30] = [
    /* 0 */ "",
    /* 1 */ "",
    /* 2 */ "",
    /* 3 */ "",
    /* 4 */ "",
    /* 5 */ "",
    /* 6 */ "",
    /* 7 */ "",
    /* 8 */ "",
    /* 9 */ "",
    /* 10 */ "",
    /* 11 */ "",
    /* 12 */ "",
    /* 13 */ "",
    /* 14 */ "__restgprlr_14",
    /* 15 */ "__restgprlr_15",
    /* 16 */ "__restgprlr_16",
    /* 17 */ "__restgprlr_17",
    /* 18 */ "__restgprlr_18",
    /* 19 */ "__restgprlr_19",
    /* 20 */ "__restgprlr_20",
    /* 21 */ "__restgprlr_21",
    /* 22 */ "__restgprlr_22",
    /* 23 */ "__restgprlr_23",
    /* 24 */ "__restgprlr_24",
    /* 25 */ "__restgprlr_25",
    /* 26 */ "__restgprlr_26",
    /* 27 */ "__restgprlr_27",
    /* 28 */ "__restgprlr_28",
    /* 29 */ "__restgprlr_29",
];

/// The **measured X360 frame layout** of one function: how much local/spill
/// space it needs above the fixed head, and how many callee-saved GPRs and FPRs
/// it keeps live across its calls.
///
/// Every rule below was read out of reference objs compiled by the real
/// toolchain at `/Ox /GS- /c`; the probe sources are one-liners of the form
/// `int g(…); T f(…){ … g(…) … }` and the byte evidence is in
/// `docs/CODEGEN_PPC_MVP.md` §"The frame model".
///
/// **Sizing.**
///
/// ```text
///   locals_base = align16(16 + 8 × max(out_slots, 8))
///   size        = align16( max(16 + 8 × max(out_slots, 8),
///                              locals_base + locals)
///                          + 8 × (saved_gprs + saved_fprs) + 8 )
/// ```
///
/// — the linkage + outgoing-parameter area, the locals above it, one 8-byte slot
/// per saved register, and the 8-byte LR slot, rounded to 16. **Two independent
/// derivations agree on it**: 44 witnesses here (which is where the
/// stack-probing and `_RtlCheckStack12` rules below come from, `locals` up to
/// 200,000) and the 441-of-480 designed refutation sweep in
/// `docs/CODEGEN_FRAMED_CALLS.md` §1.2, which is where the `out_slots` term
/// comes from — every probe of this rung had `out_slots ≤ 8`, where the two
/// forms coincide at `align16(80 + locals + 8 + 8×saved)`. Exact while the
/// allocator does not spill; see [`FRAME_MAX_SAVED_NO_SPILL`].
///
/// Every row the roadmap had recorded as "96 B for one by-value temporary, 112 B
/// for two" is really the *saved-register* count, not a temporary count:
///
/// ```text
///   saved GPRs 0 1 2 3 4 5 6 7   frame 96 96 112 112 128 128 144 144
///   locals 1 → 96   locals 9 → 112   locals 64 → 160   locals 3600 → 3696
/// ```
///
/// **Register file.** Callee-saved GPRs are `r(32−n)…r31` and FPRs
/// `f(32−n)…f31` — always a contiguous run ending at the top of the file. They
/// share one descending array of 8-byte slots directly under the LR slot, GPRs
/// first: with two GPRs and one FPR, `r31` is at `−16(r1)`, `r30` at `−24` and
/// `f31` at `−32`. GPRs are stored with `std` (64-bit) and FPRs with `stfd`.
///
/// **Helpers.** Above a measured threshold c2 calls a save/restore helper
/// instead of open-coding the stores: **3 or more GPRs** →
/// `bl __savegprlr_(32−n)` / `b __restgprlr_(32−n)` (which save and restore the
/// LR too, so the `stw r12,-8(r1)` disappears and the epilogue *tail-branches*
/// into the restore helper), and **4 or more FPRs** →
/// `addi r12,r1,−(8 + 8×gprs)` + `bl __savefpr_(32−n)` /
/// `bl __restfpr_(32−n)`. Both are REL24 calls to externals.
///
/// **Stack probing.** A frame smaller than five pages is probed inline, one
/// `ld r12,−4096k(r1)` per page boundary crossed (`floor((F−1)/4096)` of them),
/// then `stwu r1,−F(r1)`. From five pages up it is
/// `li r12,−F` (or `lis`+`ori` past 32768) + `bl _RtlCheckStack12` +
/// `stwux r1,r1,r12`.
///
/// The emitter below covers only the layouts that need **no external helper and
/// no stack check** — everything else is refused by name, because those shapes
/// need a second REL24 site per function that the obj writer does not model.
/// The thresholds are therefore load-bearing gates, not decoration.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FrameLayout {
    /// Bytes of addressed locals + compiler temporaries, above
    /// [`Self::locals_base`].
    pub locals: u32,
    /// **The argument count of the widest call this function makes** (0 for a
    /// leaf), floored at [`FRAME_MIN_OUT_SLOTS`]. Measured to be the maximum
    /// over the body's calls and not the last or first one — two calls of
    /// different arity in either order give the same frame
    /// (`docs/CODEGEN_FRAMED_CALLS.md` §1.2).
    pub out_slots: u8,
    /// Callee-saved GPRs: `r(32−n)…r31`.
    pub saved_gprs: u8,
    /// Callee-saved FPRs: `f(32−n)…f31`.
    pub saved_fprs: u8,
}

impl FrameLayout {
    /// The number of 8-byte register save slots, including the LR slot.
    fn save_slots(&self) -> u32 {
        1 + self.saved_gprs as u32 + self.saved_fprs as u32
    }

    /// Total callee-saved registers, the input to the spill boundary.
    fn n_saved(&self) -> u8 {
        self.saved_gprs.saturating_add(self.saved_fprs)
    }

    /// Where addressed locals start, relative to the new SP: the linkage area
    /// plus the outgoing-parameter home area, **16-aligned**. The alignment is
    /// measured, not assumed — with 9 outgoing slots the parameter area ends at
    /// SP+88 and the locals still start at SP+96, which an 8-aligned model
    /// mispredicts (`docs/CODEGEN_FRAMED_CALLS.md` §1.2).
    pub fn locals_base(&self) -> u32 {
        self.param_area_end().div_ceil(16) * 16
    }

    fn param_area_end(&self) -> u32 {
        16 + 8 * (self.out_slots as u32).max(FRAME_MIN_OUT_SLOTS)
    }

    /// The allocated frame size in bytes (the `stwu` displacement, negated).
    pub fn size(&self) -> u32 {
        let body = self.param_area_end().max(self.locals_base() + self.locals);
        (body + 8 * self.save_slots()).div_ceil(16) * 16
    }

    /// `-8` for the LR slot, then `-16, -24, …` for the saved registers: GPRs
    /// from `r31` downwards, then FPRs from `f31` downwards.
    fn gpr_slot(&self, i: u8) -> i16 {
        -16 - 8 * i as i16
    }
    fn fpr_slot(&self, i: u8) -> i16 {
        -16 - 8 * (self.saved_gprs as i16 + i as i16)
    }

    /// Page boundaries the frame crosses, i.e. how many inline probes the
    /// prologue emits. `F = 4096` crosses none; `F = 4112` crosses one.
    pub fn probe_pages(&self) -> u32 {
        self.size().saturating_sub(1) / FRAME_PAGE
    }

    /// True when the frame is allocated through `_RtlCheckStack12` rather than
    /// inline probes + `stwu`. Measured boundary: `F = 20464` is inline and
    /// `F = 20480 = 5 × 4096` is the helper.
    pub fn needs_stack_check(&self) -> bool {
        self.size() >= 5 * FRAME_PAGE
    }

    /// True when the GPR saves go through `__savegprlr_N` / `__restgprlr_N`.
    /// Measured: 2 saved GPRs are open-coded `std`s, 3 are the helper.
    pub fn needs_gpr_helper(&self) -> bool {
        self.saved_gprs >= 3
    }

    /// True when the FPR saves go through `__savefpr_N` / `__restfpr_N`.
    /// Measured: 3 saved FPRs are open-coded `stfd`s, 4 are the helper — a
    /// *different* threshold from the GPR one, which is why they are two
    /// predicates and not one.
    pub fn needs_fpr_helper(&self) -> bool {
        self.saved_fprs >= 4
    }

    /// The `__savegprlr_N` / `__restgprlr_N` **width** for this layout:
    /// `N = 32 − saved_gprs`, and `None` when the saves are open-coded.
    ///
    /// Measured across `_29` (3 saved) … `_24` (8 saved) in
    /// `docs/CODEGEN_FRAMED_CALLS.md` §2.3, and again at `_26` (6 saved) on
    /// `src/xdk/xlrc/xlrcimpl.cpp`'s own obj, which is the witness the Class C
    /// emitter below is graded against.
    pub fn gpr_helper_n(&self) -> Option<u8> {
        self.needs_gpr_helper().then(|| 32 - self.saved_gprs)
    }

    /// `__savegprlr_N` — the prologue helper's external name, or `None`.
    ///
    /// A `&'static str` out of [`SAVE_GPR_HELPERS`] rather than a formatted
    /// `String`, because the name goes straight into `coff::Function`'s
    /// `&'a str` symbol lists and a per-call allocation there would need a
    /// lifetime the frame layout does not have.
    pub fn save_gpr_helper_name(&self) -> Option<&'static str> {
        self.gpr_helper_n().and_then(|n| SAVE_GPR_HELPERS.get(n as usize).copied())
    }

    /// `__restgprlr_N` — the epilogue helper's external name, or `None`.
    pub fn rest_gpr_helper_name(&self) -> Option<&'static str> {
        self.gpr_helper_n().and_then(|n| REST_GPR_HELPERS.get(n as usize).copied())
    }

    /// The refusal reason for a **Class C** layout — one whose GPR saves go
    /// through `__savegprlr_N` — or `None`.
    ///
    /// This is deliberately a *second* predicate rather than a flag on
    /// [`Self::out_of_class_ctx`]. That one is the gate every shipped emitter
    /// runs, and it refuses the helper; widening it would change the verdict of
    /// every class at once, where the whole safety argument for W-XLR is that
    /// the helper prologue is reachable **only** from an emitter that asked for
    /// it by name. A layout that does *not* need the helper is refused here for
    /// the mirror-image reason: Class C's three-word prologue is wrong for it.
    pub fn out_of_class_ctx_gpr_helper(&self) -> Option<&'static str> {
        if !self.needs_gpr_helper() {
            return Some("frame-gpr-helper-class-without-a-helper");
        }
        if self.needs_fpr_helper() {
            return Some("frame-savefpr-helper");
        }
        if self.needs_stack_check() {
            return Some("frame-rtlcheckstack12");
        }
        if self.n_saved() > FRAME_MAX_SAVED_NO_SPILL {
            return Some("frame-allocator-spill");
        }
        None
    }

    /// **Class C prologue** — `mflr r12` / `bl __savegprlr_N` / `stwu r1,−F(r1)`.
    ///
    /// Three words whatever `saved_gprs` is, because the helper does the stores:
    /// the register saves that Class B open-codes as `std`s between the `stw
    /// r12,−8(r1)` and the `stwu` are gone, **and so is that `stw`** — the
    /// helper expects LR in r12 and spills it itself, which is why `mflr r12`
    /// still runs first. `docs/CODEGEN_FRAMED_CALLS.md` §2.3, and byte for byte
    /// `work/w-xlr/ref/xlrcimpl/dis.txt` offsets 0x00…0x08.
    ///
    /// `base_off` is the function's own `.text` offset; the `bl` word encodes
    /// `−(its own offset)` in MSVC's placeholder convention, so it is the one
    /// word here that is not position-independent. The REL24 site is
    /// `base_off + 4`.
    pub fn prologue_gpr_helper(&self, base_off: u32) -> Result<Vec<u8>, BackendError> {
        if let Some(ctx) = self.out_of_class_ctx_gpr_helper() {
            return Err(out_of_class(ctx));
        }
        let neg = i32::try_from(self.size())
            .ok()
            .and_then(|v| i16::try_from(-v).ok())
            .ok_or_else(|| out_of_class("frame larger than a stwu immediate"))?;
        let mut w: Vec<u8> = Vec::with_capacity(12);
        w.extend_from_slice(&FRAME_MFLR_R12.to_be_bytes());
        w.extend_from_slice(&crate::codegen::calls::encode_call_branch(base_off + 4));
        w.extend_from_slice(&encode_stwu(1, 1, neg));
        Ok(w)
    }

    /// **Class C epilogue** — `addi r1,r1,F` / `b __restgprlr_N`, and
    /// **there is no `blr` at all**.
    ///
    /// The helper restores the saved GPRs *and* LR and then returns on the
    /// caller's behalf, so the function's last word is an **unlinked** REL24
    /// branch (`LK = 0`). Every other epilogue this port emits ends in
    /// `0x4E800020`; this one ends in a relocation, which is why it is a
    /// separate function and not a branch inside [`Self::epilogue`].
    ///
    /// The REL24 site is `base_off + 4` — the second of the two words.
    pub fn epilogue_gpr_helper(&self, base_off: u32) -> Result<Vec<u8>, BackendError> {
        if let Some(ctx) = self.out_of_class_ctx_gpr_helper() {
            return Err(out_of_class(ctx));
        }
        let pos = i16::try_from(self.size())
            .map_err(|_| out_of_class("frame larger than an addi immediate"))?;
        let mut w: Vec<u8> = Vec::with_capacity(8);
        w.extend_from_slice(&encode_addi(1, 1, pos));
        w.extend_from_slice(&crate::codegen::calls::encode_tail_branch(base_off + 4));
        Ok(w)
    }

    /// The refusal reason for a **frameless Class C** layout — one whose GPR
    /// saves go through `__savegprlr_N` and which allocates **no stack frame at
    /// all** — or `None`.
    ///
    /// A **third** predicate for the same reason [`Self::out_of_class_ctx_gpr_helper`]
    /// is a second one: `out_of_class_ctx` is the gate every shipped emitter
    /// runs and it refuses the helper outright, and the helper gate refuses a
    /// layout with no frame is not its business either. Each of the three
    /// prologues is wrong for the other two shapes, so each refuses the other
    /// two by name and none of them can be reached from an emitter that did not
    /// ask for it.
    ///
    /// The extra conditions over [`Self::out_of_class_ctx_gpr_helper`] are the
    /// three things that would make c2 allocate a frame: an addressed local, an
    /// outgoing-parameter area (i.e. a call), and saved FPRs. All three are zero
    /// on the one witness — `?GetBuffer@JsonWriter@@QAAJPAGPAK@Z`, whose IL
    /// carries **no CALL token at all** — and any of them being non-zero is a
    /// `stwu` this shape does not emit.
    pub fn out_of_class_ctx_gpr_helper_leaf(&self) -> Option<&'static str> {
        if let Some(ctx) = self.out_of_class_ctx_gpr_helper() {
            return Some(ctx);
        }
        if self.locals != 0 {
            return Some("frame-gpr-helper-leaf-with-a-local");
        }
        if self.out_slots != 0 {
            return Some("frame-gpr-helper-leaf-with-outgoing-slots");
        }
        if self.saved_fprs != 0 {
            return Some("frame-gpr-helper-leaf-with-saved-fprs");
        }
        None
    }

    /// **Frameless Class C prologue** — `mflr r12` / `bl __savegprlr_N`, and
    /// **there is no `stwu` at all**.
    ///
    /// TWO words, against Class C's three. A function that saves GPRs but has no
    /// addressed local, no outgoing-parameter area and makes no call needs no
    /// frame, so c2 allocates none and the helper's stores land below `r1`.
    /// Byte for byte `work/w-json/probe/ref.obj` offsets 0x00…0x04, and the
    /// `.pdata` record's `PrologLen` for that function is **2**.
    ///
    /// `base_off` is the function's own `.text` offset; the `bl` word encodes
    /// `−(its own offset)` in MSVC's placeholder convention. The REL24 site is
    /// `base_off + 4`.
    pub fn prologue_gpr_helper_leaf(&self, base_off: u32) -> Result<Vec<u8>, BackendError> {
        if let Some(ctx) = self.out_of_class_ctx_gpr_helper_leaf() {
            return Err(out_of_class(ctx));
        }
        let mut w: Vec<u8> = Vec::with_capacity(8);
        w.extend_from_slice(&FRAME_MFLR_R12.to_be_bytes());
        w.extend_from_slice(&crate::codegen::calls::encode_call_branch(base_off + 4));
        Ok(w)
    }

    /// **Frameless Class C epilogue** — `b __restgprlr_N` and nothing else.
    ///
    /// ONE word, against Class C's two: there is no frame to free, so the
    /// `addi r1,r1,F` is gone as well as the `blr`. The REL24 site is
    /// `base_off` itself — this word *is* the epilogue.
    pub fn epilogue_gpr_helper_leaf(&self, base_off: u32) -> Result<Vec<u8>, BackendError> {
        if let Some(ctx) = self.out_of_class_ctx_gpr_helper_leaf() {
            return Err(out_of_class(ctx));
        }
        Ok(crate::codegen::calls::encode_tail_branch(base_off).to_vec())
    }

    /// The refusal reason for a layout this emitter cannot produce, or `None`.
    /// Each arm is a shape whose prologue contains a second REL24 call site.
    pub fn out_of_class_ctx(&self) -> Option<&'static str> {
        if self.needs_gpr_helper() {
            return Some("frame-savegprlr-helper");
        }
        if self.needs_fpr_helper() {
            return Some("frame-savefpr-helper");
        }
        if self.needs_stack_check() {
            return Some("frame-rtlcheckstack12");
        }
        // Unreachable behind the two helper thresholds today (3 and 4), and kept
        // as the second lock because the *sizing* rule stops being exact here
        // and a wrong `stwu` immediate is one silent byte.
        if self.n_saved() > FRAME_MAX_SAVED_NO_SPILL {
            return Some("frame-allocator-spill");
        }
        None
    }

    /// The prologue: `mflr`, the LR + register saves, the probes, the `stwu`.
    /// Its byte length is the function's `$M(n)` label value and, divided by
    /// four, the `PrologLen` field of its `.pdata` record.
    pub fn prologue(&self) -> Result<Vec<u8>, BackendError> {
        if let Some(ctx) = self.out_of_class_ctx() {
            return Err(out_of_class(ctx));
        }
        let f = self.size();
        // A frame this emitter can build always fits the `stwu` immediate: the
        // stack-check threshold (5 pages) is well under 32768. Assert rather
        // than truncate, because a silent wrap is a valid `stwu` of the wrong
        // size — exactly the fuzzy-invisible corruption the boundary rule is
        // about.
        let neg = i32::try_from(f)
            .ok()
            .and_then(|v| i16::try_from(-v).ok())
            .ok_or_else(|| out_of_class("frame larger than a stwu immediate"))?;
        let mut w: Vec<u8> = Vec::with_capacity(4 * (3 + self.save_slots() as usize));
        w.extend_from_slice(&FRAME_MFLR_R12.to_be_bytes());
        w.extend_from_slice(&FRAME_LR_STORE.to_be_bytes());
        // GPRs ascending in slot address: r(32-n) lowest, r31 at -16.
        for i in (0..self.saved_gprs).rev() {
            w.extend_from_slice(&encode_std(31 - i, 1, self.gpr_slot(i)));
        }
        // Then the FPRs, again ascending in address — and BELOW the GPRs.
        for i in (0..self.saved_fprs).rev() {
            w.extend_from_slice(&encode_stfd(31 - i, 1, self.fpr_slot(i)));
        }
        for k in 1..=self.probe_pages() {
            let d = -((k * FRAME_PAGE) as i32) as i16;
            w.extend_from_slice(&encode_ld(12, 1, d));
        }
        w.extend_from_slice(&encode_stwu(1, 1, neg));
        Ok(w)
    }

    /// The epilogue: free the frame, restore LR, restore the saved registers in
    /// ascending slot address (so FPRs, which sit lower, come first), `blr`.
    pub fn epilogue(&self) -> Result<Vec<u8>, BackendError> {
        if let Some(ctx) = self.out_of_class_ctx() {
            return Err(out_of_class(ctx));
        }
        let f = self.size();
        let mut w: Vec<u8> = Vec::with_capacity(4 * (4 + self.save_slots() as usize));
        if let Ok(pos) = i16::try_from(f) {
            w.extend_from_slice(&encode_addi(1, 1, pos));
        } else {
            w.extend_from_slice(&FRAME_BACKCHAIN.to_be_bytes());
        }
        w.extend_from_slice(&FRAME_LR_LOAD.to_be_bytes());
        w.extend_from_slice(&FRAME_MTLR_R12.to_be_bytes());
        for i in (0..self.saved_fprs).rev() {
            w.extend_from_slice(&encode_lfd(31 - i, 1, self.fpr_slot(i)));
        }
        for i in (0..self.saved_gprs).rev() {
            w.extend_from_slice(&encode_ldr(31 - i, 1, self.gpr_slot(i)));
        }
        w.extend_from_slice(&encode_blr());
        Ok(w)
    }

    /// [`Self::epilogue`] **without its closing `blr`** — the straight-line run
    /// of the epilogue block, for a class laid out through
    /// [`super::block_ir::BodyLayout`].
    ///
    /// The `blr` is [`super::block_ir::Terminator::Blr`]'s word, so a body that
    /// kept it in the run would emit it twice, and a body that dropped it
    /// without checking would emit a return the frame never wrote. This checks
    /// rather than trims blind: the epilogue is built here, four lines up, and
    /// the assertion is cheap insurance against a later frame class whose last
    /// word is something else (`epilogue_gpr_helper`'s is an external `b`, and
    /// it is a different method for that reason).
    pub fn epilogue_run(&self) -> Result<Vec<u8>, BackendError> {
        let mut w = self.epilogue()?;
        let n = w.len();
        if n < 4 || w[n - 4..] != encode_blr()[..] {
            return Err(out_of_class(
                "a materialised epilogue whose last word is not `blr`: \
                 `Terminator::Blr` is that word, and trimming a word that is not \
                 it would emit a body short of its own return",
            ));
        }
        w.truncate(n - 4);
        Ok(w)
    }
}

/// **SURFACE[frame.out_of_class]** — the registered decision surface's domain
/// (`crate::surface`, board **#3743**, lane `w-doctrine`).
///
/// **Three predicates, not one**, because there are three prologue emitters and
/// each refuses the other two by name: [`FrameLayout::out_of_class_ctx`] (the
/// gate every shipped emitter runs),
/// [`FrameLayout::out_of_class_ctx_gpr_helper`] (Class C) and
/// [`FrameLayout::out_of_class_ctx_gpr_helper_leaf`] (frameless Class C). This
/// module's own header says why that separation is the safety argument for
/// W-XLR — *"the helper prologue is reachable only from an emitter that asked
/// for it by name"* — and a separation that nothing enumerates is a separation
/// that can be lost in a one-token edit.
///
/// The `saved_gprs` axis jumps to 17 and 18 for one reason: `n_saved() >
/// FRAME_MAX_SAVED_NO_SPILL` is a **second lock** behind the helper thresholds,
/// described in its own comment as *"unreachable behind the two helper
/// thresholds today"*. Unreachable-behind-another-guard is exactly the state in
/// which a widening is invisible, so the domain reaches it deliberately.
pub fn surface_rows() -> Vec<crate::surface::Row> {
    let mut rows = Vec::new();
    let emitters: [(&str, fn(&FrameLayout) -> Option<&'static str>); 3] = [
        ("base", FrameLayout::out_of_class_ctx),
        ("clsC", FrameLayout::out_of_class_ctx_gpr_helper),
        ("clsCleaf", FrameLayout::out_of_class_ctx_gpr_helper_leaf),
    ];
    for (ename, refuses) in emitters {
        for saved_gprs in [0u8, 1, 2, 3, 4, 17, 18] {
            for saved_fprs in 0..=5u8 {
                // `20_360` straddles the `_RtlCheckStack12` threshold *through*
                // [`FRAME_MIN_OUT_SLOTS`]: at the shipped floor of 8 the frame
                // sizes to 20,448 and is admitted, and at a floor of 16 it
                // sizes to 20,512 and is refused. That is deliberate. With the
                // domain's earlier `20_000` **the floor was not reachable at
                // all** — raising it 8 -> 16 moved not one line — so naming it a
                // guard was a false coverage claim, and this is the repair.
                // `out_slots = 20` sits above the floor so the cells that
                // depend on it and the cells that do not are both enumerated.
                for locals in [0u32, 20_360] {
                    for out_slots in [0u8, 20] {
                        let l = FrameLayout { locals, out_slots, saved_gprs, saved_fprs };
                        let outcome = match refuses(&l) {
                            Some(ctx) => format!("{} {ctx}", crate::surface::REFUSE),
                            None => format!("probes={}", l.probe_pages()),
                        };
                        rows.push(crate::surface::Row::new(
                            format!(
                                "emitter={ename} gprs={saved_gprs:02} fprs={saved_fprs} \
                                 locals={locals:05} out={out_slots}"
                            ),
                            outcome,
                        ));
                    }
                }
            }
        }
    }
    rows
}

#[cfg(test)]
mod tests {
    // The single `mod tests` this was split out of opened with
    // `use super::*;`; the glob keeps that reach.
    #[allow(unused_imports)]
    use super::*;
    #[allow(unused_imports)]
    use crate::codegen::*;
    #[allow(unused_imports)]
    use c2_il::{IlFunction, IlOp};
    #[allow(unused_imports)]
    use crate::codegen::testutil::*;
    /// **The frame-size formula, against every captured witness.**
    ///
    /// `size = align16(80 + locals + 8 + 8 × saved)`. Rows are
    /// `(locals, gprs, fprs) -> frame`, each read off a reference obj's `stwu`
    /// displacement (`docs/CODEGEN_PPC_MVP.md` §"The frame model" names the probe
    /// source for each). The saved-register column is what the roadmap had
    /// recorded as "96 B for one by-value temporary, 112 B for two" — the driver
    /// is the callee-saved register count, and a by-value temporary moves the
    /// *locals* column instead.
    #[test]
    fn frame_size_fits_every_captured_witness() {
        let rows: &[(u32, u8, u8, u32)] = &[
            // saved GPRs 0..7, no locals: g(a)+1 … g(a)+b+c+d+e+f+g+h
            (0, 0, 0, 96),
            (0, 1, 0, 96),
            (0, 2, 0, 112),
            (0, 3, 0, 112),
            (0, 4, 0, 128),
            (0, 5, 0, 128),
            (0, 6, 0, 144),
            (0, 7, 0, 144),
            // saved FPRs 1..5: float g(a)*b … the FPR file uses the same slots
            (0, 0, 1, 96),
            (0, 0, 2, 112),
            (0, 0, 3, 112),
            (0, 0, 4, 128),
            (0, 0, 5, 128),
            // mixed: GPRs above FPRs in one shared descending slot array. The
            // 8-byte locals are the int→double conversion spill at 80(r1).
            (8, 2, 1, 128),
            (8, 3, 2, 144),
            (8, 4, 3, 160),
            (8, 0, 1, 112),
            // locals only (`char buf[n]` / `int buf[n]` passed to the callee)
            (1, 0, 0, 96),
            (5, 0, 0, 96),
            (9, 0, 0, 112),
            (64, 0, 0, 160),
            (3600, 0, 0, 3696),
            (4080, 0, 0, 4176),
            (4096, 0, 0, 4192),
            (8096, 0, 0, 8192),
            (8097, 0, 0, 8192),
            (16384, 0, 0, 16480),
            (12000, 0, 0, 12096),
            (16000, 0, 0, 16096),
            (16296, 0, 0, 16384),
            (16312, 0, 0, 16400),
            (17000, 0, 0, 17088),
            (20000, 0, 0, 20096),
            (20376, 0, 0, 20464),
            (20392, 0, 0, 20480),
            (24000, 0, 0, 24096),
            (32000, 0, 0, 32096),
            (32664, 0, 0, 32752),
            (32680, 0, 0, 32768),
            (32696, 0, 0, 32784),
            (40000, 0, 0, 40096),
            (200000, 0, 0, 200096),
            (4008, 0, 0, 4096),
            (4009, 0, 0, 4112),
            // locals AND saved registers together (`char buf[30000]` + 2/3 live)
            (30000, 2, 0, 30112),
            (30000, 3, 0, 30112),
        ];
        for &(locals, saved_gprs, saved_fprs, want) in rows {
            let l = FrameLayout { locals, out_slots: 0, saved_gprs, saved_fprs };
            assert_eq!(l.size(), want, "frame for {l:?}");
        }
        // The `out_slots` term, from the independent 480-case refutation sweep
        // (`docs/CODEGEN_FRAMED_CALLS.md` §1.2). None of this rung's own probes
        // could see it — they all pass eight arguments or fewer, where the two
        // forms of the rule coincide.
        let wide: &[(u32, u8, u8, u8, u32)] = &[
            // `int g();` with `int b[20]`: 80 bytes of locals and NO outgoing
            // arguments still reserves the 8-slot parameter area. A "frame >= 96"
            // model predicts 112 and is refuted by 176.
            (80, 0, 0, 0, 176),
            // Two calls of different arity, either order: nOutSlots = 12,
            // nSaved = 2 -> align16(16 + 96 + 16 + 8) = 144.
            (0, 12, 2, 0, 144),
            // 9 outgoing slots: the parameter area ends at SP+88 and the locals
            // still start at SP+96, so the frame steps at 4L + 96 + 8 crossing 16.
            (4, 9, 0, 0, 112),
            (32, 9, 0, 0, 144),
        ];
        for &(locals, out_slots, saved_gprs, saved_fprs, want) in wide {
            let l = FrameLayout { locals, out_slots, saved_gprs, saved_fprs };
            assert_eq!(l.size(), want, "frame for {l:?}");
        }
        assert_eq!(FrameLayout { locals: 0, out_slots: 9, ..Default::default() }.locals_base(), 96);
        assert_eq!(FrameLayout::default().locals_base(), 80);
    }

    /// The measured thresholds. Each boundary is a *pair* of captures, because a
    /// threshold read off one side is a guess.
    #[test]
    fn frame_helper_and_probe_thresholds_are_where_the_captures_put_them() {
        let g = |n| FrameLayout { saved_gprs: n, ..Default::default() };
        let f = |n| FrameLayout { saved_fprs: n, ..Default::default() };
        // GPRs: 2 open-coded `std`s, 3 is `__savegprlr_29`.
        assert!(!g(2).needs_gpr_helper());
        assert!(g(3).needs_gpr_helper());
        // FPRs: 3 open-coded `stfd`s, 4 is `__savefpr_28` — a DIFFERENT threshold.
        assert!(!f(3).needs_fpr_helper());
        assert!(f(4).needs_fpr_helper());
        // Stack probing: F = 20464 is four inline `ld`s, F = 20480 = 5 pages is
        // `_RtlCheckStack12`.
        let l = |locals| FrameLayout { locals, ..Default::default() };
        assert_eq!(l(20376).size(), 20464);
        assert!(!l(20376).needs_stack_check());
        assert_eq!(l(20376).probe_pages(), 4);
        assert_eq!(l(20392).size(), 20480);
        assert!(l(20392).needs_stack_check());
        // A frame that lands exactly on a page boundary crosses one boundary
        // fewer than a frame one word past it: F = 4096 probes nothing.
        assert_eq!(l(4008).probe_pages(), 0);
        assert_eq!(l(4009).probe_pages(), 1);
        assert_eq!(l(0).probe_pages(), 0);
        // Every helper shape refuses by name rather than emitting a prologue with
        // an unrelocated call in it.
        assert_eq!(g(3).out_of_class_ctx(), Some("frame-savegprlr-helper"));
        // **W-JSON — the frameless Class C pair is a THIRD predicate**, and the
        // three refuse each other by name. A layout with four saved GPRs and no
        // locals, no outgoing slots and no FPRs is the only one it admits.
        let leaf = FrameLayout { locals: 0, out_slots: 0, saved_gprs: 4, saved_fprs: 0 };
        assert_eq!(leaf.out_of_class_ctx_gpr_helper_leaf(), None);
        assert_eq!(leaf.out_of_class_ctx(), Some("frame-savegprlr-helper"));
        assert_eq!(
            FrameLayout { locals: 4, ..leaf }.out_of_class_ctx_gpr_helper_leaf(),
            Some("frame-gpr-helper-leaf-with-a-local")
        );
        assert_eq!(
            FrameLayout { out_slots: 3, ..leaf }.out_of_class_ctx_gpr_helper_leaf(),
            Some("frame-gpr-helper-leaf-with-outgoing-slots")
        );
        assert_eq!(
            FrameLayout { saved_fprs: 1, ..leaf }.out_of_class_ctx_gpr_helper_leaf(),
            Some("frame-gpr-helper-leaf-with-saved-fprs")
        );
        assert_eq!(
            g(2).out_of_class_ctx_gpr_helper_leaf(),
            Some("frame-gpr-helper-class-without-a-helper")
        );
        // The frameless pair is TWO words and ONE, against Class C's three and
        // two, and both are pinned to `work/w-json/probe/ref.obj`.
        assert_eq!(
            leaf.prologue_gpr_helper_leaf(0).unwrap(),
            vec![0x7d, 0x88, 0x02, 0xa6, 0x4b, 0xff, 0xff, 0xfd]
        );
        assert_eq!(leaf.epilogue_gpr_helper_leaf(0x12c).unwrap(), vec![0x4b, 0xff, 0xfe, 0xd4]);
        // …and the words a Class C layout would have emitted are NOT these: the
        // `stwu` and the `addi r1,r1,F` are exactly what this shape drops.
        assert_eq!(leaf.prologue_gpr_helper(0).unwrap().len(), 12);
        assert_eq!(leaf.epilogue_gpr_helper(0).unwrap().len(), 8);
        assert_eq!(f(4).out_of_class_ctx(), Some("frame-savefpr-helper"));
        assert_eq!(l(20392).out_of_class_ctx(), Some("frame-rtlcheckstack12"));
        assert!(g(3).prologue().is_err() && f(4).epilogue().is_err());
        // `stwux r1,r1,r12` is the allocation `_RtlCheckStack12` pairs with; the
        // word is captured, the shape is refused. Pinned so the constant cannot
        // rot while it is unreachable.
        assert_eq!(FRAME_STWUX.to_be_bytes(), [0x7C, 0x21, 0x61, 0x6E]);
    }

    /// The prologue and epilogue of every layout the emitter will build, word for
    /// word against the reference objs.
    #[test]
    fn frame_prologue_and_epilogue_match_the_reference_words() {
        let w = |v: &[u8]| -> Vec<u32> {
            v.chunks(4).map(|c| u32::from_be_bytes([c[0], c[1], c[2], c[3]])).collect()
        };
        // `int f(int a,int b){ return g(a) + b; }` — one saved GPR, frame 96.
        let one = FrameLayout { saved_gprs: 1, ..Default::default() };
        assert_eq!(one.size(), 96);
        assert_eq!(
            w(&one.prologue().unwrap()),
            vec![0x7D8802A6, 0x9181FFF8, 0xFBE1FFF0, 0x9421FFA0]
        );
        assert_eq!(
            w(&one.epilogue().unwrap()),
            vec![0x38210060, 0x8181FFF8, 0x7D8803A6, 0xEBE1FFF0, 0x4E800020]
        );
        // Two saved GPRs, frame 112: saved ascending in slot address, restored the
        // same way, and the restores come AFTER the `mtlr`.
        let two = FrameLayout { saved_gprs: 2, ..Default::default() };
        assert_eq!(
            w(&two.prologue().unwrap()),
            vec![0x7D8802A6, 0x9181FFF8, 0xFBC1FFE8, 0xFBE1FFF0, 0x9421FF90]
        );
        assert_eq!(
            w(&two.epilogue().unwrap()),
            vec![0x38210070, 0x8181FFF8, 0x7D8803A6, 0xEBC1FFE8, 0xEBE1FFF0, 0x4E800020]
        );
        // `float f(float a,float b,float c){ return g(a)*b*c; }` — two FPRs.
        let f2 = FrameLayout { saved_fprs: 2, ..Default::default() };
        assert_eq!(
            w(&f2.prologue().unwrap()),
            vec![0x7D8802A6, 0x9181FFF8, 0xDBC1FFE8, 0xDBE1FFF0, 0x9421FF90]
        );
        assert_eq!(
            w(&f2.epilogue().unwrap()),
            vec![0x38210070, 0x8181FFF8, 0x7D8803A6, 0xCBC1FFE8, 0xCBE1FFF0, 0x4E800020]
        );
        // Two GPRs and one FPR: the GPRs take the two slots under LR and the FPR
        // the one below them, but the PROLOGUE stores GPRs first (descending in
        // address after the run) while the EPILOGUE restores in ascending address
        // — so the two lists are not mirror images. Reference: `float f(int a,int
        // b,float x,float y){ return g(x)*y + (float)(a+b); }`, frame 128.
        let mix = FrameLayout { locals: 8, out_slots: 0, saved_gprs: 2, saved_fprs: 1 };
        assert_eq!(mix.size(), 128);
        assert_eq!(
            w(&mix.prologue().unwrap()),
            vec![0x7D8802A6, 0x9181FFF8, 0xFBC1FFE8, 0xFBE1FFF0, 0xDBE1FFE0, 0x9421FF80]
        );
        assert_eq!(
            w(&mix.epilogue().unwrap()),
            vec![0x38210080, 0x8181FFF8, 0x7D8803A6, 0xCBE1FFE0, 0xEBC1FFE8, 0xEBE1FFF0, 0x4E800020]
        );
        // Locals with page probes: `int f(int a){ char buf[4009]; … }`, frame
        // 4112, one probe. And `int buf[4096]`, frame 16480, four probes.
        let p1 = FrameLayout { locals: 4009, ..Default::default() };
        assert_eq!(
            w(&p1.prologue().unwrap()),
            vec![0x7D8802A6, 0x9181FFF8, 0xE981F000, 0x9421EFF0]
        );
        let p4 = FrameLayout { locals: 16384, ..Default::default() };
        assert_eq!(
            w(&p4.prologue().unwrap()),
            vec![0x7D8802A6, 0x9181FFF8, 0xE981F000, 0xE981E000, 0xE981D000, 0xE981C000, 0x9421BFA0]
        );
        // The `.pdata` PrologLen is the prologue's word count, which is now a
        // function of the layout rather than the hardcoded 3.
        assert_eq!(FrameLayout::default().prologue().unwrap().len() / 4, 3);
        assert_eq!(one.prologue().unwrap().len() / 4, 4);
        assert_eq!(two.prologue().unwrap().len() / 4, 5);
        assert_eq!(p4.prologue().unwrap().len() / 4, 7);
    }

}
