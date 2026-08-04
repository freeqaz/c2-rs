#!/usr/bin/env python3
"""ilseed.py — is the control-flow surcharge visible in the IL, or only in the obj?

`work/w-label/PREREG.md` §1.4 registered *"no body without a backward branch
charges more than +1"*, and the held-out cell `ho-ternary` **refuted it**: it is
forward-only, has one interior join, and charges **+2**. `cf-ifelse` is the same
emitted shape — a `bc` over a two-word block into a shared `bl`, `CFG_SHAPE.md`
§3.4.1's tail-merge — and charges **+1**.

Two bodies whose emitted CFGs are near-identical and whose surcharges differ is
the strongest form of *the counter is not a function of the obj*. Lane w-order
established the same shape one level up: c2's emit ORDER is readable from the IL
and **is not in the obj**. This script asks the same question of the counter:
it captures the IL for a pair of probes and prints, per pair, the `.gl` label
counter that seeds the TU and the sizes of the five captured files.

If the surcharge were a c2 layout quantity it would be recoverable from the
emitted blocks, and it is not. If it is a front-end quantity it is upstream of
c2 entirely — in which case the `.gl` seed is where to look, because that is the
one label-counter number the front end hands over.

    work/w-label/ilseed.py            # every probe named below
"""

import os
import struct
import subprocess
import sys
import tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.dirname(os.path.dirname(HERE))
sys.path.insert(0, os.path.join(REPO, "scripts"))
import gt_label_stride as G  # noqa: E402
sys.path.insert(0, HERE)
import cflabels  # noqa: E402

C2RS = os.path.join(REPO, "target", "release", "c2rs")

# The pairs that matter, each holding the emitted shape as close to fixed as the
# source allows and moving only the thing under test.
WANT = ["cf-none", "cf-if2", "cf-ifelse", "ho-ternary", "cf-merge2",
        "cf-dowhile", "cf-while"]


def main(argv):
    mode = "/O1 /GS- /c"
    wd = tempfile.mkdtemp(prefix="ilseed")
    pool = {p[0]: p for p in cflabels.PROBES + cflabels.HELDOUT}
    print("%-14s %10s %10s %s" % ("probe", "gl-counter", "gl-bytes", "note"))
    for name in WANT:
        p = pool[name]
        src = G.build_src(p[1], p[2], p[3])
        cpp = os.path.join(wd, name.replace("-", "_") + ".cpp")
        open(cpp, "w").write(src)
        keep = os.path.join(wd, name.replace("-", "_") + "_il")
        os.makedirs(keep, exist_ok=True)
        r = subprocess.run([C2RS, "capture", cpp, "--keep-il", keep],
                           capture_output=True, text=True)
        if r.returncode != 0:
            print("%-14s  capture failed: %s" % (name, r.stderr.strip()[:80]))
            continue
        gl = None
        for f in os.listdir(keep):
            if f.endswith(".gl"):
                gl = os.path.join(keep, f)
        if gl is None:
            print("%-14s  no .gl in %s (%s)" % (name, keep, os.listdir(keep)))
            continue
        d = open(gl, "rb").read()
        # `docs/OBJ_GY_SHAPES.md` §3.5: the TU's first compiler label is
        # u32(.gl[7..11]) + 9 (`coff::LABEL_SEED_GAP`).
        counter = struct.unpack_from("<I", d, 7)[0] if len(d) >= 11 else -1
        print("%-14s %10d %10d  %s" % (name, counter, len(d), p[4][:60]))
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
