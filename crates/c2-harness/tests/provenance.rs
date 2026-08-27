//! **`#1406`, discharged for the two provenance instruments.** Board **#3682**.
//!
//! `CLAUDE.md` § Hard constraints:
//!
//! > anything whose output is quoted as evidence must run under `cargo test`
//! > or `scripts/gate.sh`
//!
//! `scripts/provenance_census.py` and `scripts/prose_audit.py` both carry a
//! `--self-test` — the controls that decide whether the instrument can still
//! detect the class it exists to detect — and until this file existed **nothing
//! invoked either of them**. They ran when a human remembered to, which is
//! `#1406`'s original complaint about `hatch_red.py`, one level outside the
//! rule that forbids it. Three lanes (`w-provenance`, `w-provext`,
//! `w-provaudit`) reported it and none could fix it: all three were fenced to
//! comment-only edits under `crates/`, and a new `tests/` file is not a
//! comment. Owner decision 19 granted the fence; this is the file.
//!
//! # What is wired, and what deliberately is NOT
//!
//! **Wired: the two `--self-test` runs.** They are the *controls*. A guard that
//! has quietly lost the ability to go red passes every run, and this project has
//! shipped nine instruments that reported green from an absence.
//!
//! **Wired 2026-08-27, and NARROWLY: the two TREE runs** — `#3684`, board
//! **#3690**. This file shipped with them deliberately absent, because a tree
//! audit under `cargo test` makes **every** lane's `cargo test` depend on
//! **every doc in the tree** being count-clean, and twenty worktrees were live
//! the day it landed. That objection was right about `prose_audit.py`'s *whole*
//! verdict and wrong about the thing `#3684` actually named:
//!
//! > the `COUNT[...]` bindings in `crates/c2-core/src/codegen/mop.rs` are
//! > checked only by a tree run, so nothing automatic checks them yet.
//!
//! So the tree runs are wired and **only C4 (BOUND-COUNT) can redden a test.**
//! C1/C2/C3/C6 are printed and not asserted on.
//!
//! **Why that split is principled and not a dodge.** C4's population is
//! *opt-in by construction*: a file is graded by it only if someone wrote an
//! explicit `COUNT[<recipe>] = N` binding into it, which is a request to have
//! the arithmetic checked. A lane that writes no binding cannot be reddened by
//! another lane's doc. C1 and C2 have the opposite shape — they grade every
//! `W-*-N` token and every absence claim anywhere in `docs/`, so gating them
//! *would* couple twenty worktrees to each other's prose. The blast radius was
//! never a property of "a tree run"; it was a property of *which check*.
//!
//! **What this still does not catch, said plainly:** a binding pointed at the
//! wrong population. C4 checks that `N` recounts to `N` under its stated
//! recipe, never that the recipe names the population the prose is about — the
//! script's own `--help` says so, and `#3643` (a count inside a read-tier
//! provenance marker, wrong by 14 rows since the file's first commit) is that
//! class.
//!
//! The tier letter is spelled out rather than written as the literal token,
//! and that is not fussiness: `provenance_census.py` scans `crates/` for that
//! token and **cannot tell a mark from a mention of one**, so prose *about*
//! markers lands in the census as a marker. `#3641` is that exact move costing
//! a subsystem's agreement census 9/28 → 13/34, and `prose_audit.py`'s C5 flag
//! caught this sentence committing it while it was being written.
//!
//! # And no `gate.sh` row — the reason is mechanical
//!
//! `#1406` says `cargo test` **or** `scripts/gate.sh`, so this file discharges
//! it. A `gate.sh` row was considered and refused because
//! `scripts/gate_identity_diff.sh` selects count-bearing rows **by shape**
//! (`/^[A-Za-z][A-Za-z0-9-]* +(PASS|FAIL|REFUSED|SKIP|NO-RESULT) /`) and excludes
//! **by hard-coded name** (`grep -Ev '^(hatch-red|ladder-red) '`). A new row
//! named anything else is counted *even with `n/a` in the mismatch column*,
//! `WANT_ROWS=21` is violated, and the script **exits 2 refusing to diff at
//! all** — for every live lane holding a 21-row base table, on a tree they did
//! not touch. Demonstrated, not asserted: `work/w-wire/gatediff_22row.out`.
//!
//! # Two rules this file follows because getting them wrong has cost this repo
//!
//! 1. **The repo root is resolved at RUNTIME.** Never
//!    `env!("CARGO_MANIFEST_DIR")`. Board `#3525`: that literal sets `.rodata`
//!    size, so three builds of one commit in directories named `b1`, `b2xx` and
//!    `b3yyyyyy` produced three differently-sized binaries. Board `#3470`: a
//!    binary built in a scratch tree resolves everything relative to *that*
//!    tree. `crates/c2-reference`'s `repo_root()` was fixed the same way and
//!    this follows it — but **not** its "prefer the ancestor that carries what
//!    I need" refinement, which is wrong here and was *measured* to be wrong;
//!    see [`repo_root`]'s own note. There is a second, sharper reason the
//!    resolution matters at all: **both scripts derive the tree they audit from
//!    their own `__file__`**, so the path this test resolves decides which tree
//!    gets audited. A baked path would audit the tree the binary was compiled
//!    in, silently, forever.
//! 2. **Absence of `python3` is a SKIP, never a failure**, per `CLAUDE.md`'s
//!    degrade-cleanly rule. Absence of the *script itself* is a **failure**, and
//!    says so in those words — `#1496`'s `LADDER-NOSUBJECT` lesson: without that
//!    split, a deleted instrument reports "the guards stopped working" when the
//!    truth is "the instrument is gone".
//!
//! Needs `python3`. Needs no toolchain, no build of the port, and no network.

use std::path::{Path, PathBuf};
use std::process::Command;

/// Does `dir` look like a c2-rs checkout root? `Cargo.toml` + `crates/`.
fn is_repo_marker(dir: &Path) -> bool {
    dir.join("Cargo.toml").is_file() && dir.join("crates").is_dir()
}

/// The **nearest** ancestor of `start` (inclusive) that is a checkout root.
fn nearest_marker(start: &Path) -> Option<PathBuf> {
    let mut cur = Some(start);
    while let Some(dir) = cur {
        if is_repo_marker(dir) {
            return Some(dir.to_path_buf());
        }
        cur = dir.parent();
    }
    None
}

/// Repo root — **resolved at RUNTIME**, and deliberately **NEAREST-MARKER
/// ONLY**, which is where this diverges from `c2-reference::repo_root`.
///
/// `C2RS_REPO_ROOT` wins verbatim and is not second-guessed — the same contract
/// `c2-reference::repo_root` gives it. Otherwise: walk up from
/// `current_exe()`, then from `current_dir()`, and take the first
/// `Cargo.toml` + `crates/` marker found. `None` — the only skip — means
/// neither the running test binary nor the cwd sits inside a checkout at all.
///
/// # Why NOT `c2-reference`'s "prefer the ancestor that carries what I need"
///
/// `c2-reference::repo_root` prefers the nearest ancestor carrying
/// `compilers/`, because its fallback when the toolchain is missing is a
/// **silent** `SKIP: toolchain absent` + exit 0 — `#3470`, a scan pair that
/// graded nothing on one arm. Preferring is how that silence is avoided there.
///
/// Here the fallback is the opposite: a missing script is a **loud assertion
/// failure** (see [`assert_self_test_passes`]), so preferring buys nothing —
/// and it actively costs, because **every lane worktree is nested inside the
/// main repo, which also carries `scripts/`**. This was written the preferring
/// way first and measured: with `scripts/prose_audit.py` moved out of the
/// worktree, the preferring version walked up past it, found the main repo's
/// copy, and printed `PASS over 48 checks` — a green verdict about a **tree the
/// test was not testing**. Nearest-marker-only turns that into the assertion it
/// should always have been.
fn repo_root() -> Option<PathBuf> {
    if let Some(v) = std::env::var_os("C2RS_REPO_ROOT") {
        return Some(PathBuf::from(v));
    }
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(Path::to_path_buf));
    let cwd = std::env::current_dir().ok();

    [exe_dir, cwd]
        .into_iter()
        .flatten()
        .find_map(|start| nearest_marker(&start))
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

/// Run `python3 scripts/<script> --self-test` and return
/// `(exit-0?, combined stdout+stderr)`.
fn run_self_test(root: &Path, script: &str) -> (bool, String) {
    let out = Command::new("python3")
        .arg(root.join("scripts").join(script))
        .arg("--self-test")
        .current_dir(root)
        .output()
        .unwrap_or_else(|e| panic!("failed to spawn python3 for {script}: {e}"));
    let mut s = String::from_utf8_lossy(&out.stdout).into_owned();
    s.push_str(&String::from_utf8_lossy(&out.stderr));
    (out.status.success(), s)
}

/// The whole body of both tests. Named per script by the caller so a red run
/// says **which instrument** broke — a test that reddens identically for two
/// subjects identifies neither.
fn assert_self_test_passes(script: &str) {
    if !python3_is_usable() {
        println!("SKIP: python3 absent — cannot run scripts/{script} --self-test");
        return;
    }
    let Some(root) = repo_root() else {
        println!("SKIP: no c2-rs checkout above this test binary or the cwd");
        return;
    };
    let path = root.join("scripts").join(script);

    // A MISSING INSTRUMENT IS A FAILURE, NOT A SKIP (`#1496`). If this were a
    // skip, deleting the script would make the guard silently stop guarding.
    assert!(
        path.is_file(),
        "scripts/{script} DOES NOT EXIST at {} — the instrument is gone. This \
         is not a portability skip: `#1406` requires this script's self-test to \
         run under `cargo test`, and it cannot run if it is not there.",
        path.display()
    );

    let (ok, report) = run_self_test(&root, script);

    // Read the VERDICT LINE, never the exit code alone. `gate.sh` prints
    // `GATE: REFUSED` and exits 0; a status is not evidence in this repo.
    assert!(
        ok && report.contains("SELF-TEST: PASS"),
        "scripts/{script} --self-test FAILED (exit-0 = {ok}). Full report:\n{report}"
    );

    // THE DENOMINATOR (`#3470`, `#1002`). A self-test that executed zero checks
    // prints `SELF-TEST: PASS` and exits 0, and is indistinguishable from one
    // that executed all of them. Only a count can tell them apart.
    //
    // Deliberately NOT a fixed floor: peers add sections to these scripts and a
    // pinned number would redden on growth, which is a false alarm on the one
    // surface that must not cry wolf. Nonzero-and-no-failures is maintenance
    // free and still catches the empty run.
    let passes = report.lines().filter(|l| l.trim_start().starts_with("PASS ")).count();
    let fails = report.lines().filter(|l| l.trim_start().starts_with("FAIL ")).count();
    assert!(
        passes > 0,
        "scripts/{script} --self-test reported PASS over ZERO checks — a green \
         run from an absence is not a green run. Full report:\n{report}"
    );
    assert_eq!(
        fails, 0,
        "scripts/{script} --self-test printed {fails} FAIL line(s) and still \
         summarised PASS. Full report:\n{report}"
    );
    println!("scripts/{script} --self-test: PASS over {passes} checks");
}

/// `scripts/provenance_census.py` — the `PROV[X]` marker census whose totals are
/// quoted in `DISCLOSURE.md` § "The in-code provenance markers".
#[test]
fn provenance_census_self_test_passes() {
    assert_self_test_passes("provenance_census.py");
}

/// `scripts/prose_audit.py` — the six-check prose-truth audit (`#3667`) whose
/// `VERDICT:` line is quoted as evidence, and whose C4 check is the only thing
/// that grades the `COUNT[...]` bindings in `crates/`.
#[test]
fn prose_audit_self_test_passes() {
    assert_self_test_passes("prose_audit.py");
}

// ---- the TREE runs (#3684 / #3690) -----------------------------------------

/// Run `python3 scripts/<script>` over the tree — no `--self-test`.
fn run_tree(root: &Path, script: &str) -> (bool, String) {
    let out = Command::new("python3")
        .arg(root.join("scripts").join(script))
        .current_dir(root)
        .output()
        .unwrap_or_else(|e| panic!("failed to spawn python3 for {script}: {e}"));
    let mut s = String::from_utf8_lossy(&out.stdout).into_owned();
    s.push_str(&String::from_utf8_lossy(&out.stderr));
    (out.status.success(), s)
}

/// The preamble every tree test shares: skip on no python3 / no checkout, but
/// **fail** on a missing script (`#1496`).
fn tree_root_or_skip(script: &str) -> Option<PathBuf> {
    if !python3_is_usable() {
        println!("SKIP: python3 absent — cannot run scripts/{script} over the tree");
        return None;
    }
    let root = repo_root()?;
    assert!(
        root.join("scripts").join(script).is_file(),
        "scripts/{script} DOES NOT EXIST under {} — the instrument is gone, \
         which is a failure and not a portability skip.",
        root.display()
    );
    Some(root)
}

/// Parse one row of `prose_audit.py`'s `CHECKS THAT CAN GO RED` table:
/// `C4   BOUND-COUNT           3         0` -> `(checked, findings)`.
///
/// Returns `None` when the row is **absent**, and every caller treats that as a
/// failure rather than as zero findings. That is the whole reason this returns
/// an `Option` instead of defaulting: a script that crashed, printed a usage
/// error, or had its table renamed produces no row at all, and "no row" and
/// "no findings" must not look alike. This repo has ~15 recorded instances of
/// absence read as success and it is the single most repeated defect in it.
fn check_row(report: &str, id: &str) -> Option<(usize, usize)> {
    report.lines().find_map(|l| {
        let f: Vec<&str> = l.split_whitespace().collect();
        // id, shape, checked, findings — and the shape column is one token, so
        // a 4-field split is exact rather than a prefix guess.
        match f.as_slice() {
            [a, _, c, d] if *a == id => Some((c.parse().ok()?, d.parse().ok()?)),
            _ => None,
        }
    })
}

/// **The one check that can redden here: C4 BOUND-COUNT.**
///
/// Every `COUNT[<recipe>] = N` in the tree is recounted under its own recipe and
/// must still equal `N`. `#3683` bound two of them in
/// `crates/c2-core/src/codegen/mop.rs`; before this test, editing the
/// `OPCODE_ROWS` literal without touching the comment beside it told nobody.
///
/// The other four red-capable checks are printed and **not** asserted on — see
/// this module's header for why the blast-radius objection applies to them and
/// not to C4.
#[test]
fn prose_audit_tree_run_finds_no_false_count_binding() {
    let Some(root) = tree_root_or_skip("prose_audit.py") else { return };
    let (exit_ok, report) = run_tree(&root, "prose_audit.py");

    let Some((checked, findings)) = check_row(&report, "C4") else {
        panic!(
            "prose_audit.py printed no `C4` row at all (exit-0 = {exit_ok}). \
             A missing row is not zero findings. Full report:\n{report}"
        );
    };

    // THE DENOMINATOR FIRST (`#3470`, `#1002`). C4 over zero bindings is green
    // and says nothing; it is also the exact state this repo would drift back
    // into if the `COUNT[...]` markers were renamed, which is `#3641`'s shape.
    assert!(
        checked > 0,
        "C4 BOUND-COUNT graded ZERO bindings. Either every `COUNT[...]` marker \
         has been removed or the recipe vocabulary stopped matching them — \
         both are silent, and both make this test decorative. Full report:\n{report}"
    );
    assert_eq!(
        findings, 0,
        "C4 BOUND-COUNT found {findings} binding(s) whose stated number does \
         not recount. Full report:\n{report}"
    );

    for id in ["C1", "C2", "C3", "C6"] {
        match check_row(&report, id) {
            Some((n, f)) => println!("  advisory {id}: {n} checked, {f} finding(s) — NOT gated here"),
            None => println!("  advisory {id}: row absent"),
        }
    }
    println!("C4 BOUND-COUNT: {checked} binding(s) recounted, 0 disagreements");
}

/// `scripts/provenance_census.py` over the tree.
///
/// This one grades no claim — it is a **census**, and its numerator is expected
/// to move every wave — so there is no coverage floor to assert and pinning one
/// would redden on ordinary growth. What it does establish is that the census
/// can still *reach* the tree: a nonzero denominator plus a clean exit. Board
/// `#3649` is why that is worth a test at all — a glob inside a `///` doc
/// comment froze this script's brace-depth tracking to EOF and silently
/// disabled `#[cfg(test)]` exclusion in 18 files, moving the published
/// denominator from 989 to 1,051 while the numerator never budged.
#[test]
fn provenance_census_tree_run_reaches_the_tree() {
    let Some(root) = tree_root_or_skip("provenance_census.py") else { return };
    let (ok, report) = run_tree(&root, "provenance_census.py");
    assert!(ok, "provenance_census.py over the tree FAILED:\n{report}");

    let denom = report
        .lines()
        .find_map(|l| l.strip_prefix("denominator: "))
        .and_then(|r| r.split_whitespace().next())
        .and_then(|n| n.parse::<usize>().ok());
    let Some(denom) = denom else {
        panic!("provenance_census.py printed no `denominator: ` line:\n{report}");
    };
    assert!(
        denom > 0,
        "the census enumerated ZERO const/static items. A clean census over \
         nothing is not a census (`#3470`). Full report:\n{report}"
    );
    println!("provenance census: {denom} const/static items enumerated");
}
