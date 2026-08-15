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
}
