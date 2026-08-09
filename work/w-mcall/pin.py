#!/usr/bin/env python3
"""Split a captured `.ex` on the `4F 1F` gate marker and print one segment as a
Rust byte array, ready to paste as a pinned test segment.

Usage: pin.py FILE.ex [INDEX]
"""
import sys

b = open(sys.argv[1], "rb").read()
starts = [i for i in range(len(b) - 1) if b[i] == 0x4F and b[i + 1] == 0x1F]
segs = []
for k, s in enumerate(starts):
    e = starts[k + 1] if k + 1 < len(starts) else len(b)
    segs.append(b[s:e])

if len(sys.argv) < 3:
    for k, s in enumerate(segs):
        print(f"[{k}] {len(s)} B  {s[:8].hex(' ')} … {s[-8:].hex(' ')}")
    sys.exit(0)

s = segs[int(sys.argv[2])]
out = []
for i in range(0, len(s), 15):
    out.append("        " + " ".join(f"0x{x:02X}," for x in s[i : i + 15]))
print("\n".join(out))
