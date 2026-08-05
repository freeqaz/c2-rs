#!/usr/bin/env python3
"""scrub.py — remove absolute machine paths from this lane's recorded logs.

`docs/CLAUDE.md` forbids committing `/home/<user>/…` paths: the toolchain's
location is env-driven by design, so a recorded absolute path is both noise and
a claim about somebody else's disk. Two lanes shipped them on 2026-08-05 and
they had to be scrubbed at the merge, which is why this is a script in the lane
rather than a careful `sed` nobody can re-run.

It rewrites in place and prints a COUNT — a status would not distinguish
"scrubbed 5" from "matched nothing", which is trap 5's shape.

Usage:
    work/w-varloop/scrub.py work/w-varloop/gate.txt [...]
"""

import os
import re
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.dirname(os.path.dirname(HERE))


def main():
    total = 0
    for path in sys.argv[1:]:
        s = open(path).read()
        before = s
        # The worktree itself, then the main checkout, then any other home.
        s = s.replace(REPO, "<repo>")
        s = re.sub(r"/home/[A-Za-z0-9_.-]+/code/milohax/c2-rs[^\s,)]*", "<repo>", s)
        s = re.sub(r"/home/[A-Za-z0-9_.-]+", "<home>", s)
        n = len(re.findall(r"/home/", before))
        if s != before:
            open(path, "w").write(s)
        left = len(re.findall(r"/home/", s))
        print("%s: %d absolute path(s) scrubbed, %d remaining" % (path, n, left))
        total += n
        if left:
            return 1
    print("TOTAL scrubbed: %d" % total)
    return 0


if __name__ == "__main__":
    sys.exit(main())
