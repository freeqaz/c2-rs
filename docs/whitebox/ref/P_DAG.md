# `P_DAG` — `dag.c` and the unnamed scheduler TU: build, priority, cycle model

> **Reference page.** **`[R]`** read from the disassembly, *not* obj-checked —
> a hypothesis. **`[O]`** confirmed against a real obj or `/FAsc` listing, with
> the witness named. **`[I]`** an interpretive step. Navigation only; nothing
> here may enter `crates/` without a [`DISCLOSURE.md`](../DISCLOSURE.md) row.
> Index: [`ADDR.tsv`](ADDR.tsv) · front door: [`README.md`](README.md)

**Coverage: 24 code entries + 8 data/table entries against a denominator of
61** — 48 Ghidra functions in `dag.c`'s span (`0x10b3219f`–`0x10b3433f`) plus
13 in the scheduler band (`0x10be5cce`–`0x10be663f`). Not covered: the machine
model's per-opcode table contents beyond the fields named in §5, the
mid-level (pre-lowering) pass's differences from the machine-level one, and
`factor.c`'s tail merging.

> ### ⛔ THE CORRECTION THIS PAGE EXISTS TO CARRY
>
> **Board `#1823` — "THIS `c2.dll` HAS NO INSTRUCTION SCHEDULER" — is
> REFUTED.** There is a cycle-driven dependence-DAG **list scheduler**, run
> **four times per function** at `/O1`.
>
> **Why the wrong claim was reasonable, which is the transferable part.**
> `#1823`'s three "independent ways" were **three absences**, and the
> load-bearing one was *"there is no `sched.c` in `c2_tus.tsv`"*. That table is
> built from **C1001 ICE sites**, so a translation unit with no ICE site is
> **invisible to it by construction** — and the scheduler band
> `0x10be5cce`–`0x10be663e` sits in an anchor gap between `except.c` and
> `emit.cpp`. A true statement about the **instrument** was read as a statement
> about the **image**. The other two absences (a dead flag `0x10c2eb40` with
> zero readers, stall strings owned by the `/QXSTALLS` listing writers) are
> perfectly consistent with a scheduler that does not use them.
>
> [`ADDR.tsv`](ADDR.tsv) marks this band `tu_conf = no-ice-site` so the index
> cannot repeat the error.

---

## 1. The four scheduler runs

Driver `0x10be6382`, gated **only** by the optimizer-on flag `DAT_10c2e2fc`
(bit 21 of the option word, set at `0x10b82429` — i.e. `/Og` vs `/Od`: at `/Od`
**none** of the four runs happen) `[R]`.

| # | mode | from | when |
|---|---|---|---|
| 1–3 | mode 1 | `0x10b7dc51` | before `globregs`, between `globregs` and the allocator, and after the allocator |
| 4 | mode 0 | `0x10b7df57` | **last**, *after* the lowering band |

`0x10b7e6af` orders them `0x10b7dc51` … `0x10b7df57` `[R]`. **Mode 0 is the
LAST schedule, not a pre-lowering one** — an earlier reading had this backwards
and the correction is recorded in `WB_DAGORDER_FINDINGS.md`'s revision box.

**`selection → schedule → registers`**: the allocator runs over the *scheduled*
order, which is why live ranges are a property of this pass and why
[`P_REGALLOC.md`](P_REGALLOC.md) §5's candidate order moves when only the DAG
moves `[O]`.

---

## 2. Entries

| addr | size | callers | callees | TU | cites | what |
|---|---:|---:|---:|---|---:|---|
| `0x10b3219f` | 48 | 1 | 2 | **`dag.c` anchor** | 25 | **the commissioned address.** Attaches a predecessor-less node to the last region-anchor (flag `0x20`) node with a kind-`0x80000` edge — the step that makes independent statements co-scheduled siblings `[R]` |
| `0x10b328da` | 2231 | 5 | 21 | `dag.c` gap | 26 | **DAG build, per region** — one forward walk of the region's tuples `[R]` |
| `0x10b32187` | 24 | 8 | 2 | *(gap)* | 1 | edge add `[R]` |
| `0x10b32101` | 18 | 1 | 0 | *(gap)* | 1 | edge lookup `[R]` |
| `0x10b32113` | 116 | 3 | 1 | *(gap)* | 1 | edge create — bumps src`+0x26` (fanout) and dst`+0x24` (pred count); edge is `{+0x10 kind, +0x14 latency}` `[R]` |
| `0x10b327cd` | 158 | 1 | 3 | `dag.c` gap | 3 | **node create — `node+0x44 = DAT_10c435cc++`, the original tuple index and THE SCHEDULER'S TIE-BREAK** `[R]` |
| `0x10b3286b` | 111 | 1 | 4 | `dag.c` gap | 1 | the barrier ("depend on all current leaves"), applied to branch/call/label-category tuples and a fixed opcode list `[R]` |
| `0x10b322ba` | 224 | 3 | 7 | `dag.c` gap | 1 | true-dep edges from last writers (register kind 4, memory kind `0x80`); walks `DAT_10c435ac`, stops at the nearest **fully covering** writer (`0x10b629af`) `[R]` |
| `0x10b3227c` | 62 | 2 | 3 | `dag.c` gap | 1 | anti-dep edges from last readers (kind 2, latency 0); list `DAT_10c435b4` `[R]` |
| `0x10b3239a` / `0x10b323cb` | 49 / 47 | 2 / 1 | 2 / 2 | `dag.c` gap | 1 | call clobber sets (operand kind `0x0b`): depend on every tracked register's writer and readers `[R]` |
| `0x10b321cf` | 33 | 1 | 1 | `dag.c` gap | 1 | the volatile chain, kind `0x2000` — a total order over volatile-marked operands `[R]` |
| `0x10be5d4b` | 101 | 1 | 1 | **no-ice-site** | 21 | **the region finder.** ≤ **`0x50` tuples**; ends at tuple category (`tuple+8`) ∈ {`0x12`,`0x14`,`0x19`,`0x1b`} or `0x17`-with-opcode-`0x30f` `[R]`. **A call ends the region** `[O]` 15/15 |
| `0x10be5df6` | 453 | 1 | 4 | **no-ice-site** | 10 | **the priority pass** — height fixpoint + priority. §3 `[R]` |
| `0x10be5cea` | 28 | 1 | 0 | **no-ice-site** | 8 | **the ready-list compare: priority DESC (unsigned), then `node+0x44` ASC** `[R]` |
| `0x10be60c0` | 428 | 1 | 7 | **no-ice-site** | 4 | **the cycle issue loop.** Width `DAT_10c3cf98` (init `0x10c1c0e5`: **2**, or 4 when `DAT_10c3d144 && DAT_10c2e2d2`); picks the first ready node with earliest-start ≤ cycle `[R]` |
| `0x10be6382` | 700 | 2 | 13 | **no-ice-site** | 17 | **the scheduler driver** `[R]` |
| `0x10be626c` | 278 | 1 | 7 | **no-ice-site** | 3 | re-links the tuple list in scheduled order (`tuple+0` next, `+0x10` prev) `[R]` |
| `0x10be5cce` | 28 | 1 | 0 | **no-ice-site** | 4 | emission helper `[R]` |
| `0x10c1bfe2` | *(`LAB_`)* | — | — | `mdmisc.c` gap | 8 | **the issue predicate**, installed as `DAT_10c3cf8c`: **unit 0 is free and uncapped; otherwise one instruction per unit per cycle, and at most TWO nonzero-unit instructions per cycle** regardless of width `[R]`. This was §8's "unpinned residual" — with it the reference simulator goes **6/8 → 7/8** and the `unit` model outscores the flat one |
| `0x10c1ba4f` | 32 | 0 | 0 | `mdmisc.c` gap | 1 | the unit's busy counter must be 0 `[R]` |
| `0x10c1bbaf` | 132 | 1 | 1 | `mdmisc.c` gap | 4 | dynamic priority bonus: ≤ 7 per cycle for unit availability — a **tie-break only**, since the static weights are 8192/256 `[R]` |
| `0x10c1ba6f` | 320 | 1 | 2 | `mdmisc.c` gap | 2 | **microcode / stall penalties**: **+15 cycles** for the opcode list at `0x10c3bfb0` (`lha`, `lwa`, `lmw`, `stmw`, `lsw*`, `stsw*` — the Xenon microcoded set); **+40** for the store-forward class, which also ends the cycle `[R]` |
| `0x10c1bdff` | 483 | 1 | 0 | `mdmisc.c` gap | 3 | **the schedule is ITERATED** — after each pass, recorded latency requests can rewrite edge slots and force a re-schedule of the same region `[R]` |
| `0x10c1c1d4` | 380 | 1 | 0 | `mdmisc.c` gap | 8 | **edge latency**, over the 11×11 matrix `0x10c3c1a8` (class index from a *second* table `0x10b221d0`) `[R]`. §5 |
| `0x10c1c25e` | *(in `0x10c1c1d4`)* | — | — | `mdmisc.c` gap | 4 | the ALU→branch cell: `0` when the producer is `cmp`/`cmpi`/`cmpl`/`cmpli` (`0x2d`–`0x30`), `2` otherwise `[R]` |
| `0x10b7dc51` | 219 | 1 | 5 | *(gap)* | 33 | the phase driver that runs the three mode-1 schedules and the allocator `[R]` |
| `0x10b7df57` | 219 | 1 | 7 | *(gap)* | 21 | the mode-0 (final) schedule `[R]` |
| `0x10b7e6af` | 106 | 1 | 9 | `main.c` gap | 28 | orders the two `[R]` |
| `0x10b7d85e` | 920 | 1 | 28 | *(gap)* | 2 | the per-function phase pipeline, each phase bracketed by the timer `0x10bec297` `[R]` |

### 2.1 Tables

| addr | what |
|---|---|
| `0x10b202b0` | the per-opcode machine table, stride 12 = `{X, slots, class}`. **`+8` class IS the unit** (`node+0x4c`), 11 units: 1 = integer ALU, 3 = branch, 8 = integer load/store, 2 = scalar FP, 4–7 = VMX, 9 = FP/VMX load-store, 0 = none. `+4 slots` is the reservation length (1 for nearly everything, **0** for `lea`/`blr`); `+0 X` seeds `node+0x4a`, the max latency out of the node `[R]` |
| `0x10b1b260` | the mnemonic table, stride 12. Machine opcodes: `addi` = 11, `addis` = 14, `b` = 31, `bl` = 43, `cmp…` = 45–48, `lwz` = 214, `stw` = 378, `li` = 624, `lis` = 625, `mr` = 626, `blr` = 645 `[R]` |
| `0x10c3bf9c` | the priority weight table (shorts) = `[-1, 13, 8, -1, -2, 10, 0]`. **The two `-1` weights right-shift a 0/1 term and contribute nothing — those terms are disabled in this build** `[R]` |
| `0x10c3c1a8` | the 11×11 edge-latency matrix `[R]` |
| `0x10b221d0` | the second class-index table the matrix is addressed through `[R]` |
| `0x10c3bfb0` | the microcoded-opcode list (+15 cycles) `[R]` |
| `0x10c435cc` | the monotonic tuple-index counter, source of `node+0x44` `[R]` |
| `0x10c435ac` / `0x10c435b4` | last-writer / last-reader lists `[R]` |

---

## 3. Priority

```
priority = (height << 13) + (fanout << 8) + (has-symbol-dest << 10)
height   = 1 + max over succ edges (succ.height + edge.latency)
mid-level store opcode 0x2b8 (pre-lowering pass only) = 0xffffffff
```

Ready-list order: **priority DESC (unsigned), then `node+0x44` ASC**
(`0x10be5cea`) — where `node+0x44` is the original tuple index `[R]`.

> **This is the DAG node's `+0x44`, not the allocator candidate's.** The
> allocator's `+0x44` is a different field of a different record in a later
> pass, compared **DESC**, and it is
> [`P_REGALLOC.md`](P_REGALLOC.md) §4's tie-break. Two records, two `+0x44`s,
> opposite directions.

---

## 4. The five commissioned answers `[O]`

1. **Where `lis @ha` goes.** Nowhere special. **c2 does NOT "hoist every `lis
   <sym>@ha` to the top of the block"** — `#3053`'s mechanism sentence is
   corrected. Address materializations get the region's greatest *heights* (the
   address edge costs 5, the largest in the machine model) and therefore the
   highest priorities; the cycle model emits them early. On `wbl_v1`'s shape the
   **sixth `lis` is emitted after the first `lwz`**. At 20 statements the same
   rule produces **software pipelining** plus scheduler-induced **spills** — the
   block-top picture is a small-`n` artifact. The `@l` half is not a separate
   tuple; it rides the consuming load/store.
2. **Dedup.** Not the scheduler's doing and **not per-block**: one `lis` per
   symbol arrives already CSE'd, and the CSE scope is at least the whole
   straight-line function **across calls** (`dg_call2` holds `dg_b@ha` in `r31`
   across the `bl` rather than re-materialize a one-word `lis`). The literal `1`
   used twice is **not** deduplicated.
3. **Within one statement.** The pre-scheduler lowering order puts the **right
   operand's chain first** (both directions tested), and `+`-chains are
   **reassociated** (`b+c+d` computed `(d+c)+b`). Ties preserve that order via
   `node+0x44`.
4. **Across statements.** Statements do not exist for the scheduler. **Fanout —
   use count — beats source position**: the twice-read symbol's whole chain,
   statements 2–3, runs ahead of statement 1 *including its store*.
   `ORDER.md`'s black-box rank *(use count desc, first-use asc)* is this
   priority's second key and its tie-break, rediscovered from bytes; its
   store-floor constant 2 is the ALU→store-data latency.
5. **Across a call.** A call ends the region; **no tuple crosses in either
   direction** (15/15 cells). But **values** cross freely — region boundaries
   bound the *scheduler*, not CSE.

---

## 5. The machine model — the numbers

| edge | latency |
|---|---:|
| ALU → ALU | 2 |
| ALU → memory **address** | **5** — the largest, and the reason address chains lead |
| ALU → store **data** | 2 |
| load → ALU | 2 |
| load → load | 5 |
| VMX → ALU | 17 |
| **ALU → branch** | **`0` when the producer is `cmp`/`cmpi`/`cmpl`/`cmpli` (`0x2d`–`0x30`), `2` otherwise** |
| CR-setting FP/VMX compare → conditional branch | 23 |
| anti-deps | 0 |

> ⛔ **Revision.** An earlier reading had the ALU→branch test **inverted** and
> concluded cmp→branch was 2. `0x10c1c25e` reads *"if the producer opcode is
> **not** in `0x2d`…`0x30`, `lat = 2`"*, so the cmp family takes the
> fall-through `lat = 0`. The consequence: `wb-chooser`'s B-RULE-2 does **not**
> "fall out of the cmp→branch latency" — the gap comes from the *other* ALU
> producer's latency-2 edge to the branch. The original claim stands beside this
> box, as `WB_DAGORDER_FINDINGS.md`'s revision rule requires.

---

## 6. What is NOT known here

* **Block emission order.** `M1`'s four arms come out in **source** order;
  `M2`'s seven switch leaves come out in **reverse** case order — same compiler,
  same day `[O]`. Until one rule covers both, a port cannot place labels, and the
  label counter is load-bearing for the COFF symbol records. `WB_REGALLOC`'s
  §4 phrase *"flow-graph construction order"* survives only because construction
  order for a switch is not source order; **as a rule a port could use it is not
  established.**
* **A second author of tuple order exists**: a dependence-DAG **block merger**
  (`WB_DAGCLIENTS_FINDINGS.md`, `WB_MERGER4_FINDINGS.md` — `0x10b3baa8` →
  `0x10b3a790`, which is *not* a DAG client). The scheduler is not the only
  thing that moves tuples.
* The **mid-level** (pre-lowering) pass's differences from the machine-level
  one, beyond the `0x2b8` store special case.
* Whether the region-finder's `0x50`-tuple cap ever binds in practice: unmeasured.
