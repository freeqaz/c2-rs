#!/usr/bin/env python3
"""gridgen.py — write the w-bind GRID cells, one directory per cell (#1045).

The two target cells (`b_target_bind`, `b_target_direct`) are written by hand
and are NOT regenerated here: they are the shipped TU and `w-f23`'s own direct
cell, and a generator that could paraphrase them is a generator that could make
the control pair agree by accident.

Everything else is a structural axis. The axes, and what each cell is the only
one that can reach, are in `work/w-bind/PREREG.md` §4 and in the rung's table.
"""
import os
import sys

HDR = """struct BE { BE* mNext; BE* mPrev; };
struct H {
    H* mFreeHead;      // 0
    H* mUsedHead;      // 4
    BE mListHead;      // 8   (mNext at 8, mPrev at 12)
    unsigned mSize;    // 16
    unsigned mCount;   // 20
};
"""

CELLS = {
    # ---- AXIS A: WHAT IS BOUND ------------------------------------------
    "b_bind_nonthis": (
        "AXIS A — the bind is to a member of the SECOND pointer formal, not of\n"
        "`this`. The axis every grid on this row has held constant (board #866's\n"
        "refutation is what this cell exists for): the production must not care\n"
        "WHICH formal the bound object hangs off.",
        HDR + """
void fn(H* a, H* b) {
    a->mSize = 1;
    BE& l = b->mListHead;
    l.mNext = &l;
    l.mPrev = &l;
}
""",
    ),
    "b_bind_global": (
        "AXIS A — the bind is to a GLOBAL. `&gList` is WR1's named-data-symbol\n"
        "address (`26 <gl-tok>` + a relocation pair), not a formal's sub-object,\n"
        "so this must refuse for a DIFFERENT reason than the target's.",
        HDR + """
BE gList;
void fn(H* h) {
    h->mSize = 1;
    BE& l = gList;
    l.mNext = &l;
    l.mPrev = &l;
}
""",
    ),
    "b_bind_stacklocal": (
        "AXIS A — the bind is to a STACK LOCAL aggregate, which is a frame\n"
        "object. The body is not a leaf at all and must refuse.",
        HDR + """
void fn(H* h) {
    BE tmp;
    BE& l = tmp;
    l.mNext = &l;
    l.mPrev = &l;
    h->mSize = 1;
}
""",
    ),
    # ---- AXIS B: HOW OFTEN THE BOUND NAME IS USED ------------------------
    "b_use1": (
        "AXIS B — the bound name stands in exactly ONE store's base position,\n"
        "and is NOT the stored value. One use, one base symbol's worth.",
        HDR + """
void fn(H* h, BE* p) {
    h->mSize = 2;
    BE& l = h->mListHead;
    l.mNext = p;
}
""",
    ),
    "b_use2": (
        "AXIS B — the bound name in TWO stores' base position, value a formal.\n"
        "Separates 'used as a base' from 'used as a value': the target uses it\n"
        "as both and cannot tell the two roles apart.",
        HDR + """
void fn(H* h, BE* p) {
    h->mSize = 2;
    BE& l = h->mListHead;
    l.mNext = p;
    l.mPrev = p;
}
""",
    ),
    "b_use_value_only": (
        "AXIS B — the bound name used ONLY as a VALUE, never in a base position.\n"
        "This is the half `parse_store_stmt` already refuses through F2 rather\n"
        "than through the base gate, so it isolates obligation 1 from obligation\n"
        "2 (w-f23 §5.1).",
        """struct BE { BE* mNext; BE* mPrev; };
struct H {
    BE* mSpare;        // 0
    H* mUsedHead;      // 4
    BE mListHead;      // 8
    unsigned mSize;    // 16
    unsigned mCount;   // 20
};

void fn(H* h) {
    BE& l = h->mListHead;
    h->mSpare = &l;
    h->mSize = 3;
}
""",
    ),
    # ---- AXIS C: WHERE THE BIND SITS AMONG THE STATEMENTS ----------------
    "b_bind_first": (
        "AXIS C — the bind is the FIRST statement of the body. The target binds\n"
        "in the MIDDLE; nothing on record varies this.",
        HDR + """
void fn(H* h, unsigned s) {
    BE& l = h->mListHead;
    l.mNext = &l;
    l.mPrev = &l;
    h->mSize = s;
    h->mCount = 0;
}
""",
    ),
    "b_bind_last": (
        "AXIS C — the bind comes AFTER every other store. Same statements as\n"
        "`b_bind_first`, permuted, so the pair is a within-grid cross-check\n"
        "(board #1174: the corpus stayed green through two wrong emits and a\n"
        "hand-written cross-product is what caught them).",
        HDR + """
void fn(H* h, unsigned s) {
    h->mSize = s;
    h->mCount = 0;
    BE& l = h->mListHead;
    l.mNext = &l;
    l.mPrev = &l;
}
""",
    ),
    # ---- AXIS D: THE KIND OF REFERENCE -----------------------------------
    "b_const_ref": (
        "AXIS D — a CONST reference. Cannot be stored through, so it exercises\n"
        "the value role under a qualifier the `2C`/volatile gates care about.",
        """struct BE { BE* mNext; BE* mPrev; };
struct H {
    const BE* mSpare;  // 0
    H* mUsedHead;      // 4
    BE mListHead;      // 8
    unsigned mSize;    // 16
    unsigned mCount;   // 20
};

void fn(H* h) {
    const BE& l = h->mListHead;
    h->mSpare = &l;
    h->mSize = 4;
}
""",
    ),
    "b_subobject": (
        "AXIS D — a reference to a SCALAR sub-object (a pointer member of the\n"
        "bound aggregate), not to the aggregate. The bound thing is the store's\n"
        "whole destination rather than its base, so the offset run is empty.",
        HDR + """
void fn(H* h) {
    BE*& n = h->mListHead.mNext;
    n = &h->mListHead;
    h->mSize = 5;
}
""",
    ),
    "b_dead": (
        "AXIS D — the bind is DEAD: bound and never used. c2 may delete the\n"
        "statement entirely. Paired with `b_dead_ctrl`.",
        HDR + """
void fn(H* h, unsigned s) {
    h->mSize = s;
    BE& l = h->mListHead;
    h->mCount = 0;
    h->mFreeHead = h;
}
""",
    ),
    "b_dead_ctrl": (
        "AXIS D — `b_dead` with the bind LINE REMOVED, and nothing else changed.\n"
        "PREREG P8 predicts the two reference bodies are IDENTICAL; if they are\n"
        "not, a dead bind is load-bearing and the production must say so.",
        HDR + """
void fn(H* h, unsigned s) {
    h->mSize = s;
    h->mCount = 0;
    h->mFreeHead = h;
}
""",
    ),
    # ---- AXIS E: THE DISPLACEMENT ----------------------------------------
    "b_off0": (
        "AXIS E — the bound sub-object is at displacement ZERO. Boards #856/#865\n"
        "measured that a `0x26` bind at displacement 0 does NOT make a second\n"
        "store-base value, so this must NOT be read as the target's shape.\n"
        "PREREG P2 registers the exclusion before this cell was compiled.",
        """struct BE { BE* mNext; BE* mPrev; };
struct Z {
    BE mListHead;      // 0   (mNext at 0, mPrev at 4)
    unsigned mSize;    // 8
    unsigned mCount;   // 12
};

void fn(Z* z, unsigned s) {
    BE& l = z->mListHead;
    l.mNext = &l;
    l.mPrev = &l;
    z->mSize = s;
}
""",
    ),
    "b_off0_ctrl": (
        "AXIS E — `b_off0` written DIRECT. #865 predicts this pair is the SAME\n"
        "body where `b_target_bind`/`b_target_direct` is not, which is what makes\n"
        "the displacement and not the bind the axis.",
        """struct BE { BE* mNext; BE* mPrev; };
struct Z {
    BE mListHead;      // 0
    unsigned mSize;    // 8
    unsigned mCount;   // 12
};

void fn(Z* z, unsigned s) {
    z->mListHead.mNext = &z->mListHead;
    z->mListHead.mPrev = &z->mListHead;
    z->mSize = s;
}
""",
    ),
    # ---- AXIS F: A POINTER LOCAL RATHER THAN A REFERENCE -----------------
    "b_ptr_local": (
        "AXIS F — a POINTER local, not a reference. Different C++, and the\n"
        "question the cell asks is whether c1xx spells it as the same `26`\n"
        "store-into-a-local. If it does, `#839` is not about references at all.",
        HDR + """
void fn(H* h, unsigned s) {
    h->mSize = s;
    BE* p = &h->mListHead;
    p->mNext = p;
    p->mPrev = p;
}
""",
    ),
    # ---- AXIS G: THE STORE-RUN SHAPE THE BIND IS CROSSED AGAINST ---------
    "b_leaf_bind": (
        "AXIS G — bind + exactly ONE store. A LEAF, not a run: the run gates\n"
        "(overlap, mixed-kind, the literal pool) are all unreachable, so this\n"
        "isolates the base-position obligation from everything the run adds.",
        HDR + """
void fn(H* h) {
    BE& l = h->mListHead;
    l.mNext = &l;
}
""",
    ),
    "b_run_call": (
        "AXIS G — bind + run + a trailing member call whose argument setup writes\n"
        "no register (board #1129's refined regime gate). The target's own shape,\n"
        "generic: not the xboxheap constructor, so a reading fitted to that one\n"
        "body shows up here as a disagreement.",
        """struct BE { BE* mNext; BE* mPrev; };
struct H {
    H* mFreeHead;
    H* mUsedHead;
    BE mListHead;
    unsigned mSize;
    unsigned mCount;
    void init(unsigned n);
    void grow(unsigned n);
};

void H::init(unsigned n) {
    mSize = n;
    mCount = 0;
    BE& l = mListHead;
    l.mNext = &l;
    l.mPrev = &l;
    grow(n);
}
""",
    ),
    "b_two_binds": (
        "AXIS G — TWO binds in one body, off two different formals. Nothing on\n"
        "record varies the NUMBER of binds, and `#865`'s axis is the number of\n"
        "distinct store-base values, so two binds is the cell that separates\n"
        "'a bind' from 'the binds'.",
        HDR + """
void fn(H* a, H* b) {
    BE& l = a->mListHead;
    BE& m = b->mListHead;
    l.mNext = &l;
    m.mNext = &m;
}
""",
    ),
    # ---- CONTROLS THAT MUST NOT MOVE -------------------------------------
    "b_ctrl_run": (
        "CONTROL — a plain store run, no bind anywhere. `match` today; PREREG\n"
        "floor D1 declines the lane if it is anything else after.",
        HDR + """
void fn(H* h, unsigned s, unsigned c) {
    h->mSize = s;
    h->mCount = c;
}
""",
    ),
    "b_ctrl_runcall": (
        "CONTROL — `w-f23`'s F3 shape with no bind: a store run then a member\n"
        "call. Its key must not change, because this lane touches neither\n"
        "obligation it depends on.",
        """struct BE { BE* mNext; BE* mPrev; };
struct H {
    H* mFreeHead;
    H* mUsedHead;
    BE mListHead;
    unsigned mSize;
    unsigned mCount;
    void init(unsigned n);
    void grow(unsigned n);
};

void H::init(unsigned n) {
    mSize = n;
    mCount = 0;
    mFreeHead = this;
    grow(n);
}
""",
    ),
}


def main() -> int:
    here = os.path.dirname(os.path.abspath(__file__))
    grid = os.path.join(here, "grid")
    for name, (why, body) in CELLS.items():
        d = os.path.join(grid, name)
        os.makedirs(d, exist_ok=True)
        banner = "\n".join("// " + l for l in why.split("\n"))
        with open(os.path.join(d, name + ".cpp"), "w") as f:
            f.write("// GRID BIND cell `%s`\n%s\n%s" % (name, banner, body))
    print("wrote %d cells" % len(CELLS))
    return 0


if __name__ == "__main__":
    sys.exit(main())
