#!/usr/bin/env python3
"""Reap worktrees whose work is already on the base branch.

Lanes land with `merge --no-ff` and their worktrees linger; at 390 trees the
estate holds ~350 that are pure clutter (branch fully merged, tree clean).
This walks `git worktree list`, classifies every non-primary tree, and — only
under `--apply` — removes what is provably loss-free to remove:

  MERGED    branch is an ancestor of the base, tree clean
            -> `git worktree remove` + `git branch -d`
  DETACHED  HEAD is an ancestor of the base, tree clean
            -> `git worktree remove`
  DETACHED  HEAD is NOT an ancestor (an unlanded tip with no branch)
            -> mint `rescue/<name>` at the tip first, then remove if clean;
               with --rescue-dirty, commit the dirty state onto the rescue
               branch inside the tree, then remove. Either way the tip and
               the tree's last state survive as refs.
  UNLANDED  branch not merged: the BRANCH is never touched. The worktree is
            removed only with --reap-unlanded and only if clean (commits live
            in refs; a clean tree holds nothing else).
  EMPTY-LANE  branch tip sits on the base's first-parent line: a lane with no
            commits yet. Reads merged+clean, but may be a live session's
            surface that has not committed — kept unless --reap-empty-lanes.

Everything else is reported, never acted on: trees whose reflog moved within
--active-hours (somebody's live surface — rebasing or removing one is how
work gets lost), locked trees, PINNED trees, dirty trees.

  PINNED    the tree holds a measurement artifact that exists nowhere else
            (board #3552, #3573). Never removed, whatever else it classifies
            as, and never `--force`d away by this script. See
            `scripts/wt_pin_audit.sh`, which owns the detector and can arm
            git's own refusal with `--lock`.

Note what the PINNED class is and is NOT. It is a second line, not the fence:
all three recorded losses came from a HAND-TYPED `git worktree remove --force`
and this script was not involved in any of them, so a check that lived only
here could not have fired (#1236 — a guard that passes exactly when it
matters). The fence is `git worktree lock`, measured on git 2.55.0 to refuse
a plain `--force` and to print the pin reason while doing it. This class
exists so that the one reaper that IS automated cannot walk past a pin that
nobody got round to locking.

Dry-run is the default and every class is counted — a reaper that ran
silently is indistinguishable from one that did not. No removal ever passes
--force: git's own refusal on a dirty tree is the backstop under this
script's classification, so a classification bug degrades to a report line,
not a loss.

One narrow dirt exemption: a tree whose only status line is an untracked
`compilers` symlink is treated as clean (old trees predate the .gitignore
entry; a symlink does not match a `compilers/` dir pattern). The symlink is
unlinked before removal.
"""

import argparse
import os
import subprocess
import sys
import time


def git(args, cwd=None, check=True):
    r = subprocess.run(
        ["git"] + args, cwd=cwd, capture_output=True, text=True
    )
    if check and r.returncode != 0:
        raise RuntimeError(f"git {' '.join(args)}: {r.stderr.strip()}")
    return r


def list_worktrees(repo):
    out = git(["worktree", "list", "--porcelain"], cwd=repo).stdout
    trees, cur = [], {}
    for line in out.splitlines():
        if not line:
            if cur:
                trees.append(cur)
                cur = {}
            continue
        k, _, v = line.partition(" ")
        cur[k] = v if v else True
    if cur:
        trees.append(cur)
    return trees


def reflog_age_hours(wt_path):
    """Age of the last HEAD movement. The index is useless here — `git
    status` refreshes it, so any survey pollutes it; the reflog moves only
    on commit/checkout/reset."""
    try:
        gd = git(["rev-parse", "--git-dir"], cwd=wt_path).stdout.strip()
        for name in ("logs/HEAD", "HEAD"):
            p = os.path.join(gd, name)
            if os.path.exists(p):
                return (time.time() - os.stat(p).st_mtime) / 3600.0
    except (RuntimeError, OSError):
        pass
    return None


def status_lines(wt_path):
    r = git(["status", "--porcelain", "--no-renames"], cwd=wt_path, check=False)
    if r.returncode != 0:
        return None
    return [l for l in r.stdout.splitlines() if l.strip()]


def only_compilers_symlink(wt_path, lines):
    if len(lines) != 1 or lines[0] != "?? compilers":
        return False
    return os.path.islink(os.path.join(wt_path, "compilers"))


PRUNE_DIRS = {"target", ".git", "compilers", "node_modules", ".claude"}
PIN_FILE = ".c2rs-pin"


def _is_elf(path):
    try:
        with open(path, "rb") as fh:
            return fh.read(4) == b"\x7fELF"
    except OSError:
        return False


def pinned_artifacts(wt_path, primary):
    """Artifacts a reap of `wt_path` would DESTROY — i.e. that exist only here.

    The same two classes `scripts/wt_pin_audit.sh` defines, re-implemented
    rather than shelled out to so this script keeps its "git and nothing else"
    property. The two are cross-checked by
    `crates/c2-harness/tests/wt_pin_audit.rs`, because two implementations of
    one predicate that nobody compares are two predicates.

    P1 — an ELF binary whose same-relative-path counterpart in the primary is
         absent or a different size. The path+size test (not a hash) is what
         drops the six inherited `work/w-biquad/c2rs.base` copies that
         `setup_worktree.sh` reflinks into every tree; hashing 6 MB binaries
         per tree per run would cost seconds to separate cases that do not
         arise. Its false negative is named in wt_pin_audit.sh.
    P2 — an explicit `.c2rs-pin` manifest. Two of the three recorded losses
         were of things a rung had declared pinned IN PROSE, which no tool can
         read; this is the channel that makes the declaration machine-readable,
         and it is the only one that can cover a non-binary (a corpus
         snapshot, a gate base table, a registered-unrun experiment).
    """
    found = []
    for dirpath, dirnames, filenames in os.walk(wt_path):
        dirnames[:] = [d for d in dirnames if d not in PRUNE_DIRS]
        for name in filenames:
            p = os.path.join(dirpath, name)
            rel = os.path.relpath(p, wt_path)
            if name == PIN_FILE:
                found.append(("P2", rel))
                continue
            if os.path.islink(p) or not os.access(p, os.X_OK):
                continue
            try:
                if not os.path.isfile(p) or not _is_elf(p):
                    continue
                size = os.stat(p).st_size
            except OSError:
                continue
            twin = os.path.join(primary, rel)
            try:
                if os.path.isfile(twin) and os.stat(twin).st_size == size:
                    continue
            except OSError:
                pass
            found.append(("P1", rel))
    return found


def first_parent_line(repo, base):
    """Commits on the base branch's first-parent line. A lane that LANDED has
    its tip on the second-parent side of its merge — never here. A tip that IS
    here is an empty lane: a worktree minted off the base with no commits yet,
    which reads merged+clean and would otherwise be reaped out from under the
    session that just created it and has not committed yet."""
    out = git(["rev-list", "--first-parent", base], cwd=repo).stdout
    return set(out.split())


def classify(repo, wt, base, active_hours, fp_line, primary=None):
    path = wt["worktree"]
    if not os.path.isdir(path):
        return {"path": path, "cls": "GONE"}
    row = {"path": path, "cls": None, "branch": None, "sha": None,
           "dirty": None, "note": "", "pins": []}
    if wt.get("locked") is not None and "locked" in wt:
        row["cls"] = "LOCKED"
        return row

    # The pin scan does NOT short-circuit the classification, deliberately.
    # GAPS §7's lane-registry trap (board #1236) is that an earlier guard
    # returns first and every later assertion silently never executes — so the
    # pin is recorded on the row and the VETO is applied at the action stage,
    # where it can be seen alongside the class the tree would otherwise have
    # had. A row reading `MERGED ... KEPT: PINNED` is the useful output; a row
    # reading only `PINNED` hides which reap would have taken it.
    if primary:
        row["pins"] = pinned_artifacts(path, primary)

    age = reflog_age_hours(path)
    row["age_h"] = age
    sha = git(["rev-parse", "HEAD"], cwd=path).stdout.strip()
    row["sha"] = sha
    branch = wt.get("branch")
    if isinstance(branch, str):
        row["branch"] = branch.removeprefix("refs/heads/")

    if age is not None and age < active_hours:
        row["cls"] = "ACTIVE"
        return row

    lines = status_lines(path)
    if lines is None:
        row["cls"] = "UNREADABLE"
        return row
    if lines and only_compilers_symlink(path, lines):
        row["note"] = "compilers-symlink-only"
        lines = []
    row["dirty"] = len(lines)

    merged = git(["merge-base", "--is-ancestor", sha, base],
                 cwd=repo, check=False).returncode == 0
    if row["branch"] is None:
        row["cls"] = "DETACHED" if merged else "DETACHED-UNLANDED"
    elif merged and sha in fp_line:
        row["cls"] = "EMPTY-LANE"
    else:
        row["cls"] = "MERGED" if merged else "UNLANDED"
    return row


def remove_worktree(repo, row, apply):
    path = row["path"]
    if not apply:
        return "would remove"
    if row["note"] == "compilers-symlink-only":
        os.unlink(os.path.join(path, "compilers"))
    r = git(["worktree", "remove", path], cwd=repo, check=False)
    if r.returncode != 0:
        return f"REFUSED: {r.stderr.strip().splitlines()[0] if r.stderr else '?'}"
    return "removed"


def delete_branch(repo, branch, apply):
    if not apply:
        return "would delete branch"
    r = git(["branch", "-d", branch], cwd=repo, check=False)
    if r.returncode != 0:
        return f"branch REFUSED: {r.stderr.strip().splitlines()[0] if r.stderr else '?'}"
    return "branch deleted"


def rescue_name(path):
    return "rescue/" + os.path.basename(path.rstrip("/"))


def rescue_clean_tip(repo, row, apply):
    name = rescue_name(row["path"])
    if not apply:
        return f"would mint {name} at {row['sha'][:9]}"
    r = git(["branch", name, row["sha"]], cwd=repo, check=False)
    if r.returncode != 0 and "already exists" not in r.stderr:
        return f"rescue REFUSED: {r.stderr.strip()}"
    return f"minted {name}"


def rescue_dirty_tree(repo, row, apply):
    """Commit the tree's dirty state onto a rescue branch inside the tree.
    `add -A` is deliberate here: a rescue is a full snapshot of the state
    being destroyed, and the message says so."""
    name = rescue_name(row["path"])
    path = row["path"]
    if not apply:
        return f"would mint {name} + commit {row['dirty']} dirty paths"
    r = git(["switch", "-c", name], cwd=path, check=False)
    if r.returncode != 0:
        return f"rescue switch REFUSED: {r.stderr.strip()}"
    git(["add", "-A"], cwd=path)
    r = git(["commit", "-m",
             f"rescue: uncommitted state of {os.path.basename(path)} at reap "
             f"(was detached at {row['sha'][:9]})"], cwd=path, check=False)
    if r.returncode != 0:
        return f"rescue commit REFUSED: {r.stderr.strip()}"
    return f"minted {name} with dirty state"


def main():
    ap = argparse.ArgumentParser(
        description=__doc__.splitlines()[0],
        epilog="Dry-run by default; nothing changes without --apply.")
    ap.add_argument("--repo", default=None,
                    help="repo root (default: toplevel of cwd)")
    ap.add_argument("--base", default="master")
    ap.add_argument("--active-hours", type=float, default=12.0,
                    help="skip trees whose reflog moved within this window")
    ap.add_argument("--apply", action="store_true")
    ap.add_argument("--reap-unlanded", action="store_true",
                    help="also remove CLEAN worktrees of unmerged branches "
                         "(the branch is kept; commits live in refs)")
    ap.add_argument("--rescue-dirty", action="store_true",
                    help="for DIRTY detached-unlanded trees: commit the dirty "
                         "state onto rescue/<name>, then remove")
    ap.add_argument("--reap-empty-lanes", action="store_true",
                    help="also remove CLEAN worktrees whose branch tip sits on "
                         "the base's first-parent line (a lane with no commits "
                         "yet — possibly abandoned, possibly a session that "
                         "has not committed; hence opt-in)")
    args = ap.parse_args()

    repo = args.repo or git(["rev-parse", "--show-toplevel"]).stdout.strip()
    trees = list_worktrees(repo)
    primary = trees[0]["worktree"]
    fp_line = first_parent_line(repo, args.base)

    rows = []
    for wt in trees[1:]:
        rows.append(classify(repo, wt, args.base, args.active_hours, fp_line,
                             primary=primary))

    counts = {}
    pinned_vetoes = 0
    actions = []
    for row in rows:
        cls = row["cls"]
        counts[cls] = counts.get(cls, 0) + 1
        act = ""
        if row.get("pins"):
            # THE VETO. Board #3552/#3573: three consecutive actors destroyed
            # pinned measurement artifacts this way. It applies whatever the
            # tree classified as, and no flag on this script overrides it —
            # `--reap-unlanded` and `--reap-empty-lanes` exist to widen what is
            # reapable and neither may widen past this.
            pinned_vetoes += 1
            counts["PINNED-VETO"] = counts.get("PINNED-VETO", 0) + 1
            listing = ", ".join(f"{k}:{v}" for k, v in row["pins"][:4])
            more = "" if len(row["pins"]) <= 4 else f" (+{len(row['pins']) - 4} more)"
            act = (f"KEPT: PINNED — would have been {cls}; holds {listing}{more}. "
                   f"Run scripts/wt_pin_audit.sh --lock to arm git's own refusal.")
            actions.append((row, act))
            continue
        if cls == "MERGED" and row["dirty"] == 0:
            act = remove_worktree(repo, row, args.apply)
            if not act.startswith("REFUSED") and row["branch"]:
                act += "; " + delete_branch(repo, row["branch"], args.apply)
        elif cls == "DETACHED" and row["dirty"] == 0:
            act = remove_worktree(repo, row, args.apply)
        elif cls == "DETACHED-UNLANDED":
            if row["dirty"] == 0:
                act = rescue_clean_tip(repo, row, args.apply)
                if "REFUSED" not in act:
                    act += "; " + remove_worktree(repo, row, args.apply)
            elif args.rescue_dirty:
                act = rescue_dirty_tree(repo, row, args.apply)
                if "REFUSED" not in act:
                    act += "; " + remove_worktree(repo, row, args.apply)
            else:
                act = "kept (dirty, no --rescue-dirty)"
        elif cls == "EMPTY-LANE":
            if args.reap_empty_lanes and row["dirty"] == 0:
                act = remove_worktree(repo, row, args.apply)
                if not act.startswith("REFUSED") and row["branch"]:
                    act += "; " + delete_branch(repo, row["branch"], args.apply)
            else:
                act = "kept (no commits yet — may be a live session's surface)"
        elif cls == "UNLANDED":
            if args.reap_unlanded and row["dirty"] == 0:
                act = remove_worktree(repo, row, args.apply)
                act += "; branch kept"
            else:
                act = "kept (branch not merged)"
        elif cls in ("MERGED", "DETACHED"):
            act = f"kept (dirty: {row['dirty']})"
        elif cls == "ACTIVE":
            act = f"kept (reflog {row['age_h']:.1f}h ago)"
        else:
            act = "kept"
        actions.append((row, act))

    mode = "APPLY" if args.apply else "DRY-RUN"
    print(f"wt_reap [{mode}]  base={args.base}  primary={primary}")
    print(f"  {len(rows)} non-primary worktrees\n")
    routine = {"removed; branch deleted", "would remove; would delete branch",
               "removed", "would remove"}
    for row, act in actions:
        if act in routine:
            continue  # the common success case is summarised, not listed
        name = os.path.basename(row["path"].rstrip("/"))
        cls = row["cls"]
        d = f" dirty:{row['dirty']}" if row.get("dirty") else ""
        print(f"  {cls:<18} {name:<40}{d}  {act}")
    print()
    for k in sorted(counts):
        print(f"  {counts[k]:4d}  {k}")
    print(f"\n  {'reaped' if args.apply else 'would reap'}: "
          f"{sum(1 for _, a in actions if 'remov' in a and not a.startswith('KEPT: PINNED'))}")
    # Printed on EVERY run, including zero. A veto count that only appears when
    # it is nonzero is indistinguishable from a scanner that never ran, which
    # is the shape #3470 and #1002 are both about.
    print(f"  pinned-artifact vetoes: {pinned_vetoes}")
    if args.apply:
        git(["worktree", "prune"], cwd=repo)
        print("  pruned")
    return 0


if __name__ == "__main__":
    sys.exit(main())
