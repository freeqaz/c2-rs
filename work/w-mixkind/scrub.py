#!/usr/bin/env python3
"""scrub.py — derive a COMMITTABLE copy of a measurement log.

    scrub.py <src> <dst>

CLAUDE.md forbids absolute machine paths in anything committed. Three rules,
each of which has been broken here recently:

1. **The path is DERIVED, never hard-coded.** The repo root comes from
   `git rev-parse --show-toplevel`, and any residual `/home/<user>/…` is matched
   as a pattern (`/home/[a-z_][a-z0-9_-]*/`). A scrubber with a baked-in path is
   the same violation one layer out.

2. **It NEVER rewrites the source.** Board **#1135**/#1236: a rewrite of a file
   whose writer holds a stale offset punches a NUL hole, and the guard every
   lane copied — compare `st_size` before and after the read — is a
   microsecond-wide timing check that passes exactly when the writer is busy.
   This reads `src` and writes a **different** `dst`. There is no in-place mode
   to get wrong, and `assert_clean.py` still walks `/proc/*/fd` before either.

3. **The result is verified by a BYTE COUNT, not by `grep`.** `grep -c $'\\0'`
   counts lines and reports 0 on a file with 170 NUL bytes in it.

Prints what it replaced, and FAILS if the output still carries an absolute path
— a scrubber that silently leaves one is worse than none.
"""

import os
import re
import subprocess
import sys

ABS = re.compile(rb"/home/[a-z_][a-z0-9_-]*/[^\s\"']*")


def main(src, dst):
    root = subprocess.run(["git", "rev-parse", "--show-toplevel"],
                          capture_output=True, text=True,
                          check=True).stdout.strip().encode()
    raw = open(src, "rb").read()
    n0 = len(ABS.findall(raw))

    out = raw.replace(root, b"<repo>")
    # anything left that is still an absolute home path — a sibling checkout, a
    # /tmp run dir under someone's home — collapses to its basename's parent.
    out = ABS.sub(lambda m: b"<abs>/" + m.group(0).rsplit(b"/", 1)[-1], out)

    n1 = len(ABS.findall(out))
    nul = len(out) - len(out.replace(b"\x00", b""))
    with open(dst, "wb") as f:
        f.write(out)

    print("  %s -> %s" % (os.path.relpath(src), os.path.relpath(dst)))
    print("    absolute paths: %d in, %d out" % (n0, n1))
    print("    NUL bytes (BYTE count, not grep): %d" % nul)
    print("    %d bytes in, %d out" % (len(raw), len(out)))
    if n1 or nul:
        print("    FAIL: the output is not committable")
        return 1
    print("    OK")
    return 0


if __name__ == "__main__":
    if len(sys.argv) != 3:
        print(__doc__)
        sys.exit(2)
    sys.exit(main(sys.argv[1], sys.argv[2]))
