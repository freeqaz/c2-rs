#!/usr/bin/env python3
"""mode_probe.py — lane w-wire, PREREG §2 H1.

Does ALLOC / ORDER / LAYOUT hold at `/Ox` as well as at `/O1`?

Every prior lane's grid (`w-sched`, `w-alloc`, `w-order2`, `w-sym`, `w-parse`,
`w-frame2`) was compiled at the WORKLOAD's flags, `/O1 /Oi /EHsc`. This lane
ships an EMITTER, and `crates/c2-core` emits at both `/O1` and `/Ox` (the
fixture gate is `/Ox`). If the two modes disagree on a multi-literal store run,
the widening MUST be gated on the mode, and a lane that shipped it unconditional
would be `#232` again.

Model-free: this compares the two modes' emitted permutations to EACH OTHER and
prints every disagreement. It does not consult the Rust model at all.
"""
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from wirelib import compile_cod, parse_cod, seq  # noqa: E402

W = os.path.dirname(os.path.abspath(__file__))

# (name, per-statement value spelling). A bare integer is a literal; `u`/`v`/`w`
# name formals. Every case is a run of stores through ONE base symbol.
CASES = [
    # --- the class the parser admits TODAY: one literal, shared -------------
    ("Z1", ["9", "9", "9"]),
    ("Z2", ["0", "0"]),
    # --- W1: two and three DISTINCT literals -------------------------------
    ("A1", ["1", "2"]),
    ("A2", ["1", "2", "3"]),
    ("A3", ["1", "2", "3", "1"]),          # B4 killer cell
    ("A4", ["1", "2", "3", "2", "1"]),     # B7 killer cell
    ("A5", ["1", "2", "1", "2"]),          # A1 killer cell
    ("A6", ["1", "1", "2", "2", "2"]),     # B6 killer cell
    ("A7", ["1", "1", "2"]),               # ORDER "011" -> P1 P0 S1 S0 S2
    ("A8", ["1", "2", "2"]),
    # --- W2: literal + formal (an UNPRODUCED store in the run) -------------
    ("M1", ["1", "u"]),                    # ORDER "0." -> P0 S1 S0
    ("M2", ["u", "1"]),                    # ORDER ".0" -> P0 S0 S1
    ("M3", ["1", "u", "v"]),               # ORDER "0.." -> P0 S1 S2 S0
    ("M4", ["u", "1", "v"]),               # ORDER ".0." -> P0 S0 S2 S1
    ("M5", ["1", "2", "u", "v"]),          # ORDER "01.." -> P0 S2 P1 S3 S0 S1
    ("M6", ["u", "v", "1", "2"]),          # ORDER "..01"
    ("M7", ["u", "1", "2", "v"]),
    # --- the pool boundary: more formals eat the pool ----------------------
    ("P1", ["1", "2"]),                    # rendered with 6 extra formals below
]
# cases that take extra dead formals, to push the pool floor up
EXTRA_FORMALS = {"P1": 6}


def render(f):
    f.write("struct S { unsigned m0,m1,m2,m3,m4,m5,m6,m7; };\n")
    for name, vals in CASES:
        formals = sorted({v for v in vals if not v.isdigit()})
        extra = EXTRA_FORMALS.get(name, 0)
        params = ["S* s"] + ["unsigned %s" % v for v in formals]
        params += ["unsigned x%d" % i for i in range(extra)]
        body = " ".join("s->m%d = %s;" % (k, v) for k, v in enumerate(vals))
        f.write("void X%s(%s) { %s }\n" % (name, ", ".join(params), body))


def main():
    src = os.path.join(W, "mode_probe.cpp")
    with open(src, "w") as f:
        render(f)
    out = {}
    for mode in ("O1", "Ox"):
        txt = compile_cod(src, os.path.join(W, "mode_probe_%s.cod" % mode),
                          os.path.join(W, "mode_probe_%s.obj" % mode), mode=mode)
        out[mode] = parse_cod(txt)

    n = agree = 0
    for name, vals in CASES:
        fn = "X" + name
        for mode in ("O1", "Ox"):
            if fn not in out[mode]:
                raise SystemExit("FAIL: %s produced no PROC at %s" % (fn, mode))
        a = seq(out["O1"][fn])
        b = seq(out["Ox"][fn])
        n += 1
        same = a == b
        agree += same
        print("%-4s %-24s %s" % (name, ",".join(vals), "SAME" if same else "DIFFER"))
        print("      /O1  %s" % " ".join(a))
        if not same:
            print("      /Ox  %s" % " ".join(b))
    print()
    print("MODE AGREEMENT: %d of %d cases" % (agree, n))
    if n == 0:
        raise SystemExit("FAIL: graded nothing")


if __name__ == "__main__":
    main()
