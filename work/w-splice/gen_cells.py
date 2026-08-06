#!/usr/bin/env python3
"""gen_cells.py — write GRID-T, the SPLICE-0-PORT boundary grid.

Lane w-splice measurement tooling. **Read-only with respect to `crates/`.**

WHAT THE GRID ASKS
------------------
`work/w-seq/` graded the SPLICE-0 *hypothesis* on 2,470 workload cells. This
grid grades the *predicate* that ships it (`work/w-splice/PREREG.md` §1, clauses
S1-S9): every cell is a boundary the rule must be on one named side of, and the
prediction for each is registered in the PREREG **before this file was run**.

    S1  the caller is Tail or Seq       -> t07 (framed) must REFUSE
    S2  exactly one call site           -> t08 (two calls) must REFUSE
    S3  the port emits nothing around   -> t04/t05/t06 (setups) must REFUSE
        the call
    S4  the callee is not the caller    -> t12 (self-recursion) must REFUSE
    S5  the callee is defined here      -> t14 (external) must REFUSE
    S6  the port has a body for it      -> t09 (unlowerable callee) must REFUSE
    S7  that body is <= 64 bytes        -> t13, if the port can build one

and the three cells that are not boundaries at all:

    t01/t02/t03   the rule FIRES and the emitted body must be byte-exact
    t10           the callee calls an external: the byte is `48000000` either
                  way and the RELOCATION is the whole verdict (board #882,
                  w-seq's `s12` reproducer)
    t11           THE FIXPOINT, which is a question and not a prediction:
                  does c2 close a two-step splice chain?

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

# (cell, source, what it varies / which clause it grades)
CELLS = [
    (
        "t01_empty_setup_tail",
        "int g(int a) { return a + 1; }\nint f(int a) { return g(a); }\n",
        "FIRES — empty setup, a leaf callee the port lowers",
    ),
    (
        "t02_empty_setup_bigger_callee",
        "int g(int a) { return a + 1 + a + 2 + a + 3 + a + 4; }\n"
        "int f(int a) { return g(a); }\n",
        "FIRES — the same at a larger callee: size must not change the answer "
        "below S7's bound",
    ),
    (
        "t03_seq_identity_tail",
        "struct B { B(); };\nstruct D { B b; D(); };\nD::D() {}\n",
        "FIRES — the Seq shape: one call, an identity tail (SavedFormal), the "
        "634-function family",
    ),
    (
        "t04_reg_move_setup",
        "int g(int a) { return a + 1; }\nint f(int a, int b) { return g(b); }\n",
        "S3 REFUSES — the setup is a register move; c2 renames a field of the "
        "callee's body (?Release@Object@Hmx@@, 286 pairs)",
    ),
    (
        "t05_arith_setup",
        "int g(int a) { return a + 1; }\nint f(int a) { return g(a + 1); }\n",
        "S3 REFUSES — the setup is arithmetic; c2 folds the two literals",
    ),
    (
        "t06_ptr_offset_setup",
        "struct A { int p; int q; };\n"
        "struct B { int z; A a; };\n"
        "int g(A* a) { return a->q; }\n"
        "int f(B* b) { return g(&b->a); }\n",
        "S3 REFUSES — the setup is a pointer offset; c2 folds the displacement",
    ),
    (
        "t07_framed_caller",
        "int g(int a) { return a + 1; }\nint f(int a) { return g(a) + 2; }\n",
        "S1 REFUSES — the caller is FRAMED; SPLICE-0 is 0 of 123 there",
    ),
    (
        "t08_two_calls",
        "int p1;\nint p2;\n"
        "void g1() { p1 = 1; }\nvoid g2() { p2 = 2; }\n"
        "void f() { g1(); g2(); }\n",
        "S2 REFUSES — two call sites; SPLICE-N is 0 of 548",
    ),
    (
        "t09_unlowerable_callee",
        "int gsink;\n"
        "int g(int a) { int t = 0; for (int i = 0; i < a; ++i) t += i * a; "
        "gsink = t; return t; }\n"
        "int f(int a) { return g(a); }\n",
        "S6 REFUSES — the port has no body for this callee, so there is "
        "nothing to splice",
    ),
    (
        "t10_callee_calls_extern",
        "void ext();\nvoid g() { ext(); }\nvoid f() { g(); }\n",
        "FIRES — and ?f's single REL24 must name ext, not g. Both bodies are "
        "the word 48000000, so the RELOCATION is the verdict (#882, w-seq s12)",
    ),
    (
        "t11_fixpoint_two_steps",
        "int h(int a) { return a + 1; }\n"
        "int g(int a) { return h(a); }\n"
        "int f(int a) { return g(a); }\n",
        "THE FIXPOINT — a question, not a prediction. Does c2 close a two-step "
        "splice chain? The port takes ONE level in this rung either way",
    ),
    (
        "t12_self_recursion",
        "int r(int a) { return a ? r(a - 1) : 0; }\nint f(int a) { return r(a); }\n",
        "S4 REFUSES — direct self-recursion; INLINE_PREDICATE §4 grades "
        "`recurse` 336/336 refused by c2 too",
    ),
    (
        "t13_size_boundary",
        "int g(int a, int b, int c, int d) {\n"
        "  return a*b + b*c + c*d + d*a + a*c + b*d + a*a + b*b + c*c + d*d\n"
        "       + a*b*c + b*c*d + c*d*a + d*a*b + a*b*c*d;\n"
        "}\n"
        "int f(int a, int b, int c, int d) { return g(a, b, c, d); }\n",
        "S7 — a callee big enough to cross the 64-byte bound if the port can "
        "lower it at all. If the port refuses it, S7 never binds and that is "
        "PRINTED, not claimed as a pass",
    ),
    (
        "t14_extern_callee_control",
        "int g(int a);\nint f(int a) { return g(a); }\n",
        "S5 REFUSES (CONTROL) — the callee is not defined here; ?f keeps its "
        "REL24 against ?g",
    ),
]


def main():
    out = sys.argv[1]
    os.makedirs(out, exist_ok=True)
    stamp = hashlib.sha256()
    names = []
    for name, body, why in CELLS:
        text = "// GRID-T cell %s — %s\n%s\n%s" % (name, why, body, ANCHOR)
        p = os.path.join(out, name + ".cpp")
        with open(p, "w") as fh:
            fh.write(text)
        stamp.update(name.encode())
        stamp.update(text.encode())
        names.append(p)
    print("cells: %d" % len(CELLS))
    print("GRID-T sha256: %s" % stamp.hexdigest())
    for p in names:
        print("  %s" % os.path.relpath(p))


if __name__ == "__main__":
    main()
