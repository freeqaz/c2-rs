#!/usr/bin/env python3
"""gridgen.py — w-heap's TWO frozen grids, generated before anything compiles.

Lane w-heap. Read-only with respect to `crates/`.

# Why this file exists before the first obj

`docs/rungs/2026-08-08-w-front2.md` §6 declined the `xboxheap.cpp` conversion on
grid discipline, naming the axes it would not enumerate:

> F3 alone has store count, store order, argument count, callee kind, receiver
> slot and return-value use free; F2 has the member offset, the element type and
> the reference-bind spelling. A widening without its grid is how #232 became a
> live wrong emit for 255 commits under a green gate.

So both grids are enumerated **structurally** here, the manifest is sha256'd and
committed, and only then is a cell compiled. The axes below are that list, plus
one this lane adds (F2's **use count**, because board #844 states the allocation
of `xboxheap`'s run is settled by use count alone and a grid that never varies
it cannot see that clause fire).

# The one axis that decides the lane

**GRID F3 axis C — the call's argument-setup cost.** Board #870 measured that a
call taking an argument moves three things at once (the object parks in a
volatile, the store base changes mid-run, the constant pool re-ranks) and that
none of the three is expressible in `alloc::allocate`'s inputs. Board #866
measured 12/12 IDENT transfer where the call is nullary. So axis C is not a
breadth axis, it is the regime boundary, and every other axis is held at
`xboxheap`'s own value while it moves.

`x6` — the cell `w-front2` §3.2 offers as "a strictly smaller sub-target" — sits
at C >= 1 with a FREE callee, and `xboxheap` sits at C = 0. They are in
different regimes. That is measured in `work/w-heap/ref/xboxheap/dis.txt` and
restated in cells `f3_c0_*` vs `f3_c1_*` here.
"""

import hashlib
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))

# ---------------------------------------------------------------------------
# The shared preamble. Field layout is `xboxheap`'s own CXboxHeap, verbatim from
# the reference obj's offsets: mFreeHead 0, mUsedHead 4, mListHead 8 (mNext 8,
# mPrev 12), mSize 16, mCount 20.
# ---------------------------------------------------------------------------
PRE = """struct BE { BE* mNext; BE* mPrev; };
"""


def hh(body, extra=""):
    return PRE + extra + body


# ===========================================================================
# GRID F3 — "a call after a store run", six structural axes
# ===========================================================================
#
#   A. store count before the call        0, 1, 3, 6
#   B. store base                         one base (`this`); two bases
#   C. ARGUMENT SETUP COST of the call    0 (args already in place), 1, 2
#   D. callee kind                        free fn; member on `this`; member on other
#   E. receiver slot 0                    n/a; `this`; a formal
#   F. return-value use                   void; ctor implicit `return this`;
#                                         `return <call>`; discarded int
#
# The legal enumeration, not the raw product: E is determined by D (a free
# function has no receiver, a member on `this` puts `this` in slot 0, a member on
# another object puts that object there), and C=0 is reachable only when every
# actual is already in its own slot register.

F3 = {}


def f3(name, decl, body, extra=""):
    F3[name] = hh(
        "struct H {\n"
        "    %s\n"
        "    H* mFreeHead; H* mUsedHead; BE mListHead; unsigned int mSize; unsigned int mCount;\n"
        "};\n" % decl
        + body,
        extra,
    )


# --- axis A: store count, held at C=0 / D=member-on-this / F=ctor -----------
# c2 passes (this=r3, initSize=r4) with no setup at all, so C=0.
for n, stores in [
    ("a0", ""),
    ("a1", "    mSize = size;\n"),
    ("a3", "    mSize = size;\n    mFreeHead = this;\n    mUsedHead = this;\n"),
    (
        "a6",
        "    mSize = size;\n    mFreeHead = this;\n    mCount = 0;\n"
        "    mUsedHead = this;\n    mListHead.mNext = 0;\n    mListHead.mPrev = 0;\n",
    ),
]:
    f3(
        "f3_%s_c0_dthis_fctor" % n,
        "H(unsigned int a, unsigned int b);\n    BE* Alloc(unsigned int);",
        "H::H(unsigned int initSize, unsigned int size) {\n%s    Alloc(initSize);\n}\n" % stores,
    )

# --- axis B: two store bases, C=0 ------------------------------------------
f3(
    "f3_b2_c0_dthis_fctor",
    "H(unsigned int a, unsigned int b, H* q);\n    BE* Alloc(unsigned int);",
    "H::H(unsigned int initSize, unsigned int size, H* q) {\n"
    "    mSize = size;\n    q->mFreeHead = this;\n    mUsedHead = this;\n"
    "    Alloc(initSize);\n}\n",
)

# --- axis C: THE REGIME BOUNDARY -------------------------------------------
# C=0 : member call on `this`, argument already in r4          -> no setup
# C=1 : free function taking one formal                        -> `mr r3,r4`
# C=1b: member call on `this` whose argument is the SECOND formal (r5 -> r4)
# C=2 : free function taking two formals in swapped order      -> two moves
f3(
    "f3_a3_c0_dthis_fctor__dup",
    "H(unsigned int a, unsigned int b);\n    BE* Alloc(unsigned int);",
    "H::H(unsigned int initSize, unsigned int size) {\n"
    "    mSize = size;\n    mFreeHead = this;\n    mUsedHead = this;\n"
    "    Alloc(initSize);\n}\n",
)
f3(
    "f3_a3_c1_dfree_fctor",
    "H(unsigned int a, unsigned int b);",
    "H::H(unsigned int initSize, unsigned int size) {\n"
    "    mSize = size;\n    mFreeHead = this;\n    mUsedHead = this;\n"
    "    g1(initSize);\n}\n",
    "extern void g1(unsigned int);\n",
)
f3(
    "f3_a3_c1b_dthis_fctor",
    "H(unsigned int a, unsigned int b);\n    BE* Alloc(unsigned int);",
    "H::H(unsigned int initSize, unsigned int size) {\n"
    "    mSize = size;\n    mFreeHead = this;\n    mUsedHead = this;\n"
    "    Alloc(size);\n}\n",
)
f3(
    "f3_a3_c2_dfree_fctor",
    "H(unsigned int a, unsigned int b);",
    "H::H(unsigned int initSize, unsigned int size) {\n"
    "    mSize = size;\n    mFreeHead = this;\n    mUsedHead = this;\n"
    "    g2(size, initSize);\n}\n",
    "extern void g2(unsigned int, unsigned int);\n",
)

# --- axis D/E: callee kind and receiver slot, all at C=0 --------------------
# D=member on ANOTHER object: slot 0 is a formal, so `mr r3,r5` -> that is C>=1
# by construction and the cell records the coupling rather than pretending to
# separate them.
f3(
    "f3_a3_c1_dother_fctor",
    "H(unsigned int a, unsigned int b, H* q);\n    BE* Alloc(unsigned int);",
    "H::H(unsigned int initSize, unsigned int size, H* q) {\n"
    "    mSize = size;\n    mFreeHead = this;\n    mUsedHead = this;\n"
    "    q->Alloc(initSize);\n}\n",
)
# D=free, ZERO arguments: the #869 cell — expected to TAIL-CALL, not frame.
f3(
    "f3_a3_c0_dfree_fvoid",
    "static void s(H* h, unsigned int a);",
    "extern void g0();\n"
    "void hf(H* h, unsigned int u, unsigned int v) {\n"
    "    h->mSize = v;\n    h->mFreeHead = h;\n    h->mUsedHead = h;\n"
    "    g0();\n}\n",
)

# --- axis F: return-value use, at C=0 / D=member-on-this -------------------
f3(
    "f3_a3_c0_dthis_fvoid",
    "void m(unsigned int a, unsigned int b);\n    BE* Alloc(unsigned int);",
    "void H::m(unsigned int initSize, unsigned int size) {\n"
    "    mSize = size;\n    mFreeHead = this;\n    mUsedHead = this;\n"
    "    Alloc(initSize);\n}\n",
)
f3(
    "f3_a3_c0_dthis_fretcall",
    "BE* m(unsigned int a, unsigned int b);\n    BE* Alloc(unsigned int);",
    "BE* H::m(unsigned int initSize, unsigned int size) {\n"
    "    mSize = size;\n    mFreeHead = this;\n    mUsedHead = this;\n"
    "    return Alloc(initSize);\n}\n",
)
f3(
    "f3_a3_c0_dthis_fdiscardint",
    "void m(unsigned int a, unsigned int b);\n    BE* Alloc(unsigned int);",
    "void H::m(unsigned int initSize, unsigned int size) {\n"
    "    mSize = size;\n    mFreeHead = this;\n    mUsedHead = this;\n"
    "    BE* r = Alloc(initSize); (void)r;\n}\n",
)

# --- the LEAF controls: the same run with NO call at all -------------------
# `L` in board #866's vocabulary. Needed to say whether the run TRANSFERS, which
# is a comparison and not a verdict on one cell.
f3(
    "f3_leaf_a3",
    "H(unsigned int a, unsigned int b);",
    "H::H(unsigned int initSize, unsigned int size) {\n"
    "    mSize = size;\n    mFreeHead = this;\n    mUsedHead = this;\n}\n",
)
f3(
    "f3_leaf_a6",
    "H(unsigned int a, unsigned int b);",
    "H::H(unsigned int initSize, unsigned int size) {\n"
    "    mSize = size;\n    mFreeHead = this;\n    mCount = 0;\n"
    "    mUsedHead = this;\n    mListHead.mNext = 0;\n    mListHead.mPrev = 0;\n}\n",
)


# ===========================================================================
# GRID F2 — "a member's address as a stored value", four structural axes
# ===========================================================================
#
#   G. member offset of the addressed sub-object   0, 8, 16
#   H. how the address is bound                    direct `&m`; reference bind
#   I. where the address is stored                 into the addressed object
#                                                  itself (xboxheap's self-link);
#                                                  into a different member;
#                                                  into a different object
#   J. USE COUNT of the address                    1, 2, 3
#
# Every F2 cell is a LEAF (no call). F2 x F3 is deliberately NOT crossed here:
# the two are separated by `w-front2`'s own orthogonality control (x3 has F2 and
# no call, x6 has a call and no F2, both refuse) and crossing them before each
# is settled alone is how a grid stops being able to attribute a failure.

F2 = {}


def f2(name, decl, body, extra=""):
    F2[name] = hh(
        "struct H {\n"
        "    %s\n"
        "    H* mFreeHead; H* mUsedHead; BE mListHead; unsigned int mSize; unsigned int mCount;\n"
        "    BE mSecond;\n"
        "};\n" % decl,
        extra,
    ) + body


CTOR = "H::H(unsigned int initSize, unsigned int size) {\n%s}\n"
DECL = "H(unsigned int a, unsigned int b);"

# --- axis G: offset 0 / 8 / 16 (the addressed object's own offset) ---------
# offset 8  = mListHead, xboxheap's own
f2("f2_g8_hdirect_iself_j2", DECL, CTOR % (
    "    mSize = size;\n    mFreeHead = this;\n    mUsedHead = this;\n"
    "    mListHead.mNext = &mListHead;\n    mListHead.mPrev = &mListHead;\n"))
# offset 0  = the whole object's own head, address is `this` + 0 -> NO addi
f2("f2_g0_hdirect_iself_j2", DECL, CTOR % (
    "    mSize = size;\n    mFreeHead = (H*)this;\n"
    "    mListHead.mNext = (BE*)this;\n    mListHead.mPrev = (BE*)this;\n"))
# offset 24 = mSecond, a second BE further in
f2("f2_g24_hdirect_iself_j2", DECL, CTOR % (
    "    mSize = size;\n    mFreeHead = this;\n    mUsedHead = this;\n"
    "    mSecond.mNext = &mSecond;\n    mSecond.mPrev = &mSecond;\n"))

# --- axis H: the reference bind spelling (board #839 is live here) ---------
f2("f2_g8_href_iself_j2", DECL, CTOR % (
    "    mSize = size;\n    mFreeHead = this;\n    mUsedHead = this;\n"
    "    BE& h = mListHead;\n    h.mNext = &h;\n    h.mPrev = &h;\n"))

# --- axis I: destination of the address ------------------------------------
# I=other-member: the address of mListHead stored into mFreeHead/mUsedHead
f2("f2_g8_hdirect_iothermember_j2", DECL, CTOR % (
    "    mSize = size;\n    mFreeHead = (H*)&mListHead;\n    mUsedHead = (H*)&mListHead;\n"))
# I=other-object: stored through a second base pointer
F2["f2_g8_hdirect_iotherobj_j2"] = PRE + (
    "struct H {\n    H(unsigned int a, unsigned int b, BE* q);\n"
    "    H* mFreeHead; H* mUsedHead; BE mListHead; unsigned int mSize; unsigned int mCount;\n"
    "    BE mSecond;\n};\n"
    "H::H(unsigned int initSize, unsigned int size, BE* q) {\n"
    "    mSize = size;\n    q->mNext = &mListHead;\n    q->mPrev = &mListHead;\n}\n")

# --- axis J: use count of the address, 1 / 2 / 3 ---------------------------
# J is the clause board #844 says settles xboxheap's allocation on its own, so
# it is varied against a `li` competitor whose own use count is held at 1.
f2("f2_g8_hdirect_iself_j1", DECL, CTOR % (
    "    mCount = 0;\n    mListHead.mNext = &mListHead;\n"))
f2("f2_g8_hdirect_iself_j2c", DECL, CTOR % (
    "    mCount = 0;\n    mListHead.mNext = &mListHead;\n    mListHead.mPrev = &mListHead;\n"))
f2("f2_g8_hdirect_iself_j3", DECL, CTOR % (
    "    mCount = 0;\n    mListHead.mNext = &mListHead;\n    mListHead.mPrev = &mListHead;\n"
    "    mSecond.mNext = &mListHead;\n"))
# J with the literal at TWO uses and the address at one — the tie-break's other
# side, so the grid cannot fit a rule that only ever sees the address winning.
f2("f2_g8_hdirect_iself_j1_lit2", DECL, CTOR % (
    "    mCount = 0;\n    mSize = 0;\n    mListHead.mNext = &mListHead;\n"))

# --- the xboxheap cell itself, exact ---------------------------------------
F2["f2_xboxheap_exact"] = PRE + (
    "struct H {\n    H(unsigned int a, unsigned int b);\n    BE* Alloc(unsigned int);\n"
    "    H* mFreeHead; H* mUsedHead; BE mListHead; unsigned int mSize; unsigned int mCount;\n};\n"
    "H::H(unsigned int initSize, unsigned int size) {\n"
    "    mSize = size;\n    mFreeHead = this;\n    mCount = 0;\n    mUsedHead = this;\n"
    "    BE& listHead = mListHead;\n    listHead.mNext = &listHead;\n"
    "    listHead.mPrev = &listHead;\n    Alloc(initSize);\n}\n")


def main():
    out = os.path.join(HERE, "grid")
    lines = []
    for grid, cells in (("F3", F3), ("F2", F2)):
        for name in sorted(cells):
            d = os.path.join(out, name)
            os.makedirs(d, exist_ok=True)
            src = os.path.join(d, name + ".cpp")
            text = "// %s — w-heap GRID %s, generated by gridgen.py. DO NOT EDIT.\n%s" % (
                name, grid, cells[name])
            with open(src, "w") as f:
                f.write(text)
            lines.append("%s  %s/%s.cpp" % (
                hashlib.sha256(text.encode()).hexdigest(), name, name))
    man = os.path.join(HERE, "GRID.sha256")
    with open(man, "w") as f:
        f.write("\n".join(lines) + "\n")
    print("GRID F3: %d cells" % len(F3))
    print("GRID F2: %d cells" % len(F2))
    print("total:   %d cells, manifest %s" % (len(F3) + len(F2), man))


if __name__ == "__main__":
    sys.exit(main())
