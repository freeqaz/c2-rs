#!/usr/bin/env python3
"""kindrule.py — the LOOP-KIND rule, FITTED TO grid3 AFTER IT WAS OPENED.

Lane **w-fenceb**. **UNSCORED, and it must stay that way until a fourth grid.**
This is the same status `w-backedge` gave `R1'` and for the same reason: every
coefficient here was chosen by looking at grid3's residuals, so its 23-of-23 is
a FIT and not a result. `R1` fitted 26 of 28 and then held out 9 of 13.

AND IT CHEATS IN A WAY THE NEXT LANE MUST NOT MISS: `E` and `W` are keyed on the
CELL NAME's first letter (`w`/`f`/`d`/`g`), i.e. the loop kind is supplied from
OUTSIDE THE IL. Nobody has an IL discriminator for `while` versus `for`. Until
one exists this is not a rule the port could implement, it is a statement about
what the missing term IS.

    work/w-fenceb/kindrule.py [grid3.tsv]
"""
import csv
import sys

E = {"w": 1, "f": 0, "d": 0, "g": 1, "z": 0}   # cost of an EXTRA backward reference (a `continue`)
W = {"w": 1, "f": 0, "d": 0, "g": 0, "z": 0}   # a `while` carrying >= 1 break pays one more

def main(argv):
    path = argv[1] if len(argv) > 1 else "work/w-fenceb/g3_o1.tsv"
    rows = list(csv.DictReader(open(path), delimiter="\t"))
    ok = 0
    for r in rows:
        k = r["cell"][0]
        bt, br = int(r["bwd_t"]), int(r["bwd_refs"])
        bu, bc = int(r["bwd_uncond"]), int(r["bwd_cond"])
        brk, nm = int(r["brk"]), int(r["named"])
        base = bt * 1 if (bt and bc and not bu) else (nm + (bt - nm) * (2 if bu else 1))
        pred = base + (br - bt) * E[k] + brk + (W[k] if brk else 0)
        hit = pred == int(r["charge"])
        ok += hit
        print("%-11s charge %2s  pred %2d %s" % (r["cell"], r["charge"], pred, "" if hit else " X"))
    print("fits %d of %d  -- A FIT, NOT A SCORE" % (ok, len(rows)))
    return 0

if __name__ == "__main__":
    sys.exit(main(sys.argv))
