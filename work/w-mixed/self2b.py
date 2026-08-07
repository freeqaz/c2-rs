#!/usr/bin/env python3
"""self2b.py — every `SELF-2B` cell in the world, transcribed from four lanes'
committed grade tables, and the rules scored against it.

**COMPILES NOTHING, FITS NOTHING.** `SELF-2B` is the residual GRID M leaves —
the class where the value is spelled as a PATH through the outer object while
the stores go through a bound reference. It is where every rule on record
disagrees, and the point of this file is to show how small it is, so the next
lane knows it is building a grid rather than reading one.

Sources, each a committed table:
    work/w-spell/fit.out            S-self-2base-*      (via w-ilx/fit.out)
    work/w-spell/holdout_grade.out  H2-self-2base-*     (frozen)
    work/w-ilx/fit.out              X-A-*
    work/w-ilx/holdout_grade.out    V7-bindself-*       (frozen)
    work/w-mixed/grade.out          C-2base-*-selfup    (frozen)
"""

import collections

# (lane, cell, ru, cu, what real c2 did)
CELLS = [
    ("w-spell S", "S-self-2base-r1k1", 1, 1, "prod"),
    ("w-spell S", "S-self-2base-r2k1", 2, 1, "prod"),
    ("w-spell S", "S-self-2base-r3k1", 3, 1, "prod"),
    ("w-spell S", "S-self-2base-r2k2", 2, 2, "prod"),
    ("w-spell S", "S-self-2base-r2k3", 2, 3, "prod"),
    ("w-spell H", "H2-self-2base-r4k1", 4, 1, "prod"),
    ("w-spell H", "H2-self-2base-r1k2", 1, 2, "prod"),
    ("w-spell H", "H2-self-2base-r2k4", 2, 4, "prod"),
    ("w-spell H", "H2-self-2base-r2k5", 2, 5, "const"),
    ("w-spell H", "H2-self-2base-r3k5", 3, 5, "prod"),
    ("w-spell H", "H2-self-2base-r4k5", 4, 5, "prod"),
    ("w-spell H", "H2-self-2base-r3k3", 3, 3, "prod"),
    ("w-ilx X", "X-A-r3k5", 3, 5, "prod"),
    ("w-ilx X", "X-A-r2k4", 2, 4, "prod"),
    ("w-ilx X", "X-A-r1k1", 1, 1, "prod"),
    ("w-ilx V", "V7-bindself-r1k1", 1, 1, "prod"),
    ("w-ilx V", "V7-bindself-r3k4", 3, 4, "prod"),
    ("w-ilx V", "V7-bindself-r3k5", 3, 5, "prod"),
    ("w-ilx V", "V7-bindself-r2k5", 2, 5, "const"),
    ("w-ilx V", "V7-bindself-r4k2", 4, 2, "prod"),
    ("w-mixed M", "C-2base-r2k4-selfup", 2, 4, "prod"),
    ("w-mixed M", "C-2base-r3k5-selfup", 3, 5, "prod"),
]

RULES = [
    ("always-prod", lambda ru, cu: "prod"),
    ("cu<=ru+1  (#892)", lambda ru, cu: "prod" if cu <= ru + 1 else "const"),
    ("cu<=ru+2", lambda ru, cu: "prod" if cu <= ru + 2 else "const"),
    ("clause-1-alone", lambda ru, cu: "prod" if ru > cu else "const"),
    ("always-const", lambda ru, cu: "const"),
]


def main():
    pts = sorted({(c[2], c[3]) for c in CELLS})
    by = collections.defaultdict(set)
    for _l, _n, ru, cu, o in CELLS:
        by[(ru, cu)].add(o)
    print("  SELF-2B, every cell on record")
    print("  %d cells across %d lanes | %d DISTINCT (ru,cu) points: %s"
          % (len(CELLS), len({c[0].split()[0] for c in CELLS}), len(pts),
             ", ".join("%d/%d" % p for p in pts)))
    dis = [k for k, v in by.items() if len(v) > 1]
    print("  points where two lanes DISAGREE on the obj: %s"
          % (dis or "none — the class is self-consistent"))
    print("\n  rule                right  WRONG   wrong cells")
    print("  " + "-" * 76)
    for name, fn in RULES:
        bad = [c for c in CELLS if fn(c[2], c[3]) != c[4]]
        print("  %-18s %5d %6d   %s"
              % (name, len(CELLS) - len(bad), len(bad),
                 ", ".join(c[1] for c in bad) or "-"))
    print("\n  the shipped refusal    0      0   (refuses all %d)" % len(CELLS))
    print("\n  `cu<=ru+2` fits this class and is 17/31 and 17/29 on GRID M's"
          "\n  SELF-1B and LOAD, so it is a CLAUSE and not a rule — and it is"
          "\n  scored here, never proposed.  No successor is fitted on the cells"
          "\n  that refuted its predecessor (w-ilx's standing instruction).")
    print("\n  The axes NONE of the four lanes varied, which is what a SELF-2B"
          "\n  grid owes first:"
          "\n    * the bind's own DISPLACEMENT (all five families bind at one)"
          "\n    * the DEPTH of the value's path (`&s->mid` vs `&s->mid.in1`)"
          "\n    * whether the path's TAIL agrees with the store's"
          "\n    * cu = ru+2 AND cu = ru+3 in the same family (only w-spell H"
          "\n      and GRID M reach either, and neither reaches both)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
