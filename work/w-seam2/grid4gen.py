#!/usr/bin/env python3
"""GRID S4 — the FRESH HOLDOUT for the live-argument transfer gate.

**Declared, and declared as a holdout for a rule this lane derived from the cells
that refuted it.** GRID S3 separated *why* the run stops transferring — a store
whose value is a formal the call keeps alive — and a rule read off the four cells
that broke is precisely how all six refuted allocation keys got written
(`w-heap` §4.1.1). So the gate is tested on the level every derivation cell holds
FIXED: **the call's arity.**

Every cell that produced the rule has a call of arity 2 (`this` + one argument),
so "slot >= 1 is live" and "slot 1 is live" are the same statement there. Here the
callee takes **two** explicit arguments, so slot 2 is live in some cells and not
in others, and the two readings come apart:

    a2_break2   the call passes `b`; the run stores `b`      -> must REFUSE
    a2_break1   the call passes `a`; the run stores `a`      -> must REFUSE
    a2_ok3      the call passes `a`,`b`; the run stores `c`  -> must EMIT
    a1_ok2      the call passes `a` only; the run stores `b` -> must EMIT
    a0_okall    the call is NULLARY; the run stores a,b,c    -> must EMIT
    a2_okthis   the call passes `a`,`b`; the run stores `this`-> must EMIT

`*_lf` is the leaf control for each: the identical run with no call. A cell that
must EMIT is only interesting beside the leaf whose run it claims to reproduce,
which is board #866's own construction and the thing GRID S got right.

The struct and the formals are deliberately NOT GRID S's — three formals rather
than two, and members at offsets GRID S never writes — so a rule that happens to
fit GRID S's register numbering has nowhere to hide.
"""

import hashlib
import os
import sys

ROOT = os.path.dirname(os.path.abspath(__file__))
GRID = os.path.join(ROOT, "grid4")

PRE = (
    "struct BE { BE* mNext; BE* mPrev; };\n"
    "struct Q {\n"
    "  Q(unsigned int a, unsigned int b, unsigned int c);\n"
    "  void lf(unsigned int a, unsigned int b, unsigned int c);\n"
    "  BE* A2(unsigned int, unsigned int);\n"
    "  BE* A1(unsigned int);\n"
    "  BE* A0();\n"
    "  Q* n0; Q* n1;\n"
    "  unsigned int k0; unsigned int k1; unsigned int k2; unsigned int k3;\n"
    "  unsigned int k4; unsigned int k5;\n"
    "};\n"
)

CTOR = "Q::Q(unsigned int a, unsigned int b, unsigned int c)"
LEAF = "void Q::lf(unsigned int a, unsigned int b, unsigned int c)"

CELLS = [
    # slot 2 (`b`) is live and the run stores it: the two readings of the rule
    # disagree here, and only "every slot >= 1" refuses it.
    ("a2_break2", ["k0 = 0;", "k1 = c;", "k2 = b;"], "A2(a, b)"),
    ("a2_break1", ["k0 = 0;", "k1 = c;", "k2 = a;"], "A2(a, b)"),
    # nothing the call keeps alive is stored — must transfer.
    ("a2_ok3",    ["k0 = 0;", "k1 = c;", "k2 = c;"], "A2(a, b)"),
    ("a2_okthis", ["k0 = 0;", "n0 = this;", "n1 = this;"], "A2(a, b)"),
    # arity 1: `b` and `c` both die, so a run over them transfers even though
    # they are formals — the rule is about the CALL's slots, not about formals.
    ("a1_ok2",    ["k0 = 0;", "k1 = b;", "k2 = c;"], "A1(a)"),
    ("a1_break1", ["k0 = 0;", "k1 = b;", "k2 = a;"], "A1(a)"),
    # arity 0: nothing is live at all, so every formal is storable.
    ("a0_okall",  ["k0 = 0;", "k1 = a;", "k2 = b;", "k3 = c;"], "A0()"),
    # width and producer count moved with the arity, so the gate cannot be
    # passing by refusing everything long.
    ("a2_ok3w",   ["k0 = 0;", "k1 = 7;", "n0 = this;", "k2 = c;", "k3 = c;",
                   "k4 = c;"], "A2(a, b)"),
]

LEAVES = [
    ("a2_break2_lf", ["k0 = 0;", "k1 = c;", "k2 = b;"]),
    ("a2_ok3_lf",    ["k0 = 0;", "k1 = c;", "k2 = c;"]),
    ("a1_ok2_lf",    ["k0 = 0;", "k1 = b;", "k2 = c;"]),
    ("a0_okall_lf",  ["k0 = 0;", "k1 = a;", "k2 = b;", "k3 = c;"]),
    ("a2_ok3w_lf",   ["k0 = 0;", "k1 = 7;", "n0 = this;", "k2 = c;", "k3 = c;",
                      "k4 = c;"]),
]


def main():
    os.makedirs(GRID, exist_ok=True)
    lines = []
    plan = [(n, CTOR, r, "  %s;\n" % c) for n, r, c in CELLS]
    plan += [(n, LEAF, r, "") for n, r in LEAVES]
    for name, header, stmts, tail in plan:
        d = os.path.join(GRID, name)
        os.makedirs(d, exist_ok=True)
        text = PRE + "%s {\n%s%s}\n" % (
            header, "".join("  %s\n" % s for s in stmts), tail)
        with open(os.path.join(d, name + ".cpp"), "w") as f:
            f.write(text)
        lines.append("%s  %s/%s.cpp" % (
            hashlib.sha256(text.encode()).hexdigest(), name, name))
    lines.sort()
    with open(os.path.join(ROOT, "GRID4.sha256"), "w") as f:
        f.write("\n".join(lines) + "\n")
    sys.stderr.write("%d holdout cells\n" % len(lines))


if __name__ == "__main__":
    main()
