#!/usr/bin/env python3
"""sanitize.py — strip absolute machine paths out of a committed log.

CLAUDE.md forbids committing `/home/<user>/…` and the lane brief says to check
grid and gate logs *before* committing.  A `c2rs gap` provenance header and a
`gate.sh` preamble both embed them, so the logs are rewritten rather than
dropped: the numbers are the evidence and the paths are not.

Substitutions are anchored on the longest prefix first so a worktree path does
not get half-replaced by the repo path inside it.

Usage:  sanitize.py <file>...
"""
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
WORKTREE = os.path.abspath(os.path.join(HERE, "..", ".."))
# .../c2-rs/.claude/worktrees/<lane>  ->  .../c2-rs
REPO = WORKTREE
for _ in range(3):
    REPO = os.path.dirname(REPO)
SIBS = os.path.dirname(REPO)
HOME = os.path.expanduser("~")

SUBS = [
    (WORKTREE, "<worktree>"),
    (REPO, "<repo>"),
    (os.path.join(SIBS, "dc3-decomp"), "<dc3>"),
    (os.path.join(SIBS, "wibo"), "<wibo>"),
    (SIBS, "<siblings>"),
    (HOME, "<home>"),
]
SUBS.sort(key=lambda kv: -len(kv[0]))

for path in sys.argv[1:]:
    s = open(path, encoding="utf-8", errors="replace").read()
    before = s.count("/home/")
    for a, b in SUBS:
        s = s.replace(a, b)
    open(path, "w", encoding="utf-8").write(s)
    after = s.count("/home/")
    print("%-42s %d -> %d absolute path(s)" % (path, before, after))
    if after:
        print("   REMAINING — inspect before committing")
        sys.exit(1)
