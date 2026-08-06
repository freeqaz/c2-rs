#!/usr/bin/env python3
"""exscan.py — histogram the `.ex` segments that contain the memset selector,
by length, so the short ones (the candidate "emits nothing" bodies) are visible
without reading 9,824 segments.

    exscan.py <file.ex> [--pat HEX] [--under N]
"""
import sys
from exdump import segments, hexs

MEMSET_SEL = bytes.fromhex("338641748 0ad000000".replace(" ", ""))


def main(argv):
    path = argv[1]
    pat = MEMSET_SEL
    under = 200
    a = 2
    while a < len(argv):
        if argv[a] == "--pat":
            pat = bytes.fromhex(argv[a + 1].replace(" ", ""))
            a += 2
        elif argv[a] == "--under":
            under = int(argv[a + 1])
            a += 2
        else:
            raise SystemExit(argv[a])
    ex = open(path, "rb").read()
    hits = [(o, off, s) for (o, off, s) in segments(ex) if pat in s]
    print(f"{len(hits)} segments carry the pattern")
    lens = {}
    for o, off, s in hits:
        lens[len(s)] = lens.get(len(s), 0) + 1
    for L in sorted(lens):
        print(f"  len {L:5d} x{lens[L]}")
    print("\n---- the shortest ----")
    for o, off, s in sorted(hits, key=lambda t: len(t[2]))[:6]:
        print(f"\n-- segment #{o} @0x{off:x} len {len(s)}")
        print(hexs(s))
    print(f"\n(only segments under {under} bytes shown above if any)")


if __name__ == "__main__":
    main(sys.argv)
