#!/usr/bin/env python3
"""w-slots — read the label LEAD of a class off REAL c2's own obj.

`w-fenceb`'s `mfix.py` in generic form. For each `.cpp` given it captures the
`.gl`/`.ex` IL and the reference obj under real `c2.dll` (wibo), then:

    base      = counter + 9 + 3*nsegs + nleaf     <- what plan_labels charges
                                                     TODAY, i.e. every leaf at
                                                     lead 0 (`lead + 1` each)
    TRUE lead = real $M of the framed function - base

The framed function must be LAST and every function before it a leaf, which is
what every cell this lane builds is. A file with no framed function prints so:
the counter never reaches its obj (board #742) and no charge can break it.

    work/w-slots/lead.py [--mode '/O1 /GS- /c'] file.cpp ...

**The obj is the judge.** Nothing here is quoted from `docs/LABEL_COUNTER.md`;
three lanes have measured that table wrong and it is mode-dependent.
"""

import os
import subprocess
import sys
import tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.dirname(os.path.dirname(HERE))
sys.path.insert(0, os.path.join(REPO, "scripts"))
sys.path.insert(0, os.path.join(REPO, "work", "w-fenceb"))
import gt_label_stride as G  # noqa: E402
from gt_dump import Obj  # noqa: E402
import labelil as L  # noqa: E402

LABEL_SEED_GAP = 9


def lead_of(cpp, mode, wd):
    """-> (lead, note). `lead` is None when the file cannot price a charge."""
    path = cpp if os.path.isabs(cpp) else os.path.join(REPO, cpp)
    r = subprocess.run([os.path.join(REPO, "scripts", "gt_capture.sh"), path]
                       + mode.split(), capture_output=True, text=True)
    objp = r.stdout.strip()
    if not objp or not os.path.exists(objp):
        return None, "CAPTURE FAILED"
    o = Obj(open(objp, "rb").read())
    framed = [g for g in G.groups(o) if g["labels"]]
    tag = os.path.basename(path)[:-4]
    fl = os.path.join(wd, "flags_%s.txt" % tag)
    open(fl, "w").write("/nologo " + mode + "\n")
    out = os.path.join(wd, "il_" + tag)
    os.makedirs(out, exist_ok=True)
    subprocess.run([os.path.join(REPO, "target", "release", "c2rs"),
                    "capture", path, "--keep-il", out, "--flags-file", fl],
                   capture_output=True, text=True)
    got = {f.rsplit(".", 1)[-1]: os.path.join(out, f) for f in os.listdir(out)}
    if "gl" not in got or "ex" not in got:
        return None, "IL CAPTURE FAILED"
    counter = int.from_bytes(open(got["gl"], "rb").read()[7:11], "little")
    segs = L.ex_segments(open(got["ex"], "rb").read())
    if not framed:
        return None, ("counter %d  %d segs  NO FRAMED FUNCTION -> the counter "
                      "never reaches this obj (board #742)" % (counter, len(segs)))
    nleaf = len(segs) - 1
    base = counter + LABEL_SEED_GAP + 3 * len(segs) + nleaf
    real = min(framed[0]["labels"])
    return real - base, ("counter %d  %d segs  base $M%d  real $M%d  framed %s"
                         % (counter, len(segs), base, real, framed[0]["name"]))


def main(argv):
    mode = "/O1 /GS- /c"
    if "--mode" in argv:
        i = argv.index("--mode"); mode = argv[i + 1]; del argv[i:i + 2]
    srcs = [a for a in argv[1:] if not a.startswith("--")]
    wd = tempfile.mkdtemp(prefix="wslots")
    print("mode: %s" % mode)
    print()
    for cpp in srcs:
        lead, note = lead_of(cpp, mode, wd)
        name = os.path.basename(cpp)
        if lead is None:
            print("  %-34s   --      %s" % (name, note))
        else:
            print("  %-34s  LEAD %+d   %s" % (name, lead, note))
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
