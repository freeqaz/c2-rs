#!/usr/bin/env python3
"""objdiff.py — the byte compare behind this lane's headline cell.

Two GRID M cells whose value expressions denote the SAME ADDRESS:

    B-2base-r2k4          P& q = t->mid.lo;   q.b0 = (int)&q;
    C-2base-r2k4-selfup   P& q = t->mid.lo;   q.b0 = (int)&t->mid;

`&q == &t->mid.lo == &t->mid == t+40` because `lo` is `Q`'s first member. Both
emit `addi rX,3,40`; the register differs. The compare is the project's own —
byte-exact with the COFF `TimeDateStamp` (file offset 4..8) zeroed.
"""

import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
PAIRS = [("B-2base-r2k4", "C-2base-r2k4-selfup"),
         ("B-2base-r3k5", "C-2base-r3k5-selfup")]


def main():
    for left, right in PAIRS:
        pa = os.path.join(HERE, "gridM", left, "ref.obj")
        pb = os.path.join(HERE, "gridM", right, "ref.obj")
        if not (os.path.exists(pa) and os.path.exists(pb)):
            print("  %s / %s — no obj; run gridm.py --grade" % (left, right))
            continue
        a, b = open(pa, "rb").read(), open(pb, "rb").read()
        d = [i for i in range(min(len(a), len(b))) if a[i] != b[i]]
        d = [i for i in d if not 4 <= i < 8] + \
            ([] if len(a) == len(b) else ["LENGTH"])
        print("== %s  vs  %s" % (left, right))
        print("   sizes %d / %d" % (len(a), len(b)))
        print("   differing bytes with TimeDateStamp (4..8) zeroed: %d" % len(d))
        for i in d:
            print("      off 0x%04x   %02x -> %02x" % (i, a[i], b[i]))
        for tag, p in ((left, pa), (right, pb)):
            dis = os.path.join(os.path.dirname(p), "dis.txt")
            if os.path.exists(dis):
                print("   -- %s" % tag)
                for line in open(dis):
                    print("      %s" % line.rstrip())
        print()
    return 0


if __name__ == "__main__":
    sys.exit(main())
