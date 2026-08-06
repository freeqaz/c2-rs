#!/usr/bin/env python3
"""gen_cells.py — write the w-empty boundary GRID's source cells.

Lane w-empty measurement tooling. **Read-only with respect to `crates/`.**

WHAT THE GRID ASKS
------------------
`docs/INLINE_PREDICATE.md` §1 states mechanism **E** — *c2 emits no call, no
relocation and no external symbol for a call whose callee is defined in this TU
and whose source body is empty* — and bounds it with fifteen probes. This lane
has to SHIP that rule into the port, so the boundary has to be walked wider than
"is there a REL24 to `?g`": the port emits BYTES, and the question it must answer
per caller is *what is the whole `.text` of `?f`*.

Two axes, therefore, and both are compiled:

* **C-cells** vary the CALLEE and hold the caller at `void f(){ g(...); }`.
  They answer "does E fire".
* **F-cells** hold the callee at a graded-E body and vary the CALL SITE and the
  CALLER. They answer "and then what does `?f` look like" — the argument setup,
  a second call, a side-effecting argument, an indirect site.

EVERY CELL IS COMPILED TWICE
----------------------------
Once at the workload's own flags and once with `/Ob0` appended, exactly as
`work/w-inline/refobj_ob0.sh` does and for the same reason: at `/O1` alone an
absent REL24 cannot tell **E** (the front end dropped the call) from **I** (the
inliner expanded it). `/Ob0` turns expansion off and leaves E untouched
(`INLINE_PREDICATE.md` §1), so the pair is the discriminator:

    REL24 at /O1?   REL24 at /Ob0?   verdict
    no              no               E     — the front end dropped it
    no              yes              I     — inline expansion
    yes             yes              NEITHER — an ordinary call

THE PER-CELL POSITIVE CONTROL
-----------------------------
Every cell carries `void anchor(){ ext_anchor(); }`, whose callee this TU does
NOT define. `?anchor` must keep exactly one REL24 in **both** compilations of
**every** cell. Without it "no relocation to `?g`" is indistinguishable from "the
reader found no relocations", which is `docs/STATUS.md` trap 5 in its most
literal form — absence read as success. The grader fails a cell whose anchor is
missing rather than scoring it.

Usage:  gen_cells.py <outdir>
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

# (id, callee-symbol, caller-symbol, body)
#
# `callee` and `caller` are the DEMANGLED spellings the grader resolves against
# the obj's mangled names; `None` for a callee this TU does not define.
CELLS = [
    # ---- C: the callee's body, caller held at a plain void call --------------
    ("c00_empty", "g", "f", """
void g() {}
void f() { g(); }
"""),
    ("c01_semicolon", "g", "f", """
void g() { ; }
void f() { g(); }
"""),
    ("c02_ignored_formal", "g", "f", """
void g(int a) {}
void f(int a) { g(a); }
"""),
    ("c03_dead_local_init", "g", "f", """
void g(int a) { int x = 1; }
void f(int a) { g(a); }
"""),
    ("c04_dead_store", "g", "f", """
void g(int a) { int x = a; }
void f(int a) { g(a); }
"""),
    ("c05_empty_loop", "g", "f", """
void g(int a) { for (int i = 0; i < a; ++i) {} }
void f(int a) { g(a); }
"""),
    ("c06_static", "g", "f", """
static void g() {}
void f() { g(); }
"""),
    ("c07_inline", "g", "f", """
inline void g() {}
void f() { g(); }
"""),
    ("c08_member_inclass", "S::g", "f", """
struct S { void g() {} };
void f(S& s) { s.g(); }
"""),
    ("c09_member_outclass", "S::g", "f", """
struct S { void g(); };
void S::g() {}
void f(S& s) { s.g(); }
"""),
    ("c10_virtual_qualified", "S::g", "f", """
struct S { virtual void g() {} };
void f(S& s) { s.S::g(); }
"""),
    ("c11_ctor_base", "S::S", "D::D", """
struct S { S() {} };
struct D : S { D(); };
D::D() {}
"""),
    ("c12_ctor_meminit", "S::S", "D::D", """
struct S { int x; S() : x(0) {} };
struct D : S { D(); };
D::D() {}
"""),
    ("c13_dtor_base", "S::~S", "D::~D", """
struct S { ~S() {} };
struct D : S { ~D(); };
D::~D() {}
"""),
    ("c14_return_stmt", "g", "f", """
void g() { return; }
void f() { g(); }
"""),
    ("c15_if_empty", "g", "f", """
void g(int a) { if (a) {} }
void f(int a) { g(a); }
"""),
    ("c16_volatile_local", "g", "f", """
void g() { volatile int x = 0; }
void f() { g(); }
"""),
    ("c17_store_global", "g", "f", """
int gv;
void g(int a) { gv = a; }
void f(int a) { g(a); }
"""),
    ("c18_calls_extern", "g", "f", """
void ext();
void g() { ext(); }
void f() { g(); }
"""),
    ("c19_ret_param", "g", "f", """
int g(int a) { return a; }
int f(int a) { return g(a); }
"""),
    ("c20_ret_const", "g", "f", """
int g() { return 0; }
int f() { return g(); }
"""),
    ("c21_ret_plus1", "g", "f", """
int g(int a) { return a + 1; }
int f(int a) { return g(a); }
"""),
    ("c22_extern_callee", None, "f", """
void g();
void f() { g(); }
"""),
    # ---- F: the call SITE and the caller, callee held at a graded-E body -----
    ("f02_perm", "g", "f", """
void g(int a, int b) {}
void f(int a, int b) { g(b, a); }
"""),
    ("f03_expr_arg", "g", "f", """
void g(int a) {}
void f(int a) { g(a + 1); }
"""),
    ("f04_deref_arg", "g", "f", """
void g(int a) {}
void f(int* p) { g(*p); }
"""),
    ("f05_side_effect_arg", "g", "f", """
int sink;
void g(int a) {}
void f() { g(sink++); }
"""),
    ("f06_two_calls", "g", "f", """
void g(int a) {}
void f(int a) { g(a); g(a); }
"""),
    ("f07_nonvoid_caller", "g", "f", """
void g(int a) {}
int f(int a) { g(a); return a; }
"""),
    ("f08_mixed", "g", "f", """
void ext();
void g(int a) {}
void f(int a) { g(a); ext(); }
"""),
    ("f09_fnptr", "g", "f", """
void g() {}
void f() { void (*p)() = g; p(); }
"""),
    ("f10_virtual_ptr", "S::g", "f", """
struct S { virtual void g() {} };
void f(S* s) { s->g(); }
"""),
]


def main(argv):
    if len(argv) != 1:
        print(__doc__)
        return 2
    outdir = argv[0]
    os.makedirs(outdir, exist_ok=True)
    stamps = []
    for cid, callee, caller, body in CELLS:
        # The cell must `#include` nothing: `work/w-fnbyte/probe.sh` drops the
        # workload's `/I` flags because they name dc3-relative directories.
        text = "// w-empty GRID cell %s\n%s%s" % (cid, body, ANCHOR)
        path = os.path.join(outdir, cid + ".cpp")
        with open(path, "w") as fh:
            fh.write(text)
        stamps.append(
            "%s  %s  callee=%s  caller=%s"
            % (hashlib.sha256(text.encode()).hexdigest(), cid, callee, caller)
        )
    manifest = "\n".join(stamps) + "\n"
    with open(os.path.join(outdir, "GRID.sha256"), "w") as fh:
        fh.write(manifest)
    fh_stamp = hashlib.sha256(manifest.encode()).hexdigest()
    with open(os.path.join(outdir, "GRID.stamp"), "w") as fh:
        fh.write(fh_stamp + "\n")
    print("cells: %d" % len(CELLS))
    print("GRID stamp: %s" % fh_stamp)
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
