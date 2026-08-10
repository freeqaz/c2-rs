#!/usr/bin/env python3
"""Is the RELAXED scan's record-position set a SUPERSET of the gate's?

It is not obvious that it is. Both scans step `p += 5` on a hit and `p += 1` on
a miss, so admitting an extra hit changes the trajectory and can step OVER a
position the narrow scan would have matched. If that ever happens, the wide
framing can bind a DIFFERENT name to a segment rather than merely more names —
and "more names" is the only version of this change with a refusal-only
failure direction.

Counted here per TU rather than argued.
"""
import sys, os, glob
from frame783 import gate_framed, wide_framed, scan, ex_splits

capdir = sys.argv[1]
tus = miss = 0
worst = []
for d in sorted(glob.glob(os.path.join(capdir, "*"))):
    done = os.path.join(d, ".done")
    if not os.path.isfile(done):
        continue
    src = open(done).read().strip()
    gl = open(glob.glob(os.path.join(d, "*.gl"))[0], "rb").read()
    g = {p for p, _, _ in scan(gl, gate_framed)}
    w = {p for p, _, _ in scan(gl, wide_framed)}
    tus += 1
    lost = g - w
    if lost:
        miss += 1
        worst.append((src, len(g), len(w), sorted(lost)[:5]))
print(f"TUs {tus}   TUs where a GATE record position is NOT in the WIDE set: {miss}")
for s, ng, nw, ex in worst[:20]:
    print(f"   {s}  gate {ng} wide {nw}  lost@{ex}")
