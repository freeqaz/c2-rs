//! `c2-harness::testsupport` — **the constants and scratch-directory helpers
//! that `crates/c2-harness/tests/*` were each keeping their own copy of.**
//!
//! It lives in the library, and not in a `tests/testsupport/mod.rs`, for the
//! ordinary Rust reason: an integration test binary can only see a crate's
//! `pub` surface, and a `mod.rs` included into thirty-odd binaries is thirty-odd
//! copies again, each with its own `dead_code` warnings for the parts that
//! binary does not use. Nothing in the shipping paths calls anything here.
//!
//! # Why the flag list is the item that mattered
//!
//! `REFACTOR_REVIEW_2026-08-20.md` §3.2 measured ~1,100 duplicated lines across
//! `crates/*/tests/` and judged nearly all of it benign lane-shaped duplication
//! the repo has already adjudicated — including the four `grade_cell` copies,
//! deliberately unmerged, reasoning at `tests/cellgrade/mod.rs:1-22`, migration
//! tracked as board #1094. **Untouched here.**
//!
//! The exception it named is [`WORKLOAD_FLAGS`]: fourteen identical spellings of
//! the workload's own compile profile, in fourteen test files, each of which
//! decides *which compiler* that file's cells are graded under. If the
//! workload's profile ever moves, a missed copy keeps grading the old mode and
//! **reads green** — the absence family wearing a flags list. One spelling, and
//! a moved profile moves every cell at once or fails to compile.

use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

/// **The dc3 workload's own compile profile**, minus the `/I` paths a
/// standalone cell cannot use — the flags `work/dc3-workload/flags.txt` carries
/// and the 878-TU scan runs at.
///
/// `/O1` is load-bearing and is why this is not the fixtures' default: **`/O1`
/// implies `/Gy`**, so each function lands in its own COMDAT `.text`, which is
/// the regime every per-function verdict in this repo is defined in. `/Ox` does
/// not imply it, and a cell captured at `/Ox` produces a packed obj with no
/// `.text` COMDAT at all — every per-function assertion over it passes
/// vacuously. Each converted site kept its own doc comment saying so in its own
/// words; those stay, because they say what the flags mean *for that cell*.
/// PROV[N] not load-bearing — a MEASUREMENT PROFILE: the dc3 workload's own compiler flags, shared by the tests that grade against it. A property of the corpus, not a value derived from `c2.dll`.
pub const WORKLOAD_FLAGS: [&str; 8] = [
    "/nologo", "/wd4355", "/wd4164", "/c", "/GR", "/O1", "/Oi", "/EHsc",
];

/// [`WORKLOAD_FLAGS`] as owned strings — the form every capture entry point
/// takes.
pub fn workload_flags() -> Vec<String> {
    WORKLOAD_FLAGS.iter().map(|s| s.to_string()).collect()
}

/// Monotonic tie-breaker for [`unique_scratch_dir`]. One per test binary, which
/// is what the per-file `static COUNTER`s it replaces were.
/// PROV[N] not load-bearing — a monotonic tie-breaker for unique scratch directories. Scratch state.
static SCRATCH_SEQ: AtomicUsize = AtomicUsize::new(0);

/// `<tmp>/c2rs-<prefix>-<tag>-<pid>`, created.
///
/// **Stable for one `(prefix, tag)` within one process** — two tests that pass
/// the same tag share the directory, which is exactly what board #1045 records
/// as a fabricated finding (four parallel tests sharing one PID-keyed directory
/// raced their captures), and exactly why the tag is a parameter. Use a distinct
/// tag per test, or [`unique_scratch_dir`] when a call needs its own.
pub fn scratch_dir(prefix: &str, tag: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("c2rs-{prefix}-{tag}-{}", std::process::id()));
    std::fs::create_dir_all(&d).unwrap();
    d
}

/// `<tmp>/c2rs-<prefix>-<tag>-<pid>-<nanos>-<seq>`, created.
///
/// A fresh directory **per call**, so repeated calls with one tag cannot alias.
pub fn unique_scratch_dir(prefix: &str, tag: &str) -> PathBuf {
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    let n = SCRATCH_SEQ.fetch_add(1, Ordering::Relaxed);
    let d = std::env::temp_dir().join(format!(
        "c2rs-{prefix}-{tag}-{}-{nanos}-{n}",
        std::process::id()
    ));
    std::fs::create_dir_all(&d).unwrap();
    d
}

/// `<tmp>/c2rs-<prefix>-<tag>-<pid>-<nanos>`, **emptied** and created.
///
/// The variant used where a test asserts on the *contents* of the directory (a
/// cache root with exactly one entry, a `--keep-il` drop) and a leftover from an
/// earlier run would be read as this run's output.
pub fn clean_scratch_dir(prefix: &str, tag: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let d = std::env::temp_dir().join(format!(
        "c2rs-{prefix}-{tag}-{}-{nanos}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The flag list is a *contract with the workload*, so it is checked against
    /// the workload's own `flags.txt` rather than against a second copy of
    /// itself: every token here must appear there, in this order, and the only
    /// tokens `flags.txt` adds are `/I` include paths a standalone cell cannot
    /// use. When `flags.txt` is absent (a checkout without the workload inputs)
    /// the test says so and returns — the file is generated, not tracked.
    #[test]
    fn workload_flags_are_the_workload_s_flags_in_order() {
        let path = crate::provenance::main_repo_root().join("work/dc3-workload/flags.txt");
        let Ok(text) = std::fs::read_to_string(&path) else {
            println!(
                "SKIP: {} absent (generated by scripts/gen_dc3_workload.sh, not tracked)",
                path.display()
            );
            return;
        };
        let actual: Vec<&str> = text.split_whitespace().collect();
        // The workload's tokens, minus the `/I <path>` pairs.
        let mut kept: Vec<&str> = Vec::new();
        let mut skip_next = false;
        for t in &actual {
            if skip_next {
                skip_next = false;
                continue;
            }
            if *t == "/I" {
                skip_next = true;
                continue;
            }
            kept.push(t);
        }
        assert_eq!(
            kept,
            WORKLOAD_FLAGS.to_vec(),
            "WORKLOAD_FLAGS no longer matches work/dc3-workload/flags.txt with the /I \
             paths removed. One of the two moved; every cell test in \
             crates/c2-harness/tests/ is graded at WORKLOAD_FLAGS and would keep \
             grading the old mode."
        );
    }

    #[test]
    fn a_shared_scratch_dir_is_stable_and_a_unique_one_is_not() {
        assert_eq!(
            scratch_dir("tsupport", "stable"),
            scratch_dir("tsupport", "stable"),
            "the pid-keyed helper must return ONE directory per (prefix, tag) — \
             tests that rely on writing a cell once depend on it"
        );
        let a = unique_scratch_dir("tsupport", "fresh");
        let b = unique_scratch_dir("tsupport", "fresh");
        assert_ne!(
            a, b,
            "the unique helper must never alias — board #1045 is a fabricated \
             finding from two tests sharing one capture directory"
        );
        assert!(a.is_dir() && b.is_dir(), "both are created, not just named");
        for d in [scratch_dir("tsupport", "stable"), a, b] {
            let _ = std::fs::remove_dir_all(d);
        }
    }
}
