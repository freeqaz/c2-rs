//! `scripts/cost_arms.py`'s design certificate and its `#3470` preflight,
//! wired so their exit status gates something. Board **#3575**.
//!
//! ---- the two things this closes ------------------------------------------
//!
//! **(1) `#1406`, for a file that already argued for it.** `cost_arms.py`'s
//! own `self_test` docstring cites `#1406` — *"an instrument whose output is
//! quoted as evidence should run under `cargo test` or `scripts/gate.sh`"* —
//! and then nothing ran it. `--self-test` was invoked by hand or not at all,
//! and this protocol has produced two published readings that disagreed in
//! SIGN, which is exactly the class of result that must not rest on a check
//! somebody remembered to type.
//!
//! **(2) `#3470` biting backwards.** The `repo_root()` fix resolves at
//! runtime and cannot reach a binary compiled before it. Every arm of every
//! historical re-run is such a binary **by construction**, so the failure is
//! not a risk to be avoided — it is the default state of the work still owed
//! to `w-s1bc` / `w-s1c2` / `w-s1c3`. The preflight refuses it, and the
//! refusal's own red states are what this file gates.
//!
//! Needs `python3` and SKIPs with a printed reason without it. No toolchain,
//! no build, no network — every arm the self-test probes is a two-line shell
//! script in a throwaway directory, because `#3470`'s signature is entirely
//! "what does it print and what does it exit".

use std::path::{Path, PathBuf};
use std::process::Command;

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn have_python() -> bool {
    Command::new("python3")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// The rotation certificate AND the preflight controls, in one run, because
/// `--self-test` is one command and splitting it here would let a future
/// caller run half of it.
#[test]
fn the_cost_protocols_own_self_test_passes() {
    if !have_python() {
        println!("SKIP: python3 absent — scripts/cost_arms.py cannot run");
        return;
    }
    let root = repo_root();
    let out = Command::new("python3")
        .arg(root.join("scripts/cost_arms.py"))
        .arg("--self-test")
        .current_dir(&root)
        .output()
        .expect("failed to run scripts/cost_arms.py --self-test");
    let mut report = String::from_utf8_lossy(&out.stdout).into_owned();
    report.push_str(&String::from_utf8_lossy(&out.stderr));

    assert!(
        out.status.success(),
        "cost_arms.py --self-test FAILED:\n{report}"
    );
    assert!(
        report.contains("self-test: PASS"),
        "self-test exited 0 without reporting a pass — read the output rather \
         than the status:\n{report}"
    );

    // **Named individually.** `self-test: PASS` is one string, and the value of
    // the run is which cases produced it. A future edit that deletes a control
    // would otherwise still print PASS. The three reds and the ONE GREEN are
    // all required: without the green control a `preflight_arm` that raised
    // unconditionally would score three-for-three on the reds.
    for needle in [
        "skip-exit-0    -> REFUSED",
        "zero-match     -> REFUSED",
        "nonzero-exit   -> REFUSED",
        "grades-one     -> PASSED, denominator 1",
        "cyclic correctly rejected",
    ] {
        assert!(
            report.contains(needle),
            "the self-test no longer exercises `{needle}` — a control that \
             stopped running still prints PASS:\n{report}"
        );
    }
}

/// **The preflight is watched refusing an end-to-end run**, not merely
/// refusing inside its own self-test.
///
/// The distinction is `#3156`'s: a check can be correct and never be reached.
/// `preflight_arm` is called from `main` before the identity block, the
/// rotation certificate and `load_at_start`; this drives the real CLI with two
/// planted arms and requires the whole invocation to exit non-zero **and** to
/// name `#3470` rather than blaming the oracle.
///
/// The wording assertion is the point of the test. The pre-existing in-flight
/// check already refused this input — measured on experiment F's void run,
/// `work/coordinator/expF/runF.txt` — with `SKIP: toolchain absent — the cost
/// protocol needs the oracle`, which sends a reader looking for a missing
/// toolchain that is in fact present and installed. A refusal that names the
/// wrong cause costs the next operator the same re-derivation the last three
/// paid for.
#[test]
fn a_pre_fix_arm_is_refused_end_to_end_and_the_message_names_3470() {
    if !have_python() {
        println!("SKIP: python3 absent");
        return;
    }
    let root = repo_root();
    let dir = std::env::temp_dir().join(format!("c2rs-prefix-arm-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("mkdir");
    let mut planted = Vec::new();
    for name in ["arm-a", "arm-b"] {
        let p = dir.join(name);
        std::fs::write(&p, "#!/bin/sh\necho \"SKIP: toolchain absent\"\nexit 0\n")
            .expect("write plant");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&p, std::fs::Permissions::from_mode(0o755))
                .expect("chmod");
        }
        planted.push(p);
    }
    let out = Command::new("python3")
        .arg(root.join("scripts/cost_arms.py"))
        .arg("--arm")
        .arg(format!("a={}", planted[0].display()))
        .arg("--arm")
        .arg(format!("b={}", planted[1].display()))
        .args(["--rounds", "4", "--port-iters", "10"])
        .current_dir(&root)
        .output()
        .expect("failed to run scripts/cost_arms.py");
    let mut report = String::from_utf8_lossy(&out.stdout).into_owned();
    report.push_str(&String::from_utf8_lossy(&out.stderr));
    let _ = std::fs::remove_dir_all(&dir);

    // A repo with no toolchain refuses earlier, for a different and correct
    // reason. That is not this test's subject and must not be read as a pass.
    if report.contains("no compilers/ under this repo") {
        println!("SKIP: no compilers/ in this tree — the #3470 branch is not reachable here");
        return;
    }

    assert!(
        !out.status.success(),
        "a pre-fix-shaped arm was ACCEPTED — the protocol would have timed a \
         run that graded nothing:\n{report}"
    );
    assert!(
        report.contains("PREFLIGHT REFUSED"),
        "the refusal did not come from the preflight; it fired mid-run, after \
         the setup a quiet box was reserved for:\n{report}"
    );
    assert!(
        report.contains("#3470 biting BACKWARDS"),
        "the refusal does not name #3470. The pre-existing message blamed the \
         oracle, which is present, and the next operator re-derives the remedy \
         a fourth time:\n{report}"
    );
    // And it must NOT have reached the expensive setup.
    assert!(
        !report.contains("arm identity (md5"),
        "the preflight fired AFTER the identity block — it is not a preflight:\n{report}"
    );
}
