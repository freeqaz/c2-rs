#!/usr/bin/env python3
"""assert_clean.py — every artefact this lane commits, checked for the two
things that have gone wrong here repeatedly.

    assert_clean.py <file>...

1. **NUL bytes, by a BYTE COUNT.** Board **#1236**: `grep -c $'\\0'` counts
   *lines* and reports 0 on a file with 170 NUL bytes in it, and
   `LC_ALL=C grep -q -P '\\x00'` does not fire at all — both verified. Only a
   byte comparison finds a hole. **And size is not integrity**: a rewrite can
   punch a NUL hole without changing length, which is how the scrubber written
   to prevent #1135 broke #1135.

2. **Absolute machine paths**, DERIVED and not hard-coded — `/home/<user>/…`
   with the user matched as a pattern, because a scrubber with one baked in is
   the same violation one layer out.

3. **Still held open by a writer**, by walking `/proc/*/fd` — a fact about
   writers, not about timing. A file another process holds is **NOT patched**;
   the repair is a clean re-run.

Exits non-zero on any finding, and prints what it checked either way, because a
silent pass is indistinguishable from a script that checked nothing.
"""

import os
import re
import sys

ABS = re.compile(rb"/home/[a-z_][a-z0-9_-]*/")


def holders(path):
    real = os.path.realpath(path)
    out = []
    for pid in os.listdir("/proc"):
        if not pid.isdigit():
            continue
        d = "/proc/%s/fd" % pid
        try:
            for fd in os.listdir(d):
                try:
                    if os.readlink(os.path.join(d, fd)) == real:
                        out.append(int(pid))
                        break
                except OSError:
                    pass
        except OSError:
            pass
    return out


def main(argv):
    bad = 0
    for p in argv:
        raw = open(p, "rb").read()
        n = len(raw)
        nn = len(raw.replace(b"\x00", b""))
        nul = n - nn
        abs_hits = len(ABS.findall(raw))
        held = holders(p)
        flags = []
        if nul:
            flags.append("%d NUL BYTES" % nul)
        if abs_hits:
            flags.append("%d ABSOLUTE PATHS" % abs_hits)
        if held:
            flags.append("HELD OPEN by pid(s) %s — #1135, repair by CLEAN"
                         " RE-RUN, never a patch" % held)
        print("  %-44s %8d bytes  %s"
              % (os.path.relpath(p), n,
                 "OK (NUL-free by byte count)" if not flags
                 else "REFUSED: " + "; ".join(flags)))
        bad += bool(flags)
    print("  checked %d file(s), %d REFUSED" % (len(argv), bad))
    return 1 if bad else 0


if __name__ == "__main__":
    if len(sys.argv) < 2:
        print(__doc__)
        sys.exit(2)
    sys.exit(main(sys.argv[1:]))
