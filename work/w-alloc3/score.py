#!/usr/bin/env python3
"""score.py — the scorecard, and the BODY-LENGTH STRATIFICATION control.

Lane w-alloc3 measurement tooling. **Read-only with respect to `crates/`.**

    score.py <gridA.tsv> <gridH.tsv>

`PREREG.md` P5 registers the `/QXSTALLS` control: `docs/BOARD.md` records a
+76.25 pp read between two populations that turned out to be **body length**
and nothing else. So RULE BIND's hit rate is printed per callee body length
before any of it is believed, on both grids and on their union.

Also printed: the per-axis breakdown, so a rule that survives only on the
family it was written from cannot read as a rule that survives.
"""

import collections
import csv
import sys


def load(path):
    return list(csv.DictReader(open(path), delimiter="\t"))


def table(rows, key, title):
    agg = collections.defaultdict(collections.Counter)
    for r in rows:
        agg[key(r)][r["verdict"]] += 1
    print("  %-22s %6s %6s %6s %8s" % (title, "HIT", "MISS", "o-o-d", "rate"))
    for k in sorted(agg):
        c = agg[k]
        graded = c["HIT"] + c["MISS"]
        rate = ("%.3f" % (c["HIT"] / graded)) if graded else "  —"
        print("  %-22s %6d %6d %6d %8s"
              % (k, c["HIT"], c["MISS"], c["OUT-OF-DOMAIN"], rate))


def main():
    a = load(sys.argv[1])
    h = load(sys.argv[2])
    for name, rows in (("GRID-A (fit)", a), ("GRID-H (HOLDOUT)", h),
                       ("UNION", a + h)):
        c = collections.Counter(r["verdict"] for r in rows)
        graded = c["HIT"] + c["MISS"]
        print("=== %s ===" % name)
        print("  cells %d · in-domain graded %d · HIT %d · **WRONG %d** · "
              "out of domain %d"
              % (len(rows), graded, c["HIT"], c["MISS"], c["OUT-OF-DOMAIN"]))
        for k in sorted(c):
            if k not in ("HIT", "MISS", "OUT-OF-DOMAIN"):
                print("  !! %s %d" % (k, c[k]))
        print()
        print("  P5 — BODY-LENGTH STRATIFICATION (the /QXSTALLS control)")
        table(rows, lambda r: "callee body %s w" % r["gwords"], "bucket")
        print()
        print("  by axis")
        table(rows, lambda r: r["axis"], "axis")
        print()
        print("  by mode")
        table(rows, lambda r: r["mode"], "mode")
        print()
        ood = collections.Counter(r["why"] for r in rows
                                  if r["verdict"] == "OUT-OF-DOMAIN")
        if ood:
            print("  refusal clauses")
            for k, v in sorted(ood.items(), key=lambda kv: -kv[1]):
                print("    %-38s %d" % (k, v))
        print()

    print("=== THE INCUMBENT, on the same cells ===")
    graded = [r for r in a + h if r["verdict"] in ("HIT", "MISS")]
    wrong = sum(1 for r in graded if r["verdict"] == "MISS")
    print("  today's shipped answer is a REFUSAL: right 0 · **wrong 0** · "
          "refused %d of %d" % (len(graded), len(graded)))
    print("  RULE BIND:                          right %d · **wrong %d** · "
          "refused 0 of %d" % (len(graded) - wrong, wrong, len(graded)))
    print()
    print("  a refusal is never wrong, so the incumbent WINS on this "
          "population by %d." % wrong)


main()
