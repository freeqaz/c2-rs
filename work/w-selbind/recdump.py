#!/usr/bin/env python3
"""Hex-dump the `.gl` neighbourhood of a NAME, and say where the split point it
would have to carry is actually spelled.

    work/w-selbind/recdump.py <bundle.gl> <name> [ex-split-point]
"""
import sys


def main():
    gl = open(sys.argv[1], "rb").read()
    name = sys.argv[2].encode()
    want = int(sys.argv[3]) if len(sys.argv) > 3 else None
    at = gl.find(name)
    while at >= 0:
        lo = max(0, at - 32)
        hi = min(len(gl), at + len(name) + 24)
        print("== %s @%d  (prev byte %02x)" % (sys.argv[2], at, gl[at - 1]))
        for i in range(lo, hi, 16):
            row = gl[i:i + 16]
            txt = "".join(chr(c) if 0x20 <= c < 0x7f else "." for c in row)
            print("   %6d  %-47s  %s" % (i, " ".join("%02x" % c for c in row), txt))
        at = gl.find(name, at + 1)
    if want is not None:
        pat = b"\x80" + want.to_bytes(4, "little")
        hits = []
        i = gl.find(pat)
        while i >= 0:
            hits.append(i)
            i = gl.find(pat, i + 1)
        print("\n`80 <LE32 %d>` occurs at %s" % (want, hits))
        for h in hits:
            lo = max(0, h - 24)
            print("   context @%d:" % h)
            for i in range(lo, min(len(gl), h + 24), 16):
                row = gl[i:i + 16]
                txt = "".join(chr(c) if 0x20 <= c < 0x7f else "." for c in row)
                print("      %6d  %-47s  %s"
                      % (i, " ".join("%02x" % c for c in row), txt))


main()
