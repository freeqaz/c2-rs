//! **The OBJECT-PLAN GRADER** — `predict(IL)` against `observe(reference obj)`,
//! per TU, per component. Lane `w-objplan`.
//!
//! # The number this exists for
//!
//! At the tree this landed on, `a-and-b-and-c` is **27** and `match` is **26**:
//! the port has converted 26 of the 27 TUs that satisfy every factor, and the
//! remaining 844 fail several factors **at once**. Progress is therefore binary
//! per TU and every single-stage improvement scores zero — a perfect reader
//! converts 2 (#3191), a perfect section emitter converts 0 (#3210), lifting
//! the whole `.gl` walk measured `match +0` (#3093). There is no continuous
//! curve to steer by.
//!
//! The object plan is the part of the conjunction that can be graded against
//! real c2 on **all** graded TUs today, including the ones that produce no IR
//! at all, because none of it needs a body.
//!
//! # The four verdicts are a PARTITION, and collapsing any two is the defect
//!
//! [`PlanVerdict`] has four values and they sum to `graded()` for every
//! component, printed as counts:
//!
//! * [`PlanVerdict::Unobservable`] — the **reference** obj did not decode. The
//!   ground truth is missing; nothing can be said.
//! * [`PlanVerdict::Unknown`] — the **port** did not look, with the reason
//!   naming the stage that owes the work.
//! * [`PlanVerdict::Differs`] — both sides answered and they disagree.
//! * [`PlanVerdict::Exact`] — both sides answered and they agree.
//!
//! Folding `Unknown` into `Differs` would make an unmodelled component look
//! like a wrong one; folding either into `Unobservable` would blame the
//! extractor for the port's silence. This is board **#3237** in its most direct
//! form: *an instrument that returns 0 because it did not look is
//! indistinguishable from one that returns 0 because there was nothing to
//! find*, unless it says which.
//!
//! # THIS IS AN INSTRUMENT AND NEVER A GATE
//!
//! No `plan-*` key may fail `scripts/gate.sh`, gate an emit, or appear in a
//! refusal predicate. **`plan-*-exact` is NECESSARY but NOT SUFFICIENT for
//! `match`**: a TU can be plan-exact on every component and mismatch on every
//! instruction byte. The two figures that ARE allowed to be a hard red are
//! `plan-bounds-violations` — a containment invariant of the instrument itself
//! — and the named control on `docs/plan/CONTROL_TUS.txt`.
//!
//! # The denominator, chosen deliberately
//!
//! **TUs.** Option A's target is stated in TUs (870 of 878), so the curve's
//! denominator is the goal's denominator. This sidesteps #3254: the `fnbyte`
//! denominator is 71.2 % bodies the shipped image never contains, because `/Gy`
//! COMDATs get discarded by the linker — a *function*-level denominator
//! problem. A TU is a TU; there is no discard.
//!
//! Every component publishes **three** nested denominators and never a bare
//! ratio — `observable ⊇ known ⊇ exact` — with `differs` derived in
//! `metrics()` and never by the reader (board #213's rule: publishing two
//! halves and letting a reader subtract is how `+82` survived both its inputs
//! moving).

use std::collections::{BTreeMap, BTreeSet};

use c2_core::plan::{Predicted, PredictedPlan};
use c2_obj::ObjPlan;

/// One component's verdict on one TU. See the module doc — these four are a
/// partition and are printed as counts against `graded()`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum PlanVerdict {
    /// Both sides answered and agree.
    Exact,
    /// Both sides answered and disagree.
    Differs,
    /// The **port** did not look. The reason lives beside this in
    /// [`TuPlan::reasons`] and names the stage that owes the work.
    Unknown,
    /// The **reference** obj did not decode, so there is no ground truth.
    Unobservable,
}

impl PlanVerdict {
    pub fn label(self) -> &'static str {
        match self {
            PlanVerdict::Exact => "exact",
            PlanVerdict::Differs => "differs",
            PlanVerdict::Unknown => "unknown",
            PlanVerdict::Unobservable => "unobservable",
        }
    }
    pub fn parse(s: &str) -> Option<PlanVerdict> {
        Some(match s {
            "exact" => PlanVerdict::Exact,
            "differs" => PlanVerdict::Differs,
            "unknown" => PlanVerdict::Unknown,
            "unobservable" => PlanVerdict::Unobservable,
            _ => return None,
        })
    }
}

/// **The components that are GRADED** — i.e. those the port predicts and the
/// reference supplies. The key order here is the printed order and the TSV
/// column order, and the names are the `gap-metric` key stems, so a rename is
/// a visible interface change rather than a silent `NO-RESULT` (STATUS trap 5).
pub const PLAN_COMPONENTS: &[&str] = &["emitset-members", "emitset-order"];

/// **BOTH COMPONENTS SHIP `Unknown`, AND THAT IS THE REGISTERED RULE APPLIED TO
/// THE LANE THAT REGISTERED IT.**
///
/// This lane's prereg (`docs/rungs/_2026-08-20-objplan-prereg.md` §3) reads, in
/// full:
///
/// > `Differs` on a control TU means the extractor or the predictor is wrong
/// > and the component **does not ship**. A component whose control is red
/// > ships as `Unknown`, never as `Differs`: the manifest must not claim to
/// > disagree with the reference on a TU the byte judge has already called
/// > equal.
///
/// | component | predictor | control | verdict |
/// |---|---|---|---|
/// | `emitset-order` | `.gl` record order (`c2_il::mangled_names`) | **12 of 26 differ** | red → ships `Unknown` |
/// | `emitset-members` | the `0x20` seed (`FN_FLAG_EMIT_SEED`) | **2 of 26 differ** | red → ships `Unknown` |
///
/// # The asymmetry the review caught, recorded because it is the interesting part
///
/// The first version of this lane withdrew `emitset-order` under exactly that
/// sentence and **shipped `emitset-members` as its headline**, on the strength
/// of a post-hoc reading — *"the port's emit path does not consult the `0x20`
/// bit, so `Differs` there is not automatically a bug"*. That reading applies
/// **verbatim** to `emitset-order` (the emit path does not consult `.gl` record
/// order either), so the rule that actually separated them was *"12 of 26 is too
/// many, 2 of 26 is fine"* — **a threshold invented after seeing the data and
/// registered nowhere.** The instrument then printed `2 differs` on the same
/// screen as its own *"ONLY `differs` REDS THE LANE"*, every run, while the lane
/// reported outcome `instrument`.
///
/// The repair is the registered rule applied **uniformly**, and it is
/// deliberately the option that costs this lane its headline. Amending a prereg
/// after the measurement, to a threshold the measurement itself suggested, is
/// the failure the prereg exists to prevent; a lane that does it once has no
/// prereg at all.
///
/// # Nothing measured is lost
///
/// Both predictors are still computed and both are still published — as
/// `plan-emitset-seed-*` and `plan-emitset-glorder-agrees`, **characterization
/// keys** and never a curve. What the rule forbids is the *claim of
/// disagreement*: publishing `members-differs 821` would be publishing 821
/// disagreements produced by a rule this lane's own §2.4 declares refuted at
/// workload scale, which is #3329's own transferable finding turned on the
/// component #3329 spared.

/// One component's four `gap-metric` key names, spelled out as `&'static str`.
///
/// **Spelled out and not `format!`ed**, for two reasons. `GapReport::metrics`
/// publishes `(&'static str, String)` pairs and a formatted key would have to be
/// leaked; and — the reason that matters — **the keys are an interface**. A
/// renamed key does not error, it returns `NO-RESULT` from `scripts/status.sh`'s
/// `sed` collector, which is STATUS trap 5 (absence read as success) with the
/// mask on. Written out here, a rename is a visible diff in one table, and
/// [`tests::the_key_table_and_the_component_list_agree`] fails if the table and
/// [`PLAN_COMPONENTS`] ever drift.
pub struct PlanKeys {
    pub component: &'static str,
    /// The reference obj decoded — ground truth exists.
    pub observable: &'static str,
    /// …and the port also answered. `known ⊆ observable`.
    pub known: &'static str,
    /// …and the two agree. `exact ⊆ known`.
    pub exact: &'static str,
    /// `known − exact`, derived HERE and never by the reader (board #213).
    pub differs: &'static str,
    /// How many DISTINCT observed values this component takes across the graded
    /// TUs. A component at `distinct == 1` is **free**: it would publish a 100 %
    /// that measures nothing, and it is labelled free in the printed block and
    /// excluded from any headline.
    pub distinct: &'static str,
}

pub const PLAN_KEYS: &[PlanKeys] = &[
    PlanKeys {
        component: "emitset-members",
        observable: "plan-emitset-members-observable",
        known: "plan-emitset-members-known",
        exact: "plan-emitset-members-exact",
        differs: "plan-emitset-members-differs",
        distinct: "plan-emitset-members-distinct",
    },
    PlanKeys {
        component: "emitset-order",
        observable: "plan-emitset-order-observable",
        known: "plan-emitset-order-known",
        exact: "plan-emitset-order-exact",
        differs: "plan-emitset-order-differs",
        distinct: "plan-emitset-order-distinct",
    },
];

/// **Observe-side inventory keys** — facts read off the reference obj that the
/// port does not model at all.
///
/// These are not a curve and are never counted as one: they are the honest
/// *description* of the population the un-conjuncted lanes will have to serve,
/// and nothing in this repo had measured any of them. `docs/BOARD.md`'s "weak
/// externals (675 TUs)" and "COMDAT synthesis (450 TUs)" are **carried**
/// figures with no locator; these keys re-derive them directly.
pub const PLAN_OBSERVED_KEYS: &[&str] = &[
    "plan-obs-weak-tus",
    "plan-obs-weak-records",
    "plan-obs-comdat-sections",
    "plan-obs-comdat-assoc-sections",
    "plan-obs-comdat-assoc-tus",
    "plan-obs-comdat-sel-nodups",
    "plan-obs-comdat-sel-any",
    "plan-obs-comdat-sel-samesize",
    "plan-obs-comdat-sel-exact",
    "plan-obs-comdat-sel-assoc",
    "plan-obs-comdat-sel-largest",
    "plan-obs-comdat-sel-unknown",
    "plan-obs-undef-records",
    "plan-obs-undef-tus",
    "plan-obs-sections",
    "plan-obs-drectve-tus",
    "plan-obs-reloc-records",
    // The DECIDING PROBE on the seed bit — see `PredictedPlan::attr_census`.
    // These describe the predictor's own INPUT, which is why they are
    // observe-side counters rather than a component: they say whether the byte
    // the seed is read out of is the right byte at all, and whether the reader
    // reaches the records it claims to describe.
    "plan-glattr-names",
    "plan-glattr-bit6",
    "plan-glattr-zero",
    "plan-glattr-seed",
    // **THE ALTERNATIVE EXPLANATION, MEASURED.** `plan-glattr-names 28,107`
    // against `158,802` emitted has two stories and the first version of the
    // probe tested one: *the bit is rare* versus *the scanner finds one record
    // in six*. `gl_function_attrs` steps `p += 1` past an unframed offset with
    // no refusal and no counter, so under-coverage looks exactly like a fact
    // about `.gl`.
    //
    // `glruns-*` is the ORTHOGONAL reader (`c2_il::mangled_names`, which does
    // not use `gl_offset_framed` at all) and the two intersections against the
    // reference obj's own emitted set are what separate the stories:
    //   * runs∩emitted ≈ emitted while attr∩emitted ≈ 28k  → it is the SCANNER
    //   * runs∩emitted ≈ attr∩emitted ≈ 28k                → it is `.gl`
    "plan-glruns-names",
    "plan-glruns-in-emitset",
    "plan-glattr-in-emitset",
    // The other six bits of the byte. A genuinely decoded field is structured
    // across them; a walk that landed on an unrelated byte at a fixed
    // displacement tends to a near-constant value. (`bit5` is the seed and
    // `bit6` is `FN_FLAG_INLINABLE`; both are published under their own names
    // above and repeated here so the histogram is complete and readable as one.)
    "plan-glattr-bit0",
    "plan-glattr-bit1",
    "plan-glattr-bit2",
    "plan-glattr-bit3",
    "plan-glattr-bit4",
    "plan-glattr-bit5",
    "plan-glattr-bit7",
    // **R2 AT WORKLOAD SCALE.** The prereg's tertiary criterion is that
    // `observe` agrees with each existing `c2-obj` accessor *"over the whole
    // workload, TU by TU"*; what shipped was three hand-written synthetic cells
    // in `tests/plan_agreement.rs`. These two carry the emit-set half over every
    // TU that captured: `-tus` is the population (a control on the control — it
    // must not be 0) and `-disagree` has known answer **0**.
    //
    // Note the two walks are ordered differently by construction — `observe`
    // builds the emit set in SECTION-table order, `text_comdat_entries` in
    // SYMBOL-table order — so the workload-wide comparison is a SET comparison
    // and says so; the ordered `assert_eq!` in `plan_agreement.rs` only ever
    // held because its three objs are ones where the two coincide.
    "plan-agree-emitset-tus",
    "plan-agree-emitset-disagree",
];

/// One TU's plan grade.
#[derive(Clone, Debug, Default)]
pub struct TuPlan {
    /// Did the **reference** obj decode into an [`ObjPlan`]?
    pub observable: bool,
    /// Component → verdict, over [`PLAN_COMPONENTS`].
    pub verdicts: BTreeMap<String, PlanVerdict>,
    /// Component → the port's named reason, when the verdict is `Unknown`.
    pub reasons: BTreeMap<String, String>,
    /// Component → a canonical rendering of the **observed** value, used only
    /// to count `distinct` across the workload. A component whose observed
    /// value is the same on every TU is **free**: it would give a 100 % that
    /// measures nothing, so it is labelled and excluded from any headline.
    pub sigs: BTreeMap<String, String>,
    /// The observe-side inventory counters, over [`PLAN_OBSERVED_KEYS`].
    pub obs: BTreeMap<String, usize>,
    /// `|predicted \ observed|` and `|observed \ predicted|` for the emit set,
    /// when both sides answered. The **sizes** of the disagreement, which a
    /// verdict alone cannot give and which prices the closure lane.
    pub emitset_extra: Option<usize>,
    pub emitset_missing: Option<usize>,
    /// Did the seed avoid over-claiming — is `predicted ⊆ observed`? `None`
    /// when either side did not answer.
    pub emitset_subset: Option<bool>,
    /// **`|predicted|` and `|observed|` for the emit set.**
    ///
    /// Published because the first workload run printed *"seed ⊆ emitted on 853
    /// of 854 TUs"* **and could not rule out that 853 of those seeds were
    /// EMPTY** — the empty set is a subset of everything. A containment claim
    /// without the claimant's size is unfalsifiable in the flattering
    /// direction, which is #3237 inside the instrument built to prevent it.
    pub emitset_pred_size: Option<usize>,
    /// **`|observed emit set|`, set on EVERY observable TU** — independently of
    /// whether the port answered.
    ///
    /// It used to be set only inside the `Known` arm, so `plan-emitset-observed-size`
    /// summed over the **854** TUs the seed answered on and was captioned as
    /// *"functions real c2 actually emitted"* — a whole-workload phrasing for an
    /// 854-TU figure, published beside `fnbyte-denominator` **162,136**, which is
    /// the same quantity over 870, with no reconciliation. The 3,334 gap is
    /// exactly the 16 `gl-attrs-refused` TUs. It is a fact about the REFERENCE
    /// obj and it does not belong behind a gate on the PORT's silence.
    ///
    /// The 854-TU restriction is still published, as
    /// `plan-emitset-observed-size-known`, because the seed-coverage ratios have
    /// to be taken over a population where both sides answered.
    pub emitset_obs_size: Option<usize>,
    /// The **refuted** `.gl`-record-order rule's verdict, kept as a
    /// characterization value and never as a component. See
    /// [`PLAN_COMPONENTS`].
    pub glorder: Option<bool>,
    /// **One NAME from each side of an emit-set disagreement**, first in sort
    /// order so it is deterministic.
    ///
    /// A count says a component differs; it cannot say *what by*, and a
    /// shortfall on a control TU has to be actionable or the control produces
    /// work nobody can start. Two names turn "TomCryptLicense.cpp differs by 1"
    /// into a mechanism.
    pub emitset_missing_witness: Option<String>,
    pub emitset_extra_witness: Option<String>,
    /// Containment violations found on this TU (see
    /// [`TuPlan::bounds_violations`]). MUST be empty.
    pub violations: Vec<String>,
    /// **How many of the containment checks were actually EVALUATED on this
    /// TU** — the denominator `plan-bounds-violations` was published without.
    ///
    /// I4 registers `plan-bounds-violations == 0` as a REQUIRED-ZERO and the
    /// key's own doc says it is published as a *count* rather than a status so
    /// that a control which checked nothing is distinguishable from one that
    /// passed. The review then showed the count itself had the property: with
    /// `emitset-order` out of `PLAN_COMPONENTS` the ladder check's lookup was
    /// permanently `None`, the subset check followed from its own antecedent by
    /// construction, and the unobservable check sat behind an early return that
    /// writes `Unobservable` for every component — **all three unreachable in
    /// production, so the zero could not be told apart from not looking.**
    ///
    /// That is the same defect one level up, so the same repair applies one
    /// level up: publish the number of checks REACHED beside the number that
    /// fired. `0 violations of 1,740 checks reached` is a measurement;
    /// `0 violations` is not.
    pub checks_reached: usize,
}

impl TuPlan {
    /// **The containment control.** These are invariants of the *instrument*,
    /// not claims about c2, and every one of them has a known answer of zero.
    /// Published as a **count and not as a status** (STATUS trap 5): a control
    /// that checks nothing and a control that passes look identical in a
    /// summary line.
    ///
    /// 1. `order-exact ⇒ members-exact` — the ordered sequence being right
    ///    while the set is wrong is arithmetically impossible, so a hit here is
    ///    a grader bug.
    /// 2. `members-exact ⇒ seed ⊆ observed` — equality implies containment.
    /// 3. No component may read `Exact` or `Differs` on an unobservable TU.
    ///
    /// Returns `(violations, checks reached)`. **The second half is not
    /// decoration** — see [`TuPlan::checks_reached`]. A check counts as reached
    /// when both of its operands are defined on this TU, which is the only
    /// state in which it could have fired.
    fn bounds_violations(&self) -> (Vec<String>, usize) {
        let mut v = Vec::new();
        let mut reached = 0usize;
        let g = |k: &str| self.verdicts.get(k).copied();
        // 1. The ladder. Reached only when BOTH components answered on this TU;
        //    while either ships `Unknown` it is dormant, and the counter is what
        //    says so instead of a silent zero.
        if let (Some(o), Some(m)) = (g("emitset-order"), g("emitset-members")) {
            if matches!(o, PlanVerdict::Exact | PlanVerdict::Differs)
                && matches!(m, PlanVerdict::Exact | PlanVerdict::Differs)
            {
                reached += 1;
                if o == PlanVerdict::Exact && m != PlanVerdict::Exact {
                    v.push("order-exact-without-members-exact".to_string());
                }
            }
        }
        // 2. Equality implies containment. **True by construction at this tree**
        //    (`Exact` is recorded iff `extra == 0 && missing == 0`, and
        //    `emitset_subset = Some(extra == 0)`), and retained as a regression
        //    fence for the day the two assignments are decoupled — which the
        //    seed/component split in this very fix round moved one step closer.
        //    Counted as reached only when the component answered.
        if let Some(m) = g("emitset-members") {
            if matches!(m, PlanVerdict::Exact | PlanVerdict::Differs) {
                reached += 1;
                if m == PlanVerdict::Exact && self.emitset_subset != Some(true) {
                    v.push("members-exact-without-subset".to_string());
                }
            }
        }
        // 2b. The same invariant on the CHARACTERIZATION seed, where it is NOT
        //     true by construction: `seed_exact` is derived from the sizes in
        //     the TSV and `emitset_subset` from the set difference, so a walk
        //     that lost a name in one and not the other fires here. This is the
        //     one of the four that is reachable on a real TU today, and it is
        //     reached on all 854 the seed answers on.
        if let (Some(extra), Some(missing)) = (self.emitset_extra, self.emitset_missing) {
            reached += 1;
            if extra == 0 && missing == 0 && self.emitset_subset != Some(true) {
                v.push("seed-exact-without-subset".to_string());
            }
        }
        // 3. No component may grade a TU with no ground truth.
        if !self.observable {
            reached += 1;
            for (k, verdict) in &self.verdicts {
                if matches!(verdict, PlanVerdict::Exact | PlanVerdict::Differs) {
                    v.push(format!("graded-without-ground-truth:{k}"));
                }
            }
        }
        (v, reached)
    }
}

/// **FNV-1a 64**, hand-rolled — the workspace is std-only and zero external
/// crates, and `DefaultHasher` is explicitly documented as unstable across
/// releases, which would make a stored signature non-reproducible.
fn fnv1a(bytes: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in bytes {
        h ^= *b as u64;
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    h
}

/// Canonicalize a component's OBSERVED value into a fixed-width signature for
/// the `distinct` count.
///
/// **Hashed rather than stored verbatim, on purpose.** The question is *"does
/// this component take more than one value across the workload"*, not *"what
/// value"* — and one TU's emit set is 158 mangled names on `src/App.cpp`, so
/// storing the text would carry tens of megabytes through a scan that also runs
/// over `mode_cross`'s ~90,000 generated cases. The length is prefixed so two
/// different-length values cannot collide on a hash alone.
fn sig_of<'a, I: IntoIterator<Item = &'a str>>(it: I) -> String {
    let v: Vec<&str> = it.into_iter().collect();
    sig_of_text(v.len(), &v.join("\u{1}"))
}

/// The same, for a component that is already one string.
fn sig_of_text(n: usize, s: &str) -> String {
    format!("{n}:{:016x}", fnv1a(s.as_bytes()))
}

/// **Grade one TU.**
///
/// `observed` is `None` when the reference obj did not decode — every component
/// then reads [`PlanVerdict::Unobservable`], which takes precedence over the
/// port's own silence: without ground truth there is nothing to be silent
/// *about*, and blaming the port for the extractor's failure is the shape this
/// partition exists to prevent.
pub fn grade(observed: Option<&ObjPlan>, predicted: &PredictedPlan) -> TuPlan {
    let mut t = TuPlan {
        observable: observed.is_some(),
        ..Default::default()
    };

    let record = |t: &mut TuPlan, key: &str, v: PlanVerdict, reason: Option<&'static str>| {
        t.verdicts.insert(key.to_string(), v);
        if let Some(r) = reason {
            t.reasons.insert(key.to_string(), r.to_string());
        }
    };

    let Some(obs) = observed else {
        for k in PLAN_COMPONENTS {
            record(&mut t, k, PlanVerdict::Unobservable, None);
        }
        (t.violations, t.checks_reached) = t.bounds_violations();
        return t;
    };

    // ---- the observe-side inventory ---------------------------------------
    for k in PLAN_OBSERVED_KEYS {
        t.obs.insert((*k).to_string(), 0);
    }
    let bump = |t: &mut TuPlan, k: &str, n: usize| {
        *t.obs.get_mut(k).expect("inventory key must be pre-seeded") += n;
    };
    bump(&mut t, "plan-obs-weak-records", obs.weak.len());
    if !obs.weak.is_empty() {
        bump(&mut t, "plan-obs-weak-tus", 1);
    }
    bump(&mut t, "plan-obs-undef-records", obs.undef.len());
    if !obs.undef.is_empty() {
        bump(&mut t, "plan-obs-undef-tus", 1);
    }
    bump(&mut t, "plan-obs-sections", obs.sections.len());
    if !obs.drectve.is_empty() {
        bump(&mut t, "plan-obs-drectve-tus", 1);
    }
    bump(
        &mut t,
        "plan-obs-reloc-records",
        obs.relocs.iter().map(|r| r.entries.len()).sum::<usize>(),
    );
    let mut assoc_here = 0usize;
    for s in &obs.sections {
        let Some(c) = &s.comdat else { continue };
        bump(&mut t, "plan-obs-comdat-sections", 1);
        // Broken out per `IMAGE_COMDAT_SELECT_*` value rather than into an
        // `other` bucket. The first run put 106,333 sections in `other` and
        // 101,119 of them were ASSOCIATIVE — a bucket that large hides exactly
        // the distinction a COMDAT-synthesis lane needs, and `unknown` (a
        // selection outside 1..=6) must be visible on its own or an undecoded
        // value reads as a known one.
        match c.selection {
            1 => bump(&mut t, "plan-obs-comdat-sel-nodups", 1),
            2 => bump(&mut t, "plan-obs-comdat-sel-any", 1),
            3 => bump(&mut t, "plan-obs-comdat-sel-samesize", 1),
            4 => bump(&mut t, "plan-obs-comdat-sel-exact", 1),
            5 => bump(&mut t, "plan-obs-comdat-sel-assoc", 1),
            6 => bump(&mut t, "plan-obs-comdat-sel-largest", 1),
            _ => bump(&mut t, "plan-obs-comdat-sel-unknown", 1),
        }
        if c.assoc.is_some() {
            assoc_here += 1;
        }
    }
    if let Some(c) = predicted.attr_census {
        bump(&mut t, "plan-glattr-names", c.names);
        bump(&mut t, "plan-glattr-bit6", c.bit6);
        bump(&mut t, "plan-glattr-zero", c.zero);
        for (b, key) in [
            (0, "plan-glattr-bit0"),
            (1, "plan-glattr-bit1"),
            (2, "plan-glattr-bit2"),
            (3, "plan-glattr-bit3"),
            (4, "plan-glattr-bit4"),
            (5, "plan-glattr-bit5"),
            (7, "plan-glattr-bit7"),
        ] {
            bump(&mut t, key, c.bits[b]);
        }
    }
    bump(&mut t, "plan-obs-comdat-assoc-sections", assoc_here);
    if assoc_here > 0 {
        bump(&mut t, "plan-obs-comdat-assoc-tus", 1);
    }

    // The `distinct` signatures. `.drectve` and the section ladder are the
    // candidates for being FREE — a component with one distinct value across
    // 870 TUs is a 100 % that measures nothing.
    t.sigs.insert(
        "sections-names".to_string(),
        sig_of(obs.section_names()),
    );
    let attrs: Vec<String> = obs
        .section_attrs()
        .iter()
        .map(|(n, c, s)| format!("{n}/{c:08x}/{s:?}"))
        .collect();
    t.sigs.insert(
        "sections-attrs".to_string(),
        sig_of(attrs.iter().map(String::as_str)),
    );
    t.sigs.insert(
        "drectve".to_string(),
        sig_of_text(
            obs.drectve.len(),
            &String::from_utf8_lossy(&obs.drectve),
        ),
    );
    t.sigs.insert(
        "emitset-members".to_string(),
        sig_of(obs.emit_set.iter().map(String::as_str)),
    );
    // **Every key that is published must have a producer here.** On the first
    // workload run `plan-emitset-order-distinct` read **0** because nothing
    // inserted its signature — a key reading zero for *"did not look"*, which is
    // the exact failure this whole instrument exists to make impossible. The
    // component is gone now, but the rule stays: `plan_distinct` reports only
    // keys present in this map, and a missing one must be an absent ROW rather
    // than a zero.
    t.sigs.insert(
        "emitset-order".to_string(),
        format!(
            "{}|{}",
            obs.emit_set.len(),
            sig_of(obs.emit_set.iter().map(String::as_str))
        ),
    );

    // ---- the two COMPONENTS ------------------------------------------------
    //
    // Both ship `Unknown`, under the lane's own registered §3 rule applied
    // uniformly — see [`PLAN_COMPONENTS`] for why, and for the asymmetry that
    // made this a fix round rather than the original design. The grading arm for
    // a `Known` component is kept live and exercised by the unit fixtures,
    // because a component that goes green is expected to ship and the arm it
    // ships through must not be dead code that has never run.
    let observed_set: BTreeSet<&str> = obs.emit_set.iter().map(String::as_str).collect();
    let grade_component = |t: &mut TuPlan, key: &str, p: &Predicted<BTreeSet<String>>| {
        match p {
            Predicted::Unknown(r) => record(t, key, PlanVerdict::Unknown, Some(r)),
            Predicted::Known(names) => {
                let pred: BTreeSet<&str> = names.iter().map(String::as_str).collect();
                let exact = pred == observed_set;
                record(
                    t,
                    key,
                    if exact { PlanVerdict::Exact } else { PlanVerdict::Differs },
                    None,
                );
            }
        }
    };
    grade_component(&mut t, "emitset-members", &predicted.emit_set_members);
    match &predicted.emit_set_order {
        Predicted::Unknown(r) => record(&mut t, "emitset-order", PlanVerdict::Unknown, Some(r)),
        Predicted::Known(p) => record(
            &mut t,
            "emitset-order",
            if *p == obs.emit_set { PlanVerdict::Exact } else { PlanVerdict::Differs },
            None,
        ),
    }

    // ---- `|observed emit set|` — a fact about the REFERENCE obj -------------
    //
    // Recorded for every observable TU and NOT behind the port's silence. See
    // [`TuPlan::emitset_obs_size`]: gating it on the `Known` arm turned a
    // 870-TU quantity into an 854-TU one under a whole-workload caption.
    t.emitset_obs_size = Some(observed_set.len());

    // ---- the `0x20` SEED, as a characterization value -----------------------
    //
    // NOT a component (see [`PLAN_COMPONENTS`]). Measured on every TU where the
    // reader answers, published as `plan-emitset-seed-*`, and never as a claim
    // that the port disagrees with c2.
    if let Predicted::Known(p) = &predicted.gl_seed_members {
        let pred: BTreeSet<&str> = p.iter().map(String::as_str).collect();
        let extra = pred.difference(&observed_set).count();
        let missing = observed_set.difference(&pred).count();
        t.emitset_extra = Some(extra);
        t.emitset_missing = Some(missing);
        t.emitset_subset = Some(extra == 0);
        // The claimant's own size, beside the containment claim.
        t.emitset_pred_size = Some(pred.len());
        bump(&mut t, "plan-glattr-seed", pred.len());
        t.emitset_missing_witness =
            observed_set.difference(&pred).next().map(|s| (*s).to_string());
        t.emitset_extra_witness =
            pred.difference(&observed_set).next().map(|s| (*s).to_string());
    }

    // ---- THE ALTERNATIVE EXPLANATION FOR 28,107 vs 158,802 ------------------
    //
    // Both intersections against the reference obj's own emitted set, per TU.
    // See [`PLAN_OBSERVED_KEYS`]: this is the measurement that separates *"the
    // bit is rare in `.gl`"* from *"the scanner reaches one record in six"*,
    // and the first version of the probe took neither.
    if let Predicted::Known(names) = &predicted.gl_attr_names {
        let n = names.iter().filter(|s| observed_set.contains(s.as_str())).count();
        bump(&mut t, "plan-glattr-in-emitset", n);
    }
    bump(&mut t, "plan-glruns-names", predicted.gl_run_names.len());
    bump(
        &mut t,
        "plan-glruns-in-emitset",
        predicted
            .gl_run_names
            .iter()
            .filter(|s| observed_set.contains(s.as_str()))
            .count(),
    );

    // ---- the REFUTED `.gl`-record-order rule, as a characterization value ---
    //
    // NOT a component (see [`PLAN_COMPONENTS`]). Kept because it is the number
    // that prices board #259's `coff::order::plan_text_order`: whatever builds
    // an order model has to beat this.
    if let Predicted::Known(p) = &predicted.gl_record_order {
        t.glorder = Some(
            p.len() == obs.emit_set.len() && p.iter().zip(&obs.emit_set).all(|(a, b)| a == b),
        );
    }

    (t.violations, t.checks_reached) = t.bounds_violations();
    t
}

/// **R2 AT WORKLOAD SCALE** — `observe`'s emit set against the incumbent
/// `text_comdat_functions` walk, on every TU that captured.
///
/// The prereg's tertiary criterion asks for agreement with each existing
/// accessor *"over the whole workload, TU by TU"*; what shipped was three
/// synthetic cells. This carries the emit-set half over the real population.
/// The other three accessors (`section_names`, `weak_externals`,
/// `text_comdat_relocs_named`) are still agreed on those three objs only, and
/// that shortfall is stated in the rung rather than papered over — this is the
/// one that mattered, because `plan-emitset-observed-size` is published off the
/// new walk and read as a denominator.
///
/// **Compared as SETS, deliberately.** `observe` builds the emit set in
/// SECTION-table order and `text_comdat_entries` in SYMBOL-table order; they
/// coincide on the three synthetic objs and are not guaranteed to on a real one,
/// so an ordered comparison here would report an ordering fact as a membership
/// disagreement. Known answer for the set comparison: **0**.
pub fn record_accessor_agreement(
    t: &mut TuPlan,
    observed: Option<&ObjPlan>,
    incumbent: Option<Vec<String>>,
) {
    let (Some(obs), Some(inc)) = (observed, incumbent) else { return };
    // Only bump keys the observable branch pre-seeded, or the counter would be
    // an insert on a TU `grade` decided not to describe.
    if !t.obs.contains_key("plan-agree-emitset-tus") {
        return;
    }
    *t.obs.get_mut("plan-agree-emitset-tus").expect("pre-seeded") += 1;
    let ours: BTreeSet<&str> = obs.emit_set.iter().map(String::as_str).collect();
    let theirs: BTreeSet<&str> = inc.iter().map(String::as_str).collect();
    if ours != theirs {
        *t.obs.get_mut("plan-agree-emitset-disagree").expect("pre-seeded") += 1;
    }
}

// ---------------------------------------------------------------------------
// The offline view — #3288's second, differently-built derivation
// ---------------------------------------------------------------------------

/// One row of `--plan-tsv`: the per-TU membership the counts are counts **of**.
///
/// Two producers, one definition, in `gap::sets`' shape: the live scan builds
/// these from [`TuPlan`], and [`parse_plan_tsv`] builds them from the file. The
/// round-trip is graded by a test, so a count re-derived from the file is a
/// count re-derived from *that scan* and not from a lookalike.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlanRow {
    pub src: String,
    pub class: String,
    pub observable: bool,
    /// Component → verdict, over [`PLAN_COMPONENTS`] in that order.
    pub verdicts: Vec<PlanVerdict>,
    pub subset: Option<bool>,
    pub extra: Option<usize>,
    pub missing: Option<usize>,
    /// `|predicted|` / `|observed|` — the containment claim's own denominators.
    pub pred_size: Option<usize>,
    pub obs_size: Option<usize>,
    /// The refuted `.gl`-record-order rule's verdict. A characterization value.
    pub glorder: Option<bool>,
    pub violations: usize,
    /// Containment checks EVALUATED on this TU — the denominator
    /// `plan-bounds-violations` was published without. See
    /// [`TuPlan::checks_reached`].
    pub checks: usize,
}

impl PlanRow {
    pub fn verdict(&self, component: &str) -> Option<PlanVerdict> {
        let i = PLAN_COMPONENTS.iter().position(|c| *c == component)?;
        self.verdicts.get(i).copied()
    }
}

/// Render the rows as the `--plan-tsv` body. Pure, so the round-trip test needs
/// no filesystem and no toolchain.
pub fn plan_tsv(rows: &[PlanRow]) -> String {
    let mut s = String::from(
        "# c2rs gap --plan-tsv — per-TU OBJECT PLAN grade (lane w-objplan)\n\
         # The plan is everything about the output obj that is INDEPENDENT OF THE\n\
         # INSTRUCTION BYTES. `predict(IL)` is the port's plan, computed without\n\
         # emitting; `observe(ref obj)` is real c2's. See crates/c2-obj/src/plan.rs.\n\
         #\n\
         # A VERDICT IS ONE OF FOUR AND THEY ARE A PARTITION:\n\
         #   exact         both sides answered and agree\n\
         #   differs       both sides answered and disagree\n\
         #   unknown       the PORT did not look (the reason names the stage)\n\
         #   unobservable  the REFERENCE obj did not decode — no ground truth\n\
         #\n\
         # THIS IS AN INSTRUMENT AND NOT A GATE. `exact` here is NECESSARY but NOT\n\
         # SUFFICIENT for `match`: a TU can be plan-exact and mismatch on every byte.\n\
         #\n\
         # ROWS ARE THE GRADED TUs ONLY. A `capture-fail` TU has no reference obj, so\n\
         # it is NOT a row — it was never measured, which is a different fact from\n\
         # every component being wrong. Do not read its absence as a failing row.\n",
    );
    s.push_str("# columns: src\tclass\tobservable");
    for c in PLAN_COMPONENTS {
        s.push('\t');
        s.push_str(c);
    }
    s.push_str(
        "\tsubset\textra\tmissing\tpred-size\tobs-size\tglorder\tviolations\tchecks\n",
    );
    s.push_str(&format!("# plan-rows {}\n", rows.len()));
    let opt = |o: Option<usize>| match o {
        Some(n) => n.to_string(),
        None => "-".to_string(),
    };
    for r in rows {
        s.push_str(&r.src);
        s.push('\t');
        s.push_str(&r.class);
        s.push('\t');
        s.push_str(if r.observable { "1" } else { "0" });
        for v in &r.verdicts {
            s.push('\t');
            s.push_str(v.label());
        }
        s.push('\t');
        s.push_str(match r.subset {
            Some(true) => "1",
            Some(false) => "0",
            None => "-",
        });
        s.push('\t');
        s.push_str(&opt(r.extra));
        s.push('\t');
        s.push_str(&opt(r.missing));
        s.push('\t');
        s.push_str(&opt(r.pred_size));
        s.push('\t');
        s.push_str(&opt(r.obs_size));
        s.push('\t');
        s.push_str(match r.glorder {
            Some(true) => "1",
            Some(false) => "0",
            None => "-",
        });
        s.push('\t');
        s.push_str(&r.violations.to_string());
        s.push('\t');
        s.push_str(&r.checks.to_string());
        s.push('\n');
    }
    s
}

/// Parse a `--plan-tsv` file back into rows — the **offline** producer.
///
/// Fail-closed on a row it cannot read: a parser that skipped a malformed row
/// would re-derive a smaller count than the scan published and the control
/// would read `DISAGREE` for a reason nobody could locate. `None` means *this
/// file is not one of ours*, never *this file has fewer rows*.
pub fn parse_plan_tsv(text: &str) -> Option<Vec<PlanRow>> {
    let n = PLAN_COMPONENTS.len();
    let mut out = Vec::new();
    for line in text.lines() {
        if line.starts_with('#') || line.trim().is_empty() {
            continue;
        }
        let f: Vec<&str> = line.split('\t').collect();
        if f.len() != 3 + n + 8 {
            return None;
        }
        let verdicts: Option<Vec<PlanVerdict>> =
            f[3..3 + n].iter().map(|s| PlanVerdict::parse(s)).collect();
        let num = |s: &str| -> Option<Option<usize>> {
            if s == "-" {
                Some(None)
            } else {
                s.parse::<usize>().ok().map(Some)
            }
        };
        out.push(PlanRow {
            src: f[0].to_string(),
            class: f[1].to_string(),
            observable: match f[2] {
                "1" => true,
                "0" => false,
                _ => return None,
            },
            verdicts: verdicts?,
            subset: match f[3 + n] {
                "1" => Some(true),
                "0" => Some(false),
                "-" => None,
                _ => return None,
            },
            extra: num(f[4 + n])?,
            missing: num(f[5 + n])?,
            pred_size: num(f[6 + n])?,
            obs_size: num(f[7 + n])?,
            glorder: match f[8 + n] {
                "1" => Some(true),
                "0" => Some(false),
                "-" => None,
                _ => return None,
            },
            violations: f[9 + n].parse().ok()?,
            checks: f[10 + n].parse().ok()?,
        });
    }
    Some(out)
}

/// **The second derivation** (#3288): re-derive every published `plan-*` count
/// from the rows, offline, and hand the caller `(key, derived)` pairs to diff
/// against what the scan printed.
///
/// Every lane that has run this check has found a wrong figure, *including*
/// over-counts that were wrong at their own base. It is cheap and it is not
/// optional.
pub fn derive_metrics(rows: &[PlanRow]) -> BTreeMap<String, usize> {
    let mut m: BTreeMap<String, usize> = BTreeMap::new();
    m.insert("plan-observable".into(), rows.iter().filter(|r| r.observable).count());
    for (i, c) in PLAN_COMPONENTS.iter().enumerate() {
        let v = |r: &PlanRow| r.verdicts[i];
        m.insert(
            format!("plan-{c}-observable"),
            rows.iter().filter(|r| v(r) != PlanVerdict::Unobservable).count(),
        );
        m.insert(
            format!("plan-{c}-known"),
            rows.iter()
                .filter(|r| matches!(v(r), PlanVerdict::Exact | PlanVerdict::Differs))
                .count(),
        );
        m.insert(
            format!("plan-{c}-exact"),
            rows.iter().filter(|r| v(r) == PlanVerdict::Exact).count(),
        );
        m.insert(
            format!("plan-{c}-differs"),
            rows.iter().filter(|r| v(r) == PlanVerdict::Differs).count(),
        );
    }
    m.insert(
        "plan-emitset-seed-subset".into(),
        rows.iter().filter(|r| r.subset == Some(true)).count(),
    );
    m.insert(
        "plan-emitset-seed-known".to_string(),
        rows.iter().filter(|r| r.pred_size.is_some()).count(),
    );
    m.insert(
        "plan-emitset-seed-extra".to_string(),
        rows.iter().filter_map(|r| r.extra).sum(),
    );
    m.insert(
        "plan-emitset-seed-missing".to_string(),
        rows.iter().filter_map(|r| r.missing).sum(),
    );
    // The seed's own agreement rate — the number `plan-emitset-members-exact`
    // used to carry before the component was withdrawn under §3. Published as
    // CHARACTERIZATION, never as a curve.
    m.insert(
        "plan-emitset-seed-exact".to_string(),
        rows.iter()
            .filter(|r| r.extra == Some(0) && r.missing == Some(0))
            .count(),
    );
    // **AND THE SAME NUMBER WITH THE VACUOUS CELLS REMOVED.** 6 of the seed's
    // `exact` TUs compare the EMPTY SET to the EMPTY SET — the reference obj
    // emits nothing there — so "the seed is exact on 33 TUs" and "27 substantive
    // agreements plus 6 empty comparisons" print identically. The review found
    // this on the control and it is the same defect on the workload figure; the
    // repair is the same one §2.3 already applied to the containment claim.
    m.insert(
        "plan-emitset-seed-exact-substantive".to_string(),
        rows.iter()
            .filter(|r| {
                r.extra == Some(0) && r.missing == Some(0) && r.obs_size.unwrap_or(0) > 0
            })
            .count(),
    );
    // **The containment claim's own denominators**, without which "seed ⊆
    // emitted on N TUs" cannot be told apart from "N empty seeds".
    m.insert(
        "plan-emitset-seed-size".to_string(),
        rows.iter().filter_map(|r| r.pred_size).sum(),
    );
    m.insert(
        "plan-emitset-observed-size".to_string(),
        rows.iter().filter_map(|r| r.obs_size).sum(),
    );
    // The same sum restricted to the TUs where the SEED also answered — the
    // population every seed-coverage ratio has to be taken over. Publishing only
    // the restricted sum under a whole-workload caption is what made the
    // 158,802-vs-162,136 discrepancy invisible.
    m.insert(
        "plan-emitset-observed-size-known".to_string(),
        rows.iter()
            .filter(|r| r.pred_size.is_some())
            .filter_map(|r| r.obs_size)
            .sum(),
    );
    m.insert(
        "plan-emitset-seed-empty-tus".to_string(),
        rows.iter().filter(|r| r.pred_size == Some(0)).count(),
    );
    m.insert(
        "plan-emitset-observed-empty-tus".to_string(),
        rows.iter().filter(|r| r.obs_size == Some(0)).count(),
    );
    // The REFUTED `.gl`-order rule, published as characterization and never as
    // a component. See [`PLAN_COMPONENTS`].
    m.insert(
        "plan-emitset-glorder-known".to_string(),
        rows.iter().filter(|r| r.glorder.is_some()).count(),
    );
    m.insert(
        "plan-emitset-glorder-agrees".to_string(),
        rows.iter().filter(|r| r.glorder == Some(true)).count(),
    );
    m.insert(
        "plan-bounds-violations".into(),
        rows.iter().map(|r| r.violations).sum(),
    );
    m.insert(
        "plan-bounds-checks-reached".into(),
        rows.iter().map(|r| r.checks).sum(),
    );
    m.extend(derive_control_metrics(rows));
    m
}

/// **The NAMED control, re-derived from the rows** — the half of #3288 the first
/// version left out.
///
/// `derive_metrics` covered 13 of the 48 published keys and the omissions
/// included **the primary grading criterion**. Every `plan-control-*` figure is
/// a join of the rows against `docs/plan/CONTROL_TUS.txt`, which is compiled in;
/// the review re-derived it in four lines of `awk`, which is the definition of
/// cheap. Kept a separate function so a caller can say which half it is
/// checking.
///
/// What is still NOT second-derived, stated rather than left to be discovered:
/// the 24 `plan-obs-*` / `plan-glattr-*` / `plan-glruns-*` inventory keys are
/// not columns in the TSV at all, so no parser over it can reach them.
/// [`uncovered_metric_keys`] names them and a unit test asserts the list is
/// exactly the uncovered set, so the gap is a printed number rather than a
/// silence.
pub fn derive_control_metrics(rows: &[PlanRow]) -> BTreeMap<String, usize> {
    let pinned = control_tus();
    let mut m: BTreeMap<String, usize> = BTreeMap::new();
    let found: BTreeSet<&str> = rows
        .iter()
        .filter(|r| r.class == "match")
        .map(|r| r.src.as_str())
        .collect();
    let present: Vec<&PlanRow> = rows
        .iter()
        .filter(|r| pinned.contains(r.src.as_str()))
        .collect();
    let entered = found.iter().filter(|s| !pinned.contains(*s)).count();
    let left = pinned.iter().filter(|s| !found.contains(*s)).count();
    m.insert("plan-control-pinned".into(), pinned.len());
    m.insert("plan-control-found".into(), found.len());
    m.insert("plan-control-diff".into(), entered + left);
    m.insert("plan-control-present".into(), present.len());
    let (mut exact_rows, mut unknown_cells, mut differ_cells) = (0, 0, 0);
    for r in &present {
        let mut ok = true;
        for v in &r.verdicts {
            match v {
                PlanVerdict::Exact => {}
                PlanVerdict::Unknown => {
                    unknown_cells += 1;
                    ok = false;
                }
                _ => {
                    differ_cells += 1;
                    ok = false;
                }
            }
        }
        if ok {
            exact_rows += 1;
        }
    }
    m.insert("plan-control-exact".into(), exact_rows);
    m.insert("plan-control-unknown".into(), unknown_cells);
    m.insert("plan-control-differs".into(), differ_cells);
    m.insert("plan-control-shortfall".into(), unknown_cells + differ_cells);
    // **THE CONTROL'S OWN SIZES** — see `GapReport::plan_control`. Without these
    // "24 of 26 exact" cannot be told apart from 24 empty comparisons.
    m.insert(
        "plan-control-obs-size".into(),
        present.iter().filter_map(|r| r.obs_size).sum(),
    );
    m.insert(
        "plan-control-obs-empty-tus".into(),
        present.iter().filter(|r| r.obs_size == Some(0)).count(),
    );
    m.insert(
        "plan-control-substantive-tus".into(),
        present
            .iter()
            .filter(|r| r.obs_size.unwrap_or(0) >= 2)
            .count(),
    );
    m
}

/// **The published `plan-*` keys the second derivation CANNOT reach**, named.
///
/// A coverage claim without its complement is #3237 one level up: "13 OK" reads
/// as done. These are the observe-side inventory counters, which are sums over
/// the reference obj and are not columns in `--plan-tsv`. Reaching them would
/// mean widening the TSV to one row per COMDAT section — 350,520 rows — which is
/// priced in the rung and declined.
pub fn uncovered_metric_keys() -> Vec<&'static str> {
    let mut v: Vec<&'static str> = PLAN_OBSERVED_KEYS.to_vec();
    v.extend([
        "plan-obs-sections-names-distinct",
        "plan-obs-sections-attrs-distinct",
        "plan-obs-drectve-distinct",
    ]);
    v.sort_unstable();
    v
}

// ---------------------------------------------------------------------------
// The NAMED control
// ---------------------------------------------------------------------------

/// **The control TUs, pinned BY NAME** — `docs/plan/CONTROL_TUS.txt`, embedded
/// at compile time.
///
/// # Why a file of names and not a count
///
/// A control pinned by *count* passes in an unprovisioned worktree the moment
/// the count matches (rungs/README rule 1, boards **#3219**/**#3231**): a fresh
/// `git worktree add` has no `compilers/`, so every capture SKIPs, the
/// red-maker reports *"3 passed"* in 0.00 s, and cargo swallows the SKIP line
/// for a passing test — a registered RED reads GREEN with a clean suite, the
/// right target count and the right exit code. **A count match with a set
/// difference is a finding.**
///
/// # Why it is `include_str!` and not a file read
///
/// The pin must be in the binary, not on the filesystem: a scan run from a tree
/// where the file is absent would otherwise silently control against nothing,
/// which is the same absence-as-success shape one layer down. It also keeps the
/// harness free of an absolute path (CLAUDE.md).
///
/// # What it is NOT
///
/// It is not a claim that these 26 TUs are the match set **now**. The set is
/// EXPECTED to move as the port converts TUs; what the lane asserts is the
/// **identity diff** — entered and left, by name — printed on every run. A
/// difference is reported before any other number, because it means the tree or
/// the workload stamp moved under the reader.
pub fn control_tus() -> BTreeSet<&'static str> {
    include_str!("../../../../docs/plan/CONTROL_TUS.txt")
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .collect()
}

/// The identity diff between the pinned control and the set a scan found.
#[derive(Clone, Debug, Default)]
pub struct ControlDiff<'a> {
    /// In the scan's `match` set and not in the pinned file.
    pub entered: Vec<&'a str>,
    /// In the pinned file and not in the scan's `match` set.
    pub left: Vec<&'static str>,
    /// `|pinned|` and `|found|`, printed beside the diff so a reader never has
    /// to infer a denominator.
    pub pinned: usize,
    pub found: usize,
}

impl ControlDiff<'_> {
    pub fn agrees(&self) -> bool {
        self.entered.is_empty() && self.left.is_empty()
    }
}

/// Diff the pinned control against the `match` set this scan found.
pub fn control_diff<'a, I: IntoIterator<Item = &'a str>>(found: I) -> ControlDiff<'a> {
    let pinned = control_tus();
    let found: BTreeSet<&'a str> = found.into_iter().collect();
    ControlDiff {
        entered: found.iter().filter(|s| !pinned.contains(*s)).copied().collect(),
        left: pinned.iter().filter(|s| !found.contains(*s)).copied().collect(),
        pinned: pinned.len(),
        found: found.len(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use c2_core::plan::PredictedPlan;

    fn unknown_plan() -> PredictedPlan {
        PredictedPlan {
            emit_set_members: Predicted::Unknown("gl-attrs-refused"),
            emit_set_order: Predicted::Unknown("plan-order-unmodelled"),
            gl_seed_members: Predicted::Unknown("gl-attrs-refused"),
            gl_attr_names: Predicted::Unknown("gl-attrs-refused"),
            gl_run_names: BTreeSet::new(),
            gl_record_order: Predicted::Unknown("gl-attrs-refused"),
            sections: Predicted::Unknown("plan-sections-unmodelled"),
            weak: Predicted::Unknown("plan-weak-unmodelled"),
            undef: Predicted::Unknown("plan-undef-unmodelled"),
            attr_census: None,
        }
    }

    /// **The partition.** With no reference obj every component is
    /// `Unobservable` — never `Unknown`, which would blame the port for the
    /// extractor's silence, and never `Differs`, which would claim a
    /// disagreement with a value that was never read.
    #[test]
    fn no_reference_obj_is_unobservable_and_not_unknown() {
        let t = grade(None, &unknown_plan());
        assert!(!t.observable);
        for c in PLAN_COMPONENTS {
            assert_eq!(t.verdicts[*c], PlanVerdict::Unobservable);
        }
        assert!(t.violations.is_empty());
    }

    /// A row that says `unknown` carries the REASON, and the reason is the
    /// census key naming the stage that owes the work.
    #[test]
    fn an_unknown_component_names_the_stage_that_owes_it() {
        let obs = super::tests_support::specimen_plan();
        let t = grade(Some(&obs), &unknown_plan());
        assert_eq!(t.verdicts["emitset-members"], PlanVerdict::Unknown);
        assert_eq!(t.reasons["emitset-members"], "gl-attrs-refused");
    }

    #[test]
    fn the_tsv_round_trips() {
        let rows = vec![
            PlanRow {
                src: "src/App.cpp".into(),
                class: "vocab-gap".into(),
                observable: true,
                verdicts: vec![PlanVerdict::Differs, PlanVerdict::Unknown],
                subset: Some(false),
                extra: Some(3),
                missing: Some(7),
                pred_size: Some(9),
                obs_size: Some(13),
                glorder: Some(false),
                violations: 0,
                checks: 2,
            },
            PlanRow {
                src: "src/b.cpp".into(),
                class: "match".into(),
                observable: false,
                verdicts: vec![PlanVerdict::Unobservable, PlanVerdict::Unobservable],
                subset: None,
                extra: None,
                missing: None,
                pred_size: None,
                obs_size: None,
                glorder: None,
                violations: 0,
                checks: 1,
            },
        ];
        // The fixture's verdict width must FOLLOW the component list, or a
        // component added or demoted turns this into a width mismatch that
        // reads as a parser bug.
        for r in &rows {
            assert_eq!(r.verdicts.len(), PLAN_COMPONENTS.len());
        }
        let back = parse_plan_tsv(&plan_tsv(&rows)).expect("our own file must parse");
        assert_eq!(rows, back);
    }

    /// A malformed row FAILS the parse rather than being skipped: a skipped row
    /// re-derives a smaller count than the scan published, and the control then
    /// reads DISAGREE for a reason nobody can locate.
    #[test]
    fn a_malformed_row_refuses_the_whole_file() {
        assert!(parse_plan_tsv("a\tb\tc\n").is_none());
        // Right column count, one unreadable verdict token.
        assert!(
            parse_plan_tsv("a\tmatch\t1\tNOPE\texact\t1\t0\t0\t1\t1\t1\t0\t1\n").is_none()
        );
        // …and the WIDTH is checked in the positive direction too: the same row
        // with every token readable must parse, or the negative case above could
        // be passing on the column count rather than on the token.
        assert!(
            parse_plan_tsv("a\tmatch\t1\texact\texact\t1\t0\t0\t1\t1\t1\t0\t1\n").is_some()
        );
    }

    /// The key table and the component list are one table in two places, so
    /// they are checked against each other rather than trusted.
    #[test]
    fn the_key_table_and_the_component_list_agree() {
        assert_eq!(PLAN_KEYS.len(), PLAN_COMPONENTS.len());
        for (k, c) in PLAN_KEYS.iter().zip(PLAN_COMPONENTS) {
            assert_eq!(k.component, *c);
            for name in [k.observable, k.known, k.exact, k.differs, k.distinct] {
                assert!(
                    name.starts_with(&format!("plan-{c}-")),
                    "{name} does not belong to component {c}"
                );
            }
        }
    }

    /// The pinned control is a file of NAMES and it must be non-empty, or the
    /// control is a control over nothing — which passes silently.
    #[test]
    fn the_pinned_control_is_a_nonempty_set_of_paths() {
        let c = control_tus();
        assert!(!c.is_empty(), "docs/plan/CONTROL_TUS.txt parsed to nothing");
        for n in &c {
            assert!(
                n.starts_with("src/") && n.ends_with(".cpp"),
                "control entry {n} is not a `cl`-spelled source path — the join key \
                 is the path as cl sees it, never a filename and never absolute"
            );
        }
    }

    /// A set that equals the pin agrees; one that differs by a single name
    /// reports that name in the right direction.
    #[test]
    fn the_control_diff_names_both_directions() {
        let pinned: Vec<&str> = control_tus().into_iter().collect();
        assert!(control_diff(pinned.iter().copied()).agrees());
        let mut short = pinned.clone();
        let dropped = short.pop().unwrap();
        let d = control_diff(short.iter().copied());
        assert!(!d.agrees());
        assert_eq!(d.left, vec![dropped]);
        assert!(d.entered.is_empty());
    }

    /// The containment control fires on the shape it is there to catch — **and
    /// reports how many checks it REACHED**, which is the repair for a zero that
    /// could not be told apart from not looking.
    #[test]
    fn bounds_violations_catch_an_impossible_ladder() {
        let mut t = TuPlan {
            observable: true,
            ..Default::default()
        };
        t.verdicts.insert("emitset-order".into(), PlanVerdict::Exact);
        t.verdicts.insert("emitset-members".into(), PlanVerdict::Differs);
        t.emitset_subset = Some(true);
        let (v, reached) = t.bounds_violations();
        assert!(v.contains(&"order-exact-without-members-exact".to_string()));
        // The ladder check and the members-exact check both answered; the seed
        // check did not (no sizes) and neither did the ground-truth one.
        assert_eq!(reached, 2, "reached: {v:?}");
    }

    /// **A DORMANT CHECK IS VISIBLY DORMANT.** With both components shipping
    /// `Unknown` — the state the registered §3 rule puts them in — the ladder
    /// check cannot fire, and the reached counter is what says so instead of a
    /// bare `0 violations`.
    #[test]
    fn a_check_that_cannot_fire_is_counted_as_not_reached() {
        let mut t = TuPlan {
            observable: true,
            ..Default::default()
        };
        t.verdicts.insert("emitset-order".into(), PlanVerdict::Unknown);
        t.verdicts.insert("emitset-members".into(), PlanVerdict::Unknown);
        let (v, reached) = t.bounds_violations();
        assert!(v.is_empty());
        assert_eq!(
            reached, 0,
            "neither component answered, so no containment check could fire — a \
             `0 violations` here must NOT be reported as a check that passed"
        );
        // …and the seed check IS reached once the characterization sizes exist,
        // which is the one of the four that runs on a real TU today.
        t.emitset_extra = Some(0);
        t.emitset_missing = Some(0);
        let (v, reached) = t.bounds_violations();
        assert_eq!(reached, 1);
        assert_eq!(
            v,
            vec!["seed-exact-without-subset".to_string()],
            "an exact seed whose subset flag is unset is a grader bug and must fire"
        );
    }

    /// **The published control keys and the offline re-derivation of them are
    /// two producers of one number** — #3288 applied to the primary grading
    /// criterion, which the first version of `derive_metrics` did not cover.
    #[test]
    fn the_control_is_re_derivable_from_the_rows() {
        let pinned: Vec<&str> = control_tus().into_iter().collect();
        let rows: Vec<PlanRow> = pinned
            .iter()
            .enumerate()
            .map(|(i, src)| PlanRow {
                src: (*src).to_string(),
                class: "match".into(),
                observable: true,
                verdicts: vec![PlanVerdict::Unknown, PlanVerdict::Unknown],
                subset: Some(true),
                extra: Some(0),
                missing: Some(0),
                pred_size: Some(0),
                // Half the cells empty, half substantive — so the size keys
                // below are checked against a population that has both.
                obs_size: Some(if i % 2 == 0 { 0 } else { 4 }),
                glorder: Some(false),
                violations: 0,
                checks: 1,
            })
            .collect();
        let m = derive_control_metrics(&rows);
        assert_eq!(m["plan-control-pinned"], pinned.len());
        assert_eq!(m["plan-control-present"], pinned.len());
        assert_eq!(m["plan-control-diff"], 0);
        assert_eq!(m["plan-control-exact"], 0);
        assert_eq!(m["plan-control-unknown"], pinned.len() * 2);
        assert_eq!(m["plan-control-differs"], 0);
        // **THE SIZES.** `exact 0 of 26` and `26 empty comparisons` would print
        // identically without these.
        assert_eq!(m["plan-control-obs-empty-tus"], pinned.len().div_ceil(2));
        assert_eq!(m["plan-control-substantive-tus"], pinned.len() / 2);
        assert_eq!(m["plan-control-obs-size"], 4 * (pinned.len() / 2));
    }

    /// The keys the second derivation cannot reach are NAMED, and the name list
    /// is exactly the uncovered set — a coverage claim whose complement is not
    /// enumerated is #3237 one level up.
    #[test]
    fn the_uncovered_key_list_is_exactly_the_keys_the_tsv_cannot_reach() {
        let uncovered: BTreeSet<&str> = uncovered_metric_keys().into_iter().collect();
        let derived = derive_metrics(&[]);
        for k in &uncovered {
            assert!(
                !derived.contains_key(*k),
                "`{k}` is listed as unreachable by the second derivation and the \
                 second derivation derives it"
            );
        }
        assert!(
            uncovered.contains("plan-glattr-names") && uncovered.contains("plan-obs-sections"),
            "the list must actually name the inventory keys"
        );
    }
}

#[cfg(test)]
pub(crate) mod tests_support {
    use c2_obj::ObjPlan;

    /// A plan with nothing in it but a decodable shell — enough for the
    /// grader's own tests, which are about the VERDICTS and not about COFF.
    pub fn specimen_plan() -> ObjPlan {
        ObjPlan {
            sections: Vec::new(),
            emit_set: vec!["?f@@YAXXZ".to_string()],
            symbols: Vec::new(),
            weak: Vec::new(),
            undef: Vec::new(),
            relocs: Vec::new(),
            drectve: Vec::new(),
        }
    }
}
