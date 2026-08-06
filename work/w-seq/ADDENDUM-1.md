# w-seq — ADDENDUM-1: GRID-S is frozen

Committed **before the first `cl.exe`** on any GRID-S cell, per `PREREG.md` P7.

    GRID-S generator : work/w-seq/gen_cells.py
    cells            : 14
    sha256           : aed86997f143f9c0074cec90d6460cccb92099c4708626c1112e7984dac10433

The stamp is over `(cell name, cell text)` for every cell in declaration order,
so editing a cell, adding one, or reordering them all move it.

## What GRID-S is for, and what it is not

The workload measurement (`work/w-seq/grid.txt`) is graded by real c2 on 3,195
functions and says **SPLICE-0 holds on 1,967 of them**. It cannot say whether
the **port** could produce those bytes, because on the workload the callee is
usually a function the port cannot lower at all — 1,774 of the 3,195 name a
parse-refused callee. GRID-S holds the callee at a shape the port **does** lower
and varies the call site, so the splice's right-hand side is a body the port
already emits byte-exactly.

## Registered before compiling — GRID-S's own predictions

Each is `PREREG.md` P1/P2 restated at the cell level, so the grid can lose:

| cell | prediction |
|---|---|
| `s01`, `s07`, `s13`, `s14` | **SPLICE-0 exact** — the port's setup is empty, so c2's caller body is c2's callee body |
| `s03`, `s11` | **SPLICE-0 fails at word 0 in a REGISTER FIELD** — c2 renames the inlined body's source operand instead of emitting the move |
| `s06` | **SPLICE-0 fails in a DISPLACEMENT FIELD** — the caller's `addi r3,r3,4` folds into the callee's load |
| `s04`, `s05` | **SPLICE-0 fails by CONSTANT FOLDING** — two literals become one |
| `s08` | **SPLICE-0 fails and c2 emits no frame** — the `?back@?$vector@…` shape |
| `s09`, `s10` | `seq`: **`s10` SPLICE-0 exact, `s09` not** — the 816/725 split the workload shows |
| `s12` | **CONTROL** — the callee calls an external, so c2 keeps a call and neither splice applies |
| `s02` | ungraded if the port refuses the caller; a refusal is a printed outcome, not a skip |

A cell whose caller the port refuses, or whose anchor loses its REL24, is
**refused rather than scored**.
