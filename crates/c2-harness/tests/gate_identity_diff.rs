//! `scripts/gate_identity_diff.sh`'s controls, gated. Board **#3579**.
//!
//! The identity diff is the **only** instrument that can see the failure
//! `w-latent` measured: a change that is `GATE: PASS`, `GATE_EXIT=0`,
//! `mismatch 0` and still costs six matched TUs. `mismatch 0` is not `match
//! unchanged`, and no gate's own verdict can tell the difference.
//!
//! It has been a prose procedure in `HOWTO_DIFF.md` that every merging lane
//! retypes — `#3451`'s standing complaint, one artifact over from the cost
//! protocol it was filed against. A retyped diff goes wrong in three specific
//! places (the prose `flags` column, the PID-named run dir, the two `n/a`
//! rows), and every one of them turns a real movement into "no differences" or
//! a phantom one into a scare.
//!
//! So the procedure is a script, and this file is what stops the script
//! quietly losing the ability to detect a movement. **The load-bearing case is
//! `#3515`'s measured one-TU-refused signature**: six `/O1` lanes at −1 each
//! plus `debug-lane` at their sum, required to come out at exactly **14 diff
//! lines over 7 rows**.
//!
//! Portable: no toolchain, no build, no network, and the self-test generates
//! its own table rather than reading one out of `work/` — which is gitignored
//! scratch and, as this very lane's `#3552` shows, one reap away from gone.

use std::path::{Path, PathBuf};
use std::process::Command;

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn run(args: &[&str]) -> (bool, String) {
    let root = repo_root();
    let out = Command::new("sh")
        .arg(root.join("scripts/gate_identity_diff.sh"))
        .args(args)
        .current_dir(&root)
        .output()
        .expect("failed to run scripts/gate_identity_diff.sh");
    let mut s = String::from_utf8_lossy(&out.stdout).into_owned();
    s.push_str(&String::from_utf8_lossy(&out.stderr));
    (out.status.success(), s)
}

/// Enumeration, the silent control, the known signature, and the truncation
/// refusal.
#[test]
fn the_identity_diffs_own_controls_all_fire() {
    let (ok, report) = run(&["--self-test"]);
    assert!(ok, "gate_identity_diff.sh --self-test FAILED:\n{report}");
    assert!(
        report.contains("SELF-TEST PASS"),
        "self-test exited 0 without reporting a pass:\n{report}"
    );

    // Named individually. `SELF-TEST PASS` is one string and the value of the
    // run is which cases produced it; a deleted control would still print it.
    for needle in [
        "enumeration: 21 count-bearing rows",
        "control: a table against itself",
        "-> 14 lines, 7 rows",
        "the signature case exits NONZERO",
        "a TRUNCATED table -> exit 2",
    ] {
        assert!(
            report.contains(needle),
            "the self-test no longer exercises `{needle}`:\n{report}"
        );
    }
}

/// **The denominator refusal, from the outside.**
///
/// `#3470` / `#1002`: a short extraction and a genuinely-identical pair both
/// print "no differences", and a diff that cannot tell them apart is the
/// absence-reads-as-success shape. This drives it on a file that is not a gate
/// table at all — the commonest way to get a zero-row extraction is to point
/// the tool at the wrong file — and requires **exit 2**, not exit 0.
#[test]
fn a_table_that_is_not_a_gate_table_is_refused_rather_than_read_as_identical() {
    let root = repo_root();
    let not_a_table = root.join("README.md");
    if !not_a_table.exists() {
        println!("SKIP: no README.md to use as a non-table input");
        return;
    }
    let p = not_a_table.to_string_lossy().into_owned();
    let out = Command::new("sh")
        .arg(root.join("scripts/gate_identity_diff.sh"))
        .args([&p, &p])
        .current_dir(&root)
        .output()
        .expect("failed to run scripts/gate_identity_diff.sh");
    let mut report = String::from_utf8_lossy(&out.stdout).into_owned();
    report.push_str(&String::from_utf8_lossy(&out.stderr));

    assert_eq!(
        out.status.code(),
        Some(2),
        "diffing a non-table against itself must exit 2, not report it identical:\n{report}"
    );
    assert!(
        report.contains("count-bearing rows, expected 21"),
        "the refusal must name the row count it got and the one it wanted:\n{report}"
    );
}
