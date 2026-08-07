#!/usr/bin/env python3
"""witness.py — print named cells word for word (board #950).

Lane w-alloc3 measurement tooling. **Read-only with respect to `crates/`.**

    witness.py <grid.tsv> <cell> [<cell> …]
"""

import csv
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", ".."))
sys.path.insert(0, os.path.join(ROOT, "scripts"))
import gt_dump  # noqa: E402


def main():
    rows = {r["name"]: r for r in csv.DictReader(open(sys.argv[1]), delimiter="\t")}
    for n in sys.argv[2:]:
        r = rows[n]
        print("== %s   callee %s   caller formals %s   mode %s   %s"
              % (n, r["callee"], r["n"], r["mode"], r["verdict"]))
        for lab, k in (("callee g ", "gbody"), ("RULE BIND", "pred"),
                       ("c2       ", "got")):
            ws = [int(x, 16) for x in r[k].split()] if r[k] else []
            print("   %s  %s" % (lab, "  ".join("%08x" % w for w in ws)))
            for w, t in zip(ws, gt_dump.disasm(ws)):
                print("        %08x  %s" % (w, t))
        print()


main()
