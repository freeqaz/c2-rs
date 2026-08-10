#!/usr/bin/env python3
"""AUDIT, part 2 — the EXPOSURE WINDOW, and what is inside it.

`audit.py` compares every landed lane's gate transcript against the `crates/`
that landed.  It over-reports by construction: this project lands lanes with
`git merge --no-ff` AFTER rebasing onto a moved master, so a transcript made
before the rebase names a commit whose `crates/` differs from the landed tip's
by every peer change in between — a difference that has nothing to do with
#2907.  This file cuts the population down to what can actually be exposed and
then reads what is left.

THE WINDOW
==========
`hatch_red_run` — and with it the `hatch_red.py --list` arm-count probe that
carries the destructive `finally` — entered `scripts/gate.sh` at **378a3cae**
(2026-08-08, board #1406) and was fixed at **c6d6602e** (2026-08-10, this lane).
A gate run whose HEAD does not have 378a3cae as an ancestor ran a `gate.sh`
with no hatch-red row at all: nothing invoked `hatch_red.py`, nothing could
revert `crates/`, and the defect is not merely absent from the transcript, it
did not exist in the code that produced it.

That is a hard bound, not a heuristic, and it is the antecedent every claim
below is conditioned on.
"""

import os
import re
import subprocess
import sys

OPEN = "378a3cae"     # hatch_red_run enters gate.sh
SHUT = "c6d6602e"     # this lane's fix
PIN = re.compile(r"^\s*sha ([0-9a-f?]+)\s+tree ([0-9a-f]+)(-dirty)?\s*$", re.M)
HR_ROW = re.compile(r"^hatch-red\s+(\S+)", re.M)
HR_INLINE = re.compile(r"^\s+(PASS|FAIL|REFUSED|NO-RESULT)\s+(\S*)", re.M)


def git(*a):
    r = subprocess.run(["git"] + list(a), capture_output=True, text=True)
    return r.returncode, r.stdout.strip()


def is_desc(anc, c):
    return git("merge-base", "--is-ancestor", anc, c)[0] == 0


def main():
    rows = []
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
            for _b, tree, dirty in PIN.findall(text):
                rows.append((p, tree, bool(dirty), text))

    inside, outside, unres = [], [], []
    for p, tree, dirty, text in rows:
        if git("cat-file", "-e", tree + "^{commit}")[0] != 0:
            unres.append((p, tree))
            continue
        (inside if is_desc(OPEN, tree) else outside).append((p, tree, dirty, text))

    print("gate transcripts with a pinned identity : %d" % len(rows))
    print("  named commit unresolvable in this repo: %d" % len(unres))
    print("  OUTSIDE the window (gate.sh had no hatch-red row; the defect did not")
    print("  exist in the code that produced them) : %d" % len(outside))
    print("  INSIDE the window %s..%s        : %d" % (OPEN, SHUT, len(inside)))
    print()

    print("THE %d RUNS INSIDE THE WINDOW, and what their own hatch-red row says:" % len(inside))
    print()
    print("  %-46s %-9s %-6s %s" % ("transcript", "tree", "dirty", "hatch-red row"))
    verdicts = {}
    for p, tree, dirty, text in sorted(inside):
        m = re.search(r"^hatch-red\s+(\S+)\s", text, re.M)
        v = m.group(1) if m else "-"
        w = ""
        m2 = re.search(r"^hatch-red\s+\S+.*?<- (\S+)", text, re.M)
        if m2:
            w = m2.group(1)
        else:
            m3 = re.search(r"^\s+(REFUSED|PASS|FAIL|NO-RESULT)\s+(\S*)", text, re.M)
            if m3:
                w = m3.group(2)
        verdicts[v + " " + w] = verdicts.get(v + " " + w, 0) + 1
        print("  %-46s %-9s %-6s %s %s" % (p[:46], tree, "yes" if dirty else "no", v, w))
    print()
    print("  hatch-red verdicts seen inside the window:")
    for k, n in sorted(verdicts.items()):
        print("    %-70s %d" % (k, n))
    print()
    print("READING")
    print("  DIRTY-TREE  = the row REFUSED before running: the tree was dirty in a way")
    print("                `git diff HEAD` could still see, i.e. STAGED. `git checkout --`")
    print("                restores from the index, so a staged edit was never at risk,")
    print("                and the row not running means nothing was written either.")
    print("  HATCH-STALE = the arms RAN. The --list probe had already reverted `crates/`")
    print("                by then, so anything UNSTAGED there is gone and the build that")
    print("                follows is of the named commit exactly.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
