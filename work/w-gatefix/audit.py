#!/usr/bin/env python3
"""AUDIT — is any LANDED gate claim on this project evidence about a tree the
gate did not grade?  Boards #2668 / #2907, lane `w-gatefix`, 2026-08-10.

THE MECHANISM, STATED SO THE TEST CAN BE DERIVED FROM IT
========================================================
`gate.sh`'s first row read the declared arm count with `hatch_red.py --list`,
whose module-level `finally` did an unconditional `git checkout -- crates/` —
ABOVE the interlock that existed to stop exactly that.  Three consequences,
each of which constrains the audit:

  * `git checkout -- <path>` restores the worktree FROM THE INDEX.  So the
    edits destroyed are precisely the **unstaged** `crates/` modifications; a
    staged one survives, is still seen by `git diff HEAD`, and therefore still
    trips the interlock and refuses the row.
  * After the revert, `crates/` **is** the index's `crates/`, which for every
    lane on this project is `HEAD`'s.  The build that follows therefore compiles
    exactly the named commit's port sources.
  * `pin_harness` prints `tree <short HEAD>[-dirty]`, and the `-dirty` marker is
    `git diff --quiet HEAD` over the whole worktree read AFTER that row.  So a
    run that started with unstaged `crates/` dirt and nothing else prints a
    CLEAN identity, and no reader can tell it from a run that was clean all
    along.  That is the invalidation vector and it is invisible in the
    transcript.

So the transcript's own `crates/` claim was never false — the revert made it
true.  What can be false is the RUNG's claim that the transcript covers the
lane's tip.  That is decidable, because lanes land by `git merge --no-ff` and
the merge's SECOND PARENT is the lane tip exactly.

THE TEST
========
For every landed lane:
  * `tip` = second parent of its `merge w-<lane>:` commit — what actually
    landed;
  * `T`   = every commit named by a `sha … tree …` identity line in a gate
    transcript under `work/<lane>/`;
  * the lane's gate claim STANDS if some `T` has `T:crates` byte-identical to
    `tip:crates` — the graded port sources are the landed port sources, whatever
    happened to any scratch on the way.

`crates/` and not the whole tree: the gate grades the port, `docs/` and `work/`
move in the landing commit by construction, and comparing commit ids would mark
every lane suspect for a reason that has nothing to do with #2907.
"""

import os
import re
import subprocess
import sys

PIN = re.compile(r"^\s*sha ([0-9a-f?]+)\s+tree ([0-9a-f]+)(-dirty)?\s*$", re.M)
MERGE = re.compile(r"^merge (w[-b][a-z0-9_-]+)")


def git(*a):
    r = subprocess.run(["git"] + list(a), capture_output=True, text=True)
    return r.returncode, r.stdout.strip()


def crates_tree(commit):
    rc, out = git("rev-parse", "%s:crates" % commit)
    return out if rc == 0 else None


def main():
    # ---- every landed lane, and the exact commit it landed ------------------
    rc, out = git("log", "--merges", "--format=%H\t%P\t%s")
    lanes = {}
    for line in out.splitlines():
        h, parents, subj = line.split("\t", 2)
        m = MERGE.match(subj)
        if not m:
            continue
        ps = parents.split()
        if len(ps) < 2:
            continue
        lanes.setdefault(m.group(1), ps[1])          # first (newest) merge wins
    print("landed lanes found by `merge w-*` merge commits: %d" % len(lanes))

    # ---- every gate transcript, grouped by the lane whose work/ it is in ----
    per_lane = {}
    dirty_lines = 0
    total_lines = 0
    for root, dirs, names in os.walk("work"):
        dirs[:] = [d for d in dirs if d not in (".git", "target")]
        for n in names:
            if not n.endswith((".txt", ".log", ".md")):
                continue
            p = os.path.join(root, n)
            try:
                with open(p, encoding="utf-8", errors="replace") as fh:
                    text = fh.read()
            except OSError:
                continue
            pins = PIN.findall(text)
            if not pins:
                continue
            lane = p.split(os.sep)[1] if os.sep in p else "?"
            for _b, tree, dirty in pins:
                total_lines += 1
                if dirty:
                    dirty_lines += 1
                per_lane.setdefault(lane, []).append((p, tree, bool(dirty)))
    print("gate transcripts carrying a pinned identity: %d lines over %d lane dirs"
          % (total_lines, len(per_lane)))
    print("  of those, %d printed `-dirty` and %d printed a clean identity"
          % (dirty_lines, total_lines - dirty_lines))
    print()

    stands, nogate, suspect, unresolved = [], [], [], []
    for lane, tip in sorted(lanes.items()):
        tt = crates_tree(tip)
        if tt is None:
            unresolved.append((lane, "landed tip %s has no crates/" % tip[:8]))
            continue
        recs = per_lane.get(lane, [])
        if not recs:
            nogate.append(lane)
            continue
        hit = None
        seen = []
        for path, tree, _d in recs:
            rc, _ = git("cat-file", "-e", tree + "^{commit}")
            if rc != 0:
                seen.append((path, tree, "UNRESOLVABLE"))
                continue
            gt = crates_tree(tree)
            seen.append((path, tree, gt))
            if gt == tt:
                hit = (path, tree)
                break
        if hit:
            stands.append((lane, hit[0], hit[1]))
        else:
            suspect.append((lane, tip[:8], tt[:8], seen))

    print("=" * 78)
    print("RESULT")
    print("=" * 78)
    print("  lanes whose gate transcript graded the EXACT crates/ that landed : %d"
          % len(stands))
    print("  lanes with no gate transcript under work/<lane>/                 : %d"
          % len(nogate))
    print("  lanes where NO transcript matches the landed crates/            : %d"
          % len(suspect))
    print("  unresolved                                                      : %d"
          % len(unresolved))
    print()
    if suspect:
        print("  the lanes to read by hand:")
        for lane, tip, tt, seen in suspect:
            print("    %-16s landed tip %s (crates %s)" % (lane, tip, tt))
            for path, tree, gt in seen[:6]:
                print("        %-44s tree %-9s crates %s"
                      % (path[:44], tree, (gt or "?")[:8]))
    print()
    if nogate:
        print("  lanes with no transcript in their own work/ dir (their gate output")
        print("  lives in the rung, in a peer's dir, or was never frozen):")
        print("    " + " ".join(sorted(nogate)))
    return 0


if __name__ == "__main__":
    sys.exit(main())
