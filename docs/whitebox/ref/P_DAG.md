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

Driver `0x10be6382`. The optimizer-on flag `DAT_10c2e2fc` (bit 21 of the option
word, set at `0x10b82429` — i.e. `/Og` vs `/Od`) is tested **first** at every
one of the four sites, so at `/Od` **none** of the four runs happen `[R]`.

> ### ⛔ CORRECTION 2026-08-20 (lane `w-stageoracle`, fix round) — **the flag is NECESSARY, NOT SUFFICIENT**
>
> This section said the runs are gated *"only"* by `DAT_10c2e2fc`, and
> `WB_DAGORDER_FINDINGS.md` §1 and board **#3067** say the same. The
> disassembly refutes it, and the refutation matters because a lane read
> *"only"* as a licence to treat *"every site fires once per function"* as a
> **structural** fact rather than an empirical one:
>
> ```
> 10b7dc53: 33 db          xor ebx,ebx          ; ...
> 10b7dc58: 43             inc ebx              ; bl == 1 from here on
> 10b7dc83: 39 3d fc e2 c2 10   cmp DWORD PTR ds:0x10c2e2fc,edi   ; the optimizer gate
> 10b7dc89: 74 1f          je  0x10b7dcaa
> 10b7dc8b: 84 5e 1c       test BYTE PTR [esi+0x1c],bl            ; A SECOND GATE
> 10b7dc8e: 74 1a          je  0x10b7dcaa
> ```
>
> The same pair sits ahead of run 2 (`0x10b7dcc2` / `0x10b7dcca`) and run 3
> (`0x10b7dd01` / `0x10b7dd09`): **bit 0 of the function record's `+0x1c`**.
> Run 4 (`sched0`, site `0x10b7e00c`) carries three more beyond its optimizer
> gate at `0x10b7dfd9` — `test DWORD PTR [eax+0x20],0x1000` (`0x10b7dfe3`,
> taken ⇒ skip, over `[esi]`), `test eax,0x400000` (`0x10b7dff2`) and
> `test al,0x8` (`0x10b7dff9`), both over `[esi+0x94]`.
>
> **What survives unchanged:** `/Od` ⇒ zero, because the optimizer `je` is
> reached first. That is the direction the `/Od`-vs-`/O1` null control asserts,
> and it is as grounded as it ever was. **What does not survive:** the converse.
> `hits == functions` at `/O1` is a property of the fixtures measured so far,
> not of the code; a function with `[esi+0x1c] & 1` clear is skipped by three of
> the four sites. `globregs` (`0x10b7dcb7`) and `color` (`0x10b7dcf6`) are not
> optimizer-gated at all — control reaches them at `0x10b7dcaa` / `0x10b7dce9`
> unconditionally.

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
| `0x10b7d85e` | 920 | 1 | 28 | *(gap)* | 2 | the per-function phase pipeline, each phase preceded by a call to **`0x10bec297`** `[R]` — see the correction below; it is **not** a timer |

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


> ### ⛔ CORRECTION 2026-08-20 (lane `w-stageoracle`) — **`0x10bec297` IS NOT "THE TIMER"**
>
> §2's `0x10b7d85e` row and `WB_REGALLOC_FINDINGS.md`'s §3 both called it one.
> It is the **abort / cancellation poll**:
>
> ```
> 10bec297: 83 3d 28 7d c3 10 00   cmp DWORD PTR ds:0x10c37d28,0x0
> 10bec29e: 74 05                  je  0x10bec2a5
> 10bec2a0: e9 97 ff ff ff         jmp 0x10bec23c
> 10bec2a5: c3                     ret
> ```
>
> `DAT_10c37d28` is the same global `_AbortCompilerPass@4` (`0x10bec2ac`) sets;
> when it is set the function tail-jumps into the unwind path and **does not
> return**. Nothing is timed.
>
> It remains a perfectly good **phase beacon** — 143 occurrences of the literal
> in the flat export, in the stereotyped shape
> `call 0x10bec297; mov ecx,esi; mov ds:0x10c2e2ec,edi; call <PHASE>` — and
> that is what a stage tap would use it for. **Whether those sites correspond
> 1:1 to the 35-entry pass-name array at `0x10c2e9e4` is NOT established**:
> `w-stageoracle` registered it at 0.15 against and did not measure it, and the
> name array has **zero code xrefs in the flat export**, so *"COLOR is index
> 14"* is a data fact while *"the pipeline dispatches through that table"* is a
> hypothesis.
>
> The four scheduler-run rows in §1 are **re-derived by measurement** in that
> lane: four separately-patched call sites (`0x10b7dc9f`, `0x10b7dcde`,
> `0x10b7dd1d`, `0x10b7e00c`) each fire exactly once per function, and the
> count equals c2's own `/FAsc` `PROC` count and the obj's `.text` COMDAT count
> (7/7/7 on `fixtures/cpp/il_call_perm.cpp`). Promotes §1's *four runs per
> function* from `[R]` to `[O]`.
>
> **And one thing §1 does not say, now measured**: the register allocator
> writes **nothing** in the tuple record. Over 83 tuples aligned across
> `sched2`→COLOR→`sched3`, a 128-byte window per tuple is byte-identical, while
> the same window moves across the scheduler and across lowering. The assigned
> register is not in the tuple; see `docs/rungs/2026-08-20-stageoracle.md` §3.

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

> ### ⛔ AMENDED 2026-08-23 (read **R7**, lane `w-read-r7`, board **#3433**) — **THE FORMULA ABOVE IS THREE TERMS OF SIX, AND TWO MORE ARE COMPUTED AND DISCARDED**
>
> The block above stands as written; this box is the correction, per this
> page's own revision rule. Grade and full derivation:
> [`WB_SCHEDCONF_FINDINGS.md`](../WB_SCHEDCONF_FINDINGS.md) §2.
>
> **The weight table is EIGHT entries, not §2.1's seven**, and is reached
> **through a pointer** at `0x10c6fe14` — written exactly once, at
> `0x10c1c13b`, with the constant `0x10c3bf9c`. The indirection is real; the
> swap never happens.
>
> | entry | value | term | read at | status |
> |---|---:|---|---|---|
> | `w[0]` | `-1` | `node+0x4e` bit 1 | `0x10be5ed6` | **DEAD** (0/1 term `>>1`) |
> | `w[1]` | `13` | height `node+0x48` | `0x10be5eec` | live |
> | `w[2]` | `8` | fanout `node+0x26` | `0x10be5f35` | live |
> | `w[3]` | `-1` | `node+0x4e` bit 0 | `0x10be5f03` | **DEAD — and bit 0 is the CRITICAL PATH** |
> | `w[4]` | `-2` | — | — | never read by the priority pass |
> | `w[5]` | `10` | `node+0x4e` bit 2 | `0x10be5f1b` | live — the "symdest" term |
> | `w[6]` | `0` | `node+0x4e` bit 3 | `0x10be5f88` | **live, worth 1, gated on typeword `0x5000` — ABSENT ABOVE** |
> | `w[7]` | `0` | dynamic unit bonus 0..7 | `0x10c1bbf6` | **live — past §2.1's "7 shorts"** |
>
> * **c2 computes the critical path and then weights it `-1`.** Bit 0 is seeded
>   on the region head at `0x10be5e5b` and propagated at `0x10be5ec3` to every
>   successor achieving the node's height exactly. Bit 1, set by `FUN_10c1bd6f`
>   at `0x10c1bdad`, is likewise discarded.
> * **The three "keys" are SUMMED into one word, not concatenated.** Fanout is
>   `movzx eax,WORD PTR [esi+0x26]` then `shl eax,8` with **no mask and no
>   clamp** (`0x10be5f39`). So `fanout = 4` contributes exactly what the bit-2
>   flag contributes, and `fanout ≥ 32` carries into height. They are separable
>   keys **only while `fanout ≤ 3`** — which is where `order.rs`'s
>   `MAX_MODELLED_PRODUCERS = 3` (board #541) happens to sit.
> * **Bits 2 and 3 are assigned once at node creation** (`FUN_10b327cd`,
>   `0x10b3280a` / `0x10b32820`): bit 2 iff the operand chain at `tuple+0x28`
>   holds a record of kind byte 2 or 6, bit 3 iff the chain at `tuple+0x2c`
>   does. **The name *"has-symbol-dest"* looks backwards** — the `<<10` term
>   reads `+0x28`, which `WB_MIDDLE_INTERFACES.md:182-183` calls the *source*
>   side. Mechanism exact, name flagged, not renamed: kinds 2 and 6 have no
>   naming table in this repo.
> * **`0x2b8` is not a sentinel, it is MAXIMAL priority** — `or DWORD
>   [esi+0x38],0xffffffff` under an unsigned compare. `FUN_10c1bbaf` reserves
>   `0xffffffff` for it and demotes a genuine overflow to `0xfffffffe`
>   (`0x10c1bc21`) so the two can never tie.
> * **The compared field is `node+0x3c`, the per-cycle WORKING copy, not
>   `+0x38`**, and **`node+0x44` is truncated to 16 bits by the caller**
>   (`movzx …,WORD PTR [x+0x44]`, `0x10be5fe8`/`0x10be5fef`) before an unsigned
>   16-bit compare. The ready list is re-priced and **fully re-sorted every
>   cycle** (`0x10c1bbaf` then `0x10be6046`), so the priority is a function of
>   (DAG, cycle, resource state) — **not a static function of the DAG.**

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

> ### ✅⛔ AMENDED 2026-08-23 (read **R7**, board **#3433**) — **ALL NINE ROWS CONFIRMED, AND THE TABLE IS A LOSSY FLATTENING OF THE MECHANISM**
>
> Every latency above reproduces from the raw image bytes — **10/10**, the two
> branch halves scored separately (`scripts/dump_sched_tables.py --verify`).
> **This section is vindicated as a set of facts.** What it is not is a
> description of how c2 gets them.
>
> * **The matrix cells are TAGS, not latencies.** `0x10c1c261` returns a cell
>   verbatim only when it is `> -2`; six negative tags dispatch on the producer
>   opcode, the consumer opcode, the consumer category and an **edge flag bit**
>   (`0x10c1c294`–`0x10c1c332`). Reading `0x10c3c1a8` as a static grid of
>   numbers yields a plausible and wrong matrix — R7's registered width check
>   went **red on the correct width** for exactly this reason.
> * **ALU→ADDRESS = 5 and ALU→DATA = 2 ARE THE SAME CELL.** Cell `(1,8)` holds
>   the tag `-2` and is **the only cell of all 121 that does**; it resolves to
>   **2** when `edge+0x19` bit 1 is set and **5** otherwise, for consumer
>   opcodes in `[0x14d,0x180]` (52 qualify, `stw` = `0x17a` among them). A model
>   that picks one number for this cell is wrong on the other half — and
>   `crates/` carries the `2` and not the `5`.
> * **The index is `CLASSTAB[opcode]` at STRIDE 12 from `0x10b221d0`**
>   (`0x10c1c234`, `0x10c1c23f`), a table **distinct** from §2.1's machine
>   table. They agree on **660 of 661** opcodes and differ at `0x292`, so
>   §2.1's *"`+8` class IS the unit"* is a near-identity, not an identity.
>   Class 0 short-circuits: **100 of 121 cells are reachable, 21 never are.**
> * **Anti-deps are 0 STRUCTURALLY**, by the `test BYTE PTR [ecx+0x10],0x21`
>   gate at `0x10c1c1e4` — not by a matrix cell.
> * **§2's row citing `0x10c1c25e` as "the ALU→branch cell" is MIS-ADDRESSED.**
>   `0x10c1c25a` is a 7-byte instruction spanning `..0x10c1c260`, so
>   `0x10c1c25e` is mid-instruction. The decision is at **`0x10c1c315`**.
>
> Data: [`SCHED_LATENCY.tsv`](SCHED_LATENCY.tsv). Full decode:
> [`WB_SCHEDCONF_FINDINGS.md`](../WB_SCHEDCONF_FINDINGS.md) §2.4.

---

## 6. What is NOT known here

* **Block emission order.** `M1`'s four arms come out in **source** order;
  `M2`'s seven switch leaves come out in **reverse** case order — same compiler,
  same day `[O]`. Until one rule covers both, a port cannot place labels, and the
  label counter is load-bearing for the COFF symbol records. `WB_REGALLOC`'s
  §4 phrase *"flow-graph construction order"* survives only because construction
  order for a switch is not source order; **as a rule a port could use it is not
  established.**

  > ### ✔ ANSWERED 2026-08-23 by R8 — and the bullet above is wrong about `M2`
  >
  > [`P_BLOCKORDER.md`](P_BLOCKORDER.md);
  > [`../WB_BLOCKORDER_FINDINGS.md`](../WB_BLOCKORDER_FINDINGS.md); board
  > **#3437**–**#3441**. Amended beside rather than rewritten, per
  > [`README.md`](README.md) §2.1 — **the bullet stands as written; it is wrong
  > in one word.**
  >
  > **There is no block-ordering pass and no block-order key.** The emit walk
  > `FUN_10b338f5` @ `0x10b338f5` follows `tuple+0` and does nothing else — no
  > sort, no comparator, no block loop. Emission order **is** tuple-list order,
  > so `M1` and `M2` were never rival rules about *blocks*; they are two
  > traversals inside the `switch` lowering (entry `0x10bd22a7`, decider
  > `0x10bd1373`, recursive driver `0x10bd1f1a`).
  >
  > * ~~*"reverse **case** order"*~~ as this page and four others gloss it —
  >   **it is descending case VALUE, and the reverse-*source* reading published
  >   at `#1906` / `WB_LOOP_FINDINGS.md:449` / `WB_REGALLOC_FINDINGS.md:541` is
  >   REFUTED.** Every prior cell wrote its cases in ascending source order,
  >   where the two are the same sequence and no such cell can separate them.
  > * The rule, `[O]` at 22 of 22 decision-tree cells including a frozen
  >   out-of-sample holdout: table and CTR-ladder lowerings emit arms in
  >   **source** order (the case list is built by append and never sorted);
  >   the decision tree emits `emit(V) = n<8 ? reverse(V) : emit(V[:p]) ++
  >   [V[p]] ++ emit(V[p+1:])`, `p = n/2`, over the **values**. The `8` is read
  >   at `0x10bd1388`.
  > * *"a port cannot place labels"* — **discharged.** `FUN_10bd415e` wraps a
  >   label symbol into the kind-`0x1b` / op-`0x308` tuple the emit walk turns
  >   into an address.
* **A second author of tuple order exists**: a dependence-DAG **block merger**
  (`WB_DAGCLIENTS_FINDINGS.md`, `WB_MERGER4_FINDINGS.md` — `0x10b3baa8` →
  `0x10b3a790`, which is *not* a DAG client). The scheduler is not the only
  thing that moves tuples.
* The **mid-level** (pre-lowering) pass's differences from the machine-level
  one, beyond the `0x2b8` store special case.
* ~~Whether the region-finder's `0x50`-tuple cap ever binds in practice:
  unmeasured.~~ **✅ MEASURED 2026-08-23 (read R7, board #3434): it does NOT
  bind — 0 of 1,461 graded regions reach it, and the longest region observed is
  14 tuples against a cap of 80.** The cap is also a **signed `>`** at
  `0x10be5d66` on a count starting at 0, so up to **81** tuples are visited past
  the head; `0x50` is the constant, not the count.

> ### ⛔ AMENDED 2026-08-23 (read **R7**, board **#3434**–**#3435**) — TWO THINGS THIS PAGE SAYS ABOUT REGIONS, AND ONE IT DOES NOT SAY ABOUT MOTION
>
> **§2's region row is right on the categories and wrong on how they act.**
> Graded **1,461 / 1,461** against the live tap over 60 fixtures
> (`scripts/grade_regions.py`), 100 % at every region length 1–14:
>
> * `0x12`, `0x14`, `0x1b` stop **INCLUSIVE** (the terminator joins the region,
>   `0x10be5d85`); `0x19` stops **EXCLUSIVE** (`0x10be5d7f`), as does
>   `0x17`-with-`0x30f` (`0x10be5d8b`). Treating the four alike is an
>   off-by-one on **every** boundary.
> * **There is a HEAD SPECIAL CASE this page does not mention**: a region whose
>   first tuple has opcode `0x30f` takes it and starts scanning at the second
>   (`0x10be5d55`). **It fires on 1,121 of the 1,461 graded pairs** — the most
>   common path in the whole rule, previously undocumented.
> * Honest coverage: only **three of the rule's seven exits** ever fired.
>   `cap>0x50`, `incl-cat-14`, `excl-cat-19` and `end-of-list` are read and
>   **ungraded**.
>
> **§2's issue row is wrong by a term.** `0x10be60c0` selects the first ready
> node with `node+0x40 <= cycle + slack(unit)` (`0x10be6174`), **not
> `<= cycle`**; `slack` is `max(reservation[unit], 0)`.
>
> **And the thing this page has never said, which R7 measured
> (`scripts/grade_reorder.py`, two independent instruments agreeing):
> ON THIS REPO'S CORPUS THE SCHEDULER ALMOST NEVER MOVES ANYTHING.** The final
> schedule (run 4, `sched0`→`after0`) changes the tuple order on **3 of 357
> functions — 0.84 %**; runs 1 and 2 on 6 and 9. For contrast `globregs` moves
> 334 of 357. Reordering is nearly a function of body length (0.00 % at every
> length ≤ 7, 355 of 456 pairs) because the median region is **2** tuples.
> **A scheduler model graded on this corpus scores ~99 % by returning its
> input**, so the corpus cannot validate one in either direction —
> [`WB_SCHEDCONF_FINDINGS.md`](../WB_SCHEDCONF_FINDINGS.md) §4.
