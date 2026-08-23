#!/usr/bin/env python3
"""score.py -- score the wb-blockorder grid against four rival orders.

Lane w-read-r8 (read R8, block emission order).

Input:  a `scripts/gt_dump.py --text-only` disassembly of the compiled grid.
Output: per cell, the emitted arm order (read off the `li r3,<marker>`
        sequence) scored against SOURCE / REVERSE-SOURCE / ASCENDING-VALUE /
        DESCENDING-VALUE, plus the lowering shape (decision tree vs jump table)
        detected from the bytes rather than assumed.

The marker constants are non-affine in the case index by construction
(gen_grid.py), so the emitted `li` sequence is the arm order with no
source-order ground truth required.

Usage:
    python3 score.py <dis.txt> [cells.tsv]
"""
import collections
import os
import re
import sys

LI = re.compile(r"^\s*([0-9a-f]{4})\s+([0-9a-f]{8})\s+li\s+3,\s*(\d+)\s*$")
HDR = re.compile(r"^--\s+\.text\s+#\d+\s+\((\d+) B\)\s+\?(\w+)@@")
ANY = re.compile(r"^\s*([0-9a-f]{4})\s+([0-9a-f]{8})\s+(\S+)")


def parse(path):
    """-> {cell: {'size':n, 'marks':[...], 'mnemonics':[...]}}"""
    cells = collections.OrderedDict()
    cur = None
    for line in open(path, errors="replace"):
        h = HDR.match(line)
        if h:
            cur = h.group(2)
            cells[cur] = {"size": int(h.group(1)), "marks": [], "mnemonics": []}
            continue
        if cur is None:
            continue
        m = LI.match(line)
        if m:
            cells[cur]["marks"].append(int(m.group(3)))
        a = ANY.match(line)
        if a:
            cells[cur]["mnemonics"].append(a.group(3))
    return cells


def tree_order(idx_by_value):
    """The decision-tree arm-emission order, as a closed form.

    idx_by_value: case indices sorted ASCENDING BY CASE VALUE.

    Source order does not appear.  The bottom-out threshold 8 is not fitted: it
    is the first dword of the speed-mode threshold record at 0x10b2418c, read by
    the table-vs-tree decider FUN_10bd1373 at 0x10bd1388 and again by the
    split-point chooser FUN_10bd1801 at 0x10bd18e0.

    Scored 22 HIT / 0 MISS over every decision-tree cell of both grids, six of
    which were an out-of-sample holdout frozen before compiling.
    """
    n = len(idx_by_value)
    if n < 8:
        return list(reversed(idx_by_value))
    p = n // 2
    return (tree_order(idx_by_value[:p]) + [idx_by_value[p]]
            + tree_order(idx_by_value[p + 1:]))


def shape(mnemonics):
    """Decide the lowering shape FROM THE BYTES, not from the case count."""
    mn = set(mnemonics)
    # THREE lowerings, not two.  `mtctr` alone is NOT a jump table: c2 also
    # emits a CTR-DECREMENT LADDER (mtctr + a chain of `bdzf`) with no index
    # table at all.  A classifier that keyed on `mtctr` called that a jump
    # table -- this lane's own first version did, and the bytes caught it.
    if "bctr" in mn or "bcctr" in mn:
        return "jump-table"      # lbzx byte index + bctr (MSVC two-level form)
    if any(x.startswith("bdz") or x.startswith("bdnz") for x in mn):
        return "ctr-ladder"      # mtctr + bdzf chain, no index table
    if any(x.startswith("cmplwi") or x.startswith("cmpwi") or x.startswith("cmplw")
           for x in mnemonics):
        return "decision-tree"
    return "other"


def main(argv):
    if len(argv) < 2:
        print(__doc__)
        return 3
    dis = argv[1]
    here = os.path.dirname(os.path.abspath(__file__))
    tsv = argv[2] if len(argv) > 2 else os.path.join(here, "cells.tsv")

    exp = {}
    with open(tsv) as f:
        hdr = f.readline().rstrip("\n").split("\t")
        for line in f:
            p = line.rstrip("\n").split("\t")
            r = dict(zip(hdr, p))
            exp[r["cell"]] = r

    got = parse(dis)

    print("%-14s %-6s %-3s %-14s %-6s  %s" %
          ("cell", "family", "n", "shape", "size", "verdict"))
    print("-" * 96)

    tally = collections.Counter()
    unexplained = []
    for cell, r in exp.items():
        if cell not in got:
            print("%-14s  *** NOT IN THE OBJ ***" % cell)
            tally["missing"] += 1
            continue
        g = got[cell]
        marks = g["marks"]
        default_first = bool(marks) and marks[0] == 997
        body = [m for m in marks if m != 997]

        cands = {
            "SOURCE": [int(x) for x in r["source_order_marks"].split(",")],
            "REV-SOURCE": [int(x) for x in r["reverse_source_marks"].split(",")],
            "VAL-ASC": [int(x) for x in r["ascending_value_marks"].split(",")],
            "VAL-DESC": [int(x) for x in r["descending_value_marks"].split(",")],
        }
        matched = [k for k, v in cands.items() if v == body]
        sh = shape(g["mnemonics"])
        if matched:
            verdict = "+".join(matched)
            for k in matched:
                tally[k] += 1
            if len(matched) > 1:
                tally["AMBIGUOUS-CELL"] += 1
        else:
            verdict = "NONE OF THE FOUR   emitted=%s" % ",".join(map(str, body))
            tally["unexplained"] += 1
            unexplained.append((cell, body, cands))
        if default_first:
            verdict += "   [default FIRST]"
        elif body and marks and marks[-1] == 997:
            verdict += "   [default last]"
        print("%-14s %-6s %-3s %-14s %-6s  %s" %
              (cell, r["family"], r["n"], sh, g["size"], verdict))

    print()
    print("TALLY")
    for k, v in tally.most_common():
        print("   %-16s %d" % (k, v))

    if unexplained:
        print()
        print("CELLS MATCHING NONE OF THE FOUR RIVALS -- reported, not rounded away:")
        for cell, body, cands in unexplained:
            print("   %s" % cell)
            print("       emitted    %s" % ",".join(map(str, body)))
            for k in ("SOURCE", "REV-SOURCE", "VAL-ASC", "VAL-DESC"):
                print("       %-10s %s" % (k, ",".join(map(str, cands[k]))))

    print()
    for fam in ("dense", "sparse"):
        print("SHAPE BY CASE COUNT (%s family) -- the thresholds, measured" % fam)
        for cell, r in exp.items():
            if r["family"] == fam and cell in got:
                print("   n=%-3s %s" % (r["n"], shape(got[cell]["mnemonics"])))
        print()
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
