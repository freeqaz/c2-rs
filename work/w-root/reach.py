#!/usr/bin/env python3
"""reach.py — TU REACH: `|{per-TU exact} n B^C|`, by NAME, never by count.

Board #345 is the standing warning this exists for: an emit-model rung moved
per-TU exact by +321 and TU reach by exactly 0, and it read as a win until
somebody intersected the two sets.  So every model this lane grades is also
intersected with the per-TU `B^C` membership `c2rs gap --factors-tsv` publishes,
and the result is printed as GAINED and LOST name lists.

    usage: reach.py <exact.json> <factors.tsv> <population.txt> <base-model>

`population.txt` bounds the join: `B^C` is 151 over the 871 GRADED TUs, but a
model scored over 650 or 200 of the 850 can only be credited on the TUs it
actually saw, so the denominator printed is `|B^C n population|` and not 151.
That is board #302's correction (on its join `B^C` was 145, not 151) applied
mechanically instead of remembered.

stdlib only.
"""
import json
import sys


def main():
    exactp, facp, popp, base = sys.argv[1:5]
    ex = json.load(open(exactp))
    pop = set(l.strip() for l in open(popp) if l.strip())
    bc = set()
    graded = set()
    for l in open(facp):
        if l.startswith("#") or not l.strip():
            continue
        f = l.rstrip("\n").split("\t")
        graded.add(f[0])
        if f[3] == "1" and f[4] == "1":
            bc.add(f[0])
    bcp = bc & pop
    print("population              %d" % len(pop))
    print("graded TUs in factors   %d   (B^C over all graded = %d)"
          % (len(graded), len(bc)))
    print("population not graded   %d" % len(pop - graded))
    print("DENOMINATOR |B^C n pop| %d" % len(bcp))
    print()
    b = set(ex[base]) & pop
    print("%-16s exact %4d   reach %4d" % (base, len(b), len(b & bcp)))
    for m in sorted(ex):
        if m == base:
            continue
        s = set(ex[m]) & pop
        g = (s & bcp) - (b & bcp)
        l = (b & bcp) - (s & bcp)
        print("%-16s exact %4d   reach %4d   REACH gained %d  LOST %d"
              % (m, len(s), len(s & bcp), len(g), len(l)))
        for x in sorted(g):
            print("     reach gain  %s" % x)
        for x in sorted(l):
            print("     REACH LOST  %s" % x)


if __name__ == "__main__":
    main()
