# GRID A — result. **A1 holds on all five cells; A2 is REFUTED on three.**

Cells compiled at the workload's own `flags.txt` (`c2rs compile --flags-file
work/dc3-workload/flags.txt`), read by `scripts/gt_dump.py`. Dumps committed as
`work/w-extdata/grida/a*.dis`, sources as `a*.cpp`.

## What each cell's `.text` actually looks like

The `.text` layout is read off the obj rather than assumed, because **two of this
grid's frozen per-cell predictions got the layout wrong** — see §"Scoring the
prediction, honestly" below.

| cell | `.text` reference order (offset) | reverse | symbol table, from index 15 |
|---|---|---|---|
| a1 | `gI`(0x0c) `g1`(0x14) `g0`(0x18) | `g0 g1 gI` | `g0 g1 gI` ✔ |
| **a2** | `g0`(0x0c) `gI`(0x10) `g1`(0x18) | `g1 gI g0` | **`g1 gI g0`** ✔ |
| **a3** | `gI`(0x0c) `g1`(0x14) `g0`(0x18) `gJ`(0x1c) `g2`(0x24) | `g2 gJ g0 g1 gI` | **`g2 gJ g0 g1 gI`** ✔ |
| **a4** | `gI`(0x0c) `g1`(0x14) `gJ`(0x18) `g2`(0x20) | `g2 gJ g1 gI` | **`g2 gJ g1 gI`** ✔ |
| a5 | `g0`(0x0c) `g3`(0x10) | `g3 g0` | `g3 g0` ✔ (control) |

## The verdict

| rival | cells predicted correctly | status |
|---|---|---|
| **A1** — one list, reverse `.text` first-reference order, kind ignored | **5 of 5** | **CONFIRMED** |
| A2 — callees (reverse), then data names (reverse) | 2 of 5 (a1, a5) | **REFUTED on a2, a3, a4** |
| A3 — declaration order | 0 of 5 | REFUTED |
| A4 — `.gl` record order | 0 of 5 | REFUTED (a2's `.gl` order is `gI g0 g1`; the table is `g1 gI g0`) |

**a2 is the smallest refutation of the shipped writer**: one call, one
data-symbol argument, four lines of C++. `coff::writer` would emit
`g1 g0 gI` — the callee loop first — and c2 emits `g1 gI g0`. The data symbol is
*between* the two callees because its `lis` is, and nothing else decides it.

**a3 is `undname.cpp`'s shape in four lines**: `data · callee · data`,
interleaved, which no ordering of two separate loops can produce.

**a4 removes the callee that could have carried the difference**: two data names
and two callees strictly alternating, and A2's inner ordering (`g2 g1 gJ gI`)
is still wrong.

## Read the second way, as the grid required

Each table was also read by **the relocation that targets each index**
(`REL24` = callee, `REFHI`/`REFLO` = data name). On a3 the relocation sequence
down `.text` is `REFHI/REFLO(19) · REL24(18) · REL24(17) · REFHI/REFLO(16) ·
REL24(15)` — strictly descending symbol index with ascending offset, for both
kinds alike. **The two readings agree**, so the rule is not a `Type`-ordering or
a name-ordering that happens to coincide: the key is the `.text` offset of the
first reference and nothing else.

## Scoring the prediction, honestly

**The RULE A1 predicts every cell. Two of the five frozen PER-CELL predictions
written from it were wrong, and both were wrong about the SCHEDULE, not about
the ordering.**

* **a2**: predicted `gI, g1, g0`, observed `g1, gI, g0`. The prediction assumed
  c2 hoists the `lis` above the *first* call; it does not — the `lis` stays with
  the call whose argument it is, at 0x10, after `bl g0` at 0x0c.
* **a3**: predicted `gJ, g2, …`, observed `g2, gJ, …`. Same error, at the second
  call: `lis gJ`(0x1c) precedes `bl g2`(0x24), so `gJ` is referenced first and
  reverse order puts `g2` ahead of it.

This is worth recording rather than quietly correcting. A grid separates rivals
only if the predictions are derived *from the rivals*; two of these were derived
from a rival **plus an unstated assumption about code layout**, and the
assumption was false. The separation survives — A1 read off the observed layout
still predicts all five and A2 still fails three, and A2's failures do not depend
on the layout at all (it puts every data name after every callee whatever the
schedule) — but a grid where the confound had touched the *separating* clause
would have been worthless. **The generalization: a per-cell prediction that
requires a fact the grid is not measuring must say so, or it is not a prediction
about the rival.**

## What this buys, and what is NOT shipped

**Decline clause D3 does NOT fire**: A1 is confirmed on a2 and a3 both. Size of
the miss: **0 cells of 5 mispredicted by the rule**, 2 of 5 mispredicted by this
lane's transcription of it.

**The writer is unchanged and `check_external_order` still refuses the
disagreeing shape.** Shipping A1 into `coff::writer` converts nothing on its own
— `undname.cpp` needs four more things (PREREG §1.5 rows 6, 8, 9 and the
recognizer/emitter) — and it would change the symbol table on **every** obj the
port emits while the gate exercised the new arm on a population of **zero**
cells. That is `docs/STATUS.md` trap 0 in its exact shape: a green control is a
statement about the population it ran over. The rule ships with its consumer, in
the same commit, or not at all.
