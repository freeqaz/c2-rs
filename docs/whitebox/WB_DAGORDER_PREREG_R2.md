# WB-DAGORDER — PREREG R2: the obj grid, frozen before the first `cl.exe`

Round 1 is [`WB_DAGORDER_PREREG.md`](WB_DAGORDER_PREREG.md), frozen at
`78971a5d` **before the first grep of the flat export**. This round freezes the
grid and its predictions **after the disassembly reading and before any
`cl.exe` this lane authored**, on the `wb-live` pattern.

**The grid is frozen by CONTENT, not by name** (board #3046):

    docs/whitebox/grids/wb-dagorder/dagorder_grid.cpp
    sha256 318bf2d258f7e690e5befed7d12ec25f82b80be1fb503ccbfa458ee02864561c

Mode: real `cl.exe` 16.00.11886.00 under wibo,
`/nologo /c /GR /O1 /Oi /EHsc` (the workload flags), order read from the
`/FAsc` listing **and** the obj via `scripts/gt_dump.py`.

---

## 1. What the disassembly reading found (summary; findings doc will carry it)

The commissioned address `0x10b3219f` is **not a tree-to-tuple walk**. It is a
48-byte `dag.c` helper that attaches every predecessor-less DAG node to the
last region-anchor node. The within-block order is produced by a **dependence
DAG over the already-lowered tuple list plus a cycle-driven list scheduler**:

* dependence DAG build `FUN_10b328da` (dag.c band `0x10b31fd4`–`0x10b33330`);
* regions found by `0x10be5d4b`: runs of ≤ `0x50` tuples ending at
  category-byte values {`0x12`, `0x14`, `0x19`, `0x1b`, label `0x30f`} —
  branches / calls / labels end regions;
* priority `0x10be5df6`: `(height << 13) + (fanout << 8) + (sym-dest << 10)`
  (weights are the shorts at `0x10c3bf9c` = `[-1, 13, 8, -1, -2, 10, 0]`;
  the two `-1`/`-2` slots disable their terms), mid-level store `0x2b8` =
  `0xffffffff`; `height` = 1 + max(succ height + edge latency);
* ready list sorted by (priority desc **unsigned**, original tuple index at
  `node+0x44` asc — `0x10be5cea`); issue loop `0x10be60c0`, width
  `DAT_10c3cf98` = 2 (4 in one mode), earliest-start times from per-edge
  latencies (`0x10c1c1d4`; 11×11 matrix `0x10c3c1a8`): ALU→ALU 2,
  ALU→load 5, ALU→store 2, load→ALU 2, load→mem 5, cmp→branch 2 (opcodes
  45–48), ALU→branch 0;
* driver `0x10be6382`, invoked mode-1 three times around globregs and the
  register allocator from `0x10b7dc51` (and mode-0 once from `0x10b7df57`),
  gated by the optimizer-on flag `DAT_10c2e2fc`.

Two round-1 predictions are therefore **already revised by the reading, before
any obj**: D1's favoured first-use order (the mechanism says fanout rank, then
original index) and D5's per-statement contiguity (the mechanism says phase
grouping). Round-1 rows will be scored as frozen; this round registers the
mechanism-derived predictions as the M-series. If the obj contradicts the
M-series, the *mechanism reading* is what goes red.

## 2. Frozen predictions

Notation: `lisS(k)` = the `@ha` materialization of statement k's source
symbol, `lisD(k)` = of its destination symbol; `L(k)`/`A(k)`/`S(k)` = load /
add / store of statement k.

| # | p | cell | prediction |
|---|---:|---|---|
| M1 | 0.75 | `dg_v1` (control) | all six `lis` precede the first `lwz` (replication; if red, STOP) |
| M2 | 0.55 | `dg_v1` | full order is **phase-grouped**: `lisS(1..3)`, `lisD(1..3)`, `L(1..3)`, `A(1..3)`, `S(1..3)` — source-`lis` before dest-`lis` (height 13+ vs ~4), each phase in statement order (tie on `+0x44`) |
| M3 | 0.70 | `dg_two`, `dg_v4` | same phase grouping at n=2 and n=4; no per-statement contiguity anywhere (round-1 D5 scored MISS if so) |
| M4 | 0.60 | `dg_one` | both `lis` first (either order admitted: both are ready at cycle 0 and width is 2; the mechanism puts `lisS` first on priority), then `lwz`, `addi`, `stw` |
| M5 | 0.55 | `dg_shared` | **one** `lis` of `dg_b@ha` (dedup happens before the scheduler; the scheduler itself never merges) |
| M6 | 0.50 | `dg_disc` (C-DISC) | `dg_d`'s materialization precedes `dg_b`'s **iff** the shared `@ha`/value is deduplicated (fanout 2 beats original index). If NOT deduplicated, original index wins and `dg_b`'s comes first. Registered as a conditional: dedup ⇒ rank order. Round-1 D1 (unconditional first-use) is scored against this cell as frozen |
| M7 | 0.85 | `dg_call` (C-CALL) | no address-formation for statement 2 appears above the `bl`; the call ends the region |
| M8 | 0.75 | `dg_call2` | `dg_b@ha` is **re-materialized** after the call (a second `lis`-class word), not carried in a callee-saved register |
| M9 | 0.70 | `dg_sub` vs `dg_sub2` | the two loads emit in **source order** in both cells (equal height, equal fanout, tie on original index); the subtract's operand roles flip, the load order follows the source |
| M10 | 0.60 | `dg_chain` | `dg_d`'s load sits **below** `dg_b`/`dg_c`'s loads (smaller height: it feeds only the second add); the adds emit in association order |
| M11 | 0.50 | `dg_lit` | the shared literal `1` has **one** producer with fanout 2 which precedes the fanout-1 producers of its height class; stores follow the producers (no `li`,`stw`,`li`,`stw` interleave) |
| M12 | 0.55 | `dg_if` | **exactly one instruction sits between the compare and its branch** (cmp→branch latency 2 with the independent statement's tuple available to fill) — B-RULE-2 reproduced from mechanism |
| M13 | 0.60 | `dg_cap` | the hoisted address-formation cluster **breaks** before the 20th statement: at least one `lis`-class word of a later statement appears *after* some earlier statement's `lwz`/`stw` — the ≤ 0x50-tuple region cap is visible. The naive block-top model predicts one unbroken 40-`lis` cluster; if that is what emits, M13 is red and the cap reading is wrong for this class |
| M14 | 0.80 | all cells | no instruction moves across a `bl` in any cell (regions never span calls) |
| M15 | 0.70 | all no-call cells | every `lis`-class word of a region precedes every `lwz` of that region (height ordering: 13+ vs ≤ 8) — the wbl_v1 hoist restated as a region property, graded per cell |

## 3. Structure verification before reading (required)

* The `/FAsc` listing must contain exactly **15 PROC** blocks (one per cell).
* Every cell's listing contains ≥ 1 `stw` and references its extern symbols.
* `dg_if` must contain a conditional branch; `dg_call*` a `bl dg_ext`.
* `dg_cap` must be a single basic block (no branches) with 20 stores.
* If any cell is folded, inlined away, or restructured, that cell is a broken
  probe and is reported as such, not read.

## 4. Controls, and whether they can go red

| control | red condition | what red would mean |
|---|---|---|
| M1 / `dg_v1` | six `lis` not at top | instrument/mode broken — stop |
| M6 / `dg_disc` | `dg_b` first despite dedup | fanout term wrong (rank model refuted) |
| M7 / `dg_call` | statement-2 forms above `bl` | region model refuted |
| M13 / `dg_cap` | one unbroken cluster | region cap wrong (or cap counted differently) |
| M2/M3 phase grouping | contiguous per-statement bodies | the cycle/latency model is wrong even though the reading is right about the structures |

## 5. Declared insufficient in advance

* Nothing here separates the **unit-conflict** sub-model (whether two loads
  can issue in one cycle) from latency effects — cells were not designed for
  it and per-cycle boundaries are invisible in a linear listing.
* Nothing here grades **mode-0 (pre-lowering) scheduling** separately; the
  final order is the composition and only the composition is observable.
* Nothing here grades the **width-4** mode (`DAT_10c2e2d2`).
* Register identities are the allocator's (wb-live) and are not re-predicted.
