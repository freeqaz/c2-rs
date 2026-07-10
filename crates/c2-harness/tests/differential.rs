//! Integration test for the harness. Toolchain-guarded: skips cleanly (never
//! fails) when `Toolchain::locate()` is `None`.
//!
//! Asserts:
//!   * `oracle_selftest` PASSES on the bundled fixtures (determinism + capture
//!     stability against the real toolchain);
//!   * the full `differential` currently returns `PortNotImplemented` (this
//!     documents the open T-E / P0.1 gate).

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use c2_core::PortC2;
use c2_harness::{differential, oracle_selftest, DiffReport, SelfTestOutcome};
use c2_reference::Toolchain;

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
        "c2rs-harness-{tag}-{}-{}-{}",
        std::process::id(),
        nanos,
        n
    ));
    std::fs::create_dir_all(&d).unwrap();
    d
}

#[test]
fn oracle_selftest_passes_on_fixtures() {
    let Some(tc) = Toolchain::locate() else {
        eprintln!("SKIP: toolchain absent");
        return;
    };
    for name in ["add3.cpp", "il_bool_materialization.cpp", "il_call_return.cpp"] {
        let w = work("selftest");
        let report = oracle_selftest(&fixture(name), &tc, &w);
        assert!(
            report.passed(),
            "oracle self-test did not pass for {name}: {:?}",
            report.outcome
        );
        // Sanity: a real capture has a non-empty .ex and a non-empty obj.
        if let SelfTestOutcome::Pass { obj_len, ex_len } = report.outcome {
            assert!(obj_len > 0 && ex_len > 0);
        }
        std::fs::remove_dir_all(&w).ok();
    }
}

#[test]
fn differential_is_port_not_implemented_today() {
    let Some(tc) = Toolchain::locate() else {
        eprintln!("SKIP: toolchain absent");
        return;
    };
    let w = work("diff");
    let port = PortC2;
    let report = differential(&fixture("add3.cpp"), &tc, &port, &w);
    match report {
        DiffReport::PortNotImplemented(_) => {}
        other => panic!("expected PortNotImplemented (the open gate), got {other:?}"),
    }
    std::fs::remove_dir_all(&w).ok();
}
