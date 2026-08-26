//! **DECODE, separated from ADMISSION** — lane `w-unfuse`, board **#3554**.
//!
//! # The fusion this module ends
//!
//! `ARCHITECTURE_PROPOSAL_2026-08-20.md` §8 row 4a(i), which
//! `docs/DECISIONS_2026-08-22.md` decision 13 funds, names the obstacle:
//!
//! > *"IR0 stops at a two-variant byte framing and `BodyShape` starts at 35
//! > whole-function grammars **that are simultaneously the admission gate**, so
//! > the semantic middle a COLOR pass would consume does not exist."*
//!
//! Before this module there was one call — `parse_segment`, returning
//! `Option<BodyShape>` — and its `None` meant two different things at once:
//!
//! * *"this port cannot read this IL"* (a **decode** fact about the bytes), and
//! * *"this port may not emit this body"* (an **admission** fact about the port).
//!
//! Because they were the same value, **widening what the port could read was
//! the same edit as widening what it would emit**. That is why every reach
//! widening in this project's history has also been an emission widening, and
//! lane `S0` measured what naive widening ships: **blind-differs 96.1 %** of
//! what it reached. Under `docs/PROGRESS_METRIC.md` a wrong emit scores
//! strictly below the refusal it replaced, so the two questions have to be
//! separately answerable before either can move.
//!
//! # The split
//!
//! [`Decoded`] answers **"what does this IL say"**. It is a total function of
//! `(segment, .sy view)` — it is constructed for every segment, it never
//! refuses on admission grounds, and it carries no admission verdict.
//!
//! [`AdmissionPolicy`] answers **"may the port emit this"**. It is a predicate
//! **over** a [`Decoded`], never a second parse, and it is the only thing
//! entitled to that answer.
//!
//! Today [`AdmissionPolicy::RecognizedShape`] — the default — admits exactly
//! when the decode reached one of the ~35 recognized whole-function grammars,
//! which is byte-for-byte the rule `parse_segment` implemented. **The admitted
//! set is unchanged by construction; what changed is that it is now a separate,
//! nameable decision.**
//!
//! # Why the raw parse is no longer reachable from outside `body`
//!
//! [`super::parse_segment_detail`] is `pub(in crate::func::body)`. Every
//! consumer outside this module tree — `func::bundle`, `func::census`,
//! `func::diag` — must go through [`Decoded`], so a future call site cannot
//! re-fuse the two questions by reaching past the seam. That visibility is the
//! mechanism; the doc comment is not.
//!
//! **This is the whole reason `census/gate disagreement` cannot drift.** It
//! used to hold by a convention stated in a comment (*"`parse_segment` is
//! `.ok()` of this"*). It now holds because there is one decode, one admission
//! predicate, and no second route to either.
//!
//! # This GENERALIZES an existing precedent; it does not invent one
//!
//! [`super::shapes::control_flow`] has been a decode-only layer since it
//! landed — its own first line is *"The control-flow statement layer — DECODE
//! ONLY"*, and *"Nothing in this file can make a function in class"*. It walks
//! the statement token stream, reports the CFG shape or the byte that stopped
//! it, and gates nothing; `census.rs` reaches it on every body through
//! `scan_full`. Lane `w-ilarms` measured the consequence — that walk reads
//! operand widths for 68 of the 95 opcodes the real dispatch handles with **no
//! admission gate in the path**.
//!
//! So the fusion decision 13 names is real but **local**: it is
//! [`super::parse_segment_shape`]'s ladder and the [`BodyShape`] it builds, not
//! the whole crate. This module applies `control_flow`'s existing shape at the
//! one layer that *is* the admission gate. What it does not do is claim that
//! layer's reach: **a width reader is not a decoder** — `w-ilarms` also found
//! that no site in any of the five crates mints an IR node for the `≥ 0x2af`
//! node opcodes — so "68 of 95" is a statement about cursor advance and must
//! not be read as decode coverage. The reach measurement is
//! `w-decodereach`'s, off [`crate::IlBundle::decode_bodies`].
//!
//! # What this module deliberately does NOT do
//!
//! It does not decode anything the port could not already decode. Row 4a(i)'s
//! general op-level decode is priced at I1 raw 1.5–4.5 engineer-months and
//! **15–45 engineer-months as a lower bound** for 4a as a whole
//! (`docs/STEP5_PRICING_2026-08-21.md` §2/§4); this module is its
//! **prerequisite**, not a slice of it. A later slice widens what [`Decoded`]
//! can say without touching [`AdmissionPolicy`], and the admitted set provably
//! cannot move, because admission reads exactly one field.

use super::{parse_segment_detail, BodyShape, Block};
use crate::func::sy::SyView;

/// **ADMISSION — may the port emit this body?** The decision surface, named,
/// per `docs/rungs/README.md` § Lane kinds, THE DECISION-SURFACE CLAUSE: a
/// general layer ships its arbitrary choices as named, enumerable parameters
/// whose **default reproduces c2 byte-exactly**, not as baked constants.
///
/// The arbitrary choice here is the one the fused parser made implicitly and
/// unnameably: *admission is exactly "the decode reached a recognized
/// whole-body grammar"*. That is now [`Self::RecognizedShape`], it is
/// [`Self::DEFAULT`], and every production call site passes it.
///
/// # ⚠ `#3556` SAID THIS ENUM COULD NEVER HAVE A SECOND VALUE. IT IS REFUTED,
/// AND BY THE TREE THAT CARRIED THE CLAIM
///
/// Lane `w-unfuse` built a second value — *admit nothing* — could not ship it,
/// and concluded that **the admission layer cannot own a refusal REASON**: a
/// [`Block`] says where the READ stopped, a policy refusing a whole-read body
/// has no such point, and minting a `Block` for it publishes a `:eof` census
/// key *"no scan can ever reach"*. Its stated consequence was that **every
/// refusal KEY in this port belongs to the decode**.
///
/// **That consequence is false, and was false when it was written.** Lane
/// `w-atend` (board **#3591**) read the two facts that settle it:
///
/// * [`Block::at_end`] exists for exactly this shape and had **7** production
///   sites, every one of them a refusal raised *after* the parse reached
///   `seg.len()`. Two of their constants document themselves as admission
///   predicates in as many words: [`super::OPT_MODE`] — *"the body parses in
///   class but the optimization-settings word is not one this port emits
///   under"* — and [`super::CALLEE_DEFINED_IN_TU`] — *"the body parses in
///   class, its callee resolves, and the TU defines that callee, so c2 may
///   inline it and **the port may not emit**"*. Neither says where a read
///   stopped.
/// * A scan reaches them, and a peer had already measured how wide: board
///   **#3582** reports `decode-reach-grammar-not-admitted` = **4,001** bodies
///   over the 878-TU workload, under five `:eof` keys the scan publishes and
///   ranks.
///
/// # What DOES survive is about the CALL SITES, not about the layer
///
/// [`Self::Nothing`]'s key is unreachable in production because **no production
/// call site passes a non-default policy** — not because admission cannot name
/// a reason. The precedent for closing that gap is already in this crate:
/// [`crate::Relax`] is a named, settable census parameter whose non-default
/// level *is* selected by production code and whose numbers *are* published
/// (`fnbyte-blind-level|<name>`). A future instrument that sweeps
/// [`Self::ALL`] and publishes per-policy counts makes this key as reachable as
/// those. Until one does, the key is an **instrument state** in the sense of
/// `docs/rungs/README.md` § Lane kinds, THE DECISION-SURFACE CLAUSE — legal,
/// licensing no emit, graded at [`Self::DEFAULT`] and nowhere else — and
/// `crates/c2-harness/tests/fence_site_census.rs` counts it in its own
/// population rather than absorbing it into the production one.
///
/// # The shape of an admission verdict, restated correctly
///
/// **The admission layer owns a reason for exactly the bodies it alone
/// refuses.** [`Decoded::into_admit`] returns the DECODE's own [`Block`] when
/// the decode stopped — that reason was never admission's — and mints
/// [`super::ADMISSION_DECLINED`] only on the `Ok` side, where the policy is the
/// only thing that refused. `#3556`'s API reading survives unchanged:
/// [`Decoded::is_admitted_under`] is still a `bool`, because a *verdict* is a
/// yes/no even when a *reason* exists beside it.
///
/// The `match` in [`Decoded::into_admit`] is **exhaustive over this enum**, so
/// a third variant cannot be added without that site being made to handle it,
/// and [`Self::index`] is exhaustive too, so it cannot be added without being
/// given a place in [`Self::ALL`]. That is the mechanism; this paragraph is
/// not.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AdmissionPolicy {
    /// **The DEFAULT, and today's rule byte-for-byte.** A body is admitted
    /// exactly when the decode reached one of the recognized whole-function
    /// grammars ([`BodyShape`]). `parse_segment` was `.ok()` of the decode, and
    /// this variant is that `.ok()` lifted out of the parser and given a name.
    RecognizedShape,
    /// **Admit nothing** — an instrument state in the **refusal** direction,
    /// which `docs/PROGRESS_METRIC.md` orders strictly above a wrong emit.
    ///
    /// It is the ablation floor of the decision surface: the answer to *"what
    /// does this port look like with the admitted set emptied"*, without
    /// touching the decode. **No production call site selects it**, and a lane
    /// that makes one do so is widening the decision surface into production
    /// and owes that its own two-sided price.
    ///
    /// Its refusal reason is [`super::ADMISSION_DECLINED`], and it is raised
    /// **only** for bodies the decode read whole — see [`Decoded::into_admit`].
    Nothing,
}

impl AdmissionPolicy {
    /// The policy every production call site uses. Reproduces the pre-split
    /// admitted set exactly.
    /// PROV[N] not load-bearing — the port's own default admission POLICY, a measurement profile in DISCLOSURE's sense. It selects which of this crate's gates run; it derives no value from c2.
    pub const DEFAULT: AdmissionPolicy = AdmissionPolicy::RecognizedShape;

    /// **Every policy**, for an instrument that sweeps the decision surface —
    /// the shape [`crate::Relax`] already ships for the census's relaxation
    /// ladder.
    ///
    /// Completeness is held by a mechanism and not by this doc comment:
    /// [`Self::index`] is an **exhaustive match**, so a new variant does not
    /// compile until it is given an index, and `all_is_complete_and_indexed`
    /// then fails until it is also given a place here.
    /// PROV[N] not load-bearing — the enumeration of this crate's own policy variants, derived entirely from the enum it lists.
    pub const ALL: &'static [AdmissionPolicy] =
        &[AdmissionPolicy::RecognizedShape, AdmissionPolicy::Nothing];

    /// This policy's position in [`Self::ALL`]. Exhaustive on purpose — see
    /// [`Self::ALL`].
    pub fn index(self) -> usize {
        match self {
            AdmissionPolicy::RecognizedShape => 0,
            AdmissionPolicy::Nothing => 1,
        }
    }

    /// A stable, published name, for an instrument that has to say **which**
    /// policy a number was taken under. Kebab-case, like every other key this
    /// crate publishes.
    pub fn name(self) -> &'static str {
        match self {
            AdmissionPolicy::RecognizedShape => "recognized-shape",
            AdmissionPolicy::Nothing => "nothing",
        }
    }
}

/// **DECODE — what one `.ex` function segment says**, with no admission verdict
/// attached.
///
/// Total over segments: [`Self::of`] is defined for every segment and never
/// refuses. A segment the port's vocabulary cannot spell produces a `Decoded`
/// whose [`Self::shape`] is `None` and whose [`Self::block`] says where the
/// read stopped — which is a **reading**, not a rejection.
///
/// It borrows the segment rather than copying it so that the decode facts a
/// consumer may want but admission never consults ([`Self::call_tokens`],
/// [`Self::body_start`]) are computed **on demand**. That is deliberate and it
/// is the `#3336` cost clause being respected at the design stage: this type is
/// constructed once per function body on `PortC2::build`'s hottest path, and an
/// eagerly-computed diagnostic field would be a throughput cost paid by every
/// caller for a fact almost none of them read.
pub struct Decoded<'a> {
    seg: &'a [u8],
    read: Result<BodyShape, Block>,
}

impl<'a> Decoded<'a> {
    /// **THE decode entry.** One parse, whose result may then be read as many
    /// ways as a caller likes — including, but not only, by asking
    /// [`AdmissionPolicy`] about it.
    ///
    /// `dispatch_reset()` runs inside [`super::parse_segment_detail`], which is
    /// where it has always run; this constructor adds no per-body state and
    /// clears none.
    ///
    /// The `.sy` view's lifetime is deliberately **not** tied to the segment's:
    /// `Bindings::locals` hands out a view borrowed from the binding, and a
    /// `Decoded` keeps only the segment, so tying them would make every
    /// `Decoded` outlive a binding it does not hold.
    pub(crate) fn of(seg: &'a [u8], sy: SyView<'_>) -> Self {
        Decoded { seg, read: parse_segment_detail(seg, sy) }
    }

    // ---- the DECODE face -----------------------------------------------------

    // There is deliberately **no** `shape() -> Option<&BodyShape>` accessor on
    // the decode face. The only consumer of a `BodyShape` in this tree is
    // emission, and emission must go through [`Self::admit`] — an accessor that
    // handed the grammar out without the admission question being asked would
    // be the fused route rebuilt one layer up, and it would compile. The decode
    // face answers *whether* a grammar was reached ([`Self::reached_shape`]),
    // where the read stopped ([`Self::block`]), and facts about the segment;
    // a later I1 slice that has a non-emitting consumer for the grammar itself
    // adds the accessor **with** that consumer, not before it.

    /// Whether the decode reached a recognized grammar at all — the **reach**
    /// question, which is what an I1 progress instrument measures and what
    /// `docs/DECISIONS_2026-08-22.md` decision 13 gives lane `w-decodereach`.
    ///
    /// Public, and public on purpose: reach is a measurement, and it licenses
    /// nothing. Nothing in `scripts/gate.sh` reads it and no emit consults it.
    pub fn reached_shape(&self) -> bool {
        self.read.is_ok()
    }

    /// Where the decode stopped, when it did not reach a grammar.
    ///
    /// The positive parser fails closed at the *first* byte it cannot account
    /// for, so this is a **first blocker and not a distance** (`#3131`) — a
    /// caution that belongs on every reader of this value.
    pub fn block(&self) -> Option<Block> {
        self.read.as_ref().err().copied()
    }

    /// The census key naming the blocking feature, when the decode stopped.
    /// `None` when it did not. See [`Block::feature`].
    pub fn feature(&self) -> Option<String> {
        self.block().map(Block::feature)
    }

    /// The segment this decode read.
    pub fn segment(&self) -> &'a [u8] {
        self.seg
    }

    /// The body-start (`LO`) offset, in both of its forms, or `None` when the
    /// segment carries no body-start token at all.
    ///
    /// A decode fact. Computed on demand — see the type's own doc for why no
    /// diagnostic is a field.
    pub fn body_start(&self) -> Option<usize> {
        crate::func::body_start(self.seg)
    }

    /// **A decode fact that admission provably does not consult.**
    ///
    /// [`super::call_tokens`]'s own doc says it: *"Diagnostic only. Nothing
    /// here is consulted by the emitter or by acceptance."* It is exposed here
    /// because a type whose only content is the admission verdict would not be
    /// a decode layer at all — it would be the same fused answer wearing a new
    /// name — and this is the cheapest already-measured fact that is on the
    /// decode side of the line and provably not on the admission side.
    ///
    /// It is a deliberate **over-count** of the calls in the body; see
    /// [`super::call_tokens`] for its six undercount and three overcount
    /// witnesses.
    pub fn call_tokens(&self) -> usize {
        super::call_tokens(self.seg)
    }

    // ---- the ADMISSION face --------------------------------------------------

    /// **THE ADMISSION QUESTION — may the port emit this body?** — asked over
    /// the decode result under an explicitly named policy, and never by a
    /// second parse.
    ///
    /// **A VERDICT is a yes/no even though a REASON exists beside it** — see
    /// [`AdmissionPolicy`]'s ⚠ section and boards **#3556** / **#3591**. The
    /// reason lives on [`Self::into_admit`]'s `Err` side, and which layer owns
    /// it depends on which layer refused.
    pub fn is_admitted_under(&self, policy: AdmissionPolicy) -> bool {
        match policy {
            AdmissionPolicy::RecognizedShape => self.reached_shape(),
            AdmissionPolicy::Nothing => false,
        }
    }

    /// [`Self::is_admitted_under`] at [`AdmissionPolicy::DEFAULT`].
    ///
    /// Public beside [`Self::reached_shape`] so that the two questions this
    /// module exists to separate can be **compared** by an instrument — which
    /// is the measurement `w-decodereach` was dispatched to make. Today they
    /// are equal for every segment, by [`AdmissionPolicy::RecognizedShape`]'s
    /// definition; the day they are not is the day a general decode has landed
    /// without a widening, which is exactly the outcome the split is for.
    pub fn is_admitted(&self) -> bool {
        self.is_admitted_under(AdmissionPolicy::DEFAULT)
    }

    /// **The emitting form of [`Self::is_admitted_under`]**: the body the port
    /// may emit under `policy`, or the DECODE's own stopping point.
    ///
    /// Consuming, because `shape_to_function` takes its [`BodyShape`] by value.
    ///
    /// The `match` is exhaustive over [`AdmissionPolicy`] on purpose: a new
    /// variant cannot be added without this site being made to handle it, and
    /// handling it means answering *"what key does an admission refusal
    /// report"*. Board **#3556** read that question as unanswerable; board
    /// **#3591** answers it, and the answer is the shape of the `Nothing` arm
    /// below.
    ///
    /// # WHICH LAYER OWNS THE REASON DEPENDS ON WHICH LAYER REFUSED
    ///
    /// The [`AdmissionPolicy::Nothing`] arm is written as two cases and it has
    /// to be:
    ///
    /// * the decode **stopped** (`Err`) — the reason is the decode's, it is
    ///   already recorded at the byte that stopped it, and overwriting it with
    ///   an admission key would destroy the histogram that ranks the roadmap.
    ///   `Err(b) => Err(b)`, untouched;
    /// * the decode read the body **whole** (`Ok`) — nothing about the read is
    ///   wrong, the policy is the only thing that refused, and it says so with
    ///   [`super::ADMISSION_DECLINED`] at [`Block::at_end`]. The `:eof` that
    ///   renders is the **true** statement (`Block::seg_len`'s own doc): the
    ///   body is grammar-complete and no second blocker hides behind the row.
    ///
    /// This is exactly the shape the seven pre-existing `Block::at_end` sites
    /// in `census.rs` already had — [`super::OPT_MODE`] and
    /// [`super::CALLEE_DEFINED_IN_TU`] are post-parse admission refusals raised
    /// only on an otherwise-in-class body — so it generalizes a shipped
    /// pattern rather than inventing one beside it.
    ///
    /// Note what the [`AdmissionPolicy::RecognizedShape`] arm is: **the
    /// identity on the decode result.** That is not a shortcut, it is that
    /// policy's entire content, written once and in one place, and it is
    /// **textually unchanged** by the lane that added the second arm — which is
    /// why the admitted set could not move.
    pub(crate) fn into_admit(self, policy: AdmissionPolicy) -> Result<BodyShape, Block> {
        match policy {
            AdmissionPolicy::RecognizedShape => self.read,
            AdmissionPolicy::Nothing => match self.read {
                Err(b) => Err(b),
                Ok(_) => Err(Block::at_end(self.seg, super::ADMISSION_DECLINED)),
            },
        }
    }

    /// [`Self::into_admit`] at [`AdmissionPolicy::DEFAULT`].
    pub(crate) fn into_admit_default(self) -> Result<BodyShape, Block> {
        self.into_admit(AdmissionPolicy::DEFAULT)
    }

    /// [`Self::into_admit_default`] as the `Option` the pre-split
    /// `parse_segment` returned. **The refusal reason is dropped here**, so a
    /// caller with anywhere to put it should not use this.
    pub(crate) fn admitted_default(self) -> Option<BodyShape> {
        self.into_admit_default().ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::func::test_fixtures::{
        free_fn, BOOL_LIT, CALL_THEN_STMT, IND_DEREF, INT_ARGTAIL, INT_PLUS0, INT_TAILRET,
        MVP_CALL, MVP_FRAMED, NARROW_LL_PACKED_REFUSED, NARROW_SHORT_TO_INT_REFUSED, NO_LOCALS,
        SEQ_CALL_VALUE, SEQ_TWO_VOID, STORE_MEMBER, TWO_CALLS,
    };

    /// **The default policy reproduces the pre-split rule exactly**: admission
    /// is decode-reach, for an accepted body and for a refused one alike.
    #[test]
    fn default_policy_is_decode_reach() {
        let ok = free_fn(MVP_CALL);
        let d = Decoded::of(&ok, NO_LOCALS);
        assert!(d.reached_shape());
        assert!(d.is_admitted());
        assert_eq!(d.reached_shape(), d.is_admitted());
        assert!(d.block().is_none() && d.feature().is_none());

        // A segment with no body-start marker at all: the decode reads nothing
        // and says so, and admission refuses — two answers, one parse.
        let junk = [0u8; 8];
        let d = Decoded::of(&junk, NO_LOCALS);
        assert!(!d.reached_shape());
        assert!(!d.is_admitted());
        assert_eq!(d.reached_shape(), d.is_admitted());
        assert_eq!(d.feature().as_deref(), Some("lo-marker:mid"));
    }

    /// **A body is DECODED without being admitted, and the two answers come off
    /// one parse.**
    ///
    /// This is the property the module exists for, and it is asserted on the
    /// refusing side because that is the side where the two questions have
    /// always been indistinguishable: before the split, a body outside the 35
    /// grammars produced a single `None` that meant both *"unreadable"* and
    /// *"unemittable"*, and nothing could say which.
    #[test]
    fn a_refused_body_is_still_decoded_and_still_readable() {
        let junk = [0u8; 8];
        let d = Decoded::of(&junk, NO_LOCALS);

        // ADMISSION says no…
        assert!(!d.is_admitted());
        assert!(!d.is_admitted_under(AdmissionPolicy::DEFAULT));
        // …and the DECODE still answers, in three ways, off the same parse.
        assert_eq!(d.feature().as_deref(), Some("lo-marker:mid"));
        assert_eq!(d.block().map(|b| b.off), Some(0));
        assert_eq!(d.call_tokens(), 0);
    }

    /// **THE ONE DUPLICATION IN THIS MODULE, AND ITS FENCE.**
    ///
    /// The policy is matched in two places — [`Decoded::is_admitted_under`]
    /// (the yes/no) and [`Decoded::into_admit`] (the emitting form) — because
    /// Rust cannot derive an owning return from a borrowing one. Two matches
    /// over one enum is exactly the *"two predicates that can drift apart"*
    /// shape the lane that built this module was forbidden to ship, so it is
    /// fenced here rather than argued away in a comment: every pinned segment
    /// in the crate's fixture set is asked both ways and the answers must
    /// agree, on the accepting side and the refusing side alike.
    ///
    /// A future policy variant handled in one match and not the other fails
    /// this test. That is the whole reason it exists.
    ///
    /// **W-ATEND: it sweeps [`AdmissionPolicy::ALL`] now, not just the
    /// default.** Asking only the default is a green control over half the
    /// population the moment a second variant exists (`docs/STATUS.md` trap 0),
    /// and this file's own fence was in that state for exactly one lane.
    #[test]
    fn the_yes_no_and_the_emitting_form_cannot_disagree_under_any_policy() {
        // Both a whole-segment corpus and two deliberate non-bodies, so the
        // population is not all on one side of the predicate (`STATUS.md`
        // trap 0: a green control is a statement about the population it ran
        // over).
        let framed: Vec<Vec<u8>> = [
            MVP_CALL, MVP_FRAMED, INT_TAILRET, INT_PLUS0, INT_ARGTAIL, TWO_CALLS,
            SEQ_TWO_VOID, SEQ_CALL_VALUE, STORE_MEMBER, IND_DEREF, BOOL_LIT,
            NARROW_SHORT_TO_INT_REFUSED, NARROW_LL_PACKED_REFUSED, CALL_THEN_STMT,
        ]
        .iter()
        .map(|b| free_fn(b))
        .collect();
        let mut segs: Vec<&[u8]> = framed.iter().map(|v| v.as_slice()).collect();
        let junk = [0u8; 8];
        let truncated = &MVP_CALL[..MVP_CALL.len() / 2];
        segs.push(&junk);
        segs.push(truncated);

        // Per policy, so a variant answering one match and not the other is
        // caught wherever it sits in `ALL` — and so the DEFAULT's own
        // two-sidedness is still asserted separately below.
        let mut default_yes = 0usize;
        let mut default_no = 0usize;
        for policy in AdmissionPolicy::ALL.iter().copied() {
            let (mut yes, mut no) = (0usize, 0usize);
            for seg in &segs {
                let a = Decoded::of(seg, NO_LOCALS).is_admitted_under(policy);
                let b = Decoded::of(seg, NO_LOCALS).into_admit(policy).is_ok();
                assert_eq!(
                    a,
                    b,
                    "the two policy matches disagree under {} on a {}-byte segment",
                    policy.name(),
                    seg.len()
                );
                if a { yes += 1 } else { no += 1 }
            }
            // The denominator, per policy, printed in the assertion that uses
            // it: `0 disagreements over 0 cases` is not a result.
            assert_eq!(yes + no, segs.len(), "{} did not answer every segment", policy.name());
            if policy == AdmissionPolicy::DEFAULT {
                (default_yes, default_no) = (yes, no);
            }
            // `Nothing` is one-sided BY DEFINITION, so the two-sidedness
            // requirement is asserted where it means something — the default —
            // and the refusing policy is asserted to be totally refusing,
            // which is the claim its own name makes.
            if policy == AdmissionPolicy::Nothing {
                assert_eq!(yes, 0, "`Nothing` admitted {yes} of {} segments", segs.len());
            }
        }
        assert!(
            default_yes > 0 && default_no > 0,
            "the DEFAULT's population is one-sided: {default_yes} admitted, {default_no} refused"
        );
    }

    /// **[`AdmissionPolicy::ALL`] is complete, and the mechanism is a compile
    /// error before it is this test.**
    ///
    /// `index()` is an exhaustive match, so a third variant does not build
    /// until it is given an index; this then fails until it is also given a
    /// place in `ALL`. Two steps, neither of them a doc comment.
    #[test]
    fn all_is_complete_and_indexed() {
        let mut seen: Vec<usize> = AdmissionPolicy::ALL.iter().map(|p| p.index()).collect();
        seen.sort_unstable();
        assert_eq!(
            seen,
            (0..AdmissionPolicy::ALL.len()).collect::<Vec<_>>(),
            "ALL's indices are not exactly 0..len — a variant is missing from ALL, \
             duplicated in it, or indexed past its end"
        );
        // Pinned, so ADDING a variant and indexing it without listing it here
        // is a failure and not a silent shrink of every sweep.
        assert_eq!(AdmissionPolicy::ALL.len(), 2);
        let mut names: Vec<&str> = AdmissionPolicy::ALL.iter().map(|p| p.name()).collect();
        names.sort_unstable();
        assert_eq!(names, ["nothing", "recognized-shape"]);
        names.dedup();
        assert_eq!(names.len(), AdmissionPolicy::ALL.len(), "two policies share a name");
    }

    /// **THE FINDING — the admission layer owns a refusal REASON for exactly
    /// the bodies it alone refuses.** Boards **#3591** / **#3556**.
    ///
    /// Both sides are asserted because the split is the content of the claim:
    /// a body the decode STOPPED on keeps the decode's key unchanged, and a
    /// body the decode read WHOLE gets admission's own.
    #[test]
    fn the_admission_layer_owns_a_reason_only_where_it_alone_refused() {
        // (a) READ WHOLE, refused by the policy: admission's key, and `:eof` is
        //     the true statement — the parse reached `seg.len()`.
        let ok = free_fn(MVP_CALL);
        assert!(Decoded::of(&ok, NO_LOCALS).reached_shape(), "fixture must decode whole");
        assert!(Decoded::of(&ok, NO_LOCALS).into_admit_default().is_ok(), "…and be admitted");
        let b = Decoded::of(&ok, NO_LOCALS)
            .into_admit(AdmissionPolicy::Nothing)
            .expect_err("`Nothing` admits nothing");
        assert_eq!(b.feature(), "admission-declined:eof");
        assert_eq!(b.ctx, super::super::ADMISSION_DECLINED);
        assert_eq!(b.off, ok.len(), "at_end records the segment end, not the last byte");
        assert_eq!(b.seg_len, ok.len());
        assert!(b.byte.is_none());
        assert!(b.completeness().is_whole(), "a whole-read body's refusal is grammar-complete");

        // (b) The decode STOPPED: the reason was never admission's, and the
        //     key must be byte-identical to what the default reports. A policy
        //     that overwrote it would destroy the histogram that ranks the
        //     roadmap — which is `OPT_MODE`'s own recorded reason for being
        //     applied last.
        let junk = [0u8; 8];
        let under_default = Decoded::of(&junk, NO_LOCALS)
            .into_admit(AdmissionPolicy::DEFAULT)
            .expect_err("junk cannot decode");
        let under_nothing = Decoded::of(&junk, NO_LOCALS)
            .into_admit(AdmissionPolicy::Nothing)
            .expect_err("junk cannot decode");
        assert_eq!(under_default, under_nothing);
        assert_eq!(under_nothing.feature(), "lo-marker:mid");
    }

    /// **The `Nothing` arm changes NOTHING about the default**, asserted over
    /// the same corpus the drift fence uses: for every segment, the default's
    /// `Result` is identical whether or not the second variant exists in the
    /// enum beside it.
    ///
    /// It cannot be otherwise — the arms are separate — and that is the point:
    /// this is the property the lane's required-zero byte delta rests on, so it
    /// is asserted rather than argued.
    #[test]
    fn the_default_is_untouched_by_the_second_variant() {
        for seg in [
            free_fn(MVP_CALL),
            free_fn(MVP_FRAMED),
            free_fn(INT_TAILRET),
            free_fn(NARROW_LL_PACKED_REFUSED),
            vec![0u8; 8],
        ] {
            let d = Decoded::of(&seg, NO_LOCALS);
            assert_eq!(d.is_admitted(), d.reached_shape());
            assert_eq!(
                Decoded::of(&seg, NO_LOCALS).into_admit_default().is_ok(),
                Decoded::of(&seg, NO_LOCALS).reached_shape(),
                "the DEFAULT is still the identity on the decode result"
            );
        }
    }

    /// The DEFAULT policy is the one every production site passes, and it is
    /// `RecognizedShape`. Pinned, because the whole required-zero grade of the
    /// rung that introduced this module is taken **at the default and nowhere
    /// else**, and a default that moved would move the admitted set silently.
    #[test]
    fn the_default_policy_is_pinned() {
        assert_eq!(AdmissionPolicy::DEFAULT, AdmissionPolicy::RecognizedShape);
    }

    /// A decode fact admission does not consult is readable off a `Decoded`
    /// whether or not the body is admitted.
    #[test]
    fn decode_facts_are_available_on_a_refused_body() {
        let junk = [0u8; 8];
        let d = Decoded::of(&junk, NO_LOCALS);
        assert!(!d.is_admitted());
        assert_eq!(d.call_tokens(), 0);
        assert_eq!(d.body_start(), None);
        assert_eq!(d.segment().len(), 8);
    }
}
