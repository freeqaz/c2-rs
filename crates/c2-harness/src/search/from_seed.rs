use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use std::time::Duration;

use c2_il::{IlBundle, IlModel};
use c2_reference::Toolchain;

use crate::corpus;
use crate::retrieval::{self, Item};

use super::engine::{beam_search, Budget, SearchOutcome, StopReason};
use super::moves::MoveSet;
use super::scorer::ReplayScorer;

// ===========================================================================
// From-unrelated-seed — the P1.3-retrieval-seeded search (the REAL pipeline)
// ===========================================================================
//
// The solvable-instance protocol above seeds from a SMALL perturbation of the
// known solution, so a byte-exact IL is one move away by construction — it prices
// the search but is not the real task. This rung attempts the real pipeline:
// given a TARGET obj whose IL is unknown, use **P1.3 retrieval** to pick the
// nearest corpus IL as the seed, then beam-search from that unrelated seed toward
// the target — terminal byte-exact. Most targets have no corpus twin (retrieval
// recall@1 is low), so the seed is only APPROXIMATE: the search must bridge the
// gap through K3a edits, and the honest solve-rate + failure taxonomy is the
// finding.

/// The seed the retrieval step picks for a target, or why there is none.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SeedChoice {
    /// The nearest non-self corpus neighbor is an exact-`.text` **twin** of the
    /// target — it compiles byte-exact with no search (a trivial / degenerate
    /// "retrieval-solved" case). Reported separately, never fed to the search.
    RetrievalTrivial { twin_id: String },
    /// The nearest non-self, non-twin neighbor's index in the item slice — the
    /// (approximate) seed the search starts from.
    Seed { index: usize },
    /// No candidate at all (the item slice held only the target).
    NoCandidate,
}

/// Pick the retrieval seed for `target` from `items` (the corpus): rank by the
/// P1.3 obj-`.text` cosine feature, take the nearest neighbor that is **not the
/// target's own row** and **not an exact-`.text` twin** (a twin is a trivial
/// solve, reported separately). Pure over the item features — deterministic, no
/// toolchain.
pub fn select_seed(target: &Item, items: &[Item]) -> SeedChoice {
    for i in retrieval::rank(target, items) {
        let cand = &items[i];
        if cand.id == target.id {
            continue; // never seed from self
        }
        if cand.text_key == target.text_key {
            // The nearest non-self neighbor is a behavioral twin → trivial solve.
            return SeedChoice::RetrievalTrivial {
                twin_id: cand.id.clone(),
            };
        }
        return SeedChoice::Seed { index: i };
    }
    SeedChoice::NoCandidate
}

/// The taxonomy bucket for one from-seed target outcome.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FromSeedClass {
    /// A `.text` twin was retrieved — solved without any search (trivial).
    RetrievalTrivial,
    /// Seed and target have different function counts — out of K3a scope
    /// (whole-function add/remove is K3b, unimplemented), so the search is not
    /// attempted.
    K3bBlocked,
    /// The search reached a byte-exact obj from the unrelated seed.
    Solved,
    /// The search stalled at a local optimum below byte-exact — the move set
    /// (K3a edits + generative vocab) could not bridge the seed→target gap
    /// (a plateau / vocabulary limit).
    Plateau,
    /// The search hit its step or compile budget still short of byte-exact.
    BudgetExhausted,
    /// A per-target error (seed bundle would not load/parse, a capture/replay
    /// failure) — reported honestly, never faked as a solve.
    Error,
}

impl FromSeedClass {
    pub fn label(&self) -> &'static str {
        match self {
            FromSeedClass::RetrievalTrivial => "retrieval-trivial",
            FromSeedClass::K3bBlocked => "k3b-blocked",
            FromSeedClass::Solved => "SOLVED",
            FromSeedClass::Plateau => "plateau/vocab",
            FromSeedClass::BudgetExhausted => "budget-exhausted",
            FromSeedClass::Error => "error",
        }
    }
}

/// One target's from-seed result.
#[derive(Clone, Debug)]
pub struct FromSeedRecord {
    pub target_id: String,
    pub target_fns: usize,
    pub seed_id: Option<String>,
    pub seed_fns: Option<usize>,
    pub class: FromSeedClass,
    /// A short human note (twin id, fn-count mismatch, error text, …).
    pub detail: String,
    /// The primary search outcome (per-function gradient for a multi-function
    /// target, whole-`.text` for single-function). `None` for a non-searched
    /// class (trivial / K3b / error before search).
    pub outcome: Option<SearchOutcome>,
    /// For a multi-function target only: the SAME search run WITHOUT the
    /// per-function gradient (whole-`.text`), the with/without comparison the
    /// plateau-fix is measured against.
    pub outcome_wholetext: Option<SearchOutcome>,
}

/// Config for a from-retrieval eval run — kept small and bounded (CPU is shared).
#[derive(Clone, Debug)]
pub struct FromSeedConfig {
    /// Total held-out targets to attempt.
    pub sample: usize,
    /// Of `sample`, how many should be multi-function (to exercise the
    /// per-function gradient); the rest are single-function.
    pub multi: usize,
    /// Deterministic sample-selection seed (a stride offset over the sorted ids).
    pub select_seed: u64,
    /// Per-target search budget.
    pub budget: Budget,
    /// Per-replay wall-clock timeout.
    pub timeout: Duration,
}

impl Default for FromSeedConfig {
    fn default() -> Self {
        FromSeedConfig {
            sample: 24,
            multi: 4,
            select_seed: 0,
            budget: Budget {
                max_steps: 10,
                max_compiles: 300,
                restarts: 0,
                beam_width: 5,
            },
            timeout: Duration::from_secs(30),
        }
    }
}

/// The aggregate from-retrieval report.
#[derive(Clone, Debug, Default)]
pub struct FromSeedReport {
    pub records: Vec<FromSeedRecord>,
    /// Corpus size the sample was drawn from.
    pub n_items: usize,
}

impl FromSeedReport {
    /// Count of records in each class, in a fixed order.
    pub fn class_counts(&self) -> Vec<(FromSeedClass, usize)> {
        let order = [
            FromSeedClass::Solved,
            FromSeedClass::Plateau,
            FromSeedClass::BudgetExhausted,
            FromSeedClass::K3bBlocked,
            FromSeedClass::RetrievalTrivial,
            FromSeedClass::Error,
        ];
        order
            .iter()
            .map(|&c| (c, self.records.iter().filter(|r| r.class == c).count()))
            .collect()
    }

    /// (searched, solved): searched excludes trivial / K3b / error (the classes
    /// where no real search ran), so the solve-rate is over genuine attempts.
    pub fn search_tally(&self) -> (usize, usize) {
        let searched = self
            .records
            .iter()
            .filter(|r| r.outcome.is_some())
            .count();
        let solved = self
            .records
            .iter()
            .filter(|r| r.class == FromSeedClass::Solved)
            .count();
        (searched, solved)
    }
}

/// Per-corpus-row metadata the from-seed runner needs (from the manifest).
pub(super) struct RowMeta {
    pub(super) source_rel: String,
    pub(super) il_dir_rel: String,
    pub(super) il_base: String,
    pub(super) fns: usize,
}

/// Deterministically pick `n` ids from the sorted `ids`, strided from a
/// `seed`-derived start so the sample spreads across the corpus and is
/// reproducible. Tops up on collisions so it always returns `min(n, len)` ids.
fn pick_ids(ids: &[String], n: usize, seed: u64) -> Vec<String> {
    let len = ids.len();
    if len == 0 || n == 0 {
        return Vec::new();
    }
    let n = n.min(len);
    let stride = (len / n).max(1);
    let start = (seed as usize) % len;
    let mut used: BTreeSet<usize> = BTreeSet::new();
    let mut out: Vec<String> = Vec::new();
    let mut step = 0usize;
    while out.len() < n && step < len {
        let k = (start + step * stride) % len;
        if used.insert(k) {
            out.push(ids[k].clone());
        }
        step += 1;
    }
    let mut k = 0usize;
    while out.len() < n && k < len {
        if used.insert(k) {
            out.push(ids[k].clone());
        }
        k += 1;
    }
    out
}

/// Run the from-unrelated-seed protocol over a deterministically-selected sample
/// of held-out corpus targets, returning the per-target records + aggregate.
///
/// For each target: pick the retrieval seed ([`select_seed`], excluding self and
/// `.text` twins); scope to a compatible function structure (same function count
/// — a mismatch is [`FromSeedClass::K3bBlocked`], NOT forced); render the target
/// obj by replaying its own IL to a fixed `-Fo` (so a byte-exact terminal is
/// reachable); then beam-search from the seed toward the target obj, judged by a
/// REAL c2 replay, terminal byte-exact. Multi-function targets are searched with
/// the per-function gradient AND, for the with/without comparison, again with the
/// whole-`.text` gradient.
pub fn from_retrieval_eval(
    tc: &Toolchain,
    root: &Path,
    moves: &MoveSet,
    cfg: &FromSeedConfig,
    scratch: &Path,
) -> std::io::Result<FromSeedReport> {
    let items = retrieval::load_items(root)?;
    let manifest = corpus::load_manifest(root)?;

    // id -> metadata (only `ok` rows with a full IL side).
    let mut meta: BTreeMap<String, RowMeta> = BTreeMap::new();
    for r in manifest {
        if r.status != "ok" {
            continue;
        }
        if let (Some(source_rel), Some(il_dir_rel), Some(il_base)) =
            (r.source_rel, r.il_dir_rel, r.il_base)
        {
            meta.insert(
                r.id.clone(),
                RowMeta {
                    source_rel,
                    il_dir_rel,
                    il_base,
                    fns: r.gl_offsets.len(),
                },
            );
        }
    }
    let idx_of: BTreeMap<String, usize> = items
        .iter()
        .enumerate()
        .map(|(i, it)| (it.id.clone(), i))
        .collect();

    // Partition ids present in BOTH the items and the metadata by function count.
    let mut single: Vec<String> = Vec::new();
    let mut multi: Vec<String> = Vec::new();
    for (id, m) in &meta {
        if !idx_of.contains_key(id) {
            continue;
        }
        if m.fns <= 1 {
            single.push(id.clone());
        } else if m.fns >= 2 {
            multi.push(id.clone());
        }
    }
    single.sort();
    multi.sort();

    let n_multi = cfg.multi.min(cfg.sample);
    let n_single = cfg.sample.saturating_sub(n_multi);
    let mut targets = pick_ids(&single, n_single, cfg.select_seed);
    targets.extend(pick_ids(&multi, n_multi, cfg.select_seed));

    let mut report = FromSeedReport {
        records: Vec::new(),
        n_items: items.len(),
    };

    for (n, target_id) in targets.iter().enumerate() {
        let t_idx = idx_of[target_id];
        let target_item = &items[t_idx];
        let t_meta = &meta[target_id];
        let target_fns = t_meta.fns;

        let mut rec = FromSeedRecord {
            target_id: target_id.clone(),
            target_fns,
            seed_id: None,
            seed_fns: None,
            class: FromSeedClass::Error,
            detail: String::new(),
            outcome: None,
            outcome_wholetext: None,
        };

        // --- seed selection (retrieval; excludes self + twins) --------------
        match select_seed(target_item, &items) {
            SeedChoice::NoCandidate => {
                rec.detail = "no retrieval candidate".into();
                report.records.push(rec);
                continue;
            }
            SeedChoice::RetrievalTrivial { twin_id } => {
                rec.seed_id = Some(twin_id.clone());
                rec.class = FromSeedClass::RetrievalTrivial;
                rec.detail = format!("exact-.text twin {twin_id} retrieved (no search)");
                report.records.push(rec);
                continue;
            }
            SeedChoice::Seed { index } => {
                let seed_item = &items[index];
                rec.seed_id = Some(seed_item.id.clone());
                let Some(s_meta) = meta.get(&seed_item.id) else {
                    rec.detail = "seed row missing from manifest".into();
                    report.records.push(rec);
                    continue;
                };

                // --- load + parse the seed IL (no toolchain) ----------------
                let seed_dir = root.join(&s_meta.il_dir_rel);
                let seed_bundle = match IlBundle::load_from_dir(&seed_dir, &s_meta.il_base) {
                    Ok(b) => b,
                    Err(e) => {
                        rec.detail = format!("seed bundle load: {e}");
                        report.records.push(rec);
                        continue;
                    }
                };
                let seed_model = match IlModel::parse(&seed_bundle) {
                    Ok(m) => m,
                    Err(e) => {
                        rec.detail = format!("seed codec: {e}");
                        report.records.push(rec);
                        continue;
                    }
                };
                let seed_fns = seed_model.ex_function_count();
                rec.seed_fns = Some(seed_fns);

                // --- in-scope filter (compatible function structure) --------
                if seed_fns != target_fns {
                    rec.class = FromSeedClass::K3bBlocked;
                    rec.detail =
                        format!("seed {seed_fns} fns vs target {target_fns} — K3b (whole-fn) out of scope");
                    report.records.push(rec);
                    continue;
                }

                // --- render the target obj (its own IL → fixed -Fo) ---------
                let inst_dir = scratch.join(format!("inst{n}"));
                let src = root.join(&t_meta.source_rel);
                let base = match tc.capture_reference(&src, &inst_dir.join("cap")) {
                    Ok(c) => c,
                    Err(e) => {
                        rec.detail = format!("target capture: {e}");
                        report.records.push(rec);
                        let _ = std::fs::remove_dir_all(&inst_dir);
                        continue;
                    }
                };
                let search_dir = inst_dir.join("search");
                let fo = search_dir.join("cand.obj");
                let target_obj =
                    match tc.replay_within(&base, &inst_dir.join("tgt_il"), &fo, cfg.timeout) {
                        Ok(o) => o,
                        Err(e) => {
                            rec.detail = format!("target replay: {e}");
                            report.records.push(rec);
                            let _ = std::fs::remove_dir_all(&inst_dir);
                            continue;
                        }
                    };

                // --- primary search (per-function gradient for multi) -------
                let mut scorer = ReplayScorer::new(
                    tc,
                    &base,
                    target_obj.clone(),
                    search_dir.clone(),
                    cfg.timeout,
                );
                scorer.per_function(target_fns);
                let outcome = beam_search(&seed_model, moves, &mut scorer, &cfg.budget);
                rec.class = classify_outcome(&outcome);
                rec.detail = format!("{:?} best_fuzzy={:.4}", outcome.reason, outcome.best_fuzzy);
                rec.outcome = Some(outcome);

                // --- with/without comparison run (multi-function only) ------
                // Same search_dir → same fixed `-Fo` → the rendered target obj
                // still matches candidates. Whole-`.text` gradient this time.
                if target_fns > 1 {
                    let mut scorer2 = ReplayScorer::new(
                        tc,
                        &base,
                        target_obj.clone(),
                        search_dir.clone(),
                        cfg.timeout,
                    );
                    // per_fn left None → whole-`.text`.
                    let out2 = beam_search(&seed_model, moves, &mut scorer2, &cfg.budget);
                    rec.outcome_wholetext = Some(out2);
                }

                let _ = std::fs::remove_dir_all(&inst_dir);
                report.records.push(rec);
            }
        }
    }

    Ok(report)
}

/// Map a finished search outcome to its taxonomy bucket.
fn classify_outcome(o: &SearchOutcome) -> FromSeedClass {
    if o.solved {
        return FromSeedClass::Solved;
    }
    match o.reason {
        StopReason::Solved => FromSeedClass::Solved,
        StopReason::LocalOptimum => FromSeedClass::Plateau,
        StopReason::StepsExhausted | StopReason::CompilesExhausted => {
            FromSeedClass::BudgetExhausted
        }
    }
}
