#!/usr/bin/env python3
"""Scrub absolute machine paths out of a captured artifact before committing.

CLAUDE.md forbids committing `/home/<user>/…`. This rewrites the three roots
this lane's artifacts can carry — the worktree, the main repo and the sibling
checkouts — to `<repo>`, `<mainrepo>` and `<sib>`, and then ASSERTS that no
`/home/` survives, so a path spelled a way this script did not anticipate fails
the run instead of being committed.

Usage: scrub.py FILE ...
"""
import os
import re
import sys

here = os.path.realpath(os.path.join(os.path.dirname(__file__), "..", ".."))
main = re.sub(r"/\.claude/worktrees/[^/]+$", "", here)
sib = os.path.dirname(main)

subs = [(here, "<repo>"), (main, "<mainrepo>"), (sib, "<sib>")]
subs.sort(key=lambda t: -len(t[0]))

for path in sys.argv[1:]:
    raw = open(path, "rb").read().decode("utf-8", "surrogateescape")
    for a, b in subs:
        raw = raw.replace(a, b)
    raw = re.sub(r"/home/[A-Za-z0-9._-]+", "<home>", raw)
    assert "/home/" not in raw, path
    open(path, "wb").write(raw.encode("utf-8", "surrogateescape"))
    print(f"scrubbed {path}")
