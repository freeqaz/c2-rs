#!/usr/bin/env python3
"""gen_cells2.py — GRID-2's cells (see `work/w-empty/ADDENDUM-1.md`).

Lane w-empty measurement tooling. **Read-only with respect to `crates/`.**

The eight cells the SHIPPED rule needs and GRID-1 does not have: an argument
that is a global's address, an FP argument, a callee defined below its caller, a
callee whose address is also taken, a literal argument, three formals, two E
edges in one TU, and a forward declaration.

Same construction as `gen_cells.py` — the per-cell anchor control included — so
`grade_cells.py` reads both grids with one reader.

Usage:  gen_cells2.py <outdir>
"""

import sys

import gen_cells

CELLS = [
    ("g01_data_addr_arg", "g", "f", """
int gv;
void g(int* p) {}
void f() { g(&gv); }
"""),
    ("g02_float_arg", "g", "f", """
void g(float x) {}
void f(float x) { g(x); }
"""),
    ("g03_define_after_use", "g", "f", """
void g();
void f() { g(); }
void g() {}
"""),
    ("g04_addr_also_taken", "g", "f", """
void g() {}
void (*keep)() = g;
void f() { g(); }
"""),
    ("g05_const_arg", "g", "f", """
void g(int a) {}
void f() { g(5); }
"""),
    ("g06_three_args", "g", "f", """
void g(int a, int b, int c) {}
void f(int a, int b, int c) { g(a, b, c); }
"""),
    ("g07_empty_calls_empty", "h", "g", """
void h() {}
void g() { h(); }
void f() { g(); }
"""),
    ("g08_empty_ext_decl", "g", "f", """
extern void g();
void f() { g(); }
void g() {}
"""),
]


def main(argv):
    if len(argv) != 1:
        print(__doc__)
        return 2
    gen_cells.CELLS = CELLS
    return gen_cells.main(argv)


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
