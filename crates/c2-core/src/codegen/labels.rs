//! The label→offset map — `docs/CFG_SHAPE.md` §6.2 item **B**, built.
//!
//! > **B. Labels as first-class, resolved by a fixup pass.** `3A`/`38`/`39`
//! > carry no direction (§2.1), so the target's offset is unknown when the
//! > branch is emitted. The IR needs a label identity, a map from label to
//! > block, and a **fixup list** of (word offset, label, form). Even §4's
//! > single-branch minimal instance needs this — it is not an optimization for
//! > the many-block case.
//!
//! # Why this exists when `calls.rs` already resolved a branch
//!
//! Board row **Z-c** (W11) reads *"the port emits an intra-section `b` and
//! resolves a real label→offset map"*. What W11 actually built is a fixup list
//! with **one implicit target**: `early_fixups`, every entry resolved against
//! `epi_start`, with the epilogue's identity carried by the *shape of the tuple*
//! rather than by any label. `calls.rs` said so itself four lines above the
//! block that resolved it — *"there is no fixup list and no label map"*. That
//! lowering is byte-exact at 12 lanes and this module does not change one byte
//! of it; what it changes is that the target is now **named**, so a second
//! target can exist.
//!
//! # The two rules this map enforces, and where each was measured
//!
//! **1. Two encodings, chosen by target kind — never one "patch the branch"
//! path.** `bc` and the intra-section `b` carry a **true self-relative
//! displacement** and take **no relocation**; an external `b`/`bl` carries
//! `−(own offset)` and takes a `REL24` (`CFG_SHAPE.md` §3.3, board **#191**).
//! `48000008` and `4bffffec` are the same instruction. **This map holds only the
//! first kind.** An external branch is not a label reference — it is a
//! relocation — and admitting it here is exactly the corruption §3.3 warns
//! about, so [`Form`] has no variant for it and [`encode_tail_branch`] stays
//! where it is.
//!
//! [`encode_tail_branch`]: super::encode::encode_tail_branch
//!
//! **2. Every reference must be FORWARD — a backward reference is refused, and
//! the refusal is a `coff/` fact, not a `codegen/` preference.**
//!
//! This is the rule lane w-label measured before writing the code
//! (`work/w-label/PREREG.md` §1; `work/w-label/cflabels.py`, 24 seed-free in-TU
//! cells at `/O1 /GS- /c`, anchor controls held on every row):
//!
//! ```text
//!   every body with a BACKWARD intra-section branch charges the
//!   compiler-label counter >= +1                                11 of 11
//!   no body without one charges more than +1                    13 of 13
//! ```
//!
//! `coff::plan_labels` charges a framed function `label_lead + 5` and
//! `IlFunction::label_lead` returns non-zero only for the signed two-call
//! comparator and `eh_bare`. So a lowering that emitted a backward branch would
//! be **one label slot low for this function and every later one in the TU** —
//! six wrong bytes in a symbol table, in an obj that still links, which is the
//! defect class `docs/LABEL_COUNTER.md` exists for and which `coff/` has shipped
//! twice.
//!
//! The magnitude on the far side is **measured and not modelled**: the same 24
//! cells read +1 (`do/while`, an explicit backward `goto`, the exit-value
//! merge), +2 (`while`, `for`), +3 (`for(;;)`+`break`, two sequential
//! `do/while`s) and +4 (two `for`s, nested `for`) — four distinct magnitudes
//! over eleven cells with no rule that survives all of them. Two candidate rules
//! *were* fitted to that table and **both are refuted by it**: "one slot per
//! interior branch target" misses 15 of 24 rows and "one per interior join"
//! misses 6, one of them across the zero boundary. Interpolating any of that
//! would be `CFG_SHAPE.md` §3.5's declined fold model a second time, so the
//! refusal ships and the measurement ships with it.
//!
//! **What the rule does NOT say.** Forward-only is *necessary*, not
//! *sufficient*: two forward-only cells (`cf-ifelse`, `cf-merge-tail`) charge +1
//! anyway, and both are §3.4.1's code-motion shapes — a block c2 created by
//! tail-merging two paths. The port refuses those for an unrelated reason (a
//! body whose arms end in the same call is out of class), so the two refusals
//! are independent and **closing this one does not close that one**. Named here
//! because a lane that closed the backward case alone would still emit a wrong
//! `$M` on the other.

use super::select::out_of_class;
use super::{encode_b_intra, encode_bc};
use crate::BackendError;

/// A label identity, minted by [`LabelMap::mint`].
///
/// Opaque and `Copy`. It carries an index into the map that minted it, so a
/// label from one map used against another is caught by
/// [`LabelMap::resolve`]'s bounds check rather than silently reading a
/// neighbour's offset.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Label(usize);

/// Which of the two intra-section branch encodings a reference site wants.
///
/// There is deliberately **no external/relocated variant** — see this module's
/// header. The discriminator is the target, not the opcode, and the two live in
/// different places for that reason.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Form {
    /// A conditional branch. `BD` is a signed 14-bit field scaled by 4, so it
    /// reaches ±32764.
    Bc { bo: u8, bi: u8 },
    /// An unconditional intra-section `b`. `LI` is a signed 24-bit field scaled
    /// by 4.
    B,
}

impl Form {
    fn encode(self, disp: i32) -> Option<[u8; 4]> {
        match self {
            Form::Bc { bo, bi } => encode_bc(bo, bi, disp),
            Form::B => encode_b_intra(disp),
        }
    }

    fn name(self) -> &'static str {
        match self {
            Form::Bc { .. } => "bc",
            Form::B => "b",
        }
    }
}

/// One pending reference: the `.text` offset of the placeholder word, the label
/// it names, and which encoding to patch it with.
struct Ref {
    at: usize,
    label: Label,
    form: Form,
}

/// The label→offset map for **one function body**.
///
/// A body's `text` is built in order; a branch whose target is not yet emitted
/// calls [`LabelMap::reference`], which appends a **zero placeholder word** and
/// records the site. [`LabelMap::define`] binds a label to the offset the body
/// has reached. [`LabelMap::resolve`] patches every site once, at the end, when
/// every offset is known.
///
/// It is scoped to one body on purpose. A label offset is a `.text`-section
/// offset and the port emits one COMDAT per function, so a map that outlived a
/// body would be holding offsets in two coordinate systems — which is the shape
/// of the `.pdata` mistake `docs/OBJ_GY_SHAPES.md` §3.3 records.
#[derive(Default)]
pub struct LabelMap {
    /// `defined[i]` is the offset bound to label `i`, or `None` while it is
    /// still forward.
    defined: Vec<Option<usize>>,
    /// A human name per label, used only in the error text. A refusal that
    /// cannot say *which* label it is about is a refusal somebody will have to
    /// re-derive.
    names: Vec<&'static str>,
    refs: Vec<Ref>,
}

impl LabelMap {
    pub fn new() -> Self {
        Self::default()
    }

    /// Mint a fresh, undefined label. `name` appears in refusal text only.
    pub fn mint(&mut self, name: &'static str) -> Label {
        self.defined.push(None);
        self.names.push(name);
        Label(self.defined.len() - 1)
    }

    /// Bind `label` to the current end of `text`.
    ///
    /// Refuses a **second** definition rather than overwriting: two blocks
    /// claiming one label is a lowering bug, and silently keeping the last one
    /// would emit a legal-looking branch to the wrong place — the same failure
    /// mode as a truncated `BD`, which the encoders already refuse rather than
    /// round.
    pub fn define(&mut self, label: Label, text: &[u8]) -> Result<(), BackendError> {
        let at = text.len();
        let slot = self
            .defined
            .get_mut(label.0)
            .ok_or_else(|| out_of_class("a label from a different function's map"))?;
        if slot.is_some() {
            return Err(out_of_class(
                "a label defined twice in one body: two blocks claiming one \
                 target is a lowering defect, not a layout",
            ));
        }
        *slot = Some(at);
        Ok(())
    }

    /// Append a placeholder word for a branch to `label` and record the fixup.
    ///
    /// The placeholder is written **here** rather than by the caller, so the
    /// "the site is still zero when we patch it" invariant in [`Self::resolve`]
    /// is a real check on the caller and not a restatement of what the caller
    /// just did.
    pub fn reference(&mut self, text: &mut Vec<u8>, label: Label, form: Form) {
        let at = text.len();
        text.extend_from_slice(&[0; 4]);
        self.refs.push(Ref { at, label, form });
    }

    /// How many references are still outstanding. Used by the callers' own
    /// assertions and by the tests; a body that finishes with a non-empty map
    /// and never resolves it is the bug [`Self::resolve`] cannot catch, because
    /// it was never called.
    pub fn pending(&self) -> usize {
        self.refs.len()
    }

    /// Patch every recorded site, then consume the map.
    ///
    /// Five invariants, each an ordinary `Err` rather than a panic — the port
    /// must degrade to `NotImplemented` honestly, and a `debug_assert` is
    /// compiled out of the release build the gate actually runs.
    ///
    /// 1. **Every referenced label is defined.** An undefined one names itself.
    /// 2. **The site is in range** of the finished text.
    /// 3. **The site still holds the zero placeholder.** A caller that wrote
    ///    over its own fixup site would otherwise get a branch patched on top of
    ///    an instruction.
    /// 4. **The reference is FORWARD** — see the module header. This is the
    ///    `coff/` rule, and it is the reason this method exists as a gate rather
    ///    than as a loop.
    /// 5. **The displacement fits the form's field.** `CFG_SHAPE.md` §3.3.1's
    ///    long-branch expansion (invert the condition, branch over an
    ///    unconditional `b`) is measured and **not built** — no fixture body is
    ///    32 KB, so it has no bytes for the oracle to compare and building it
    ///    would be an ungraded code path by construction (w-frame row **F-c**).
    pub fn resolve(self, text: &mut [u8]) -> Result<(), BackendError> {
        for r in &self.refs {
            let target = match self.defined.get(r.label.0).copied().flatten() {
                Some(t) => t,
                None => {
                    return Err(out_of_class(&format!(
                        "a branch to label `{}`, which no block defined",
                        self.names.get(r.label.0).copied().unwrap_or("?")
                    )))
                }
            };
            if r.at + 4 > text.len() || target > text.len() {
                return Err(out_of_class(
                    "a label fixup outside the body it belongs to",
                ));
            }
            if text[r.at..r.at + 4] != [0; 4] {
                return Err(out_of_class(
                    "a label fixup site that is no longer the zero placeholder: \
                     something was emitted over a pending branch",
                ));
            }
            if target <= r.at {
                // §1.4 of `work/w-label/PREREG.md`: >= +1 on the compiler-label
                // counter in 11 of 11 measured cells, and `coff::plan_labels`
                // charges 0. Emitting this would be a wrong `$M` for this
                // function and every later one in the TU.
                return Err(out_of_class(&format!(
                    "a BACKWARD branch to label `{}`: c2 charges the \
                     compiler-label counter at least one extra slot for every \
                     body with a backward intra-section branch (11 of 11 cells, \
                     work/w-label/PREREG.md §1.4) and `coff::plan_labels` \
                     charges none, so the obj would carry a wrong $M as well as \
                     a wrong block. The magnitude is measured (+1/+2/+3/+4) and \
                     NOT modelled",
                    self.names.get(r.label.0).copied().unwrap_or("?")
                )));
            }
            let disp = (target - r.at) as i32;
            let word = r.form.encode(disp).ok_or_else(|| {
                out_of_class(&format!(
                    "a `{}` past its displacement field: the long-branch \
                     expansion is measured in docs/CFG_SHAPE.md §3.3.1 and not \
                     built",
                    r.form.name()
                ))
            })?;
            text[r.at..r.at + 4].copy_from_slice(&word);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The map's own happy path: two references to one label, resolved after
    /// the label is defined, both carrying their true self-relative
    /// displacement and neither taking a relocation.
    #[test]
    fn two_references_to_one_forward_label_resolve_to_their_own_displacements() {
        let mut m = LabelMap::new();
        let epi = m.mint("epilogue");
        let mut t = Vec::new();
        t.extend_from_slice(&[0x38, 0x60, 0x00, 0x05]); // li r3,5
        m.reference(&mut t, epi, Form::B); // at 4
        t.extend_from_slice(&[0x38, 0x60, 0x00, 0x0b]); // li r3,11
        m.reference(&mut t, epi, Form::B); // at 12
        t.extend_from_slice(&[0x38, 0x60, 0x00, 0x00]); // li r3,0
        assert_eq!(m.pending(), 2);
        m.define(epi, &t).unwrap();
        m.resolve(&mut t).unwrap();
        // 4 -> 20 is +16; 12 -> 20 is +8.
        assert_eq!(&t[4..8], &[0x48, 0x00, 0x00, 0x10]);
        assert_eq!(&t[12..16], &[0x48, 0x00, 0x00, 0x08]);
    }

    #[test]
    fn a_conditional_reference_patches_the_bc_form() {
        let mut m = LabelMap::new();
        let join = m.mint("join");
        let mut t = Vec::new();
        m.reference(&mut t, join, Form::Bc { bo: 12, bi: 26 }); // at 0
        t.extend_from_slice(&[0x38, 0x60, 0x00, 0x05]);
        m.define(join, &t).unwrap();
        m.resolve(&mut t).unwrap();
        assert_eq!(&t[0..4], &[0x41, 0x9a, 0x00, 0x08]);
    }

    /// **The ordering rule this module exists for.** A backward reference is
    /// refused, and the refusal names the label and the counter.
    #[test]
    fn a_backward_reference_is_refused_because_it_moves_the_label_counter() {
        let mut m = LabelMap::new();
        let top = m.mint("loop-top");
        let mut t = Vec::new();
        m.define(top, &t).unwrap(); // at 0
        t.extend_from_slice(&[0x38, 0x60, 0x00, 0x05]);
        m.reference(&mut t, top, Form::Bc { bo: 4, bi: 26 }); // at 4, target 0
        let err = m.resolve(&mut t).unwrap_err();
        let s = format!("{err:?}");
        assert!(s.contains("BACKWARD"), "{s}");
        assert!(s.contains("loop-top"), "{s}");
        assert!(s.contains("plan_labels"), "{s}");
    }

    /// A self-reference is backward by the same rule (`target <= at`), which is
    /// the boundary case the `<=` rather than `<` is there for.
    #[test]
    fn a_self_reference_is_refused_by_the_same_rule() {
        let mut m = LabelMap::new();
        let l = m.mint("self");
        let mut t = Vec::new();
        m.define(l, &t).unwrap();
        m.reference(&mut t, l, Form::B);
        // The reference sits at 0 and the label is bound to 0.
        let err = m.resolve(&mut t).unwrap_err();
        assert!(format!("{err:?}").contains("BACKWARD"));
    }

    #[test]
    fn an_undefined_label_is_refused_and_names_itself() {
        let mut m = LabelMap::new();
        let l = m.mint("never-defined");
        let mut t = Vec::new();
        m.reference(&mut t, l, Form::B);
        let err = m.resolve(&mut t).unwrap_err();
        assert!(format!("{err:?}").contains("never-defined"));
    }

    #[test]
    fn defining_a_label_twice_is_refused() {
        let mut m = LabelMap::new();
        let l = m.mint("join");
        let mut t = Vec::new();
        m.define(l, &t).unwrap();
        t.extend_from_slice(&[0; 4]);
        let err = m.define(l, &t).unwrap_err();
        assert!(format!("{err:?}").contains("defined twice"));
    }

    /// A caller that overwrote its own pending site gets a refusal rather than a
    /// branch patched on top of an instruction.
    #[test]
    fn a_fixup_site_that_was_written_over_is_refused() {
        let mut m = LabelMap::new();
        let l = m.mint("epilogue");
        let mut t = Vec::new();
        m.reference(&mut t, l, Form::B);
        t[0..4].copy_from_slice(&[0x38, 0x60, 0x00, 0x05]); // the caller's bug
        t.extend_from_slice(&[0; 4]);
        m.define(l, &t).unwrap();
        let err = m.resolve(&mut t).unwrap_err();
        assert!(format!("{err:?}").contains("zero placeholder"));
    }

    /// The displacement-range refusal, per form. `CFG_SHAPE.md` §3.3.1 brackets
    /// the real transition between +32628 (direct) and +34148 (expanded); the
    /// encoders already refuse past the architectural limit and this checks the
    /// map propagates that refusal rather than truncating.
    #[test]
    fn a_bc_past_its_field_is_refused_with_the_expansion_named() {
        let mut m = LabelMap::new();
        let l = m.mint("far");
        let mut t = Vec::new();
        m.reference(&mut t, l, Form::Bc { bo: 12, bi: 26 });
        t.resize(40_000, 0x60); // well past the 14-bit BD field
        m.define(l, &t).unwrap();
        let err = m.resolve(&mut t).unwrap_err();
        let s = format!("{err:?}");
        assert!(s.contains("displacement field"), "{s}");
        assert!(s.contains("3.3.1"), "{s}");
    }

    /// …and the same body is *fine* for the wider `LI` field, which is what
    /// makes the previous test a statement about the field rather than about
    /// the length.
    #[test]
    fn the_same_distance_is_in_range_for_the_wider_b_field() {
        let mut m = LabelMap::new();
        let l = m.mint("far");
        let mut t = Vec::new();
        m.reference(&mut t, l, Form::B);
        t.resize(40_000, 0x60);
        m.define(l, &t).unwrap();
        m.resolve(&mut t).unwrap();
        assert_eq!(&t[0..4], &(0x4800_0000u32 | 40_000).to_be_bytes());
    }

    /// A label that is minted and defined but never referenced is not an error:
    /// the `/Ox` arm of an early return duplicates the epilogue instead of
    /// branching to it, so a body legitimately finishes with labels nobody
    /// named.
    #[test]
    fn an_unreferenced_label_is_not_an_error() {
        let mut m = LabelMap::new();
        let l = m.mint("epilogue");
        let mut t = vec![0x38, 0x60, 0x00, 0x05];
        m.define(l, &t).unwrap();
        assert_eq!(m.pending(), 0);
        m.resolve(&mut t).unwrap();
        assert_eq!(&t[..], &[0x38, 0x60, 0x00, 0x05]);
    }

    /// A label minted by one map and used against another is caught rather than
    /// silently reading the neighbouring label's offset.
    #[test]
    fn a_label_from_another_map_is_refused() {
        let mut a = LabelMap::new();
        let _ = a.mint("a0");
        let stray = a.mint("a1");
        let mut b = LabelMap::new();
        let mut t = Vec::new();
        let err = b.define(stray, &t).unwrap_err();
        assert!(format!("{err:?}").contains("different function's map"));
        // …and on the reference side, where the index is out of range.
        let mut b2 = LabelMap::new();
        b2.reference(&mut t, stray, Form::B);
        assert!(b2.resolve(&mut t).is_err());
    }
}
