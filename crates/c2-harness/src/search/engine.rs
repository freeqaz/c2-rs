use std::collections::BTreeSet;

use c2_il::IlModel;

use super::moves::MoveSet;

// ===========================================================================
// Scorer — the judge abstraction (real replay vs. mock, same climber)
// ===========================================================================

/// The verdict on one candidate model.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Judged {
    /// The candidate's obj is byte-exact (timestamp-normalized, full obj) to the
    /// target — the ONLY success. Terminal.
    ByteExact,
    /// The candidate compiled; `.text` fuzzy gradient in `0.0..=1.0`. A guide,
    /// never a success — a `1.0` here that is not also `ByteExact` means the code
    /// matched but the obj did not (relocs/headers differ), and the search
    /// continues.
    Fuzzy(f64),
    /// The candidate did not compile — a replay crash / timeout, or (upstream) a
    /// refused edit. A clean per-candidate reject; the search skips it.
    Reject,
}

/// Judges a candidate [`IlModel`] against an (implicit) target, counting every
/// real compile. The climber ([`hill_climb`]) is written entirely against this
/// trait, so the same accept/terminal/budget logic runs under a toolchain-free
/// mock ([`MockScorer`]) and the real c2 replay ([`ReplayScorer`]).
pub trait Scorer {
    /// Judge `model`. Implementations MUST count a real compile here (see
    /// [`Scorer::compiles`]); the mock counts a comparison.
    fn judge(&mut self, model: &IlModel) -> Judged;
    /// Total judgements performed — the compiles-to-solve metric.
    fn compiles(&self) -> usize;
}

// ===========================================================================
// The climber
// ===========================================================================

/// Search budget. A hill-climb stops at the first of: byte-exact (success),
/// `max_steps` accepted moves, `max_compiles` judgements, or no improving
/// neighbor (a local optimum, unless restarts remain).
#[derive(Clone, Copy, Debug)]
pub struct Budget {
    pub max_steps: usize,
    pub max_compiles: usize,
    /// Deterministic restarts from the seed on hitting a local optimum. `0` = a
    /// single greedy descent. Restarts re-run the same deterministic
    /// neighborhood, so they only help a scorer with nondeterminism or a future
    /// randomized tie-break; kept for the interface, defaulted off.
    pub restarts: usize,
    /// Beam width — how many best candidates the search keeps at each step. `1`
    /// is pure greedy hill-climb (accept only a strictly-improving move, stop at
    /// a local optimum). `≥ 2` is a beam that keeps the top-`k` candidates by
    /// fuzzy gradient **even when none improves on the parent**, so the search can
    /// take a non-improving (lateral/downhill) step to cross a plateau and reach
    /// the byte-exact basin the greedy climb stalls before (the d≥2 add-term
    /// stall). The terminal is unchanged — only a byte-exact obj wins; the beam
    /// only widens which candidates are compiled.
    pub beam_width: usize,
}

impl Default for Budget {
    fn default() -> Self {
        Budget {
            max_steps: 8,
            max_compiles: 400,
            restarts: 0,
            beam_width: 4,
        }
    }
}

/// Why the climb stopped.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StopReason {
    /// A byte-exact candidate was found (success).
    Solved,
    /// A local optimum: no neighbor strictly improved the fuzzy score.
    LocalOptimum,
    /// `max_steps` reached without a byte-exact candidate.
    StepsExhausted,
    /// `max_compiles` reached without a byte-exact candidate.
    CompilesExhausted,
}

/// Outcome of one hill-climb.
#[derive(Clone, Debug)]
pub struct SearchOutcome {
    pub solved: bool,
    pub steps: usize,
    pub compiles: usize,
    pub best_fuzzy: f64,
    pub reason: StopReason,
    /// The move labels accepted along the winning/best path (for the log).
    pub path: Vec<String>,
}

/// A live beam node: the model, its fuzzy gradient, and the move path that
/// reached it.
struct BeamNode {
    fuzzy: f64,
    model: IlModel,
    path: Vec<String>,
}

/// The `.ex` bytes of a model (its dedup / identity key). Empty if it has no
/// `.ex` file (never, for a captured/hand-built function model).
pub(super) fn ex_bytes(model: &IlModel) -> Vec<u8> {
    model
        .encode()
        .get("ex")
        .map(|b| b.to_vec())
        .unwrap_or_default()
}

/// Greedy IL-space hill-climb from `seed` — the width-1 special case of
/// [`beam_search`] (accept only a strictly-improving move; stop at a local
/// optimum). Kept as the name the portable greedy tests and the terminal-pin
/// drive; forces `beam_width = 1` regardless of `budget`.
///
/// Deterministic: the neighborhood order is fixed and ties are broken by
/// first-seen (the enumeration order), so with a deterministic scorer the whole
/// climb is reproducible — no wall-clock, no RNG.
///
/// TERMINAL is byte-exact ([`Judged::ByteExact`]) and nothing else — a fuzzy
/// `1.0` that is not byte-exact keeps the search going. On a compile/replay
/// reject the candidate is skipped, never fatal.
pub fn hill_climb(
    seed: &IlModel,
    moves: &MoveSet,
    scorer: &mut dyn Scorer,
    budget: &Budget,
) -> SearchOutcome {
    let mut b = *budget;
    b.beam_width = 1;
    beam_search(seed, moves, scorer, &b)
}

/// IL-space **beam search** from `seed`, judged by `scorer`, bounded by `budget`,
/// exploring the [`MoveSet`] neighborhood. Keeps the top-`budget.beam_width`
/// candidates by fuzzy gradient at each step.
///
/// - **width 1** is pure greedy: accept the single strictly-improving best move,
///   stop [`StopReason::LocalOptimum`] when none improves (identical to the
///   original hill-climb; that is what [`hill_climb`] calls).
/// - **width ≥ 2** keeps the top-`k` candidates **even when none beats the
///   parent**, so the search can take a non-improving (lateral / slightly
///   downhill) step to cross a fuzzy plateau and reach the byte-exact basin the
///   greedy climb stalls before (the d≥2 add-term stall: no single term-delete
///   raises the whole-`.text` gradient, but two deletes reach the exact IL, and
///   the byte-exact terminal — not the gradient — fires when they do).
///
/// Deterministic: neighborhoods are enumerated in a fixed order and the beam is
/// truncated by `(fuzzy desc, .ex bytes asc)`; every judged model is globally
/// de-duplicated by its `.ex` bytes, so no model is compiled twice and the
/// compile budget is spent on new candidates. No wall-clock, no RNG.
///
/// TERMINAL is byte-exact ([`Judged::ByteExact`]) and nothing else — a fuzzy
/// `1.0` that is not byte-exact keeps the search going. On a compile/replay
/// reject the candidate is skipped, never fatal. Budget-bounded (`max_steps`
/// beam rounds, `max_compiles` judgements); an exhausted budget is an honest
/// failure, never a fuzzy "success".
pub fn beam_search(
    seed: &IlModel,
    moves: &MoveSet,
    scorer: &mut dyn Scorer,
    budget: &Budget,
) -> SearchOutcome {
    let width = budget.beam_width.max(1);

    // Judge the seed. (A perturbed seed is not byte-exact, but a caller may hand
    // us an already-solved model — honor it.)
    let seed_judged = scorer.judge(seed);
    if seed_judged == Judged::ByteExact {
        return SearchOutcome {
            solved: true,
            steps: 0,
            compiles: scorer.compiles(),
            best_fuzzy: 1.0,
            reason: StopReason::Solved,
            path: Vec::new(),
        };
    }
    let seed_fuzzy = match seed_judged {
        Judged::Fuzzy(f) => f,
        _ => 0.0, // seed itself did not compile — climb from a zero floor
    };
    let mut best_fuzzy = seed_fuzzy;
    // The highest-fuzzy path seen (the honest "best effort" path on a non-solve).
    let mut best_path: Vec<String> = Vec::new();

    // Global dedup: never compile the same `.ex` twice.
    let mut seen: BTreeSet<Vec<u8>> = BTreeSet::new();
    seen.insert(ex_bytes(seed));

    let mut frontier: Vec<BeamNode> = vec![BeamNode {
        fuzzy: seed_fuzzy,
        model: seed.clone(),
        path: Vec::new(),
    }];

    let done = |solved, path: Vec<String>, best_fuzzy, reason, compiles| SearchOutcome {
        solved,
        steps: path.len(),
        compiles,
        best_fuzzy,
        reason,
        path,
    };

    for _round in 0..budget.max_steps {
        // Expand every frontier node into fresh, de-duplicated, judged candidates.
        let mut cands: Vec<BeamNode> = Vec::new();
        for node in &frontier {
            for (label, cand) in moves.neighbors(&node.model) {
                if !seen.insert(ex_bytes(&cand)) {
                    continue; // already judged this exact model
                }
                if scorer.compiles() >= budget.max_compiles {
                    return done(false, best_path, best_fuzzy, StopReason::CompilesExhausted, scorer.compiles());
                }
                match scorer.judge(&cand) {
                    Judged::ByteExact => {
                        let mut p = node.path.clone();
                        p.push(label);
                        return done(true, p, 1.0, StopReason::Solved, scorer.compiles());
                    }
                    Judged::Fuzzy(f) => {
                        let mut p = node.path.clone();
                        p.push(label);
                        if f > best_fuzzy {
                            best_fuzzy = f;
                            best_path = p.clone();
                        }
                        cands.push(BeamNode {
                            fuzzy: f,
                            model: cand,
                            path: p,
                        });
                    }
                    Judged::Reject => {} // skip cleanly
                }
            }
        }

        if cands.is_empty() {
            // No new distinct candidates anywhere in the beam — converged.
            return done(false, best_path, best_fuzzy, StopReason::LocalOptimum, scorer.compiles());
        }

        if width == 1 {
            // Greedy: the single best candidate, and only if it strictly improves
            // on the parent (else a local optimum). First-seen wins ties.
            let cur = frontier[0].fuzzy;
            let best_idx = cands
                .iter()
                .enumerate()
                .fold(0usize, |bi, (i, n)| if n.fuzzy > cands[bi].fuzzy { i } else { bi });
            if cands[best_idx].fuzzy > cur {
                let chosen = cands.remove(best_idx);
                frontier = vec![chosen];
            } else {
                return done(false, best_path, best_fuzzy, StopReason::LocalOptimum, scorer.compiles());
            }
        } else {
            // Beam: keep the top-k by (fuzzy desc, .ex asc) — a NON-improving step
            // is allowed, which is what crosses the plateau. Deterministic order.
            cands.sort_by(|a, b| {
                b.fuzzy
                    .partial_cmp(&a.fuzzy)
                    .unwrap_or(std::cmp::Ordering::Equal)
                    .then_with(|| ex_bytes(&a.model).cmp(&ex_bytes(&b.model)))
            });
            cands.truncate(width);
            frontier = cands;
        }
    }

    done(false, best_path, best_fuzzy, StopReason::StepsExhausted, scorer.compiles())
}
