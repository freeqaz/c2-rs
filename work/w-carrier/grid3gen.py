#!/usr/bin/env python3
"""grid3gen.py — GRID K3, w-carrier's second **declared POST-HOC** holdout.

DECLARED POST-HOC in this header before a cell of it was compiled, exactly as
GRID K2 was, and for a reason GRID K2 itself produced: three of its cells missed
**by construction of the cell**, not by a model being wrong.

  * `g_width1/2/8` spell TWO distinct literal values, so the multi-producer
    clause refuses them before the store WIDTH is ever reached. The width axis is
    still unexercised through a bound base.
  * `g_livearg` binds a member at offset **0**, which the zero-offset exclusion
    (`w-bind` §4.2, boards #856/#865) refuses one layer earlier, so the
    live-argument-base clause never sees it.

Each cell carries its PREDICTION in its own source, written before it was
compiled.
"""
import hashlib
import os
import sys

ROOT = os.path.dirname(os.path.abspath(__file__))
GRID = os.path.join(ROOT, "grid3")

WIDE = """struct BE { BE* mNext; BE* mPrev; };
struct W { char c0; char c1; short h0; short h1; long long q0; long long q1; };
struct H {
    H* mFreeHead;      // 0
    H* mUsedHead;      // 4
    BE mListHead;      // 8
    unsigned mSize;    // 16
    unsigned mCount;   // 20
    W  mWide;          // 24  (c0 24, c1 25, h0 26, h1 28, q0 32, q1 40)
};
"""

CTOR = """struct BE { BE* mNext; BE* mPrev; };
struct S { unsigned pad; BE list; unsigned n; };   // list at 4, NOT 0
struct H {
    unsigned mSize;
    BE* mSpare;
    H(unsigned a, S* s);
    BE* Alloc(unsigned a, S* s);
};
"""

CTOR1 = """struct BE { BE* mNext; BE* mPrev; };
struct S { unsigned pad; BE list; unsigned n; };   // list at 4, NOT 0
struct H {
    unsigned mSize;
    BE* mSpare;
    H(unsigned a, S* s);
    BE* Alloc(unsigned a);
};
"""

CELLS = []


def cell(name, targets, predicted, body, decl=WIDE, sig="void fn(H* h)"):
    CELLS.append((name, targets, predicted, decl, sig, body))


# ---- the WIDTH axis, with ONE literal value so the run has ONE producer -----
cell("h_width1", "a 1-byte store through a bound base", "IN-CLASS / match",
     "h->mSize = 1;\n    W& w = h->mWide;\n    w.c0 = 1;\n    w.c1 = 1;")
cell("h_width1_c", "control", "IN-CLASS / match",
     "h->mSize = 1;\n    h->mWide.c0 = 1;\n    h->mWide.c1 = 1;")
cell("h_width2", "a 2-byte store through a bound base", "IN-CLASS / match",
     "h->mSize = 7;\n    W& w = h->mWide;\n    w.h0 = 7;\n    w.h1 = 7;")
cell("h_width2_c", "control", "IN-CLASS / match",
     "h->mSize = 7;\n    h->mWide.h0 = 7;\n    h->mWide.h1 = 7;")
cell("h_width8", "an 8-byte DS-form store through a bound base", "IN-CLASS / match",
     "h->mSize = 0;\n    W& w = h->mWide;\n    w.q0 = 0;\n    w.q1 = 0;")
cell("h_width8_c", "control", "IN-CLASS / match",
     "h->mSize = 0;\n    h->mWide.q0 = 0;\n    h->mWide.q1 = 0;")
cell("h_widthmix", "three WIDTHS through one bound base, one literal",
     "IN-CLASS / match",
     "W& w = h->mWide;\n    w.c0 = 0;\n    w.h0 = 0;\n    w.q0 = 0;")

# ---- the live-argument-base clause, with the bind at a NON-zero offset ------
cell("h_livearg", "STORE_RUN_BIND_LIVE_ARG_BASE", "store-run-bind-live-arg-base",
     "mSize = a;\n    BE& l = s->list;\n    l.mNext = 0;\n    Alloc(a, s);",
     CTOR, "H::H(unsigned a, S* s)")
cell("h_livearg_ctrl", "the same bind where the call does NOT pass `s`",
     "IN-CLASS / match",
     "mSize = a;\n    BE& l = s->list;\n    l.mNext = 0;\n    Alloc(a);",
     CTOR1, "H::H(unsigned a, S* s)")


def main():
    os.makedirs(GRID, exist_ok=True)
    lines = []
    for name, targets, predicted, decl, sig, body in CELLS:
        d = os.path.join(GRID, name)
        os.makedirs(d, exist_ok=True)
        text = (
            f"// GRID K3 cell `{name}` — w-carrier, board #1199.\n"
            f"// DECLARED POST-HOC (see grid3gen.py's header) — not frozen ahead.\n"
            f"// TARGETS:   {targets}\n"
            f"// PREDICTED: {predicted}\n"
            f"// Compiled at the WORKLOAD's own /GR /O1 /Oi /EHsc (#1112).\n"
            f"{decl}\n{sig} {{\n    {body}\n}}\n"
        )
        with open(os.path.join(d, name + ".cpp"), "w") as f:
            f.write(text)
        lines.append(
            f"{hashlib.sha256(text.encode()).hexdigest()}  "
            f"work/w-carrier/grid3/{name}/{name}.cpp"
        )
    with open(os.path.join(ROOT, "GRID3.sha256"), "w") as f:
        f.write("\n".join(sorted(lines)) + "\n")
    print(f"{len(CELLS)} cells")


if __name__ == "__main__":
    sys.exit(main())
