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
}

/// Aggregated scan report.
pub struct GapReport {
    pub results: Vec<TuResult>,
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
    };

    // 1. Capture: real flags, real cwd, strace keeps bundle + obj.
    let captured =
        match tc.capture_reference_with(src, work, &cfg.flags, cfg.cwd.as_deref()) {
            Ok(c) => c,
            Err(e) => {
                let (key, detail) = normalize_cl_error(&e.to_string());
                res.reason = key;
                res.detail = detail;
                return res;
            }
        };
    res.ex_len = captured.bundle.ex().map(|b| b.len()).unwrap_or(0);
    res.fn_names = captured
        .bundle
        .get("gl")
        .map(|gl| c2_il::mangled_names(gl).len())
        .unwrap_or(0);

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
    let port = PortC2::new(obj_name.clone());
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
pub fn gap_scan(
    tc: &Toolchain,
    cfg: &GapConfig,
    progress: &(dyn Fn(usize, usize, &TuResult) + Sync),
) -> std::io::Result<GapReport> {
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
            scope.spawn(move || loop {
                let i = next.fetch_add(1, Ordering::Relaxed);
                if i >= sources.len() {
                    break;
                }
                let src = sources[i];
                let work = cfg.work.join(format!("tu{i:05}"));
                let _ = std::fs::create_dir_all(&work);
                let do_replay = cfg.replay_every > 0 && i % cfg.replay_every == 0;
                let r = scan_one(tc, cfg, src, &work, do_replay);
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

    if let Some(path) = &cfg.jsonl {
        let mut f = std::fs::File::create(path)?;
        for r in &results {
            writeln!(
                f,
                "{{\"src\":{},\"class\":{},\"reason\":{},\"detail\":{},\"ex_len\":{},\"fn_names\":{},\"replay_ok\":{}}}",
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
            )?;
        }
    }

    Ok(GapReport { results })
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

    #[test]
    fn report_ranks_reasons_by_count() {
        let mk = |reason: &str| TuResult {
            src: "s".into(),
            class: TuClass::CodegenGap,
            reason: reason.into(),
            detail: String::new(),
            ex_len: 0,
            fn_names: 0,
            replay_ok: None,
        };
        let rep = GapReport {
            results: vec![mk("b"), mk("a"), mk("b")],
        };
        assert_eq!(
            rep.top_reasons(TuClass::CodegenGap),
            vec![("b".to_string(), 2), ("a".to_string(), 1)]
        );
        assert_eq!(rep.count(TuClass::Match), 0);
    }
}
