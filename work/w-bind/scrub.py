#!/usr/bin/env python3
"""scrub.py — remove absolute machine paths from this lane's committed logs.

CLAUDE.md forbids committing `/home/<user>/…`, and the lane brief adds: **do not
hard-code the path in the scrubber either** — derive it. So the pattern is a
regex over the shape of a home directory, not this box's own.

Board **#1135**: never rewrite a file another process still holds open. This is
run only after the writer has exited, and it asserts the result is NUL-free
before it is committed, because a log that cannot be grepped is
indistinguishable from a log that was never written.

Usage:  scrub.py <file> [<file> ...]
"""
import re
import sys

HOME = re.compile(rb"/home/[a-z][a-z0-9_-]*")
# The repo's own root, wherever it happens to live, becomes a stable token so a
# reader can still tell "inside this checkout" from "somewhere else".
WORKTREE = re.compile(rb"<home>/[A-Za-z0-9_./-]*?/c2-rs(/\.claude/worktrees/[A-Za-z0-9_-]+)?")


def main(argv):
    bad = 0
    for path in argv:
        data = open(path, "rb").read()
        if b"\0" in data:
            print("REFUSING %s: NUL bytes — repair by a clean re-run, not by "
                  "rewriting (board #1135)" % path)
            bad = 1
            continue
        out = WORKTREE.sub(b"<repo>", HOME.sub(b"<home>", data))
        out = out.replace(b"<home>", b"<home>")
        if out != data:
            open(path, "wb").write(out)
            print("scrubbed %s" % path)
        if HOME.search(out):
            print("STILL DIRTY %s" % path)
            bad = 1
    return bad


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
