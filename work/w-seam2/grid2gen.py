#!/usr/bin/env python3
"""GRID S2 — the DECLARED POST-HOC holdout for the `mr r31,r3` slot rule.

**Post-hoc, and labelled so in its own header rather than in a footnote.** GRID S
was frozen before a cell compiled and is this lane's fit set. This grid was
written *after* reading GRID S's answer, for one purpose: GRID S scores #867's
`nprod - 1 + min(u, 2)` at **32 HIT / 4 MISS**, and every miss is `nprod = 0`.
Two rules explain GRID S equally well —

    R867   stores_before_mr = nprod - 1 + min(u, 2)          nprod >= 1 only
    R0     stores_before_mr = min(u, 1)                      nprod == 0

— and a two-clause rule fitted on the cells that produced it is exactly how all
six refuted allocation keys got written (`w-heap` §4.1.1, board #836/#868). So
this grid holds the levels GRID S varied FIXED and varies the ones it held
fixed:

  * **`nprod = 3`**, which GRID S has no cell of at all;
  * **`u = 2, 4, 5, 6`**, where GRID S has only `u ∈ {0, 1, 3}` — and `u = 2` is
    the level that decides whether `min(u, 2)`'s cap is real or an artifact of
    jumping 1 -> 3;
  * **a produced store written FIRST in source order**, which separates "the
    total count of unproduced stores" from "the LEADING RUN of unproduced stores
    in the final order" (`order::layout_slots`'s own `u`, board #584). Every
    GRID S cell has the two equal, so GRID S cannot tell them apart and no rule
    fitted on it may claim either reading.

The struct EXTENDS GRID S's with four more `unsigned int` members, because the
frozen struct has only three offsets that can carry an unproduced store and this
grid needs seven. That is a difference from GRID S and it is stated here rather
than buried: cells of the two grids are not byte-comparable to each other, only
to their own rule.
"""

import hashlib
import os
import sys

ROOT = os.path.dirname(os.path.abspath(__file__))
GRID = os.path.join(ROOT, "grid2")

PRE = (
    "struct BE { BE* mNext; BE* mPrev; };\n"
    "struct H {\n"
    "  H(unsigned int initSize, unsigned int size);\n"
    "  BE* Alloc(unsigned int);\n"
    "  H* mFreeHead; H* mUsedHead; BE mListHead;\n"
    "  unsigned int mSize; unsigned int mCount; BE mSecond;\n"
    "  unsigned int mFlags; unsigned int mPeak;\n"
    "  unsigned int mA; unsigned int mB; unsigned int mC; unsigned int mD;\n"
    "};\n"
)

# Stores that materialise NOTHING — the value is already in a register (`this` is
# r3, `initSize` r4, `size` r5). Disjoint offsets from the producer pool below.
UNPROD = [
    "mFreeHead = this;",     # 0
    "mUsedHead = this;",     # 4
    "mSize = size;",         # 16
    "mA = initSize;",        # 40
    "mB = size;",            # 44
    "mC = initSize;",        # 48
    "mD = size;",            # 52
]

# Distinct VALUES, because equal constants CSE to one `li` and would silently be
# one producer rather than two — the same identity `scheduled_gpr_run_text` uses.
PROD = [
    "mCount = 0;",           # 20
    "mFlags = 7;",           # 32
    "mPeak = 13;",           # 36
]

CTOR = "H::H(unsigned int initSize, unsigned int size)"


def cell(np, u):
    return "h_np%d_u%d" % (np, u), PROD[:np] + UNPROD[:u]


def main():
    os.makedirs(GRID, exist_ok=True)
    plan = []
    # `nprod = 3` is entirely outside GRID S; `u = 2, 4, 5, 6` are outside it at
    # every `nprod`. `u = 0, 1, 3` are re-run only at `nprod = 3`.
    for np, us in ((0, (2, 4, 5, 6)),
                   (1, (2, 4, 6)),
                   (2, (2, 4)),
                   (3, (0, 1, 2, 3, 4))):
        for u in us:
            plan.append(cell(np, u))

    # The ORDER probe: a produced store written FIRST in source order, with the
    # unproduced ones after it. If `u` means "the leading run of unproduced
    # stores in the FINAL order" this cell's `u` is whatever c2's schedule
    # produces; if it means "the total count of unproduced stores" it is 3. The
    # two readings differ here and nowhere in GRID S.
    plan.append(("h_ordmix",
                 ["mCount = 0;", "mFreeHead = this;", "mUsedHead = this;",
                  "mSize = size;"]))
    # And its mirror with TWO producers, so the probe is not a single cell.
    plan.append(("h_ordmix2",
                 ["mCount = 0;", "mFlags = 7;", "mFreeHead = this;",
                  "mUsedHead = this;", "mSize = size;"]))

    lines = []
    for name, stmts in plan:
        d = os.path.join(GRID, name)
        os.makedirs(d, exist_ok=True)
        text = PRE + "%s {\n%s  Alloc(initSize);\n}\n" % (
            CTOR, "".join("  %s\n" % s for s in stmts))
        with open(os.path.join(d, name + ".cpp"), "w") as f:
            f.write(text)
        lines.append("%s  %s/%s.cpp" % (
            hashlib.sha256(text.encode()).hexdigest(), name, name))
    lines.sort()
    with open(os.path.join(ROOT, "GRID2.sha256"), "w") as f:
        f.write("\n".join(lines) + "\n")
    sys.stderr.write("%d holdout cells\n" % len(lines))


if __name__ == "__main__":
    main()
