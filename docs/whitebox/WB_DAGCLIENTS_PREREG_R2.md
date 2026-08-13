# WB_DAGCLIENTS — PREREG R2: the obj grid, frozen by CONTENT HASH before the first `cl.exe`

> R1 is [`WB_DAGCLIENTS_PREREG.md`](WB_DAGCLIENTS_PREREG.md), committed at
> `cff4a8db` before the first grep of the export. **This file is written after
> the export read and before any `cl.exe` invocation by this lane.** Its
> sources are frozen by the sha256 of their bytes, not by their names —
> `w-keygen` proved a hold-out frozen by NAME is not frozen (its population
> moved −10.8% underneath it).

## 0. What the read established, and therefore what the grid must show

The export read (scored in the findings doc) says all three `0x10b3b*` clients
**splice the tuple list**: each calls `FUN_10bd38b0` @ `0x10bd38b0` (unlink,
then insert **before** an anchor) or `FUN_10bd3892` @ `0x10bd3892` (unlink,
insert **after**), both of which rewrite `tuple+0` (next) and `tuple+0x10`
(prev) — the same two links `WB_DAGORDER_FINDINGS.md` §2 names as the
scheduler's own re-link fields. So **M1 is answered YES by reading.**

A read is not this lane's deliverable. `#3071` exists because `wb-dagorder`'s
grid could not have detected this; a lane that answers by reading alone
reproduces that blind spot with the sign flipped. **The grid must exhibit the
motion in emitted code, positively, in a cell that would look different if the
motion did not happen.**

The three clients are, per the read:

| client | what it is | ready-list helper | splice | extra gate |
|---|---|---|---|---|
| K1 `0x10b3b167` | **tail merge / cross-jump** at a two-way branch | `0x10b3ada1` (nodes with **fanout 0** — sinkable) | `0x10bd38b0` (insert **before** the branch) | — |
| K2 `0x10b3b41b` | **head merge / hoist** | `0x10b3ad62` (nodes with **pred count 0** — hoistable) | `0x10bd3892` (insert **after**) | — |
| K3 `0x10b3b5fd` | tail merge to a **searched** second block | `0x10b3ada1` | `0x10bd38b0` | **`DAT_10c2e310 == 0`** |
| K4 `0x10c1ce93` | `/QXSTALLS` stall report over the **whole function** | — | none found | `/QXSTALLS` |

`DAT_10c2e310` is **already identified black-box** by `wb-memcpy` (board
`#1611`, `DISCLOSURE.md` W-MEMCPY-2) as **bit 23 of the option word = the
favor-speed flag**: `1` at `/O2`, `/Ox`, `/O1 /Ot`; `0` at `/O1`, `/O2 /Os`.
So **K3 runs only in favor-SIZE builds** — including the dc3 workload's own
`/O1 /Oi`. That gives a structural axis whose levels are already established
by a different lane's 180 cells, and it is the deciding quad of `#1611`
reused: `/O1` vs `/O1 /Ot` and `/O2 /Os` vs `/O2` separate favor-speed from
`/O<n>`.

## 1. Sources, frozen by sha256

The grid is one file, committed in the **same commit as this one** and frozen
by content hash — `wb-dagorder`'s convention (`grids/wb-dagorder/`), not a name:

    docs/whitebox/grids/wb-dagclients/dagclients_grid.cpp
    sha256 4847a3a6ff68809e799f378ac6e9041d169852baa023967b7df69416708ff25b

It claims no `fixtures/` prefix. **Any later edit is a NEW cell with a new
hash and must be reported as such**, never as a correction to these numbers.

Nine family-T/H functions, plus five K4 shapes:

**Family T — tail merge (K1/K3 shape): a two-way `if`/`else`, common work at
the END of the arms.**

| id | shape | why |
|---|---|---|
| `dt_sfx` | arm A `ga=1; gc=9;` · arm B `gb=2; gc=9;` | common statement already the **suffix** of both arms — a naive textual suffix matcher merges this |
| `dt_sink` | arm A `ga=1; gc=9;` · arm B **`gc=9; gb=2;`** | common statement is **not** the suffix of arm B, and `gb` is a distinct global so the sink past it is dependence-legal. **A naive suffix matcher merges NOTHING here. A DAG-gated sinker moves `gc=9` down and then merges.** |
| `dt_dep` | arm A `ga=1; gc=9;` · arm B **`gc=9; *p=2;`** (`p` an unknown `int*` parameter) | same shape, but the tail store may alias `gc`, so the sink is dependence-**illegal**. Discriminates "DAG-gated motion" from "motion regardless" |
| `dt_none` | arm A `ga=1;` · arm B `gb=2;` | negative control — nothing may merge |
| `dt_mid` | as `dt_sink`, with **10** intervening stores | attribution: still inside K1's backward window |
| `dt_far` | as `dt_sink`, with **30** intervening stores | attribution: **outside** it. `FUN_10b397ba` @ `0x10b397ba` bounds K1's backward scan at `0x1d` = **29 tuples**; a merge in `dt_sink`/`dt_mid` but not in `dt_far` exhibits that constant black-box, which a generic store-sinker has no reason to respect |

**Family H — head merge (K2 shape): common work at the START of the arms.**

| id | shape |
|---|---|
| `dh_pfx` | arm A `gc=9; ga=1;` · arm B `gc=9; gb=2;` — common **prefix** |
| `dh_hoist` | arm A `gc=9; ga=1;` · arm B **`gb=2; gc=9;`** — hoist past an independent store |
| `dh_dep` | arm A `gc=9; ga=1;` · arm B **`*p=2; gc=9;`** — hoist past a may-alias store |

Every function takes the condition as a parameter so the branch survives, and
every arm's distinguishing store is to a distinct global so the arms cannot be
folded away. Exact text is in the frozen files; the table is the claim.

## 2. Axes crossed (structural first)

| axis | levels |
|---|---|
| **S-SHAPE** | the 9 family-T/H functions above (family × suffix / needs-motion / motion-blocked / none / window-distance) |
| **S-OPT** | `/Od` · `/O1` · `/O1 /Ot` · `/O2 /Os` · `/O2` — the last four are `#1611`'s deciding quad, crossing favor-speed against `/O<n>` |
| **S-QXSTALLS** | off · on (K4 only) |
| **S-REGION** (K4) | body with no interior barrier · with a branch · with a call · with a label/loop |
| **S-SIZE** (K4) | ≲ `0x50` tuples · > `0x50` tuples |

9 × 5 = **45** family-T/H cells, plus **10** K4 cells (5 shapes × on/off) at the
workload mode, plus the replay cells in §4.

## 3. Registered predictions — and WHICH CELL GOES RED

**The red-capable cell is `dt_sink` at `/O1`.** The positive question, in the
form R1 §1 demands:

> *If none of K1–K3 reordered tuples, what would `dt_sink@/O1` look like?*
> **It would look exactly like `dt_none@/O1`: two copies of `gc=9`, one per
> arm, because `gc=9` is not a common suffix of the two arms as written.** A
> merger that cannot move tuples has nothing to merge here.
> *If one of them does reorder?* **One copy of `gc=9`, reached from both arms.**

That is a single scalar read off the `/FAsc` listing (occurrences of the store
to `gc` inside the function) and it cannot come out "inconclusive".

| id | prediction | p |
|---|---|---|
| **P1** | At `/Od`, **every** family-T/H cell emits **two** copies of the common store (no merging at all) | 0.90 |
| **P2** | `dt_sfx` at `/O1` emits **one** copy | 0.80 |
| **P3** | **`dt_sink` at `/O1` emits ONE copy** — the motion is exhibited in emitted code | **0.70** |
| **P4** | `dt_dep` at `/O1` emits **two** copies — the may-alias tail blocks the sink, so the motion is dependence-gated | 0.60 |
| **P5** | `dt_none` emits one `ga` store and one `gb` store at every level, and nothing merges | 0.95 |
| **P6** | `dh_hoist` at `/O1` emits **one** copy (head merge moves too) | 0.50 |
| **P7** | At least one cell differs between `/O1` and `/O1 /Ot`, **and the same cell differs the same way between `/O2 /Os` and `/O2`** — i.e. the split follows **favor-speed**, not `/O<n>`, which is K3's `DAT_10c2e310` gate | 0.45 |
| **P8** | `/QXSTALLS` leaves the obj byte-identical (TimeDateStamp zeroed) in **all 4** region shapes and at both sizes, with the stall summary positively present in every ON cell | 0.85 |
| **P9** | The `/QXSTALLS` ON cell whose body exceeds `0x50` tuples is also identical — i.e. K4's whole-function DAG, which bypasses the region cap, still mutates nothing | 0.80 |
| **P10** | Every `/O1`-class cell's listing shows the expected PROC count and a conditional branch (probe validation passes before any count is read) | 0.90 |
| **P11** | `dt_mid` at `/O1` merges (one copy) and **`dt_far` does not** (two copies) — K1's `0x1d`-tuple backward window exhibited black-box | 0.35 |

**Falsifier for the lane's headline:** if P3 comes out **two copies** at every
optimization level and every family-T shape, then the read's "they splice the
list" is **not demonstrated in emitted code** by this grid, and the finding is
downgraded to a read-only claim with the grid recorded as a **negative** — the
honest outcome, and it must be stated as such rather than argued around.

## 4. The same-IL cell (strongest control, contingent)

`Toolchain::replay` reruns **standalone c2 over a captured IL bundle with the
captured c2 argv verbatim**, swapping only `-il` and `-Fo`. If the captured
argv contains a token that carries the favor-speed bit, then running the *same
IL bytes* through c2 twice, toggling only that token, isolates a c2-internal
difference with the front end held **exactly** constant — which `/O1` vs
`/O1 /Ot` at `cl` level does not (cl passes the flag to `c1xx` too).

* **R-1**: same IL, favor-speed token toggled ⇒ obj differs on `dt_sink`
  (p = 0.40; contingent on such a token existing in the argv).
* **R-2**: same IL, argv verbatim, run twice ⇒ obj byte-identical (the
  determinism floor; if this fails nothing else in §4 is readable). p = 0.97.

If no such token exists in the argv, §4 is filed **grey-zone** with no board
row, per R1 §4's rule, and §3 stands alone.

## 5. Probe validation, before any result is read

For each cell, in this order, and recorded:

1. the listing exists and contains the expected `PROC` for the function;
2. the function's listing contains a conditional branch mnemonic (`b<cc>`) —
   a cell whose `if` was folded away tests nothing;
3. only then is the count of common-store occurrences read.

A cell failing (1) or (2) is reported as **invalid**, not as a zero.

## 6. Scope, unchanged

Docs-only. `git diff master..HEAD -- crates fixtures scripts` empty at the tip.
Grid sources and outputs live in `work/w-dagclients/`.
