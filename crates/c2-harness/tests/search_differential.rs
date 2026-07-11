//! T-A — the IL-space search prototype, gated against the live oracle.
//!
//! Proves the inversion loop **closes byte-exact** end-to-end on the MVP class:
//! build a solvable instance from a fixture (solution IL = its parse, target obj
//! = its replay), perturb the solution by a single K3a edit inside the move set,
//! then hill-climb back — every candidate judged by a REAL standalone-c2 replay —
//! and assert the climber recovers an IL whose obj is byte-exact
//! (timestamp-normalized) to the target.
//!
//! Toolchain-gated: skips cleanly (never fails) when the toolchain / strace /
//! mingw are absent, exactly like `edit_differential.rs`.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use c2_harness::search::{self, Budget, MoveSet, Perturb};
use c2_reference::Toolchain;

const TIMEOUT: Duration = Duration::from_secs(60);
static COUNTER: AtomicU64 = AtomicU64::new(0);

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("fixtures/cpp")
        .join(name)
}

fn work(tag: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    let d = std::env::temp_dir().join(format!(
        "c2rs-search-{tag}-{}-{}-{}",
        std::process::id(),
        nanos,
        n
    ));
    std::fs::create_dir_all(&d).unwrap();
    d
}

/// The toolchain + replay prerequisites, or `None` to skip cleanly.
fn ready() -> Option<Toolchain> {
    let tc = Toolchain::locate()?;
    if !tc.has_strace() {
        eprintln!("SKIP: strace absent");
        return None;
    }
    if !tc.has_mingw() {
        eprintln!("SKIP: i686-w64-mingw32-gcc absent");
        return None;
    }
    Some(tc)
}

/// A single d=1 add-term instance on `a+5` recovers byte-exact: seed `(a+5)+a`
/// (an inserted redundant term), climbed back to the `a+5` target obj by a real
/// c2-judged delete move.
#[test]
fn search_solves_addterm_d1_byte_exact() {
    let Some(tc) = ready() else { return };
    let w = work("addterm");

    let r = search::solve_instance(
        &tc,
        &fixture("mvp_edit_addk.cpp"),
        Perturb::AddTerm,
        1,
        &MoveSet::default(),
        &Budget::default(),
        &w,
        TIMEOUT,
    );

    assert!(r.error.is_none(), "instance errored: {:?}", r.error);
    let outcome = r.outcome.expect("add-term has a site on a+5");
    assert!(
        outcome.solved,
        "the inversion loop must close byte-exact: {outcome:?}"
    );
    // Recovery of a single inserted term is one accepted move (a delete).
    assert!(outcome.steps >= 1, "a real recovery took at least one move");
    // Every judgement was a real compile; the seed + neighborhood were bounded.
    assert!(outcome.compiles >= 1);

    std::fs::remove_dir_all(&w).ok();
}

/// A d=1 literal-value nudge recovers byte-exact: perturb `a+5`'s literal to
/// `a+8`, climb back to `a+5` via a value move (the flat-gradient enumeration
/// case). Uses the last/only-function regime (`mvp_edit_addk`) so no `.gl`
/// bookkeeping beyond what the API discharges is needed.
#[test]
fn search_solves_litnudge_d1_byte_exact() {
    let Some(tc) = ready() else { return };
    let w = work("litnudge");

    let r = search::solve_instance(
        &tc,
        &fixture("mvp_edit_addk.cpp"),
        Perturb::LitNudge,
        1,
        &MoveSet::default(),
        &Budget::default(),
        &w,
        TIMEOUT,
    );

    assert!(r.error.is_none(), "instance errored: {:?}", r.error);
    let outcome = r.outcome.expect("lit-nudge has a site on a+5");
    assert!(
        outcome.solved,
        "literal-value recovery must close byte-exact: {outcome:?}"
    );

    std::fs::remove_dir_all(&w).ok();
}
