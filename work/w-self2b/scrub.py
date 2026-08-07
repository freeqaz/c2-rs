#!/usr/bin/env python3
"""scrub.py — replace this box's absolute paths in a lane artefact.

CLAUDE.md forbids committing absolute machine paths **and** forbids hard-coding
them in the scrubber, so the prefixes are DERIVED:

  * the sweep outdir comes out of the artefact's own `TALLY dir=<...>` line;
  * anything else matching `/home/<user>/...` is matched by PATTERN
    (`/home/[a-z][a-z0-9_-]*/`), never by a baked-in name.

Board #1135: never rewrite a file another process still holds open. This asserts
the file is **NUL-free before and after**, and refuses a file that grew while it
was being read — the repair for a corrupt artefact is a clean re-run of
`tally.sh`, never a patch.

**THE SIZE CHECK IS NOT ENOUGH, AND THIS LANE PROVED IT ON ITSELF.** The first
revision compared `st_size` before and after the read, which is a window of
microseconds; `w-self2b` ran it over `gate_tip.txt` while the gate's own
`>` redirect still held the file, the sizes matched because the gate happened to
be inside a 30-minute `mode_cross` step, and the rewrite shortened a file whose
writer holds a stale offset — the NUL-hole #1135 describes, created by the very
script written to prevent it. So the check is now **a scan of `/proc/*/fd` for
any process holding the path**, which is a fact about writers and not about
timing, and the size check is kept as a second line.

    scrub.py <file>...
"""

import os
import re
import sys

HOME_RX = re.compile(rb"/home/[a-z][a-z0-9_-]*/[^\s\"']*")
DIR_RX = re.compile(rb"TALLY dir=(\S+)")


def holders(path):
    """Every pid holding `path` open, by walking `/proc/*/fd`. No `lsof`, no
    `fuser`, no pattern that could match this script's own argv."""
    real = os.path.realpath(path)
    out = []
    me = str(os.getpid())
    for pid in os.listdir("/proc"):
        if not pid.isdigit() or pid == me:
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


def scrub(path):
    h = holders(path)
    if h:
        print("  %s REFUSED: still held open by pid(s) %s — #1135. The repair"
              " is a clean re-run, never a patch." % (path, h))
        return 1
    n0 = os.path.getsize(path)
    b = open(path, "rb").read()
    if b"\x00" in b:
        print("  %s REFUSED: already contains a NUL — re-run tally.sh" % path)
        return 1
    if os.path.getsize(path) != n0:
        print("  %s REFUSED: grew while being read — a writer still holds it"
              % path)
        return 1
    m = DIR_RX.search(b)
    out = b
    if m:
        out = out.replace(m.group(1) + b"/", b"<sweepdir>/")
        out = out.replace(b"dir=" + m.group(1), b"dir=<sweepdir>")
    out = HOME_RX.sub(b"<abs>", out)
    if b"\x00" in out:
        print("  %s REFUSED: scrub introduced a NUL" % path)
        return 1
    if out != b:
        open(path, "wb").write(out)
    print("  %-40s %d B -> %d B%s"
          % (os.path.basename(path), len(b), len(out),
             "" if m else "   (no TALLY dir= line; only /home/ scrubbed)"))
    return 0


def main():
    bad = 0
    for p in sys.argv[1:]:
        bad += scrub(p)
    print("  REFUSED: %d" % bad)
    return 1 if bad else 0


if __name__ == "__main__":
    sys.exit(main())
