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

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::time::Duration;

use c2_il::{ExToken, IlModel};
use c2_obj::{ObjDiff, ObjImage};
use c2_reference::{CapturedReference, Toolchain};

use crate::retrieval::text_section;

// ===========================================================================
// Gradient — the `.text` fuzzy score (search guide ONLY, never terminal)
// ===========================================================================

/// PPC-word (4-byte) match ratio between a candidate obj and the target, over
/// their COFF `.text` sections. `1.0` iff the emitted code matches word-for-word
/// (which, combined with matching relocs/headers, is the byte-exact case the
/// terminal check confirms separately); `0.0` on disjoint code.
///
/// `.text`-only by design (per P1.3 / il-witness P1.3): the full obj embeds its
/// `/Fo` path in `S_OBJNAME`, so a whole-obj ratio would be path-dominated. The
/// gradient scores the *code*; the terminal success check is full
/// timestamp-normalized byte equality (see [`Judged`]). Objs are compared on
/// their normalized bytes so the COFF `TimeDateStamp` never perturbs the score.
pub fn fuzzy_text(cand: &ObjImage, target: &ObjImage) -> f64 {
    let cn = cand.normalized();
    let tn = target.normalized();
    let (ct, _) = text_section(&cn);
    let (tt, _) = text_section(&tn);
    word_match_ratio(ct, tt)
}

/// Fraction of aligned 4-byte words that are equal, over `max(words_a, words_b)`
/// (so a length mismatch is penalized). Trailing bytes shorter than a word are
/// compared as a final partial word. Two empty slices score `1.0` (vacuously
/// equal); one empty and one not scores `0.0`.
fn word_match_ratio(a: &[u8], b: &[u8]) -> f64 {
    if a.is_empty() && b.is_empty() {
        return 1.0;
    }
    let wa = a.len().div_ceil(4);
    let wb = b.len().div_ceil(4);
    let denom = wa.max(wb);
    if denom == 0 {
        return 1.0;
    }
    let mut matched = 0usize;
    for w in 0..wa.min(wb) {
        let lo = w * 4;
        let hi_a = (lo + 4).min(a.len());
        let hi_b = (lo + 4).min(b.len());
        if a[lo..hi_a] == b[lo..hi_b] {
            matched += 1;
        }
    }
    matched as f64 / denom as f64
}

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
// Move set — the K3a-licensed neighborhood
// ===========================================================================

/// Which K3a edit families the neighborhood enumerates. The default is the full
/// licensed family; [`MoveSet::length_only`] restricts to the pure length moves
/// (widen/narrow + term add/delete) that P0.6a proved re-optimize byte-exact.
#[derive(Clone, Debug)]
pub struct MoveSet {
    /// Widen/narrow the varint form of each int literal (same value; P0.6a A/B).
    pub widen_narrow: bool,
    /// Nudge each int literal by each delta in `value_nudges` (relative, so a
    /// value perturbation of magnitude ≤ the nudge range is recoverable). The
    /// emitted immediate is a flat field, so this is search-by-trial, not a
    /// smooth gradient — kept to a small local window.
    pub literal_value: bool,
    /// Relative deltas tried by `literal_value` (`current + delta`).
    pub value_nudges: Vec<i32>,
    /// Delete a trailing `<operand> <op>` term (`a+b+c` → `a+b`; P0.6a F).
    pub term_delete: bool,
    /// Insert a `<operand> <op>` term after an existing op (`a+5` → `(a+5)+5`;
    /// P0.6a E). Operands reused from the function; ops from `insert_ops`.
    pub term_insert: bool,
    /// Binary ops used when inserting a term.
    pub insert_ops: Vec<ExToken>,
}

impl Default for MoveSet {
    fn default() -> Self {
        MoveSet {
            widen_narrow: true,
            literal_value: true,
            value_nudges: vec![-10, -5, -3, -2, -1, 1, 2, 3, 5, 10],
            term_delete: true,
            term_insert: true,
            insert_ops: vec![ExToken::Add, ExToken::Sub, ExToken::Mul],
        }
    }
}

impl MoveSet {
    /// The pure length-move family: widen/narrow + term add/delete, no literal
    /// value enumeration. This is the family K3a/P0.6a licensed as
    /// re-optimize-to-byte-exact, and the one whose recovery the climber guides
    /// structurally rather than by trial.
    pub fn length_only() -> Self {
        MoveSet {
            widen_narrow: true,
            literal_value: false,
            value_nudges: Vec::new(),
            term_delete: true,
            term_insert: true,
            insert_ops: vec![ExToken::Add, ExToken::Sub, ExToken::Mul],
        }
    }

    /// Enumerate the bounded neighborhood of `model`: every in-scope K3a edit,
    /// applied to a fresh clone. A refused edit ([`c2_il::EditError`]) is skipped
    /// (fail-closed — the model is left untouched by a failed splice). Candidates
    /// are deduplicated by their `.ex` bytes and returned in a deterministic
    /// order, each labelled for the readout/log.
    pub fn neighbors(&self, model: &IlModel) -> Vec<(String, IlModel)> {
        let mut out: Vec<(String, IlModel)> = Vec::new();
        let mut seen: BTreeSet<Vec<u8>> = BTreeSet::new();
        // The seed's own `.ex` is not a neighbor of itself.
        if let Some(ex) = model.encode().get("ex") {
            seen.insert(ex.to_vec());
        }

        let nfns = model.ex_function_count();
        for fi in 0..nfns {
            let Ok(tokens) = model.function_tokens(fi) else {
                continue; // opaque body — not token-addressable
            };

            // ---- widen / narrow each literal -------------------------------
            if self.widen_narrow {
                for (ti, tok) in tokens.iter().enumerate() {
                    if let ExToken::Lit { wide, .. } = tok {
                        let mut cand = model.clone();
                        if cand.set_literal_wide(fi, ti, !wide).is_ok() {
                            let label = format!(
                                "fn{fi} lit@{ti} {}",
                                if *wide { "narrow" } else { "widen" }
                            );
                            push(&mut out, &mut seen, label, cand);
                        }
                    }
                }
            }

            // ---- nudge each literal by a local delta ------------------------
            if self.literal_value {
                for (ti, tok) in tokens.iter().enumerate() {
                    if let ExToken::Lit { value, wide } = *tok {
                        for &d in &self.value_nudges {
                            let Some(v) = value.checked_add(d) else {
                                continue;
                            };
                            if v == value {
                                continue;
                            }
                            let mut cand = model.clone();
                            // Keep the same varint width where the value permits;
                            // a narrow slot with a wide value must widen.
                            let want_wide = wide || !(0..=0x7F).contains(&v);
                            let repl = vec![ExToken::Lit { value: v, wide: want_wide }];
                            if cand
                                .splice_function_tokens(fi, ti..ti + 1, repl)
                                .is_ok()
                            {
                                push(&mut out, &mut seen, format!("fn{fi} lit@{ti} {v:+}"), cand);
                            }
                        }
                    }
                }
            }

            // ---- delete a trailing `<operand> <op>` term -------------------
            if self.term_delete {
                for i in 0..tokens.len().saturating_sub(1) {
                    if is_operand(&tokens[i]) && is_binop(&tokens[i + 1]) {
                        let mut cand = model.clone();
                        if cand.splice_function_tokens(fi, i..i + 2, vec![]).is_ok() {
                            push(&mut out, &mut seen, format!("fn{fi} del term@{i}"), cand);
                        }
                    }
                }
            }

            // ---- insert a `<operand> <op>` term after a value token ---------
            // A value-producing token (operand OR binop) leaves exactly one net
            // value on the stack, so appending `<operand> <op>` after it is
            // always valid postfix (`… V` → `… (V op W)`). Anchoring after
            // operands too — not only ops — lets insert reconstruct a term even
            // when the seed body has no remaining binop (e.g. a fully-dropped
            // single term), the direction P0.6a E exercised.
            if self.term_insert {
                let operands: Vec<ExToken> = distinct_operands(&tokens);
                for (i, tok) in tokens.iter().enumerate() {
                    if !is_operand(tok) && !is_binop(tok) {
                        continue;
                    }
                    for operand in &operands {
                        for op in &self.insert_ops {
                            let mut cand = model.clone();
                            let repl = vec![operand.clone(), op.clone()];
                            if cand
                                .splice_function_tokens(fi, i + 1..i + 1, repl)
                                .is_ok()
                            {
                                let label = format!("fn{fi} ins@{i} {}", op_name(op));
                                push(&mut out, &mut seen, label, cand);
                            }
                        }
                    }
                }
            }
        }
        out
    }
}

fn push(
    out: &mut Vec<(String, IlModel)>,
    seen: &mut BTreeSet<Vec<u8>>,
    label: String,
    cand: IlModel,
) {
    let ex = cand
        .encode()
        .get("ex")
        .map(|b| b.to_vec())
        .unwrap_or_default();
    if seen.insert(ex) {
        out.push((label, cand));
    }
}

fn is_operand(t: &ExToken) -> bool {
    matches!(t, ExToken::Load(_) | ExToken::Lit { .. })
}

fn is_binop(t: &ExToken) -> bool {
    matches!(t, ExToken::Add | ExToken::Sub | ExToken::Mul)
}

fn op_name(t: &ExToken) -> &'static str {
    match t {
        ExToken::Add => "add",
        ExToken::Sub => "sub",
        ExToken::Mul => "mul",
        _ => "op",
    }
}

/// The distinct operand tokens in a body (each `Load`/`Lit`), in first-seen
/// order — the operands a term-insert reuses.
fn distinct_operands(tokens: &[ExToken]) -> Vec<ExToken> {
    let mut out: Vec<ExToken> = Vec::new();
    for t in tokens {
        if is_operand(t) && !out.contains(t) {
            out.push(t.clone());
        }
    }
    out
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
}

impl Default for Budget {
    fn default() -> Self {
        Budget {
            max_steps: 8,
            max_compiles: 400,
            restarts: 0,
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

/// Greedy IL-space hill-climb from `seed`, judged by `scorer`, bounded by
/// `budget`, exploring the [`MoveSet`] neighborhood.
///
/// Deterministic: the neighborhood order is fixed and ties are broken by
/// first-seen (the enumeration order), so with a deterministic scorer the whole
/// climb is reproducible — no wall-clock, no RNG on the default (0-restart) path.
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
    // Baseline: judge the seed. (A perturbed seed is not byte-exact, but a caller
    // may hand us an already-solved model — honor it.)
    let mut current = seed.clone();
    let seed_judged = scorer.judge(&current);
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
    let mut current_fuzzy = match seed_judged {
        Judged::Fuzzy(f) => f,
        _ => 0.0, // seed itself did not compile — climb from a zero floor
    };
    let mut best_fuzzy = current_fuzzy;
    let mut path: Vec<String> = Vec::new();

    for _restart in 0..=budget.restarts {
        // A restart returns to the seed (deterministic; see Budget::restarts).
        if _restart > 0 {
            current = seed.clone();
            current_fuzzy = match scorer.judge(&current) {
                Judged::ByteExact => {
                    return solved_now(scorer, path, best_fuzzy);
                }
                Judged::Fuzzy(f) => f,
                Judged::Reject => 0.0,
            };
            if scorer.compiles() >= budget.max_compiles {
                return SearchOutcome {
                    solved: false,
                    steps: path.len(),
                    compiles: scorer.compiles(),
                    best_fuzzy,
                    reason: StopReason::CompilesExhausted,
                    path,
                };
            }
        }

        for _step in 0..budget.max_steps {
            let neighbors = moves.neighbors(&current);
            let mut best: Option<(f64, String, IlModel)> = None;
            for (label, cand) in neighbors {
                if scorer.compiles() >= budget.max_compiles {
                    return SearchOutcome {
                        solved: false,
                        steps: path.len(),
                        compiles: scorer.compiles(),
                        best_fuzzy,
                        reason: StopReason::CompilesExhausted,
                        path,
                    };
                }
                match scorer.judge(&cand) {
                    Judged::ByteExact => {
                        path.push(label);
                        return solved_now(scorer, path, 1.0);
                    }
                    Judged::Fuzzy(f) => {
                        if f > best_fuzzy {
                            best_fuzzy = f;
                        }
                        // Strictly-better-than-current, first-seen wins ties.
                        let better = match &best {
                            Some((bf, _, _)) => f > *bf,
                            None => true,
                        };
                        if better {
                            best = Some((f, label, cand));
                        }
                    }
                    Judged::Reject => {} // skip cleanly
                }
            }

            match best {
                Some((f, label, cand)) if f > current_fuzzy => {
                    current = cand;
                    current_fuzzy = f;
                    path.push(label);
                }
                _ => break, // local optimum — try a restart if any remain
            }

            if path.len() >= budget.max_steps {
                return SearchOutcome {
                    solved: false,
                    steps: path.len(),
                    compiles: scorer.compiles(),
                    best_fuzzy,
                    reason: StopReason::StepsExhausted,
                    path,
                };
            }
        }
    }

    SearchOutcome {
        solved: false,
        steps: path.len(),
        compiles: scorer.compiles(),
        best_fuzzy,
        reason: StopReason::LocalOptimum,
        path,
    }
}

fn solved_now(scorer: &dyn Scorer, path: Vec<String>, best_fuzzy: f64) -> SearchOutcome {
    SearchOutcome {
        solved: true,
        steps: path.len(),
        compiles: scorer.compiles(),
        best_fuzzy,
        reason: StopReason::Solved,
        path,
    }
}

// ===========================================================================
// ReplayScorer — the real c2 judge
// ===========================================================================

/// Judges candidates by a REAL standalone-c2 replay through the toolchain, to a
/// FIXED `-Fo` path (so the embedded `S_OBJNAME` matches the target and a
/// byte-exact terminal is achievable), bounded by a timeout (a replay
/// crash/timeout is a clean [`Judged::Reject`], per P0.6c). This is the sole
/// judge on the real path — no simulated scoring.
pub struct ReplayScorer<'a> {
    tc: &'a Toolchain,
    base: &'a CapturedReference,
    target: ObjImage,
    scratch: PathBuf,
    /// The FIXED `-Fo` path every replay (target render + all candidates) uses,
    /// so the embedded `S_OBJNAME` matches and a byte-exact terminal is possible.
    pub fo: PathBuf,
    timeout: Duration,
    counter: usize,
    compiles: usize,
}

impl<'a> ReplayScorer<'a> {
    /// `base` supplies the captured c2 argv (its `-il`/`-Fo` are swapped per
    /// replay); `target` is the obj to reach byte-exact; `scratch` is a private
    /// work dir (candidate bundles + the fixed `-Fo` obj land under it). The
    /// fixed `-Fo` is [`ReplayScorer::fo`] — render the target to it (see
    /// [`solve_instance`]) so target and candidates share the embedded path.
    pub fn new(
        tc: &'a Toolchain,
        base: &'a CapturedReference,
        target: ObjImage,
        scratch: PathBuf,
        timeout: Duration,
    ) -> Self {
        let fo = scratch.join("cand.obj");
        ReplayScorer {
            tc,
            base,
            target,
            scratch,
            fo,
            timeout,
            counter: 0,
            compiles: 0,
        }
    }

    /// The fixed `-Fo` path candidates and the target both replay to.
    pub fn fo_path(&self) -> &Path {
        &self.fo
    }
}

impl<'a> Scorer for ReplayScorer<'a> {
    fn judge(&mut self, model: &IlModel) -> Judged {
        self.compiles += 1;
        self.counter += 1;
        let cap = CapturedReference {
            bundle: model.encode(),
            ..self.base.clone()
        };
        let dir = self.scratch.join(format!("cand{}", self.counter));
        let verdict = match self
            .tc
            .replay_within(&cap, &dir, &self.fo, self.timeout)
        {
            Ok(obj) => {
                if matches!(ObjImage::diff(&obj, &self.target), ObjDiff::Identical) {
                    Judged::ByteExact
                } else {
                    Judged::Fuzzy(fuzzy_text(&obj, &self.target))
                }
            }
            Err(_) => Judged::Reject, // crash / timeout / no obj — skip cleanly
        };
        let _ = std::fs::remove_dir_all(&dir);
        verdict
    }

    fn compiles(&self) -> usize {
        self.compiles
    }
}

// ===========================================================================
// Solvable-instance harness — the honest solve-rate
// ===========================================================================

/// One perturbation family used to build a solvable instance from a solution IL.
///
/// Each family's inverse is in the [`MoveSet`], so a byte-exact IL is reachable
/// by construction — a failure is a real *search* failure. Note that `WidenLit`
/// is **obj-invisible** on the real toolchain (P0.6a A: c2 re-optimizes a
/// widened literal to byte-identical code), so it is a valid perturbation only in
/// `.ex`-space (the mock scorer / unit tests); the real solve-rate roster uses
/// the obj-changing families (`AddTerm`, `LitNudge`, `DropTerm`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Perturb {
    /// Widen a narrow literal — `.ex`-visible but obj-invisible (see above).
    WidenLit,
    /// Insert `d` redundant `+<operand>` terms (seed longer; recover by `d`
    /// deletes — a genuine obj change, gradient-guided for `d ≥ 2`).
    AddTerm,
    /// Nudge a literal by `+3d` (seed's immediate differs; recover by a value
    /// move — flat gradient, so recovery is enumeration within the nudge window).
    LitNudge,
    /// Delete a trailing term (seed shorter; recover by insert — only where the
    /// dropped operand survives elsewhere in the body).
    DropTerm,
}

impl Perturb {
    /// A short name for logs/readouts.
    pub fn label(&self) -> &'static str {
        match self {
            Perturb::WidenLit => "widen-lit",
            Perturb::AddTerm => "add-term",
            Perturb::LitNudge => "lit-nudge",
            Perturb::DropTerm => "drop-term",
        }
    }
}

/// Apply a `d`-step perturbation to `solution`, returning the seed model, or
/// `None` if there is no site (the instance is skipped, never faked). `d` is the
/// perturbation distance: `d` stacked edits, whose `d`-move inverse the climber
/// must find.
pub fn perturb(solution: &IlModel, kind: Perturb, d: usize) -> Option<IlModel> {
    let mut m = solution.clone();
    for _ in 0..d.max(1) {
        m = perturb_step(&m, kind)?;
    }
    // A perturbation that produced no net `.ex` change is not a real instance.
    if m.encode().get("ex") == solution.encode().get("ex") {
        return None;
    }
    Some(m)
}

/// A d=1 perturbation (one edit). Retained for the unit tests.
pub fn perturb_once(solution: &IlModel, kind: Perturb) -> Option<IlModel> {
    perturb(solution, kind, 1)
}

fn perturb_step(model: &IlModel, kind: Perturb) -> Option<IlModel> {
    let nfns = model.ex_function_count();
    for fi in 0..nfns {
        let Ok(tokens) = model.function_tokens(fi) else {
            continue;
        };
        match kind {
            Perturb::WidenLit => {
                for (ti, tok) in tokens.iter().enumerate() {
                    if matches!(tok, ExToken::Lit { wide: false, .. }) {
                        let mut m = model.clone();
                        if m.set_literal_wide(fi, ti, true).is_ok() {
                            return Some(m);
                        }
                    }
                }
            }
            Perturb::AddTerm => {
                // Duplicate the first operand as a `+<operand>` term after the
                // first binop — a redundant term the climber removes by delete.
                let operands = distinct_operands(&tokens);
                let first_op = tokens.iter().position(is_binop);
                if let (Some(operand), Some(i)) = (operands.first(), first_op) {
                    let mut m = model.clone();
                    let repl = vec![operand.clone(), ExToken::Add];
                    if m.splice_function_tokens(fi, i + 1..i + 1, repl).is_ok() {
                        return Some(m);
                    }
                }
            }
            Perturb::LitNudge => {
                for (ti, tok) in tokens.iter().enumerate() {
                    if let ExToken::Lit { value, wide } = *tok {
                        let v = value.wrapping_add(3);
                        let mut m = model.clone();
                        let want_wide = wide || !(0..=0x7F).contains(&v);
                        let repl = vec![ExToken::Lit { value: v, wide: want_wide }];
                        if m.splice_function_tokens(fi, ti..ti + 1, repl).is_ok() {
                            return Some(m);
                        }
                    }
                }
            }
            Perturb::DropTerm => {
                for i in 0..tokens.len().saturating_sub(1) {
                    if is_operand(&tokens[i]) && is_binop(&tokens[i + 1]) {
                        let mut m = model.clone();
                        if m.splice_function_tokens(fi, i..i + 2, vec![]).is_ok() {
                            return Some(m);
                        }
                    }
                }
            }
        }
    }
    None
}

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

    let outcome = hill_climb(&seed, moves, &mut scorer, budget);
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

#[cfg(test)]
mod tests {
    use super::*;

    // A toolchain-free scorer: judges a candidate by comparing its `.ex` bytes to
    // a fixed target model. ByteExact on equality; else a fuzzy score over the
    // fraction of matching bytes (a stand-in gradient). Exercises the climber's
    // accept / terminal / budget / reject logic with zero toolchain.
    struct MockScorer {
        target_ex: Vec<u8>,
        compiles: usize,
        /// `.ex` byte prefixes that should be treated as a compile reject.
        reject_if_contains: Option<Vec<u8>>,
    }

    impl MockScorer {
        fn new(target: &IlModel) -> Self {
            MockScorer {
                target_ex: target.encode().get("ex").unwrap().to_vec(),
                compiles: 0,
                reject_if_contains: None,
            }
        }
    }

    impl Scorer for MockScorer {
        fn judge(&mut self, model: &IlModel) -> Judged {
            self.compiles += 1;
            let ex = model.encode().get("ex").unwrap().to_vec();
            if let Some(marker) = &self.reject_if_contains {
                if ex.windows(marker.len()).any(|w| w == marker.as_slice()) {
                    return Judged::Reject;
                }
            }
            if ex == self.target_ex {
                return Judged::ByteExact;
            }
            let matched = ex
                .iter()
                .zip(&self.target_ex)
                .filter(|(a, b)| a == b)
                .count();
            let denom = ex.len().max(self.target_ex.len()).max(1);
            Judged::Fuzzy(matched as f64 / denom as f64)
        }
        fn compiles(&self) -> usize {
            self.compiles
        }
    }

    // A hand-built model: one function, body `LOAD a + 5`, with a `.gl` offset —
    // reuses the corpus synthetic-bundle shape but adds a literal so the move set
    // has widen/narrow + value + insert/delete sites.
    fn model_add_lit(lit: i32, wide: bool) -> IlModel {
        use c2_il::IlBundle;
        let mut b = IlBundle::new("_search_test");
        let mut ex: Vec<u8> = Vec::new();
        ex.extend_from_slice(&c2_il::EX_MAGIC);
        ex.extend_from_slice(&[0x00; 8]);
        let fn_start = ex.len() as u32;
        ex.extend_from_slice(&[0x4F, 0x1F]); // fn start
        ex.extend_from_slice(&[0x11, 0x22]); // opaque meta
        ex.push(0x46); // Formals
        ex.extend_from_slice(&[0x2D, 0xE3, 0x01]); // Formal a
        ex.extend_from_slice(&[0x4C, 0x4F, 0x11]); // LO
        ex.push(0x53); // Ss
        ex.extend_from_slice(&[0xB9, 0xE3, 0x01, 0x86, 0x41, 0x74]); // Load a
        // literal
        ex.push(0x33);
        ex.extend_from_slice(&[0x86, 0x41, 0x74]);
        if wide {
            ex.push(0x80);
            ex.extend_from_slice(&lit.to_le_bytes());
        } else {
            ex.push(lit as u8);
        }
        ex.push(0x02); // Add
        ex.extend_from_slice(&[0x54, 0x02, 0x29, 0xE3, 0x00]); // Return
        ex.extend_from_slice(&[0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00]); // FnTail
        ex.extend_from_slice(&[0x4F, 0x02, 0x20, 0x00, 0x4F, 0x01, 0x00, 0x4D]); // ModuleEnd
        b.set("ex", ex);

        let mut gl: Vec<u8> = Vec::new();
        gl.extend_from_slice(b"?addk@@YAHH@Z\x00");
        gl.extend_from_slice(&[0x80, 0x01, 0x10, 0x00, 0x00, 0x00, 0x00]);
        gl.push(0x80);
        gl.extend_from_slice(&fn_start.to_le_bytes());
        b.set("gl", gl);
        b.set("sy", b"a\x00\x00".to_vec());
        b.set("in", vec![0x86, 0x41, 0x74, 0x00]);
        b.set("db", Vec::new());
        IlModel::parse(&b).expect("hand-built model parses")
    }

    // A hand-built model with body `a + a` (LOAD a, LOAD a, ADD) — a repeated
    // operand, so a dropped `+a` term is reconstructable by insert (the operand
    // survives in the seed). Same framing/`.gl` shape as `model_add_lit`.
    fn model_add_aa() -> IlModel {
        use c2_il::IlBundle;
        let mut b = IlBundle::new("_search_test_aa");
        let mut ex: Vec<u8> = Vec::new();
        ex.extend_from_slice(&c2_il::EX_MAGIC);
        ex.extend_from_slice(&[0x00; 8]);
        let fn_start = ex.len() as u32;
        ex.extend_from_slice(&[0x4F, 0x1F]);
        ex.extend_from_slice(&[0x11, 0x22]);
        ex.push(0x46);
        ex.extend_from_slice(&[0x2D, 0xE3, 0x01]); // Formal a
        ex.extend_from_slice(&[0x4C, 0x4F, 0x11]); // LO
        ex.push(0x53); // Ss
        ex.extend_from_slice(&[0xB9, 0xE3, 0x01, 0x86, 0x41, 0x74]); // Load a
        ex.extend_from_slice(&[0xB9, 0xE3, 0x01, 0x86, 0x41, 0x74]); // Load a
        ex.push(0x02); // Add
        ex.extend_from_slice(&[0x54, 0x02, 0x29, 0xE3, 0x00]); // Return
        ex.extend_from_slice(&[0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00]); // FnTail
        ex.extend_from_slice(&[0x4F, 0x02, 0x20, 0x00, 0x4F, 0x01, 0x00, 0x4D]); // ModuleEnd
        b.set("ex", ex);
        let mut gl: Vec<u8> = Vec::new();
        gl.extend_from_slice(b"?adda@@YAHH@Z\x00");
        gl.extend_from_slice(&[0x80, 0x01, 0x10, 0x00, 0x00, 0x00, 0x00]);
        gl.push(0x80);
        gl.extend_from_slice(&fn_start.to_le_bytes());
        b.set("gl", gl);
        b.set("sy", b"a\x00\x00".to_vec());
        b.set("in", vec![0x86, 0x41, 0x74, 0x00]);
        b.set("db", Vec::new());
        IlModel::parse(&b).expect("hand-built aa model parses")
    }

    #[test]
    fn word_match_ratio_basics() {
        assert_eq!(word_match_ratio(&[], &[]), 1.0);
        assert_eq!(word_match_ratio(&[1, 2, 3, 4], &[1, 2, 3, 4]), 1.0);
        assert_eq!(word_match_ratio(&[1, 2, 3, 4], &[9, 9, 9, 9]), 0.0);
        // one of two words matches
        let r = word_match_ratio(&[1, 2, 3, 4, 5, 6, 7, 8], &[1, 2, 3, 4, 0, 0, 0, 0]);
        assert!((r - 0.5).abs() < 1e-9);
        // length mismatch penalized (1 word vs 2)
        let r = word_match_ratio(&[1, 2, 3, 4], &[1, 2, 3, 4, 5, 6, 7, 8]);
        assert!((r - 0.5).abs() < 1e-9);
    }

    #[test]
    fn neighbors_are_in_scope_and_deduped() {
        let m = model_add_lit(5, false);
        let moves = MoveSet::default();
        let ns = moves.neighbors(&m);
        assert!(!ns.is_empty(), "expected a non-empty neighborhood");
        // Every neighbor round-trips (a refused edit is never emitted) and is
        // distinct from the seed and from each other by `.ex`.
        let seed_ex = m.encode().get("ex").unwrap().to_vec();
        let mut seen = BTreeSet::new();
        for (_label, cand) in &ns {
            let ex = cand.encode().get("ex").unwrap().to_vec();
            assert_ne!(ex, seed_ex, "a neighbor equals the seed");
            assert!(seen.insert(ex), "duplicate neighbor emitted");
        }
        // There is a widen move (the narrow literal → wide).
        assert!(ns.iter().any(|(l, _)| l.contains("widen")));
    }

    #[test]
    fn climber_recovers_a_widen_perturbation() {
        // Target = solution `a + 5` (narrow). Seed = widened literal (d=1). The
        // narrow move must recover the target byte-exact in one step.
        let solution = model_add_lit(5, false);
        let seed = perturb_once(&solution, Perturb::WidenLit).expect("has a lit");
        assert_ne!(
            seed.encode().get("ex"),
            solution.encode().get("ex"),
            "perturbation must change the seed"
        );
        let mut scorer = MockScorer::new(&solution);
        let out = hill_climb(&seed, &MoveSet::default(), &mut scorer, &Budget::default());
        assert!(out.solved, "d=1 widen must be recoverable: {out:?}");
        assert_eq!(out.reason, StopReason::Solved);
        assert!(out.steps <= 1, "widen recovery is one move");
    }

    #[test]
    fn climber_recovers_an_added_term_by_delete() {
        // Seed = solution + a redundant term; delete recovers it.
        let solution = model_add_lit(5, false);
        let seed = perturb_once(&solution, Perturb::AddTerm).expect("has an operand+op");
        let mut scorer = MockScorer::new(&solution);
        let out = hill_climb(&seed, &MoveSet::default(), &mut scorer, &Budget::default());
        assert!(out.solved, "added term must be removable: {out:?}");
    }

    #[test]
    fn budget_bounds_compiles_and_reports_failure() {
        // An unreachable target (different literal, value moves off) with a tiny
        // compile budget must stop honestly, not loop.
        let solution = model_add_lit(5, false);
        let seed = model_add_lit(9, true); // 2 edits away, value moves disabled
        let mut scorer = MockScorer::new(&solution);
        let budget = Budget {
            max_steps: 8,
            max_compiles: 6,
            restarts: 0,
        };
        let out = hill_climb(&seed, &MoveSet::length_only(), &mut scorer, &budget);
        assert!(!out.solved);
        assert!(scorer.compiles() <= 6, "compile budget must bound the run");
        assert!(matches!(
            out.reason,
            StopReason::CompilesExhausted | StopReason::LocalOptimum
        ));
    }

    #[test]
    fn climber_skips_rejects_cleanly() {
        // Mark every wide-literal candidate a reject; the climber must skip them
        // and still find another path — the value nudge 8 + (−3) = 5 recovers the
        // target (−3 is in the default nudge window).
        let solution = model_add_lit(5, false);
        let seed = model_add_lit(8, false);
        let mut scorer = MockScorer::new(&solution);
        // Reject any candidate carrying the wide-literal marker `80` after the
        // int-type — forces the value path rather than widen.
        scorer.reject_if_contains = Some(vec![0x86, 0x41, 0x74, 0x80]);
        let out = hill_climb(&seed, &MoveSet::default(), &mut scorer, &Budget::default());
        assert!(out.solved, "value nudge 8->5 recovers despite rejects: {out:?}");
    }

    #[test]
    fn already_solved_seed_is_zero_step_success() {
        let solution = model_add_lit(5, false);
        let mut scorer = MockScorer::new(&solution);
        let out = hill_climb(&solution, &MoveSet::default(), &mut scorer, &Budget::default());
        assert!(out.solved);
        assert_eq!(out.steps, 0);
    }

    #[test]
    fn perturb_drop_then_recover_by_insert() {
        // Solution `a + a` (a repeated operand); drop the trailing `+a` → seed
        // `a`; the insert move must put `+a` back. The dropped operand (`a`) is
        // still available in the seed body, so insert-recovery reconstructs it
        // byte-exact — the direction P0.6a E exercised (a genuinely-grown stream).
        let solution = model_add_aa();
        let seed = perturb_once(&solution, Perturb::DropTerm).expect("has a term");
        assert_ne!(
            seed.encode().get("ex"),
            solution.encode().get("ex"),
            "drop must shorten the seed"
        );
        let mut scorer = MockScorer::new(&solution);
        let out = hill_climb(&seed, &MoveSet::default(), &mut scorer, &Budget::default());
        assert!(out.solved, "dropped term must be reinsertable: {out:?}");
    }
}
