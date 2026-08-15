//! **`IL_STMT_GRAMMAR.md` §14.2 step 5's fail-closed boundary, as a decidable
//! pre-emission predicate.** Lane `w-stmt5`.
//!
//! Step 5 is *"`38`/`39` conditional branches, `3A` jump, `29 <tok>` label
//! definitions; build a token → position map in a first pass over the body"*,
//! and its boundary is the longest one in that document:
//!
//! > *this is where the temptation is greatest and the risk is highest.
//! > Decoding a CFG is not lowering one. Emission must stay gated on a whitelist
//! > of shapes, never on "the branches decoded". Also: refuse any body where a
//! > label is targeted before it is defined **and** the port's codegen has no
//! > fixup pass — a backward jump is a loop, and a loop needs register
//! > allocation across a back edge.*
//!
//! Two things about that text have moved since it was written and this module is
//! written against the amended form, not the original:
//!
//! 1. **"Whitelist of shapes" means a decidable pre-emission predicate**, not an
//!    enumerated list of function names — `CFG_SHAPE.md` §6.3's dated
//!    2026-08-13 block, adopted by the user. A general lowering behind a checked
//!    predicate satisfies the rule; `codegen/fold.rs`'s `FoldShape::admit` is
//!    the reference implementation and this is the reader-side counterpart.
//! 2. **The back-edge clause is a conjunction and its second conjunct has
//!    changed.** `LabelMap`'s fixup pass exists, with nine production clients,
//!    and `LabelMap::admitting_back_edges(ChargedClass)` is a per-map admission
//!    fixed at construction (board **#3151**). So *"and the port's codegen has no
//!    fixup pass"* is no longer universally true, and a lane that refused every
//!    back edge on the strength of the original sentence would be enforcing a
//!    fence that was lifted.
//!
//!    **This module refuses back edges anyway, and states why in the one place a
//!    later lane will look.** `admitting_back_edges` takes a **`ChargedClass`** —
//!    a closed enum whose every variant is graded against the label-counter gate
//!    — so what exists is not "back edges are fine now", it is "a back edge is
//!    admissible *for a class that has been charged*". A reader-side predicate
//!    that admitted a back edge for an *unnamed* class would hand codegen a body
//!    with no charge behind it, which is precisely the wrong-`$M` case
//!    invariant 4 was built for. The refusal is therefore **parameterless on
//!    purpose**: when a charged CFG class exists, this predicate takes the class
//!    as a parameter exactly as `FoldShape::admit` takes its band, and not
//!    before.
//!
//! ## What this module is NOT
//!
//! **It admits nothing into [`BodyShape`](super::super::BodyShape) and it moves
//! no byte.** Nothing calls it from the accepting parser. That is a deliberate
//! and measured decision, not an oversight, and the measurement is in
//! `docs/rungs/2026-08-15-stmt5.md` §3: on the 878-TU workload's emitted
//! population — the one `fnbyte-refused-parse` is counted over — the bodies step
//! 5 can *reach* number **5 of 113,612**, because a reader step can only move a
//! body it can model and `emit-cflow-modeled-key` says what the modelable
//! population is blocked on. An accept built on this predicate would be a
//! **membership check standing in for a model**: `CfResidue::Modeled` says every
//! operand token is *inside the vocabulary the port's emitters consume*, which is
//! a different and much weaker claim than "an expression tree was built". Moving
//! `fnbyte-refused-parse` on that basis would move the phase's own numerator
//! without the phase having happened, and `w-readphase` §6.1's rule — a body
//! leaving `fnbyte-refused-parse` lands in `fnbyte-refused-codegen`, meaning
//! *the emitter declined* — would then be false in the tree that publishes it.
//!
//! What it is for is the other half of `CLAUDE.md`'s scoring rule: a fence is
//! priced two-sided **before** it ships. This is the fence, written down as a
//! check a later lane can run, mutate and disagree with, instead of as a
//! paragraph a later lane has to re-derive.

use super::control_flow::{CfResidue, CfScan, CfShape, LabelTable};

/// The verdict of [`CfgAdmit`]. Every refusal names its own clause; a lane that
/// wants to know *why* a body is out never has to re-run the predicate to find
/// out.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CfgVerdict {
    /// Every clause passed. Carries the shape, so a caller that admits only some
    /// shapes does not have to ask twice — the same parameterisation
    /// `FoldShape::admit` uses for its band.
    Admit(CfShape),
    /// The first pass did not reach the function tail. **Nothing below is
    /// knowable**, so this is first and is not merged with any other refusal.
    Undecoded,
    /// `CfShape::Switch`. §14.2: *"`switch` is deliberately last"* — three more
    /// opcodes and a jump table in `.rdata` or `.text`.
    Switch,
    /// Two `29`s carry one token: the map is not a function.
    AmbiguousLabel,
    /// A `38`/`39`/`3A` names a token no `29` defines.
    UnresolvedTarget,
    /// A label is targeted before it is defined — §14.2 step 5's own clause.
    BackEdge,
    /// The operand stream leaves `CfResidue::Modeled`.
    UnmodeledOperand,
}

impl CfgVerdict {
    /// The census sub-key. Stable strings: they are what crosses the
    /// `c2-il`/`c2-harness` seam and what a scan's `gap-metric` rows are named
    /// after.
    pub(crate) fn name(self) -> &'static str {
        match self {
            CfgVerdict::Admit(s) => match s {
                CfShape::Straight => "admit-straight",
                CfShape::MultiExit => "admit-multi-exit",
                CfShape::Forward(_) => "admit-forward",
                // Unreachable through `CfgAdmit::of` — the `Switch` and `Loop`
                // clauses precede the admit — and named rather than
                // `unreachable!()` so a future caller that constructs a verdict
                // by hand gets a wrong census row instead of a panic in a
                // census that runs over 1.7M bodies.
                CfShape::Loop => "admit-loop-UNEXPECTED",
                CfShape::Switch => "admit-switch-UNEXPECTED",
            },
            CfgVerdict::Undecoded => "refuse-undecoded",
            CfgVerdict::Switch => "refuse-switch",
            CfgVerdict::AmbiguousLabel => "refuse-ambiguous-label",
            CfgVerdict::UnresolvedTarget => "refuse-unresolved-target",
            CfgVerdict::BackEdge => "refuse-back-edge",
            CfgVerdict::UnmodeledOperand => "refuse-unmodeled-operand",
        }
    }

    /// Whether this verdict permits emission. One method, so no caller spells
    /// the test itself and the admitted set has exactly one definition.
    ///
    /// **No production caller yet, and that is the module's own headline** — see
    /// the "what this module is NOT" section. It is exercised by the tests that
    /// grade the predicate; the attribute records the absence rather than
    /// hiding it, so the day an emitter takes this fence the attribute is what
    /// comes off.
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn admits(self) -> bool {
        matches!(self, CfgVerdict::Admit(_))
    }
}

/// **The predicate.** Decidable, total, and computed entirely from facts the
/// first pass already collected — no second walk, no heuristic, no cost model.
pub(crate) struct CfgAdmit;

impl CfgAdmit {
    /// Decide one body.
    ///
    /// ## The clause order, and why it is not arbitrary
    ///
    /// Two of the five orderings below are load-bearing and the rest are not;
    /// stating which is the difference between a rule and a ritual.
    ///
    /// 1. **`decoded` first.** Every question the map answers is a question
    ///    about a body that was read end to end. On a partial walk the map holds
    ///    whatever the walk got to before it stopped, so `back_edges() == 0`
    ///    there means *"no back edge in the prefix"*, which is not the claim.
    /// 2. **`Switch` before every map-derived clause — LOAD-BEARING.**
    ///    [`LabelTable`] records `29`/`38`/`39`/`3A` and nothing else, while
    ///    `3B`/`3C`/`3D` carry label tokens of their own (§11). On a `switch`
    ///    body the map is therefore incomplete *by construction*, and
    ///    `unresolved()` / `back_edges()` read off it are answers to a question
    ///    about a different body. Asking the shape first is the only order in
    ///    which they are never read.
    /// 3. **`duplicate_defs` before `unresolved` and `back_edges` —
    ///    LOAD-BEARING.** Both of those call
    ///    [`LabelTable::position_of`], which returns *the first* definition. If
    ///    a token has two, "the position of `tok`" has no referent and the two
    ///    clauses below are computing with a value that does not mean anything.
    /// 4. **`unresolved` before `back_edges` — LOAD-BEARING.**
    ///    `back_edges` treats an unknown position as *not backward*, which is
    ///    the permissive reading. That is safe only because a body with an
    ///    unknown position has already been refused. Swap these two and an
    ///    undefined target passes as a forward DAG.
    /// 5. **`back_edges` before `residue` — NOT load-bearing**, and the lane
    ///    registered a mutant on it as GREEN in advance. Neither reads anything
    ///    the other computes, so swapping them changes which refusal a body that
    ///    is *both* reports and changes the admitted set by nothing. The
    ///    admitted-set test is green under the swap and the verdict test is red,
    ///    and the pair of results is what says the order is presentational here
    ///    and structural above.
    pub(crate) fn of(scan: &CfScan) -> CfgVerdict {
        let Ok(cf) = &scan.body else {
            return CfgVerdict::Undecoded;
        };
        if cf.shape == CfShape::Switch {
            return CfgVerdict::Switch;
        }
        let t: &LabelTable = &scan.labels;
        if t.duplicate_defs() > 0 {
            return CfgVerdict::AmbiguousLabel;
        }
        if t.unresolved() > 0 {
            return CfgVerdict::UnresolvedTarget;
        }
        if t.back_edges() > 0 {
            return CfgVerdict::BackEdge;
        }
        if cf.residue != CfResidue::Modeled {
            return CfgVerdict::UnmodeledOperand;
        }
        CfgVerdict::Admit(cf.shape)
    }

    /// **The consistency control between the two readings of one collection.**
    ///
    /// `CfShape::Loop` and this predicate's `BackEdge` clause are computed from
    /// the same `Site` vectors in the same pass, so they cannot disagree about
    /// what the body *contains*; they can drift about what to conclude. `true`
    /// means they did, on a non-`switch` decoded body, and the count is
    /// published per scan (`cfg-admit-backedge-shape-disagree`) with a target of
    /// **0**.
    ///
    /// **It is a consistency check and NOT independent evidence, and saying so
    /// is the point.** [`super::control_flow::shape_of`] derives `Loop` by the
    /// same `definition earlier than reference` test [`LabelTable::back_edges`]
    /// uses, so a zero here confirms the two have not drifted and confirms
    /// nothing about whether either is right. The facts the map contributes that
    /// are *not* available anywhere else are `unresolved`, `duplicate_defs` and
    /// `dead_defs`; those are the ones worth a number.
    pub(crate) fn backedge_disagrees_with_shape(scan: &CfScan) -> bool {
        let Ok(cf) = &scan.body else { return false };
        if cf.shape == CfShape::Switch {
            return false;
        }
        (cf.shape == CfShape::Loop) != (scan.labels.back_edges() > 0)
    }

    /// **The map's own liveness check** — `true` when a body decoded end to end
    /// and the table came back **wholly** empty.
    ///
    /// The failure mode it exists for is the one `back_edges() == 0` cannot
    /// distinguish itself from: an empty map answers *no* to every question, so
    /// an instrument that had silently stopped collecting sites would look
    /// perfect. Target **0**, published in the alarm cell.
    ///
    /// ## Why it is `&&` and not `||`, which is a measured correction
    ///
    /// It was written `defs == 0 || refs == 0`, on `IL_STMT_GRAMMAR.md` §9's
    /// *"every body has an epilogue `3A` and the `29` it targets"*. **That
    /// fired on three real bodies and §9 is the thing that is wrong**, in one
    /// direction: a body may carry the epilogue label with **no jump to it at
    /// all**, reaching it by fallthrough. The witness is sixteen bytes and is
    /// pinned in this module's tests —
    ///
    /// ```text
    ///   4C 4F 11 · 53 · 54 02 · 29 <tok> · 4F 12 47 54 01 54 00
    /// ```
    ///
    /// — `src/Main.cpp`, `MidiParserMgr.cpp` and `MidiReader.cpp`, one each, out
    /// of 2,410,886 scanned bodies. Two of the three were scoring
    /// `admit-straight` and were **right** to: a body with no branch at all is
    /// the most admissible body there is, and the guard was refusing it for
    /// having exactly the property [`LabelTable::dead_defs`] is documented as
    /// tolerating. The count is kept as its own measurement — see
    /// [`CfgAdmit::has_fallthrough_epilogue`] — rather than discarded with the
    /// guard, because a grammar counterexample found by an over-strong control
    /// is the most valuable thing that control produced.
    pub(crate) fn label_map_is_empty_on_a_decoded_body(scan: &CfScan) -> bool {
        if scan.body.is_err() {
            return false;
        }
        let (defs, refs) = scan.labels.sizes();
        defs == 0 && refs == 0
    }

    /// **`IL_STMT_GRAMMAR.md` §9's counterexample, counted** — a decoded body
    /// that defines at least one label and references none.
    ///
    /// §9 says the epilogue `3A` is always there. It is not, on three bodies of
    /// the workload, and this is the standing count of them: a published number
    /// so the exception cannot quietly grow or quietly vanish. It is **not** a
    /// refusal and not an alarm — the bodies are admissible and are admitted.
    pub(crate) fn has_fallthrough_epilogue(scan: &CfScan) -> bool {
        if scan.body.is_err() {
            return false;
        }
        let (defs, refs) = scan.labels.sizes();
        defs > 0 && refs == 0
    }
}

#[cfg(test)]
mod tests {
    use super::super::control_flow::tests::{EARLY_RETURN, EMPTY, IF_ELSE, WHILE};
    use super::super::control_flow::{scan_full, CfShape};
    use super::*;
    use crate::func::bundle::LO_MARKER;
    use crate::func::readers::find_subslice;

    /// Decode one pinned segment and score it against the boundary.
    fn verdict(seg: &[u8]) -> CfgVerdict {
        let lo = find_subslice(seg, &LO_MARKER).expect("a body marker");
        CfgAdmit::of(&scan_full(seg, lo))
    }

    /// The map's four questions on one pinned segment:
    /// `(defs, refs, back_edges, unresolved, dead_defs, duplicate_defs)`.
    fn table(seg: &[u8]) -> (usize, usize, usize, usize, usize, usize) {
        let lo = find_subslice(seg, &LO_MARKER).expect("a body marker");
        let t = scan_full(seg, lo).labels;
        let (d, r) = t.sizes();
        (d, r, t.back_edges(), t.unresolved(), t.dead_defs(), t.duplicate_defs())
    }

    /// **The calibration.** `EMPTY` is one epilogue jump and one epilogue label,
    /// which is the smallest body there is, so every count below is the floor
    /// the other segments are read against. Asserted as a whole tuple rather
    /// than field by field: a partial assertion here is how a map that silently
    /// stopped collecting reads as correct.
    #[test]
    fn the_map_on_the_smallest_body_is_one_definition_and_one_reference() {
        assert_eq!(table(EMPTY), (1, 1, 0, 0, 0, 0));
        assert_eq!(verdict(EMPTY), CfgVerdict::Admit(CfShape::Straight));
    }

    /// `IF_ELSE` is §7's diamond: three labels (`else`, `join`, epilogue), three
    /// references (the `38`, the `3A` to the join, the epilogue `3A`), all
    /// forward, none dangling, none dead.
    #[test]
    fn the_map_on_a_diamond_is_three_forward_references_and_no_dead_label() {
        assert_eq!(table(IF_ELSE), (3, 3, 0, 0, 0, 0));
        assert_eq!(verdict(IF_ELSE), CfgVerdict::Admit(CfShape::Forward(1)));
    }

    /// **`EARLY_RETURN` is the one that separates `dead_defs` from zero.** Two
    /// returns converge on ONE epilogue label, and the `if`'s skip label is
    /// defined but — because the then-clause `return`s rather than falling
    /// through past a `3A` — is targeted by the `38` alone. It is the segment
    /// that proves `dead_defs` is a control and not a refusal: a predicate that
    /// refused on it would refuse this body, which is `Forward` and modeled.
    #[test]
    fn a_body_with_two_returns_to_one_label_still_admits() {
        let (defs, refs, back, unres, _dead, dup) = table(EARLY_RETURN);
        assert_eq!((defs, refs, back, unres, dup), (2, 3, 0, 0, 0));
        assert_eq!(verdict(EARLY_RETURN), CfgVerdict::Admit(CfShape::Forward(1)));
    }

    /// **`WHILE` is §14.2 step 5's own clause, on a real capture.** The
    /// `3A E8 09` at the end of the body targets the `29 E8 09` that opened it,
    /// and `3A` carries no direction — so nothing but the recorded positions can
    /// tell, which is the sentence the map exists to make true.
    #[test]
    fn the_back_edge_clause_refuses_a_real_while_loop() {
        let (_defs, _refs, back, unres, _dead, dup) = table(WHILE);
        assert_eq!((back, unres, dup), (1, 0, 0));
        assert_eq!(verdict(WHILE), CfgVerdict::BackEdge);
    }

    /// **A body the walk did not finish is `Undecoded` and nothing else.** The
    /// witness is `EMPTY` with its scope-close depth corrupted — every field
    /// width still correct, so only the depth invariant catches it, and the map
    /// it leaves behind is a prefix. A predicate that read `back_edges() == 0`
    /// off that prefix would be answering about a body it never saw.
    #[test]
    fn a_partial_walk_scores_undecoded_and_not_admit() {
        let mut bad = EMPTY.to_vec();
        // Located by pattern, never by an arithmetic offset: a hand-computed
        // index that drifts silently corrupts a byte the test is not about and
        // still fails closed, which reads as the test passing for its reason.
        let k = bad
            .windows(3)
            .position(|w| w == [0x54, 0x02, 0x29])
            .expect("the body scope close")
            + 1;
        bad[k] = 0x07;
        assert_eq!(verdict(&bad), CfgVerdict::Undecoded);
        assert!(!verdict(&bad).admits());
    }

    /// **`unresolved` fires, and it fires on a body `CfShape` calls `Forward`.**
    /// That is the whole reason the clause exists: retarget `IF_ELSE`'s `38` to
    /// a token no `29` defines and the shape is unchanged — one conditional, no
    /// backward reference — while the CFG has stopped being readable. Before the
    /// map this body was indistinguishable from the diamond above.
    #[test]
    fn a_dangling_branch_target_refuses_while_the_shape_still_reads_forward() {
        let mut bad = IF_ELSE.to_vec();
        let at = bad.windows(3).position(|w| w == [0x38, 0xE8, 0x09]).expect("the 38");
        bad[at + 1] = 0xEE; // a token nothing defines
        let lo = find_subslice(&bad, &LO_MARKER).unwrap();
        let s = scan_full(&bad, lo);
        assert_eq!(s.body.as_ref().map(|c| c.shape), Ok(CfShape::Forward(1)));
        assert_eq!(s.labels.unresolved(), 1);
        assert_eq!(CfgAdmit::of(&s), CfgVerdict::UnresolvedTarget);
    }

    /// **`duplicate_defs` fires, and it must be tested BEFORE the two clauses
    /// that call `position_of`.** Give `IF_ELSE`'s join label the else label's
    /// token and one token has two `29`s; `position_of` then returns the first
    /// of two and every question below it is computing with a value that has no
    /// referent.
    #[test]
    fn two_definitions_of_one_token_refuse_before_any_position_is_read() {
        let mut bad = IF_ELSE.to_vec();
        let at = bad.windows(3).position(|w| w == [0x29, 0xE9, 0x09]).expect("the join 29");
        bad[at + 1] = 0xE8; // now two 29s carry E8 09
        let lo = find_subslice(&bad, &LO_MARKER).unwrap();
        let s = scan_full(&bad, lo);
        assert!(s.labels.duplicate_defs() > 0);
        assert_eq!(CfgAdmit::of(&s), CfgVerdict::AmbiguousLabel);
    }

    /// **The clause order that is load-bearing, asserted as an order and not as
    /// two independent facts.** A body with BOTH a dangling target and a back
    /// edge must report `UnresolvedTarget`: `back_edges` reads an unknown
    /// position as *not backward*, so if these two swapped, this body would pass
    /// the back-edge clause and be admitted as a forward DAG.
    #[test]
    fn unresolved_outranks_back_edge_and_the_body_that_proves_it_is_both() {
        let mut bad = WHILE.to_vec();
        // Keep the back edge; add a dangling target on the conditional.
        let at = bad.windows(3).position(|w| w == [0x38, 0xE9, 0x09]).expect("the 38");
        bad[at + 1] = 0xEE;
        let lo = find_subslice(&bad, &LO_MARKER).unwrap();
        let s = scan_full(&bad, lo);
        assert_eq!(s.labels.back_edges(), 1, "the back edge is still there");
        assert_eq!(s.labels.unresolved(), 1);
        assert_eq!(CfgAdmit::of(&s), CfgVerdict::UnresolvedTarget);
    }

    /// **The consistency control, on every pinned segment this file can reach.**
    /// `CfShape::Loop` and the `BackEdge` clause are two readings of one
    /// collection, and this is the assertion the corpus-wide
    /// `step5-backedge-shape-disagree` key generalises — it reads **0** over
    /// 2,410,886 bodies.
    #[test]
    fn the_back_edge_clause_and_the_loop_shape_never_disagree() {
        for seg in [EMPTY, IF_ELSE, EARLY_RETURN, WHILE] {
            let lo = find_subslice(seg, &LO_MARKER).unwrap();
            assert!(!CfgAdmit::backedge_disagrees_with_shape(&scan_full(seg, lo)));
        }
    }

    /// **`IL_STMT_GRAMMAR.md` §9's counterexample, pinned to the byte.**
    ///
    /// §9 says every body carries an epilogue `3A` and the `29` it targets. This
    /// sixteen-byte body carries the label and **no jump at all** — it reaches
    /// the epilogue by fallthrough. It is transcribed from
    /// `src/system/midi/MidiReader.cpp` at the workload's own flags, and there
    /// are exactly three like it in 2,410,886 scanned bodies (`src/Main.cpp`,
    /// `MidiParserMgr.cpp`, `MidiReader.cpp`, one each).
    ///
    /// It is here because it is the body that **corrected this module's own
    /// guard**: `label_map_is_empty_on_a_decoded_body` was written
    /// `defs == 0 || refs == 0` on the strength of §9's sentence, and this is
    /// what it fired on. The guard was wrong and §9 is wrong; the body is
    /// admissible and is admitted.
    const FALLTHROUGH_EPILOGUE: &[u8] = &[
        0x4C, 0x4F, 0x11, // LO
        0x53, // SS — the body scope
        0x54, 0x02, // close it
        0x29, 0x49, 0x46, // the epilogue label — and NOTHING jumps to it
        0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00, // fn tail
    ];

    #[test]
    fn a_body_can_define_its_epilogue_label_and_never_jump_to_it() {
        // One definition, ZERO references — which is what §9 forbids.
        assert_eq!(table(FALLTHROUGH_EPILOGUE), (1, 0, 0, 0, 1, 0));
        // The liveness guard must NOT fire: the map is not empty, it is
        // reference-free. `&&`, not `||`.
        let lo = find_subslice(FALLTHROUGH_EPILOGUE, &LO_MARKER).unwrap();
        let s = scan_full(FALLTHROUGH_EPILOGUE, lo);
        assert!(!CfgAdmit::label_map_is_empty_on_a_decoded_body(&s), "THE correction");
        assert!(CfgAdmit::has_fallthrough_epilogue(&s));
        // …and the body admits, because a body with no branch at all is the
        // most admissible body there is.
        assert_eq!(CfgAdmit::of(&s), CfgVerdict::Admit(CfShape::Straight));
    }

    /// **The liveness guard fires on a wholly empty map and only there.** An
    /// instrument that had stopped collecting sites would answer *no* to every
    /// question the map is asked, and every clause would pass — so the one
    /// reading that must be distinguished from "simple control flow" is
    /// "nothing was collected".
    #[test]
    fn a_wholly_empty_map_on_a_decoded_body_is_the_alarm() {
        // The same body with its one `29` removed: still lands on the tail,
        // still depth-consistent, and now collects nothing at all.
        let bare: &[u8] = &[
            0x4C, 0x4F, 0x11, 0x53, 0x54, 0x02, //
            0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00,
        ];
        let lo = find_subslice(bare, &LO_MARKER).unwrap();
        let s = scan_full(bare, lo);
        assert!(s.body.is_ok(), "it decodes — that is what makes it dangerous");
        assert_eq!(s.labels.sizes(), (0, 0));
        assert!(CfgAdmit::label_map_is_empty_on_a_decoded_body(&s));
        // …and the alarm does NOT fire on any of the real segments.
        for seg in [EMPTY, IF_ELSE, EARLY_RETURN, WHILE, FALLTHROUGH_EPILOGUE] {
            let lo = find_subslice(seg, &LO_MARKER).unwrap();
            assert!(!CfgAdmit::label_map_is_empty_on_a_decoded_body(&scan_full(seg, lo)));
        }
    }

    /// **Every verdict has a distinct census name, checked over the whole
    /// enum.** A name collision would silently merge two clauses in a histogram,
    /// which is the one failure `GAPS.md` §6 says a census instrument cannot
    /// survive. `Forward`'s payload is deliberately dropped by `name` — one
    /// bucket, not 255 — so the two `Forward` arms below must agree.
    #[test]
    fn the_verdict_names_are_distinct_and_forward_is_one_bucket() {
        let all = [
            CfgVerdict::Admit(CfShape::Straight),
            CfgVerdict::Admit(CfShape::MultiExit),
            CfgVerdict::Admit(CfShape::Forward(1)),
            CfgVerdict::Undecoded,
            CfgVerdict::Switch,
            CfgVerdict::AmbiguousLabel,
            CfgVerdict::UnresolvedTarget,
            CfgVerdict::BackEdge,
            CfgVerdict::UnmodeledOperand,
        ];
        let mut names: Vec<&str> = all.iter().map(|v| v.name()).collect();
        names.sort_unstable();
        let n = names.len();
        names.dedup();
        assert_eq!(names.len(), n, "two verdicts share a census name");
        assert_eq!(
            CfgVerdict::Admit(CfShape::Forward(1)).name(),
            CfgVerdict::Admit(CfShape::Forward(9)).name()
        );
        // …and exactly three of the nine admit.
        assert_eq!(all.iter().filter(|v| v.admits()).count(), 3);
    }

    /// **The admitted set is closed under the clauses, stated as the property
    /// rather than as a list.** Nothing but `Admit` admits, and `Admit` is only
    /// ever reached on a decoded, non-switch, back-edge-free, dangling-free,
    /// modeled body — so a body that admits satisfies all five, checked here by
    /// asking the map again instead of trusting the verdict.
    #[test]
    fn every_admitted_body_satisfies_all_five_clauses_independently() {
        for seg in [EMPTY, IF_ELSE, EARLY_RETURN, WHILE] {
            let lo = find_subslice(seg, &LO_MARKER).unwrap();
            let s = scan_full(seg, lo);
            if !CfgAdmit::of(&s).admits() {
                continue;
            }
            let cf = s.body.as_ref().expect("admitted implies decoded");
            assert_ne!(cf.shape, CfShape::Switch);
            assert_ne!(cf.shape, CfShape::Loop);
            assert_eq!(s.labels.back_edges(), 0);
            assert_eq!(s.labels.unresolved(), 0);
            assert_eq!(s.labels.duplicate_defs(), 0);
            assert_eq!(cf.residue, CfResidue::Modeled);
        }
    }
}
