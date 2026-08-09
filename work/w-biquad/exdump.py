#!/usr/bin/env python3
"""w-biquad — split a captured `.ex` on the port's `4F 1F` anchor and hexdump
each segment, 16 bytes to a line with the segment-relative offset.

Scratch instrument. The split is the SAME anchor `IlBundle::ex_segment_count`
uses (`docs/ROADMAP.md` §10.11), so a segment index here is the segment index
the port's reader sees; the census's `4C 4F 11` anchor is a different split and
this tool deliberately does not implement it.
"""
import sys

ANCHOR = bytes([0x4F, 0x1F])


def segments(buf: bytes):
    idx, out = [], []
    i = 0
    while True:
        j = buf.find(ANCHOR, i)
        if j < 0:
            break
        idx.append(j)
        i = j + 1
    for n, s in enumerate(idx):
        e = idx[n + 1] if n + 1 < len(idx) else len(buf)
        out.append((s, buf[s:e]))
    return out


def main():
    buf = open(sys.argv[1], "rb").read()
    want = int(sys.argv[2]) if len(sys.argv) > 2 else None
    segs = segments(buf)
    print(f"{len(buf)} B, {len(segs)} segments on 4F 1F")
    for n, (off, seg) in enumerate(segs):
        if want is not None and n != want:
            continue
        print(f"-- segment {n}  file@0x{off:04x}  {len(seg)} B")
        for k in range(0, len(seg), 16):
            row = seg[k:k + 16]
            hexs = " ".join(f"{b:02x}" for b in row)
            print(f"   {k:04x}  {hexs}")


main()
