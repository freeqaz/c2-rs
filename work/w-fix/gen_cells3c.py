#!/usr/bin/env python3
"""gen_cells3c.py — GRID-3c: the SETUP KINDS, moved into the middle of a chain.

Lane w-fix measurement tooling. **Read-only with respect to `crates/`.**

`w-empty` graded six argument-setup kinds — a register permutation, an
arithmetic argument, a literal, an FP argument, three formals, a global's
address — and it graded every one of them at **depth 1**, where the elided call
is the caller's own. The fixpoint fires on a mid-node with the same setups one
level down, and `docs/STATUS.md` trap 4's lesson applies: a shape graded at one
position is not graded at another.

GRID-3b already moved the *arithmetic* argument and the *definition order* into
the middle (`m8`, `m6`). This grid moves the four that are left, plus the two
call-linkage spellings the workload's own family uses:

    n1  an FP argument at every link
    n2  a member function at every link
    n3  a `virtual` member, called qualified, at every link
    n4  `inline` linkage at every link
    n5  three formals at every link
    n6  a CONSTRUCTOR chain — the ??0 mirror of GRID-3's ??1 dtor chain

Usage:  gen_cells3c.py <outdir>
"""

import sys

import gen_cells3

CELLS = [
    ("n1_fp_arg_mid", [("g1", "h"), ("f", "g1")], """
void h(float x) {}
void g1(float x) { h(x); }
void f(float x) { g1(x); }
"""),
    ("n2_member_mid", [("S::g1", "S::h"), ("f", "S::g1")], """
struct S { void h() {} void g1() { h(); } };
void f(S& s) { s.g1(); }
"""),
    ("n3_virtual_mid", [("S::g1", "S::h"), ("f", "S::g1")], """
struct S { virtual void h() {} void g1() { S::h(); } };
void f(S& s) { s.S::g1(); }
"""),
    ("n4_inline_mid", [("g1", "h"), ("f", "g1")], """
inline void h() {}
inline void g1() { h(); }
void f() { g1(); }
"""),
    ("n5_three_args_mid", [("g1", "h"), ("f", "g1")], """
void h(int a, int b, int c) {}
void g1(int a, int b, int c) { h(a, b, c); }
void f(int a, int b, int c) { g1(a, b, c); }
"""),
    ("n6_ctor_chain", [("C::C", "B::B"), ("B::B", "A::A")], """
struct A { A() {} };
struct B { A a; B(); };
struct C { B b; C(); };
B::B() {}
C::C() {}
"""),
]


def main(argv):
    if len(argv) != 1:
        print(__doc__)
        return 2
    gen_cells3.CELLS = CELLS
    return gen_cells3.main(argv)


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
