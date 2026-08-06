#!/usr/bin/env python3
"""board_rows.py — insert this lane's rows #886..#895 into docs/BOARD.md.

Each row goes into the table its verdict belongs in, and the anchors are the
row NUMBERS the rows sit after rather than line numbers, so a peer lane landing
between this file being written and being run cannot silently shift the
insertion. Idempotent: it refuses to run twice.
"""

import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", ".."))
BOARD = os.path.join(ROOT, "docs", "BOARD.md")

OPEN_ROWS = [
    "| **890**<sub>w-spell</sub> | **THE SPELLING AXIS IS A POPULATION OF FOUR GROUPS, AND THE GROUP IS NOT THE MNEMONIC** — sixteen producer spellings x five use-count points x two store-base structures | **MEASURED. 160 selected / 160 reached / 146 GRADED / 14 out-of-regime / 0 compile-failed**, at the workload's own `/O1 /Oi /EHsc /GR`. `G0` `self` wins everywhere; `G1` `cross addi add srawi` wins iff `ru >= 2`; `G2` `extsh lwz` wins iff `cu == 1`; `G3` `sub and or xor neg nor slwi srwi` wins iff `ru >= 2` and `cu == 1` and one store base. **PREREG S2 is a MISS: `self` and `cross` are the SAME instruction — `addi rX,r3,K` — and disagree at 1-vs-1**, so no per-opcode table is the answer. **S5 HIT: `srawi` and `srwi`, one C operation at two signednesses, land in different groups — 6 of 10 cells.** **S3 HIT: no additive key over `ru`, `cu` and a per-spelling bonus fits — 11 residual cells** under an exhaustive search of bonuses in `[-8,8]` | rungs/2026-08-06-w-spell.md §3; `work/w-spell/spellgrid.py`, `spellgrid.out`, `gridS_dis.txt` | **The deliverable, and the brief said it would be if no rule stated.** Three instrument facts it rests on: a `formal` copy has **no producer instruction at all** (10 cells — the value is stored straight out of its own register); a `lwz` at 2+ uses under a bind is **rematerialised** and #644 declines it; and c2 prints `not` for `~u` and `sub` for `u - v`. The grader matches **no per-spelling regex** — it reads the producer's register off its own store and records the mnemonic c2 printed, which is #843's mitigation built in rather than applied after the fact |",
    "| **891**<sub>w-spell</sub> | **THE SAME ADDRESS WRITTEN TWO WAYS GETS TWO REGISTERS** — `(int)&s->inner` and `(int)&q` (where `L& q = s->inner`) compile to the same `addi rX,r3,K`, into the same object, and allocate differently | **MEASURED, 18 cells, 18 graded, and it reproduces a two-lane disagreement exactly.** At `(3 uses vs 5)` and `(2 vs 4)`, with the bind, the layout, the store targets and the source order held fixed: `&s->inner` gives the producer the top register and `&q` gives it to the constant. **X2 HIT** — moving to w-alloc2's own struct layout changes nothing. **X3 control HIT** — all six configurations are `prod` at `(1,1)`, which is why four lanes of 1-vs-1 and 2-vs-1 grids never saw it. The `&q` row reproduces w-alloc2's `F1-r3k5` (`const`) exactly, so **neither lane's obj was wrong — the two cells were not the same cell** | rungs/2026-08-06-w-spell.md §5; `work/w-spell/bisect.py`, `bisect.out`, `gridX_dis.txt` | **A schedule and an allocation decided by a difference with no instruction of its own** — board #856's shape in a second place. The `E` row stops it being read as *\"the bind is the axis\"*: `&s->inner` with **no** bind, storing directly, is `const`, so it is the pair and not either half. The mechanism is **named and not measured** — `&q` reads a bound temp, `&s->inner` recomputes — and `work/w-refbind/ilcmp.py` would settle it in one run |",
    "| **892**<sub>w-spell</sub> | **THE `add` GROUP'S ADVANTAGE IS BOUNDED IN THE CONSTANT'S USE COUNT AFTER ALL** — `add addi srawi` lose at `(2,4) (2,5) (3,5)` and win at `(4,5)` and `(3,3)` | **MEASURED as nine of GRID H's fourteen misses**, at counts GRID S never reached. `cu <= ru + 1` fits every one of them and every GRID S cell, and it is **NOT FITTED HERE ON PURPOSE**: these are the cells that refuted the rule, and the standing instruction after a refutation is that they are not the cells to fit the successor on | rungs/2026-08-06-w-spell.md §4.2; `work/w-spell/holdout_grade.out` | Wants a **frozen** grid at `cu = 6..8` before anybody writes it down as a rule. It is w-alloc2 §4's *\"the bonus is a MAGNITUDE, not an override\"* turning out to hold for a group GRID S made look immune |",
    "| **893**<sub>w-spell</sub> | **#865's RIVAL NEEDS ITS OWN HOLDOUT** — *the schedule pins to source order iff the CONSTANT's store and the PRODUCER's stores have different bases* | **20 of 20 on GRID B**, plus the six discriminating rows w-refbind fitted #865 on — **which is precisely the standing #865 itself had when w-refbind refused to ship it, and #865 is now refuted (#888)** | rungs/2026-08-06-w-spell.md §6; `work/w-spell/basegrid2.py` | **DELIBERATELY NOT PROMOTED.** The population to freeze is named: three runs where the constant's and the producer's bases DO differ while a third run shares one, and base counts above two with the two measured runs held together. Twenty cells of one grid is not a rule, and this project has now killed six keys that had more |",
    "| **894**<sub>w-spell</sub> | **FOUR FRESH MNEMONICS NEVER REACHED THE GRADER, AND THE FROZEN SOURCE IS WHY** — `slw` `srw` `sraw` `lbz` | **26 of 26 out-of-regime cells in GRID H.** The variable shifts were spelled `(u << (v & 31))` and the mask is a second instruction, so the producer's register is defined twice; `(int)s->c0` is `lbz` **then** `extsb` — signed `char` — two definitions again. #644 declines all four | rungs/2026-08-06-w-spell.md §4.5; `work/w-spell/holdout_grade.out` | **The instrument working, and the sha256 pin forbidding a fix.** `sraw` was the single riskiest prediction the class principle made — an algebraic right shift, which the principle puts in the `add` group — and it is **ungraded**, so H4's 7-of-8 does not cover it. Respell as `u << v` without the mask, and an `unsigned char` load |",
]

REFUTED_ROWS = [
    "| **886**<sub>w-spell</sub> | **RULE W** — the two-bit spelling key GRID S states: *(ru>=2 or A) and (cu==1 or B) and (bases==1 or A or B)*, with `A` = the value is stored into the object it points at, or the instruction is a load or an extension, and `B` = an add form or an algebraic right shift | **REFUTED WITHOUT COMPILING ANYTHING — wrong on 7 of 388 cells**, re-graded out of the four prior lanes' own committed disassembly (`w-refbind/holdout_dis.txt`, `bindgrid_dis.txt`, `w-next/allocgrid.out`, `w-seam/grida.out`, `w-alloc2/freshgrid.out`) beside GRID S. All seven are a `self` producer at `cu >= 3`, where RULE W claims immunity to the constant's use count and w-alloc2 §4 had already published that the bonus is a **magnitude** | right 381 / **WRONG 7** / refused 0, against the shipped refusal's right 0 / **WRONG 0** / refused 388 | rungs/2026-08-06-w-spell.md §4.1; `work/w-spell/rule.py`, `fit.py`, `fit.out` |",
    "| **887**<sub>w-spell</sub> | **RULE W2** — RULE W with its one refuted clause replaced by H-self's already-published magnitude `2*ru + 3 > 2*cu` | **REFUTED ON A FROZEN HOLDOUT — 14 misses of 106 graded never-fitted cells.** It is right on **388 of 388** cells of every population on record, including every recorded refutation cell of w-next, w-alloc2, w-refbind and w-seam, and then dies on fresh ones. Predictions and every source's sha256 were committed at `f2fdec8` **before a cell was compiled**; the grader re-checks the sha256 and reads the frozen column. Three miss families: `add addi srawi` at high `cu` (9 — board #892), `subfic` misclassified (3), `self` at `(2,4)`/`(3,5)` (2 — board #891) | fit 388/388; holdout right 92 / **WRONG 14** / refused 0 — it **LOSES to the shipped refusal** and nothing is proposed for shipping | rungs/2026-08-06-w-spell.md §4.2; `work/w-spell/holdout.py`, `holdout_pred.tsv`, `holdout_grade.out` |",
    "| **888**<sub>w-spell</sub> | **BOARD #865** — *the schedule pins to source order iff the body carries more than ONE distinct store-base value* | **REFUTED ON ITS FIRST HOLDOUT — 2 misses of 20 graded, both `N6`**, which is **the exact cell w-refbind §5.2 named and did not build**: three runs across two bases with the constant and the producer sharing one. #865 says pinned; the obj says free; the rival w-refbind named beside it says free and is **20 of 20**. Base counts **3** and **4**, a **derived** base (`S* p = s->nxt;`) and a displacement-0 bind beside a real second base all came back as #865 predicted — it broke only on the cell built to break it | #865 hit 18 / **MISS 2**; the rival hit 20 / MISS 0; 20 frozen, 20 sha256 OK, 20 graded, 0 out-of-regime | rungs/2026-08-06-w-spell.md §6; `work/w-spell/basegrid2.py`, `basegrid2_pred.tsv`, `basegrid2_grade.out` |",
]

DONE_ROWS = [
    "| **889**<sub>w-spell</sub> | **A POPULATION MAPPING, NOT A MATCHER, MANUFACTURED SIX REFUTATIONS — #843's shape where nothing looks broken.** `fit.py`'s first run mapped *\"the cell name does not say none\"* to *two store bases* and scored six `P2-shift-*` cells as RULE W misses. They are `ref-unused`, `ptr-unused`, `ref-other`, `local-int`, `outer-ref` and `val-temp` — **the six modes w-refbind §4 measured as NONE-LIKE** — because an unused bind is deleted by the front end, `S& z = *s` names `r3` itself, and `int w = <expr>` names a value and not a base. The corrected mapping is #865's own predicate applied to the declared modes. **`work/w-spell/fit_v1.out` is committed beside `fit.out`**, before the fix, because a corrected number cannot make this point | RULE W wrong 13 -> **7** of 388; the six are not refutations | rungs/2026-08-06-w-spell.md §4.1 |",
    "| **895**<sub>w-spell</sub> | **THE ALLOCATION READINGS ARE NOT FLAG-CONDITIONAL** — GRID S re-compiled at `/O1 /GS- /c` instead of the workload's `/O1 /Oi /EHsc /GR` | **146 of 146 cells identical in WINNER, in producer MNEMONIC and in EMISSION ORDER; 0 cells graded at only one profile.** Registered as PREREG S7 with the losing branch stated: a single disagreement would have made every allocation figure on this project's record flag-conditional | rungs/2026-08-06-w-spell.md §2, §7; `work/w-spell/s7.sh`, `s7cmp.py`, `s7.out` |",
]


def insert_after_row(lines, marker, rows):
    """Insert `rows` immediately after the LAST consecutive table row of the
    block containing the row that starts with `marker`."""
    idx = None
    for n, l in enumerate(lines):
        if l.startswith(marker):
            idx = n
            break
    if idx is None:
        raise SystemExit("anchor not found: %s" % marker)
    n = idx
    while n + 1 < len(lines) and lines[n + 1].startswith("|"):
        n += 1
    return lines[:n + 1] + rows + lines[n + 1:]


def main():
    text = open(BOARD).read()
    if "**890**<sub>w-spell</sub>" in text:
        print("already inserted — nothing to do")
        return 0
    lines = text.split("\n")
    lines = insert_after_row(lines, "| **353**<sub>w-vocab+w-reach</sub>",
                             OPEN_ROWS)
    lines = insert_after_row(lines, "| **855**<sub>w-magic</sub>",
                             REFUTED_ROWS)
    lines = insert_after_row(lines, "| **769**<sub>w-hash</sub>", DONE_ROWS)
    open(BOARD, "w").write("\n".join(lines))
    print("inserted %d open, %d refuted, %d done"
          % (len(OPEN_ROWS), len(REFUTED_ROWS), len(DONE_ROWS)))
    return 0


if __name__ == "__main__":
    sys.exit(main())
