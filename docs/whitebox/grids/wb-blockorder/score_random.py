#!/usr/bin/env python3
"""score_random.py -- score the block-order rule over the randomized corpus.

Lane w-read-r8.  Usage: score_random.py <dis.txt> [random_cells.tsv]

The rule under test, from ref/P_BLOCKORDER.md 5, stated over CASE VALUES with
no reference to source order:

    jump table / CTR ladder  -> arms in SOURCE order
    decision tree            -> emit(V), V = values ascending:
                                  n < 8 : reverse(V)
                                  else  : emit(V[:p]) ++ [V[p]] ++ emit(V[p+1:]),  p = n//2
"""
import collections
import csv
import os
import re
import sys

LI = re.compile(r"^\s*[0-9a-f]{4}\s+[0-9a-f]{8}\s+li\s+3,\s*(\d+)\s*$")
LIS = re.compile(r"^\s*[0-9a-f]{4}\s+[0-9a-f]{8}\s+(lis|ori|addi)\s")
HDR = re.compile(r"^--\s+\.text\s+#\d+\s+\((\d+) B\)\s+\?(\w+)@@")
ANY = re.compile(r"^\s*[0-9a-f]{4}\s+[0-9a-f]{8}\s+(\S+)")


def emit(idx):
    n = len(idx)
    if n < 8:
        return list(reversed(idx))
    p = n // 2
    return emit(idx[:p]) + [idx[p]] + emit(idx[p + 1:])


def shape(mn):
    s = set(mn)
    if "bctr" in s or "bcctr" in s:
        return "jump-table"
    if any(x.startswith("bdz") or x.startswith("bdnz") for x in s):
        return "ctr-ladder"
    if any(x.startswith("cmp") for x in mn):
        return "decision-tree"
    return "other"


def main(argv):
    dis = argv[1]
    here = os.path.dirname(os.path.abspath(__file__))
    tsv = argv[2] if len(argv) > 2 else os.path.join(here, "random_cells.tsv")

    cells = {}
    cur = None
    for line in open(dis, errors="replace"):
        h = HDR.match(line)
        if h:
            cur = h.group(2)
            cells[cur] = {"marks": [], "mn": [], "wide": 0}
            continue
        if cur is None:
            continue
        m = LI.match(line)
        if m:
            cells[cur]["marks"].append(int(m.group(1)))
        if LIS.match(line):
            cells[cur]["wide"] += 1
        a = ANY.match(line)
        if a:
            cells[cur]["mn"].append(a.group(1))

    tally = collections.Counter()
    fails = []
    for r in csv.DictReader(open(tsv), delimiter="\t"):
        c = r["cell"]
        if c not in cells:
            tally["missing"] += 1
            continue
        g = cells[c]
        vals = [int(x) for x in r["values"].split(",")]
        marks = [int(x) for x in r["marks"].split(",")]
        srcidx = [int(x) for x in r["source_order_idx"].split(",")]
        n = len(vals)
        body = [m for m in g["marks"] if m != 32000]
        sh = shape(g["mn"])

        # A marker only survives as `li r3,imm` when it fits a signed 16-bit
        # field; anything wider becomes lis/ori and this extractor cannot see
        # it.  Such cells are reported UNREADABLE, never scored as a pass.
        if len(body) != n:
            tally["unreadable (%s)" % sh] += 1
            continue

        if sh == "decision-tree":
            order = sorted(range(n), key=lambda k: vals[k])
            pred = [marks[k] for k in emit(order)]
        else:
            pred = [marks[k] for k in srcidx]
        if pred == body:
            tally["HIT %s" % sh] += 1
        else:
            tally["MISS %s" % sh] += 1
            fails.append((c, n, sh, vals, pred, body))

    print("RANDOMIZED CORPUS -- %d cells, seed-deterministic" % sum(tally.values()))
    for k in sorted(tally):
        print("   %-26s %d" % (k, tally[k]))
    hits = sum(v for k, v in tally.items() if k.startswith("HIT"))
    miss = sum(v for k, v in tally.items() if k.startswith("MISS"))
    print()
    print("   SCORED %d   HIT %d   MISS %d" % (hits + miss, hits, miss))
    if fails:
        print()
        print("MISSES -- reported, not rounded away:")
        for c, n, sh, vals, pred, body in fails[:10]:
            print("   %s n=%d %s" % (c, n, sh))
            print("       values %s" % ",".join(map(str, vals)))
            print("       pred   %s" % ",".join(map(str, pred)))
            print("       got    %s" % ",".join(map(str, body)))
    return 1 if miss else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
