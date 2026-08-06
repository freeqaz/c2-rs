#!/usr/bin/env python3
"""rivals.py — score the stated rivals on GRID V's graded table.

**Nothing is fitted here and nothing is proposed.** w-spell §8 records the
standing instruction — *after a refutation, no successor is fitted on the cells
that produced it* — and this lane takes it as binding. What this file does is
w-spell §4.1's cheap step in the other direction: it scores rules that were
**already written down elsewhere** against a table that now exists, so the next
lane knows which of them are already dead and does not spend a grid finding out.

It compiles NOTHING. It reads `holdout_grade.out` and `holdout_pred.tsv`.

The rules, each with its published source:

    shipped-refusal   `codegen::alloc::allocate` refuses a mixed run.
                      Wrong on 0 by construction. The decline floor.
    KEY ILX           this lane's, `exdec.key_ilx`. Frozen in
                      `holdout_pred.tsv` before any obj was compiled.
    cu<=ru+1          `cu <= ru + 1`, board **#892**, w-spell §4.2 — published
                      as fitting GRID S and GRID H's nine misses and
                      **deliberately not fitted there**. It is a rule about the
                      use counts ALONE and reads nothing from the IL.
    clause-1-alone    the producer wins iff `ru >= cu`. w-alloc2 §4.
    always-prod       the trivial constant rule, printed so the others have a
                      floor that is not 0.
    always-const      likewise.

SHIPS NOTHING.  Usage:  rivals.py
"""

import os
import re
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
GRADE = os.path.join(HERE, "holdout_grade.out")

ROW = re.compile(r"^\s+(\S+-r(\d+)k(\d+))\s+(\S+)\s+(prod|const)\s+(prod|const)")

RULES = {
    "KEY ILX (frozen)": None,          # read from the frozen column
    "cu<=ru+1 (#892)": lambda ru, cu, cl: "prod" if cu <= ru + 1 else "const",
    "clause-1-alone": lambda ru, cu, cl: "prod" if ru >= cu else "const",
    "always-prod": lambda ru, cu, cl: "prod",
    "always-const": lambda ru, cu, cl: "const",
}


FIT = os.path.join(HERE, "fit.out")
FITROW = re.compile(
    r"^\s+(\S+-r(\d+)k(\d+))\s+(prod|const)\s+(prod|const)\s+(\S+)")


def table(path, which):
    rows = []
    for line in open(path):
        if which == "V":
            m = ROW.match(line)
            if m:
                rows.append((m.group(1), int(m.group(2)), int(m.group(3)),
                             m.group(4), m.group(5), m.group(6)))
        else:
            m = FITROW.match(line)
            if m:
                rows.append((m.group(1), int(m.group(2)), int(m.group(3)),
                             m.group(6), m.group(5), m.group(4)))
    return rows


def score(rows, label):
    print("\n  %s — %d graded cells\n" % (label, len(rows)))
    print("  %-20s %6s %6s %s" % ("rule", "right", "WRONG", "refused"))
    print("  " + "-" * 52)
    print("  %-20s %6d %6d %6d   <- the decline floor"
          % ("shipped refusal", 0, 0, len(rows)))
    out = {}
    for name, fn in RULES.items():
        r = w = 0
        for _c, ru, cu, cl, frozen, obj in rows:
            pred = frozen if fn is None else fn(ru, cu, cl)
            r += pred == obj
            w += pred != obj
        out[name] = (r, w)
        print("  %-20s %6d %6d %6d" % (name, r, w, 0))
    return out


def main():
    if not os.path.exists(GRADE):
        print("  no graded table — run holdout.py --grade first")
        return 1
    rows = table(GRADE, "V")
    if not rows:
        print("  **PARSED 0 ROWS — the grade log's format moved**")
        return 1
    fit = table(FIT, "F") if os.path.exists(FIT) else []
    if fit:
        score(fit, "GRID S + GRID X, the FIT population (fit.out)")
    score(rows, "GRID V, the FROZEN HOLDOUT (holdout_grade.out)")
    if fit:
        score(fit + rows, "BOTH POPULATIONS")
    print("\n  Every rule above with WRONG > 0 LOSES to the shipped refusal.")
    print("  None is proposed for shipping and none is fitted here: `cu<=ru+1`"
          " is board #892's\n  published wording and the rest are prior lanes'."
          "  A rule scoring well on a table\n  it was scored against after the"
          " table existed has the standing RULE W2 had at 388\n  of 388, which"
          " is none.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
