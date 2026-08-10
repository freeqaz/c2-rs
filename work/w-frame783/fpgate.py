#!/usr/bin/env python3
"""The SHIPPING framing's own false positives — the control on fp783.py.

#2783 is stated as *"the gate's framing is precise and the relaxed one is not"*.
That is a claim with a number on both sides, and this counts the gate's side.
"""
import sys, os, glob
from frame783 import gate_framed, scan, ex_splits

capdir = sys.argv[1]
for d in sorted(glob.glob(os.path.join(capdir, "*"))):
    done = os.path.join(d, ".done")
    if not os.path.isfile(done):
        continue
    src = open(done).read().strip()
    gl = open(glob.glob(os.path.join(d, "*.gl"))[0], "rb").read()
    ex = open(glob.glob(os.path.join(d, "*.ex"))[0], "rb").read()
    segs = set(ex_splits(ex))
    bad = [(p, v, pv) for p, v, pv in scan(gl, gate_framed) if v not in segs]
    if not bad:
        continue
    print(f"{src}   segments {len(segs)}   NOT-A-SPLIT {len(bad)}")
    for p, v, pv in bad:
        print(f"    @gl+{p:<7d} value {v:<12d} PREV 0x{pv:04x}  "
              f"bytes {gl[p-7:p+5].hex(' ')}")
