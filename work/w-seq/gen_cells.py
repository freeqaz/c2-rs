#!/usr/bin/env python3
"""gen_cells.py — write GRID-S, the SPLICE grid.

Lane w-seq measurement tooling. **Read-only with respect to `crates/`.**

WHAT THE GRID ASKS
------------------
The workload measurement says c2's body for a differing caller is very often
**c2's own body for its callee** (SPLICE-0, 1,967 of 3,195) and almost never the
port's argument setup with the callee's body appended (SPLICE-P, 578 of 3,195,
every one of them at a port body of exactly one word).

That is a statement about c2's obj. It is **not** a statement about what the
port could emit, because on the workload the callee is usually one the port
cannot lower at all. GRID-S closes that: every cell's callee is a shape the port
**does** lower — a straight-line integer leaf — so the splice is a body the port
already has, and the only open question is whether it is c2's.

Each cell varies exactly one thing about the CALL SITE, because that is what the
workload's failure witnesses point at:

    the port's setup is EMPTY                     -> SPLICE-0 should hold
    the port's setup is a register move           -> the `?Release@Object` family
    the port's setup is a pointer offset          -> the `??1?$pair` family
    the port's setup is arithmetic                -> constant folding
    the caller is FRAMED                          -> the `?back@?$vector` family
    the caller is a SEQ                           -> the 1,541-function family

THE PER-CELL POSITIVE CONTROL
-----------------------------
Every cell carries `void anchor(){ ext_anchor(); }`, whose callee this TU does
NOT define. `?anchor` must keep exactly one REL24. Without it, "the port and c2
agree" cannot be told from "the reader found nothing" — `docs/STATUS.md` trap 5.

Every cell also carries the CALLEE as a separately emitted function, which under
`/Gy` it is: c2 emits a callee whether or not it inlined it
(`INLINE_PREDICATE.md` §2), and that is what makes the splice's right-hand side
readable from the same obj.

Usage:  gen_cells.py <outdir>
"""

import hashlib
import os
import sys

ANCHOR = "void ext_anchor();\nvoid anchor() { ext_anchor(); }\n"

# (cell, caller spelling, what it varies, what the splice predicts)
CELLS = [
    (
        "s01_void_tail_no_setup",
        "int gv_sink;\nint g(int a) { return a + 1; }\nint f(int a) { return g(a); }\n",
        "empty setup, int tail call, passthrough formal",
    ),
    (
        "s02_void_call_no_setup",
        "int q(int a) { return a + 1; }\nint p;\nvoid f() { p = q(p); }\n",
        "a caller whose call is not in tail position",
    ),
    (
        "s03_reg_move_setup",
        "int g(int a) { return a + 1; }\nint f(int a, int b) { return g(b); }\n",
        "setup is a register move — the `?Release@Object@Hmx@@` family",
    ),
    (
        "s04_arith_setup",
        "int g(int a) { return a + 1; }\nint f(int a) { return g(a + 1); }\n",
        "setup is arithmetic — does c2 fold the two literals",
    ),
    (
        "s05_lit_setup",
        "int g(int a) { return a + 1; }\nint f() { return g(7); }\n",
        "setup is a literal",
    ),
    (
        "s06_ptr_offset_setup",
        "struct A { int p; int q; };\n"
        "struct B { int z; A a; };\n"
        "int g(A* a) { return a->q; }\n"
        "int f(B* b) { return g(&b->a); }\n",
        "setup is a pointer offset — the `??1?$pair@…` displacement fold",
    ),
    (
        "s07_ptr_field_load",
        "struct A { int p; int q; };\n"
        "int g(A* a) { return a->q; }\n"
        "int f(A* a) { return g(a); }\n",
        "empty setup, a callee that loads through its formal",
    ),
    (
        "s08_framed_add",
        "int g(int a) { return a + 1; }\nint f(int a) { return g(a) + 2; }\n",
        "the caller is FRAMED — the `?back@?$vector@…` family's shape",
    ),
    (
        "s09_seq_two_calls",
        "int p1;\nint p2;\n"
        "void g1() { p1 = 1; }\nvoid g2() { p2 = 2; }\n"
        "void f() { g1(); g2(); }\n",
        "the caller is a SEQ over two same-TU callees",
    ),
    (
        "s10_seq_one_call_tail",
        "int g(int a) { return a + 1; }\n"
        "int f(int a) { int t = g(a); return t + t; }\n",
        "a SEQ with one call and a non-void tail — the 816-function shape",
    ),
    (
        "s11_two_arg_perm",
        "int g(int a, int b) { return a - b; }\n"
        "int f(int a, int b) { return g(b, a); }\n",
        "setup is a two-register permutation (#843: `sub` is not `subf`)",
    ),
    (
        "s12_callee_calls_extern",
        "void ext();\nvoid g() { ext(); }\nvoid f() { g(); }\n",
        "CONTROL — the callee is not lowerable-leaf; c2 keeps a call",
    ),
    (
        "s13_deep_callee",
        "int g(int a) { return a + 1 + a + 2 + a + 3 + a + 4; }\n"
        "int f(int a) { return g(a); }\n",
        "a larger callee — does size change the answer at an empty setup",
    ),
    (
        "s14_cmp_callee",
        "int g(int a) { return a > 3; }\nint f(int a) { return g(a); }\n",
        "the callee is a comparison leaf, a different port shape",
    ),
]


def main():
    out = sys.argv[1]
    os.makedirs(out, exist_ok=True)
    stamp = hashlib.sha256()
    names = []
    for name, body, why in CELLS:
        text = "// GRID-S cell %s — %s\n%s\n%s" % (name, why, body, ANCHOR)
        p = os.path.join(out, name + ".cpp")
        with open(p, "w") as fh:
            fh.write(text)
        stamp.update(name.encode())
        stamp.update(text.encode())
        names.append(p)
    print("cells: %d" % len(CELLS))
    print("GRID-S sha256: %s" % stamp.hexdigest())
    for p in names:
        print("  %s" % os.path.relpath(p))


if __name__ == "__main__":
    main()
