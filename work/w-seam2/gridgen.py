#!/usr/bin/env python3
"""GRID S — the composition seam's frozen grid, one directory per cell (#1045).

**The axis vocabulary is `scripts/sweep.d/88-store-run-call.py`'s, reused rather
than re-invented** (the brief's own instruction, and `w-gen` §3 is where each
level's justification lives). Same struct `H`, same member offsets, same run
names, same setup names, same form names. A second vocabulary over one family is
`GAPS.md` §6's "one fact, two locators" applied to grids.

What this grid varies that the generator's cross does not: nothing structural.
What it does differently is the **profile** — every cell here is compiled at the
WORKLOAD's own `/GR /O1 /Oi /EHsc` (board #1112), where the sweep's own cross runs
at `/Ox /GS- /c`. Those are different populations and the two instruments are
kept apart on purpose.

The cross is deliberately small and the REFUSAL CONTROLS are half of it. A grid
that only enumerated the cells the seam accepts could not tell "the seam is right"
from "the seam accepts everything".

    S-A  the ACCEPT cross      run-kind x width x setup, form = fctor, bind off
    S-R  the REFUSAL controls  one cell per boundary the seam claims to hold
    S-L  the LEAF controls     the same run with NO call, which master already
                               emits byte-exactly — so "the run transfers" is a
                               comparison and not a claim about one cell (#866)
"""

import hashlib
import os
import sys

ROOT = os.path.dirname(os.path.abspath(__file__))
GRID = os.path.join(ROOT, "grid")

# `xboxheap`'s own CXboxHeap layout, extended exactly as `w-gen` extends it so
# that no two axes ever write the same offset — a dead store makes c2 emit ONE,
# which silently shortens a run (`82-store-run.py` §8).
#
#   mFreeHead 0 · mUsedHead 4 · mListHead 8 (mNext 8, mPrev 12) · mSize 16
#   mCount 20 · mSecond 24 (mNext 24, mPrev 28) · mFlags 32 · mPeak 36
PRE = (
    "struct BE { BE* mNext; BE* mPrev; };\n"
    "extern BE* g1(unsigned int);\n"
    "struct H {\n"
    "  H(unsigned int initSize, unsigned int size);\n"
    "  void mv(unsigned int initSize, unsigned int size);\n"
    "  BE* mr(unsigned int initSize, unsigned int size);\n"
    "  void md(unsigned int initSize, unsigned int size);\n"
    "  BE* Alloc(unsigned int);\n"
    "  BE* Reset();\n"
    "  H* mFreeHead; H* mUsedHead; BE mListHead;\n"
    "  unsigned int mSize; unsigned int mCount; BE mSecond;\n"
    "  unsigned int mFlags; unsigned int mPeak;\n"
    "};\n"
)

# ---- axis: the RUN, by PRODUCER COUNT (not store count — #1132) -------------
# The `p*` names and their bodies are `w-gen`'s. Only the no-F2 levels appear in
# the ACCEPT cross: an `AddrOf` value is a four-op group `parse_simple_gpr_run`
# declines, so those levels are refusal controls (S-R) and not accept cells.
RUNS_NOF2 = [
    ("p0", []),                                        # 0 producers, 0 stores
    ("pL1", ["mCount = 0;"]),                          # 1 producer, 1 use
    ("pLu2", ["mCount = 0;", "mFlags = 0;"]),          # 1 producer, TWO uses
    ("pL2", ["mCount = 0;", "mFlags = 7;"]),           # 2 producers, same kind
    ("pZ1", ["mListHead.mNext = 0;"]),                 # 1 producer through a sub-object
    ("pZ2", ["mListHead.mNext = 0;", "mListHead.mPrev = 0;"]),
]

# ---- axis: the WIDTH, padded with stores that need NO producer --------------
# Held orthogonal to the producer count: `mSize = size` is a formal already in a
# register and `mFreeHead = this` is r3. A grid where width and producer count
# move together cannot attribute a schedule change to either (#1099's error).
WIDTHS = [
    ("w0", []),
    ("w1", ["mSize = size;"]),
    ("w3", ["mSize = size;", "mFreeHead = this;", "mUsedHead = this;"]),
]

# ---- axis C: what the call's ARGUMENT SETUP writes (#1129) ------------------
# Only the two EMPTY-setup levels are accept cells. The reader's gate is stricter
# than #1129's prose — every slot `i` must already hold `params[i]` — so `c1b`
# (`Alloc(size)`, whose slot 1 is formal 2) refuses in the READER, and it is an
# S-R control rather than an accept cell.
SETUPS_EMPTY = [
    ("c0", "Alloc(initSize)"),   # member on `this`, arg already in r4
    ("c0n", "Reset()"),          # member on `this`, nullary
]

CTOR = "H::H(unsigned int initSize, unsigned int size)"


def body(header, prologue, stmts, tail):
    return "%s {\n%s%s%s}\n" % (
        header, prologue, "".join("  %s\n" % s for s in stmts), tail)


def cells():
    out = []

    # ===== S-A: the ACCEPT cross ============================================
    # run-kind x width x empty-setup, form = fctor, bind OFF. 6 x 3 x 2 = 36.
    for rname, run in RUNS_NOF2:
        for wname, pad in WIDTHS:
            for cname, call in SETUPS_EMPTY:
                stmts = pad + run
                if not stmts:
                    # A ctor whose body is ONLY a call is `w-heap`'s axis-A
                    # zero level — a different production entirely
                    # (`expr-call-in-expr-recv-load-then-plumbing-0x3A`), not a
                    # store run of length 0. Kept, and expected to refuse: a
                    # grid that dropped it could not tell "the seam declines
                    # the empty run" from "the seam was never asked".
                    pass
                out.append((
                    "sa_%s_%s_%s" % (rname, wname, cname),
                    body(CTOR, "", stmts, "  %s;\n" % call),
                ))

    # ===== S-L: the LEAF controls ===========================================
    # The identical run with NO call. Master already emits these byte-exactly, so
    # they are what makes "the run transfers into a framed body" a COMPARISON
    # (#866's own construction) rather than a claim about one cell. They must not
    # move: a seam that changed them has changed the shipped store-run emitter.
    for rname, run in RUNS_NOF2:
        for wname, pad in WIDTHS:
            stmts = pad + run
            if not stmts:
                continue
            out.append((
                "sl_%s_%s_cnone" % (rname, wname),
                body("void H::mv(unsigned int initSize, unsigned int size)",
                     "", stmts, ""),
            ))

    # ===== S-R: the REFUSAL controls ========================================
    # One cell per boundary the seam claims to hold. Every one of these is a
    # body the READER may well accept; what is being graded is that the port
    # still refuses, or still emits what it emitted before.
    W3 = ["mSize = size;", "mFreeHead = this;", "mUsedHead = this;"]
    L1 = ["mCount = 0;"]

    # C — the argument setup. `c1b` writes r4 and still transfers per #1129, but
    # the READER's gate is stricter and refuses it; `c1r3` writes r3 and is
    # #870's broken transfer. Both must refuse, for two DIFFERENT reasons, and
    # the grid records which.
    out.append(("sr_c1b", body(CTOR, "", W3 + L1, "  Alloc(size);\n")))
    out.append(("sr_c1r3", body(CTOR, "", W3 + L1, "  g1(initSize);\n")))

    # F — the return-value use. Only the ctor frames; these three are frame
    # words 0 and TAIL-CALL behind the run (#869/#1131). If the seam ever framed
    # one of them it would be a wrong body, not a gap.
    out.append(("sr_fvoid", body(
        "void H::mv(unsigned int initSize, unsigned int size)",
        "", W3 + L1, "  Alloc(initSize);\n")))
    out.append(("sr_fretcall", body(
        "BE* H::mr(unsigned int initSize, unsigned int size)",
        "", W3 + L1, "  return Alloc(initSize);\n")))
    out.append(("sr_fdiscard", body(
        "void H::md(unsigned int initSize, unsigned int size)",
        "", W3 + L1, "  BE* r = Alloc(initSize); (void)r;\n")))

    # F2 — the `AddrOf` value. `parse_simple_gpr_run` declines the four-op group
    # one step before `order::schedule` or `alloc::allocate` is asked anything,
    # so the seam must REFUSE rather than fall through to `store_leaf_text`'s
    # source-order walk. `pAL` is `xboxheap`'s own mix (addr@2uses + lit@1) and
    # is the cell #868/#836 is live on.
    out.append(("sr_pA1", body(
        CTOR, "", W3 + ["mListHead.mNext = &mListHead;"],
        "  Alloc(initSize);\n")))
    out.append(("sr_pAL", body(
        CTOR, "",
        W3 + ["mCount = 0;", "mListHead.mNext = &mListHead;",
              "mListHead.mPrev = &mListHead;"],
        "  Alloc(initSize);\n")))

    # #839 — the reference-bind spelling. The bind makes the store BASE a local,
    # which `parse_store_stmt` refuses; and `w-heap` §4.2 measured that the two
    # spellings emit DIFFERENT bodies, so a seam that collapsed them would emit
    # the other body's words. Not paid here; the cell pins that it stays refused.
    out.append(("sr_bind", body(
        CTOR, "  BE& lh = mListHead;\n",
        W3 + ["mCount = 0;", "lh.mNext = &lh;", "lh.mPrev = &lh;"],
        "  Alloc(initSize);\n")))

    # The call's RESULT consumed by a store — one of `w-gen`'s 48 over-accept
    # guards (#1141). The reader requires the `4B` discard; this is the backstop
    # cell.
    out.append(("sr_resultused", body(
        CTOR, "", W3, "  mFreeHead = (H*)Alloc(initSize);\n")))

    # TWO calls after the run. That is a Class A/B `CallSeq` with a run in front
    # of it, a strictly larger composition than the one this lane models, and
    # the reader's single-call production refuses it. Pinned so a later widening
    # cannot claim this lane graded it.
    out.append(("sr_twocalls", body(
        CTOR, "", W3 + L1, "  Alloc(initSize);\n  Reset();\n")))

    return out


def main():
    os.makedirs(GRID, exist_ok=True)
    lines = []
    for name, src in cells():
        d = os.path.join(GRID, name)
        os.makedirs(d, exist_ok=True)
        path = os.path.join(d, name + ".cpp")
        text = PRE + src
        with open(path, "w") as f:
            f.write(text)
        lines.append("%s  %s/%s.cpp" % (
            hashlib.sha256(text.encode()).hexdigest(), name, name))
    lines.sort()
    with open(os.path.join(ROOT, "GRID.sha256"), "w") as f:
        f.write("\n".join(lines) + "\n")
    sys.stderr.write("%d cells\n" % len(lines))


if __name__ == "__main__":
    main()
