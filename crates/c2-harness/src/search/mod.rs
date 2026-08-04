//! T-A (angle C) — the **IL-space search prototype**.
//!
//! The inversion thesis reduced to practice: given a TARGET `.obj`, search IL
//! space — starting from a seed [`IlModel`], applying K3a edit moves, judging
//! each candidate by compiling it through **real c2** — for an IL whose obj is
//! **byte-exact** to the target. K3a gave the verified edit primitive; this
//! closes the loop into a hill-climber and measures how efficiently it does so.
//!
//! ## Doctrine (CLAUDE.md correctness boundary)
//!
//! The compiler + obj compare is the **sole judge**. A candidate is a SUCCESS
//! only when its c2-compiled obj is **byte-exact** (timestamp-normalized) to the
//! target — [`Judged::ByteExact`], full-obj [`ObjImage::diff`] `Identical`. The
//! `.text` fuzzy score ([`fuzzy_text`]) is the search **gradient ONLY**; it
//! guides the climb and is never a terminal criterion. Every candidate is judged
//! by a REAL replay ([`Toolchain::replay_within`], timeout-bounded per P0.6c) —
//! no simulated scoring on the toolchain path. Edits go through the K3a
//! fail-closed API; an out-of-scope edit refuses cleanly and the search skips it.
//!
//! ## The loop
//!
//! `propose → compile → score → accept`, terminal = byte-exact:
//! 1. From the current [`IlModel`], enumerate a bounded neighborhood of K3a
//!    edits ([`MoveSet::neighbors`]) — each is a fresh candidate model; a refused
//!    edit ([`c2_il::EditError`]) is simply not emitted.
//! 2. Judge each candidate with a [`Scorer`] (the real one replays through c2;
//!    the mock one scores against a target model for the portable tests).
//! 3. If any candidate is byte-exact → **solved**. Else greedily accept the
//!    highest-fuzzy candidate that strictly improves on the current model and
//!    repeat; on no improvement, stop (or take a deterministic restart).
//! Budget-bounded (`max_steps`, `max_compiles`); an exhausted budget is reported
//! as an honest failure, never a fuzzy "success".
//!
//! ## Solvable-instance protocol (the honest solve-rate)
//!
//! A failure must be a real *search* failure, not an unreachable target. So the
//! instances put a solution one move away by construction: capture a fixture,
//! let the **solution IL** be its parsed model and the **target obj** its replay,
//! then perturb the solution by a SMALL known edit *inside the move set* (widen a
//! literal, add/drop a term) to make the **seed**. The inverse edit is in the
//! neighborhood, so a byte-exact IL is provably reachable — the climber either
//! recovers it (measuring search efficiency) or reveals a real gradient failure.
//! [`solve_rate`] runs this over a fixture roster and reports solve-rate@d plus
//! mean compiles-to-solve.

mod engine;
mod from_lifter;
mod from_seed;
mod moves;
mod perturb;
mod scorer;
mod similarity;
mod solve;
#[cfg(test)]
mod tests;

pub use engine::{beam_search, hill_climb, Budget, Judged, Scorer, SearchOutcome, StopReason};
pub use from_lifter::{
    from_lifter_eval, load_lifter_gens, LifterClass, LifterRecord, LifterReport,
};
pub use from_seed::{
    from_retrieval_eval, select_seed, FromSeedClass, FromSeedConfig, FromSeedRecord,
    FromSeedReport, SeedChoice,
};
pub use moves::MoveSet;
pub use perturb::{perturb, perturb_once, Perturb};
pub use scorer::ReplayScorer;
pub use similarity::{fuzzy_text, insn_text_similarity, insn_text_similarity_perfn};
pub use solve::{solve_instance, solve_rate, InstanceResult, SolveReport};
