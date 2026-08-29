#!/usr/bin/env python3
"""w-dagprice: THE ARTIFACT TEST FOR THIS LANE'S OWN RANKING.

Registered in work/w-dagprice/PREREG.md §2 before the ranking existed:

  "my ranking of [dag] read targets is an artifact if the rank order is
   predicted by a property of the binary (function byte size, band span, arm
   count, callee count, hop distance) rather than by a named, cited downstream
   consumer ... I register |rho| >= 0.7 against size as the artifact threshold."

#3505 is six for six on lanes that moved a number by constructing one, and
MEMORY's "ranking instruments measure themselves" is four for four.  This runs
the check and prints the answer whichever way it falls.
"""
import os
import collections

EXP = os.path.expanduser("~/ghidra-projects/export/c2")
REPO = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", ".."))

size = {}
with open(os.path.join(EXP, "functions.tsv")) as f:
    next(f)
    for line in f:
        p = line.split("\t")
        size[int(p[0], 16)] = int(p[1])

cover = {}
with open(os.path.join(REPO, "docs/whitebox/ref/FUNCS.tsv")) as f:
    for line in f:
        if line.startswith("#"):
            continue
        p = line.rstrip("\n").split("\t")
        if p[0] == "addr":
            continue
        cover[int(p[0], 16)] = p[7]

DAGC = [a for a in size if 0x10B3219F <= a <= 0x10B3433F]
rd3 = sum(size[a] for a in DAGC if cover.get(a) == "none")
rd8 = sum(size[a] for a in size if 0x10BE663E <= a <= 0x10BE717F)

# rank -> (name, named bodies' total bytes).  The rank is fixed by what each row
# unblocks (WB_DAGPRICE_FINDINGS.md §5), and was written before this ran.
ROWS = [
    (1, "RD1 edge+0x19 bit 1, the -2 cell's discriminator", 114 + 116 + 380),
    (2, "RD2 DAG build FUN_10b328da",                        2231),
    (3, "RD3 the 31 unread bodies of the dag.c band",        rd3),
    (4, "RD4 the mid-level (mode 1) pass",                   700),
    (5, "RD5 the re-schedule iteration FUN_10c1bdff",        483),
    (6, "RD6 the DAG's two client callbacks",                116 + 158),
    (7, "RD7 0x10be663e's cluster (a denominator fix)",      1197 + 223 + 229),
    (8, "RD8 the 29 gap neighbours above the band",          rd8),
]


def spearman(xs, ys):
    def ranks(v):
        order = sorted(range(len(v)), key=lambda i: v[i])
        r = [0.0] * len(v)
        i = 0
        while i < len(order):
            j = i
            while j + 1 < len(order) and v[order[j + 1]] == v[order[i]]:
                j += 1
            avg = (i + j) / 2.0 + 1
            for k in range(i, j + 1):
                r[order[k]] = avg
            i = j + 1
        return r
    rx, ry = ranks(xs), ranks(ys)
    n = len(xs)
    mx, my = sum(rx) / n, sum(ry) / n
    num = sum((a - mx) * (b - my) for a, b in zip(rx, ry))
    den = (sum((a - mx) ** 2 for a in rx) * sum((b - my) ** 2 for b in ry)) ** 0.5
    return num / den if den else 0.0


print("rank  bytes  row")
for r, name, b in ROWS:
    print("%4d %6d  %s" % (r, b, name))
rho = spearman([r for r, _, _ in ROWS], [b for _, _, b in ROWS])
print("\nSpearman rho(rank, named-body bytes) = %+.3f  over n=%d" % (rho, len(ROWS)))
print("registered artifact threshold: |rho| >= 0.700")
print("VERDICT: %s" % ("ARTIFACT — the ranking is size-shaped, publish it as such"
                       if abs(rho) >= 0.7 else
                       "NOT FIRED — the ranking is not predicted by candidate size"))
print("\nnote the sign: a size-driven ranking would give rho NEGATIVE "
      "(big first = rank 1); rho positive means the plan puts SMALL reads first.")
