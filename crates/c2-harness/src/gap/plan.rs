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
    "plan-obs-comdat-sel-nonduplicates",
    "plan-obs-comdat-sel-any",
    "plan-obs-comdat-sel-other",
    "plan-obs-undef-records",
    "plan-obs-undef-tus",
    "plan-obs-sections",
    "plan-obs-drectve-tus",
    "plan-obs-reloc-records",
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
    /// Containment violations found on this TU (see
    /// [`TuPlan::bounds_violations`]). MUST be empty.
    pub violations: Vec<String>,
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
    fn bounds_violations(&self) -> Vec<String> {
        let mut v = Vec::new();
        let g = |k: &str| self.verdicts.get(k).copied();
        if g("emitset-order") == Some(PlanVerdict::Exact)
            && g("emitset-members") != Some(PlanVerdict::Exact)
        {
            v.push("order-exact-without-members-exact".to_string());
        }
        if g("emitset-members") == Some(PlanVerdict::Exact) && self.emitset_subset != Some(true) {
            v.push("members-exact-without-subset".to_string());
        }
        if !self.observable {
            for (k, verdict) in &self.verdicts {
                if matches!(verdict, PlanVerdict::Exact | PlanVerdict::Differs) {
                    v.push(format!("graded-without-ground-truth:{k}"));
                }
            }
        }
        v
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
        t.violations = t.bounds_violations();
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
        match c.selection {
            1 => bump(&mut t, "plan-obs-comdat-sel-nonduplicates", 1),
            2 => bump(&mut t, "plan-obs-comdat-sel-any", 1),
            _ => bump(&mut t, "plan-obs-comdat-sel-other", 1),
        }
        if c.assoc.is_some() {
            assoc_here += 1;
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

    // ---- emit set, members -------------------------------------------------
    let observed_set: BTreeSet<&str> = obs.emit_set.iter().map(String::as_str).collect();
    match &predicted.emit_set_members {
        Predicted::Unknown(r) => record(&mut t, "emitset-members", PlanVerdict::Unknown, Some(r)),
        Predicted::Known(p) => {
            let pred: BTreeSet<&str> = p.iter().map(String::as_str).collect();
            let extra = pred.difference(&observed_set).count();
            let missing = observed_set.difference(&pred).count();
            t.emitset_extra = Some(extra);
            t.emitset_missing = Some(missing);
            t.emitset_subset = Some(extra == 0);
            record(
                &mut t,
                "emitset-members",
                if extra == 0 && missing == 0 {
                    PlanVerdict::Exact
                } else {
                    PlanVerdict::Differs
                },
                None,
            );
        }
    }

    // ---- emit set, order ---------------------------------------------------
    match &predicted.emit_set_order {
        Predicted::Unknown(r) => record(&mut t, "emitset-order", PlanVerdict::Unknown, Some(r)),
        Predicted::Known(p) => {
            let same = p.len() == obs.emit_set.len()
                && p.iter().zip(&obs.emit_set).all(|(a, b)| a == b);
            record(
                &mut t,
                "emitset-order",
                if same {
                    PlanVerdict::Exact
                } else {
                    PlanVerdict::Differs
                },
                None,
            );
        }
    }

    t.violations = t.bounds_violations();
    t
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
    pub violations: usize,
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
    s.push_str("\tsubset\textra\tmissing\tviolations\n");
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
        s.push_str(&r.violations.to_string());
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
        if f.len() != 3 + n + 4 {
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
            violations: f[6 + n].parse().ok()?,
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
        "plan-bounds-violations".into(),
        rows.iter().map(|r| r.violations).sum(),
    );
    m
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
            emit_set_order: Predicted::Unknown("gl-attrs-refused"),
            sections: Predicted::Unknown("plan-sections-unmodelled"),
            weak: Predicted::Unknown("plan-weak-unmodelled"),
            undef: Predicted::Unknown("plan-undef-unmodelled"),
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
                verdicts: vec![PlanVerdict::Differs, PlanVerdict::Differs],
                subset: Some(false),
                extra: Some(3),
                missing: Some(7),
                violations: 0,
            },
            PlanRow {
                src: "src/b.cpp".into(),
                class: "match".into(),
                observable: false,
                verdicts: vec![PlanVerdict::Unobservable, PlanVerdict::Unobservable],
                subset: None,
                extra: None,
                missing: None,
                violations: 0,
            },
        ];
        let back = parse_plan_tsv(&plan_tsv(&rows)).expect("our own file must parse");
        assert_eq!(rows, back);
    }

    /// A malformed row FAILS the parse rather than being skipped: a skipped row
    /// re-derives a smaller count than the scan published, and the control then
    /// reads DISAGREE for a reason nobody can locate.
    #[test]
    fn a_malformed_row_refuses_the_whole_file() {
        assert!(parse_plan_tsv("a\tb\tc\n").is_none());
        assert!(parse_plan_tsv("a\tmatch\t1\texact\tNOPE\t1\t0\t0\t0\n").is_none());
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

    /// The containment control fires on the shape it is there to catch.
    #[test]
    fn bounds_violations_catch_an_impossible_ladder() {
        let mut t = TuPlan {
            observable: true,
            ..Default::default()
        };
        t.verdicts.insert("emitset-order".into(), PlanVerdict::Exact);
        t.verdicts.insert("emitset-members".into(), PlanVerdict::Differs);
        t.emitset_subset = Some(true);
        assert!(t
            .bounds_violations()
            .contains(&"order-exact-without-members-exact".to_string()));
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
