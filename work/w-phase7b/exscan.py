#!/usr/bin/env python3
"""Histogram every `.ex` function segment's OPT WORD and its first bytes.

The question this exists to answer: is there any per-BODY field in `.ex` that
separates the bodies c2 EMITS from the ones it discards? `vec.cpp` is the sharp
cell — 811 segments, 2 emitted — so a field that splits 2 : 809 is a candidate
and anything uniform is not.

    work/w-phase7b/exscan.py <bundle.ex>
"""
import sys
from collections import Counter


def segs(ex):
    out = []
    i = 0
    while i + 1 < len(ex):
        if ex[i] == 0x4F and ex[i + 1] == 0x1F:
            out.append(i)
            i += 2
        else:
            i += 1
    out.append(len(ex))
    return [(out[k], out[k + 1]) for k in range(len(out) - 1)]


def opt_word(ex, s):
    if s + 3 > len(ex):
        return None
    b = ex[s + 2]
    if b == 0x80:
        if s + 7 > len(ex):
            return None
        return int.from_bytes(ex[s + 3:s + 7], "little")
    if b < 0x80:
        return b
    return None


def main():
    ex = open(sys.argv[1], "rb").read()
    ss = segs(ex)
    print("%s: %d B, %d segments" % (sys.argv[1], len(ex), len(ss)))
    c = Counter(opt_word(ex, a) for a, _ in ss)
    print("  opt word histogram:")
    for v, n in c.most_common(12):
        print("     %-12s %d" % (hex(v) if v is not None else "None", n))
    # the byte immediately after the opt word — the next record field
    def after(a):
        b = ex[a + 2]
        q = a + 7 if b == 0x80 else a + 3
        return ex[q:q + 3].hex()
    c2 = Counter(after(a) for a, _ in ss)
    print("  3 bytes after the opt word:")
    for v, n in c2.most_common(12):
        print("     %-12s %d" % (v, n))
    lens = Counter()
    for a, b in ss:
        lens[b - a] += 1
    print("  distinct segment lengths: %d; the 6 rarest:" % len(lens))
    for v, n in sorted(lens.items(), key=lambda kv: kv[1])[:6]:
        print("     len %-8d x%d" % (v, n))


main()
