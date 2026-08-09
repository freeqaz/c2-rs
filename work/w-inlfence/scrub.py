#!/usr/bin/env python3
"""w-inlfence — path-scrub a committed analysis file, and ASSERT the scrub worked.

Usage: scrub.py FILE...

Rewrites in place, replacing this worktree's absolute path with `<worktree>`,
the sibling dc3 tree with `<dc3>` and any surviving `/home/<user>/…` prefix with
`<home>`. Then it **asserts** that no `/home/` remains — a scrubber that is
merely believed to have worked is CLAUDE.md's "never commit absolute machine
paths" on the honour system.
"""
import os
import re
import sys

HERE = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", ".."))

for path in sys.argv[1:]:
    s = open(path).read()
    s = s.replace(HERE, "<worktree>")
    s = re.sub(r"/home/[^/\s]+/code/milohax/dc3-decomp", "<dc3>", s)
    s = re.sub(r"/home/[^/\s]+/code/milohax/c2-rs", "<c2rs>", s)
    s = re.sub(r"/home/[^/\s]+", "<home>", s)
    assert "/home/" not in s, "%s: a /home/ path survived the scrub" % path
    open(path, "w").write(s)
    print("scrubbed %s" % path)
