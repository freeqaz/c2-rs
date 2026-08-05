#!/usr/bin/env python3
"""boundary_probe.py — lane w-wire, PREREG §4.

Compile the five CONSTRUCTED COUNTEREXAMPLES through real `c2.dll` and print
what it actually emits. The port REFUSES all of them; this probe exists to show
each refusal is a **different measured regime** rather than caution, which is
the standard `docs/ALLOC.md` and `docs/ORDER.md` hold their own refusals to.

Model-free. Nothing here is consulted by `crates/`.
"""
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from wirelib import compile_cod, parse_cod, seq  # noqa: E402

W = os.path.dirname(os.path.abspath(__file__))

# (name, values, extra dead formals, why the port refuses)
CASES = [
    ("C1", ["1", "2", "3", "4"], 0,
     "four distinct literals — past MAX_MODELLED_PRODUCERS, board #541"),
    ("C1b", ["1", "2", "3"], 0,
     "…and three, which IS in domain — the boundary, not a blanket refusal"),
    ("C3", ["1", "2"], 7,
     "pool boundary: 8 formals hold r4..r11, one register free, two wanted"),
    ("C4", ["100000", "1"], 0,
     "a WIDE literal beside a narrow one — lis+ori is two words, one producer"),
    ("C4b", ["100000", "200000"], 0,
     "two wide literals"),
]


def main():
    src = os.path.join(W, "boundary_probe.cpp")
    with open(src, "w") as f:
        f.write("struct S { unsigned m0,m1,m2,m3,m4,m5,m6,m7; };\n")
        for name, vals, extra, _why in CASES:
            params = ["S* s"] + ["unsigned x%d" % i for i in range(extra)]
            body = " ".join("s->m%d = %s;" % (k, v) for k, v in enumerate(vals))
            f.write("void X%s(%s) { %s }\n" % (name, ", ".join(params), body))

    n = 0
    for mode in ("O1", "Ox"):
        txt = compile_cod(src, os.path.join(W, "boundary_%s.cod" % mode),
                          os.path.join(W, "boundary_%s.obj" % mode), mode=mode)
        fns = parse_cod(txt)
        print("=== %s ===" % mode)
        for name, vals, extra, why in CASES:
            fn = "X" + name
            if fn not in fns:
                raise SystemExit("FAIL: %s produced no PROC at %s" % (fn, mode))
            n += 1
            print("%-5s %-22s %s" % (name, ",".join(vals), why))
            print("      %s" % " ".join(seq(fns[fn])))
    print()
    print("BOUNDARY CASES COMPILED: %d" % n)
    if n == 0:
        raise SystemExit("FAIL: graded nothing")


if __name__ == "__main__":
    main()
