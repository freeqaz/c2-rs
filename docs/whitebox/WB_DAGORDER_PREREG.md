# WB-DAGORDER — PREREG: the `dag.c` tree-to-tuple lowering order

> **PROVENANCE — DISASSEMBLY-DERIVED (prospectively).** This lane will read the
> flat export of the exact image pinned in [`C2_MAP_METHOD.md`](C2_MAP_METHOD.md)
> §0 — `sha256 c80981c015166effecc71ad8112d5577a065b2300891dfdb02b9c13787a66258`,
> **verified against `compilers/X360/16.00.11886.00/c2.dll` at the top of this
> lane before this file was written**. Navigation only; this lane adopts nothing
> into `crates/` and adds no `DISCLOSURE.md` row.

**FROZEN BEFORE THE FIRST PROBE OF THIS LANE** — before any grep of
`~/ghidra-projects/export/c2/` and before any `cl.exe` invocation this lane
authors. The grid itself is frozen by content hash in a second round
(`WB_DAGORDER_PREREG_R2.md`) after the disassembly reading and before the first
`cl.exe`, on the `wb-live` pattern (its R2 cites board #3046: a hold-out frozen
by name is not frozen).

**The commissioned question** (board #3053/#3057; `CFG_SHAPE.md` §6.2 item F's
dated 2026-08-13 block): *what is the order in which `dag.c`'s tree-to-tuple
walk at `0x10b3219f` lowers an expression tree / statement list into the
instruction tuples of a block?* This is the sole remaining characterization
blocker for item F: `wb-live` proved live ranges are a property of the lowered
instruction order, and its V1/V3 cells are the committed receipt that the naive
per-statement model is wrong — c2 hoists every `lis <sym>@ha` to the top of the
block.

Five sub-questions, each of which must end with an address or a grid cell:

1. **Q-HOIST** — where does address-formation (`lis @ha`) go?
2. **Q-DEDUP** — is the hoist per-symbol, per-block, deduplicated?
3. **Q-OPND** — in what order do one statement's operands lower?
4. **Q-STMT** — in what order do statements interleave?
5. **Q-CALL** — what happens across a call?

## 0. Floors

* **P0.1 (deliverable floor, outcome `built`)** — all five sub-questions
  answered; the hoist-membership rule (which tuple class moves and to where)
  located to an address range; ≥3 grid cells capable of separating rival
  orderings; every load-bearing claim cites a `dag.c`-band address or a grid
  cell. `p = 0.65`.
* **P0.2 (decline floor)** — if the mechanism cannot be located in the export
  at this budget, the lane reports the black-box rule with its grid evidence
  and the outcome is **FAILED** (the preregistered deliverable includes the
  address); it does not rebrand a partial as a compound headline.
* **P0.3** — this lane is docs-only: `git diff -- crates fixtures scripts`
  empty at tip; no `DISCLOSURE.md` row. `p = 0.9`.

## 1. Neutrality predictions (exact, required)

Docs-only lane ⇒ the binary under test is byte-identical to base `8fbe6ef5`:

* 878-TU workload: **match 25 · Δ 0 · mismatch 0 · codegen-gap 0**, vocab-gap
  845, capture-fail 8 — digit-for-digit the base.
* Census delta: **+0**.
* `cargo test --workspace --release --no-fail-fast`: **1,527 passed / 41
  targets**, Δ 0 on both numbers.
* `scripts/gate.sh --jobs 4`: PASS with `graded tree` identity equal at both
  ends of the run of record (#3058's rule; a moved identity voids the run).

## 2. Prior art this prereg builds on (all landed; no probe of this lane)

* `wb-live` §7.3 + #3053: all six `lis` of `wbl_v1` emitted at `+0x00`…`+0x18`
  ahead of the first `lwz`; `wbl_r1`: no hoist across a call, `lis 11` twice.
* #1727 (w-extdata): a `lis` does **not** hoist above the *preceding* call —
  it stays with the call whose argument it is.
* WB_CHOOSER §3 B-RULE: one `lis` per pool symbol per function, at the top of
  the earliest block dominating every use; the `lfs` stays **at the use**; two
  `lis` sharing a block emit in **first-use order** (B5); one `lis`, one `lfs`
  serve two uses (B6). B-RULE-2: exactly one hoisted instruction is placed
  between a compare and its branch when one is available.
* #1786 (w-xlr): two constants sharing a high half get **one** `lis` hoisted
  above the `cmplwi`, one `ori` per arm.
* #1741 (w-undname): REFHI/REFLO pair positionally; the low half (`addi`)
  stays at the use; two hoist distances inside one body.
* `ORDER.md` / `STORE_SCHEDULE.md` / `SYMBOL.md` (black-box, exact on
  thousands of cells): in a literal store run through one base symbol,
  producers are ranked (use count desc, first-use asc), stores floored at
  `u + j`, producers grouped before store slot `u`; wide `lis`/`ori` pairs are
  **split** by other producers (#644).
* `WB_REGALLOC_FINDINGS.md` §4 claim O: no scheduler pass exists; emitted
  order = the order `dag.c`/`tuple.c` built; **selection → order → registers**.
  Its §4 explicitly did not read the walk. §10 names `dag.c` unread.
* #2364 (w-vsnprnc): in `guard_chain_shared_tail` the hoisted `lis` goes
  immediately before the first rotate move whose destination is r6 or lower —
  a placement sensitive to what it is hoisted *into*.
* #2339 (w-xtea): `Encipher`'s `lwzx` is emitted above the adds it feeds
  (loop body) — recorded there as "software-pipelined"; this lane treats it as
  an open hazard for D5, not as settled.

## 3. Whitebox predictions (W-series, probability form)

| # | p | prediction |
|---|---:|---|
| W1 | 0.75 | The within-block order is fixed in the `dag.c`/`tuple.c` lowering band — no later pass reorders for latency (replicates claim O at the mechanism level) |
| W2 | 0.50 | The hoist mechanism lives **in the `dag.c` TU itself** — the walk builds a per-block DAG and the linearization emits materialization/leaf tuples ahead of their consumers. Rival (`p = 0.35`): the hoist is a separate motion pass in another TU (`lur.c` / `globlopt.c` band) over tuples built in plain source order. Remainder 0.15: something else |
| W3 | 0.60 | The DAG is per **basic block**, keyed on (op, operands) — local CSE; one node per (symbol `@ha`) per block is what makes Q-DEDUP true |
| W4 | 0.85 | Statement roots are processed in **source order** |
| W5 | 0.60 | One statement's tree lowers **depth-first postorder, left operand first** (operands before operator, value tree before the store tuple) |
| W6 | 0.60 | A **call is a region boundary** for the DAG (the walk flushes/kills at calls), which is why nothing hoists across one |
| W7 | 0.35 | `ORDER.md`'s rank rule (use count desc, first-use asc) is recognizable inside the linearization as the emission priority for shared producer nodes |

## 4. Behavioral predictions (D-series; cell bindings frozen in R2)

| # | p | prediction |
|---|---:|---|
| D1 | 0.60 | In a call-free straight-line block, every `lis` materialization (symbol `@ha` and constant high-half alike) emits ahead of the first load, in **first-use order**. Rival (`p = 0.25`): rank order by use count desc. Remainder 0.15 |
| D2 | 0.70 | The hoist is **deduplicated per symbol per block**: a symbol addressed by two statements in one block gets **one** `lis` |
| D3a | 0.85 | `lis` for statements after a call emits **after** the call (replication of #1727 / `wbl_r1` on new cells) |
| D3b | 0.65 | A symbol addressed on **both** sides of a call is **re-materialized** after it (a second `lis`), not carried across in a callee-saved register |
| D4 | 0.75 | Within one statement, a binary operator's two loads emit in **source order** (tested on non-commutative `-`, both operand orders); the `@l` low half stays at the use |
| D5 | 0.50 | Apart from the hoisted materializations, statement bodies (`lwz` / op / `stw`) stay **contiguous per statement in source order** — no cross-statement interleave of loads with another statement's ops in the global-to-global class |
| D6 | 0.55 | A **literal store run to globals** (the `ORDER.md` class re-targeted at `@ha` symbols) still groups value-producers before stores in rank order, with the `lis` hoist layered on top |
| D7 | 0.70 | Statement count (1 → 8) changes register consumption only (allocator side, known), not the ordering rule; the hoist region grows with the number of distinct symbols |

## 5. Grid axes (structural axes first; values vary inside cells)

Frozen as axes now; exact cells + content hash in R2. Statement count
∈ {1, 2, 3, 4, 8}; symbols per statement ∈ {1, 2, 3+}; symbol sharing across
statements ∈ {disjoint, shared}; call interposed ∈ {none, one, two}; same
symbol on both sides of a call ∈ {no, yes}; operand order of a non-commutative
op ∈ {ab, ba}; materialization kind ∈ {symbol `@ha`, constant high-half,
mixed}; producer kind ∈ {loaded value, literal}. Mode: the workload flags
`/nologo /c /GR /O1 /Oi /EHsc` (from `work/dc3-workload/flags.txt`), read via
`/FAsc` listing (order-faithful intra-function — the listing seam) **and** the
obj via `scripts/gt_capture.sh` + `scripts/gt_dump.py` so no claim rests on the
listing alone.

**Controls that can go red, with the positive question asked in advance:**

* **C-DISC** (first-use vs rank): symbol A used once by statement 1; symbol B
  used by statements 2 and 3. *Would this go red if D1 were false in the most
  likely way (rank order)?* **Yes** — rank puts B's `lis` first, first-use puts
  A's first. The cell separates them by construction.
* **C-CALL**: same statement pair as a hoisting cell with a call inserted.
  *Would this go red if the hoist ignored call boundaries?* **Yes** — the
  second `lis` would appear above the `bl`.
* **C-POS** (instrument control): a `wbl_v1` replica must reproduce the known
  top-of-block hoist; if it does not, the instrument or mode is wrong and no
  other cell is read.
* **Structure verification before reading**: the `/FAsc` listing must contain
  exactly one `PROC` per cell, every cell ≥1 `stw`, and every extern symbol
  referenced; a cell folded away or inlined is a broken probe, not data.

## 6. Score discipline

`C2_MAP_METHOD.md` §7: misses stay misses; cells are not re-scored in the
rule's favour; a revised rule fitted after the run is labelled post-hoc and
predicts nothing. Empty scan output is a failure to investigate, not a pass.
