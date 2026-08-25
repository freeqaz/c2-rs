//! The `#3156` funnel check, wired so its exit status gates something.
//! Board **#3545**.
//!
//! `scripts/tracked_artifact_audit.sh` answers "does the index contain anything
//! `CLAUDE.md` says is never committed". This file is what makes a red answer
//! *cost* something: a validating script whose exit status gates nothing is
//! decorative, and `#3156` sat open for ten days as a note in a lane's head
//! while the count grew from 19 to 21.
//!
//! **Two tests, and the second is the load-bearing one.** Auditing the tree
//! catches the next `git add -f`. Running the script's own `--self-test`
//! catches something worse and quieter: the guard silently losing the ability
//! to detect a class. A guard that cannot fail passes every run, and this
//! project has now shipped nine instruments that reported green from an
//! absence. So the controls are gated, not just the result.
//!
//! Needs `git`; needs no toolchain, no build and no network.

use std::path::{Path, PathBuf};
use std::process::Command;

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// A tree exported without `.git` (a tarball, `git archive`) cannot be audited
/// and is not a failure of the audit. This is the ONLY skip, it is narrow, and
/// it prints its reason — an audit that skipped for any other cause would be
/// the absence-read-as-success shape the script itself is written against.
fn git_is_usable(root: &Path) -> bool {
    Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["rev-parse", "--git-dir"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn run(args: &[&str]) -> (bool, String) {
    let root = repo_root();
    let out = Command::new("sh")
        .arg(root.join("scripts/tracked_artifact_audit.sh"))
        .args(args)
        .current_dir(&root)
        .output()
        .expect("failed to run scripts/tracked_artifact_audit.sh");
    let mut s = String::from_utf8_lossy(&out.stdout).into_owned();
    s.push_str(&String::from_utf8_lossy(&out.stderr));
    (out.status.success(), s)
}

/// No `.obj`, no `_CL_*`, no `.il`, no build tree, and no absolute machine path
/// on a code surface — the classes `CLAUDE.md` § Commits enumerates.
///
/// The assertion quotes the whole report on failure **on purpose**: the useful
/// output of a red run is the file list, and a bare `assert!(ok)` would make a
/// developer re-run the script by hand to learn anything.
#[test]
fn the_index_carries_no_artifact_claude_md_forbids() {
    let root = repo_root();
    if !git_is_usable(&root) {
        println!("SKIP: no git checkout at {} — cannot audit an index", root.display());
        return;
    }
    let (ok, report) = run(&[]);
    assert!(ok, "tracked-artifact audit FAILED:\n{report}");

    // The denominator, asserted rather than trusted. `#3470` and `#1002`: a
    // clean report over zero files is not a clean report, and the script's own
    // exit-2 path is the thing being checked here from the outside.
    assert!(
        report.contains("tracked files examined: ")
            && !report.contains("tracked files examined: 0"),
        "the audit must print a NONZERO denominator; got:\n{report}"
    );
}

/// **The guard's own controls, gated.**
///
/// `--self-test` plants one violation per covered class in a throwaway repo and
/// requires the audit to go red on each, green again when the plant is removed,
/// and to refuse an empty index with exit 2. If someone narrows a pattern later
/// and a class stops being detectable, the tree audit above stays green — it is
/// green *now* — and only this test can tell.
#[test]
fn every_class_the_guard_claims_is_watched_going_red() {
    let root = repo_root();
    if !git_is_usable(&root) {
        println!("SKIP: no git checkout at {} — --self-test needs git init", root.display());
        return;
    }
    let (ok, report) = run(&["--self-test"]);
    assert!(ok, "tracked-artifact audit SELF-TEST failed:\n{report}");
    assert!(
        report.contains("SELF-TEST PASS"),
        "self-test exited 0 without reporting a pass — read the output rather \
         than the status:\n{report}"
    );
}
