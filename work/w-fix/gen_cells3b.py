#!/usr/bin/env python3
"""gen_cells3b.py — GRID-3b, the addendum cells (see `work/w-fix/ADDENDUM-1.md`).

Lane w-fix measurement tooling. **Read-only with respect to `crates/`.**

GRID-3 grades the empty chain at depths 1…4. The rule this lane would ship
iterates to a fixpoint and therefore fires at every depth, so the deeper points
are graded here rather than extrapolated: 5, 6 and 8. Plus the four axes a
fixpoint has to cross in the MIDDLE of a chain and GRID-3 crossed only at the
top — a mixed `Seq` link, internal linkage at every link, every definition below
its use, and an argument computed at every link — and one more level of board
#924's own destructor family.

Same construction as `gen_cells3.py`, per-cell anchor control included, so
`grade3.py` reads both grids with one reader.

Usage:  gen_cells3b.py <outdir>
"""

import sys

import gen_cells3


def chain(n):
    """A chain `f -> g1 -> … -> g{n-1} -> h`, every body empty.

    `n` is the DEPTH: the number of call edges between `?f` and the empty `?h`.
    """
    src = ["void h() {}"]
    # `g{n-1}` calls h, `g{n-2}` calls `g{n-1}`, …, `f` calls `g1`.
    for i in range(n - 1, 0, -1):
        callee = "h" if i == n - 1 else "g%d" % (i + 1)
        src.append("void g%d() { %s(); }" % (i, callee))
    src.append("void f() { %s(); }" % ("g1" if n > 1 else "h"))
    edges = []
    for i in range(1, n):
        edges.append(("g%d" % i, "h" if i == n - 1 else "g%d" % (i + 1)))
    edges.append(("f", "g1" if n > 1 else "h"))
    return edges, "\n" + "\n".join(src) + "\n"


CELLS = [
    ("m1_chain_d5",) + chain(5),
    ("m2_chain_d6",) + chain(6),
    ("m3_chain_d8",) + chain(8),
    ("m4_seq_mixed_mid", [("g1", "h"), ("g1", "ext"), ("f", "g1")], """
void ext();
void h() {}
void g1() { h(); ext(); }
void f() { g1(); }
"""),
    ("m5_static_chain", [("g2", "h"), ("g1", "g2"), ("f", "g1")], """
static void h() {}
static void g2() { h(); }
static void g1() { g2(); }
void f() { g1(); }
"""),
    ("m6_defined_after", [("g2", "h"), ("g1", "g2"), ("f", "g1")], """
void h();
void g2();
void g1();
void f() { g1(); }
void g1() { g2(); }
void g2() { h(); }
void h() {}
"""),
    ("m7_dtor_chain_d4", [("E::~E", "D::~D"), ("D::~D", "C::~C"),
                          ("C::~C", "B::~B"), ("B::~B", "A::~A")], """
struct A { ~A() {} };
struct B { A a; ~B(); };
struct C { B b; ~C(); };
struct D { C c; ~D(); };
struct E { D d; ~E(); };
B::~B() {}
C::~C() {}
D::~D() {}
E::~E() {}
"""),
    ("m8_arg_every_link", [("g2", "h"), ("g1", "g2"), ("f", "g1")], """
void h(int a) {}
void g2(int a) { h(a + 1); }
void g1(int a) { g2(a * 2); }
void f(int a) { g1(a - 3); }
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
