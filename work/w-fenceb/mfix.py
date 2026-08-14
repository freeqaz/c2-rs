#!/usr/bin/env python3
"""mfix.py — run the PREREG §4 control against a TRACKED FIXTURE.

Lane **w-backedge**. `mchan.py` constructs its own two-function TUs; this runs
the same predictor against a `.cpp` that already exists in the tree, so the
price of a fence can be argued from a cell the gate already carries.

**POST-HOC.** These sources are not in either frozen grid and were chosen
*after* grid2 was scored. Nothing here is counted toward `PREREG.md` §2's
claims; it is reported as a separate, labelled reading.

The target is `fixtures/cpp/whash_loop_then_framed.cpp` — board **#747**'s
constructed shape, a loop leaf followed by a framed function — which the port
**refuses today**, because `IlFunction::label_slots` returns `None` for the
loop shape and the three-valued gate in `IlBundle::functions` then rejects the
whole TU. The question this answers is the fence's price: *would a
charge-aware `label_slots` get this TU's `$M` right?*

    work/w-backedge/mfix.py [file.cpp ...]
"""

import os
import subprocess
import sys
import tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.dirname(os.path.dirname(HERE))
sys.path.insert(0, os.path.join(REPO, "scripts"))
sys.path.insert(0, HERE)
import gt_label_stride as G  # noqa: E402
from gt_dump import Obj  # noqa: E402
import labelil as L  # noqa: E402

LABEL_SEED_GAP = 9
DEFAULT = ["fixtures/cpp/whash_loop_then_framed.cpp",
           "fixtures/cpp/whash_ptr_walk_loop.cpp"]


def main(argv):
    mode = "/O1 /GS- /c"
    if "--mode" in argv:
        i = argv.index("--mode"); mode = argv[i + 1]; del argv[i:i + 2]
    srcs = [a for a in argv[1:] if not a.startswith("--")] or DEFAULT
    wd = tempfile.mkdtemp(prefix="wbef")
    print("mode: %s   POST-HOC — not in either frozen grid, not scored in §2" % mode)
    print()
    ok = 0
    for cpp in srcs:
        path = os.path.join(REPO, cpp)
        r = subprocess.run([os.path.join(REPO, "scripts", "gt_capture.sh"), path]
                           + mode.split(), capture_output=True, text=True)
        objp = r.stdout.strip()
        if not objp or not os.path.exists(objp):
            print("%s  CAPTURE FAILED" % cpp); continue
        o = Obj(open(objp, "rb").read())
        gs = G.groups(o)
        framed = [g for g in gs if g["labels"]]
        tag = os.path.basename(cpp)[:-4]
        fl = os.path.join(wd, "flags.txt")
        open(fl, "w").write("/nologo " + mode + "\n")
        out = os.path.join(wd, "il_" + tag)
        os.makedirs(out, exist_ok=True)
        subprocess.run([os.path.join(REPO, "target", "release", "c2rs"),
                        "capture", path, "--keep-il", out, "--flags-file", fl],
                       capture_output=True, text=True)
        got = {f.rsplit(".", 1)[-1]: os.path.join(out, f) for f in os.listdir(out)}
        gl = open(got["gl"], "rb").read()
        ex = open(got["ex"], "rb").read()
        counter = int.from_bytes(gl[7:11], "little")
        segs = L.ex_segments(ex)
        charges = [2 * L.ex_cflow(s)["bwd_uncond"] + L.ex_cflow(s)["bwd_cond"]
                   for s in segs]
        print("%s" % cpp)
        print("   counter %d   %d segments   R1 charge per function: %s"
              % (counter, len(segs), charges))
        if not framed:
            print("   no framed function -> the counter's value never reaches "
                  "this obj (board #742). Nothing to predict, and nothing a "
                  "wrong charge could break.")
            ok += 1
            continue
        # The framed function is last here; everything before it is a leaf, so
        # `plan_labels` charges each of them `lead + 1`.
        nleaf = len(segs) - 1
        base = counter + LABEL_SEED_GAP + 3 * len(segs) + nleaf
        real = min(framed[0]["labels"])
        pred = base + sum(charges[:nleaf])
        print("   framed %-28s real $M%d" % (framed[0]["name"], real))
        print("   predicted with R1        $M%d   %s"
              % (pred, "GREEN" if pred == real else "RED"))
        print("   predicted with charge 0  $M%d   %s   <- what plan_labels "
              "charges today" % (base, "GREEN" if base == real else "RED"))
        print("   the TRUE lead is %d" % (real - base))
        if pred == real:
            ok += 1
    print("\n%d of %d" % (ok, len(srcs)))
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
