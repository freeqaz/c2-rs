#!/usr/bin/env python3
"""rivals.py — GRID M re-partitioned by the axis that decided it, and every
already-published rule scored against the table that now exists.

**COMPILES NOTHING, FITS NOTHING.** It re-reads `grade.out` and `pred.tsv`,
which were produced by `gridm.py --grade` after `gridm.py --freeze` was
committed at `efdcf6e6`. Its only purpose is to let the next lane know which
rules are dead without spending a grid — w-ilx's `rivals.py` is the precedent
and its caveat is taken verbatim:

    A rule scoring well on a table it was scored against AFTER the table existed
    has the standing RULE W2 had at 388 of 388, which is none.

THE RE-PARTITION  (this lane's §4)
-----------------------------------
GRID M was frozen with ONE structural axis called `2base`, and grading split it
in two. The splitter is **the source spelling of the value**, and both spellings
denote the SAME ADDRESS:

    LOADSPELL   the value is the bound name itself, `(int)&q`
                -> `.ex` carries a bare `B9 <tok> <TYPE>` and NO offset-adds
    PATHSPELL   the value is the path through the outer object, `(int)&t->mid`
                -> `.ex` carries `B9 <tok> <TYPE> 33 <int> <varint 40> 27 <PTR>`

`&q == &t->mid.lo == &t->mid == t+40` in every cell of the pair, the emitted
producer is `addi rX,3,40` in both, and the two objs differ in **8 bytes, every
one a register field** (`work/w-mixed/objdiff.out`). These are w-ilx's `LOAD`
and `SELF-2B` classes (board #909), and the frozen `2base` column mixed them.
"""

import os
import re
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
GRADE = os.path.join(HERE, "grade.out")
PRED = os.path.join(HERE, "pred.tsv")

ROW = re.compile(r"^  (\S+)\s+(1base|2base)\s+(\S+)\s+(\d+)\s+(\S+)\s+"
                 r"(prod|const)\s*(.*)$")


def klass(cell, base, target):
    """The IL class each cell lands in, named as w-ilx named them (#909).

    `-selfup` spells the value as a path through the outer object; every other
    in-domain 2base cell spells it as the bound name. 1base has no bind, so its
    value is always a path and its store base is the constants' own token."""
    if target in ("cross", "otherobj"):
        return "CROSS"
    if base == "1base":
        return "SELF-1B"
    return "SELF-2B" if cell.endswith("-selfup") else "LOAD"


RULES = {
    # every one of these is published elsewhere; none is invented here
    "cu<=ru+1  (#892)": lambda ru, cu, b: "prod" if cu <= ru + 1 else "const",
    "cu<=ru+2": lambda ru, cu, b: "prod" if cu <= ru + 2 else "const",
    "H-MIX (frozen)": lambda ru, cu, b: ("prod" if cu <= ru + 1 + b
                                         else "const"),
    "always-prod": lambda ru, cu, b: "prod",
    "always-const": lambda ru, cu, b: "const",
    "clause-1-alone": lambda ru, cu, b: "prod" if ru > cu else "const",
    "KEY ILX LOAD (cu<=1)": lambda ru, cu, b: "prod" if cu <= 1 else "const",
}
ORDER = ["cu<=ru+1  (#892)", "cu<=ru+2", "H-MIX (frozen)", "clause-1-alone",
         "KEY ILX LOAD (cu<=1)", "always-prod", "always-const"]


def main():
    spec = {}
    for line in open(PRED):
        if line.startswith("#") or line.startswith("cell\t"):
            continue
        p = line.rstrip("\n").split("\t")
        spec[p[0]] = (int(p[2]), int(p[3]), p[4], p[5], p[6])

    rows = []
    for line in open(GRADE):
        m = ROW.match(line.rstrip("\n"))
        if not m or m.group(1) not in spec:
            continue
        cell, base, target, nst, _hm, obj, _tag = m.groups()
        ru, cu, _b, _t, dom = spec[cell]
        rows.append((cell, ru, cu, base, target, dom,
                     klass(cell, base, target), int(nst), obj))

    print("\n  GRID M re-partitioned — %d graded cells\n" % len(rows))
    print("  %-10s %-6s %5s %6s %6s   %s"
          % ("class", "cells", "prod", "const", "stores", "the (ru,cu) frontier"
             " where it flips"))
    print("  " + "-" * 88)
    for k in ("SELF-1B", "LOAD", "SELF-2B", "CROSS"):
        sub = [r for r in rows if r[6] == k]
        if not sub:
            continue
        pr = sum(1 for r in sub if r[8] == "prod")
        lo = sorted({(r[1], r[2]) for r in sub if r[8] == "prod"})
        hi = sorted({(r[1], r[2]) for r in sub if r[8] == "const"})
        flip = "prod at %s ; const at %s" % (
            ",".join("%d/%d" % p for p in lo) or "-",
            ",".join("%d/%d" % p for p in hi) or "-")
        nst = sorted({r[7] for r in sub})
        print("  %-10s %-6d %5d %6d %6s   %s"
              % (k, len(sub), pr, len(sub) - pr,
                 "%d-%d" % (nst[0], nst[-1]), flip[:120]))

    print("\n  BODY-LENGTH STRATIFICATION — the /QXSTALLS lesson.  If a class's"
          "\n  verdict were really body length, these two columns would separate."
          "\n  %-10s %-22s %s" % ("class", "stores when prod", "stores when const"))
    print("  " + "-" * 70)
    for k in ("SELF-1B", "LOAD", "SELF-2B", "CROSS"):
        sub = [r for r in rows if r[6] == k]
        if not sub:
            continue
        p = sorted({r[7] for r in sub if r[8] == "prod"})
        c = sorted({r[7] for r in sub if r[8] == "const"})
        print("  %-10s %-22s %s" % (k, p or "-", c or "-"))

    print("\n  RULES, scored on a table that already existed."
          "  IN-DOMAIN cells only (%d)\n"
          % sum(1 for r in rows if r[5] == "in"))
    print("  %-22s %6s %6s %8s   %s"
          % ("rule", "right", "WRONG", "refused", "its wrong cells"))
    print("  " + "-" * 92)
    ind = [r for r in rows if r[5] == "in"]
    print("  %-22s %6d %6d %8d   <- the decline floor, and it WINS"
          % ("the shipped refusal", 0, 0, len(ind)))
    for name in ORDER:
        fn = RULES[name]
        bad = [r for r in ind
               if fn(r[1], r[2], 1 if r[3] == "2base" else 0) != r[8]]
        print("  %-22s %6d %6d %8d   %s"
              % (name, len(ind) - len(bad), len(bad), 0,
                 ", ".join(sorted({r[6] for r in bad})) or "-"))

    print("\n  PER CLASS — the number that matters, because the classes"
          " disagree\n")
    print("  %-22s %s" % ("rule", "  ".join("%-10s" % k for k in
                                            ("SELF-1B", "LOAD", "SELF-2B"))))
    print("  " + "-" * 62)
    for name in ORDER:
        fn = RULES[name]
        cells = []
        for k in ("SELF-1B", "LOAD", "SELF-2B"):
            sub = [r for r in rows if r[6] == k and r[5] == "in"]
            bad = sum(1 for r in sub
                      if fn(r[1], r[2], 1 if r[3] == "2base" else 0) != r[8])
            cells.append("%d/%d" % (len(sub) - bad, len(sub)))
        print("  %-22s %s" % (name, "  ".join("%-10s" % c for c in cells)))

    print("\n  CONTROLS (declared out of domain at freeze, ADDENDUM 1 §A1.2)\n")
    for r in rows:
        if r[5] != "in":
            print("      %-30s %-8s ru=%d cu=%d  obj=%s  cu<=ru+1 says %s"
                  % (r[0], r[4], r[1], r[2], r[8],
                     "prod" if r[2] <= r[1] + 1 else "const"))
    return 0


if __name__ == "__main__":
    sys.exit(main())
