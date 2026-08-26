//! **The Phase-7 factor sets as SETS, and the join that intersects them with
//! somebody else's per-TU list** (lane `w-bcgap`, boards **#1520**–**#1524**).
//!
//! # The number this exists for
//!
//! `_2026-08-04-w-emitp-findings.md` §5 ends by declining a multiplication:
//!
//! > Board #213 prices a *perfect* emit predicate at **+124 by reach**
//! > (`B∧C − A∧B∧C` = 151 − 27). This predicate is not perfect — it is exact on
//! > 55.5 % of graded TUs — so the reach it buys is `|{TU : model exact} ∩ B∧C|`
//! > […] Multiplying 151 by 0.555 would be exactly the error that left `B∧C`
//! > stale at 107 for weeks, so it is **not done**.
//!
//! Half of what that needed landed the next day: [`super::GapReport::factor_tsv`]
//! (`c2rs gap --factors-tsv`, board **#352**) publishes the per-TU A/B/C/D/E
//! membership, so `B∧C` *is* a list now and not only a count.
//!
//! **The half that did not land is the join**, and its failure mode is the
//! reason this module is not a shell one-liner. The two sides are joined on a
//! **source path string**. A candidate set carried out of a python lane spells
//! its TUs `src__App.cpp` (a filename), a scan spells them `src/App.cpp` (the
//! argument passed to `cl`), and something that walked a tree spells them
//! absolutely. Join those with `comm` or `join` and you get **0**, and
//! `|model ∩ B∧C| = 0` reads as *"this model buys no reach"* — a measurement —
//! when what happened is that the key was wrong. That is the same shape as the
//! column that was zero by construction, the 130 575 reader refusals published
//! under a codegen label, and the `noform-` keys where `None` was overloaded.
//!
//! So every intersection here is preceded by a **join report that is loud when
//! it resolves nothing**: [`JoinCheck`] counts what matched, names what did not,
//! and *guesses the normalization* that would have fixed it without applying it
//! — a hint is a diagnosis, and silently canonicalizing paths would be the same
//! defect one layer down.
//!
//! # One definition, two producers
//!
//! Every set below is defined **once**, over [`FactorRow`], and reached two ways:
//!
//! * live, from a scan — [`super::GapReport::factor_rows`];
//! * offline, from a `--factors-tsv` file — [`parse_factors_tsv`].
//!
//! `the_tsv_view_and_the_live_report_are_the_same_rows` grades that the two
//! agree row-for-row, so the offline tool is a **view** of the scan's
//! measurement rather than a second implementation of it that can drift. And
//! the set names below are deliberately the `gap-metric` **keys**, so the
//! known-answer control is mechanical: [`check_metrics`] reads a scan's own
//! `gap-metric` lines and compares them against the counts re-derived from the
//! rows. A published figure and its listing cannot disagree without that check
//! going red.

use std::collections::{BTreeMap, BTreeSet};

/// **One graded TU, reduced to exactly what the published sets are defined
/// over**: its source path, its class label, and the five factor bits.
///
/// This is deliberately *not* [`super::TuResult`]. A `TuResult` carries the
/// census, the emit map, the byte counts — none of which any set below reads —
/// and it cannot be reconstructed from a TSV row. Narrowing to the three fields
/// the algebra actually uses is what lets the live scan and the offline file
/// produce the *same* type and therefore the same sets.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FactorRow {
    /// Source argument as scanned — the join key, used verbatim and never
    /// canonicalized. See [`JoinCheck`] for why normalizing here would be a bug.
    pub src: String,
    /// `TuClass::label()` — `match`, `vocab-gap`, … A `capture-fail` TU is never
    /// a row (it was not measured; `docs/STATUS.md` trap 5).
    pub class: String,
    /// `[A, B, C, D, E]`.
    pub f: [bool; 5],
}

impl FactorRow {
    /// Is this TU already byte-exact?
    pub fn is_match(&self) -> bool {
        self.class == "match"
    }
    /// `D ∨ E` — the port has *some* accepted route to this TU's contents.
    /// Named rather than spelled inline because board #179's whole content is
    /// that neither disjunct is necessary alone.
    pub fn accepted(&self) -> bool {
        self.f[3] || self.f[4]
    }
    /// The fixed-width letter string, `"AB-D-"`. Identical to
    /// [`super::GapReport::factor_letters`] by construction — the TSV writer
    /// uses that one and the parser checks its output against these bits.
    pub fn letters(&self) -> String {
        ['A', 'B', 'C', 'D', 'E']
            .iter()
            .zip(self.f.iter())
            .map(|(ch, on)| if *on { *ch } else { '-' })
            .collect()
    }
}

/// A named, published set of graded TUs.
pub struct NamedSet {
    /// **The `gap-metric` key**, where one exists — that is what makes
    /// [`check_metrics`] a control rather than a restatement. The four sets with
    /// no metric key (`reach-pool`, `frontier-pool`, `d-or-e`, and the two
    /// pools' complements) carry a `metric` of `None` and are excluded from the
    /// control instead of being quietly given a key that does not exist.
    pub name: &'static str,
    /// The corresponding `gap-metric` key, or `None` when the scan publishes no
    /// count for this set.
    pub metric: Option<&'static str>,
    pub pred: fn(&FactorRow) -> bool,
    /// One line, printed beside the count. A set whose definition lives only in
    /// a reader's head is how `B∧C` came to be quoted at a value taken under a
    /// different `C`.
    pub blurb: &'static str,
}

/// **Every set this module can intersect against**, in print order.
///
/// The definitions are transcriptions of the ones in [`super::factors`] — see
/// that module for *why* each is what it is. What is added here is that they are
/// sets rather than counts, and that the `metric` column ties each to the figure
/// the scan already publishes.
///
/// **`frontier` and `frontier-if-a` both exclude `match` TUs**, exactly as
/// `factor_frontier` and `factor_frontier_if_a` do. Dropping that clause is the
/// easiest way to make this file disagree with the scan by 11, which is why the
/// control is mechanical.
/// PROV[N] not load-bearing — this instrument's own named TU sets, whose agreement with the scan is asserted mechanically rather than by hand.
pub const NAMED_SETS: &[NamedSet] = &[
    NamedSet {
        name: "factor-a",
        metric: Some("factor-a"),
        pred: |r| r.f[0],
        blurb: "`.ex` segments == obj `.text` COMDATs (gate-anchored `4F 1F`)",
    },
    NamedSet {
        name: "factor-b",
        metric: Some("factor-b"),
        pred: |r| r.f[1],
        blurb: "every emitted symbol binds",
    },
    NamedSet {
        name: "factor-c",
        metric: Some("factor-c"),
        pred: |r| r.f[2],
        blurb: "obj section set ⊆ the port writer's section names",
    },
    NamedSet {
        name: "factor-d",
        metric: Some("factor-d"),
        pred: |r| r.f[3],
        blurb: "every emitted COMDAT in the per-function codegen class",
    },
    NamedSet {
        name: "factor-e",
        metric: Some("factor-e"),
        pred: |r| r.f[4],
        blurb: "a REGISTERED whole-TU recognizer accepts this bundle",
    },
    NamedSet {
        name: "d-or-e",
        metric: None,
        pred: |r| r.accepted(),
        blurb: "the port has some accepted route to this TU's contents",
    },
    NamedSet {
        name: "b-and-c",
        metric: Some("b-and-c"),
        pred: |r| r.f[1] && r.f[2],
        blurb: "the near-term joint ceiling: perfect emit model + perfect binding, \
                at today's writer vocabulary",
    },
    NamedSet {
        name: "a-and-b-and-c",
        metric: Some("a-and-b-and-c"),
        pred: |r| r.f[0] && r.f[1] && r.f[2],
        blurb: "the same ceiling reachable WITHOUT an emit-set model",
    },
    NamedSet {
        name: "a-and-b-and-c-and-d",
        metric: Some("a-and-b-and-c-and-d"),
        pred: |r| r.f[0] && r.f[1] && r.f[2] && r.f[3],
        blurb: "§10.19's original conjunction — the one board #179 refutes, kept measurable",
    },
    NamedSet {
        name: "a-and-b-and-c-and-d-or-e",
        metric: Some("a-and-b-and-c-and-d-or-e"),
        pred: |r| r.f[0] && r.f[1] && r.f[2] && r.accepted(),
        blurb: "the model's joint — the claim is that this set IS the match set",
    },
    NamedSet {
        name: "match",
        metric: Some("match"),
        pred: FactorRow::is_match,
        blurb: "byte-exact against real c2 today — the only set the sole judge produced",
    },
    NamedSet {
        name: "frontier",
        metric: Some("frontier"),
        pred: |r| !r.is_match() && r.f[0] && r.f[1] && r.f[2] && !r.accepted(),
        blurb: "A∧B∧C, not a match, no acceptance path — codegen breadth is the whole distance",
    },
    NamedSet {
        name: "frontier-if-a",
        metric: Some("frontier-if-a"),
        pred: |r| !r.is_match() && r.f[1] && r.f[2] && !r.accepted(),
        blurb: "the counterfactual frontier if a PERFECT emit-set model existed",
    },
    NamedSet {
        name: "projection-divergence",
        metric: None,
        pred: |r| !r.is_match() && !r.f[0] && r.f[1] && r.f[2] && r.accepted(),
        blurb: "the TUs on which board #213's two arithmetics disagree",
    },
    NamedSet {
        name: "reach-pool",
        metric: None,
        pred: |r| r.f[1] && r.f[2] && !(r.f[0] && r.f[1] && r.f[2]),
        blurb: "B∧C ∖ A∧B∧C — THE POOL BOARD #213 PRICES AT +124. A partial emit \
                model buys |model ∩ this|",
    },
    NamedSet {
        name: "frontier-pool",
        metric: None,
        pred: |r| !r.is_match() && !r.f[0] && r.f[1] && r.f[2] && !r.accepted(),
        blurb: "frontier-if-A ∖ FRONTIER — board #213's OTHER arithmetic, +122",
    },
];

/// Look a set up by name.
pub fn named(name: &str) -> Option<&'static NamedSet> {
    NAMED_SETS.iter().find(|s| s.name == name)
}

/// The members of a named set, by source path, in row order.
pub fn members<'a>(rows: &'a [FactorRow], name: &str) -> Option<Vec<&'a str>> {
    let s = named(name)?;
    Some(rows.iter().filter(|r| (s.pred)(r)).map(|r| r.src.as_str()).collect())
}

/// `|S|` for a named set.
pub fn count(rows: &[FactorRow], name: &str) -> Option<usize> {
    named(name).map(|s| rows.iter().filter(|r| (s.pred)(r)).count())
}

// ---------------------------------------------------------------------------
// The TSV view
// ---------------------------------------------------------------------------

/// The exact `# columns:` line [`super::GapReport::factor_tsv`] writes.
///
/// Compared for **equality**, not parsed. A file whose columns moved is a file
/// whose bits mean something else, and reading it positionally anyway is how a
/// join produces a confident wrong answer. This constant going stale breaks a
/// unit test in the same crate, not a lane's arithmetic three days later.
/// PROV[N] not load-bearing — the instrument's own TSV header, pinned so going stale breaks a unit test in the same crate rather than a lane's arithmetic three days later.
pub const TSV_COLUMNS: &str =
    "# columns: src<TAB>class<TAB>A<TAB>B<TAB>C<TAB>D<TAB>E<TAB>letters";

/// Parse a `c2rs gap --factors-tsv` file into rows.
///
/// # What is refused, and why each refusal is not pedantry
///
/// * **A missing or changed `# columns:` header** — see [`TSV_COLUMNS`].
/// * **A `# graded-rows N` that disagrees with the row count** — the writer
///   emits both; if they diverge the file was truncated or concatenated, and a
///   truncated file yields a *smaller* intersection, which looks like a weaker
///   model rather than a broken input.
/// * **A `0`/`1` column that disagrees with the `letters` column** — the two are
///   redundant by construction, so disagreement means the file was edited or
///   assembled by something that did not understand it.
/// * **Zero rows.** An empty population makes every intersection 0 and every
///   ratio undefined, and 0 is a number a reader will happily believe.
pub fn parse_factors_tsv(text: &str) -> Result<Vec<FactorRow>, String> {
    let mut rows: Vec<FactorRow> = Vec::new();
    let mut declared: Option<usize> = None;
    let mut saw_columns = false;
    for (ln, line) in text.lines().enumerate() {
        let no = ln + 1;
        if line.trim().is_empty() {
            continue;
        }
        if let Some(rest) = line.strip_prefix('#') {
            if line == TSV_COLUMNS {
                saw_columns = true;
            }
            if let Some(n) = rest.trim().strip_prefix("graded-rows ") {
                declared = n.trim().parse::<usize>().ok();
            }
            continue;
        }
        let c: Vec<&str> = line.split('\t').collect();
        if c.len() != 8 {
            return Err(format!("line {no}: expected 8 tab-separated columns, got {}", c.len()));
        }
        let mut f = [false; 5];
        for (i, col) in c[2..7].iter().enumerate() {
            f[i] = match *col {
                "0" => false,
                "1" => true,
                other => {
                    return Err(format!("line {no}: factor column {} is {other:?}, not 0 or 1", i + 3))
                }
            };
        }
        let row = FactorRow { src: c[0].to_string(), class: c[1].to_string(), f };
        if row.letters() != c[7] {
            return Err(format!(
                "line {no}: the letters column is {:?} but the 0/1 columns say {:?} — the two are \
                 redundant by construction, so this file was not written by `gap --factors-tsv`",
                c[7],
                row.letters()
            ));
        }
        rows.push(row);
    }
    if !saw_columns {
        return Err(format!(
            "no `{TSV_COLUMNS}` header — refusing to read columns positionally out of a file \
             that does not declare them"
        ));
    }
    if rows.is_empty() {
        return Err("0 rows — an empty population makes every intersection 0, which is a \
                    number a reader will believe"
            .to_string());
    }
    match declared {
        None => Err("no `# graded-rows N` line — the writer emits one and it is the only \
                     check that the file is whole"
            .to_string()),
        Some(n) if n != rows.len() => Err(format!(
            "`# graded-rows {n}` but {} rows parsed — the file is truncated or concatenated, \
             and a truncated file yields a SMALLER intersection, which reads as a weaker model",
            rows.len()
        )),
        Some(_) => Ok(rows),
    }
}

// ---------------------------------------------------------------------------
// The candidate set, and the join
// ---------------------------------------------------------------------------

/// A candidate per-TU set carried in from somewhere else — a model's exact set,
/// a hand list, another lane's scan.
#[derive(Clone, Debug)]
pub struct CandidateSet {
    pub name: String,
    /// Distinct names, in file order of first appearance.
    pub names: Vec<String>,
    /// Non-comment, non-blank lines read.
    pub lines: usize,
    /// Lines that repeated a name already seen. Reported rather than ignored: a
    /// duplicate means the producer emitted per-*something-else* and the set is
    /// not the set its author thinks it is.
    pub duplicates: usize,
}

/// Read a candidate set: one TU name per line, `#` comments and blanks skipped.
///
/// **Whitespace is trimmed and nothing else is touched.** No path
/// canonicalization, no separator rewriting — see [`JoinCheck`].
pub fn parse_candidate(name: &str, text: &str) -> Result<CandidateSet, String> {
    let mut names: Vec<String> = Vec::new();
    let mut seen: BTreeSet<String> = BTreeSet::new();
    let (mut lines, mut duplicates) = (0usize, 0usize);
    for line in text.lines() {
        let t = line.trim();
        if t.is_empty() || t.starts_with('#') {
            continue;
        }
        lines += 1;
        if seen.insert(t.to_string()) {
            names.push(t.to_string());
        } else {
            duplicates += 1;
        }
    }
    if names.is_empty() {
        return Err(format!(
            "candidate set {name:?} has 0 names — an empty set intersects everything at 0, \
             and 0 would be published as a measurement"
        ));
    }
    Ok(CandidateSet { name: name.to_string(), names, lines, duplicates })
}

/// **The join report — printed before any intersection, always.**
///
/// # Why this is a type and not a comment
///
/// The join key is a source path *string*. `w-emitp`'s corpus spells
/// `src/App.cpp` as `src__App.cpp` because its truth files are named after it;
/// a tree walk spells it `<workload-root>/src/App.cpp`; the scan spells it
/// the way `cl` was invoked. Any of those against any other yields **0
/// matches**, and every intersection downstream is then 0 — a perfectly
/// well-formed table of zeros that says *the model buys no reach*.
///
/// This project has shipped that shape at least four times this week under
/// other names. So: the join is reported, `resolved == 0` is an **error exit**
/// and never a table, and a plausible normalization is *named* in
/// [`Self::hint`] and **not applied**. Applying it would move the same defect
/// one layer down — a canonicalizer that silently maps two real, different TUs
/// onto one key produces an intersection that is too *large*, and nothing
/// downstream could see it.
#[derive(Clone, Debug)]
pub struct JoinCheck {
    /// Candidate names that matched a graded row, in candidate order.
    pub resolved: Vec<String>,
    /// Candidate names that matched nothing.
    pub unresolved: Vec<String>,
    /// Graded rows no candidate name mentions. Not an error — a candidate set
    /// legitimately covers a sub-corpus (w-emitp graded 850 of 871) — but a
    /// number that must be *looked at*, because "the model is exact on 472" and
    /// "the model was never run on 21" are different facts.
    pub absent: usize,
    /// A normalization that would resolve more names, if one is apparent.
    pub hint: Option<String>,
}

impl JoinCheck {
    /// Did the join resolve nothing at all? The caller must treat this as an
    /// error, not as a result.
    pub fn is_empty(&self) -> bool {
        self.resolved.is_empty()
    }
}

/// The normalizations [`join`] *tries*, in order, purely to explain a poor join.
/// None is ever applied to the data.
///
/// Each is a real spelling seen in this repo's lane scratch:
/// `src__App.cpp` (truth-file naming), a leading `./`, a Windows separator, and
/// an absolute path with the workload root prefixed.
fn hint_for(unresolved: &[String], keys: &BTreeSet<&str>) -> Option<String> {
    let probes: [(&str, fn(&str) -> String); 4] = [
        ("`__` → `/` (truth-file naming)", |s| s.replace("__", "/")),
        ("strip a leading `./`", |s| s.trim_start_matches("./").to_string()),
        ("`\\` → `/` (Windows separator)", |s| s.replace('\\', "/")),
        ("keep only the tail after the last `/` matched as a suffix", |s| s.to_string()),
    ];
    let mut best: Option<(usize, &str)> = None;
    for (label, f) in probes.iter().take(3) {
        let n = unresolved.iter().filter(|u| keys.contains(f(u).as_str())).count();
        if n > 0 && best.map(|(b, _)| n > b).unwrap_or(true) {
            best = Some((n, label));
        }
    }
    // The suffix probe is separate: it is a scan over the keys, not a rewrite.
    let n_suffix = unresolved
        .iter()
        .filter(|u| keys.iter().any(|k| k.ends_with(u.as_str()) || u.ends_with(k)))
        .count();
    if n_suffix > 0 && best.map(|(b, _)| n_suffix > b).unwrap_or(true) {
        best = Some((n_suffix, probes[3].0));
    }
    best.map(|(n, label)| {
        format!(
            "{n} of the {} unresolved names would match under {label} — NOT APPLIED. Fix the \
             producer's spelling; a canonicalizer here could silently fold two real TUs onto one \
             key and make the intersection too LARGE, which nothing downstream can see.",
            unresolved.len()
        )
    })
}

/// Join a candidate set against the graded rows.
pub fn join(rows: &[FactorRow], cand: &CandidateSet) -> JoinCheck {
    let keys: BTreeSet<&str> = rows.iter().map(|r| r.src.as_str()).collect();
    let (mut resolved, mut unresolved) = (Vec::new(), Vec::new());
    for n in &cand.names {
        if keys.contains(n.as_str()) {
            resolved.push(n.clone());
        } else {
            unresolved.push(n.clone());
        }
    }
    let cset: BTreeSet<&str> = cand.names.iter().map(String::as_str).collect();
    let absent = rows.iter().filter(|r| !cset.contains(r.src.as_str())).count();
    let hint = if unresolved.is_empty() { None } else { hint_for(&unresolved, &keys) };
    JoinCheck { resolved, unresolved, absent, hint }
}

/// `|cand ∩ S|` for every named set, keyed by set name.
///
/// Takes the **resolved** names, so an intersection can never exceed the join
/// the report above printed.
pub fn intersections(rows: &[FactorRow], resolved: &[String]) -> BTreeMap<&'static str, usize> {
    let r: BTreeSet<&str> = resolved.iter().map(String::as_str).collect();
    NAMED_SETS
        .iter()
        .map(|s| {
            let n = rows
                .iter()
                .filter(|row| r.contains(row.src.as_str()) && (s.pred)(row))
                .count();
            (s.name, n)
        })
        .collect()
}

// ---------------------------------------------------------------------------
// The known-answer control
// ---------------------------------------------------------------------------

/// One line of the metrics control.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MetricCheck {
    pub key: &'static str,
    /// What the scan published, or `None` if the key was not in the file.
    pub published: Option<usize>,
    /// What the rows re-derive.
    pub derived: usize,
}

impl MetricCheck {
    pub fn verdict(&self) -> &'static str {
        match self.published {
            None => "ABSENT",
            Some(p) if p == self.derived => "OK",
            Some(_) => "DISAGREE",
        }
    }
}

/// Scrape `gap-metric <key> <value>` lines out of a scan log.
pub fn scrape_metrics(text: &str) -> BTreeMap<String, usize> {
    let mut m = BTreeMap::new();
    for line in text.lines() {
        let t = line.trim();
        let Some(rest) = t.strip_prefix("gap-metric ") else { continue };
        let mut it = rest.split_whitespace();
        if let (Some(k), Some(v)) = (it.next(), it.next()) {
            if let Ok(n) = v.parse::<usize>() {
                m.insert(k.to_string(), n);
            }
        }
    }
    m
}

/// **The control**: for every set that has a `gap-metric` key, does the count
/// re-derived from the rows equal the count the scan published?
///
/// Plus `graded`, which is not a set but is the population every one of them is
/// taken over — a row count that silently differs makes every other line agree
/// about the wrong corpus.
///
/// A `DISAGREE` is the finding; an `ABSENT` is a key the log did not carry and
/// is reported as its own verdict rather than skipped, because a control that
/// checks nothing and a control that passes look identical in a summary line.
pub fn check_metrics(rows: &[FactorRow], published: &BTreeMap<String, usize>) -> Vec<MetricCheck> {
    let mut out = vec![MetricCheck {
        key: "graded",
        published: published.get("graded").copied(),
        derived: rows.len(),
    }];
    for s in NAMED_SETS {
        let Some(key) = s.metric else { continue };
        out.push(MetricCheck {
            key,
            published: published.get(key).copied(),
            derived: rows.iter().filter(|r| (s.pred)(r)).count(),
        });
    }
    out
}
