# w-one — PREREG

Committed **before the first probe**. Lane `w-one`, worktree `wt-w-one` off
master **`56912b72`**.

The rung: measure a per-TU ladder for the **seven FRONTIER TUs that read
`1 | 1`** — one blocked emitted function of one emitted function:

    src/Main.cpp
    src/system/math/Primes.cpp
    src/xdk/LIBCMT/osfinfo.cpp
    src/xdk/LIBCMT/undname.cpp
    src/xdk/LIBCMT/vswprnc.cpp
    src/xdk/xjson/jsonwriter.cpp
    src/xdk/xlrc/xlrcimpl.cpp

Method: `work/w-front3/{hatch.py,ladder.py}`, repaired first (board **#1322**),
run on the real dc3 TU at the workload's own flags and cwd.

---

## 0. What is READ before the first probe, and is therefore not a prediction

Two readings off committed artifacts, stated here so they cannot later be
re-told as findings of this lane:

* **R1.** `work/w-front3/lad/ladder.json` round 0 shows all seven have
  `fn_blockers` = exactly **one key, count 1**. So each has exactly one blocked
  function **in the whole `.ex` split**, not merely one blocked *emitted*
  function. `w-front3`'s P-LOSS (*the READER integer is a UNION over a TU's
  blocked functions*) is therefore **vacuous on these seven**: the union is over
  one function.
* **R2.** `ladder.py` subtracts a 7-token `SCAFFOLD`
  (`op:41 29 3A 4B 4F 53 54`) from every published count. On the seven, `raw −
  net` is 0/6/6/2/3/6/0. `w-front3`'s document never mentions the subtraction.

## 1. Predicted READER ladder depth, per TU, at `56912b72`

`w-front3` measured at `503f8937`. Master has since **pinned opcode `0xBD`**
(board #1314, lane `w-bd`) and **paid `store-run-bind-mixed-kind`**
(`w-mrslot`/`w-midrun`). #1317 says the nine rows that exited at `noform-0xBD`
now walk to `noform-0x4C`. So the three `-0xBD` rows should climb further.

| TU | `w-front3` (`503f8937`) | **predicted here** | predicted exit |
|---|---:|---:|---|
| `xlrcimpl.cpp` | 0 climbable | **0 climbable** | `assign-rhs-call-0x26`, no branch to lift |
| `Main.cpp` | 2 | **2** | `expr-convert-no-value-0x2C`, no lift |
| `undname.cpp` | ≥ 5 | **≥ 7** | `noform-0x4C` |
| `vswprnc.cpp` | ≥ 5 | **≥ 7** | `noform-0x4C` |
| `Primes.cpp` | 7 CLEAR | **7 CLEAR** | terminal `0x4F` |
| `osfinfo.cpp` | ≥ 12 | **≥ 14** | `noform-0x4C` |
| `jsonwriter.cpp` | 16 CLEAR | **16 CLEAR** | terminal `0x4F` |

**P1 — the cheapest of the seven is `Main.cpp` at 2 climbed rungs, and it is
STUCK rather than clear.** The cheapest row that reaches a terminal is
`Primes.cpp` at 7.

**P2 — NONE of the seven converts.** Predicted outcome of this lane is a
measured table and a decline. A conversion needs the READER column *and* the
CODEGEN column, and every CODEGEN cell for these seven is INFERRED from
`w-conv`'s hand-count (6 / 6 / 6 / 8 / 6 / ≥10 / 6).

**P3 — the direction I expect to be wrong in is OPTIMISTIC, i.e. every number
above is a lower bound and the measured value comes back the same or DEARER.**
Board #770 is nine-for-nine on this and `w-front3`'s own P-DIR was scored a MISS
in exactly this direction. If any row comes back **cheaper** than the table
above, that is the surprise and it must be explained by a *paid* rung and not by
a re-count.

**P4 — the SCAFFOLD subtraction is not free.** Predicted: a leave-one-out over
the raw token set shows the scaffold tokens that a row actually added are
**load-bearing** (removing one shortens that row's climb), so the published
`net` integers under-report by `raw − net`. Predicted discriminating cells on
the leave-one-out grid: **≥ 90 %** of the raw tokens on the rows that reach a
terminal.

**P5 — the null-lift grid over the seven has very few discriminating cells.**
Only `Main.cpp` has a *hatch* as its round-0 key (`param-width`); the other six
open on sink tokens. Predicted: **1** discriminating cell over the 7 × (hatches)
grid. If that is what it reads, the hatch grid is a **vacuous control on this
population** and must be said to be one rather than counted as a pass — which is
why P4's leave-one-out is registered as the control that can actually go red.

**P6 — the hatch fix.** `hatch.py apply` today writes 7 of 8 edits and then
raises, leaving `crates/` dirty. Predicted after the fix: it writes **nothing**,
names edit `store-run-bind-mixed-kind` / `leaf_store.rs`, and exits non-zero.

**P7 — unreachable rungs.** Predicted: **0** new structurally-unreachable rungs
on these seven's ladders. `w-front3`'s variant screen already read 0 of 21 and
these seven are expression-layer rows, where the sink reaches every token by
construction. Registered as a prediction I expect to hold rather than as a
search I expect to pay.

## 2. Registered losses — things I expect to go wrong

* **L1.** The `expr-chain-*` sink is **poisoned**, so a `READER-CLEAR` row is not
  a row whose reader accepts anything. "CLEAR" means *the chain walked the whole
  body without meeting a token this tree cannot spell*. It is a statement about
  the instrument's vocabulary, not about `IlBundle::functions()`.
* **L2.** A hatched tree can emit. No differential verdict from a hatched run is
  quoted in either direction, per `w-front3`'s own rule.
* **L3.** `net` counts *distinct tokens*, and one token can stand for an
  arbitrary amount of emitter work (`op:26` is a call in an expression). A rung
  is a unit of the instrument, not a unit of labour.

## 3. Grading

Pass for this lane = seven ladders with a provenance tag on every cell, the
hatch failing closed, and the leave-one-out control's discriminating-cell count
**printed whether it is large or zero**. A conversion is not required and is not
expected.
