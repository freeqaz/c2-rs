#!/usr/bin/env python3
"""Print one `.ex` function segment as bytes, split at the `4F 1F` starts.

    work/w-front5/exseg.py <bundle.ex> [index]
"""
import sys

ex = open(sys.argv[1], "rb").read()
idx = int(sys.argv[2]) if len(sys.argv) > 2 else None

starts = []
i = 0
while i + 1 < len(ex):
    if ex[i] == 0x4F and ex[i + 1] == 0x1F:
        starts.append(i)
        i += 2
    else:
        i += 1
bounds = starts + [len(ex)]
for k, s in enumerate(starts):
    if idx is not None and k != idx:
        continue
    e = bounds[k + 1]
    print("== segment %d  [%d..%d)  %d B" % (k, s, e, e - s))
    seg = ex[s:e]
    for off in range(0, len(seg), 16):
        chunk = seg[off:off + 16]
        print("  %04x  %-47s  %s" % (
            off,
            " ".join("%02x" % c for c in chunk),
            "".join(chr(c) if 0x20 <= c < 0x7f else "." for c in chunk)))
