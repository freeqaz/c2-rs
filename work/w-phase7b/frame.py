#!/usr/bin/env python3
"""How many `.gl` framed defined records does each FRAMING variant find, and do
the offsets it finds equal the `.ex` `4F 1F` split points exactly?

`codec::gl_offset_framed`'s doc says it locates the body-start offset field *"by
position within the record, not by what its value happens to be"*. It does that
for the offset field itself and **not** for the field in front of it: the frame
is `80 <LE32 PREV> 00 00` and the test pins `gl[o-5] == 0x10`, so `PREV` is
required to lie in `[0x1000, 0x10FF]`. This script asks what each relaxation of
that one byte finds.

    work/w-phase7b/frame.py <bundle.gl> <bundle.ex>
"""
import sys
from collections import Counter


def ex_starts(ex):
    out = []
    i = 0
    while i + 1 < len(ex):
        if ex[i] == 0x4F and ex[i + 1] == 0x1F:
            out.append(i)
            i += 2
        else:
            i += 1
    return out


VARIANTS = {
    # name: (predicate on (gl, o))
    "shipping  80 <PREV=0x10XX> 00 00": lambda g, o: (
        g[o - 7] == 0x80 and g[o - 5] == 0x10 and g[o - 4] == 0
        and g[o - 3] == 0 and g[o - 2] == 0 and g[o - 1] == 0),
    "PREV < 0x10000": lambda g, o: (
        g[o - 7] == 0x80 and g[o - 4] == 0 and g[o - 3] == 0
        and g[o - 2] == 0 and g[o - 1] == 0),
    "PREV any 32-bit": lambda g, o: (
        g[o - 7] == 0x80 and g[o - 2] == 0 and g[o - 1] == 0),
    "no frame at all": lambda g, o: True,
}


def main():
    gl = open(sys.argv[1], "rb").read()
    ex = open(sys.argv[2], "rb").read()
    st = ex_starts(ex)
    sset = set(st)
    print("%s: %d B .gl; .ex %d segments" % (sys.argv[1], len(gl), len(st)))
    for name, pred in VARIANTS.items():
        offs = []
        prevs = Counter()
        p = 7
        while p + 5 <= len(gl):
            if gl[p] == 0x80 and pred(gl, p):
                v = int.from_bytes(gl[p + 1:p + 5], "little")
                offs.append(v)
                prevs[int.from_bytes(gl[p - 6:p - 2], "little")] += 1
                p += 5
                continue
            p += 1
        hit = sum(1 for v in offs if v in sset)
        eq = offs == st
        print("  %-36s found %5d  of which are .ex starts %5d  ==.ex-order? %s"
              % (name, len(offs), hit, eq))
        if name.startswith("PREV < "):
            print("     PREV histogram (top 8): %s"
                  % [(hex(k), n) for k, n in prevs.most_common(8)])
            miss = [v for v in offs if v not in sset]
            print("     offsets found that are NOT .ex starts: %d %s"
                  % (len(miss), [hex(m) for m in miss[:8]]))
            missing = [s for s in st if s not in set(offs)]
            print("     .ex starts with NO framed record: %d %s"
                  % (len(missing), missing[:8]))


main()
