#!/usr/bin/env python3
"""scrub.py — replace absolute machine paths in this lane's recorded artifacts.

`<worktree>` for this lane's own tree, `<c2-rs>` for the main checkout,
`<milohax>` for the parent, and `<home>` for anything else under a user's home,
so the committed evidence names no box and no user (CLAUDE.md's hard constraint,
and a lane leaked them into 56 files this week).

**BOARD #1135 — never rewrite a file another process still holds open.** A scrub
that raced a backgrounded `gate.sh` punched a NUL hole into a PASSING gate's log
and made a waiter report TIMEOUT; the mirror case makes `grep -q FAIL` read clean
on a FAILING one. So this script:

  * takes an explicit file list — never a glob, never a directory walk;
  * REFUSES any file that already contains a NUL byte, because that is the
    symptom of the race and the repair is a clean re-run, not a patch;
  * writes through a temporary file and renames, so a reader never sees a
    half-written one;
  * asserts the result is NUL-free before it returns.

Usage:  scrub.py <file> [file ...]
"""

import os
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
MAIN = os.path.realpath(os.path.join(ROOT, "..", "..", ".."))
PARENT = os.path.dirname(MAIN)
HOME = os.path.expanduser("~")

# Longest first: the worktree path CONTAINS the main checkout's path.
SUBS = [
    (ROOT, "<worktree>"),
    (MAIN, "<c2-rs>"),
    (PARENT, "<milohax>"),
    (HOME, "<home>"),
]


def main(paths):
    if not paths:
        sys.exit("usage: scrub.py <file> [file ...]")
    for p in paths:
        if not os.path.isfile(p):
            print("skip (not a file): %s" % p)
            continue
        raw = open(p, "rb").read()
        if b"\0" in raw:
            sys.exit(
                "REFUSING %s — it contains a NUL byte. That is board #1135's\n"
                "signature (a writer still held the fd). Repair by a CLEAN "
                "RE-RUN, never by patching." % p
            )
        text = raw.decode("utf-8", "replace")
        for frm, to in sorted(SUBS, key=lambda kv: -len(kv[0])):
            text = text.replace(frm, to)
        tmp = p + ".scrub.tmp"
        with open(tmp, "w") as f:
            f.write(text)
        os.replace(tmp, p)
        assert b"\0" not in open(p, "rb").read(), p
    print("scrubbed %d file(s)" % len(paths))


if __name__ == "__main__":
    main(sys.argv[1:])
