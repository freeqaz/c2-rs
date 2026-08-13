# WB-LIVE — PREREG R2: the obj grid, frozen before the first `cl.exe`

Round 1 is [`WB_LIVE_PREREG.md`](WB_LIVE_PREREG.md), frozen before the first
grep of the flat export. This round freezes the **black-box grid** and its
predictions, after the disassembly reading and **before any `cl.exe` this lane
authored**. Scored in [`WB_LIVE_FINDINGS.md`](WB_LIVE_FINDINGS.md) §7.

**The grid is frozen by CONTENT, not by name** — `w-keygen` found a hold-out
frozen by TU name was not frozen at all (board #3046):

    docs/whitebox/grids/wb-live/live_grid.cpp
    sha256 fc1d42d95e324f4bde7c70423a3b443b92da3457818506575f49a84d58cf694b

Mode: real `cl.exe` 16.00.11886.00 under wibo, `/nologo /c /GR /O1 /Oi /EHsc`,
and again at `/nologo /O1 /GS- /c`.

---

## 1. The two models

**I0 — the INCUMBENT, and the control.** The port's shipped register model
(`docs/CFG_SHAPE.md` §6.2 item F; `docs/CODEGEN_W6_COMPARE.md` §6): formals in
`ARG_REGS` by declaration order, result in `r3`, **temps descend from `r11` in
emission order**, one register per temp, **no notion of a live range ending**.

**L0 — this lane's reading.** A candidate carries a bitset of *still-allowed
physical registers*, initialised to the whole class and narrowed by every
physical register defined while the candidate is live; the selector takes the
minimum-cost still-allowed register, ties to the head of
`r11,r10,…,r3,r31,…,r14`. Two candidates whose live ranges do **not** overlap
never narrow each other, so they get the **same** register. A call carries a
register-set operand naming every volatile, so a candidate live across a call
loses `r3`…`r12` outright and its first surviving candidate is `r31`.

**The axis that separates them is live-range OVERLAP.** wb-regalloc's grid held
it constant (G1–G4 are all *simultaneously* live) and therefore could not tell
I0 from L0 on any cell it ran — its six surviving cells are consistent with
both. This grid varies it.

## 2. Frozen predictions

`—` means the cell does not discriminate.

| cell | shape | L0 (this lane) | I0 (incumbent) |
|---|---|---|---|
| **V1** | 3 temps live in sequence | value-holding GPRs are **`r11` and `r10` only**; `r9` **absent** | `r11,r10,r9,r8,…` — `r9` **present** |
| **V3** | 8 temps live in sequence | **no `__savegprlr_*`, no callee-saved GPR saved**; ≤ 3 distinct GPRs; the same registers repeat 8 times | the 9 volatiles are exhausted, callee-saved registers are taken and the body is **framed** |
| **P2** | 3 values live at once (positive control) | **three distinct** GPRs, `r11,r10,r9` | `r11,r10,r9` — **identical**, by design |
| **X1** | 1 formal across a call | the formal moves to **`r31`**; `__savegprlr_31`-family save; framed | the formal stays in `r3` or moves to `r11` — either way **no callee-saved GPR** |
| **X2** | 2 formals across a call | **`r31`, `r30`** | no callee-saved |
| **X5** | 3 formals across a call | **`r31`, `r30`, `r29`** | no callee-saved |
| **X3** | non-leaf, **nothing** across the call | **no callee-saved GPR is saved** — LR only | — |
| **X4** | §6.2 F case 1: value live out of the entry block, both arms clobber | the value is copied out of `r3` into **`r31`** in the **entry block**, before the branch | the value stays in `r3` and is destroyed by the first call |
| **X6** | live across the call in one arm only | a callee-saved GPR **is** taken (the crossing range exists on one path) | no callee-saved |
| **R1** | 2 temps, one each side of a call, neither crossing | **no callee-saved GPR**; both temps take **`r11`** | `r11` then `r10` |

## 3. Non-rival predictions (scored, no rival column)

| # | prediction |
|---|---|
| **F1** | `r12` appears only as the LR shuttle in every cell (wb-regalloc F7, re-run on a new population) |
| **F2** | `r13` never appears in any cell (wb-regalloc F8) |
| **F3** | X1/X2/X5 take callee-saved registers **from the top** (`r31`, then `r30`, then `r29`) and never `r14`,`r15`,`r16` |
| **F4** | The number of callee-saved GPRs saved in X1/X2/X5 is exactly 1/2/3 — i.e. it tracks the number of ranges crossing the call, not the arity |
| **F5** | Both flag modes produce byte-identical code sections (wb-regalloc §7.7, on a new population) |

## 4. Separation assertion, before the run

| pair | discriminating cells | n |
|---|---|---|
| L0–I0 on **reuse** | V1, V3, R1 | 3 |
| L0–I0 on **call interference** | X1, X2, X5, X4, X6 | 5 |
| **total** | | **8** |

**P0.3's decline floor is met in advance only if I0 actually goes red.** If V1,
V3 and R1 all come back showing descending non-reused registers, **L0 is
refuted on its central claim** and this lane publishes that, not a rule. The
control can go red; that is the point of including V3, whose I0 prediction
(a frame and callee-saved saves on a leaf body that touches no argument) is
extreme and unmistakable.

**Declared insufficient in advance**: nothing in this grid separates "ties go
to the earliest list entry" from "only one candidate was legal" — that is
wb-regalloc's #1821 caveat and it is unchanged. Nothing here grades the
*spiller*: no cell is designed to exceed 27 simultaneously-live values, so a
green run says nothing about spilling.

## 5. What would falsify the mechanism rather than the outcome

The outcome predictions above can be right for a reason other than L0. Two
registered discriminators:

* **X3 vs X1** — same call, same frame requirement; they differ only in whether
  a value *crosses*. If X3 also saves a callee-saved GPR, then "framed non-leaf
  bodies save GPRs" explains X1 without any liveness, and X1 is worthless as
  evidence.
* **R1 vs V1** — same reuse question with and without an intervening call. If
  R1 shows reuse but V1 does not (or the reverse), the reuse is not a property
  of live ranges.
