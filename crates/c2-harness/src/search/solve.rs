use std::path::{Path, PathBuf};
use std::time::Duration;

use c2_il::IlModel;
use c2_reference::Toolchain;

use super::engine::{beam_search, Budget, SearchOutcome};
use super::moves::MoveSet;
use super::perturb::{perturb, Perturb};
use super::scorer::ReplayScorer;

/// One instance's result within a [`SolveReport`].
#[derive(Clone, Debug)]
pub struct InstanceResult {
    pub fixture: String,
    pub perturb: Perturb,
    pub d: usize,
    /// `None` = no site for this perturbation on this fixture (skipped).
    pub outcome: Option<SearchOutcome>,
    /// A toolchain/capture error (also skipped, reported honestly).
    pub error: Option<String>,
}

/// Aggregate solve-rate over a roster of solvable instances.
#[derive(Clone, Debug, Default)]
pub struct SolveReport {
    pub instances: Vec<InstanceResult>,
}

impl SolveReport {
    /// (attempted, solved, mean-compiles-to-solve) — attempted excludes skipped
    /// (no-site) and errored instances so the rate is over real search attempts.
    pub fn tally(&self) -> (usize, usize, f64) {
        Self::tally_of(self.instances.iter())
    }

    /// Per-`(family, d)` breakdown, in first-seen order — so a lumped headline
    /// never hides that different families have different reachability.
    pub fn by_family(&self) -> Vec<((Perturb, usize), (usize, usize, f64))> {
        let mut keys: Vec<(Perturb, usize)> = Vec::new();
        for r in &self.instances {
            let k = (r.perturb, r.d);
            if !keys.contains(&k) {
                keys.push(k);
            }
        }
        keys.into_iter()
            .map(|k| {
                let t = Self::tally_of(
                    self.instances.iter().filter(|r| (r.perturb, r.d) == k),
                );
                (k, t)
            })
            .collect()
    }

    fn tally_of<'a, I: Iterator<Item = &'a InstanceResult>>(it: I) -> (usize, usize, f64) {
        let mut attempted = 0usize;
        let mut solved = 0usize;
        let mut compiles_sum = 0usize;
        for r in it {
            if let Some(o) = &r.outcome {
                attempted += 1;
                if o.solved {
                    solved += 1;
                    compiles_sum += o.compiles;
                }
            }
        }
        let mean = if solved > 0 {
            compiles_sum as f64 / solved as f64
        } else {
            0.0
        };
        (attempted, solved, mean)
    }
}

/// Build a solvable instance from a fixture `.cpp` and one perturbation, then
/// climb it, judged by real c2. Captures the fixture, takes the parsed model as
/// the solution and its replay as the target, perturbs to a seed, and hill-climbs
/// back. Requires a ready toolchain (see [`Toolchain::has_strace`]/`has_mingw`).
#[allow(clippy::too_many_arguments)]
pub fn solve_instance(
    tc: &Toolchain,
    cpp: &Path,
    kind: Perturb,
    d: usize,
    moves: &MoveSet,
    budget: &Budget,
    scratch: &Path,
    timeout: Duration,
) -> InstanceResult {
    let fixture = cpp
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    let mk = |outcome, error| InstanceResult {
        fixture: fixture.clone(),
        perturb: kind,
        d,
        outcome,
        error,
    };

    let base = match tc.capture_reference(cpp, &scratch.join("cap")) {
        Ok(c) => c,
        Err(e) => return mk(None, Some(format!("capture: {e}"))),
    };
    let solution = match IlModel::parse(&base.bundle) {
        Ok(m) => m,
        Err(e) => return mk(None, Some(format!("codec: {e}"))),
    };
    // The scorer's fixed `-Fo` is a pure function of its scratch dir, so compute
    // it up front, render the TARGET (the solution IL replayed) to it, then hand
    // the target's bytes to the scorer. Target and every candidate thus share the
    // embedded `S_OBJNAME`, making a byte-exact terminal reachable.
    let search_dir = scratch.join("search");
    let fo = search_dir.join("cand.obj");
    let target = match tc.replay_within(&base, &scratch.join("tgt_il"), &fo, timeout) {
        Ok(o) => o,
        Err(e) => return mk(None, Some(format!("target replay: {e}"))),
    };
    let mut scorer = ReplayScorer::new(tc, &base, target, search_dir, timeout);
    debug_assert_eq!(scorer.fo_path(), fo.as_path());

    let Some(seed) = perturb(&solution, kind, d) else {
        return mk(None, None); // no site — skipped, not a failure
    };

    // The real solve-rate path runs the beam (width from `budget.beam_width`;
    // width 1 degrades to greedy) so multi-move descents can cross the plateaus
    // greedy stalls on. TERMINAL is unchanged — byte-exact obj only.
    let outcome = beam_search(&seed, moves, &mut scorer, budget);
    mk(Some(outcome), None)
}

/// Run the solvable-instance protocol over a roster of fixtures × perturbations,
/// returning the aggregate [`SolveReport`]. Deterministic given the roster.
#[allow(clippy::too_many_arguments)]
pub fn solve_rate(
    tc: &Toolchain,
    fixtures: &[PathBuf],
    perturbs: &[(Perturb, usize)],
    moves: &MoveSet,
    budget: &Budget,
    scratch: &Path,
    timeout: Duration,
) -> SolveReport {
    let mut report = SolveReport::default();
    let mut n = 0usize;
    for cpp in fixtures {
        for &(kind, d) in perturbs {
            let dir = scratch.join(format!("inst{n}"));
            n += 1;
            let r = solve_instance(tc, cpp, kind, d, moves, budget, &dir, timeout);
            let _ = std::fs::remove_dir_all(&dir);
            report.instances.push(r);
        }
    }
    report
}
