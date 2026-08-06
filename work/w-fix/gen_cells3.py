#!/usr/bin/env python3
"""gen_cells3.py — write GRID-3, the FIXPOINT boundary cells (board #924).

Lane w-fix measurement tooling. **Read-only with respect to `crates/`.**

WHAT THIS GRID ASKS, AND WHY IT IS PER-EDGE
-------------------------------------------
`w-empty` shipped the ONE-STEP mechanism E and measured, on a single cell
(`g07_empty_calls_empty`), that c2 closes E under itself: in

    void h() {}
    void g() { h(); }        // source body NOT empty
    void f() { g(); }

c2 emits **both** `?f` and `?g` as a bare `blr`. One cell is not a rule. This
grid walks the chain: depth 1…4, a non-empty body at each depth, branching
chains, a two-node cycle and direct self-recursion, a mechanism-I link mid-chain,
a mid-node that carries an argument the caller computes, a mid-node with a
side-effecting argument, a mid-node that writes a global, an indirect mid-site,
and the workload's own destructor-chain shape.

**Every cell is graded per EDGE, not per cell.** A depth-3 chain has three call
edges and each is scored separately, because "the whole chain collapsed" and
"only the bottom link collapsed" are different observations and a per-cell
verdict cannot tell them apart. That is the difference between board #924 being
answered and being restated.

EVERY CELL IS COMPILED TWICE
----------------------------
At the workload's own flags and again with `/Ob0` appended. At `/O1` alone an
absent REL24 cannot distinguish **E** (the call was dropped) from **I** (the
inliner expanded it) — `docs/INLINE_PREDICATE.md` §0. `/Ob0` turns expansion off
and leaves E untouched, so the pair is the discriminator:

    REL24 at /O1?   REL24 at /Ob0?   verdict
    no              no               E     — the call was dropped
    no              yes              I     — inline expansion
    yes             yes              CALL  — an ordinary call

THE PER-CELL POSITIVE CONTROL
-----------------------------
Every cell carries `void anchor(){ ext_anchor(); }`, whose callee this TU does
not define. `?anchor` must keep exactly one REL24 in **both** compilations of
**every** cell or the cell is refused rather than scored — `docs/STATUS.md`
trap 5, absence read as success, in its most literal form.

Usage:  gen_cells3.py <outdir>
"""

import hashlib
import os
import sys

ANCHOR = """
// The per-cell positive control: a callee this TU does not define. `?anchor`
// must keep exactly one REL24 at BOTH flag settings or the cell is not graded.
void ext_anchor();
void anchor() { ext_anchor(); }
"""

# (id, [(caller, callee), …], body)
#
# The edge spellings are DEMANGLED; `grade3.py` derives the mangled-name prefix
# from the spelling's structure and resolves it against the obj's own symbol
# table (#644 — never a positional read).
CELLS = [
    # ---- P1: the chain, 1…4 deep, every link empty -------------------------
    ("k1_chain_d1", [("f", "h")], """
void h() {}
void f() { h(); }
"""),
    ("k2_chain_d2", [("g1", "h"), ("f", "g1")], """
void h() {}
void g1() { h(); }
void f() { g1(); }
"""),
    ("k3_chain_d3", [("g2", "h"), ("g1", "g2"), ("f", "g1")], """
void h() {}
void g2() { h(); }
void g1() { g2(); }
void f() { g1(); }
"""),
    ("k4_chain_d4", [("g3", "h"), ("g2", "g3"), ("g1", "g2"), ("f", "g1")], """
void h() {}
void g3() { h(); }
void g2() { g3(); }
void g1() { g2(); }
void f() { g1(); }
"""),
    # ---- P2: a NON-EMPTY body at each depth; the chain must stop there ------
    ("k5_stop_d1", [("f", "h")], """
void ext();
void h() { ext(); }
void f() { h(); }
"""),
    ("k6_stop_d2", [("g1", "h"), ("f", "g1")], """
void ext();
void h() { ext(); }
void g1() { h(); }
void f() { g1(); }
"""),
    ("k7_stop_d3", [("g2", "h"), ("g1", "g2"), ("f", "g1")], """
void ext();
void h() { ext(); }
void g2() { h(); }
void g1() { g2(); }
void f() { g1(); }
"""),
    # ---- branching: one caller, two empty callees; and a shared callee ------
    ("k8_two_empty", [("g1", "h1"), ("g1", "h2"), ("f", "g1")], """
void h1() {}
void h2() {}
void g1() { h1(); h2(); }
void f() { g1(); }
"""),
    ("k9_diamond", [("ga", "h"), ("gb", "h"), ("f", "ga"), ("f", "gb")], """
void h() {}
void ga() { h(); }
void gb() { h(); }
void f() { ga(); gb(); }
"""),
    # ---- the CYCLE: two-node, and direct self-recursion ---------------------
    ("k10_cycle2", [("a", "b"), ("b", "a"), ("f", "a")], """
void b();
void a() { b(); }
void b() { a(); }
void f() { a(); }
"""),
    ("k11_self", [("r", "r"), ("f", "r")], """
void r() { r(); }
void f() { r(); }
"""),
    # ---- mechanism I crossing the chain ------------------------------------
    ("k12_cross_i", [("g1", "m"), ("f", "g1")], """
int m(int a) { return a; }
int g1(int a) { return m(a); }
int f(int a) { return g1(a); }
"""),
    ("k13_i_result_dropped", [("g1", "m"), ("f", "g1")], """
int m(int a) { return a; }
void g1(int a) { m(a); }
void f(int a) { g1(a); }
"""),
    # ---- an argument the caller computes, carried down the chain ------------
    ("k14_arg_chain", [("g1", "h"), ("f", "g1")], """
void h(int a) {}
void g1(int a) { h(a + 1); }
void f(int a) { g1(a * 2); }
"""),
    ("k15_side_effect_mid", [("g1", "h"), ("f", "g1")], """
int sink;
void h(int a) {}
void g1() { h(sink++); }
void f() { g1(); }
"""),
    ("k16_mid_stores_global", [("g1", "h"), ("f", "g1")], """
int gv;
void h(int a) {}
void g1(int a) { gv = a; h(a); }
void f(int a) { g1(a); }
"""),
    # ---- the workload's own shape: a destructor chain (board #924's 143) ----
    ("k17_dtor_chain_d2", [("C::~C", "B::~B"), ("B::~B", "A::~A")], """
struct A { ~A() {} };
struct B { A a; ~B(); };
struct C { B b; ~C(); };
B::~B() {}
C::~C() {}
"""),
    ("k18_dtor_chain_d3", [("D::~D", "C::~C"), ("C::~C", "B::~B"), ("B::~B", "A::~A")], """
struct A { ~A() {} };
struct B { A a; ~B(); };
struct C { B b; ~C(); };
struct D { C c; ~D(); };
B::~B() {}
C::~C() {}
D::~D() {}
"""),
    # ---- an INDIRECT site mid-chain: E does not fire there (w-empty §7) -----
    ("k19_fnptr_mid", [("g1", "h"), ("f", "g1")], """
void h() {}
void g1() { void (*p)() = h; p(); }
void f() { g1(); }
"""),
    # ---- a register permutation at every link ------------------------------
    ("k20_perm_chain", [("g1", "h"), ("f", "g1")], """
void h(int a, int b) {}
void g1(int a, int b) { h(b, a); }
void f(int a, int b) { g1(b, a); }
"""),
]


def main(argv):
    if len(argv) != 1:
        print(__doc__)
        return 2
    outdir = argv[0]
    os.makedirs(outdir, exist_ok=True)
    stamps = []
    for cid, edges, body in CELLS:
        # The cell must `#include` nothing: `probe2.sh` drops the workload's `/I`
        # flags because they name dc3-relative directories.
        text = "// w-fix GRID-3 cell %s\n%s%s" % (cid, body, ANCHOR)
        with open(os.path.join(outdir, cid + ".cpp"), "w") as fh:
            fh.write(text)
        stamps.append(
            "%s  %s  edges=%s"
            % (
                hashlib.sha256(text.encode()).hexdigest(),
                cid,
                ";".join("%s->%s" % e for e in edges),
            )
        )
    manifest = "\n".join(stamps) + "\n"
    with open(os.path.join(outdir, "GRID.sha256"), "w") as fh:
        fh.write(manifest)
    stamp = hashlib.sha256(manifest.encode()).hexdigest()
    with open(os.path.join(outdir, "GRID.stamp"), "w") as fh:
        fh.write(stamp + "\n")
    print("cells: %d   edges: %d" % (len(CELLS), sum(len(e) for _, e, _ in CELLS)))
    print("GRID-3 stamp: %s" % stamp)
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
