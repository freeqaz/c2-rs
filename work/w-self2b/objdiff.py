#!/usr/bin/env python3
"""objdiff.py — the byte compare behind this lane's two witnesses.

The compare is the project's own: byte-exact with the COFF `TimeDateStamp`
(file offset 4..8) zeroed.

    THE TIGHTER PAIR   Z2-r2k4  vs  Z6-r2k4
        W& k = d->core.u0;                       k.m0 = (int)&k;   -> const
        W& k = d->core.u0;  W& j = d->core.u0;   j.m0 = (int)&k;   -> prod

    Both spell the value as a BIND'S OWN NAME.  Both write the producer's
    stores through a BIND.  `&k == &j == &d->core.u0 == d+48`.  The whole
    source difference is a SECOND NAME for an object that already had one, and
    that name is never used to compute anything — and c2 swaps two registers.
    This is a strictly tighter witness than w-mixed's (board #1217), which
    varied the value's spelling between a bind name and a path.

    THE ASYMMETRY PAIR Z3-r2k4  vs  Z5-r2k4
        W& k = d->core.u0;  k.m0          = (int)&d->core.u0;  -> prod
        W& k = d->core.u0;  d->core.u0.m0 = (int)&k;           -> const

    The SAME two spellings, swapped between the value and the store
    designator.  The answer is not symmetric in them, which is what refutes
    H-2X.

SHIPS NOTHING.
"""

import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
PAIRS = [
    ("Z2-r2k4", "Z6-r2k4", "the TIGHTER pair — a second name, used nowhere"),
    ("Z2-r3k5", "Z6-r3k5", "its (3,5) twin"),
    ("Z3-r2k4", "Z5-r2k4", "the ASYMMETRY pair — the two spellings swapped"),
    ("Z3-r3k5", "Z5-r3k5", "its (3,5) twin"),
]


def main():
    for left, right, why in PAIRS:
        pa = os.path.join(HERE, "gridZ", left, "ref.obj")
        pb = os.path.join(HERE, "gridZ", right, "ref.obj")
        if not (os.path.exists(pa) and os.path.exists(pb)):
            print("  %s / %s — no obj; run gridz.py --grade" % (left, right))
            continue
        a, b = open(pa, "rb").read(), open(pb, "rb").read()
        d = [i for i in range(min(len(a), len(b))) if a[i] != b[i]]
        d = [i for i in d if not 4 <= i < 8] + \
            ([] if len(a) == len(b) else ["LENGTH"])
        print("== %s  vs  %s   (%s)" % (left, right, why))
        print("   sizes %d / %d" % (len(a), len(b)))
        print("   differing bytes with TimeDateStamp (4..8) zeroed: %d" % len(d))
        for i in d:
            print("      off 0x%04x   %02x -> %02x" % (i, a[i], b[i]))
        for tag, p in ((left, pa), (right, pb)):
            dis = os.path.join(os.path.dirname(p), "dis.txt")
            if os.path.exists(dis):
                print("   -- %s" % tag)
                for line in open(dis):
                    if line.strip():
                        print("      %s" % line.rstrip())
        print()
    return 0


if __name__ == "__main__":
    sys.exit(main())
