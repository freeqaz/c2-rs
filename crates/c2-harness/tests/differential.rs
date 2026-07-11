//! Integration test for the harness. Toolchain-guarded: skips cleanly (never
//! fails) when `Toolchain::locate()` is `None`.
//!
//! Asserts:
//!   * `oracle_selftest` PASSES on the bundled fixtures (determinism + capture
//!     stability against the real toolchain);
//!   * the full `differential` reports the reference replay is **byte-exact**
//!     (P0.1 proven) AND the port is still `NotImplemented` (open T-E gate).

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use c2_core::PortC2;
use c2_harness::{differential, oracle_selftest, DiffReport, PortStatus, SelfTestOutcome};
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
fn differential_reference_byte_exact_port_not_implemented() {
    let Some(tc) = Toolchain::locate() else {
        eprintln!("SKIP: toolchain absent");
        return;
    };
    if !tc.has_strace() {
        eprintln!("SKIP: strace absent (needed to keep the IL bundle)");
        return;
    }
    if !tc.has_mingw() {
        eprintln!("SKIP: i686-w64-mingw32-gcc absent (needed to build c2host)");
        return;
    }
    let w = work("diff");
    let port = PortC2::default();
    let report = differential(&fixture("add3.cpp"), &tc, &port, &w);
    match report {
        DiffReport::ReferenceReplayByteExact { port, .. } => {
            assert!(
                matches!(port, PortStatus::NotImplemented(_)),
                "expected the port to still be NotImplemented, got {port:?}"
            );
        }
        other => panic!(
            "expected ReferenceReplayByteExact (P0.1 proven) with PortNotImplemented, got {other:?}"
        ),
    }
    std::fs::remove_dir_all(&w).ok();
}

/// MVP milestone: the native port emits a **byte-exact** `.obj` for the single
/// straight-line int add-chain function `int add3(int,int,int)`. The harness
/// threads the reference's exact `-Fo` path into the port (S_OBJNAME wiring),
/// so the whole obj — header, 5 sections, symbol + string tables — matches on
/// timestamp-normalized bytes.
#[test]
fn differential_mvp_add3_port_byte_exact() {
    let Some(tc) = Toolchain::locate() else {
        eprintln!("SKIP: toolchain absent");
        return;
    };
    if !tc.has_strace() {
        eprintln!("SKIP: strace absent (needed to keep the IL bundle)");
        return;
    }
    if !tc.has_mingw() {
        eprintln!("SKIP: i686-w64-mingw32-gcc absent (needed to build c2host)");
        return;
    }
    let w = work("mvp");
    let port = PortC2::default();
    let report = differential(&fixture("mvp_add3.cpp"), &tc, &port, &w);
    match report {
        DiffReport::ReferenceReplayByteExact { port, .. } => {
            assert_eq!(
                port,
                PortStatus::Match,
                "expected the port to be byte-exact on mvp_add3, got {port:?}"
            );
        }
        other => panic!("expected ReferenceReplayByteExact, got {other:?}"),
    }
    std::fs::remove_dir_all(&w).ok();
}

/// Multi-function widening: a TU of two straight-line int functions
/// (`add2`, `add4`) is byte-exact. Exercises the multi-`.text`-symbol COFF path
/// — cumulative `Value` offsets, contiguous packing, `NumberOfSymbols = 13+N`.
#[test]
fn differential_mvp_two_multifunction_byte_exact() {
    let Some(tc) = Toolchain::locate() else {
        eprintln!("SKIP: toolchain absent");
        return;
    };
    if !tc.has_strace() || !tc.has_mingw() {
        eprintln!("SKIP: strace/mingw absent");
        return;
    }
    let w = work("mvptwo");
    let port = PortC2::default();
    let report = differential(&fixture("mvp_two.cpp"), &tc, &port, &w);
    match report {
        DiffReport::ReferenceReplayByteExact { port, .. } => {
            assert_eq!(
                port,
                PortStatus::Match,
                "expected the port to be byte-exact on mvp_two, got {port:?}"
            );
        }
        other => panic!("expected ReferenceReplayByteExact, got {other:?}"),
    }
    std::fs::remove_dir_all(&w).ok();
}

/// W3: literals / immediates. `mvp_lit.cpp` is a 3-function TU: `a+5` (addi),
/// `a-5` (addi with negated imm), and `return 42` (li = addi rD,r0,k). Proves
/// the operand-stack Reg/Imm model and the constant-folding into `addi`.
#[test]
fn differential_mvp_lit_immediates_byte_exact() {
    let Some(tc) = Toolchain::locate() else {
        eprintln!("SKIP: toolchain absent");
        return;
    };
    if !tc.has_strace() || !tc.has_mingw() {
        eprintln!("SKIP: strace/mingw absent");
        return;
    }
    let w = work("mvplit");
    let port = PortC2::default();
    let report = differential(&fixture("mvp_lit.cpp"), &tc, &port, &w);
    match report {
        DiffReport::ReferenceReplayByteExact { port, .. } => {
            assert_eq!(
                port,
                PortStatus::Match,
                "expected the port to be byte-exact on mvp_lit, got {port:?}"
            );
        }
        other => panic!("expected ReferenceReplayByteExact, got {other:?}"),
    }
    std::fs::remove_dir_all(&w).ok();
}

/// W2: non-commutative int ops. `mvp_sub.cpp` is a 3-function TU mixing `-`
/// (subf, reversed operands), `*` (mullw), and `+`. Byte-exact here proves the
/// subf operand-order mapping AND the 8-byte inter-function `.text` alignment
/// (three 12-byte functions → offsets 0x0/0x10/0x20 with zero-padding between).
#[test]
fn differential_mvp_sub_noncommutative_byte_exact() {
    let Some(tc) = Toolchain::locate() else {
        eprintln!("SKIP: toolchain absent");
        return;
    };
    if !tc.has_strace() || !tc.has_mingw() {
        eprintln!("SKIP: strace/mingw absent");
        return;
    }
    let w = work("mvpsub");
    let port = PortC2::default();
    let report = differential(&fixture("mvp_sub.cpp"), &tc, &port, &w);
    match report {
        DiffReport::ReferenceReplayByteExact { port, .. } => {
            assert_eq!(
                port,
                PortStatus::Match,
                "expected the port to be byte-exact on mvp_sub, got {port:?}"
            );
        }
        other => panic!("expected ReferenceReplayByteExact, got {other:?}"),
    }
    std::fs::remove_dir_all(&w).ok();
}
