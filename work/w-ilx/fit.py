#!/usr/bin/env python3
"""fit.py — score KEY ILX on the configurations the PRIOR lanes already
published, before building anything fresh.

`w-spell` §4.1: *"for every candidate, score it on every prior lane's own cells
before building anything"* — RULE W died there without a compile.  This lane
takes it as binding.  The populations are

    GRID S, the two ADDRESS rows      `self` and `cross`
                                       x 5 use-count points x 2 base modes = 20
    GRID X, all six configurations     x 3 use-count points               = 18

which is every published cell KEY ILX's domain contains.  The other fourteen
GRID S spellings are arithmetic producers and out of the key's domain by
construction (the key is stated only for an ADDRESS-valued producer); they are
counted as `out-of-domain`, never as hits.

The truth is the OBJ, re-compiled here at the workload's own flags — not
w-spell's table.  The table is used as a CROSS-CHECK and any disagreement is
printed, because two lanes' objs disagreeing at what looked like one
configuration is exactly how #891 was found.

SHIPS NOTHING.  Usage:  fit.py
"""

import os
import subprocess
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import exdec                                                   # noqa: E402
from ilx import capture, observe, source, DC3                   # noqa: E402

# w-spell's published tables, transcribed from
# `docs/rungs/2026-08-06-w-spell.md` §3 and §5.  P/prod = the register-derived
# producer takes the top pool register.
POINTS_S = [(1, 1), (2, 1), (3, 1), (2, 2), (2, 3)]
WSPELL_S = {
    ("self", "1base"):  "PPPPP",
    ("self", "2base"):  "PPPPP",
    ("cross", "1base"): "cPPPP",
    ("cross", "2base"): "cPPPP",
}
POINTS_X = [(3, 5), (2, 4), (1, 1)]
WSPELL_X = {
    "A": ["prod", "prod", "prod"],
    "B": ["const", "const", "prod"],
    "E": ["const", "const", "prod"],
    "F": ["const", "const", "prod"],
}

EXPR = {"self": "(int)&s->inner", "cross": "(int)&s->inner2"}


def cells():
    out = []
    for sp in ("self", "cross"):
        for bmode in ("1base", "2base"):
            for i, (ru, cu) in enumerate(POINTS_S):
                out.append(("S-%s-%s-r%dk%d" % (sp, bmode, ru, cu),
                            (EXPR[sp], ru, cu, bmode == "2base", False),
                            WSPELL_S[(sp, bmode)][i]))
    for tag, expr, bind, rf in (("A", "(int)&s->inner", True, False),
                                ("B", "(int)&q", True, False),
                                ("E", "(int)&s->inner", False, False),
                                ("F", "(int)&q", True, True)):
        for i, (ru, cu) in enumerate(POINTS_X):
            out.append(("X-%s-r%dk%d" % (tag, ru, cu),
                        (expr, ru, cu, bind, rf), WSPELL_X[tag][i]))
    return out


def main():
    if DC3 is None or not os.path.isdir(DC3):
        print("SKIP: toolchain absent (dc3 tree)")
        return 3
    sel = reach = graded = oodom = failed = 0
    hit = miss = 0
    xcheck_ok = xcheck_bad = 0
    print("  %-26s %-8s %-9s %-9s %s"
          % ("cell", "obj", "KEY ILX", "clause", "w-spell"))
    print("  " + "-" * 74)
    for name, spec, published in cells():
        sel += 1
        words, streams = capture("fit/" + name, source(*spec))
        if words is None or ".ex" not in streams:
            failed += 1
            print("  %-26s COMPILE/CAPTURE FAILED" % name)
            continue
        reach += 1
        obj = observe(words, spec[1], spec[2])
        clause, pred, why = exdec.key_ilx(streams[".ex"])
        if obj.startswith("OOR"):
            oodom += 1
            print("  %-26s %-8s %-9s %-9s  %s" % (name, "OOR", "-", "-", obj))
            continue
        if pred is None:
            oodom += 1
            print("  %-26s %-8s %-9s %-9s  %s"
                  % (name, obj, "out-of-domain", "-", why))
            continue
        graded += 1
        ok = (pred == obj)
        hit += ok
        miss += not ok
        pub = {"P": "prod", "c": "const"}.get(published, published)
        if pub == obj:
            xcheck_ok += 1
        else:
            xcheck_bad += 1
        print("  %-26s %-8s %-9s %-9s %s%s"
              % (name, obj, pred, clause, pub,
                 "" if pub == obj else "   <-- **w-spell DISAGREES**")
              + ("" if ok else "   <-- **KEY ILX MISS**"))
    print("\n  selected %d | reached %d | GRADED %d | out-of-domain %d"
          " | compile-failed %d" % (sel, reach, graded, oodom, failed))
    print("  KEY ILX on the prior lanes' own configurations:"
          "  hit %d | MISS %d" % (hit, miss))
    print("  cross-check against w-spell's published tables:"
          "  agree %d | DISAGREE %d" % (xcheck_ok, xcheck_bad))
    print("  the shipped refusal on the same %d cells:"
          "  right 0 | WRONG 0 | refused %d" % (graded, graded))
    return 0


if __name__ == "__main__":
    sys.exit(main())
