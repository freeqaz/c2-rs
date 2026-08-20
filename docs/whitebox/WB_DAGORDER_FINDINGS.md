# WB_DAGORDER — the `dag.c` "tree-to-tuple walk" is a dependence-DAG list scheduler

> **PROVENANCE — DISASSEMBLY-DERIVED.** Every address below is an absolute VA in
> the exact image pinned in [`C2_MAP_METHOD.md`](C2_MAP_METHOD.md) §0 —
> `sha256 c80981c015166effecc71ad8112d5577a065b2300891dfdb02b9c13787a66258`,
> verified at the top of this lane. Navigation only. **This lane adopts nothing
> into `crates/` and adds no `DISCLOSURE.md` row.**

PREREG: [`WB_DAGORDER_PREREG.md`](WB_DAGORDER_PREREG.md) committed at
`78971a5d` **before the first grep of the export**;
[`WB_DAGORDER_PREREG_R2.md`](WB_DAGORDER_PREREG_R2.md) (the grid, frozen by
content hash `318bf2d2…`) at `7a91894f` **before the first `cl.exe`**. Scored
in §6. **Lane wrapped early on user instruction** — §8 lists what was cut.

> ### ⚠ REVISED after first publication — three of this document's own claims were WRONG
>
> A delegated second read of the scheduler's machine model landed after the
> first version of this file was committed (`8c240193`). It contradicted three
> load-bearing claims here; **each was re-checked against the export by hand
> before being accepted, and all three of mine were the wrong ones**:
>
> 1. **cmp→branch latency is `0`, not `2`** — and non-cmp ALU→branch is `2`.
>    `0x10c1c25e` reads *"if the producer opcode is **not** in `0x2d`…`0x30`
>    goto lat = 2"*, so the cmp family takes the fall-through `lat = 0`. I had
>    the sense of the test inverted, and §3's claim that wb-chooser's B-RULE-2
>    "falls out of the cmp→branch latency" was therefore right about the cell
>    and **wrong about the mechanism**: the gap comes from the *other* ALU
>    producer's latency-2 edge to the branch.
> 2. **There is a per-unit issue rule**, `LAB_10c1bfe2` (§2): at most **one
>    instruction per unit per cycle** and at most **two nonzero-unit
>    instructions per cycle**, whatever the width says. This was the whole of
>    §8's "unpinned residual" — with it, the simulator goes **6/8 → 7/8** and
>    the `unit` model **outscores** the flat one, so the rule is now
>    discriminated rather than assumed.
> 3. **Mode-0 is the LAST schedule, not a pre-lowering one.** `FUN_10b7e6af`
>    (`0x10b7e6af`) runs `FUN_10b7dc51` (the three mode-1 passes) **first** and
>    `FUN_10b7df57` (mode 0) **last**, after the lowering band.
>
> The corrections are folded into the text below. This box stays because a
> document that silently absorbs its own corrections is one nobody can grade.

**The commissioned question** (#3053/#3057; `CFG_SHAPE.md` §6.2 item F): the
order in which `dag.c`'s walk at `0x10b3219f` lowers a statement list into a
block's instruction tuples.

---

## 1. THE HEADLINE — two published claims are corrected

1. **Board #1823 ("THIS `c2.dll` HAS NO INSTRUCTION SCHEDULER") is REFUTED.**
   There is a cycle-driven dependence-DAG **list scheduler**, reached at `/O1`
   on every optimized function: driver `0x10be6382`, invoked **three times**
   (before globregs `0x10b57633`, between globregs and the register allocator
   `0x10b31c9a`, and after the allocator) from the phase driver `0x10b7dc51`,
   plus a **fourth and final** pass (mode 0) from `0x10b7df57` *after* the
   lowering band — `FUN_10b7e6af` orders them `0x10b7dc51` … `0x10b7df57`;
   gated only by the optimizer-on flag `DAT_10c2e2fc` (bit 21 of the option
   word, set at `0x10b82429`, i.e. `/Og` vs `/Od`: at `/Od` **none** of the
   four runs).
   **⛔ CORRECTED 2026-08-20 (`w-stageoracle`, fix round): "only" is wrong.**
   The optimizer flag is tested *first* at all four sites, so the `/Od` ⇒ none
   direction stands — but three of the four also test `[esi+0x1c] & 1` per
   function (`0x10b7dc8b`, `0x10b7dcca`, `0x10b7dd09`) and the mode-0 run
   carries three further tests (`0x10b7dfe3`, `0x10b7dff2`, `0x10b7dff9`).
   `P_DAG.md` §1 carries the bytes. The consequence is not academic: *"gated
   only by"* was quoted as licence to treat "one run per function" as
   structural, and it is empirical.
   #1823's three "independent ways" were
   three *absences* — no `sched.c` in the ICE-derived TU table (the scheduler
   band `0x10be5cce`–`0x10be663e` sits in an anchor gap between `except.c` and
   `emit.cpp`: a TU with **no ICE site is invisible to that table by
   construction**), a dead flag variable, and stall strings owned by the
   listing writers. Absence read as coverage — the repo's standing defect
   family, in the whitebox record itself.
2. **#3053's mechanism sentence is corrected.** c2 does **not** "hoist every
   `lis <sym>@ha` to the top of the block". Address materializations *lead*
   a scheduling region because the scheduler's priority is dominated by
   **critical-path height** and the ALU→address-consumer latency is the
   largest in the machine model (5), not because any rule names `lis`. On
   `wbl_v1`'s own shape the sixth `lis` is emitted **after** the first `lwz`
   (`dg_v1` at `+0x14`/`+0x18` — and wb-live's recorded span "`+0x00`…`+0x18`"
   for six `lis` already contains that gap; its prose "ahead of every `lwz`"
   is the part that was wrong). At 20 statements (`dg_cap`) the same priority
   rule produces **software pipelining** (`lis`/`lwz`/`addi`/`stw`
   interleaved steady-state) plus scheduler-induced **spills** — the
   block-top picture is a small-n artifact. #2339's "software-pipelined"
   `Encipher` is this scheduler, now located.

## 2. Where it lives

| what | address | notes |
|---|---|---|
| **the commissioned address** | **`0x10b3219f`** | dag.c; 48 bytes: attach a predecessor-less node to the last region-anchor (flag `0x20`) node with a kind-`0x80000` edge — the step that makes independent statements co-scheduled siblings |
| **DAG build, per region** | **`0x10b328da`** | dag.c band (`0x10b31fd4`–`0x10b3333f`); one forward walk of the region's tuples |
| edge add / lookup / create | `0x10b32187` / `0x10b32101` / `0x10b32113` | edge `{+0x10 kind, +0x14 latency}`; create bumps src`+0x26` (fanout) and dst`+0x24` (pred count) |
| node create | `0x10b327cd` | **`node+0x44 = DAT_10c435cc++` — the original tuple index, the tie-break** |
| barrier ("depend on all current leaves") | `0x10b3286b` | applied to branch/call/label-category tuples and a fixed opcode list |
| true-dep edges from last writers | `0x10b322ba` (register kind 4, memory kind 0x80) | walks the last-writer list `DAT_10c435ac`; stops at the nearest **fully covering** writer (`0x10b629af`) |
| anti-dep edges from last readers | `0x10b3227c` (kind 2, latency 0) | list `DAT_10c435b4` |
| call clobber sets (operand kind `0x0b`) | `0x10b3239a` / `0x10b323cb` | dep on every tracked register's writer/readers |
| volatile chain | `0x10b321cf` | kind `0x2000`, total order over volatile-marked operands |
| **region finder** | **`0x10be5d4b`** | ≤ **`0x50` tuples**; ends at tuple category byte (`tuple+8`) ∈ {`0x12`,`0x14`,`0x19`,`0x1b`} or `0x17`-with-opcode-`0x30f` |
| the category enum | ctors `0x10bd3750` (writer), sizes `0x10b18910` | **`0x12` = branch (conditional AND unconditional — the cc is `tuple+0xa & 0x1f`), `0x14` = CALL, `0x1b` = label, `0x19` = body end, `0x17`/`0x30d` = entry**; `0x0d`–`0x16` are real instructions and carry `tuple+9` bit 0, which is the DAG builder's actual gate |
| **priority pass** | **`0x10be5df6`** | height fixpoint + priority; weight table = the **shorts** at `0x10c3bf9c` = `[-1,13,8,-1,-2,10,0]`; the two `-1` weights right-shift a 0/1 term and so contribute **nothing** — those terms are disabled in this build |
| ready-list compare | `0x10be5cea` | **priority desc (unsigned), then `node+0x44` asc** |
| **cycle issue loop** | **`0x10be60c0`** | width `DAT_10c3cf98` (init `0x10c1c0e5`: **2**, or 4 when `DAT_10c3d144 && DAT_10c2e2d2`); picks first ready node with earliest-start ≤ cycle |
| **the issue predicate** | **`LAB_10c1bfe2`** (installed as `DAT_10c3cf8c`) | **unit 0 is free and uncapped; otherwise one instruction per unit per cycle, and at most TWO nonzero-unit instructions per cycle** regardless of width. Then `0x10c1ba4f`: the unit's busy counter must be 0 |
| the unit / latency / reservation table | `0x10b202b0` stride 12 = `{X, slots, class}` | **`+8` class IS the unit** (`node+0x4c`), 11 units: 1 = integer ALU, 3 = branch, 8 = integer load/store, 2 = scalar FP, 4–7 = VMX, 9 = FP/VMX load-store, 0 = none. `+4 slots` is the reservation length (1 for nearly everything, **0** for `lea`/`blr`); `+0 X` seeds `node+0x4a`, the max latency out of the node |
| dynamic priority bonus | `0x10c1bbaf` | per cycle, adds ≤ 7 for unit availability — a tie-break only, since the static weights are 8192/256 |
| microcode / stall penalties | `0x10c1ba6f` | **+15 cycles** for the opcode list at `0x10c3bfb0` (`lha`, `lwa`, `lmw`, `stmw`, `lsw*`, `stsw*` — the Xenon microcoded set); **+40** for the store-forward class, which also ends the cycle |
| the schedule is ITERATED | `0x10c1bdff` | after each pass, recorded latency requests can rewrite edge slots and force a re-schedule of the same region |
| scheduler driver / emission | `0x10be6382` / `0x10be626c` / `0x10be5cce` | re-links the tuple list in scheduled order (tuple`+0` next / `+0x10` prev) |
| per-opcode machine table | `0x10b202b0` stride 12 (`{X, slots, class}`); mnemonic table `0x10b1b260` stride 12 | machine opcodes: `addi`=11, `addis`=14, `b`=31, `bl`=43, `cmp…`=45–48, `lwz`=214, `stw`=378, `li`=624, `lis`=625, `mr`=626, `blr`=645 |
| **edge latency** | **`0x10c1c1d4`**, 11×11 matrix `0x10c3c1a8` (class index from a *second* table `0x10b221d0`) | ALU→ALU **2**; ALU→memory-address **5**; ALU→store-data **2** (flag at `0x10c1bc78`); load→ALU **2**; VMX→ALU **17**; load→load **5**; **ALU→branch: `0` when the producer is `cmp`/`cmpi`/`cmpl`/`cmpli` (`0x2d`–`0x30`), `2` otherwise** (the `-8` cell, `0x10c1c25e` — see the revision box); **CR-setting FP/VMX compare → conditional branch 23** (the `-6` cell: `fcmpo`/`fcmpu` or the `vcmp*` range `0x1ba`–`0x1dd` with the `0xc000` recording nibble); anti-deps 0 |

Priority (`0x10be5df6`, machine-level pass):

    priority = (height << 13) + (fanout << 8) + (has-symbol-dest << 10)
    height   = 1 + max over succ edges (succ.height + edge.latency)
    mid-level store opcode 0x2b8 (pre-lowering pass only) = 0xffffffff

`cost`-free, register-free: **selection → schedule → registers**, and the
allocator (wb-live) runs over the *scheduled* order — which is why live
ranges are a property of this pass (#3053, confirmed at mechanism level).

## 3. THE ANSWER, in the five commissioned parts

1. **Where `lis @ha` goes (Q-HOIST).** Nowhere special. Address
   materializations get the region's greatest heights (the address edge costs
   5 of the deepest chain) and therefore the highest priorities; the cycle
   model then emits them early. Small regions *look* block-top-grouped;
   large ones software-pipeline. The `@l` half is not a separate tuple — it
   rides the consuming load/store (REFLO at the use, every cell).
2. **Dedup (Q-DEDUP).** Not the scheduler's doing, and **not per-block**:
   one `lis` per symbol arrives at the scheduler already (CSE by earlier
   passes), and the CSE scope is at least the whole straight-line function
   **across calls** — `dg_call2` holds `dg_b@ha` in **r31 across the `bl`**
   (the crossing value takes a callee-saved register and the body a frame,
   wb-live's #3051 mechanism) rather than re-materialize a 1-word `lis`.
   The literal `1` used twice is **not** deduplicated (`dg_lit`, two `li 1`).
3. **Within one statement (Q-OPND).** The pre-scheduler lowering order puts
   the **right operand's chain first** (`dg_sub`/`dg_sub2`: `lwz` of the
   *second* source first, both directions), and `+`-chains are
   **reassociated** (`dg_chain`: `b+c+d` computed `(d+c)+b`). Ties in the
   scheduler preserve that order via `node+0x44`.
4. **Across statements (Q-STMT).** Statements do not exist for the
   scheduler: every tuple of a region competes by (height, fanout, original
   index) under the cycle model. **Fanout — use count — beats source
   position** (`dg_disc`: the twice-read symbol's whole chain, statements 2–3,
   runs ahead of statement 1, including its *store*). `ORDER.md`'s
   black-box rank *(use count desc, first-use asc)* is this priority's
   second key and tie-break, rediscovered from bytes; its store-floor
   constant 2 is the ALU→store data latency.
5. **Across a call (Q-CALL).** A call ends the region (`0x10be5d4b`); no
   tuple crosses in either direction (15/15 cells, and #1727 was already
   this fact). But **values** cross freely — region boundaries bound the
   *scheduler*, not CSE (`dg_call2`).

**Reproduction quality, counted, not eyeballed**: a reference simulator of
§2's model ([`scripts/dagorder_sim.py`](scripts/dagorder_sim.py)) reproduces
**7 of 8** call-free cells **instruction-for-instruction** (`one`, `two`,
`v1`, `sub`, `chain`, `lit`, `if` — the last including the
compare/branch-separation slot, i.e. wb-chooser's B-RULE-2 emerges from the
latency model, though via the non-cmp producer's edge, not the cmp's).

**The per-unit issue rule is discriminated, not assumed.** The simulator runs
both micro-models over every cell: the `unit` model (`LAB_10c1bfe2`: one per
unit per cycle, cap 2 nonzero) scores **7/8**, the flat "any two per cycle"
model **6/8**, and the cell that separates them is `v1` — the flat model
front-loads all six `lis`, the unit model interleaves the fifth and sixth with
the first loads exactly as c2 does. Width 2 and width 4 are
**indistinguishable** on this grid, as expected: the cap of 2 binds first.

The one remaining miss (`v4`) is a single transposition — c2 issues the first
`addi` one cycle earlier than the model, where the model prefers the third
load. Candidate causes, unresolved: the dynamic unit-availability bonus
(`0x10c1bbaf`, ≤ 7) breaking a priority tie, or the re-schedule iteration
(`0x10c1bdff`). The residual never reorders across priority classes.

## 4. THE OBJ CHECK

Grid `docs/whitebox/grids/wb-dagorder/dagorder_grid.cpp` (15 cells, sha256
frozen in PREREG R2), compiled with real `cl.exe` 16.00.11886.00 under wibo at
`/nologo /c /GR /O1 /Oi /EHsc`; order read from the `/FAsc` listing and
confirmed against the obj (`gt_dump.py`) — the two agree word-for-word on
every cell checked. Structure verified: 15 PROCs, every cell's externs
present, `dg_cap` a single block. Dumps are `work/w-dagorder/run/` (not
committed); reproduce with `scripts/gt_capture.sh` + `/FAsc`.

| cell | emitted (schematic) | reading |
|---|---|---|
| `dg_one` | `lisS lisD lwz addi stw` | M4 ✓ |
| `dg_two` | `lisS×2 lisD×2 lwz×2 addi×2 stw×2` | phase-grouped, M3 ✓ (n=2) |
| `dg_v1` | 5×`lis`, `lwz`, `lis`, `lwz`×2, `addi`×3, `stw`×3 | **the sixth `lis` follows the first `lwz`** — #3053's prose corrected |
| `dg_v4` | pipelining begins (`stw` of stmt 1 before `addi` of stmt 2) | M3 ✗ at n=4 |
| `dg_shared` | one `lis`, **one `lwz`**, two `addi`, two `stw` | dedup incl. the load, M5 ✓ |
| `dg_disc` | `dg_d`'s chain (stmts 2–3) leads; store order 2,1,3 | **rank beats first-use**, M6 ✓; round-1 D1 ✗ |
| `dg_call` | nothing crosses the `bl` | M7 ✓, M14 ✓ |
| `dg_call2` | `lis r31` before the call, **reused after** (`lwz …(r31)`) | M8 ✗ — carried, not re-materialized |
| `dg_sub`/`dg_sub2` | loads in **reverse source order**, both directions | M9 ✗ — right-operand-first |
| `dg_chain` | `d,c,b` — reassociated | M10 ✗ |
| `dg_lit` | 3×`lis`, 3×`li`, 3×`stw`; **two `li 1`** | producers grouped; no literal CSE, M11 ✗ |
| `dg_if` | `…, addi, cmpwi, stw, beqlr` | **exactly one instruction between compare and branch** — M12 ✓, B-RULE-2 mechanized |
| `dg_cap` | software pipelining + **spill code** (`stw/lwz -0x48..-0x50(r1)`), `__savegprlr_26` | the block-top model's refutation at scale; M13 letter ✓, cap attribution **not established** |

## 5. Corrections and connections to standing records

* **#1823** — refuted (§1). `WB_REGALLOC_FINDINGS.md` §4 claim O ("no pass
  reorders instructions within a block for latency"; "emitted order = the
  order the lowering built") falls with it. Its S1 consistency cell could not
  separate "no scheduler" from "a scheduler that agreed there", and its own
  text said so.
* **#3053** — the miss's *conclusion* (live ranges are a property of the
  lowered order; `dag.c` is the blocker) stands and is now discharged; the
  *mechanism sentence* is corrected (§1.2).
* **`ORDER.md`** — the fitted rank *(use count desc, first-use asc)* =
  `(fanout << 8)` + the `+0x44` tie-break; the store floors' constant 2 = the
  data-edge latency. Navigation-level note; nothing in `ORDER.md`'s shipped
  domain moves.
* **wb-chooser B-RULE-2** — "exactly one instruction between a compare and
  its branch when one is available" is reproduced by the simulator on
  `dg_if`, but **not by the mechanism B-RULE-2's wording suggests**: the
  cmp→branch edge is latency **0** (`0x10c1c25e`), and the separation comes
  from the *other* work in the region being schedulable into that slot under
  the per-unit rule. A port that implements "hold the branch 2 cycles after
  its compare" reproduces `dg_if` and is wrong in general.
* **#1727** ("the `lis` stays with the call whose argument it is") — the
  region boundary, at mechanism level.

## 6. PREREG score

Round 1 (frozen before any probe): **P0.1 cleared** (5/5 sub-questions
address-backed; ≥3 discriminating cells), P0.3 ✓ docs-only.
W1 **M** (a latency scheduler exists — the refutation in §1), W2 **M**
(neither registered alternative: DAG in dag.c, driver in an anchor-invisible
TU), W3 **M** (the DAG is a dependence graph, not value-CSE), W4 **H**
(build walks tuples in list order), W5 **M** (right-first, not left), W6 **H**
(call = region boundary), W7 **H** (registered 0.35 — ORDER.md's rank is the
priority's second key).
D1 **M** (rank beats first-use), D2 **H** (dedup ≥ block; stronger than
registered), D3a **H**, D3b **M** (carried in r31, not re-materialized), D4
**M** (load order clause), D5 **M** (phase grouping/pipelining), D6 **H**, D7
**M** (hoist-region-grows clause fails at n=20).
Round 2: M1 **M** (five of six `lis` precede the first `lwz`, not six — and
this *is* wb-live's span, misdescribed), M2 **M** (one transposition), M3
**M** (n=4 half), M4 **H**, M5 **H**, M6 **H**, M7 **H**, M8 **M**, M9 **M**,
M10 **M**, M11 **M**, M12 **H**, M13 **H** on the registered observable with
the cap **attribution unestablished**, M14 **H**, M15 **M**.

**9 H · 14 M across rounds 1+2's scored rows** — a miss-heavy lane whose
misses are the deliverable: every M above is a false model this document
replaces with an address- or cell-cited rule, and none was re-scored.

## 7. What a port must implement for item F's step 0

```
0. per region (≤ 0x50 tuples, split at branch/call/label — categories
   0x12 / 0x14 / 0x1b / 0x19):
1.   build the dependence DAG: true deps from nearest covering writer
     (registers AND tracked memory symbols), anti-deps latency 0, call
     clobber sets as operands, volatile ops totally ordered, barriers
     for branch/call/label tuples
2.   latencies: ALU→ALU 2, ALU→mem-address 5, ALU→store-data 2,
     load→ALU 2, cmp→branch 0, other ALU→branch 2, anti 0
3.   priority = (height << 13) + (fanout << 8) + (sym-dest << 10);
     ready list (priority desc, original index asc)
4.   cycle loop: issue the first ready node whose earliest-start ≤ cycle
     AND whose unit is unused this cycle, at most 2 nonzero-unit
     instructions per cycle (unit 0 is free); successors' earliest-start
     = cycle + edge latency
5.   the allocator runs AFTER this order exists (three invocations:
     the last one after coloring; a fourth schedule runs after lowering)
```

Steps 1–4 reproduce **7 of 8** call-free grid cells byte-order-exactly today
(`dagorder_sim.py`); the residual is §8 item 1. **Step 4's unit clause is
load-bearing** — dropping it costs a cell.

## 8. What this lane did NOT establish (wrapped early on user instruction)

1. **One transposition in `dg_v4`** — the last simulator miss (§3). The
   per-cycle machinery is now read (`LAB_10c1bfe2`, `0x10c1ba6f`,
   `0x10c1bbaf`), so what is unresolved is narrow: which of the dynamic bonus
   or the re-schedule iteration moves that one `addi`. The two config shorts
   per unit at `0x10c3bf70` have **no reader found**.
2. **Each pass's individual effect** — only the composed order is graded, and
   there are **four** passes (three mode-1, one mode-0 after lowering).
   `dg_disc`'s statement rotation is attributed to the fanout term at
   whichever level the shared node exists; the level was not isolated.
   Widths differ per pass (`DAT_10c2e2d2` is a phase marker set at the tail
   of `0x10b57633`, so passes 2 and 3 run at width **4**) — but the cap of 2
   in `LAB_10c1bfe2` makes that unobservable on this grid.
3. **The region cap's seam** (`dg_cap`) — the naive model is refuted there,
   but pipelining already explains the break; the 0x50 cap itself has no
   isolated witness.
4. **Which TU the scheduler band belongs to** (`mdlist.c` is a candidate by
   name; it has an ICE anchor at `0x10c11060`, far from the band) — left
   open; the ICE-anchor method cannot see a TU without ICE sites, which is
   exactly how #1823 happened.
4b. **Four OTHER clients of the DAG builder** — `0x10b3b167`, `0x10b3b41b`,
   `0x10b3b5fd` (under `0x10b3c2cc`) and `0x10c1ce93` (under the `/QXSTALLS`
   listing writer `0x10b71d8f`) build the same dependence DAG *without*
   `0x10be5d4b`'s region enders. **Unread.** If any of them reorders tuples,
   the ordering story has a second author and this document is incomplete in
   a way its grid cannot detect.

   > ### ⚠ 2026-08-14 — ANSWERED, and the answer is YES (`w-dagclients`)
   >
   > Board `#3071` was right to file this, and it resolved against this
   > document. **`0x10b3b167` and `0x10b3b41b` are a dependence-DAG block
   > merger** (tail-merge / cross-jump and head-merge / hoist): they unlink and
   > re-insert tuples through `0x10bd38b0` / `0x10bd3892` — the same `tuple+0`
   > next / `tuple+0x10` prev links §2 gives the scheduler — and they run at
   > `0x10b7ded5`, i.e. **before** `0x10b7df57`'s final schedule, so the
   > `node+0x44` tie-break is assigned from the *post-merge* order. Ablating
   > them in a patched copy of the image turns a 13-instruction one-copy
   > `if`/`else` into a 16-instruction two-copy one, with the common store
   > moving from above the branch back inside both arms.
   >
   > **Nothing in §1–§7 is contradicted; §7's recipe is INCOMPLETE.** It is
   > sound only on a function whose branches do not admit a merge — which
   > every one of this lane's 15 cells was, by construction. `0x10b3b5fd` is
   > reached but never fires on either of `w-dagclients`' grids, and
   > `0x10c1ce93` is read-only and reachable only under `/QXSTALLS` **and**
   > `/FAsc` — where it runs even at `/Od`, so §1's "at `/Od` **none** of the
   > four runs" is true of the four *scheduler passes* and **not** of the four
   > DAG clients.
   >
   > See [`WB_DAGCLIENTS_FINDINGS.md`](WB_DAGCLIENTS_FINDINGS.md) §1, §4, §5, and
   > board `#3099`–`#3103`.
4c. **A possible latency bug, flagged rather than smoothed**: in the `-6`
   arm of `0x10c1c1d4`, a producer that is neither `fcmpo`/`fcmpu` nor a
   recording `vcmp*` leaves `local_10 = -6`, which is stored into the edge's
   latency slot and read back as a **ushort** by `0x10be60c0` (65530 — an
   effectively infinite earliest-start). No guard was found. No cell of this
   grid reaches it.
5. **The lowering walk itself** (`lower.c` `0x10c053e7` band) — right-first
   operand order and `+`-reassociation are grid facts here, not read code.
6. **Width-4 mode** (`DAT_10c2e2d2`), POGO paths, floats/VMX (the nibble-5
   priority term), and the full gate — the standing gate was **not run** by
   this lane; the diff is docs-only (`git diff -- crates fixtures scripts`
   empty), so the 878-TU numbers cannot move, but that is a construction
   argument, not a measured one.

## 9. Pre-drafted DISCLOSURE rows — NONE

Nothing here is adopted by `crates/`. A future lane that implements §7 in the
port adopts the priority weights, the latency matrix, and the region rule from
addresses named in §2 and owes `DISCLOSURE.md` rows for `0x10be5df6`,
`0x10c3bf9c`, `0x10c1c1d4`, `0x10c3c1a8`, `0x10be5d4b`, `0x10be5cea` and
`0x10b328da` in the same commit. The black-box alternative (re-deriving from
grids alone) is **not** sufficient for the latency numbers — the grid shows
their consequences, not their values.
