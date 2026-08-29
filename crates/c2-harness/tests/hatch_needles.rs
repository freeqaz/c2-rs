//! **`work/w-front3/hatch.py`'s needles must resolve against this tree.**
//!
//! Coordinator, 2026-08-29. Board **#3786**, **#3787**.
//!
//! # The twenty days this exists because of
//!
//! `hatch.py` is the frontier ladder's lift: eight `(key, path, needle, lift)`
//! edits it splices into `crates/` and reverts. `scripts/gate.sh`'s `hatch-red`
//! row fires every one of them. A needle is an **exact substring** of the
//! clause it lifts, so any edit to that clause — even a whitespace one —
//! unmoors it.
//!
//! `call-arg-lit-permuted`'s clause gained a fourth conjunct (`!lit_inserted`)
//! at `530383eca` on **2026-08-09**. The needle stopped matching there and
//! stayed unmoored for **1,681 commits**. Across that span every gate run
//! printed `GATE: PASS (HATCH-RED REFUSED)`, five waves of lanes quoted the
//! qualifier as inherited noise, and per `#1406` none of those runs
//! established what a full one does.
//!
//! **Three layers each reported it and none acted:**
//!
//! 1. `gate.sh` treats a drifted needle as `REFUSED`, not `FAIL` — correct,
//!    because drift is a property of the tree and a lane mid-wave is routinely
//!    in that state. But `REFUSED` exits 0 and costs only a parenthetical.
//! 2. `hatch.py check` printed the `HATCH-DRIFT` line **and then printed
//!    `CLEAN` and exited 0** — a check that could not fail, which is worse than
//!    no check, because it signs off on the defect it just described. Fixed in
//!    the same commit as this file, and watched red on a one-space plant.
//! 3. Nothing invoked `check`. `#3679`'s standing lesson: a `work/` script no
//!    funnel runs is not enforcement. That is what this file is.
//!
//! The through-line, and it is the reason this is a test and not a paragraph:
//! **a thing being printed is not a thing being watched.** `#3689` measured the
//! same shape when an absolute-path count printed on every run drifted 16 → 18
//! inside one wave with nobody reading it, and answered it the same way — with
//! something that goes red.
//!
//! # What this is NOT
//!
//! **Not a `gate.sh` row** (`#3691`). A 22nd count-bearing row makes
//! `scripts/gate_identity_diff.sh` exit 2 and refuse to diff for every lane
//! holding a 21-row base. This is a `cargo test` target, which the merge funnel
//! already runs on the merged tree.
//!
//! **Not a judge of the port.** It grades an instrument's anchoring, nothing
//! else. The sole judge stays real `c2.dll` under wibo plus a byte-exact obj
//! compare.

use std::path::{Path, PathBuf};
use std::process::Command;

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("crates/c2-harness/../.. is the repo root")
        .to_path_buf()
}

/// Is a usable `python3` on `PATH`? Probed by running it, not by looking for a
/// file — a `python3` that is present and broken is absent for our purposes.
fn python3_is_usable() -> bool {
    Command::new("python3")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// `(exit-0?, stdout+stderr)` from `hatch.py check`.
fn run_check(root: &Path) -> (bool, String) {
    let out = Command::new("python3")
        .arg(root.join("work/w-front3/hatch.py"))
        .arg("check")
        .current_dir(root)
        .output()
        .unwrap_or_else(|e| panic!("failed to spawn python3 for hatch.py: {e}"));
    let mut s = String::from_utf8_lossy(&out.stdout).into_owned();
    s.push_str(&String::from_utf8_lossy(&out.stderr));
    (out.status.success(), s)
}

/// The needle count, compiled in so that **shrinking the table is a code change
/// somebody reviews** rather than a silent narrowing of what green covers —
/// `#3748`'s degenerate re-bless, where a domain collapsed to one cell and the
/// baseline was re-blessed in the same motion, leaving the check green over
/// nothing.
const NEEDLES: usize = 8;

#[test]
fn every_hatch_needle_still_resolves_against_the_tree() {
    let root = repo_root();
    if !python3_is_usable() {
        println!("SKIP: python3 absent — cannot run work/w-front3/hatch.py");
        return;
    }
    if !root.join("work/w-front3/hatch.py").is_file() {
        println!("SKIP: work/w-front3/hatch.py absent");
        return;
    }

    let (ok, report) = run_check(&root);

    // A DIRTY `crates/` is not this test's business. `hatch.py check` refuses
    // `PARTIAL` (some edits present, some not), which is what a tree mid-`apply`
    // looks like, and a peer lane's worktree is routinely there. Only the
    // DRIFTED verdict — a needle that does not resolve — is a defect in the
    // instrument's anchoring, which is what this file grades.
    if report.contains("hatch: PARTIAL") {
        println!("SKIP: tree is mid-apply (hatch: PARTIAL) — not a needle question");
        return;
    }

    assert!(
        !report.contains("HATCH-DRIFT"),
        "a hatch needle no longer resolves against this tree. `hatch.py apply` \
         will REFUSE and every gate run will print `HATCH-RED REFUSED` until it \
         is re-spelled — which is exactly what happened for 1,681 commits from \
         530383eca. Re-derive the needle against the current clause text; do NOT \
         retire the key unless the refusal it lifts has actually been paid \
         (`w-park`, #1920).\n\n{report}"
    );
    assert!(
        ok,
        "hatch.py check exited non-zero without naming a drift:\n{report}"
    );

    // Green must be green FOR A REASON. Without this, a `check` that stopped
    // examining anything at all would satisfy every assertion above — `#3470`,
    // a clean report over zero rows.
    let counted = format!("of {NEEDLES} edit(s) present");
    assert!(
        report.contains(&counted),
        "expected the report to account for all {NEEDLES} needles (`{counted}`). \
         If the table legitimately changed size, change NEEDLES here in the same \
         commit and say why.\n\n{report}"
    );
}
