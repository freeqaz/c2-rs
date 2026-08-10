#!/usr/bin/env python3
"""What SHAPE do the not-a-split-point framed offsets have, at BOTH widths?

If they are separable by a property of the record rather than by "we checked it
against `.ex`", that is a reader repair. If they are not, the only sound answer
is the whole-TU refusal `Bindings::selective` clause 1 already makes.
"""
import sys, os, glob
from collections import Counter
from frame783 import gate_framed, wide_framed, scan, ex_splits

capdir = sys.argv[1]
top = Counter()
topgood = Counter()
n_bad = n_good = 0
maxgood = 0
for d in sorted(glob.glob(os.path.join(capdir, "*"))):
    done = os.path.join(d, ".done")
    if not os.path.isfile(done):
        continue
    gl = open(glob.glob(os.path.join(d, "*.gl"))[0], "rb").read()
    ex = open(glob.glob(os.path.join(d, "*.ex"))[0], "rb").read()
    segs = set(ex_splits(ex))
    for p, v, pv in scan(gl, wide_framed):
        if v in segs:
            n_good += 1
            topgood[v >> 24] += 1
            maxgood = max(maxgood, v)
        else:
            n_bad += 1
            top[v >> 24] += 1
print(f"WIDE framed records: on-a-split {n_good}, not-a-split {n_bad}")
print(f"  top byte of the offset value, NOT-a-split: {dict(sorted(top.items()))}")
print(f"  top byte of the offset value, ON-a-split:  {dict(sorted(topgood.items()))}")
print(f"  largest on-a-split offset: {maxgood} (0x{maxgood:x})")
