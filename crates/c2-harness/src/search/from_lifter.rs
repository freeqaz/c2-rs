use std::collections::BTreeMap;
use std::path::Path;
use std::time::Duration;

use c2_il::IlModel;
use c2_reference::Toolchain;

use crate::corpus;

use super::engine::{beam_search, Budget};
use super::from_seed::RowMeta;
use super::moves::MoveSet;
use super::scorer::ReplayScorer;

// ===========================================================================
// From-lifter — the T-B obj->IL(source) lifter eval (the REAL byte-exact metric)
// ===========================================================================
//
// The lifter (angle B) generates candidate C++ SOURCE from the target obj
// (path-free `.text` + mangled symbols). This rung measures the campaign
// deliverable — byte-exact-through-search pass@k — against the ~9.6% P1.3
// retrieval floor. For each held-out target: render its obj (its own IL replayed
// to a fixed `-Fo`), then for each of the k generated sources: capture source ->
// IL (the corpus pipeline; there is no from-scratch `.ex` emitter), seed
// `beam_search` from that model, terminal judged byte-exact by REAL c2 replay. A
// generation that captures to the byte-identical obj solves at search step 0; a
// near-miss is refined by K3a moves. pass@1 uses generation slot 0 (greedy);
// pass@k uses any of the k slots. The identity barrier is intrinsic: the
// generation must reproduce the target's symbol identity (carried in its
// `.gl`/`.sy`) or the obj can never be byte-exact — a wrong-identity generation
// simply never solves, exactly as from-unrelated-seed measured 0/14.

/// The outcome class for one target under the lifter eval.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LifterClass {
    /// >=1 generation reached a byte-exact obj (possibly after K3a search).
    Solved,
    /// Generations compiled but none reached byte-exact.
    NoSolve,
    /// No generation even compiled (every capture failed — malformed source).
    NoCompile,
    /// Target render error (its own capture/replay failed) — excluded from rates.
    Error,
}

impl LifterClass {
    pub fn label(&self) -> &'static str {
        match self {
            LifterClass::Solved => "SOLVED",
            LifterClass::NoSolve => "no-solve",
            LifterClass::NoCompile => "no-compile",
            LifterClass::Error => "error",
        }
    }
}

/// Per-target lifter-eval record.
#[derive(Clone, Debug)]
pub struct LifterRecord {
    pub target_id: String,
    pub target_fns: usize,
    pub k: usize,
    /// # of the k generations that compiled (captured) cleanly.
    pub captured: usize,
    /// The first (0-based) generation slot that reached byte-exact, if any.
    pub solved_slot: Option<usize>,
    /// Did generation slot 0 (greedy) solve? — the pass@1 numerator.
    pub pass1: bool,
    /// Best `.text` fuzzy over all generations (search-gradient context only).
    pub best_fuzzy: f64,
    pub class: LifterClass,
    pub detail: String,
}

/// Aggregate lifter-eval report.
#[derive(Clone, Debug, Default)]
pub struct LifterReport {
    pub records: Vec<LifterRecord>,
    pub n_items: usize,
}

impl LifterReport {
    /// (attempted, pass1, passk): attempted excludes target-render errors, so the
    /// rate is over targets we genuinely searched.
    pub fn tally(&self) -> (usize, usize, usize) {
        let attempted = self
            .records
            .iter()
            .filter(|r| r.class != LifterClass::Error)
            .count();
        let pass1 = self.records.iter().filter(|r| r.pass1).count();
        let passk = self
            .records
            .iter()
            .filter(|r| r.solved_slot.is_some())
            .count();
        (attempted, pass1, passk)
    }
}

/// Load a lifter generations JSONL: one object per line,
/// `{"id": ..., "generations": [str, ...]}`. Rows missing `id` are skipped;
/// `generations` defaults to empty. Returns `(id, gens)` in file order.
pub fn load_lifter_gens(path: &Path) -> std::io::Result<Vec<(String, Vec<String>)>> {
    let text = std::fs::read_to_string(path)?;
    let mut out = Vec::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let Some(obj) = corpus::json::parse(line).and_then(|j| j.into_object()) else {
            continue;
        };
        let mut id: Option<String> = None;
        let mut gens: Vec<String> = Vec::new();
        for (key, v) in obj {
            match key.as_str() {
                "id" => id = v.as_str().map(|s| s.to_string()),
                "generations" => {
                    if let corpus::json::Json::Arr(items) = v {
                        for it in items {
                            if let corpus::json::Json::Str(s) = it {
                                gens.push(s);
                            }
                        }
                    }
                }
                _ => {}
            }
        }
        if let Some(id) = id {
            out.push((id, gens));
        }
    }
    Ok(out)
}

/// Run the T-B lifter eval over generated sources. `gens` maps each target id to
/// its k generation sources (slot 0 = the pass@1 sample). For each target present
/// in both `gens` and the corpus manifest (`ok` rows): render the target obj,
/// then `beam_search` from each captured generation toward it, terminal
/// byte-exact. `limit` caps the number of processed targets (0 = all).
#[allow(clippy::too_many_arguments)]
pub fn from_lifter_eval(
    tc: &Toolchain,
    root: &Path,
    gens: &[(String, Vec<String>)],
    k: usize,
    limit: usize,
    moves: &MoveSet,
    budget: &Budget,
    timeout: Duration,
    scratch: &Path,
) -> std::io::Result<LifterReport> {
    let manifest = corpus::load_manifest(root)?;
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

    let mut report = LifterReport {
        records: Vec::new(),
        n_items: meta.len(),
    };

    for (n, (target_id, all_gens)) in gens.iter().enumerate() {
        if limit > 0 && report.records.len() >= limit {
            break;
        }
        let Some(t_meta) = meta.get(target_id) else {
            continue; // not a corpus row (or not `ok`) — skip silently
        };
        let target_fns = t_meta.fns;
        let kk = k.min(all_gens.len());

        let mut rec = LifterRecord {
            target_id: target_id.clone(),
            target_fns,
            k: kk,
            captured: 0,
            solved_slot: None,
            pass1: false,
            best_fuzzy: 0.0,
            class: LifterClass::Error,
            detail: String::new(),
        };

        let inst_dir = scratch.join(format!("lift{n}"));

        // --- render the target obj (its own IL -> fixed -Fo) ----------------
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
        let target_obj = match tc.replay_within(&base, &inst_dir.join("tgt_il"), &fo, timeout) {
            Ok(o) => o,
            Err(e) => {
                rec.detail = format!("target replay: {e}");
                report.records.push(rec);
                let _ = std::fs::remove_dir_all(&inst_dir);
                continue;
            }
        };

        // --- search from each generation ------------------------------------
        rec.class = LifterClass::NoCompile;
        for (slot, gen_src) in all_gens.iter().take(kk).enumerate() {
            let gen_dir = inst_dir.join(format!("gen{slot}"));
            let gen_cpp = gen_dir.join("gen.cpp");
            if std::fs::create_dir_all(&gen_dir).is_err() || std::fs::write(&gen_cpp, gen_src).is_err()
            {
                continue;
            }
            // capture the generation source -> its IL bundle (the seed model)
            let gen_cap = match tc.capture_reference(&gen_cpp, &gen_dir.join("cap")) {
                Ok(c) => c,
                Err(_) => {
                    // malformed / non-compiling source — not a solve.
                    let _ = std::fs::remove_dir_all(&gen_dir);
                    continue;
                }
            };
            let seed_model = match IlModel::parse(&gen_cap.bundle) {
                Ok(m) => m,
                Err(_) => {
                    let _ = std::fs::remove_dir_all(&gen_dir);
                    continue;
                }
            };
            rec.captured += 1;

            let mut scorer = ReplayScorer::new(
                tc,
                &base,
                target_obj.clone(),
                search_dir.clone(),
                timeout,
            );
            scorer.per_function(target_fns);
            let outcome = beam_search(&seed_model, moves, &mut scorer, budget);
            if outcome.best_fuzzy > rec.best_fuzzy {
                rec.best_fuzzy = outcome.best_fuzzy;
            }
            let _ = std::fs::remove_dir_all(&gen_dir);
            if outcome.solved {
                rec.solved_slot = Some(slot);
                rec.pass1 = slot == 0;
                rec.class = LifterClass::Solved;
                rec.detail = format!("solved at slot {slot} (compiles={})", outcome.compiles);
                break;
            } else if rec.class == LifterClass::NoCompile {
                rec.class = LifterClass::NoSolve;
            }
        }
        if rec.solved_slot.is_none() && rec.detail.is_empty() {
            rec.detail = if rec.captured == 0 {
                "no generation compiled".into()
            } else {
                format!(
                    "no byte-exact over {} gens (best_fuzzy={:.4})",
                    rec.captured, rec.best_fuzzy
                )
            };
        }

        let _ = std::fs::remove_dir_all(&inst_dir);
        report.records.push(rec);
    }

    Ok(report)
}
