#!/usr/bin/env python3
"""misses.py — every rule's MISS CELLS by name, out of the frozen column and the
graded answers.

    misses.py            reads pred.tsv and grade.out

`gridx.py --grade` prints a count per rule and H-CHAIN's own miss cells. A count
is not a finding: two rules wrong on the same number of cells can be wrong on
DIFFERENT cells, and which cells is the whole content of a refutation (board
#836's `slwi` row, #1227's `Z5`). Read-only; scores nothing new.
"""
import os
import re
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
PRED = os.path.join(HERE, "pred.tsv")
GRADE = os.path.join(HERE, "grade.out")

hdr, rows = None, {}
for line in open(PRED):
    if line.startswith("#"):
        continue
    f = line.rstrip("\n").split("\t")
    if hdr is None:
        hdr = f
        continue
    rows[f[0]] = dict(zip(hdr, f))

obs = {}
for line in open(GRADE):
    m = re.match(r"^\s{4}(\S+)\s+(prod|const|OOR.*|COMPILE-FAILED)\s*$", line)
    if m:
        obs[m.group(1)] = m.group(2)

rules = [c for c in hdr if c not in
         ("cell", "fam", "class", "ru", "cu", "domain", "sha256_src")]
order = [n for n in rows if rows[n]["domain"] == "in" and obs.get(n) in ("prod", "const")]
print("in-domain graded cells: %d" % len(order))
for r in rules:
    miss = sorted(n for n in order if rows[n][r] != obs[n])
    print("\n  %-12s %2d WRONG" % (r, len(miss)))
    for n in miss:
        print("      %-12s %-14s ru=%s cu=%s   said %-5s  c2 says %s"
              % (n, rows[n]["class"], rows[n]["ru"], rows[n]["cu"],
                 rows[n][r], obs[n]))
print("\n  refusal       0 WRONG   (it emits nothing)")
sys.exit(0)
