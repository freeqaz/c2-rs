//! `scripts/status.sh --check`, wired so its exit status gates something.
//! Board **#3577**, closing the assertion half of **#3510**.
//!
//! ---- what was actually wrong -------------------------------------------
//!
//! `docs/STATUS.md`'s generated block is what this project reads to answer
//! "where is this". Two of its rows are the emit-set ceilings, and the
//! function behind them — `GapReport::emit_set_violations` — carries a control
//! whose own doc says a nonzero value **voids the ceiling**:
//!
//! > *"a nonzero here means `fn_total` and `emit-emitted` are not counting the
//! > things this reading says they count, and the ceiling above is void … It
//! > is the control that makes the ceiling a measurement rather than an
//! > argument."*
//!
//! It reads **1**. Every scan printed it. **Nothing read it**, and the block
//! published `Emit-set ceiling, LO-anchored … 26 of 870` regardless — on the
//! page whose own header says it carries *the traps*.
//!
//! `status.sh` grew a `--check` mode for exactly this class of defect, and
//! `--check` was itself **wired into nothing**: no `cargo test` target, no
//! `gate.sh` row. A validating flag whose exit status gates nothing is
//! decorative, which is the same sentence `tracked_artifact_audit.sh` was
//! written under. This file is the wiring.
//!
//! `--check` needs no toolchain, no dc3 tree and no network: it runs the real
//! parsers and the real `ceiling_row` branch against a captured probe report
//! that ships inside the script.

use std::path::{Path, PathBuf};
use std::process::Command;

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn run_check(root: &Path) -> (bool, String) {
    let out = Command::new("sh")
        .arg(root.join("scripts/status.sh"))
        .arg("--check")
        .current_dir(root)
        .output()
        .expect("failed to run scripts/status.sh --check");
    let mut s = String::from_utf8_lossy(&out.stdout).into_owned();
    s.push_str(&String::from_utf8_lossy(&out.stderr));
    (out.status.success(), s)
}

/// The registry, every parser, and the ceiling's VOID branch.
#[test]
fn the_status_collectors_own_check_passes() {
    let root = repo_root();
    let (ok, report) = run_check(&root);
    assert!(ok, "scripts/status.sh --check FAILED:\n{report}");
    assert!(
        report.contains("STATUS CHECK: PASS"),
        "--check exited 0 without reporting a pass — read the output rather \
         than the status:\n{report}"
    );
}

/// **The ceiling's VOID branch, named individually.**
///
/// `STATUS CHECK: PASS` is one string, and a future edit that deleted the void
/// case would still print it — the remaining parser checks all read a CLEAN
/// control and stay green without it. Both poles are required: a `ceiling_row`
/// that always returned VOID would satisfy the red case on its own, and a
/// `ceiling_row` that never did would satisfy the clean one.
///
/// The third line is the guard against the quietest failure of the three. The
/// red case is produced by `sed`-ing the probe log, and a `sed` that matched
/// nothing would leave a clean log and "pass" by testing the clean path twice
/// — a mutation that did not apply, reported as a control that fired.
#[test]
fn the_voided_ceiling_is_watched_refusing_to_publish_a_number() {
    let root = repo_root();
    let (_, report) = run_check(&root);
    for needle in [
        "ceiling rows, control CLEAN -> published as numbers",
        "ceiling rows, control RED (1 violation) -> VOID, both anchors",
        "the control itself is published as its own row",
    ] {
        assert!(
            report.contains(needle),
            "--check no longer exercises `{needle}` — #3510's control would go \
             back to being printed and unasserted:\n{report}"
        );
    }
}

/// The registry denominator, asserted rather than trusted.
///
/// `#3470` / `#1002`: a clean report over zero metrics is not a clean report,
/// and `--check`'s whole design rests on walking the registry rather than the
/// set of lines that happen to exist. A registry that shrank to nothing would
/// otherwise print PASS.
#[test]
fn the_check_reports_a_nonzero_registry_denominator() {
    let root = repo_root();
    let (_, report) = run_check(&root);
    let n: usize = report
        .split("STATUS CHECK: PASS — ")
        .nth(1)
        .and_then(|s| s.split_whitespace().next())
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    assert!(
        n >= 20,
        "the metric registry reads {n} — a --check over an empty registry is \
         not a check:\n{report}"
    );
}
