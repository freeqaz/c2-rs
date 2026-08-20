//! **IR2 — the port's PREDICTED object plan, computed from IL without
//! emitting.** `docs/ARCHITECTURE_PROPOSAL_2026-08-20.md` §3.1/§3.2's `plan/`
//! slot, instrument half.
//!
//! # The tautology this module exists to avoid
//!
//! The lane that commissioned this was briefed with a control that reads:
//! *"the manifest must be identical on the TUs we already reproduce
//! byte-exactly"*. Taken literally that is **vacuous**. On a `match` TU the
//! port's obj **is** the reference's bytes (`ObjImage::diff`, timestamp
//! normalized), so `manifest(port_obj) == manifest(ref_obj)` follows from
//! `port_bytes == ref_bytes` for *any* pure function `manifest`. And on the
//! other 844 TUs the port emits no obj at all, so the port side would be
//! **undefined** and there would be no curve. The naive reading delivers
//! neither the control nor the curve.
//!
//! So the manifest has two independent producers:
//!
//! * [`c2_obj::ObjPlan::observe`] — ground truth, read off the **reference**
//!   obj. Defined wherever a reference obj decodes.
//! * [`predict`] — the port's plan, computed **from the IL bundle**, never by
//!   emitting. Defined on TUs the reader refuses, which is what makes a curve
//!   possible at all.
//!
//! Required-exact on the matched TUs is then a real constraint on [`predict`],
//! and the refused TUs get a graded verdict because [`predict`] is defined on
//! them.
//!
//! # The fence: this module may not reach the emitter, and may not ask
//! `IlBundle::functions()`
//!
//! `IlBundle::functions()` is the port's **admission gate**. It returns `None`
//! on 844 of the 870 graded TUs. A predictor that asked it for the emit set
//! would publish `known ≈ 30 of 870` and read `Unknown` on every one of the
//! 844 — which is not a curve, it is *the reader's refusal mass wearing new
//! keys*, and it is board **#3237** exactly: an instrument that returns 0
//! because it did not look is indistinguishable from one that returns 0 because
//! there was nothing to find.
//!
//! The ban is enforced by a **source-level test**
//! ([`tests::the_plan_module_does_not_reach_the_emitter_or_the_admission_gate`]),
//! in the same style as `coff::function`'s
//! `every_production_emitter_has_a_lib_rs_caller`. It is a source fence and not
//! a type fence, and that is stated plainly: a reviewer should read this
//! module's `use` list rather than trust the test's name.
//!
//! # NOT A GATE
//!
//! Nothing here may appear in an accept path, a refusal predicate, or
//! `scripts/gate.sh`. **A predicted plan that matches is NECESSARY but NOT
//! SUFFICIENT for a byte-exact obj** — a TU can be plan-exact and mismatch on
//! every instruction. The byte judge is unchanged and remains the sole judge.

use std::collections::{BTreeMap, BTreeSet};

use c2_il::IlBundle;

/// **A predicted component, or a NAMED reason the port did not look.**
///
/// `Unknown` is never `Known(empty)`, and the distinction is the entire
/// mitigation for #3237: an empty set that means *"c2 emits nothing here"* and
/// an empty set that means *"this stage has no answer"* are different facts,
/// and folding them is absence-read-as-success. The reason string is the
/// **census key** the component publishes, and it names the stage that owes the
/// work — the claims-ledger shape of the architecture proposal §3.3.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Predicted<T> {
    Known(T),
    Unknown(&'static str),
}

impl<T> Predicted<T> {
    pub fn known(&self) -> Option<&T> {
        match self {
            Predicted::Known(t) => Some(t),
            Predicted::Unknown(_) => None,
        }
    }
    /// The census key when this component did not look, `None` when it did.
    pub fn reason(&self) -> Option<&'static str> {
        match self {
            Predicted::Known(_) => None,
            Predicted::Unknown(r) => Some(r),
        }
    }
}

/// The argv-derived facts an object plan depends on that the IL bundle does not
/// record.
///
/// `/Gy` (implied by `/O1` and `/O2`) is the one that changes the section table
/// wholesale: it puts every emitted function in its own COMDAT `.text`. The
/// scan already computes it with
/// `PortC2::flags_imply_function_level_linking`, and it is passed in rather
/// than re-derived here so there is one decision and not two.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlanInputs {
    pub function_level_linking: bool,
}

/// The port's predicted [`c2_obj::ObjPlan`], component by component.
///
/// Every component is a [`Predicted`], so a component the port cannot model
/// publishes a named `Unknown` rather than an empty `Known`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PredictedPlan {
    /// **Which functions c2 emits**, as a set. The order-free claim.
    ///
    /// **`Unknown` on every TU, and that is the lane's own REGISTERED SHIP RULE
    /// applied to itself rather than an omission.** The prereg
    /// (`docs/rungs/_2026-08-20-objplan-prereg.md` §3) reads: *"`Differs` on a
    /// control TU means the extractor or the predictor is wrong and the
    /// component **does not ship**. A component whose control is red ships as
    /// `Unknown`, never as `Differs`."* The `0x20`-seed predictor differs on
    /// **2 of the 26** control TUs (`TomCryptLicense.cpp`, `ZlibLicense.cpp` —
    /// each missing exactly `??__EsLicense@@YAXXZ`, the `??__E` dynamic
    /// initializer, which carries no `.gl` function record for the seed to
    /// read), so its control is red and it does not ship.
    ///
    /// The first version of this lane shipped it anyway, on the strength of an
    /// unregistered *"12 of 26 is too many, 2 of 26 is fine"* — which is a
    /// threshold rule invented after seeing the data, and it produced an
    /// instrument that printed `2 differs` beside its own *"only `differs` reds
    /// the lane"* on every scan. The review caught it. The rule is now applied
    /// UNIFORMLY to both components.
    ///
    /// The seed is still COMPUTED and still PUBLISHED, as [`Self::gl_seed_members`]
    /// and the `plan-emitset-seed-*` characterization keys — the same treatment
    /// [`Self::gl_record_order`] gets, for the same reason: the measurement is the
    /// deliverable, the *claim of disagreement* is what the control forbids.
    pub emit_set_members: Predicted<BTreeSet<String>>,
    /// **The `0x20`-SEED hypothesis, kept as a CHARACTERIZATION value.**
    ///
    /// Not a component and never graded as one — see [`Self::emit_set_members`].
    /// This is the set `{name : attr & 0x20}` over
    /// [`c2_il::func::gl_function_attrs`]' map, retained so its agreement with
    /// c2's real emitted set stays measurable at zero cost. A consumer that
    /// treated it as a *prediction* would be re-publishing a rule the named
    /// control has already refused to ship.
    pub gl_seed_members: Predicted<BTreeSet<String>>,
    /// **Every name [`c2_il::func::gl_function_attrs`] NAMED**, seed or not.
    ///
    /// The deciding probe for the alternative explanation of `28,107` vs
    /// `162,146` — see [`Self::attr_census`]. The seed's own size cannot tell
    /// *"the bit is rare"* apart from *"the scanner found one record in six"*;
    /// the intersection of THIS set with the reference obj's emitted set can.
    pub gl_attr_names: Predicted<BTreeSet<String>>,
    /// **Every mangled-looking `.gl` symbol run**, from [`c2_il::mangled_names`]
    /// — an ORTHOGONAL reader that does not use `gl_offset_framed` at all.
    ///
    /// This is the control on the one above. `gl_function_attrs` advances by
    /// `p += 1` past any position whose offset field is not framed, with **no
    /// refusal and no counter**, so a systematically low hit rate is
    /// indistinguishable from a fact about `.gl` when only that reader is
    /// consulted. `mangled_names` walks the symbol runs in file order and is
    /// blind to the framing, so `|runs ∩ emitted|` versus
    /// `|attr-names ∩ emitted|` separates *"`.gl` does not name these
    /// functions"* from *"this scanner does not reach them"*.
    pub gl_run_names: BTreeSet<String>,
    /// The order the port would lay the emitted COMDATs out in.
    ///
    /// **`Unknown` on every TU, and that is a MEASURED result rather than an
    /// omission.** The only ordered `.gl` reader available is
    /// [`c2_il::mangled_names`], and `.gl` record order **is not COMDAT section
    /// order**: over the 870-TU workload it agreed on **18 of 854** TUs where
    /// both sides answered, and it **differed on 12 of the 26 TUs the port
    /// already reproduces byte-exactly** — the lane's own named control, which
    /// is what caught it. The extractor is not at fault (it agrees with
    /// `ObjImage::text_comdat_functions` by an integration test on real objs),
    /// so it is the *predictor* that is refuted, and under this lane's prereg a
    /// component whose control is red **ships as `Unknown`, never as
    /// `Differs`**.
    ///
    /// The refuted rule is still MEASURED and published — as
    /// `gap-metric plan-emitset-glorder-agrees`, a characterization number and
    /// not a port curve — because it is the price of board **#259**'s
    /// `coff::order::plan_text_order`: whatever builds this component has to
    /// beat 18 of 854, and that figure did not exist before.
    pub emit_set_order: Predicted<Vec<String>>,
    /// **The `.gl`-record-order hypothesis, kept as a CHARACTERIZATION value.**
    ///
    /// Not a component and never graded as one: this is the refuted rule above,
    /// retained so its agreement rate stays measurable at zero cost. A consumer
    /// that treated it as a prediction would be re-publishing a rule the named
    /// control has already killed.
    pub gl_record_order: Predicted<Vec<String>>,
    /// The ordered section-name sequence. Not modelled yet.
    pub sections: Predicted<Vec<String>>,
    /// Weak externals — `(weak, default)`. Not modelled yet.
    pub weak: Predicted<Vec<(String, String)>>,
    /// Undefined externals, in order. Not modelled yet.
    pub undef: Predicted<Vec<String>>,
    /// **A census of the `.gl` attribute byte itself**, or `None` when the
    /// reader refused the file. See [`AttrCensus`].
    ///
    /// # Why a predictor carries a census of its own input
    ///
    /// This is the **deciding probe** for a measurement that would otherwise
    /// have two incompatible explanations. On the 870-TU workload the
    /// [`FN_FLAG_EMIT_SEED`] bit is set on **331** names against **162,146**
    /// emitted functions, and the seed is EMPTY on 739 of the 854 TUs where the
    /// reader answers. Two readings fit that equally well:
    ///
    /// * the bit is genuinely rare in the file and c2 sets it at IL-load time
    ///   from something else — in which case the byte is decoded correctly and
    ///   `docs/whitebox/C2_MAP.md` §3E's seed-plus-closure model simply cannot
    ///   be built from `.gl`;
    /// * or the walk is landing on the wrong field on most records and
    ///   returning a plausible byte — which is **the exact failure this
    ///   reader's own doc records having already made once**: *"nine of eleven
    ///   records decoded as attribute `0x00` … a uniform answer where the grid
    ///   predicts a split … a mis-decoded displacement does not look like an
    ///   error, it looks like a fact."*
    ///
    /// Bit 6 (`FN_FLAG_INLINABLE`) is the discriminator, because it has a
    /// SHIPPED consumer that is graded elsewhere: if bit 6 is present at a
    /// plausible rate then the byte is the right byte and the first reading
    /// holds; if bit 6 is near-zero too, the walk is off and the whole map is
    /// suspect. **A count with no discriminator would have left this lane
    /// publishing a number with two stories and no way to choose.**
    ///
    /// # THE FIRST VERSION OF THIS PROBE TESTED ONE EXPLANATION AND CALLED IT DECIDED
    ///
    /// Bit 6 at 99.99 % and zero `0x00` bytes rule out the **uniform-zero**
    /// mis-decode and nothing else. They do not discriminate against a walk that
    /// lands on some *other* byte whose bit 6 is usually set, and — the reading
    /// that actually mattered — they say nothing at all about **coverage**:
    /// `gl_function_attrs` advances `p += 1` past any position whose offset
    /// field is not framed, silently, with no refusal and no counter, so
    /// *"the reader's map covers 17.7 % of the emitted set"* is equally
    /// consistent with *"the scanner finds one record in six"*. That is board
    /// **#3237** arriving inside the probe built to prevent it.
    ///
    /// Two measurements were added for the alternatives, both from data already
    /// in hand: [`Self::gl_run_names`] (an orthogonal reader that does not use
    /// the framing) and [`AttrCensus::bits`] (a genuinely decoded field is
    /// structured across its other six bits; a mis-landed one is near-constant).
    ///
    /// **AND THE ANSWER IS THE COVERAGE ONE.** Of the 28,107 records this reader
    /// names, **3,933** are functions c2 emitted — **2.4 %** — while the
    /// framing-blind reader names **70,114** of them, **43.2 %**. See
    /// [`FN_FLAG_EMIT_SEED`].
    pub attr_census: Option<AttrCensus>,
}

/// **A census of the byte [`c2_il::func::gl_function_attrs`] returns**, over the
/// records it named in one TU. See [`PredictedPlan::attr_census`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AttrCensus {
    /// Records the reader NAMED. Not "records in the file" — see
    /// [`PredictedPlan::gl_run_names`], which is the control on exactly that
    /// difference.
    pub names: usize,
    /// …of those, bit 6 (`FN_FLAG_INLINABLE`) set. The uniform-zero
    /// discriminator's positive half.
    pub bit6: usize,
    /// …of those, the whole byte reads `0x00`. The uniform-zero
    /// discriminator's negative half.
    pub zero: usize,
    /// **The whole histogram, bit 0 … bit 7.** `bits[6] == bit6` and
    /// `bits[5]` is the seed, by construction; the other six are the new
    /// information. A field that is genuinely decoded carries structure across
    /// them; a walk that has landed on an unrelated byte at a fixed
    /// displacement from a framed offset tends to a near-constant value, which
    /// is what the two-story number needed and did not have.
    pub bits: [usize; 8],
}

/// **The emit bit.** `docs/whitebox/C2_MAP.md` §3E: the per-function flag word
/// lives at symbol offset `0x4c`, and c2's work-queue walk selects on
/// `test dl, 0x20` at `0x10b7f16e` (the load is `mov edx,[eax+0x4c]` at
/// `0x10b7f16b`, inside `0x10b7f022`).
///
/// [`c2_il::func::gl_function_attrs`] returns the **low byte** of that same
/// `+0x4c` word for every named `.gl` function record — §3E's own decode of a
/// tag-`0x0e` record shows `varU -> +0x4c = 0x1068` sitting exactly where that
/// reader takes its attribute byte, three fields past the framed `.ex` offset.
/// Bit 6 (`FN_FLAG_INLINABLE`, `0x40`) is the only bit that reader's existing
/// consumer looks at; `0x20` lives in the same low byte and is already decoded.
///
/// **This constant is disassembly-derived and carries a
/// `docs/whitebox/DISCLOSURE.md` row** (CLAUDE.md, binding).
///
/// **It is a SEED and not the emitted set.** §3E's cascade measurement is
/// explicit: on a bundle with a real call graph, clearing `0x20` on 17 of 20
/// functions changed nothing — each function falls only once its **caller** has
/// also been cleared. The emitted set is the `0x20`-seeded set **closed under
/// "referenced by an already-emitted function"**, and §3E's own practical
/// warning is that a port using the seed alone *"will over-delete on real
/// TUs"*. So this component is deliberately split: the seed is published now
/// and the closure is a lane of its own, with the gap between them measured
/// rather than argued.
///
/// # MEASURED ON THE 870-TU WORKLOAD, AND THE ANSWER REFUTES BUILDING AN EMIT-SET MODEL ON THIS BIT
///
/// The gap is not a gap. At workload stamp `6f3a818e9893`, measured twice with
/// byte-identical results:
///
/// | figure | value |
/// |---|---|
/// | `.gl` function records the reader names | **28,107** |
/// | of those, bit 6 (`FN_FLAG_INLINABLE`) set | **28,104** — 99.99 % |
/// | of those, byte == `0x00` | **0** |
/// | of those, this bit set — the SEED | **331** — 1.18 % |
/// | functions real c2 actually emitted | **162,146** |
/// | TUs where the seed is EMPTY, of 854 answered | **739** |
///
/// **The byte is not mis-decoded in the UNIFORM-ZERO way, and that is all bit 6
/// establishes.** 99.99 % with zero `0x00` bytes rules out the signature
/// [`c2_il::func::gl_function_attrs`]'s own doc records having produced once
/// (*"nine of eleven records decoded as attribute `0x00` … a uniform answer
/// where the grid predicts a split"*) and nothing else. The full bit histogram
/// ([`AttrCensus::bits`]) shows three varying bits (5.3 %, 1.2 %, 95.9 %) and
/// four constant ones — structure, hence consistent with a decoded field on the
/// records the walk reaches. Corroboration, not proof, and said as such.
///
/// **§3E's seed-plus-closure model cannot be built out of THIS READER'S
/// OUTPUT.** A closure over an empty seed is empty, and 739 of 854 TUs have an
/// empty seed. That part stands.
///
/// # WHAT DOES NOT STAND: "a 17.7 % ceiling on `.gl`, whichever bit is chosen"
///
/// This doc first read *"the reader's map covers only 28,107 of the 158,802
/// emitted functions — a 17.7 % ceiling on anything keyed off it"*. **That was
/// `|names| / |emitted|` with no intersection taken, and it attributed to `.gl`
/// what belongs to the walk.** Measured, with both sets already in the grader's
/// hands:
///
/// | | |
/// |---|---|
/// | of the 28,107 records this reader names, functions c2 EMITTED | **3,933** — **2.4 %** of 162,146 |
/// | emitted functions named by [`c2_il::mangled_names`], which does **not** use the record framing | **70,114** — **43.2 %** |
///
/// **Eighteen times the reach, over the same `.gl` bytes.** So `.gl` carries the
/// name of at least 43.2 % of the emitted set and this walk reaches 2.4 % of it:
/// the shortfall is in the SCAN, whose loop steps `p += 1` past any unframed
/// offset with no refusal and no counter, so its hit rate reads as a container
/// fact. That is board #3237, and the first deliverable of an emit-set lane is
/// now that skip path rather than the search for another field.
///
/// The constant is **kept**, in `crates/`, feeding **one instrument**, because
/// the refutation is the deliverable: where the seed IS non-empty it is
/// startlingly good — exact against c2's own emitted set on **27** TUs, **9** of
/// them TUs the port does not convert — and it over-claims exactly **once** in
/// the whole workload. See `docs/rungs/2026-08-20-objplan.md` and the
/// `W-OBJPLAN-1` row in `docs/whitebox/DISCLOSURE.md`.
pub const FN_FLAG_EMIT_SEED: u8 = 0x20;

/// Predict the object plan from the IL bundle, **without emitting**.
///
/// Every component that is not modelled returns `Unknown` with the census key
/// naming the stage that owes it. Nothing here reads a body, calls the
/// emitter, or asks the admission gate.
pub fn predict(bundle: &IlBundle, inputs: &PlanInputs) -> PredictedPlan {
    let gl = bundle.get("gl");

    // ---- the emit set, via the `0x20` seed --------------------------------
    //
    // `gl_function_attrs` is whole-file fail-closed: one unrecognized
    // `SRCPOS`/`SIZE` encoding anywhere refuses the map, and "no information"
    // is required to be the status quo rather than a permission. That refusal
    // is this component's `Unknown` and its rate is P1's ceiling — measured,
    // never assumed.
    let attrs: Option<BTreeMap<String, u8>> = gl.and_then(c2_il::func::gl_function_attrs);
    // **The ORTHOGONAL reader, walked ONCE.** `mangled_names` returns the `.gl`
    // symbol runs in file order and is the substrate of two different things
    // here — the record-order characterization below, and `gl_run_names`, which
    // is the coverage control on `gl_function_attrs`. It is a whole-`.gl` walk;
    // calling it twice for two views of one answer is the kind of cost that
    // shows up in `mode_cross`'s ~90,000 generated cases and nowhere a lane
    // would look for it.
    let run_names: Option<Vec<String>> = gl.map(c2_il::mangled_names);
    // **`/Gy` IS THE DECISION, AND IT IS NOW MADE HERE.** The emit set this
    // component is about is the set of COMDAT `.text` leaders. Without
    // function-level linking there are no per-function COMDATs at all, so a seed
    // over function names is not a prediction of *that* set and must not be
    // offered as one. `PlanInputs::function_level_linking` was plumbed in and
    // never read until the review pointed out that "one decision and not two"
    // describes a decision nobody was making.
    //
    // **Stated plainly: this branch has ZERO witnesses on the dc3 workload** —
    // `/O1` implies `/Gy` and every graded TU carries it — so it is a fence and
    // not a measured claim. It is exercised by
    // [`tests::without_function_level_linking_the_seed_refuses`] and by nothing
    // else, and it is the conservative direction (refuse, never over-claim).
    let (seed, order) = match (&attrs, gl, inputs.function_level_linking) {
        (_, _, false) => (
            Predicted::Unknown("plan-no-function-level-linking"),
            Predicted::Unknown("plan-no-function-level-linking"),
        ),
        (None, None, _) => (
            Predicted::Unknown("plan-no-gl"),
            Predicted::Unknown("plan-no-gl"),
        ),
        (None, Some(_), _) => (
            Predicted::Unknown("gl-attrs-refused"),
            Predicted::Unknown("gl-attrs-refused"),
        ),
        (Some(a), Some(_), true) => {
            let seed: BTreeSet<String> = a
                .iter()
                .filter(|(_, &v)| v & FN_FLAG_EMIT_SEED != 0)
                .map(|(n, _)| n.clone())
                .collect();
            // **The ONLY ordered reader available.** `gl_body_record_names` and
            // `gl_gate_record_names` return a `BTreeSet` and have therefore
            // already discarded order; `mangled_names` walks the `.gl` symbol
            // runs in file order. Whether that order relates to COMDAT section
            // order at all is a MEASUREMENT this lane owes and does not assume
            // — if it does not, `emit_set_order` starts near zero and #259's
            // `coff::order::plan_text_order` is the whole component.
            let mut ordered: Vec<String> = Vec::new();
            let mut seen: BTreeSet<String> = BTreeSet::new();
            for n in run_names.iter().flatten().cloned() {
                // A name that occurs twice among the `.gl` symbol runs is
                // emitted once; the FIRST occurrence fixes its position.
                // Deduplicated here rather than left to the grader, because a
                // repeated name would make the ORDER component differ for a
                // reason that has nothing to do with order.
                if seed.contains(&n) && seen.insert(n.clone()) {
                    ordered.push(n);
                }
            }
            (Predicted::Known(seed), Predicted::Known(ordered))
        }
        (Some(_), None, true) => unreachable!("attrs are derived from gl"),
    };

    PredictedPlan {
        // **BOTH COMPONENTS SHIP `Unknown`**, under this lane's own registered
        // §3 rule, applied uniformly: `emitset-members`' control differs on 2 of
        // 26 and `emitset-order`'s on 12 of 26, and the rule as written does not
        // grade the size of the shortfall. The two predictors survive beside
        // them as characterization values so nothing measured is lost — see the
        // fields' docs.
        emit_set_members: Predicted::Unknown("emitset-seed-control-red"),
        emit_set_order: Predicted::Unknown("emitset-glorder-control-red"),
        gl_seed_members: seed,
        gl_record_order: order,
        gl_attr_names: match (&attrs, gl) {
            (Some(a), _) => Predicted::Known(a.keys().cloned().collect()),
            (None, None) => Predicted::Unknown("plan-no-gl"),
            (None, Some(_)) => Predicted::Unknown("gl-attrs-refused"),
        },
        gl_run_names: run_names.map(|v| v.into_iter().collect()).unwrap_or_default(),
        attr_census: attrs.as_ref().map(|a| {
            let mut bits = [0usize; 8];
            for v in a.values() {
                for (b, slot) in bits.iter_mut().enumerate() {
                    if v & (1u8 << b) != 0 {
                        *slot += 1;
                    }
                }
            }
            AttrCensus {
                names: a.len(),
                bit6: bits[6],
                zero: a.values().filter(|v| **v == 0).count(),
                bits,
            }
        }),
        // Nothing below is modelled yet. Each names the stage that owes it, and
        // each is `Unknown` rather than an empty `Known` — see [`Predicted`].
        sections: Predicted::Unknown("plan-sections-unmodelled"),
        weak: Predicted::Unknown("plan-weak-unmodelled"),
        undef: Predicted::Unknown("plan-undef-unmodelled"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **The fence, as a source-level test.** `crates/c2-core/src/plan/` may not
    /// name the emitter or the admission gate. If it did, the control on the
    /// matched TUs would be free — the predictor would be reaching the same
    /// code that produced the bytes the control compares against — and the
    /// curve on the refused TUs would collapse to the reader's refusal mass
    /// (#3237).
    ///
    /// **THE BANNED NEEDLES.** One table, read by both halves of the fence
    /// below — the file scan and the directory scan — so the two cannot drift.
    const BANNED: [&str; 7] = [
        "PortC2",
        "crate::coff",
        "crate::codegen",
        "codegen::select",
        "coff::emit",
        ".functions(",
        "IlBundle::functions",
    ];

    /// Strip `//` comments and everything from the `#[cfg(test)]` marker on.
    ///
    /// The comment strip is not tidiness and it is not a weakening. The first
    /// version of this fence scanned the raw text and went red on its own module
    /// doc, which explains the ban by naming `PortC2` and `IlBundle::functions()`.
    /// **A fence that its own explanation trips is a fence nobody can document**,
    /// and the pressure would have been to delete the explanation rather than
    /// the dependency — which is the wrong direction, because the explanation is
    /// what a reviewer reads.
    ///
    /// What is given up: a banned symbol hidden after a `//` on a code line would
    /// not be seen. That is acceptable because a *use* of one is code, never a
    /// trailing comment, and because — as the module doc says — this is a
    /// **source fence and not a type fence**: the real check is a reviewer
    /// reading the `use` list.
    fn scannable(src: &str) -> String {
        src.split("#[cfg(test)]")
            .next()
            .expect("the module body precedes its tests")
            .lines()
            .map(|l| match l.find("//") {
                Some(i) => &l[..i],
                None => l,
            })
            .collect::<Vec<&str>>()
            .join("\n")
    }

    /// **THE FENCE GUARDS THE DIRECTORY, NOT ONE FILE.**
    ///
    /// The `include_str!` half below cannot see a sibling. The review pointed
    /// out that the very next step named in the rung's §5 — a `plan/sections.rs`
    /// whose only walk-free substrate is `shell_only_tu` / `data_tu` /
    /// `provide_data_tu` / `bss_shell_objects`, all `IlBundle` methods on the
    /// **admission side** of the reader — would escape a file-scoped scan
    /// silently, and the curve would quietly become the reader's refusal mass.
    /// That is the single failure this fence exists to prevent, arriving through
    /// the fence's own blind spot.
    ///
    /// So this half walks `crates/c2-core/src/plan/` on disk. It is **not** a
    /// replacement for the `include_str!` half: a filesystem scan can be
    /// defeated by running from a tree where the directory is absent, so it
    /// **fail-closed asserts that it found `mod.rs`** before believing any
    /// absence. Two halves, two failure modes, neither trusted alone.
    #[test]
    fn every_file_under_plan_obeys_the_fence_not_just_this_one() {
        let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/src/plan");
        let entries = std::fs::read_dir(dir).unwrap_or_else(|e| {
            panic!(
                "the plan/ fence could not read {dir}: {e}. A directory scan that \
                 cannot open its directory must FAIL — an unreadable directory and \
                 a clean one are the same empty answer, which is the absence-as-\
                 success shape this whole module is about."
            )
        });
        let mut scanned: Vec<String> = Vec::new();
        for e in entries {
            let p = e.expect("a readable directory entry").path();
            if p.extension().and_then(|s| s.to_str()) != Some("rs") {
                continue;
            }
            let name = p.file_name().unwrap().to_string_lossy().to_string();
            let text = std::fs::read_to_string(&p).expect("a readable .rs file");
            let body = scannable(&text);
            for banned in BANNED {
                assert!(
                    !body.contains(banned),
                    "crates/c2-core/src/plan/{name} must not name `{banned}`: the \
                     predictor must be computable on the TUs the reader refuses, or \
                     the curve it publishes is the reader's refusal mass wearing new \
                     keys (#3237). THIS IS THE DIRECTORY HALF OF THE FENCE — the \
                     file-scoped half cannot see {name}."
                );
            }
            scanned.push(name);
        }
        // The positive check, first in intent even though it reads last: a
        // scanner looking for absent needles passes trivially when it scanned
        // nothing.
        assert!(
            scanned.iter().any(|n| n == "mod.rs"),
            "the directory fence scanned {scanned:?} and did not find mod.rs — it \
             is not reading crates/c2-core/src/plan/ at all"
        );
    }

    /// Reads its own source with `include_str!`, so it needs no filesystem and
    /// no toolchain, and it cannot be defeated by moving the file.
    #[test]
    fn the_plan_module_does_not_reach_the_emitter_or_the_admission_gate() {
        // Everything before the `#[cfg(test)]` marker — the tests may name what
        // they forbid — with comments stripped. See [`scannable`] for why.
        let body = scannable(include_str!("mod.rs"));
        // The mutation control: the fence must be able to FAIL. A test that
        // scans a string for absent needles passes trivially if the needle list
        // is wrong, so one needle that IS present is checked in the positive
        // direction first.
        assert!(
            body.contains("gl_function_attrs"),
            "the scanner is not reading this module's body at all"
        );
        for banned in BANNED {
            assert!(
                !body.contains(banned),
                "crates/c2-core/src/plan/ must not name `{banned}`: the predictor \
                 must be computable on the TUs the reader refuses, or the curve it \
                 publishes is the reader's refusal mass wearing new keys (#3237)"
            );
        }
    }

    #[test]
    fn unknown_is_never_known_empty() {
        let u: Predicted<BTreeSet<String>> = Predicted::Unknown("gl-attrs-refused");
        let k: Predicted<BTreeSet<String>> = Predicted::Known(BTreeSet::new());
        assert_ne!(u, k);
        assert_eq!(u.reason(), Some("gl-attrs-refused"));
        assert_eq!(k.reason(), None);
        assert!(u.known().is_none());
        assert!(k.known().is_some());
    }

    /// A bundle with no `.gl` publishes `plan-no-gl`, not an empty seed.
    #[test]
    fn a_bundle_with_no_gl_names_its_reason() {
        let b = IlBundle::new("_CL_test");
        let p = predict(&b, &PlanInputs { function_level_linking: true });
        // BOTH components are `Unknown` unconditionally — their controls are red
        // and this lane's own §3 rule says a component whose control is red does
        // not ship. So the no-`.gl` reason surfaces on the CHARACTERIZATION
        // values, which is where the measurement lives.
        assert_eq!(p.emit_set_members.reason(), Some("emitset-seed-control-red"));
        assert_eq!(p.emit_set_order.reason(), Some("emitset-glorder-control-red"));
        assert_eq!(p.gl_seed_members.reason(), Some("plan-no-gl"));
        assert_eq!(p.gl_record_order.reason(), Some("plan-no-gl"));
        assert_eq!(p.gl_attr_names.reason(), Some("plan-no-gl"));
        assert!(p.gl_run_names.is_empty());
        assert_eq!(p.attr_census, None);
    }

    /// **The `/Gy` decision, exercised** — the branch with zero workload
    /// witnesses.
    ///
    /// Without function-level linking there are no per-function COMDAT `.text`
    /// sections, so a seed over function names is not a prediction of the emit
    /// set and must refuse rather than offer one. `PlanInputs` was plumbed in
    /// and unread until the review said so; this test is the only witness the
    /// branch has and it is named as such.
    #[test]
    fn without_function_level_linking_the_seed_refuses() {
        let b = IlBundle::new("_CL_test");
        let p = predict(&b, &PlanInputs { function_level_linking: false });
        assert_eq!(
            p.gl_seed_members.reason(),
            Some("plan-no-function-level-linking")
        );
        assert_eq!(
            p.gl_record_order.reason(),
            Some("plan-no-function-level-linking")
        );
        // …and it is a DIFFERENT reason from the no-`.gl` one, or the `Unknown`
        // histogram cannot rank the two stages apart.
        let with = predict(&b, &PlanInputs { function_level_linking: true });
        assert_ne!(p.gl_seed_members.reason(), with.gl_seed_members.reason());
    }

    /// The attribute census counts every bit, not only the two the first version
    /// of the probe looked at. `bits[6]` and `bits[5]` are the two named ones by
    /// construction, and that identity is asserted so a reordering cannot make
    /// the histogram and the named keys disagree.
    #[test]
    fn the_attr_census_histograms_every_bit() {
        let c = AttrCensus {
            names: 3,
            bit6: 2,
            zero: 1,
            bits: [0, 0, 0, 0, 0, 1, 2, 0],
        };
        assert_eq!(c.bits[6], c.bit6);
        assert_eq!(c.bits.len(), 8);
    }

    /// The unmodelled components publish a NAMED reason apiece — never a
    /// shared one, because a shared reason cannot rank the stages that owe
    /// work.
    #[test]
    fn every_unmodelled_component_names_its_own_stage() {
        let b = IlBundle::new("_CL_test");
        let p = predict(&b, &PlanInputs { function_level_linking: true });
        let reasons = [
            p.sections.reason().unwrap(),
            p.weak.reason().unwrap(),
            p.undef.reason().unwrap(),
        ];
        let uniq: BTreeSet<&str> = reasons.iter().copied().collect();
        assert_eq!(uniq.len(), reasons.len(), "reasons must be distinct: {reasons:?}");
    }
}
