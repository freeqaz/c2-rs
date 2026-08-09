#!/usr/bin/env python3
"""scrub.py — rewrite every absolute machine path in a committed text artifact
to a repo-relative one, and ASSERT that none survives.

CLAUDE.md forbids committing absolute machine paths (`/home/<user>/...`).  The
disassembly dumps `scripts/gt_dump.py` writes lead with the obj's absolute
path, so every one of them has to pass through here before it is staged.  It
is an assert and not a best-effort filter: a scrubber that silently leaves one
behind is worse than no scrubber, because the pre-commit check is this file.

Usage:  scrub.py <file> [<file> ...]      rewrites in place, exits non-zero if
                                          anything still matches `/home/`.
"""
import os
import re
import sys

# The worktree root, derived from this file's own location.
ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", ".."))

bad = 0
for path in sys.argv[1:]:
    with open(path, "r", encoding="utf-8", errors="replace") as fh:
        text = fh.read()
    text = text.replace(ROOT + "/", "")
    text = text.replace(ROOT, ".")
    # Any other absolute home path (a sibling checkout, a temp dir) is replaced
    # by a marker rather than a guess: the point is that no machine path ships.
    text = re.sub(r"/home/[^\s\"'()]*", "<scrubbed-abs-path>", text)
    with open(path, "w", encoding="utf-8") as fh:
        fh.write(text)
    if "/home/" in text:
        print(f"SCRUB FAILED: {path} still contains /home/", file=sys.stderr)
        bad += 1

sys.exit(1 if bad else 0)
