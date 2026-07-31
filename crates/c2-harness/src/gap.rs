//! `c2-harness::gap` — **real-workload gap scan**: run the whole pipeline
//! (capture → port → byte compare) over a list of *real* project TUs with the
//! project's *real* compile flags, and aggregate exactly where and why the
//! port falls short. This is the measuring tool that turns "the port is a toy"
//! into a ranked list of gaps to close.
//!
//! Each TU lands in exactly one class, ordered from farthest-from-goal to
//! done:
//!
//! * **`capture-fail`** — the reference pipeline itself couldn't compile the
//!   TU here (usually front-end: missing headers, flags we don't replicate).
//!   A gap in the *harness*, not the port; until it captures, the TU can't
//!   even be measured.
//! * **`vocab-gap`** — the bundle captured, but `c2-il` cannot decode its
//!   functions (unknown IL vocabulary). Gap in the IL *model*.
//! * **`codegen-gap`** — functions decode, but `PortC2` returns
//!   `NotImplemented` (feature outside the ported class). Gap in the *port*,
//!   with the codegen's own reason string as the key.
//! * **`mismatch`** — the port *emitted an obj and it differs* from real c2.
//!   The alarming class; anything here is a correctness bug, not a gap.
//! * **`match`** — byte-exact against real c2 (timestamp-normalized RAW
//!   compare; in fact the S_OBJNAME path is threaded in, so RAW-identical).
//!
//! An optional soundness lane replays every Nth captured bundle through
//! standalone c2 (`--replay-every N`) — extending the P0.1 byte-exactness
//! claim from fixtures to real workloads.
//!
//! Nothing here is a gate: the scan always reports; only harness errors (bad
//! list file etc.) fail. The scan writes one JSONL record per TU when asked,
//! for longitudinal diffing of successive scans (are we closing gaps?).

use std::collections::BTreeMap;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

use c2_core::{Backend, BackendError, PortC2};
use c2_obj::{ObjDiff, ObjImage};
use c2_reference::Toolchain;

use crate::capture_cache::CaptureCache;
use crate::provenance::Provenance;

/// Scan configuration (see `c2rs gap --help` for the CLI mapping).
pub struct GapConfig {
    /// Source arguments, passed to `cl.exe` verbatim (relative to `cwd`).
    pub sources: Vec<String>,
    /// Compile flags (should include `/c`; `/Bd` is added by the capture).
    pub flags: Vec<String>,
    /// Working directory for the compiles (project root for relative paths).
    pub cwd: Option<PathBuf>,
    /// Scan at most this many TUs.
    pub limit: Option<usize>,
    /// Concurrency (worker threads; each TU gets its own work subdir).
    pub jobs: usize,
    /// Replay every Nth captured bundle through standalone c2 (0 = never).
    pub replay_every: usize,
    /// Write one JSON record per TU here.
    pub jsonl: Option<PathBuf>,
    /// Scratch root; per-TU subdirs are created below it.
    pub work: PathBuf,
    /// Reference-capture cache root (`None` = `--no-cache`). See
    /// [`crate::capture_cache`] — the key is content-addressed over source
    /// bytes, flags, toolchain and workload-tree identity, never mtimes.
    pub cache: Option<PathBuf>,
    /// Re-capture and byte-compare every Nth cache **hit** (0 = never). The
    /// bypass-and-compare validator that makes a poisoned cache detectable.
    pub validate_cache: usize,
}

/// Outcome class for one TU (see module docs for the ladder).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum TuClass {
    CaptureFail,
    VocabGap,
    CodegenGap,
    PortError,
    Mismatch,
    Match,
}

impl TuClass {
    pub fn label(self) -> &'static str {
        match self {
            TuClass::CaptureFail => "capture-fail",
            TuClass::VocabGap => "vocab-gap",
            TuClass::CodegenGap => "codegen-gap",
            TuClass::PortError => "port-error",
            TuClass::Mismatch => "mismatch",
            TuClass::Match => "match",
        }
    }
}

/// Per-TU scan record.
#[derive(Clone, Debug)]
pub struct TuResult {
    /// Source argument as scanned (relative path, as passed to cl).
    pub src: String,
    pub class: TuClass,
    /// Normalized aggregation key (short, stable): cl error code for
    /// capture-fail, codegen reason for codegen-gap, etc.
    pub reason: String,
    /// Full detail line (first error line / mismatch offsets) for the JSONL.
    pub detail: String,
    /// `.ex` size in bytes (0 when capture failed).
    pub ex_len: usize,
    /// Function count per `.gl` mangled names (0 when capture failed).
    pub fn_names: usize,
    /// Standalone-c2 replay soundness check: `None` = not run this TU.
    pub replay_ok: Option<bool>,
    /// **P2b function-level census**: `.ex` function segments in this TU.
    pub fn_total: usize,
    /// How many of those parse as a modeled shape (the true in-class numerator;
    /// `fn_total - fn_in_class` are blocked).
    pub fn_in_class: usize,
    /// Blocking-feature counts for this TU's out-of-class functions.
    pub fn_blockers: BTreeMap<String, usize>,
    /// **The D6 frame measure** (`docs/IL_CALL_IN_EXPR.md` §18): census key crossed
    /// with the body's CALL-token count class, `"<calls-class>|<census key>"`, over
    /// **every** function including the in-class ones.
    ///
    /// A separate map rather than a suffix on [`TuResult::fn_blockers`]'s keys,
    /// deliberately: the ranked histogram is the widening order and four sessions
    /// of documented tables name its keys, so renaming all of them to carry an
    /// orthogonal fact would break every recorded comparison for no gain. This is
    /// the second axis, kept beside the first.
    pub fn_frames: BTreeMap<String, usize>,
    /// **The control-flow axis** (roadmap #25/#61): the body's decoded CFG shape,
    /// and — for the bodies whose statement layer decodes end to end — that shape
    /// crossed with the census key, `"<cflow class>|<census key>"`.
    ///
    /// This is the **sizing of the block-IR restructure**. Every previous estimate
    /// of that work came from summing blocker rows named after the byte a
    /// straight-line parser stopped on, which `docs/GAPS.md` §6's
    /// unstable-attribution rule says is not the shape's population: a row's size is
    /// not its yield, and a first-blocker attribution is not a shape. The cross
    /// product is, because it says of each shape *how many* bodies have it and
    /// *what else* those bodies are waiting on.
    ///
    /// The cross-tab is emitted only for a decoded body. An undecoded one
    /// contributes just the bare `cf-…` key naming where the statement-layer walk
    /// stopped — crossing "we could not read this body's control flow" with a
    /// blocker would be a product of two ignorances.
    pub fn_cflow: BTreeMap<String, usize>,
    /// **The census/gate cross-check** (roadmap #44): of the functions this TU's
    /// census calls in class, how many does `PortC2`'s own per-function selector
    /// **refuse**, keyed by the refusal.
    ///
    /// It must be empty. Acceptance is supposed to live in the IL parser
    /// precisely so the census and the gate cannot disagree; anything here is the
    /// census over-claiming, and the headline numerator is an upper bound by the
    /// sum of these counts. `docs/GAPS.md` §6: a diagnostic that runs outside the
    /// parser needs a population whose answer is already known, and this is that
    /// population — every in-class function, whose answer should be "accepted".
    pub fn_gate_refusals: BTreeMap<String, usize>,
    /// **The `.gl` binding invariants** (D14). A binding decides *which symbol* a
    /// token names, and the oracle cannot grade a correspondence — a green
    /// differential only says the binding chosen and the binding c2 chose agreed
    /// on the IL tested (`docs/GAPS.md` §6, the `.sy` bullet). So the two facts
    /// the container itself settles are counted on every scan:
    ///
    /// * `"gl-token-ambiguous-dropped"` / `"gl-token-conflict-mangled"` — operand
    ///   tokens `.gl` claims for two different names, dropped rather than resolved
    ///   to the first ([`c2_il::gl_symbol_conflicts`]). Only the second has a known
    ///   answer of 0: `.gl` assigns one token per symbol, so a `?`-mangled name can
    ///   never be one of a disagreeing pair. The first is the type table brushing
    ///   against this reader's record shape, and costs nothing.
    /// * `"dtor-callee-<class>"` — the mangling class of the callee every
    ///   in-class generated empty destructor resolves to. The shape is a
    ///   destructor delegating to a sub-object's destructor, so every one of them
    ///   must be a destructor mangling (`??1` / `??_G` / `??_E` / `??_D`);
    ///   `dtor-callee-other` is the count that says the binding names something
    ///   the shape cannot delegate to, and its known answer is 0.
    pub bind_checks: BTreeMap<String, usize>,
}

/// Aggregated scan report.
pub struct GapReport {
    pub results: Vec<TuResult>,
    /// What produced these numbers (roadmap #46/#48): both trees' git HEADs, the
    /// resolved toolchain paths, the wibo version. `None` only when a report is
    /// built by hand in a test.
    pub provenance: Option<Provenance>,
    /// Capture-cache counters for this scan (all zero when `--no-cache`).
    pub cache: crate::capture_cache::CacheStats,
}

impl GapReport {
    pub fn count(&self, class: TuClass) -> usize {
        self.results.iter().filter(|r| r.class == class).count()
    }

    /// Reasons for `class`, most frequent first, with TU counts.
    pub fn top_reasons(&self, class: TuClass) -> Vec<(String, usize)> {
        let mut map: BTreeMap<&str, usize> = BTreeMap::new();
        for r in self.results.iter().filter(|r| r.class == class) {
            *map.entry(r.reason.as_str()).or_insert(0) += 1;
        }
        let mut v: Vec<(String, usize)> =
            map.into_iter().map(|(k, n)| (k.to_string(), n)).collect();
        v.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        v
    }

    /// **P2b headline**: (functions in class, functions total) across the scan.
    /// Unlike the TU classes this is monotone and fine-grained — it moves on
    /// every widening step, where TU-level `match` stays 0 until a whole TU
    /// happens to be in class.
    pub fn fn_coverage(&self) -> (usize, usize) {
        self.results
            .iter()
            .fold((0, 0), |(a, b), r| (a + r.fn_in_class, b + r.fn_total))
    }

    /// Blocking features across all scanned functions, most frequent first.
    /// **This histogram is the widening order** (docs/ROADMAP.md §G5/P2b).
    pub fn fn_blocker_histogram(&self) -> Vec<(String, usize)> {
        let mut map: BTreeMap<&str, usize> = BTreeMap::new();
        for r in &self.results {
            for (k, n) in &r.fn_blockers {
                *map.entry(k.as_str()).or_insert(0) += n;
            }
        }
        let mut v: Vec<(String, usize)> =
            map.into_iter().map(|(k, n)| (k.to_string(), n)).collect();
        v.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        v
    }

    /// **The D6 frame measure**, aggregated: `"<calls-class>|<census key>"` counts
    /// over every scanned function, most frequent first.
    pub fn fn_frame_histogram(&self) -> Vec<(String, usize)> {
        let mut map: BTreeMap<&str, usize> = BTreeMap::new();
        for r in &self.results {
            for (k, n) in &r.fn_frames {
                *map.entry(k.as_str()).or_insert(0) += n;
            }
        }
        let mut v: Vec<(String, usize)> =
            map.into_iter().map(|(k, n)| (k.to_string(), n)).collect();
        v.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        v
    }

    /// **The control-flow axis**, aggregated, most frequent first. Rows are either
    /// a bare class (`cflow-…` decoded, `cf-…` the decoder's own residue) or a
    /// `"<cflow class>|<census key>"` cross-tab; see [`TuResult::fn_cflow`].
    pub fn fn_cflow_histogram(&self) -> Vec<(String, usize)> {
        let mut map: BTreeMap<&str, usize> = BTreeMap::new();
        for r in &self.results {
            for (k, n) in &r.fn_cflow {
                *map.entry(k.as_str()).or_insert(0) += n;
            }
        }
        let mut v: Vec<(String, usize)> =
            map.into_iter().map(|(k, n)| (k.to_string(), n)).collect();
        v.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        v
    }

    /// How many scanned functions the statement-layer scanner decoded end to end,
    /// and how many it did not — `(decoded, undecoded)`.
    ///
    /// The ratio is the honest bound on everything the control-flow axis claims: a
    /// shape histogram over half the corpus is a shape histogram over half the
    /// corpus, and the other half's CFG is simply not known yet.
    pub fn cflow_decoded_totals(&self) -> (usize, usize) {
        let mut d = 0;
        let mut u = 0;
        for r in &self.results {
            for (k, n) in &r.fn_cflow {
                if k.contains('|') {
                    continue; // a cross-tab row, already counted in its bare class
                }
                if k.starts_with("cflow-") {
                    d += n;
                } else {
                    u += n;
                }
            }
        }
        (d, u)
    }

    /// The three frame classes' totals across the scan, in `calls-0`, `calls-1`,
    /// `calls-2plus` order.
    pub fn frame_class_totals(&self) -> [usize; 3] {
        let mut t = [0usize; 3];
        for r in &self.results {
            for (k, n) in &r.fn_frames {
                let i = match k.split('|').next() {
                    Some("calls-0") => 0,
                    Some("calls-1") => 1,
                    _ => 2,
                };
                t[i] += n;
            }
        }
        t
    }

    /// **The census/gate disagreement**, aggregated: how many censused-in-class
    /// functions `PortC2` refuses, per refusal, most frequent first.
    ///
    /// Every entry is an error term on [`GapReport::fn_coverage`]'s numerator.
    /// The target is an empty list.
    pub fn fn_gate_histogram(&self) -> Vec<(String, usize)> {
        let mut map: BTreeMap<&str, usize> = BTreeMap::new();
        for r in &self.results {
            for (k, n) in &r.fn_gate_refusals {
                *map.entry(k.as_str()).or_insert(0) += n;
            }
        }
        let mut v: Vec<(String, usize)> =
            map.into_iter().map(|(k, n)| (k.to_string(), n)).collect();
        v.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        v
    }

    /// Total censused-in-class functions the port refuses across the scan.
    pub fn fn_gate_disagreement(&self) -> usize {
        self.results
            .iter()
            .map(|r| r.fn_gate_refusals.values().sum::<usize>())
            .sum()
    }

    /// **The `.gl` binding invariants**, aggregated (see [`TuResult::bind_checks`]).
    pub fn bind_check_histogram(&self) -> Vec<(String, usize)> {
        let mut map: BTreeMap<&str, usize> = BTreeMap::new();
        for r in &self.results {
            for (k, n) in &r.bind_checks {
                *map.entry(k.as_str()).or_insert(0) += n;
            }
        }
        let mut v: Vec<(String, usize)> =
            map.into_iter().map(|(k, n)| (k.to_string(), n)).collect();
        v.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        v
    }

    /// The binding invariant that must be **zero**: a generated destructor bound to
    /// a callee that is not a destructor. Nonzero means the `.gl` reader is naming
    /// the wrong symbol in a way no obj comparison over this corpus could have
    /// shown, because these bodies rarely reach an emitter.
    ///
    /// The ambiguity counts are deliberately **not** in here. A token two records
    /// disagree about is dropped, so it is an over-refusal with a measurable cost,
    /// not a wrong binding; the workload's residual is 7, all of them one `.gl`
    /// record form this reader does not model (`$…$initializer$` local statics), and
    /// their measured cost is 0 functions.
    pub fn bind_violations(&self) -> usize {
        self.results
            .iter()
            .map(|r| {
                r.bind_checks
                    .iter()
                    .filter(|(k, _)| {
                        k.as_str() == "dtor-callee-other" || k.as_str() == "dtor-callee-none"
                    })
                    .map(|(_, n)| *n)
                    .sum::<usize>()
            })
            .sum()
    }

    /// Replay soundness: (checked, diverged).
    pub fn replay_stats(&self) -> (usize, usize) {
        let checked = self.results.iter().filter(|r| r.replay_ok.is_some()).count();
        let bad = self
            .results
            .iter()
            .filter(|r| r.replay_ok == Some(false))
            .count();
        (checked, bad)
    }
}

/// Pull a normalized headline out of a cl.exe failure blob: the first line
/// containing `error C`, else the first non-empty line, truncated.
fn normalize_cl_error(blob: &str) -> (String, String) {
    let detail = blob
        .lines()
        .map(str::trim)
        .find(|l| l.contains("error C"))
        .or_else(|| blob.lines().map(str::trim).find(|l| !l.is_empty()))
        .unwrap_or("(no output)")
        .to_string();
    // Aggregation key = the `Cnnnn` code when present, else a clipped line.
    let key = detail
        .split_whitespace()
        .find(|t| {
            let t = t.trim_end_matches(':');
            t.len() >= 4
                && t.starts_with('C')
                && t[1..].chars().all(|c| c.is_ascii_digit())
        })
        .map(|t| t.trim_end_matches(':').to_string())
        .unwrap_or_else(|| clip(&detail, 60));
    (key, clip(&detail, 200))
}

/// A stable bucket key for one of the port's per-function refusals.
///
/// The refusal messages are prose (they are what a `codegen-gap` TU reports), so
/// the key is the leading clause — everything before the first `:` — clipped.
/// That is deliberately the message's own words rather than a hand-maintained
/// enum: a key nobody has to remember to add is a key that cannot go stale, and
/// `docs/GAPS.md` §6's rule against guessed names applies here too. The keys are
/// meant to reach zero, not to be ranked forever.
fn gate_key(msg: &str) -> String {
    let head = msg.split(':').next().unwrap_or(msg).trim();
    clip(head, 72)
}

/// Which destructor mangling a generated empty destructor's resolved callee is —
/// `"other"` when it is not one at all, which is the count that must stay 0.
///
/// MSVC spells the four: `??1` an ordinary destructor, `??_G` the scalar deleting
/// destructor, `??_E` the vector deleting one, `??_D` the vbase destructor. The
/// shape [`c2_il`] parses is a destructor whose whole body delegates to a
/// sub-object's destructor, so the callee is one of these by construction of the
/// *source*, independently of how `.gl` was read — which is what makes this a
/// grader for the binding rather than a restatement of it.
pub fn dtor_callee_class(name: &str) -> &'static str {
    for (p, k) in [
        ("??1", "1"),
        ("??_G", "G"),
        ("??_E", "E"),
        ("??_D", "D"),
    ] {
        if name.starts_with(p) {
            return k;
        }
    }
    "other"
}

fn clip(s: &str, n: usize) -> String {
    if s.len() <= n {
        s.to_string()
    } else {
        let mut end = n;
        while !s.is_char_boundary(end) {
            end -= 1;
        }
        format!("{}…", &s[..end])
    }
}

/// Scan one TU. `work` must be a private (per-TU) directory.
fn scan_one(
    tc: &Toolchain,
    cfg: &GapConfig,
    cache: Option<&CaptureCache>,
    src: &str,
    work: &Path,
    do_replay: bool,
) -> TuResult {
    let mut res = TuResult {
        src: src.to_string(),
        class: TuClass::CaptureFail,
        reason: String::new(),
        detail: String::new(),
        ex_len: 0,
        fn_names: 0,
        replay_ok: None,
        fn_total: 0,
        fn_in_class: 0,
        fn_blockers: BTreeMap::new(),
        fn_frames: BTreeMap::new(),
        fn_cflow: BTreeMap::new(),
        fn_gate_refusals: BTreeMap::new(),
        bind_checks: BTreeMap::new(),
    };

    // 1. Capture: real flags, real cwd, strace keeps bundle + obj. Served from
    //    the content-addressed cache when one is configured — the cache dir IS
    //    the capture dir, so the `-Fo` path c2 bakes into the obj is a function
    //    of the key and a hit is byte-identical to the capture that filled it
    //    (`crate::capture_cache`).
    let capture_result = match cache {
        Some(c) => c.capture(tc, src, &cfg.flags, cfg.cwd.as_deref(), work).0,
        None => tc.capture_reference_with(src, work, &cfg.flags, cfg.cwd.as_deref()),
    };
    let captured =
        match capture_result {
            Ok(c) => c,
            Err(e) => {
                let (key, detail) = normalize_cl_error(&e.to_string());
                res.reason = key;
                res.detail = detail;
                return res;
            }
        };
    // The obj's shape depends on argv the IL bundle does not record: /Gy (implied
    // by /O1 and /O2) puts each function in its own COMDAT .text. Two of the
    // port's per-function refusals are /Gy-only, so the cross-check below needs
    // the same flag the emitter gets.
    let gy = PortC2::flags_imply_function_level_linking(&cfg.flags);
    res.ex_len = captured.bundle.ex().map(|b| b.len()).unwrap_or(0);
    res.fn_names = captured
        .bundle
        .get("gl")
        .map(|gl| c2_il::mangled_names(gl).len())
        .unwrap_or(0);

    // 1b. P2b function-level census — runs regardless of the TU class below, so
    //     even a `vocab-gap` TU contributes its per-function ranking. This is
    //     the only measurement that moves before whole TUs come in class.
    if let Some(census) = captured.bundle.census_functions() {
        res.fn_total = census.len();
        for (f, gate) in &census {
            if f.verdict.in_class() {
                res.fn_in_class += 1;
                // 1c. The cross-check: run the port's own per-function selector
                //     over every function the census claims. A refusal here is a
                //     census/gate disagreement, and it is recorded under its own
                //     key rather than left as a rumour — the numerator is the
                //     public claim, so its error term has to be measured on every
                //     scan (roadmap #44, `docs/GAPS.md` §6).
                let key = match gate {
                    Err(e) => Some((*e).to_string()),
                    Ok(func) => match c2_core::codegen::opt_mode_of_word(f.opt_word) {
                        Err(_) => Some("opt-mode".to_string()),
                        Ok(mode) => c2_core::codegen::function_gate(func, mode, gy)
                            .err()
                            .map(|e| gate_key(&e.to_string())),
                    },
                };
                if let Some(k) = key {
                    *res.fn_gate_refusals.entry(k).or_insert(0) += 1;
                }
            } else {
                *res.fn_blockers.entry(f.verdict.key()).or_insert(0) += 1;
            }
            // The D6 frame axis, over *every* function: the in-class shapes are
            // the control group (all of them are leaves or single tail calls, so
            // a `calls-2plus` reading among them would indict the measure).
            *res.fn_frames
                .entry(format!("{}|{}", f.frame_class(), f.verdict.key()))
                .or_insert(0) += 1;
            // The control-flow axis, likewise over every function — the in-class
            // shapes are the control group here too, and they must all read
            // `cflow-straight`.
            *res.fn_cflow.entry(f.cflow.clone()).or_insert(0) += 1;
            if f.cflow.starts_with("cflow-") {
                *res.fn_cflow
                    .entry(format!("{}|{}", f.cflow, f.verdict.key()))
                    .or_insert(0) += 1;
            }
            // 1d. The binding invariant (D14): what did the `.gl` symbol index
            //     say a generated destructor delegates to? A destructor, always —
            //     anything else is a binding the oracle would have had no chance
            //     to catch, because these bodies rarely reach an emitter.
            if f.verdict.key().starts_with("empty-dtor") {
                if let Ok(func) = gate {
                    let k = match &func.tail_call {
                        Some(c) => dtor_callee_class(c),
                        None => "none",
                    };
                    *res.bind_checks
                        .entry(format!("dtor-callee-{k}"))
                        .or_insert(0) += 1;
                }
            }
        }
    }
    if let Some(gl) = captured.bundle.get("gl") {
        let (dropped, mangled) = c2_il::gl_symbol_conflicts(gl);
        if dropped > 0 {
            *res.bind_checks
                .entry("gl-token-ambiguous-dropped".to_string())
                .or_insert(0) += dropped;
        }
        if mangled > 0 {
            *res.bind_checks
                .entry("gl-token-conflict-mangled".to_string())
                .or_insert(0) += mangled;
        }
    }

    // 2. Optional soundness lane: standalone-c2 replay must reproduce the
    //    pipeline obj on this real bundle.
    if do_replay {
        let ref_obj_path = captured.ref_obj_path.clone();
        res.replay_ok = Some(
            match tc.replay(&captured, &work.join("replay_il"), &ref_obj_path) {
                Ok(obj) => {
                    matches!(ObjImage::diff(&captured.ref_obj, &obj), ObjDiff::Identical)
                }
                Err(_) => false,
            },
        );
        // The replay must write to the reference's own `-Fo` path (that string
        // is inside the obj), which under a cache is the cache entry itself —
        // so restore the captured bytes afterwards. Without this a diverging
        // replay would leave its own output behind as the "cached capture",
        // i.e. the scan would poison its own cache with the thing it was
        // checking for.
        if cache.is_some() {
            let _ = std::fs::write(&ref_obj_path, captured.ref_obj.as_bytes());
        }
    }

    // 3. Vocabulary: can the IL model even decode this bundle's functions?
    if captured.bundle.functions().is_none() {
        res.class = TuClass::VocabGap;
        res.reason = "il function decode failed".to_string();
        res.detail = format!(
            ".ex {} B, {} .gl names — c2_il::functions() = None",
            res.ex_len, res.fn_names
        );
        return res;
    }

    // 4. The port, threaded with the reference's exact -Fo path (S_OBJNAME).
    let obj_name = c2_reference::to_wibo_path(&captured.ref_obj_path);
    // The obj's shape depends on argv the IL bundle does not record: /Gy
    // (implied by /O1 and /O2) puts each function in its own COMDAT .text.
    // Pass the project's real flags so the port can refuse rather than emit a
    // packed .text against a per-function-COMDAT reference.
    let port = PortC2::new(obj_name.clone()).with_function_level_linking(gy);
    match port.compile_to(&captured.bundle, &obj_name) {
        Ok(obj) => match ObjImage::diff(&captured.ref_obj, &obj) {
            ObjDiff::Identical => {
                res.class = TuClass::Match;
                res.reason = "byte-exact".to_string();
            }
            ObjDiff::Differs { first_offset, .. } => {
                res.class = TuClass::Mismatch;
                res.reason = "bytes diverge".to_string();
                res.detail = format!(
                    "first divergence at {first_offset} (ref {} B, port {} B)",
                    captured.ref_obj.len(),
                    obj.len()
                );
            }
        },
        Err(BackendError::NotImplemented(msg)) => {
            res.class = TuClass::CodegenGap;
            res.reason = clip(&msg, 80);
            res.detail = clip(&msg, 200);
        }
        Err(e) => {
            res.class = TuClass::PortError;
            res.reason = clip(&e.to_string(), 80);
            res.detail = clip(&e.to_string(), 200);
        }
    }
    res
}

/// Run the scan: worker pool over the source list, per-TU work subdirs.
/// `progress` is called per finished TU (from worker threads, serialized).
///
/// The scan also records **what produced the numbers** — see
/// [`Provenance`]: a scan whose corpus moved and whose `fn_total` matched anyway
/// is a scan that lied, and the denominator guard alone is proven insufficient.
pub fn gap_scan(
    tc: &Toolchain,
    cfg: &GapConfig,
    progress: &(dyn Fn(usize, usize, &TuResult) + Sync),
) -> std::io::Result<GapReport> {
    let provenance = Provenance::collect(tc, cfg.cwd.as_deref());
    let cache = match &cfg.cache {
        Some(root) => Some(CaptureCache::new(
            root.clone(),
            tc,
            cfg.cwd.as_deref(),
            cfg.validate_cache,
        )?),
        None => None,
    };
    let sources: Vec<&str> = cfg
        .sources
        .iter()
        .map(|s| s.as_str())
        .take(cfg.limit.unwrap_or(usize::MAX))
        .collect();
    let total = sources.len();
    std::fs::create_dir_all(&cfg.work)?;

    let next = AtomicUsize::new(0);
    let done = AtomicUsize::new(0);
    let results: Mutex<Vec<TuResult>> = Mutex::new(Vec::with_capacity(total));
    let jobs = cfg.jobs.max(1).min(total.max(1));

    std::thread::scope(|scope| {
        for worker in 0..jobs {
            let sources = &sources;
            let next = &next;
            let done = &done;
            let results = &results;
            let cache = cache.as_ref();
            scope.spawn(move || loop {
                let i = next.fetch_add(1, Ordering::Relaxed);
                if i >= sources.len() {
                    break;
                }
                let src = sources[i];
                let work = cfg.work.join(format!("tu{i:05}"));
                let _ = std::fs::create_dir_all(&work);
                let do_replay = cfg.replay_every > 0 && i % cfg.replay_every == 0;
                let r = scan_one(tc, cfg, cache, src, &work, do_replay);
                // Bound scratch usage: captured bundles/objs for huge scans
                // add up; the JSONL is the durable record.
                let _ = std::fs::remove_dir_all(&work);
                let n = done.fetch_add(1, Ordering::Relaxed) + 1;
                progress(n, total, &r);
                results.lock().unwrap().push(r);
                let _ = worker;
            });
        }
    });

    let mut results = results.into_inner().unwrap();
    results.sort_by(|a, b| a.src.cmp(&b.src));

    let cache_stats = cache
        .as_ref()
        .map(|c| c.stats())
        .unwrap_or_default();

    if let Some(path) = &cfg.jsonl {
        let mut f = std::fs::File::create(path)?;
        // Record 0 is the provenance header (roadmap #46). Per-TU rows below are
        // unchanged and carry no `record` field, so two scans' rows stay
        // byte-comparable; a consumer skips this one with
        // `if r.get("record"): continue`.
        let extra: Vec<(&str, String)> = vec![
            (
                "cache_root",
                match &cfg.cache {
                    Some(p) => crate::jstr(&p.display().to_string()),
                    None => "null".to_string(),
                },
            ),
            (
                "cache_context",
                match cache.as_ref() {
                    Some(c) => crate::jstr(&c.context_digest()),
                    None => "null".to_string(),
                },
            ),
            ("cache_hits", cache_stats.hits.to_string()),
            ("cache_misses", cache_stats.misses.to_string()),
            ("cache_validated", cache_stats.validated.to_string()),
            ("cache_poisoned", cache_stats.poisoned.to_string()),
            ("tu_count", results.len().to_string()),
            ("replay_every", cfg.replay_every.to_string()),
            ("flags", crate::jstr(&cfg.flags.join(" "))),
        ];
        writeln!(f, "{}", provenance.to_json(&extra))?;
        for r in &results {
            let blockers = r
                .fn_blockers
                .iter()
                .map(|(k, n)| format!("{}:{}", crate::jstr(k), n))
                .collect::<Vec<_>>()
                .join(",");
            let frames = r
                .fn_frames
                .iter()
                .map(|(k, n)| format!("{}:{}", crate::jstr(k), n))
                .collect::<Vec<_>>()
                .join(",");
            let cflow = r
                .fn_cflow
                .iter()
                .map(|(k, n)| format!("{}:{}", crate::jstr(k), n))
                .collect::<Vec<_>>()
                .join(",");
            let gate = r
                .fn_gate_refusals
                .iter()
                .map(|(k, n)| format!("{}:{}", crate::jstr(k), n))
                .collect::<Vec<_>>()
                .join(",");
            let binds = r
                .bind_checks
                .iter()
                .map(|(k, n)| format!("{}:{}", crate::jstr(k), n))
                .collect::<Vec<_>>()
                .join(",");
            writeln!(
                f,
                "{{\"src\":{},\"class\":{},\"reason\":{},\"detail\":{},\"ex_len\":{},\"fn_names\":{},\"replay_ok\":{},\"fn_total\":{},\"fn_in_class\":{},\"fn_blockers\":{{{}}},\"fn_frames\":{{{}}},\"fn_cflow\":{{{}}},\"fn_gate_refusals\":{{{}}},\"bind_checks\":{{{}}}}}",
                crate::jstr(&r.src),
                crate::jstr(r.class.label()),
                crate::jstr(&r.reason),
                crate::jstr(&r.detail),
                r.ex_len,
                r.fn_names,
                match r.replay_ok {
                    None => "null".to_string(),
                    Some(b) => b.to_string(),
                },
                r.fn_total,
                r.fn_in_class,
                blockers,
                frames,
                cflow,
                gate,
                binds,
            )?;
        }
    }

    Ok(GapReport {
        results,
        provenance: Some(provenance),
        cache: cache_stats,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cl_error_normalization_extracts_code() {
        let blob = "capture failed\n  stdout:\n    x.cpp\n    src/x.h(12): fatal error C1083: Cannot open include file: 'foo.h': No such file\n";
        let (key, detail) = normalize_cl_error(blob);
        assert_eq!(key, "C1083");
        assert!(detail.contains("foo.h"));
    }

    #[test]
    fn cl_error_normalization_survives_codeless_blobs() {
        let (key, _) = normalize_cl_error("wibo: something exploded\n");
        assert_eq!(key, "wibo: something exploded");
    }

    fn mk_report(results: Vec<TuResult>) -> GapReport {
        GapReport {
            results,
            provenance: None,
            cache: crate::capture_cache::CacheStats::default(),
        }
    }

    fn mk(reason: &str) -> TuResult {
        TuResult {
            src: "s".into(),
            class: TuClass::CodegenGap,
            reason: reason.into(),
            detail: String::new(),
            ex_len: 0,
            fn_names: 0,
            replay_ok: None,
            fn_total: 0,
            fn_in_class: 0,
            fn_blockers: BTreeMap::new(),
            fn_frames: BTreeMap::new(),
            fn_cflow: BTreeMap::new(),
            fn_gate_refusals: BTreeMap::new(),
            bind_checks: BTreeMap::new(),
        }
    }

    #[test]
    fn report_ranks_reasons_by_count() {
        let rep = mk_report(vec![mk("b"), mk("a"), mk("b")]);
        assert_eq!(
            rep.top_reasons(TuClass::CodegenGap),
            vec![("b".to_string(), 2), ("a".to_string(), 1)]
        );
        assert_eq!(rep.count(TuClass::Match), 0);
    }

    #[test]
    fn fn_census_aggregates_across_tus() {
        // Two TUs: 10 functions each, 3 + 4 in class, blockers summed by key.
        // The point of P2b: coverage is measurable (7/20) even though NO whole
        // TU is in class, so both TUs classify as `codegen-gap` above.
        let mut a = mk("x");
        a.fn_total = 10;
        a.fn_in_class = 3;
        a.fn_blockers.insert("expr-cmp-gt".into(), 5);
        a.fn_blockers.insert("expr-shift".into(), 2);
        let mut b = mk("x");
        b.fn_total = 10;
        b.fn_in_class = 4;
        b.fn_blockers.insert("expr-cmp-gt".into(), 6);
        let rep = mk_report(vec![a, b]);
        assert_eq!(rep.fn_coverage(), (7, 20));
        assert_eq!(
            rep.fn_blocker_histogram(),
            vec![("expr-cmp-gt".to_string(), 11), ("expr-shift".to_string(), 2)]
        );
    }
}
