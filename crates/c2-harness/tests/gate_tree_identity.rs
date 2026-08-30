//! `scripts/gate.sh --selftest`, run by something (board **#3835**, lane
//! `w-gatehash`).
//!
//! # What this file is for
//!
//! `gate.sh` has 205 self-test cases that drive its real `collect`/`decide`
//! path with fabricated lane logs, need no toolchain and no compiler, and take
//! about 35 seconds. Measured at base `1d52f8902`: **nothing ran them.** Not
//! `cargo test --workspace`, not `gate.sh` itself, not any script under
//! `scripts/`, not `scripts/gate_identity_diff.sh`. `grep -rn selftest` over
//! `crates/` and `scripts/` finds every *other* instrument's self-test wired
//! into something and this one wired into nothing — it ran when a human typed
//! it, which on this project means it ran when somebody was already suspicious.
//!
//! That is the same defect the cases themselves are about, one level up: a
//! check nobody runs and a check nobody reads are the same check.
//!
//! # And the specific hole it closes
//!
//! `w-gatefix` (**#2943**) made every run print the identity of the tree it
//! graded — a content hash over `crates/ fixtures/ scripts/` — at both ends,
//! and compared them. `w-globset` then ran a base gate in the worktree it was
//! authoring in, got **808 files at the start and 810 at the end**, and filed
//! **#3835** as *"gate.sh prints its tree hash twice and nothing compares
//! them"*.
//!
//! The comparison was there the whole time and it exited 1. What it could not
//! do was change the `GATE:` verdict line, because it ran in the epilogue,
//! after `decide` had printed `GATE: PASS`. Everything that reads a gate run
//! reads that line: every dispatch brief says *"read the `GATE:` verdict LINE,
//! never the exit code"* (because `REFUSED` exits 0), and
//! `gate_identity_diff.sh` says in its own header that it reads **neither**
//! `GATE:` nor any status — it cuts 21 count-bearing rows that are printed
//! before the epilogue exists. A check that fires, exits nonzero, and is still
//! reported as absent is a check that is not being delivered.
//!
//! `w-gatehash` reproduced it end to end before changing a line — a full
//! `--jobs 16 --require-graded` run with one untracked file created 45 s in:
//! `graded tree c1eb31f530bd (810 files)`, then `GATE: PASS — 18/18 lanes ran
//! and every one of them graded a corpus`, then fourteen lines later
//! `*** THE TREE MOVED`. Transcript: `work/w-gatehash/gate_base_moved.txt`.
//!
//! # Not a `gate.sh` row (`#3691`)
//!
//! A 22nd count-bearing row makes `gate_identity_diff.sh` exit 2 and refuse to
//! diff **for every lane on a 21-row base**, so this is a `cargo test` target
//! instead — which `#1406` names as the other admissible home for anything
//! whose output is quoted as evidence. Nothing here prints a
//! `<name> PASS <n> <n>` line, so the extractor's row count is untouched.
//!
//! # Cost
//!
//! Two `--selftest` runs, ~35 s each, and they are the two that matter: one
//! that the suite is green, one that it can go red. Both are wall-clock, not
//! CPU — the selftest is mostly `sleep`s and `sha256sum`s over scratch trees.

use std::path::{Path, PathBuf};
use std::process::Command;

/// The repo root, from this crate's manifest dir (`crates/c2-harness`).
///
/// `CARGO_MANIFEST_DIR` and not `current_dir()`: cargo runs integration tests
/// with an unspecified working directory, and the harness's own tests have been
/// bitten by that before.
fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..").canonicalize().expect("repo root")
}

/// Run a gate script in `--selftest` mode and return `(ok, stdout+stderr)`.
///
/// `--work` points at a private directory under `target/` so two of these can
/// never collide, and so a `cargo test` run leaves nothing in `/tmp` that the
/// gate's own reaper is not responsible for.
fn selftest(script: &Path, work: &Path) -> (bool, String) {
    let out = Command::new("sh")
        .arg(script)
        .arg("--selftest")
        .arg("--work")
        .arg(work)
        .current_dir(repo_root())
        .output()
        .unwrap_or_else(|e| panic!("could not run {}: {e}", script.display()));
    let mut text = String::from_utf8_lossy(&out.stdout).into_owned();
    text.push_str(&String::from_utf8_lossy(&out.stderr));
    (out.status.success(), text)
}

/// The `--selftest` summary line's case count, e.g. `205`.
fn case_count(text: &str) -> Option<u32> {
    let line = text.lines().find(|l| l.contains("--selftest: PASS —"))?;
    let rest = line.split("PASS —").nth(1)?.trim_start();
    rest.split_whitespace().next()?.parse().ok()
}

/// **`gate.sh --selftest` is green, and it is a suite rather than a slogan.**
///
/// The floor is asserted here as well as inside the script. `gate.sh` checks
/// its own count and would exit 1 on a truncated run, but a floor that only
/// lives inside the thing it measures cannot say *"the suite shrank"* to
/// anybody who is not already running it — which, until this file, was nobody.
#[test]
fn gate_selftest_is_green_and_has_not_shrunk() {
    let root = repo_root();
    let script = root.join("scripts").join("gate.sh");
    if !script.is_file() {
        eprintln!("SKIP: scripts/gate.sh absent");
        return;
    }
    let work = root.join("target").join("gate-selftest-green");
    let (ok, text) = selftest(&script, &work);
    assert!(
        ok,
        "scripts/gate.sh --selftest did not pass.\n\
         Read the FAIL lines below, never the exit code alone.\n\n{}",
        tail(&text, 60)
    );
    assert!(
        text.contains("--selftest: PASS —"),
        "the selftest exited 0 without printing its PASS summary — an absence \
         reading as a success, which is this project's most-repeated defect \
         and is exactly what the suite is built against.\n\n{}",
        tail(&text, 40)
    );
    let n = case_count(&text).expect("the PASS summary names a case count");
    assert!(
        n >= 205,
        "gate.sh --selftest ran {n} cases against a floor of 205. Cases are \
         added, never removed: if a case was genuinely retired, lower the floor \
         in scripts/gate.sh AND here in the same commit, so the decision is in \
         the diff rather than in a number that quietly fell."
    );
    eprintln!("gate.sh --selftest: PASS, {n} cases");
}

/// **The tree-moved check can go red — watched, not assumed (`#1236`).**
///
/// Builds a mutant `gate.sh` with the `GATE: FAIL (TREE MOVED UNDER THIS RUN)`
/// block deleted from `decide` and **nothing else touched**, so the mutant is
/// precisely the pre-fix gate: the epilogue comparison is still there, it still
/// sets exit 1, and the headline still says `PASS`. The suite must reject it.
///
/// The mutant is written under `target/`, which is gitignored *and* outside
/// `GRADED_DIRS` — so this test cannot move the graded tree of a `gate.sh` run
/// happening in the same worktree, which would be a fine joke and a real
/// problem. `gate.sh` derives `repo_root` as `dirname $0/..`, so a copy one
/// level down from the root still resolves the real repository.
///
/// Three properties are asserted, and the third is the one that makes this more
/// than a tautology: the **controls stay green** under the mutation. A check
/// that reddened everything would satisfy the first two.
#[test]
fn deleting_the_tree_moved_check_reddens_the_selftest() {
    let root = repo_root();
    let script = root.join("scripts").join("gate.sh");
    if !script.is_file() {
        eprintln!("SKIP: scripts/gate.sh absent");
        return;
    }
    let src = std::fs::read_to_string(&script).expect("read gate.sh");

    const OPEN: &str = "    if [ \"${gate_tree_moved:-0}\" -eq 1 ]; then";
    const EMIT: &str = "        echo \"GATE: FAIL (TREE MOVED UNDER THIS RUN)";
    assert!(
        src.contains(OPEN) && src.contains(EMIT),
        "gate.sh no longer contains decide()'s tree-moved block. Either it was \
         removed — in which case a moved tree is silently green again and this \
         test is the only thing saying so — or it was reindented, in which case \
         update the anchors here. Do not delete the test."
    );

    // Delete exactly the block: from its `if` to the first `fi` at the same
    // indent. Line-based, so a mutation that swallowed the epilogue comparison
    // is detectable rather than silent.
    let mut mutant = String::with_capacity(src.len());
    let mut skipping = false;
    let mut cut = 0usize;
    for line in src.lines() {
        if !skipping && line == OPEN {
            skipping = true;
        }
        if skipping {
            cut += 1;
            if line == "    fi" {
                skipping = false;
            }
            continue;
        }
        mutant.push_str(line);
        mutant.push('\n');
    }
    assert!(cut > 5, "the mutation removed only {cut} lines — it did not apply");
    assert!(
        !mutant.contains(EMIT),
        "decide() still prints the tree-moved headline after the mutation"
    );
    assert!(
        mutant.contains("*** THE TREE MOVED UNDER THIS RUN"),
        "the mutation also ate the EPILOGUE comparison, so the mutant is not \
         the pre-fix gate and this demonstration would prove nothing"
    );

    let dir = root.join("target");
    std::fs::create_dir_all(&dir).expect("target/");
    let mpath = dir.join("gate_tree_identity_mutant.sh");
    std::fs::write(&mpath, &mutant).expect("write mutant");

    let work = dir.join("gate-selftest-mutant");
    let (ok, text) = selftest(&mpath, &work);
    let _ = std::fs::remove_file(&mpath);

    assert!(
        !ok,
        "THE MUTANT PASSED. gate.sh --selftest is green with decide()'s \
         tree-moved check deleted, so the cases that are supposed to be \
         guarding it are asserting nothing — #3787's shape exactly, where a \
         checker printed the defect, printed CLEAN and exited 0.\n\n{}",
        tail(&text, 40)
    );
    for case in [
        "tree-moved-turns-pass-red",
        "tree-moved-turns-skipped-red",
        "tree-moved-turns-sampled-red",
    ] {
        assert!(
            text.lines().any(|l| l.starts_with("  FAIL") && l.contains(case)),
            "case `{case}` did not go red on the mutant — it is not testing the \
             deleted check.\n\n{}",
            tail(&text, 40)
        );
    }
    // AND THE CONTROLS HELD. Without this, a check that reddened every case
    // would pass the three assertions above.
    for control in ["tree-still-keeps-pass", "tree-still-keeps-skipped"] {
        assert!(
            text.lines().any(|l| l.starts_with("  ok") && l.contains(control)),
            "control `{control}` also went red on the mutant. The cases are not \
             discriminating a moved tree from an unmoved one; they are just \
             failing.\n\n{}",
            tail(&text, 40)
        );
    }
    let reds = text.lines().filter(|l| l.starts_with("  FAIL")).count();
    eprintln!("mutant gate.sh --selftest: FAIL, {reds} red checks, both controls green");
}

fn tail(s: &str, n: usize) -> String {
    let lines: Vec<&str> = s.lines().collect();
    lines[lines.len().saturating_sub(n)..].join("\n")
}
