//! The fold record — `docs/CFG_SHAPE.md` §6.2 item **G**, built.
//!
//! > **G. A place to record the folds, per accepted shape.** §3.5's three bands
//! > are not passes to reproduce; they are the reason the accepted class must be
//! > *stated as a shape*, and the shape must be checkable before emission.
//! > Concretely: the port must be able to say "this `cflow-if-1` is band 3" and
//! > refuse otherwise, rather than emitting a branch and being wrong on 6 of 7
//! > leaf bodies.
//!
//! # What this is, in one paragraph
//!
//! [`FoldShape`] is the four facts `CFG_SHAPE.md` §3.5's band rules are *stated
//! over* — where each arm ends, whether both arms are constants, and whether the
//! relation is an equality — and [`FoldShape::band`] is those rules read back as
//! a decision. [`FoldShape::admit`] is the gate: a class **states the band it was
//! drawn in**, and a shape that does not decide to exactly that band is refused
//! before a byte is emitted. Nothing here emits anything, chooses anything, or
//! reproduces a fold; §3.5's own banner declines the band-1 ↔ band-2 rule and so
//! does this module, by carrying the undecided region as a **value**
//! ([`BandVerdict::BranchlessOrConditionalReturn`]) rather than by picking a
//! side.
//!
//! # Why "band 3 or refuse" is not the rule, and "the band this class was drawn
//! in, or refuse" is
//!
//! Item G's own sentence is written for `cflow-if-1`, where band 3 is the
//! interesting answer. But its *title* is **"per accepted shape"**, and this
//! crate ships two byte-exact classes in **two different bands**:
//!
//! | class | band | witness |
//! |---|---|---|
//! | [`super::cond_tail`] (`?MemFree`) | **3**, a real forward `bc` | §4.1's thirty-six published bytes |
//! | [`super::pool_free_list`] (`?Pool::Alloc`, `?Pool::Free`) | **2**, `bclr 12,26` | §3.5's own last two `bclr` rows |
//!
//! A module-wide "band 3 or refuse" would refuse a shipped, graded class. So the
//! band is a **parameter of the check** and not a constant of it, which is also
//! what makes the predicate shared rather than a tautology at one call site: the
//! same rules have to come out `Branch` for one client and `ConditionalReturn`
//! for the other, off different inputs.
//!
//! # The form: a decidable pre-emission predicate (`CFG_SHAPE.md` §6.3, amended)
//!
//! §6.3's 2026-08-13 amendment restates the emission gate as *"a decidable
//! pre-emission predicate: emit only where every byte is determined by a rule the
//! port can state and check. A general lowering behind a checked predicate
//! satisfies this rule; a hand-enumerated catalogue of named function shapes is
//! one implementation of it, not the rule itself"* — and names item G as having
//! *"already been written this way"*. This module is that form, and the thing
//! that makes it decidable rather than fitted is the next paragraph.
//!
//! # Band 1 can be EXCLUDED and never CONFIRMED, and that is a property of §3.5's
//! own logical form
//!
//! §3.5 states band 1 as a **necessary** condition — *"Reached **only when** the
//! relation is `==`/`!=` … **and** both arms are constants **and** the constant
//! pair is cheap to build from a 0/1 or 0/−1 mask"* — and then declines the third
//! conjunct outright: *"every fitted rule I could state is consistent with the
//! eighteen rows above and none of them is tested by them"* (board **#187**,
//! OPEN). A necessary condition can be **falsified** by one failing conjunct and
//! can never be **verified** without all three. So:
//!
//! * an ordered relation, or an arm that is not a constant ⇒ **not band 1**, and
//!   with both arms at the epilogue that leaves band 2 — decided, with no cost
//!   model read;
//! * an equality with two constant arms ⇒ band 1 **or** band 2, and this port
//!   says exactly that. [`BandVerdict::BranchlessOrConditionalReturn`] is one
//!   variant on purpose: two variants would be an invitation to narrow it, and
//!   narrowing it *is* #187.
//!
//! Seven of §3.5's eighteen rows fold to no branch at all, and every one of them
//! lands in that undecided region rather than being called band 2 — which is the
//! assertion [`tests::the_eighteen_rows_of_section_3_5_classify_to_their_measured_branch_column`]
//! makes row by row.
//!
//! # This is a RE-EXPRESSION, and its whole success criterion is zero moved bytes
//!
//! Built as a construct rung (`docs/rungs/README.md` § "Lane kinds"; precedent
//! board **#290**, then **#3072** for item A and **#3078** for item E) by taking
//! two classes that were **already byte-exact against real `c2.dll`** and making
//! each *state* its band through this predicate. The lane converts **zero** TUs
//! by design; a conversion would have meant behaviour moved.
//!
//! # Where this fact used to live: five times in English, zero times in a type
//!
//! Checked before the first name here was chosen, and recorded because *"two
//! encodings of one fact"* is the shape `docs/GAPS.md` §6 keeps recording:
//!
//! * `c2_il::func::body::shapes::cond_tail`'s header — *"drawn to sit inside
//!   band 3 by construction … a **band predicate spelled as a syntactic one**,
//!   which is what §6.2 item G asks for"*. That is the closest prior art there
//!   is, and it is prose in the **parser**;
//! * `c2_il::func::body::shapes::pool_free_list`'s header — band 2, and *"band 1
//!   is unreachable by the class's own precondition"* (#2596);
//! * [`super::cond_tail`]'s header — *"a shape where one arm falls through to the
//!   epilogue is fold band 2 and is out of class"*;
//! * [`super::pool_free_list`]'s header — *"the guard is fold band 2"*, and its
//!   `/Ox` refusal, which names band 3 in a sentence;
//! * [`super::block_ir::Terminator::Bclr`]'s doc — *"this is §3.5's fold
//!   band 2"*.
//!
//! **None of the five is executable**, so this module narrows nothing and shadows
//! nothing: at base `git grep -i 'foldband\|fold_band\|band('` over `crates/`
//! returned nothing at all. The two `c2_il` headers are **not** edited — that
//! crate is another lane's — and the emitter-side check is a second *asking* of
//! one rule rather than a second *statement* of it, which is the precedent
//! [`super::pool_free_list`]'s mode clause already sets (*"restated here even
//! though the recognizer already asked it"*, boards #1638/#1710).
//!
//! # What it deliberately does NOT carry (`CFG_SHAPE.md` §6.3)
//!
//! No cost model — the point of the module is that the one cost model in §3.5 is
//! *named as undecided* instead of fitted. No code motion, no loop rotation, no
//! CTR-loop discovery, no instruction scheduling, no neutrality classifier. It
//! does not reproduce a fold: nothing here can emit band 1's arithmetic select or
//! band 2's `bclr`. It is a **predicate over a shape**, and the shape is
//! described by its caller.

use c2_il::Rel;

use super::cond_tail::branch_sense;
use super::encode::CR_BIT_EQ;
use super::select::out_of_class;
use crate::BackendError;

/// **The three fold bands `CFG_SHAPE.md` §3.5 measured**, and no fourth.
///
/// This is the *record* item G asks for: the bands as a thing the port can name,
/// rather than as a sentence in five module headers. A class names the band it
/// was drawn in; [`FoldShape::admit`] is where the naming is checked.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FoldBand {
    /// **Band 1** — no branch at all: a branchless arithmetic select.
    /// `?a_eq`'s `subf ; cntlzw ; rlwinm ; xori ; addi`.
    ///
    /// **Nothing in this crate emits one**, and [`FoldShape::band`] never
    /// returns it — see the module header on why §3.5's own statement of band 1
    /// can be falsified and not verified. It is spelled because the *record* has
    /// three bands and because [`BandVerdict::BranchlessOrConditionalReturn`]
    /// names it; deleting it would make the undecided pair unspeakable.
    Branchless,
    /// **Band 2** — a conditional return, `bclr`: no label, no displacement.
    /// §3.5's majority band, and [`super::pool_free_list`]'s.
    ConditionalReturn,
    /// **Band 3** — a real forward `bc` with a displacement.
    /// [`super::cond_tail`]'s, and the band §5.1 draws the accepted class inside.
    Branch,
}

impl FoldBand {
    /// The band's number as `CFG_SHAPE.md` §3.5 numbers it. Diagnostic, and the
    /// reason a refusal can be read against the document without a translation
    /// table.
    pub fn number(self) -> u8 {
        match self {
            FoldBand::Branchless => 1,
            FoldBand::ConditionalReturn => 2,
            FoldBand::Branch => 3,
        }
    }

    /// What the band emits, in §3.5's own words.
    pub fn what(self) -> &'static str {
        match self {
            FoldBand::Branchless => "a branchless arithmetic select, no branch at all",
            FoldBand::ConditionalReturn => "a conditional return, `bclr`, no label and no displacement",
            FoldBand::Branch => "a real forward `bc` with a displacement",
        }
    }
}

/// **What this port can decide** about a shape's band — which is deliberately
/// not the same type as [`FoldBand`].
///
/// Three-valued for the reason [`super::cond::CondSource`] is three-valued: the
/// two non-answers are different from each other and neither may be quietly
/// turned into a band. [`Self::BranchlessOrConditionalReturn`] is a **positive**
/// finding about a shape §3.5 measured and declined to explain;
/// [`Self::Unmeasured`] is the absence of a row.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BandVerdict {
    /// Exactly one band, from §3.5's stated rules and no fitted one.
    Is(FoldBand),
    /// **Band 1 or band 2, and which is board #187's declined c2 cost model.**
    ///
    /// Reached when both arms end at the epilogue, the relation is an equality
    /// and both arms are constants — i.e. when band 1's two *checkable*
    /// conjuncts hold and only the third, *"the constant pair is cheap to build
    /// from a 0/1 or 0/−1 mask"*, is left. §3.5's separating pair is inside this
    /// region and is the reason it exists: `?f_eq59` (`a==b → 5:9`) takes a
    /// `bclr` and `?f_eqzk` (`a==0 → 5:9`) folds to nothing, with the same
    /// constant pair.
    ///
    /// **One variant, not two.** Splitting it would be a place for a later lane
    /// to write the fitted rule #187 exists to prevent.
    BranchlessOrConditionalReturn,
    /// §3.5's eighteen-row table has **no row** for this combination of arm
    /// ends, so its band is not derivable from the document. Never read as a
    /// band, and never as [`Self::BranchlessOrConditionalReturn`] — that one is
    /// a measured region and this is an unmeasured one.
    Unmeasured,
}

/// **Where one arm of a `cflow-if-1` ends** — the fact all three of §3.5's band
/// boundaries are stated over.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ArmEnd {
    /// The arm ends in a **control transfer that is not this function's
    /// epilogue** — a tail call. §3.5 band 3's first clause, and `?MemFree`'s
    /// two arms and `?f_eqcall`'s then-arm.
    Transfer,
    /// The arm's last act is to **return**: it is, or falls into, the function's
    /// epilogue. Every band-1 and band-2 row.
    Epilogue,
    /// The arm has content and **joins** the other arm at a common continuation.
    /// §3.5 band 3's second clause, and `?a_var`'s two arms.
    Join,
}

/// **Whether both arms are constants** — band 1's second conjunct, and the only
/// one of its three that a caller can answer.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ArmValues {
    /// Both arms yield a literal: `{1,2}`, `{5,9}`, `{70,0}`, `{3,4}`.
    BothConstants,
    /// At least one arm yields something else — a loaded value (`?f_eq3`'s
    /// `return c`), a store sequence (`?a_store`, `?Pool::Free`), or a call.
    NotBothConstants,
}

/// **Whether the relation is an equality** — band 1's first conjunct.
///
/// §3.5: band 1 is *"reached only when the relation is `==`/`!=` (or reducible
/// to it)"*, and *"ordered relations (`<`, `>`) never fold — their branchless
/// bool spine is 4–6 instructions before the select, so the branch is cheaper"*.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Relation {
    /// `==` or `!=`.
    Equality,
    /// `<`, `<=`, `>`, `>=`.
    Ordered,
}

impl Relation {
    /// Read an IL relation's kind **through the existing reader**.
    ///
    /// [`branch_sense`] is this crate's one reader of *"which `(BO, bit)` pair an
    /// IL relation becomes"*, and the relation is an equality **exactly when**
    /// the branch it becomes tests the `EQ` bit: `Eq`/`Ne` → [`CR_BIT_EQ`],
    /// `Lt`/`Ge` → `CR_BIT_LT`, `Gt`/`Le` → `CR_BIT_GT`. Deriving it there rather
    /// than writing a second `match` over [`Rel`] is deliberate — a second
    /// enumeration of the six relations is a second thing to keep true, and
    /// `docs/GAPS.md` §6's recurring defect is exactly one fact with two
    /// encodings.
    pub fn of(rel: Rel) -> Self {
        if branch_sense(rel).1 == CR_BIT_EQ {
            Relation::Equality
        } else {
            Relation::Ordered
        }
    }
}

/// **A `cflow-if-1` shape, in the four facts §3.5's band rules are stated over.**
///
/// Not the body, not the IL, not the bytes — only what the bands are decided by.
/// A class describes the shape it admits and asks [`Self::admit`] whether that
/// description lands in the band the class was drawn in.
///
/// **The rules are symmetric in the two arms**, which is asserted rather than
/// assumed ([`tests::the_band_rule_is_symmetric_in_the_two_arms`]): every clause
/// in §3.5 quantifies over *"an arm"* or *"both arms"* and none of them names the
/// then-arm or the else-arm specifically, so a caller that swaps them cannot get
/// a different band.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct FoldShape {
    /// Band 1's first conjunct.
    pub rel: Relation,
    /// Where the then-arm ends.
    pub then_end: ArmEnd,
    /// Where the else-arm ends.
    pub else_end: ArmEnd,
    /// Band 1's second conjunct.
    pub values: ArmValues,
}

impl FoldShape {
    /// **The band, from §3.5's stated rules.**
    ///
    /// The clause order is load-bearing and is the one §3.5 states, not a
    /// convenience: band 3 is tested **first**, because `?a_var` is an equality
    /// with two constant arms — band 1's whole checkable precondition — and it
    /// emits a `bc`. A lowering that asked about the constants first would call
    /// it band 1 or 2 and refuse a body §3.5 measures in band 3.
    ///
    /// | clause | §3.5's words | verdict |
    /// |---|---|---|
    /// | an arm ends in a transfer that is not the epilogue | band 3's first | `Is(Branch)` |
    /// | both arms have content that joins | band 3's second | `Is(Branch)` |
    /// | one arm joins, the other returns | *no row* | `Unmeasured` |
    /// | both at the epilogue, band 1 falsified | band 2 by elimination | `Is(ConditionalReturn)` |
    /// | both at the epilogue, band 1 not falsified | the declined boundary | `BranchlessOrConditionalReturn` |
    pub fn band(self) -> BandVerdict {
        // ---- band 3, first --------------------------------------------------
        // §3.5: "Reached when neither arm can be the fall-through-plus-
        // conditional-return: because an arm ends in a transfer that is not the
        // epilogue (`MemFree`'s two tail calls, `f_eqcall`'s tail call), or
        // because both arms have content that joins (`a_var`)."
        if self.then_end == ArmEnd::Transfer || self.else_end == ArmEnd::Transfer {
            return BandVerdict::Is(FoldBand::Branch);
        }
        if self.then_end == ArmEnd::Join && self.else_end == ArmEnd::Join {
            return BandVerdict::Is(FoldBand::Branch);
        }
        // ---- one arm joins and the other returns ----------------------------
        // Band 3's clause does not fire (the returning arm *can* be the
        // conditional return) and band 2's does not either (its other successor
        // is a join block, not a short fall-through). No row in §3.5's table has
        // this shape, so the answer is that there is no answer.
        if self.then_end == ArmEnd::Join || self.else_end == ArmEnd::Join {
            return BandVerdict::Unmeasured;
        }
        // ---- both arms end at the epilogue ----------------------------------
        // Band 1 is stated as a NECESSARY condition, so it can be falsified by
        // one conjunct and never verified without the third — which is the one
        // §3.5 declines (board #187). Falsified leaves band 2, which is §3.5's
        // band-2 rule read the way §3.5 states it: "one successor IS the
        // function's epilogue and the other is short enough to fall through".
        if self.rel == Relation::Ordered || self.values == ArmValues::NotBothConstants {
            return BandVerdict::Is(FoldBand::ConditionalReturn);
        }
        BandVerdict::BranchlessOrConditionalReturn
    }

    /// **The gate**: this shape must decide to exactly `class_band`, or the body
    /// is refused before a byte of it is emitted.
    ///
    /// `class_band` is the band the *class* was drawn in and is a property of the
    /// lowering, not of the body — [`super::cond_tail`] is band 3 and
    /// [`super::pool_free_list`] is band 2, and each says so at its own call
    /// site. Item G's *"say this `cflow-if-1` is band 3 and refuse otherwise"* is
    /// the `FoldBand::Branch` instance of this.
    ///
    /// Both non-answers refuse, with **different** messages, and neither is ever
    /// narrowed to a band: refusing on the declined pair is the whole point, and
    /// refusing on an unmeasured combination is the absence-is-not-evidence rule
    /// this crate applies to every decoder it owns.
    pub fn admit(self, class_band: FoldBand, site: &str) -> Result<(), BackendError> {
        match self.band() {
            BandVerdict::Is(b) if b == class_band => Ok(()),
            BandVerdict::Is(b) => Err(out_of_class(&format!(
                "{site}: this shape is CFG_SHAPE.md §3.5 fold band {} ({}), and the \
                 class was drawn in band {} ({}) — so the class's bytes are the wrong \
                 bytes for it, which is §3.5's headline defect: 6 of 7 leaf `cflow-if-1` \
                 bodies emit no branch at all",
                b.number(),
                b.what(),
                class_band.number(),
                class_band.what(),
            ))),
            BandVerdict::BranchlessOrConditionalReturn => Err(out_of_class(&format!(
                "{site}: both arms end at the epilogue, the relation is an equality and \
                 both arms are constants, so this shape is CFG_SHAPE.md §3.5 fold band 1 \
                 OR band 2 and WHICH is board #187's declined c2 cost model — eighteen \
                 rows fitted by every cell and tested by none. `?f_eq59` (a==b -> 5:9) \
                 takes a `bclr` and `?f_eqzk` (a==0 -> 5:9) folds to nothing. Refused \
                 rather than guessed"
            ))),
            BandVerdict::Unmeasured => Err(out_of_class(&format!(
                "{site}: one arm joins and the other ends at the epilogue — CFG_SHAPE.md \
                 §3.5's eighteen-row table has no row for that combination, so this \
                 shape's fold band is unmeasured rather than derivable, and an unmeasured \
                 band is not a band"
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// What §3.5's table's last column says the obj carries.
    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    enum Measured {
        /// "**none**" — no branch at all, band 1.
        NoBranch,
        /// "**`bclr`**" — band 2.
        Bclr,
        /// "**`bc`**" — band 3.
        Bc,
    }

    fn shape(rel: Relation, then_end: ArmEnd, else_end: ArmEnd, values: ArmValues) -> FoldShape {
        FoldShape { rel, then_end, else_end, values }
    }

    /// `docs/CFG_SHAPE.md` §3.5's eighteen rows, transcribed: the shape each row
    /// describes, the branch its **obj** carries, and the verdict this model
    /// gives.
    #[rustfmt::skip]
    fn section_3_5_table() -> Vec<(&'static str, FoldShape, Measured, BandVerdict)> {
        use ArmEnd::{Epilogue as E, Join as J, Transfer as T};
        use ArmValues::{BothConstants as K, NotBothConstants as V};
        // Local consts rather than `use Relation::Equality as Eq`: `Eq` and
        // `Ord` are prelude trait names and shadowing them inside a function
        // that also derives them is a needless hazard.
        const EQU: Relation = Relation::Equality;
        const ORD: Relation = Relation::Ordered;
        let pair = BandVerdict::BranchlessOrConditionalReturn;
        let band2 = BandVerdict::Is(FoldBand::ConditionalReturn);
        let band3 = BandVerdict::Is(FoldBand::Branch);
        vec![
            // body            rel  then else values      obj          verdict
            ("?a_eq",     shape(EQU,  E,   E,   K), Measured::NoBranch, pair),
            ("?a_ne",     shape(EQU,  E,   E,   K), Measured::NoBranch, pair),
            ("?a_eqk",    shape(EQU,  E,   E,   K), Measured::NoBranch, pair),
            ("?a_else",   shape(EQU,  E,   E,   K), Measured::NoBranch, pair),
            ("?d_early",  shape(EQU,  E,   E,   K), Measured::NoBranch, pair),
            ("?f_eqzk",   shape(EQU,  E,   E,   K), Measured::NoBranch, pair),
            ("?d_switch", shape(EQU,  E,   E,   K), Measured::NoBranch, pair),
            ("?a_lt",     shape(ORD,  E,   E,   K), Measured::Bclr,     band2),
            ("?f_eq59",   shape(EQU,  E,   E,   K), Measured::Bclr,     pair),
            ("?f_gt59",   shape(ORD,  E,   E,   K), Measured::Bclr,     band2),
            ("?f_eq3",    shape(EQU,  E,   E,   V), Measured::Bclr,     band2),
            ("?a_store",  shape(EQU,  E,   E,   V), Measured::Bclr,     band2),
            ("?f_eqvoid", shape(EQU,  E,   E,   V), Measured::Bclr,     band2),
            ("?Pool::Alloc", shape(EQU, E, E,   V), Measured::Bclr,     band2),
            ("?Pool::Free",  shape(EQU, E, E,   V), Measured::Bclr,     band2),
            ("?a_var",    shape(EQU,  J,   J,   K), Measured::Bc,       band3),
            ("?f_eqcall", shape(EQU,  T,   E,   V), Measured::Bc,       band3),
            ("?MemFree",  shape(EQU,  T,   T,   V), Measured::Bc,       band3),
        ]
    }

    /// **The known-answer control, from the obj column of a published table.**
    ///
    /// Every one of `CFG_SHAPE.md` §3.5's eighteen rows, classified by the rules
    /// this module reads off §3.5's prose, checked against what §3.5 read off the
    /// **objs**. The counts are asserted so that deleting a row fails here rather
    /// than quietly narrowing what "graded" means.
    ///
    /// The safety property is the second half and it is the one item G exists
    /// for: **a row is called band 3 exactly when its obj carries a `bc`.** A
    /// model that called `?a_eq` band 3 would emit a branch for a body that has
    /// none, which is §3.5's headline and board #186's *"6 of 7"*.
    #[test]
    fn the_eighteen_rows_of_section_3_5_classify_to_their_measured_branch_column() {
        let table = section_3_5_table();
        assert_eq!(table.len(), 18, "eighteen rows graded, and the count is the assertion");
        let (mut band3, mut band2, mut undecided) = (0, 0, 0);
        for (name, shape, measured, want) in &table {
            assert_eq!(shape.band(), *want, "{name}");
            match shape.band() {
                BandVerdict::Is(FoldBand::Branch) => band3 += 1,
                BandVerdict::Is(FoldBand::ConditionalReturn) => band2 += 1,
                BandVerdict::BranchlessOrConditionalReturn => undecided += 1,
                v => panic!("{name}: {v:?}"),
            }
            // The safety property, both ways round.
            assert_eq!(
                shape.band() == BandVerdict::Is(FoldBand::Branch),
                *measured == Measured::Bc,
                "{name}: band 3 iff the obj carries a `bc`"
            );
            // …and a row whose obj carries NO branch is never called band 2 on
            // its own: that decision is #187's and this model does not make it.
            if *measured == Measured::NoBranch {
                assert_eq!(shape.band(), BandVerdict::BranchlessOrConditionalReturn, "{name}");
            }
        }
        assert_eq!((band3, band2, undecided), (3, 7, 8));
        assert_eq!(band3 + band2 + undecided, 18);
    }

    /// **Band 1 is never confirmed.** §3.5 states it as a necessary condition
    /// whose third conjunct is declined, so `Is(Branchless)` is a verdict this
    /// model cannot produce — asserted over all eighteen rows and over the
    /// most-band-1-looking shape constructible.
    #[test]
    fn band_1_is_excluded_and_never_confirmed() {
        for (name, shape, ..) in section_3_5_table() {
            assert_ne!(shape.band(), BandVerdict::Is(FoldBand::Branchless), "{name}");
        }
        let most_band_1_looking = shape(
            Relation::Equality,
            ArmEnd::Epilogue,
            ArmEnd::Epilogue,
            ArmValues::BothConstants,
        );
        assert_eq!(
            most_band_1_looking.band(),
            BandVerdict::BranchlessOrConditionalReturn
        );
        // One conjunct falsified either way excludes band 1 and leaves band 2 —
        // decided, with no cost model read.
        assert_eq!(
            FoldShape { rel: Relation::Ordered, ..most_band_1_looking }.band(),
            BandVerdict::Is(FoldBand::ConditionalReturn)
        );
        assert_eq!(
            FoldShape { values: ArmValues::NotBothConstants, ..most_band_1_looking }.band(),
            BandVerdict::Is(FoldBand::ConditionalReturn)
        );
    }

    /// **The declined region refuses against BOTH bands, and its message names
    /// the board row.** A gate that admitted it for band 2 would be shipping
    /// #187's fitted rule with the fitting left out.
    #[test]
    fn the_declined_cost_model_refuses_whichever_band_is_asked_for() {
        let undecided = shape(
            Relation::Equality,
            ArmEnd::Epilogue,
            ArmEnd::Epilogue,
            ArmValues::BothConstants,
        );
        for band in [FoldBand::Branchless, FoldBand::ConditionalReturn, FoldBand::Branch] {
            let s = format!("{:?}", undecided.admit(band, "a probe").unwrap_err());
            assert!(s.contains("#187"), "{s}");
            assert!(s.contains("band 1 OR band 2"), "{s}");
            assert!(s.contains("a probe"), "{s}");
        }
    }

    /// **An unmeasured combination is refused as unmeasured**, and is not the
    /// declined pair — the two non-answers are kept apart, which is the same
    /// discipline `CondSource::Unknown` vs `NotInThisBlock` records.
    #[test]
    fn a_join_against_an_epilogue_is_unmeasured_and_is_not_the_declined_pair() {
        let mixed = shape(
            Relation::Equality,
            ArmEnd::Join,
            ArmEnd::Epilogue,
            ArmValues::BothConstants,
        );
        assert_eq!(mixed.band(), BandVerdict::Unmeasured);
        assert_ne!(mixed.band(), BandVerdict::BranchlessOrConditionalReturn);
        let s = format!("{:?}", mixed.admit(FoldBand::Branch, "a probe").unwrap_err());
        assert!(s.contains("no row"), "{s}");
        assert!(s.contains("unmeasured"), "{s}");
        // …but a join against a TRANSFER is band 3, by §3.5's first clause read
        // literally: an arm ends in a transfer that is not the epilogue.
        assert_eq!(
            FoldShape { else_end: ArmEnd::Transfer, ..mixed }.band(),
            BandVerdict::Is(FoldBand::Branch)
        );
    }

    /// **Every clause in §3.5 quantifies over "an arm" or "both arms", so the
    /// rules are symmetric** — asserted over all eighteen rows with the arms
    /// swapped, because a caller that fills the struct the other way round must
    /// not get a different band.
    #[test]
    fn the_band_rule_is_symmetric_in_the_two_arms() {
        let table = section_3_5_table();
        assert_eq!(table.len(), 18);
        for (name, shape, _, _) in &table {
            let swapped =
                FoldShape { then_end: shape.else_end, else_end: shape.then_end, ..*shape };
            assert_eq!(swapped.band(), shape.band(), "{name} swapped");
        }
        // Including the asymmetric-looking rows, positively: `?f_eqcall` is
        // (Transfer, Epilogue) and reads band 3 either way round.
        let f_eqcall = shape(
            Relation::Equality,
            ArmEnd::Transfer,
            ArmEnd::Epilogue,
            ArmValues::NotBothConstants,
        );
        assert_eq!(f_eqcall.band(), BandVerdict::Is(FoldBand::Branch));
        assert_eq!(
            FoldShape { then_end: ArmEnd::Epilogue, else_end: ArmEnd::Transfer, ..f_eqcall }.band(),
            BandVerdict::Is(FoldBand::Branch)
        );
    }

    /// **The equality clause is read through `branch_sense` and not restated.**
    /// All six IL relations, and the split is exactly `Eq`/`Ne` — which is the
    /// same table `cond_tail::the_branch_sense_negates_every_relation` pins and
    /// `the_relation_grid_matches_the_real_obj_bytes` grades against real obj
    /// bytes.
    #[test]
    fn the_equality_clause_comes_from_the_existing_relation_reader() {
        let cells = [
            (Rel::Eq, Relation::Equality),
            (Rel::Ne, Relation::Equality),
            (Rel::Lt, Relation::Ordered),
            (Rel::Ge, Relation::Ordered),
            (Rel::Gt, Relation::Ordered),
            (Rel::Le, Relation::Ordered),
        ];
        assert_eq!(cells.len(), 6, "six relations graded, and the count is the assertion");
        for (rel, want) in cells {
            assert_eq!(Relation::of(rel), want, "{rel:?}");
        }
        // …and the two ordered relations §3.5 names by hand are ordered: "?a_lt"
        // and "?f_gt59" are the rows that make the clause do work.
        assert_eq!(Relation::of(Rel::Lt), Relation::Ordered);
        assert_eq!(Relation::of(Rel::Gt), Relation::Ordered);
    }

    /// **The gate refuses a shape that decides to a different band, and names
    /// both.** Band 3's shape offered by a band-2 class and the other way round.
    #[test]
    fn a_shape_in_another_band_is_refused_and_the_refusal_names_both_bands() {
        let band3 = shape(
            Relation::Equality,
            ArmEnd::Transfer,
            ArmEnd::Transfer,
            ArmValues::NotBothConstants,
        );
        let band2 = shape(
            Relation::Ordered,
            ArmEnd::Epilogue,
            ArmEnd::Epilogue,
            ArmValues::BothConstants,
        );
        let s = format!("{:?}", band3.admit(FoldBand::ConditionalReturn, "a band-2 class").unwrap_err());
        assert!(s.contains("fold band 3"), "{s}");
        assert!(s.contains("band 2"), "{s}");
        assert!(s.contains("a band-2 class"), "{s}");
        let s2 = format!("{:?}", band2.admit(FoldBand::Branch, "a band-3 class").unwrap_err());
        assert!(s2.contains("fold band 2"), "{s2}");
        assert!(s2.contains("band 3"), "{s2}");
        // …and each is admitted by its own band, positively.
        assert!(band3.admit(FoldBand::Branch, "x").is_ok());
        assert!(band2.admit(FoldBand::ConditionalReturn, "x").is_ok());
    }

    /// **The two shipped clients sit in two different bands, off the same
    /// rules.** This is what makes the predicate a shared one and not a
    /// tautology at one call site: `cond_tail`'s shape must come out band 3 and
    /// `pool_free_list`'s band 2, and each must be REFUSED by the other's band.
    #[test]
    fn the_two_shipped_clients_decide_to_two_different_bands() {
        // `?MemFree`: both arms end in a tail `b` to a different external.
        let cond_tail = shape(
            Relation::Equality,
            ArmEnd::Transfer,
            ArmEnd::Transfer,
            ArmValues::NotBothConstants,
        );
        // `?Pool::Alloc` / `?Pool::Free`: the guarded arm IS the epilogue jump
        // and the fall-out arm is a store sequence ending at the same epilogue.
        let pool = shape(
            Relation::Equality,
            ArmEnd::Epilogue,
            ArmEnd::Epilogue,
            ArmValues::NotBothConstants,
        );
        assert_eq!(cond_tail.band(), BandVerdict::Is(FoldBand::Branch));
        assert_eq!(pool.band(), BandVerdict::Is(FoldBand::ConditionalReturn));
        assert_ne!(cond_tail.band(), pool.band());
        assert!(cond_tail.admit(FoldBand::Branch, "x").is_ok());
        assert!(pool.admit(FoldBand::ConditionalReturn, "x").is_ok());
        assert!(cond_tail.admit(FoldBand::ConditionalReturn, "x").is_err());
        assert!(pool.admit(FoldBand::Branch, "x").is_err());
        // **`pool` decides, and does NOT land in #187's declined region** — that
        // is #2596's claim ("band 1 is unreachable by the class's own
        // precondition") as a check rather than as a paragraph.
        assert_ne!(pool.band(), BandVerdict::BranchlessOrConditionalReturn);
    }

    /// **Band 3's clause dominates band 1's precondition**, and the clause order
    /// is why. `?a_var` is an equality with two constant arms — band 1's entire
    /// checkable precondition — and its obj carries a `bc`.
    #[test]
    fn the_join_clause_beats_band_1s_precondition_which_is_the_clause_order() {
        let a_var = shape(
            Relation::Equality,
            ArmEnd::Join,
            ArmEnd::Join,
            ArmValues::BothConstants,
        );
        assert_eq!(a_var.band(), BandVerdict::Is(FoldBand::Branch));
        // The same body with its arms returning instead of joining is the
        // undecided region — so the join is doing the work, not the constants.
        assert_eq!(
            FoldShape { then_end: ArmEnd::Epilogue, else_end: ArmEnd::Epilogue, ..a_var }.band(),
            BandVerdict::BranchlessOrConditionalReturn
        );
    }

    /// The record itself: three bands, numbered as §3.5 numbers them, each with
    /// its own name. Written as a count so that adding a fourth band fails here
    /// rather than in a refusal message nobody reads.
    #[test]
    fn the_record_carries_three_bands_numbered_as_the_document_numbers_them() {
        let bands = [FoldBand::Branchless, FoldBand::ConditionalReturn, FoldBand::Branch];
        assert_eq!(bands.len(), 3);
        assert_eq!(bands.map(|b| b.number()), [1, 2, 3]);
        assert!(bands[0].what().contains("no branch at all"));
        assert!(bands[1].what().contains("`bclr`"));
        assert!(bands[2].what().contains("`bc`"));
        // The three names are distinct, which a `what()` copied by hand would
        // not guarantee.
        assert_ne!(bands[0].what(), bands[1].what());
        assert_ne!(bands[1].what(), bands[2].what());
    }

    /// **Band 1's two conjuncts are inert once an arm transfers**, and this
    /// states it positively rather than leaving it to be inferred: neither the
    /// relation nor the constant-ness can move a transfer shape off band 3. It
    /// is why `cond_tail` may describe its arms' values without that description
    /// being load-bearing.
    #[test]
    fn band_1s_conjuncts_cannot_move_a_transfer_shape() {
        let mut graded = 0;
        for rel in [Relation::Equality, Relation::Ordered] {
            for values in [ArmValues::BothConstants, ArmValues::NotBothConstants] {
                let s = shape(rel, ArmEnd::Transfer, ArmEnd::Transfer, values);
                assert_eq!(s.band(), BandVerdict::Is(FoldBand::Branch), "{rel:?} {values:?}");
                graded += 1;
            }
        }
        assert_eq!(graded, 4, "four cells graded, not zero");
    }
}
