#!/usr/bin/env python3
"""AUDIT, part 3 — the intersection, and a verdict on each lane in it.

`audit.py`  : which landed lanes have a gate transcript whose `crates/` is the
              `crates/` that landed.
`audit2.py` : which transcripts were produced by a `gate.sh` that HAD the
              defect at all (the 378a3cae..c6d6602e window).

A landed gate claim can only be suspect if it is in BOTH sets: inside the
window, and naming a `crates/` that is not the one the lane shipped.  This file
takes that intersection and, for each lane in it, separates the two reasons a
transcript's `crates/` can differ from the landed tip's:

  REBASE   the named commit is NOT an ancestor of the landed tip.  The lane
           gated, then rebased onto a moved master before merging — the
           standing landing rule on this box.  The difference is every peer
           change in between, and the lane's own work is in both trees.  This
           is not #2907 and no gate claim depends on it being absent.

  AHEAD    the named commit IS an ancestor of the landed tip and `crates/`
           moved between them.  The lane committed port work AFTER its last
           frozen gate.  That is a stale gate — a real reading hazard, and the
           one #2907 can produce SILENTLY, because the eaten edit is exactly
           work the lane then had to re-apply and commit.

`AHEAD` is the only column where a landed claim can be wrong for this lane's
reason, and even there the finding is "re-gate", not "the port is broken":
`crates/` moving forward does not make the earlier PASS false about the earlier
tree.
"""

import os
import re
import subprocess
import sys

OPEN = "378a3cae"
PIN = re.compile(r"^\s*sha ([0-9a-f?]+)\s+tree ([0-9a-f]+)(-dirty)?\s*$", re.M)
MERGE = re.compile(r"^merge (w[-b][a-z0-9_-]+)")


def git(*a):
    r = subprocess.run(["git"] + list(a), capture_output=True, text=True)
    return r.returncode, r.stdout.strip()


def ctree(c):
    rc, out = git("rev-parse", "%s:crates" % c)
    return out if rc == 0 else None


def main():
    rc, out = git("log", "--merges", "--format=%H\t%P\t%s")
    lanes = {}
    for line in out.splitlines():
        h, parents, subj = line.split("\t", 2)
        m = MERGE.match(subj)
        ps = parents.split()
        if m and len(ps) >= 2:
            lanes.setdefault(m.group(1), ps[1])

    per_lane = {}
    for root, dirs, names in os.walk("work"):
        dirs[:] = [d for d in dirs if d not in (".git", "target")]
        for n in names:
            if not n.endswith((".txt", ".log", ".md")):
                continue
            p = os.path.join(root, n)
            try:
                text = open(p, encoding="utf-8", errors="replace").read()
            except OSError:
                continue
            hits = PIN.findall(text)
            if not hits:
                continue
            lane = p.split(os.sep)[1]
            for _b, tree, dirty in hits:
                per_lane.setdefault(lane, []).append((p, tree, bool(dirty)))

    print("%-14s %-8s %-8s %-9s %s" % ("lane", "landed", "graded", "relation", "transcript"))
    print("-" * 88)
    n_rebase = n_ahead = n_clean = n_out = 0
    ahead = []
    for lane in sorted(lanes):
        tip = lanes[lane]
        tt = ctree(tip)
        recs = per_lane.get(lane, [])
        if not recs or tt is None:
            continue
        inwin = []
        for p, tree, dirty in recs:
            if git("cat-file", "-e", tree + "^{commit}")[0] != 0:
                continue
            if git("merge-base", "--is-ancestor", OPEN, tree)[0] == 0:
                inwin.append((p, tree, dirty))
        if not inwin:
            n_out += 1
            continue
        if any(ctree(t) == tt for _p, t, _d in inwin):
            n_clean += 1
            print("%-14s %-8s %-8s %-9s %s" % (lane, tip[:8], "=", "EXACT",
                  "a frozen gate graded the crates/ that landed"))
            continue
        for p, tree, _d in inwin:
            anc = git("merge-base", "--is-ancestor", tree, tip)[0] == 0
            rel = "AHEAD" if anc else "REBASE"
            if anc:
                n_ahead += 1
                ahead.append((lane, tip, tree, p))
            else:
                n_rebase += 1
            print("%-14s %-8s %-8s %-9s %s" % (lane, tip[:8], tree[:8], rel, p[:44]))
    print("-" * 88)
    print("lanes with an in-window transcript that graded the LANDED crates/ : %d" % n_clean)
    print("in-window transcripts differing by REBASE (peers moved, not #2907) : %d" % n_rebase)
    print("in-window transcripts differing by AHEAD (lane committed after)    : %d" % n_ahead)
    print("landed lanes whose transcripts are all OUTSIDE the window          : %d" % n_out)
    if ahead:
        print()
        print("THE AHEAD ROWS — what `crates/` moved between the gate and the landing:")
        for lane, tip, tree, p in ahead:
            rc, out = git("diff", "--stat", "%s..%s" % (tree, tip), "--", "crates/")
            print("  %s   %s -> %s" % (lane, tree[:8], tip[:8]))
            tail = out.splitlines()[-1] if out else "(no crates/ change)"
            print("     %s" % tail.strip())
            rc, out2 = git("log", "--format=  %h %s", "%s..%s" % (tree, tip), "--", "crates/")
            for ln in out2.splitlines()[:8]:
                print("   " + ln[:100])
    return 0


if __name__ == "__main__":
    sys.exit(main())
