#!/usr/bin/env python3
"""grid2gen.py — GRID K2, w-carrier's **declared POST-HOC holdout**.

DECLARED POST-HOC, in this header, before a cell of it was compiled. It is not
frozen-before-the-fact the way `GRID.sha256` is, and it is not scored as if it
were: it exists because reading GRID K's own graded table showed the two things
that grid could not.

  1. **Three of `bind_run_ops`' six gates fired on NOTHING in GRID K** — the
     symbol-crossing clause, the live-argument-base clause and the group-shape
     clause. Board #1175: a gate that refuses nothing is indistinguishable from a
     gate that is not there, and `w-seam2`'s live-argument gate keyed on the
     wrong predicate and was inert, visible only to a cross-check. Every cell
     below that targets a gate is there to make it FIRE, and its twin is there to
     show the gate is a boundary and not a blanket.
  2. **GRID K holds the store WIDTH fixed at 4** — every bound member it touches
     is a `BE*`. The width picks the opcode and the DS-form alignment bound, and
     neither has ever been exercised through a bound base.

Each cell carries its PREDICTION in its own source, written before it was
compiled, so the table below is scored rather than described.
"""
import hashlib
import os
import sys

ROOT = os.path.dirname(os.path.abspath(__file__))
GRID = os.path.join(ROOT, "grid2")

WIDE = """struct BE { BE* mNext; BE* mPrev; };
struct W { char c0; char c1; short h0; short h1; long long q0; long long q1; };
struct H {
    H* mFreeHead;      // 0
    H* mUsedHead;      // 4
    BE mListHead;      // 8   (mNext at 8, mPrev at 12)
    unsigned mSize;    // 16
    unsigned mCount;   // 20
    W  mWide;          // 24  (c0 24, c1 25, h0 26, h1 28, q0 32, q1 40)
    BE* mSpare;        // 48
};
"""

FAR = """struct BE { BE* mNext; BE* mPrev; };
struct F {
    unsigned mPad[8000];   // 0 .. 31999
    BE mFar;               // 32000 (mNext 32000, mPrev 32004)
    unsigned mSize;        // 32008
};
"""

CTOR = """struct BE { BE* mNext; BE* mPrev; };
struct S { BE list; unsigned n; };
struct H {
    unsigned mSize;
    BE* mSpare;
    H(unsigned a, S* s);
    BE* Alloc(unsigned a, S* s);
};
"""

WIDECTOR = """struct BE { BE* mNext; BE* mPrev; };
struct H {
    H* mFreeHead;
    H* mUsedHead;
    BE mListHead;
    unsigned mSize;
    void nine(unsigned a, unsigned b, unsigned c, unsigned d,
              unsigned e, unsigned f, unsigned g, unsigned h);
};
"""

DECL = """struct BE { BE* mNext; BE* mPrev; };
struct H {
    H* mFreeHead;      // 0
    H* mUsedHead;      // 4
    BE mListHead;      // 8
    unsigned mSize;    // 16
    unsigned mCount;   // 20
};
"""

CELLS = []


def cell(name, targets, predicted, body, decl=DECL, sig="void fn(H* h, BE* p)"):
    CELLS.append((name, targets, predicted, decl, sig, body))


# ---- the symbol-crossing clause: make it FIRE, and show it is a boundary ----
cell("g_cross3", "STORE_RUN_BIND_SYMBOL_CROSSINGS", "store-run-bind-symbol-crossings",
     "BE& l = h->mListHead;\n    h->mSize = 2;\n    l.mNext = p;\n"
     "    h->mFreeHead = h;\n    l.mPrev = p;")
cell("g_cross2", "the same clause one step INSIDE its bound", "IN-CLASS / match",
     "BE& l = h->mListHead;\n    h->mSize = 2;\n    l.mNext = p;\n"
     "    l.mPrev = p;\n    h->mFreeHead = h;")

# ---- the live-argument-base clause: make it FIRE ---------------------------
cell("g_livearg", "STORE_RUN_BIND_LIVE_ARG_BASE", "store-run-bind-live-arg-base",
     "mSize = a;\n    BE& l = s->list;\n    l.mNext = 0;\n    Alloc(a, s);",
     CTOR, "H::H(unsigned a, S* s)")

# ---- the pool clause -------------------------------------------------------
cell("g_pool", "the pool clause of STORE_RUN_BIND_MULTI_PRODUCER",
     "store-run-bind-multi-producer",
     "mSize = 2;\n    BE& l = mListHead;\n    l.mNext = 0;",
     WIDECTOR,
     "void H::nine(unsigned a, unsigned b, unsigned c, unsigned d,\n"
     "          unsigned e, unsigned f, unsigned g, unsigned hh)")

# ---- the WIDTH axis GRID K held fixed at 4 ---------------------------------
cell("g_width1", "a 1-byte store through a bound base", "IN-CLASS / match",
     "h->mSize = 2;\n    W& w = h->mWide;\n    w.c0 = 1;\n    w.c1 = 1;",
     WIDE)
cell("g_width2", "a 2-byte store through a bound base", "IN-CLASS / match",
     "h->mSize = 2;\n    W& w = h->mWide;\n    w.h0 = 7;\n    w.h1 = 7;",
     WIDE)
cell("g_width8", "an 8-byte DS-form store through a bound base", "IN-CLASS / match",
     "h->mSize = 2;\n    W& w = h->mWide;\n    w.q0 = 0;\n    w.q1 = 0;",
     WIDE)
cell("g_width1_c", "control", "IN-CLASS / match",
     "h->mSize = 2;\n    h->mWide.c0 = 1;\n    h->mWide.c1 = 1;", WIDE)
cell("g_width8_c", "control", "IN-CLASS / match",
     "h->mSize = 2;\n    h->mWide.q0 = 0;\n    h->mWide.q1 = 0;", WIDE)

# ---- the displacement SUM, at and past the 16-bit bound --------------------
cell("g_bigoff", "a bind at +32000 — the SUM still inside a signed 16-bit field",
     "IN-CLASS / match",
     "f->mSize = 2;\n    BE& l = f->mFar;\n    l.mNext = p;\n    l.mPrev = p;",
     FAR, "void fn(F* f, BE* p)")

# ---- a POINTER local, not a reference (#1203) ------------------------------
cell("g_ptrlocal", "board #1203 — a pointer local is the SAME construct",
     "IN-CLASS / match",
     "h->mSize = 2;\n    BE* l = &h->mListHead;\n    l->mNext = p;\n    l->mPrev = p;")


def main():
    os.makedirs(GRID, exist_ok=True)
    lines = []
    for name, targets, predicted, decl, sig, body in CELLS:
        d = os.path.join(GRID, name)
        os.makedirs(d, exist_ok=True)
        text = (
            f"// GRID K2 cell `{name}` — w-carrier, board #1199.\n"
            f"// DECLARED POST-HOC (see grid2gen.py's header) — not frozen ahead.\n"
            f"// TARGETS:   {targets}\n"
            f"// PREDICTED: {predicted}\n"
            f"// Compiled at the WORKLOAD's own /GR /O1 /Oi /EHsc (#1112).\n"
            f"{decl}\n{sig} {{\n    {body}\n}}\n"
        )
        path = os.path.join(d, name + ".cpp")
        with open(path, "w") as f:
            f.write(text)
        lines.append(
            f"{hashlib.sha256(text.encode()).hexdigest()}  "
            f"work/w-carrier/grid2/{name}/{name}.cpp"
        )
    with open(os.path.join(ROOT, "GRID2.sha256"), "w") as f:
        f.write("\n".join(sorted(lines)) + "\n")
    print(f"{len(CELLS)} cells")


if __name__ == "__main__":
    sys.exit(main())
