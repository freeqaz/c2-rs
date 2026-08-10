#!/usr/bin/env python3
"""The WIDE framing's false positives, per TU and per offset — the safety case.

A `false positive` is a framed `80 <LE32>` whose value is not an `.ex` `4F 1F`
split point. Under the incumbent 1:1 contract it only costs a refusal; under a
SELECTIVE binding it is a name bound to an address that is not a body, which is
the direction that emits.
"""
import sys, os, glob, json
from frame783 import gate_framed, wide_framed, scan, ex_splits

capdir = sys.argv[1]
for d in sorted(glob.glob(os.path.join(capdir, "*"))):
    done = os.path.join(d, ".done")
    if not os.path.isfile(done):
        continue
    src = open(done).read().strip()
    gl = open(glob.glob(os.path.join(d, "*.gl"))[0], "rb").read()
    ex = open(glob.glob(os.path.join(d, "*.ex"))[0], "rb").read()
    segs = set(ex_splits(ex))
    hits = scan(gl, wide_framed)
    bad = [(p, v, pv) for p, v, pv in hits if v not in segs]
    if not bad:
        continue
    print(f"{src}   segments {len(segs)}  wide records {len(hits)}  "
          f"NOT-A-SPLIT {len(bad)}")
    for p, v, pv in bad:
        near = min((abs(v - s), s) for s in segs) if segs else (0, 0)
        print(f"    @gl+{p:<7d} value {v:<9d} PREV 0x{pv:04x}  "
              f"nearest split {near[1]} (delta {near[0]})  "
              f"bytes {gl[p-7:p+5].hex(' ')}")
