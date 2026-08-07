#!/usr/bin/env python3
"""gridgen.py — w-carrier's frozen GRID K, one directory per cell (#1045).

Every cell is a whole TU at the WORKLOAD's own `/GR /O1 /Oi /EHsc` (#1112).
Every accept candidate is paired with a `_c` CONTROL that is the same body with
the reference bind removed and the member named directly — because board #1128
says the two spellings emit DIFFERENT bodies, and a grid that graded only the
bind half could not tell a carrier from a collapse.

The axes this grid varies are named in `work/w-carrier/PREREG.md` §4, and the
one it deliberately refuses to hold fixed is **the number of stores between the
bind and its first use** — the quantity `order::layout_slots` computes its
symbol-crossing count over. Four earlier grids held it fixed.

The manifest `GRID.sha256` is committed BEFORE the first `cl.exe`.
"""
import hashlib
import os
import sys

ROOT = os.path.dirname(os.path.abspath(__file__))
GRID = os.path.join(ROOT, "grid")

# The struct every cell shares, with its offsets stated. Same field order as
# `work/w-bind/grid/`'s so the published displacements carry over.
DECL = """struct BE { BE* mNext; BE* mPrev; };
struct H {
    H* mFreeHead;      // 0
    H* mUsedHead;      // 4
    BE mListHead;      // 8   (mNext at 8, mPrev at 12)
    unsigned mSize;    // 16
    unsigned mCount;   // 20
    BE mSecond;        // 24  (mNext at 24, mPrev at 28)
    BE* mSpare;        // 32
    unsigned mA;       // 36
    unsigned mB;       // 40
};
"""

CTOR_DECL = """struct BE { BE* mNext; BE* mPrev; };
struct H {
    H* mFreeHead;      // 0
    H* mUsedHead;      // 4
    BE mListHead;      // 8   (mNext at 8, mPrev at 12)
    unsigned mSize;    // 16
    unsigned mCount;   // 20
    BE* mSpare;        // 24
    H(unsigned initSize, unsigned size);
    BE* AllocatePageBlock(unsigned n);
};
"""

# (name, note, decl, body-text). `fn` bodies take `(H* h, BE* p)` unless the
# body text starts with a full signature.
CELLS = []


def cell(name, note, body, decl=DECL, sig="void fn(H* h, BE* p)"):
    CELLS.append((name, note, decl, sig, body))


# ---- AXIS 1: the ROLE of the bound name -----------------------------------
cell("k_base1", "base only, ONE use, one constant producer beside it",
     "h->mSize = 2;\n    BE& l = h->mListHead;\n    l.mNext = p;")
cell("k_base1_c", "control: the same body written directly",
     "h->mSize = 2;\n    h->mListHead.mNext = p;")
cell("k_base2", "base only, TWO uses, one constant producer",
     "h->mSize = 2;\n    BE& l = h->mListHead;\n    l.mNext = p;\n    l.mPrev = p;")
cell("k_base2_c", "control",
     "h->mSize = 2;\n    h->mListHead.mNext = p;\n    h->mListHead.mPrev = p;")
cell("k_base0p", "base only, NO producer anywhere in the run",
     "h->mFreeHead = h;\n    BE& l = h->mListHead;\n    l.mNext = p;\n    l.mPrev = p;")
cell("k_base0p_c", "control",
     "h->mFreeHead = h;\n    h->mListHead.mNext = p;\n    h->mListHead.mPrev = p;")
cell("k_val1", "VALUE only — the bound name is never a base",
     "BE& l = h->mListHead;\n    h->mSpare = &l;\n    h->mSize = 3;")
cell("k_val1_c", "control",
     "h->mSpare = &h->mListHead;\n    h->mSize = 3;")
cell("k_both1", "base AND value, one use each, no other producer — a LEAF",
     "BE& l = h->mListHead;\n    l.mNext = &l;")
cell("k_both1_c", "control",
     "h->mListHead.mNext = &h->mListHead;")
cell("k_both2", "base AND value, TWO uses, still the only producer",
     "BE& l = h->mListHead;\n    l.mNext = &l;\n    l.mPrev = &l;")
cell("k_both2_c", "control",
     "h->mListHead.mNext = &h->mListHead;\n    h->mListHead.mPrev = &h->mListHead;")

# ---- AXIS 2: the producers beside it, and their KIND (#836/#868's axis) ----
cell("k_mix_c1", "MIXED: an interior address at 1 use beside a literal at 1",
     "BE& l = h->mListHead;\n    l.mNext = &l;\n    h->mCount = 0;")
cell("k_mix_c1_c", "control",
     "h->mListHead.mNext = &h->mListHead;\n    h->mCount = 0;")
cell("k_mix_c2", "MIXED at the TARGET's counts: address 2 uses, literal 1",
     "BE& l = h->mListHead;\n    l.mNext = &l;\n    l.mPrev = &l;\n    h->mCount = 0;")
cell("k_mix_c2_c", "control",
     "h->mListHead.mNext = &h->mListHead;\n    h->mListHead.mPrev = &h->mListHead;\n    h->mCount = 0;")
cell("k_2const", "base-only bind beside TWO distinct literals",
     "h->mA = 2;\n    h->mB = 3;\n    BE& l = h->mListHead;\n    l.mNext = p;")
cell("k_2const_c", "control",
     "h->mA = 2;\n    h->mB = 3;\n    h->mListHead.mNext = p;")
cell("k_3const", "base-only bind beside THREE distinct literals",
     "h->mA = 2;\n    h->mB = 3;\n    h->mSize = 4;\n    BE& l = h->mListHead;\n    l.mNext = p;")
cell("k_3const_c", "control",
     "h->mA = 2;\n    h->mB = 3;\n    h->mSize = 4;\n    h->mListHead.mNext = p;")

# ---- AXIS 3: the bind's POSITION in the run -------------------------------
cell("k_pos_first", "the bind is the body's FIRST statement",
     "BE& l = h->mListHead;\n    l.mNext = p;\n    h->mSize = 2;\n    h->mFreeHead = h;")
cell("k_pos_first_c", "control",
     "h->mListHead.mNext = p;\n    h->mSize = 2;\n    h->mFreeHead = h;")
cell("k_pos_last", "the bind AFTER every other store",
     "h->mSize = 2;\n    h->mFreeHead = h;\n    BE& l = h->mListHead;\n    l.mNext = p;")
cell("k_pos_last_c", "control",
     "h->mSize = 2;\n    h->mFreeHead = h;\n    h->mListHead.mNext = p;")

# ---- AXIS 4: the DISPLACEMENT (#856 — a one-byte IL axis) -----------------
cell("k_off24", "bound at +24, not the target's +8",
     "h->mSize = 2;\n    BE& l = h->mSecond;\n    l.mNext = p;\n    l.mPrev = p;")
cell("k_off24_c", "control",
     "h->mSize = 2;\n    h->mSecond.mNext = p;\n    h->mSecond.mPrev = p;")

# ---- AXIS 5: the TAIL — #1129's call --------------------------------------
cell("k_call", "base-only bind, then the ctor call tail (#1129)",
     "mSize = size;\n    BE& l = mListHead;\n    l.mNext = 0;\n    AllocatePageBlock(initSize);",
     CTOR_DECL, "H::H(unsigned initSize, unsigned size)")
cell("k_call_c", "control",
     "mSize = size;\n    mListHead.mNext = 0;\n    AllocatePageBlock(initSize);",
     CTOR_DECL, "H::H(unsigned initSize, unsigned size)")
cell("k_callmix", "the ctor tail with the MIXED kind — the target's own shape",
     "mSize = size;\n    mCount = 0;\n    BE& l = mListHead;\n    l.mNext = &l;\n    AllocatePageBlock(initSize);",
     CTOR_DECL, "H::H(unsigned initSize, unsigned size)")
cell("k_callmix_c", "control",
     "mSize = size;\n    mCount = 0;\n    mListHead.mNext = &mListHead;\n    AllocatePageBlock(initSize);",
     CTOR_DECL, "H::H(unsigned initSize, unsigned size)")

# ---- AXIS 6: the NUMBER of binds ------------------------------------------
cell("k_2binds", "TWO binds off two formals, base-only each",
     "BE& l = a->mListHead;\n    BE& m = b->mListHead;\n    l.mNext = q;\n    m.mNext = q;",
     DECL, "void fn(H* a, H* b, BE* q)")
cell("k_2binds_c", "control",
     "a->mListHead.mNext = q;\n    b->mListHead.mNext = q;",
     DECL, "void fn(H* a, H* b, BE* q)")
cell("k_2binds_same", "TWO binds off the SAME formal at two offsets",
     "BE& l = h->mListHead;\n    BE& m = h->mSecond;\n    l.mNext = p;\n    m.mNext = p;")
cell("k_2binds_same_c", "control",
     "h->mListHead.mNext = p;\n    h->mSecond.mNext = p;")

# ---- AXIS 7: the base FORMAL ----------------------------------------------
cell("k_nonthis", "bound off the SECOND pointer formal, base-only",
     "a->mSize = 2;\n    BE& l = b->mListHead;\n    l.mNext = q;",
     DECL, "void fn(H* a, H* b, BE* q)")
cell("k_nonthis_c", "control",
     "a->mSize = 2;\n    b->mListHead.mNext = q;",
     DECL, "void fn(H* a, H* b, BE* q)")

# ---- AXIS 8: the axis the four earlier grids HELD FIXED --------------------
# stores through the OTHER symbol between the bind and its first use.
cell("k_gap0", "bind, then immediately its use",
     "BE& l = h->mListHead;\n    l.mNext = p;\n    h->mSize = 2;")
cell("k_gap1", "bind, ONE store on the other symbol, then its use",
     "BE& l = h->mListHead;\n    h->mSize = 2;\n    l.mNext = p;")
cell("k_gap2", "bind, TWO stores on the other symbol, then its use",
     "BE& l = h->mListHead;\n    h->mSize = 2;\n    h->mFreeHead = h;\n    l.mNext = p;")
cell("k_gap3", "bind, THREE stores on the other symbol, then its use",
     "BE& l = h->mListHead;\n    h->mSize = 2;\n    h->mFreeHead = h;\n    h->mUsedHead = h;\n    l.mNext = p;")
cell("k_gap0_c", "control", "h->mListHead.mNext = p;\n    h->mSize = 2;")
cell("k_gap1_c", "control", "h->mSize = 2;\n    h->mListHead.mNext = p;")
cell("k_gap2_c", "control",
     "h->mSize = 2;\n    h->mFreeHead = h;\n    h->mListHead.mNext = p;")
cell("k_gap3_c", "control",
     "h->mSize = 2;\n    h->mFreeHead = h;\n    h->mUsedHead = h;\n    h->mListHead.mNext = p;")

# ---- The DECLINED pair, carried so the decline is graded not asserted ------
cell("k_off0", "the ZERO-offset bind — w-bind excluded it, and it stays excluded",
     "h->mSize = 2;\n    H*& f = h->mFreeHead;\n    f = h;",
     DECL, "void fn(H* h, BE* p)")
cell("k_off0_c", "its twin — w-bind measured these two as byte-IDENTICAL",
     "h->mSize = 2;\n    h->mFreeHead = h;")
cell("k_dead", "a bind nothing reads — w-bind declined it and so does this lane",
     "h->mSize = 2;\n    BE& l = h->mListHead;\n    h->mFreeHead = h;")
cell("k_dead_c", "its twin — byte-IDENTICAL, which is why the decline stands",
     "h->mSize = 2;\n    h->mFreeHead = h;")

# ---- The CONTROLS that must not stop matching (floor D1) -------------------
cell("k_ctrl_run", "a plain store run, no bind. MUST stay `match`",
     "h->mSize = 2;\n    h->mCount = 3;")
cell("k_ctrl_leaf", "a plain store leaf, no bind. MUST stay `match`",
     "h->mSize = 2;")
cell("k_ctrl_ctorcall", "#844's composition with no bind. MUST stay `match`",
     "mSize = size;\n    mCount = 0;\n    AllocatePageBlock(initSize);",
     CTOR_DECL, "H::H(unsigned initSize, unsigned size)")

# ---- The TARGET itself, and its twin ---------------------------------------
TARGET_DECL = """struct BE { BE* mNext; BE* mPrev; };
struct CXboxHeap {
    CXboxHeap* mFreeHead;
    CXboxHeap* mUsedHead;
    BE  mListHead;
    unsigned mSize;
    unsigned mCount;
    CXboxHeap(unsigned initSize, unsigned size);
    BE* AllocatePageBlock(unsigned n);
};
"""
cell("k_target",
     "`src/xdk/nuispeech/xboxheap.cpp`'s ctor in its SHIPPED spelling",
     "mSize = size;\n    mFreeHead = this;\n    mCount = 0;\n    mUsedHead = this;\n"
     "    auto& listHead = mListHead;\n    listHead.mNext = &listHead;\n"
     "    listHead.mPrev = &listHead;\n    AllocatePageBlock(initSize);",
     TARGET_DECL, "CXboxHeap::CXboxHeap(unsigned initSize, unsigned size)")
cell("k_target_direct",
     "the same ctor WITHOUT the bind — #1128's four-word control",
     "mSize = size;\n    mFreeHead = this;\n    mCount = 0;\n    mUsedHead = this;\n"
     "    mListHead.mNext = &mListHead;\n    mListHead.mPrev = &mListHead;\n"
     "    AllocatePageBlock(initSize);",
     TARGET_DECL, "CXboxHeap::CXboxHeap(unsigned initSize, unsigned size)")


def main():
    os.makedirs(GRID, exist_ok=True)
    lines = []
    for name, note, decl, sig, body in CELLS:
        d = os.path.join(GRID, name)
        os.makedirs(d, exist_ok=True)
        text = (
            f"// GRID K cell `{name}` — w-carrier, board #1199.\n"
            f"// {note}\n"
            f"// Compiled at the WORKLOAD's own /GR /O1 /Oi /EHsc (#1112).\n"
            f"{decl}\n{sig} {{\n    {body}\n}}\n"
        )
        path = os.path.join(d, name + ".cpp")
        with open(path, "w") as f:
            f.write(text)
        h = hashlib.sha256(text.encode()).hexdigest()
        lines.append(f"{h}  work/w-carrier/grid/{name}/{name}.cpp")
    with open(os.path.join(ROOT, "GRID.sha256"), "w") as f:
        f.write("\n".join(sorted(lines)) + "\n")
    print(f"{len(CELLS)} cells")


if __name__ == "__main__":
    sys.exit(main())
