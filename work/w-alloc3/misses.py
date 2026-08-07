#!/usr/bin/env python3
"""misses.py — disassemble GRID-H's refutation cells, word for word.

Lane w-alloc3 measurement tooling. **Read-only with respect to `crates/`.**

    misses.py <grid.tsv> [<grid.tsv> …]

Board **#950**: the bytes are printed beside every verdict. A refutation that
quotes only a count is a refutation nobody can re-derive, and this lane's whole
output is a refutation.
"""

import csv
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", ".."))
sys.path.insert(0, os.path.join(ROOT, "scripts"))
import gt_dump  # noqa: E402


def show(label, hexwords):
    ws = [int(x, 16) for x in hexwords.split()] if hexwords else []
    txt = gt_dump.disasm(ws) if ws else []
    print("      %-11s %s" % (label, "  ".join("%08x" % w for w in ws)))
    for w, t in zip(ws, txt):
        print("          %08x  %s" % (w, t))


def main():
    for path in sys.argv[1:]:
        rows = list(csv.DictReader(open(path), delimiter="\t"))
        bad = [r for r in rows if r["verdict"] == "MISS"]
        print("=== %s — %d of %d cells MISS ===" % (path, len(bad), len(rows)))
        for r in bad:
            print()
            print("  %s   axis %s   callee %s   caller formals %s   mode %s"
                  % (r["name"], r["axis"], r["callee"], r["n"], r["mode"]))
            show("callee g", r["gbody"])
            show("RULE BIND", r["pred"])
            show("c2", r["got"])
        print()


main()
