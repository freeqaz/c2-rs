//! The reap guard, wired so its exit status gates something. Board **#3573**,
//! closing **#3552**.
//!
//! `scripts/wt_pin_audit.sh` answers "would reaping this worktree destroy a
//! measurement artifact that exists nowhere else". This file is what makes a
//! red answer *cost* something — `#3552` is the third occurrence of a failure
//! the record already carried twice, and both earlier records were prose.
//!
//! **Four tests, and the last two are the load-bearing ones.**
//!
//! 1. The estate is clean or armed.
//! 2. The script's own `--self-test` still drives every class red — a guard
//!    that silently loses the ability to detect a class passes every run
//!    (`#1236`, this repo's canonical instance: a NUL check that could not
//!    fire was quoted as clean 20+ times).
//! 3. **`scripts/wt_reap.py`'s veto is watched firing**, on a planted tree, in
//!    a throwaway repo. Nothing on the live estate exercises it: the two trees
//!    that hold pins are `git worktree lock`ed, and the lock short-circuits the
//!    classification before the pin scan runs. A veto whose green has never
//!    been contrasted with a red is not evidence.
//! 4. **The two implementations are compared.** `wt_pin_audit.sh` and
//!    `wt_reap.py` each implement the P1/P2 predicate — the shell one so the
//!    audit needs nothing but `git`, the Python one so the reaper keeps its
//!    "git and nothing else" property. Two implementations of one predicate
//!    that nobody compares are two predicates, and the day they disagree is
//!    the day the reaper walks past something the audit calls pinned.
//!
//! Needs `git`; test 3 and 4 additionally need `python3` and SKIP with a
//! printed reason if it is absent. No toolchain, no build, no network.

use std::path::{Path, PathBuf};
use std::process::Command;

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn git_is_usable(root: &Path) -> bool {
    Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["rev-parse", "--git-dir"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn have_python() -> bool {
    Command::new("python3")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn run_audit(args: &[&str]) -> (bool, String) {
    let root = repo_root();
    let out = Command::new("sh")
        .arg(root.join("scripts/wt_pin_audit.sh"))
        .args(args)
        .current_dir(&root)
        .output()
        .expect("failed to run scripts/wt_pin_audit.sh");
    let mut s = String::from_utf8_lossy(&out.stdout).into_owned();
    s.push_str(&String::from_utf8_lossy(&out.stderr));
    (out.status.success(), s)
}

/// No worktree holds a pinned measurement artifact without `git worktree lock`
/// armed on it.
///
/// A red here is not a style complaint: it means a `git worktree remove
/// --force` typed by anybody would succeed and take the artifact with it, and
/// that has happened three times (`w-adjacency` §7.6, `w-hygiene` §2.1,
/// `#3552`). The remedy is one command and the assertion names it.
#[test]
fn no_worktree_holds_an_unlocked_pinned_artifact() {
    let root = repo_root();
    if !git_is_usable(&root) {
        println!("SKIP: no git checkout at {} — nothing to audit", root.display());
        return;
    }
    let (ok, report) = run_audit(&[]);
    assert!(
        ok,
        "reap guard FAILED — a worktree holds a pinned artifact and is NOT locked.\n\
         Remedy: scripts/wt_pin_audit.sh --lock\n{report}"
    );

    // The denominator, asserted rather than trusted (`#3470`, `#1002`): a clean
    // estate report over zero worktrees is not a clean report, and the script's
    // own exit-2 path is what is being checked from the outside here.
    assert!(
        report.contains("worktrees examined: ") && !report.contains("worktrees examined: 0"),
        "the audit must print a NONZERO worktree denominator; got:\n{report}"
    );
}

/// **The guard's own controls, gated.**
///
/// Includes the one case that is not a plant at all: a real `git worktree
/// remove --force` run against a locked throwaway tree, required to be
/// REFUSED. Every other case is about *finding* a pin; that one is the only
/// evidence that finding it accomplishes anything, and it is a claim about the
/// installed `git` rather than about this repo — so it is re-checked on every
/// run rather than recorded once in a comment.
#[test]
fn every_class_the_reap_guard_claims_is_watched_going_red() {
    let root = repo_root();
    if !git_is_usable(&root) {
        println!("SKIP: no git checkout at {} — --self-test needs git init", root.display());
        return;
    }
    let (ok, report) = run_audit(&["--self-test"]);
    assert!(ok, "reap-guard SELF-TEST failed:\n{report}");
    assert!(
        report.contains("SELF-TEST PASS"),
        "self-test exited 0 without reporting a pass — read the output rather \
         than the status:\n{report}"
    );
    // Named individually, because "SELF-TEST PASS" is one string and the value
    // of the run is which cases produced it.
    for needle in [
        "plant P1 unique cost-arm binary",
        "plant P1b binary rebuilt in place",
        "plant P2 .c2rs-pin manifest",
        "-> REFUSED",
    ] {
        assert!(
            report.contains(needle),
            "the self-test no longer exercises `{needle}`:\n{report}"
        );
    }
}

/// A throwaway repo with one linked worktree holding one unique ELF binary.
/// Returns the temp dir root; caller removes it.
fn plant_repo(tag: &str) -> Option<PathBuf> {
    let base = std::env::temp_dir().join(format!(
        "c2rs-reapveto-{tag}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .ok()?
            .as_nanos()
    ));
    let main = base.join("main");
    std::fs::create_dir_all(&main).ok()?;
    let g = |args: &[&str]| {
        Command::new("git")
            .arg("-C")
            .arg(&main)
            .args(args)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    };
    if !Command::new("git")
        .args(["init", "-q"])
        .arg(&main)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
    {
        return None;
    }
    g(&["config", "user.email", "a@b.c"]);
    g(&["config", "user.name", "t"]);
    std::fs::write(main.join("keep.txt"), b"ok\n").ok()?;
    g(&["add", "-A"]);
    g(&["commit", "-qm", "base"]);
    let lane = base.join("lane");
    if !g(&["worktree", "add", "-q", lane.to_str()?, "-b", "lane1"]) {
        return None;
    }
    let armdir = lane.join("work/lane");
    std::fs::create_dir_all(&armdir).ok()?;
    let arm = armdir.join("c2rs-b1");
    // Assembled rather than written as a literal: this file lives under
    // `crates/`, and a raw `\x7fELF` string in a tracked source file is the
    // kind of thing a future artifact rule notices. `#3545`'s guard flagged
    // ITSELF one commit after going green for the mirror-image reason.
    let mut magic = vec![0x7fu8];
    magic.extend_from_slice(b"ELF");
    magic.extend_from_slice(b"-planted-cost-arm\n");
    std::fs::write(&arm, &magic).ok()?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&arm, std::fs::Permissions::from_mode(0o755)).ok()?;
    }
    Some(base)
}

/// Dry-run the reaper over `repo` under the flags that make the planted tree
/// genuinely reapable, so that "kept" means the veto and nothing else.
///
/// Both flags are load-bearing and both were added after the first version of
/// this test passed on the wrong evidence:
///
/// * `--active-hours 0` — a worktree created seconds ago has a seconds-old
///   reflog, so the default 12 h window classifies it ACTIVE and it is kept
///   for a reason unrelated to the pin.
/// * `--reap-empty-lanes` — the planted tree has no commits, so it is
///   EMPTY-LANE, which is kept by default too. Without this the "clean" arm
///   printed `would reap: 0` and the contrast was vacuous.
///
/// Neither flag can destroy anything here: there is no `--apply`.
fn reap_dryrun(root: &Path, repo: &Path) -> String {
    let out = Command::new("python3")
        .arg(root.join("scripts/wt_reap.py"))
        .arg("--repo")
        .arg(repo)
        .arg("--active-hours")
        .arg("0")
        .arg("--reap-empty-lanes")
        .output()
        .expect("failed to run scripts/wt_reap.py");
    let mut s = String::from_utf8_lossy(&out.stdout).into_owned();
    s.push_str(&String::from_utf8_lossy(&out.stderr));
    s
}

/// **`wt_reap.py`'s veto, watched firing.**
///
/// `--active-hours 0` is required: a freshly created worktree's reflog is
/// seconds old, so the default 12 h window classifies it ACTIVE and the tree
/// is kept for a reason that has nothing to do with the pin. Without it this
/// test passes on the wrong evidence — which is the failure it exists to
/// exclude.
#[test]
fn the_reaper_vetoes_a_tree_holding_a_unique_binary() {
    let root = repo_root();
    if !git_is_usable(&root) {
        println!("SKIP: no git checkout — cannot run the reaper");
        return;
    }
    if !have_python() {
        println!("SKIP: python3 absent — scripts/wt_reap.py cannot run");
        return;
    }
    let base = match plant_repo("veto") {
        Some(b) => b,
        None => {
            println!("SKIP: could not build the throwaway repo (git worktree add failed)");
            return;
        }
    };
    let repo = base.join("main");
    let planted = reap_dryrun(&root, &repo);

    // Remove the plant and re-read: the veto must be a property of the plant,
    // not of the throwaway repo's shape.
    std::fs::remove_file(base.join("lane/work/lane/c2rs-b1")).expect("remove plant");
    let clean = reap_dryrun(&root, &repo);

    let _ = std::fs::remove_dir_all(&base);

    assert!(
        planted.contains("pinned-artifact vetoes: 1") && planted.contains("KEPT: PINNED"),
        "the reaper did NOT veto a tree holding a unique binary:\n{planted}"
    );
    // The veto must OVERRIDE a reap the flags would otherwise authorise —
    // `--reap-empty-lanes` says take this tree, and the pin says do not.
    assert!(
        planted.contains("would have been EMPTY-LANE") && planted.contains("would reap: 0"),
        "the veto did not override an authorised reap; it may just be reporting \
         a tree that was being kept anyway:\n{planted}"
    );
    assert!(
        clean.contains("pinned-artifact vetoes: 0"),
        "the veto did not clear when the plant was removed — it is not keyed on \
         the plant:\n{clean}"
    );
    assert!(
        clean.contains("would reap: 1"),
        "with the plant gone the tree should be reapable; if it is not, the \
         `planted` run above proved nothing about the pin:\n{clean}"
    );
}

/// **The two implementations of the P1/P2 predicate must agree.**
///
/// Same planted tree, both tools, both required to call it pinned; then the
/// plant is removed and both are required to call it clean. Four readings, not
/// two, because agreement on a positive alone is satisfied by two detectors
/// that always say yes.
#[test]
fn the_shell_audit_and_the_python_reaper_agree_on_what_is_pinned() {
    let root = repo_root();
    if !git_is_usable(&root) {
        println!("SKIP: no git checkout");
        return;
    }
    if !have_python() {
        println!("SKIP: python3 absent — one of the two implementations cannot run");
        return;
    }
    let base = match plant_repo("agree") {
        Some(b) => b,
        None => {
            println!("SKIP: could not build the throwaway repo");
            return;
        }
    };
    let repo = base.join("main");

    let audit_of = |repo: &Path| -> (bool, String) {
        let out = Command::new("sh")
            .arg(root.join("scripts/wt_pin_audit.sh"))
            .env("C2RS_PIN_AUDIT_ROOT", repo)
            .current_dir(repo)
            .output()
            .expect("failed to run scripts/wt_pin_audit.sh");
        let mut s = String::from_utf8_lossy(&out.stdout).into_owned();
        s.push_str(&String::from_utf8_lossy(&out.stderr));
        (out.status.success(), s)
    };

    let (audit_ok_planted, audit_planted) = audit_of(&repo);
    let reap_planted = reap_dryrun(&root, &repo);

    std::fs::remove_file(base.join("lane/work/lane/c2rs-b1")).expect("remove plant");
    let (audit_ok_clean, audit_clean) = audit_of(&repo);
    let reap_clean = reap_dryrun(&root, &repo);

    let _ = std::fs::remove_dir_all(&base);

    let reap_says_pinned = |s: &str| s.contains("pinned-artifact vetoes: 1");
    assert!(
        !audit_ok_planted && reap_says_pinned(&reap_planted),
        "the two implementations DISAGREE on the planted tree — audit red? {} \
         / reaper vetoed? {}\n--- audit ---\n{audit_planted}\n--- reaper ---\n{reap_planted}",
        !audit_ok_planted,
        reap_says_pinned(&reap_planted)
    );
    assert!(
        audit_ok_clean && !reap_says_pinned(&reap_clean),
        "the two implementations DISAGREE on the cleaned tree — audit green? {} \
         / reaper quiet? {}\n--- audit ---\n{audit_clean}\n--- reaper ---\n{reap_clean}",
        audit_ok_clean,
        !reap_says_pinned(&reap_clean)
    );
}
