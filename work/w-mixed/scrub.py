#!/usr/bin/env python3
"""scrub.py — remove absolute machine paths from this lane's committed artefacts.

CLAUDE.md forbids absolute machine paths in the tree, and it equally forbids a
scrubber with one baked in — two lanes shipped scrubbers that named this box.
So the patterns are **derived**:

  * `/home/<user>/...` by the regex `/home/[a-z][a-z0-9_-]*/`, never by a name;
  * the repo root and the worktree root by walking up from this file.

Board **#1135**: never rewrite a file another process still holds open. A scrub
that raced a backgrounded `gate.sh` punched a NUL hole into a PASSING gate's log
and `grep` then returned nothing, which on a FAILING gate would have read clean.
So this asserts the file is NUL-free **before and after**, refuses to touch a
file that grew while it was reading, and reports rather than repairs — the
repair for a corrupted artefact is a clean re-run, never a patch.

    scrub.py <file>...
"""

import os
import re
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(os.path.dirname(HERE))

HOME = re.compile(r"/home/[a-z][a-z0-9_-]*/[^\s'\"()]*")
TMPD = re.compile(r"/tmp/[A-Za-z0-9._-]*/[^\s'\"()]*")


def rel(m):
    p = m.group(0)
    if p.startswith(ROOT):
        r = os.path.relpath(p, ROOT)
        return "<repo>/" + r if r != "." else "<repo>"
    # a path outside this repo: keep the last two components only, so the
    # artefact still says WHAT it was without saying WHERE this box keeps it
    parts = p.rstrip("/").split("/")
    return "<abs>/" + "/".join(parts[-2:]) if len(parts) > 2 else "<abs>"


def scrub(path):
    with open(path, "rb") as f:
        raw = f.read()
    n0 = raw.count(b"\0")
    if n0:
        print("  %s: %d NUL byte(s) BEFORE the scrub — REFUSING. This artefact "
              "is corrupt; re-run its producer, do not patch it (#1135)"
              % (path, n0))
        return 1
    size = os.path.getsize(path)
    text = raw.decode("utf-8", "surrogateescape")
    out = TMPD.sub(rel, HOME.sub(rel, text))
    if out == text:
        print("  %s: clean already" % path)
        return 0
    if os.path.getsize(path) != size:
        print("  %s: GREW while being read — a writer still owns it. REFUSING "
              "(#1135)" % path)
        return 1
    with open(path, "wb") as f:
        f.write(out.encode("utf-8", "surrogateescape"))
    with open(path, "rb") as f:
        back = f.read()
    n1 = back.count(b"\0")
    if n1:
        print("  %s: %d NUL byte(s) AFTER the scrub — the write raced something."
              % (path, n1))
        return 1
    left = HOME.findall(back.decode("utf-8", "surrogateescape"))
    print("  %s: scrubbed, %d absolute path(s) left, 0 NULs"
          % (path, len(left)))
    return 1 if left else 0


if __name__ == "__main__":
    rc = 0
    for p in sys.argv[1:]:
        rc |= scrub(p)
    sys.exit(rc)
