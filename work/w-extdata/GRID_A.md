# GRID A — the symbol table's undefined-external ORDER

**Frozen before the first `cl.exe` on these cells.** Five one-function TUs,
compiled at the workload's own `flags.txt` through `work/w-frame/refobj.sh`, read
by `scripts/gt_dump.py`.

## The question

`coff::writer` emits a function's undefined externals as **callees in reverse
first-reference order, and THEN data symbols** — two loops, two index lists
(`introduced` / `introduced_data`, `writer.rs:462`–`479`). Three workload objs
say the reference is **one list in reverse first-reference order over all of
them, kind ignored** (PREREG §1.4). The two rules agree on every obj this port
has ever emitted, because WR1 required the `lis` to be the body's **first word**
and a first-word reference is always the first-referenced name.

**W-EXTDATA relaxed that position rule** (`vswprnc`'s `lis` is word 14), so the
agreement is no longer structural, and `crate::check_external_order` refuses the
disagreeing shape rather than emitting a symbol table that is right by accident.
This grid is what would let a lane *ship* the rule instead of refusing it — and
what R2 (`undname.cpp`, whose externals are `data · callee · data`) needs first.

## The rivals

| | rule |
|---|---|
| **A1** | ONE list over callees ∪ data names, **reverse `.text` first-reference order** |
| **A2** | callees (reverse first-reference), **then** data names (reverse first-reference) — the writer's current shape with its inner order corrected |
| **A3** | callees then data names, each in **source declaration** order |
| **A4** | `.gl` record order |

## The cells and the frozen predictions

Predictions are the **order of the undefined externals** in the symbol table,
counting from the first one after the function's own `$M` end label (a leaf here,
so: after the function symbol). `gI`/`gJ` are data names, `g0`/`g1`/`g2`/`g3`
are callees.

| cell | body | A1 predicts | A2 predicts | A3 predicts | A4 predicts |
|---|---|---|---|---|---|
| **a1** | `g1(&gI); g0();` | `g0, g1, gI` | `g0, g1, gI` | `g1, g0, gI` | *(gl order)* |
| **a2** | `g0(); g1(&gI);` | **`gI, g1, g0`** | **`g1, g0, gI`** | `g0, g1, gI` | *(gl order)* |
| **a3** | `g1(&gI); g0(); g2(&gJ);` | **`gJ, g2, g0, g1, gI`** | **`g2, g0, g1, gJ, gI`** | `g1, g0, g2, gI, gJ` | *(gl order)* |
| **a4** | `g1(&gI); g2(&gJ);` | `g2, gJ, g1, gI` | **`g2, g1, gJ, gI`** | `g1, g2, gI, gJ` | *(gl order)* |
| **a5** | `g0(); g3();` | `g3, g0` | `g3, g0` | `g0, g3` | *(gl order)* |

**Separation, asserted before compiling:**

* **a2** separates **A1** from **A2** (`gI, g1, g0` vs `g1, g0, gI`) and both
  from **A3**.
* **a3** separates all three, and is the shape `undname.cpp` actually has —
  a data name, a callee, a data name, interleaved.
* **a4** separates **A1** from **A2** on the *interleaving* alone, with no
  callee between the two data names to carry the difference.
* **a5** is the **control**: no data name at all, so every rival that gets the
  callee half right must predict `g3, g0`. A cell that fails here invalidates the
  grid rather than any rival, because that ordering is already shipped and
  graded.
* **A4** is not given a per-cell prediction because `.gl` order is not knowable
  without reading the capture; it is falsified for any cell where A1/A2/A3 and
  `.gl` order disagree, and that is checked after the fact from the capture.

## Read two ways

w-data's GRID C caught a writer interleaving defect **only because the cell was
read by ORDER as well as by content**. Each cell here is read:

1. by **symbol INDEX** — the order the records appear in the table;
2. by **the relocation that targets each index** — `REL24` for a callee,
   `REFHI`/`REFLO` for a data name — so a rule that got the order right for the
   wrong reason (e.g. by name, or by `Type`) is separable from one that got it
   right by first reference.

A reading that agrees on (1) and disagrees on (2) falsifies the rule as stated
even if the byte order happens to match.

## Decline clause D3, restated with its size

If A1 is not confirmed on **a2 and a3 both**, R2 is declined and the measurement
ships instead. **Size of the decline:** the number of cells whose symbol table A1
mispredicts, out of five, plus the name of the rival that survives.
