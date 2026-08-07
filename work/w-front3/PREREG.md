# w-front3 — PREREGISTRATION

Committed **before the first lift**. Nothing in this file may be edited after
the first `C2RS_SINK_*` run or the first scratch-tree hatch; the rung doc scores
it as written.

Lane `w-front3`, worktree `wt-w-front3`, off master **`503f8937`**.

---

## 0. What is ALREADY measured at the time of writing, stated so it is not counted as a prediction

Three things were run on the **shipped tree with nothing lifted** before this
file was written. They are baseline reads, not probes of a lifted parser, and
none of them is a price:

1. the 878-TU scan at master (`work/w-front3/scan_base.out`) — `match 10 ·
   mismatch 0 · codegen-gap 0 · vocab-gap 861 · capture-fail 7`, `frontier 17`,
   139 `gap-metric` lines;
2. the FRONTIER's 17 members by name off that scan
   (`work/w-front3/tus.txt`) — the same 17 `w-front2` priced;
3. **the first-refusal key of every blocked function in all 17**
   (`work/w-front3/keys_base.txt`), by `c2rs census` at the workload's own
   flags and cwd.

(3) is the thing this lane is about not over-reading, so it is worth being
explicit: **knowing the head of a ladder is not knowing its length.** That is
the coordinator's own error restated, and every prediction below is a
prediction of *depth*, made with the heads already in hand.

---

## 1. The registered table — predicted price per frontier TU

`READER` = productions `crates/c2-il` must gain. `CODEGEN` = refusals in
`crates/c2-core`. Cheapest first, as I predict the ordering will come out.

| # | TU | READER | CODEGEN | total | basis of the prediction |
|--:|---|---:|---:|---:|---|
| 1 | `src/xdk/nuispeech/xboxheap.cpp` | 1 | 1 | **2** | `w-mrslot` §5 measured this by lifting, 2026-08-09. I am carrying a measurement, not predicting |
| 2 | `src/Main.cpp` | ≥ 2 | 6 | **≥ 8** | `w-front2` ≥ 7; its READER cell is `≥1²` and its own §2 says `D = 0` here means the instrument cannot see it |
| 3 | `src/system/negate_test.cpp` | ≥ 2 | 7 | **≥ 9** | `w-front2` ≥ 8, READER `≥1²` |
| 4 | `src/xdk/xlrc/xlrcimpl.cpp` | ≥ 2 | 6 | **≥ 8** | `w-front2` ≥ 7, READER `≥1²` |
| 5 | `src/xdk/LIBCMT/vsnprnc.cpp` | ≥ 2 | 6 | **≥ 8** | `w-front2` ≥ 7, READER `≥1²` |
| 6 | `src/system/utl/Pool.cpp` | 6 | 4 | **≥ 10** | `w-front2` ≥ 10 |
| 7 | `src/xdk/nuispeech/mmio.cpp` | 6 | 5 | **≥ 11** | `w-front2` ≥ 11 |
| 8 | `src/xdk/LIBCMT/undname.cpp` | 5 | 6 | **≥ 11** | `w-front2` ≥ 11 |
| 9 | `src/xdk/LIBCMT/vswprnc.cpp` | 5 | 6 | **≥ 11** | `w-front2` ≥ 11 |
| 10 | `src/xdk/LIBCMT/osfinfo.cpp` | ≥ 5 | 6 | **≥ 11** | `w-front2` ≥ 11 |
| 11 | `src/system/synth_xbox/Biquad.cpp` | 7 | 5 | **≥ 12** | `w-front2` ≥ 12 |
| 12 | `src/system/math/Primes.cpp` | 7 | 8 | **≥ 15** | `w-front2` ≥ 15, and its READER chain is the one `w-front2` calls `CLEAR` |
| 13 | `src/system/utl/EncryptXTEA.cpp` | 6 | ≥ 10 | **≥ 16** | `w-front2` ≥ 16 |
| 14 | `src/system/synth_xbox/IPP_basicmath_xbox.cpp` | 12 | ≥ 6 | **≥ 18** | `w-front2` ≥ 18 |
| 15 | `src/system/rndobj/wordwrap.cpp` | 11 | ≥ 12 | **≥ 23** | `w-front2` ≥ 23 |
| 16 | `src/xdk/xjson/jsonwriter.cpp` | 14 | ≥ 10 | **≥ 24** | `w-front2` ≥ 24 |
| 17 | `src/keygen_xbox.cpp` | ≥ 4 | ≥ 21 | **≥ 25** | `w-front2` ≥ 21; 18 blocked functions and ten distinct keys among them |

**Registered minimum over the seventeen: 2.** Registered second-cheapest: **≥ 8**.

---

## 2. The headline prediction, and the direction I expect to be wrong in

> **P-DIR — LIFTING DISPERSES THE DISTRIBUTION. It will make the cheap rows
> CHEAPER and the expensive rows DEARER, and the record is biased in BOTH
> directions at once rather than in one.**

The argument, so it is falsifiable rather than hedged:

* every price on record above `xboxheap` was obtained by *reading* a
  disassembly and counting constructs (`w-conv`) or by re-running the chain
  instrument (`w-front2`'s `²` cells). Both count things a body **contains**.
  Lifting counts things the parser **stops at**, and a body can contain five
  constructs that one production admits in one step. So the expensive rows
  should shed phantom rungs;
* against that, every one of the six re-derivations board **#770** records came
  back **dearer**, and `w-front2` supplied the sixth against its own registered
  prediction. Lifting also *finds* rungs a disassembly cannot show, because a
  refusal can sit on a construct that is not in the emitted bytes at all.

I register that the second effect will dominate on the rows with many blocked
functions (`keygen_xbox`, `wordwrap`, `jsonwriter`, `IPP_basicmath_xbox`) and
the first on the rows with one (`Main`, `xlrcimpl`, `vsnprnc`, `negate_test`).
**If both effects appear, P-DIR is a HIT; if every row moves the same way, it is
a MISS**, and the miss is more informative than the hit.

## 2.1 The specific losses I expect

| # | registered | why I expect to lose it |
|---|---|---|
| **P1** | the minimum over the seventeen stays **2** and `xboxheap` stays the cheapest row | `w-mrslot` measured it four commits ago on this same tree; if it moves, the ladder is less stable than anyone has assumed |
| **P2** | at least **8 of the 17** READER cells on `w-front2`'s table are REFUTED by lifting — moved, not merely re-quoted | twelve of its cells are `²` (a chain-depth lower bound) and it says so |
| **P3** | I find **exactly 1** further structurally-unreachable rung besides `value_bound` | **This is the one I most expect to lose, and I expect to lose it UPWARD.** A defensive backstop written before the reader could reach it is not a rare accident; `value_bound` is the archetype and archetypes come in families. If I find 0, the search was not wide enough and I must say so rather than report a clean negative |
| **P4** | all three of `w-front2` §3.1's paid cells (`x1`, `x2`, `x7`) re-verify **byte-exact** — the paid ledger stands | four merges have landed on `codegen/` since. A paid rung that is not paid is the most expensive error available here, so this must be re-run and not re-read |
| **P5** | `Pool.cpp`'s READER comes back **higher** than `w-front2`'s 6 | its base keys are `2 x expr-op-0x27` + `1 x expr-brtrue`, and `expr-op-0x27` is the fall-through position `w-front2` §4.3 proved is not a description of the body |
| **P6** | the chain instrument `EXIT`s on **≥ 6 of the 17** before it reaches a terminal, so ≥ 6 rows cannot be closed by the committed sink alone and need a scratch-tree hatch | six of the seventeen already report a *production* key rather than an expression-layer one at round 0 |

## 2.2 The loss I register as most likely of all

> **P-LOSS — the READER column is not a well-defined number for a multi-function
> TU, and this lane will find that out the expensive way.** Every price on
> record is written as one integer per TU. `keygen_xbox` has 18 blocked
> functions with ten distinct heads; `mmio` has 3 blocked of 11 with one head
> repeated. A "price" that is the union of per-function chains double-counts
> nothing but also *orders* nothing, and the union's size is not the number of
> rungs a lane would climb. If this fires I must publish the per-function
> structure rather than a single integer, and say that every prior single
> integer on this row is a category error.

---

## 3. Metrics — every one predicted UNCHANGED, and shown measured

This lane touches **no file under `crates/`**. Predicted at the tip, against the
base in §0:

| | predicted at tip |
|---|---|
| TU match · mismatch · codegen-gap · vocab-gap · capture-fail | 10 · 0 · 0 · 861 · 7 |
| the whole `gap-metric` block | **139 lines, `diff` EMPTY** |
| factors A/B/C/D/E · `B∧C` · `A∧B∧C` · FRONTIER | 28 · 338 · 169 · 10 · 2 · 151 · 27 · 17 |
| `cargo test --workspace` | unchanged, **36 targets**, 0 failed |
| the 17 FRONTIER members | the same 17, by name |

A `crates/` diff that is not empty at the end of this lane is a **hard fail**,
not a finding.

---

## 4. Method, registered so it cannot be relaxed later

* **LIFTED** is the only tag that counts as measured, and it means: the clause
  was disabled (committed `C2RS_SINK_*` hatch, or an uncommitted scratch-tree
  hatch documented in `work/w-front3/ladder.sh`), the scan re-run **on the real
  dc3 TU at the workload's own flags and cwd**, and the next key read off it.
* **BOUND** is a lower bound with what makes it one named in the same cell.
* **INFERRED** is not a price and is never quoted as one.
* Every lift that a replica cell exists for is run on the replica **and** the
  real TU and must agree cell for cell (`w-mrslot` §5's standard). Where no
  replica exists that is stated, and the entry is not upgraded to LIFTED on the
  strength of the real TU alone unless the lift is a *committed* instrument
  whose poison guarantees it cannot accept.
* **Discriminating-cell counts are printed for every negative.** "No
  disagreement" over zero comparable cells is a vacuous negative and is reported
  as a failure of the control, not as a pass.
