# WB_F0PRICE — item F0 priced by reading: the 8 and the 4 were never in disagreement, and both are floors

**Lane `w-f0price`, 2026-08-27.** Characterization lane under decision 20.
Prereg frozen at `28f32c4f8` in
[`WB_F0PRICE_PREREG.md`](WB_F0PRICE_PREREG.md), **before the image was
opened**; graded in §7 below (4 HIT · 2 MISS · 1 PARTIAL over 7 registered
predictions, and **the two misses are the lane's most useful result**).

Instrument: [`scripts/f0_pipeline.py`](scripts/f0_pipeline.py), five
subcommands, `--verify` first. Tree `42f76b849`. Pinned image
`compilers/X360/16.00.11886.00/c2.dll`, sha256
`c80981c015166effecc71ad8112d5577a065b2300891dfdb02b9c13787a66258` — **checked,
MATCH**, so decline criterion 1 is clear and every address below is quotable.
Flat export dated 2026-08-04.

---

## 0. The one-paragraph answer

**There is no disagreement to settle.** The `4` is a **dated revision** of the
`8` (2026-08-21 supersedes 2026-08-15) and `P_REGALLOC.md` §7 still says `8`
because the commit that edited §7 two days *after* the revision struck a
different bullet and left this one standing. That is a doc-maintenance defect
with a one-line fix, not a live dispute, and this lane did not need to open the
image to find it.

**What the image says is the finding.** Both numbers rest on **one enumeration**
— `WB_ITEMF_FINDINGS.md` §6.1's eight named sub-items — and that enumeration's
scope claim is measurably too small. Read off the pinned image: the pipeline
downstream of the register allocator is **7 stage drivers, 34 distinct depth-1
passes, 10,714 bytes, 17 translation units**, of which **27 of 34 are
`cover=none`** — no document in this repository mentions them at all. F0's eight
sub-items name **one** of the 34.

**And sub-item 7 is under-priced by the exact check its own author proposed.**
`rungs/2026-08-15-itemfprice.md` §10 item 4 says of the lowering band
*"reading their sizes is a `grep` away and would tighten F0's 8"*. **That check
returns 425 bytes and would have tightened F0 downward, wrongly** — the three
addresses hold **zero transformation logic**; they are pass drivers, and they
drive **15 depth-1 passes / 3,101 B across 11 TUs** (11 live at the workload's
`/O1 /EHsc`, 2,684 B, 9 TUs). Sub-item 7 is priced at 1 lane for "three
passes". It is not three passes.

> **F0 ≥ 10 raw sub-lanes on the enumeration's own scope, and that scope is
> incomplete, so 10 is a FLOOR and not a price.** Both published figures — 8
> and 4 — are below it. The honest deliverable is not a scalar; it is §6's
> table, which says which parts are priced, which are UNPRICED, and against
> what denominator.

---

## 1. The chronology — measured from git, not from the pages

| date | commit | what it did to F0's price |
|---|---|---|
| 2026-08-15 | `c9ead721b` | `WB_ITEMF_FINDINGS.md` §6.1 publishes **F0 = 8**, as an **enumeration of eight named sub-items**, *"ceiling, NO discount factor"* |
| 2026-08-18 | `14c3cc259` | `ref/P_REGALLOC.md` created; §7's F0 bullet quotes *"F0 — priced at 8"* (via `#3243`, which quotes `#3170`) |
| **2026-08-21** | `194c1a6a6` | `STEP5_PRICING_2026-08-21.md` §3 **re-prices F0 8 → 4 lanes raw (×5 = 20)**. Item F goes 17 → 13 |
| 2026-08-21 | `a0d3bb58b` | `READ_PLAN` §3 row **R7** is written, and its *"F0 re-priced 8 → 4 raw"* **quotes that same-day re-price as R7's justification** — R7 does not itself claim to re-price anything |
| **2026-08-23** | `2577daaac` | R4's corrections **edit `P_REGALLOC.md` §7** (hunk `@@ -298,7 +338,15 @@`), striking the *F1* bullet — and **leave the F0 bullet's `8` untouched**, two days after it was superseded |

> **So `P_REGALLOC` §7 vs `READ_PLAN` R7 is a STALENESS collision.** Not a units
> collision, not two live estimates, and not a question that needed a read. The
> brief's framing — *"quoted at 8 in one place and 4 raw in another"* — is
> accurate as an observation and the inference *"pick the read that settles
> it"* over-reads it: the tree already contains the settlement, dated, and the
> defect is that one page did not receive it.

### 1.1 There IS a units problem beside it, and it is the damaging one

The secondary question the brief raised is real, but it is not the 8-vs-4 pair:

* the **8** is published as *"Ceiling. **No discount factor applied** — five of
  the six times one was applied on this project it was the error"*
  (`WB_ITEMF_FINDINGS.md` §6.1);
* the **4** is published as a lower bound with `CEILING` §5's ~5:1 applied **in
  the same sentence**: *"F0 8 → 4 lanes raw (**×5 = 20**)"* (`STEP5_PRICING` §3).

**The two figures a reader can actually find in this tree are 8 and 20.** Read
as published, the *"re-price 8 → 4"* is a **2.5× increase**. Raw-to-raw it is a
halving. Neither comparison is stated anywhere, and `8 × 5 = 40` is never
published, so no page offers a like-for-like pair. **An upper bound and a lower
bound are being quoted side by side as if they were rival point estimates** —
including by the brief and by decision 20, in good faith, because the pages
give them nothing else.

**And applying `CEILING` §5's multiplier to the 8 would be wrong anyway.** §5's
~5:1 is an **optimism** correction — the project's forward estimates come in
low. The 8 was constructed as a ceiling *precisely to not be optimistic*, with
a registered anti-inflation check that fired twice and whose collapses were
taken **out** of the total. Multiplying it corrects a bias its author already
removed by hand.

---

## 2. What the 4's own justification rests on — and what happened to it

`STEP5_PRICING` §3's argument is one paragraph:

> *"That order is now **directly readable at six phases plus after run 4**, and
> the register assignment is readable beside it. F0's cost changes in KIND:
> from a black-box search over obj-visible consequences to a differential read
> against a live trace."*

Two things about it, neither of which is a criticism of the lane that wrote it:

1. **It halves a number without re-enumerating what the number counts.** The 8
   is eight named sub-items. §3 says *"the 4 that leave are **search lanes**,
   not construction lanes"* — but **at most one of the eight is a search
   lane**: sub-item 3, whose cell says *"with DISCLOSURE rows, since the grid
   shows the latencies' **consequences** and not their **values**"*. Sub-items
   1, 2, 4, 5, 6, 7 and 8 are all construction (build an IR, build a region
   finder and a DAG builder, build a cycle loop, build K1/K2, build M4, build
   the band, build the interleave). **There is no set of four search lanes in
   the enumeration to remove.** Registered as **P2** and scored **HIT**.

2. **The lane sent to validate it found it could not be validated.**
   `READ_PLAN` R7 was justified as *"F0 re-priced 8 → 4 raw"* by turning
   `P_DAG`'s `[R]` model `[O]` against the tap. R7 ran, and board **#3435** is
   its own verdict on that premise:

   > *"This re-prices R7's own premise. The read plan justified R7 as 'F0
   > re-priced 8 → 4 raw' by confronting the read model against the tap; **that
   > confrontation is not available on this corpus at any price**"* — the final
   > schedule reorders **3 of 357 functions (0.84 %)**, and a simulator
   > returning its input scores **98.9 %**.

   So the `[R]` → `[O]` step the 4 assumed is not merely undone, it has **no
   population**. `WB_SCHEDCONF_FINDINGS.md` §8 prices building one at ≈1 day.
   **That day is not in either the 8 or the 4.**

---

## 3. The lowering band — this lane's read, and the check that would have misled

`WB_ITEMF_FINDINGS.md` §6.1, sub-item 7 verbatim: *"the lowering band
`0x10b7dd2c`/`0x10b7ddff`/`0x10b7de4a` — **three passes, unread by any lane**,
and §4.2 puts them on item F's path"*. One lane.

**Two things in that sentence are wrong, and they are wrong in opposite
directions**, which is why the naive check fails.

### 3.1 They are not passes. They are drivers with no transformation logic.

Read whole from the flat export. `FUN_10b7ddff` @ `0x10b7ddff`, complete:

```c
void __fastcall FUN_10b7ddff(int *param_1)
{
  DAT_10c2e2dc = 0;
  FUN_10bec297();                       /* the abort poll — P_DAG.md §2's correction */
  _DAT_10c2e2ec = 0;                    /* the phase beacon */
  if (DAT_10c2e2fc != 0) {              /* /Og */
    FUN_10b39e59((int)param_1);
    FUN_10b3668d((int)param_1);
  }
  if ((param_1[0x25] & 0x8000U) == 0) {
    FUN_10b3c6e5(param_1,0);            /* THE BLOCK MERGER, mode 0 */
  }
  _DAT_10c2e2ec = 0;
  return;
}
```

`0x10b7dd2c` (211 B) and `0x10b7de4a` (139 B) have the identical shape: abort
poll, beacon store, gated call, repeat. **Not one arithmetic operation on a
tuple in any of the three.**

| | measured |
|---|---:|
| the three **entries** | **425 B** — `211 + 75 + 139` |
| what they **drive**, depth 1 | **15 passes, 3,101 B, 11 TUs** |
| live at the workload's `/O1 /EHsc /GR` | **11 passes, 2,684 B, 9 TUs** |
| POGO-dead (`DAT_10c3de20 == 2`) | 4 passes, 417 B — `0x10bb3256`, `0x10bb537d`, `0x10b37b30`, `0x10ba60bf`, all in `pogoopt.c`/`pogocg.c`, and squarely inside `WB_ITEMF` §9's named `/LTCG:PGI` coverage bound |
| transitive closure, depth 2 | 92 functions, 24,686 B |

> ### ⚠ THE GENERALIZABLE RESULT — the proposed check would have moved F0 the WRONG WAY
>
> `rungs/2026-08-15-itemfprice.md` §10 item 4, verbatim: *"**`0x10b7dd2c` /
> `0x10b7ddff` / `0x10b7de4a`, the lowering band**, are priced as one sub-lane
> on the strength of their *position*. **Reading their sizes is a `grep` away
> and would tighten F0's 8.** Not taken: it is F0's work, not a pricing lane's."*
> And §2: *"**If any of the three is large, F0 is larger.**"*
>
> **None of the three is large. 425 B, the smallest stage in the pipeline.** The
> proposed `grep` returns "small", the registered inference says "F0 is not
> larger", and **the truth is the opposite**: the band drives 11 live passes
> across 9 TUs, and it is the *only* sub-item that is a pipeline stage rather
> than a component.
>
> The lane that wrote it was right that the size was one `grep` away and right
> to leave it; what neither it nor this lane's own prereg anticipated is that
> **the size of a driver is uncorrelated with the work it drives**, so the
> cheap check was not merely insufficient — it was **anti-correlated with the
> answer**. This lane registered **P3** (*"the three total > 2,000 B"*, p 0.55)
> and **P4** (*"> 40 distinct direct callees"*, p 0.50) and **both are
> MISSES** — 425 B and 15. The misses are the finding, and they are the fourth
> instance of `#3505`'s family in a new dress: **the instrument (byte size)
> measured itself (driver framing), not the quantity.**

### 3.2 "Unread by any lane" is false for one of the three

`0x10b7dd2c` carries a **hand label at `medium` confidence** in `ADDR.tsv`,
filed by `w-select`:

> `emit.pass.pipeline: A pass-pipeline driver: FUN_10bc6487, then the peephole
> 0x10c182b4 (gated), then 0x10bd1068, 0x10c113f3, 0x10c2764e, 0x10c2226b. The
> pass ORDER around register allocation was not established by this lane`

`ADDR.tsv` also shows it cited in `WB_SELECT_FINDINGS_R2.md` (3×) and
`WB_SELECT_RECONCILED.md` (2×), **both of which predate the price**. The label
even names the shape — *"a pass-pipeline driver"* — that this lane had to
re-derive. Registered as **P5**, **HIT**.

### 3.3 The band contains a merger run nobody prices

`FUN_10b3c6e5(param_1, **0**)` sits inside `0x10b7ddff`. `0x10b3c6e5` is the
block-merger driver, and `WB_DAGCLIENTS_FINDINGS.md` §2 already publishes the
complete picture:

> *"`0x10b3c6e5` has exactly four callers — `0x10b7ddff`, `0x10b7ded5`,
> `0x10b7e032`, `0x10be0af1` — and **three of them pass `mode = 0`**"*, while
> K1/K2/K3 are all gated `mode == 2`.

So the merger driver runs **three times downstream of the allocator** — mode 0
in the band (S3), mode 2 in `0x10b7ded5` (S5), mode 0 again in `0x10b7e032`
(S7) — and **F0's sub-items 5 and 6 price only the mode-2 run's clients.** The
two mode-0 runs reach `0x10b3c2cc`'s *other* dispatch arms, which no sub-item
names.

`0x10b3c2cc` (1,033 B) dispatches to **27 callees, 6,468 B**. Sub-items 5 and 6
name **3 of the 27** (K1 `0x10b3b167`, K2 `0x10b3b41b`, M4 `0x10b3baa8`); with
K3 `0x10b3b5fd` — which `#3170` itself lists as *unpriceable* — that is 4 of 27.
Ten of the 27 are `cover=none`.

---

## 4. The scope gap — the pipeline is seven stages, and the enumeration counts four

`WB_ITEMF_FINDINGS.md` §4.2's headline, which is `#3166` and which every later
price inherits:

> *"The instruction order in the obj is that, plus schedule pass 3, plus the
> three-pass lowering band, plus five block mergers, plus the final mode-0
> schedule. **FOUR order-changing stages**, and the order that decided the
> registers is not in the object file."*

The composition is right and the count is short. `FUN_10b7e6af` @ `0x10b7e6af`,
read whole (`--stages` reproduces this):

| stage | driver | B | depth-1 passes | B | named by an F0 sub-item? |
|---|---|---:|---:|---:|---|
| **S1** sched pass 3 | tail of `0x10b7dc51` | — | 1 | 700 | yes — `0x10be6382` |
| **S2** | `0x10b7dd2c` | 211 | 8 | 1,998 | the driver, by sub-item 7 |
| **S3** | `0x10b7ddff` | 75 | 3 | 669 | the driver, by sub-item 7 |
| **S4** | `0x10b7de4a` | 139 | 4 | 434 | the driver, by sub-item 7 |
| **S5** | `0x10b7ded5` | 130 | 5 | 2,128 | 2 of its 5, via `0x10b3c6e5`'s clients |
| **S6** the final schedule | `0x10b7df57` | 219 | 6 | 3,516 | 1 of 6 — `0x10be6382` |
| **S7 the emit tail** | **`0x10b7e032`** | **225** | **10** | **2,489** | ⛔ **NONE. No sub-item names this stage or anything in it.** |
| S8 | `0x10b9c836` | 134 | 0 | 0 | n/a (POGO-instrument gated) |

**Union: 34 distinct passes, 10,714 B, 17 TUs. 27 of the 34 are `cover=none`.
F0's eight sub-items name 1 of the 34.**

### 4.1 S7 is not a formality — it moves tuples and it ends in the emit walk

`FUN_10b7e032` @ `0x10b7e032`, read whole:

```c
if ((*(uint *)(*param_1 + 0x20) & 0x1000) != 0) {      /* the EH gate — /EHsc */
    FUN_10c21b03(param_1); FUN_10be46f0((int)param_1);
    if (DAT_10c3de20 == 0) { FUN_10b3c6e5((int *)param_1,0); }   /* MERGER, mode 0 */
    FUN_10b35c78((int)param_1);
}
if (DAT_10c3de20 == 2) { FUN_10b9d6be((int *)param_1); }
FUN_10b36169(...);  FUN_10c12099(...);  /* DAT_10c2e308 -> */ FUN_10b821c3(...);
FUN_10c275a7(...);  FUN_10b3421b(...);
```

* it is **not `/Og`-gated at all** — the tail runs at `/Od` too;
* `DAT_10c3de20 == 0` is *no POGO*, i.e. **the workload's own mode**, so the
  mode-0 merger call is live;
* **`FUN_10b35c78` @ `0x10b35c78` (86 B, `factor.c`) is a DIRECT caller of a
  tuple-splice primitive** — and it is the function
  `WB_BLOCKORDER_FINDINGS.md` §6 leaves open by name as R8's *"named, unread
  candidate"* for whether the decision tree's arm order is a block **move** or a
  leaf **materialization**;
* `FUN_10b3421b` → `FUN_10b338f5` is **the emit walk itself**
  (`ref/P_BLOCKORDER.md` §3).

`ref/P_BLOCKORDER.md` §3 (R8, 2026-08-23) already tabulates this pipeline as
**eight** numbered passes with `FUN_10b7e032` as *"the emit tail"* and lists its
ten callees. **Two documents in this tree contain the correction to `#3166`'s
count and neither is joined to it** — which is `#3151`'s disease exactly, and
`#3165` is the row that named it.

### 4.2 A derived denominator, bracketed rather than asserted

F0 is denominated in *order*. Order is authored through the tuple list's splice
primitives — `0x10bd3852` unlink, `0x10bd38b0` insert-before, `0x10bd3892`
insert-after (all three read whole; `tuple+0` = next, `tuple+0x10` = prev), plus
the scheduler's bulk relink `0x10be626c`. Partitioning the 34 (`--splice`):

| group | n | B | meaning |
|---|---:|---:|---|
| **A** direct caller of a splice | **4** | 1,674 | **CAN move tuples**: `0x10c21b03` (S7), `0x10be6382` (S1/S6), `0x10b3668d` (S3, in the band), `0x10b35c78` (S7) |
| **B** reaches one transitively | 18 | 4,068 | MAY move tuples |
| **C** reaches none | 12 | 4,972 | cannot reorder\* |

> **\* The bracket is the honest form and group C is the soft edge.** Group C is
> sound only under the premise that no pass rewires `tuple+0`/`tuple+0x10`
> inline. **This lane did not verify that premise**, so C is a count under a
> stated assumption, not a proof. Two documents name the same primitive set
> independently (`WB_DAGCLIENTS_FINDINGS.md` §2 *"the splices"*;
> `ref/P_BLOCKORDER.md` §1's emit walk *"follows `tuple+0` and does nothing
> else"*) — corroboration, not closure. Note also that group C is *"cannot
> **reorder**"*, not *"cannot affect the obj"*: `0x10c182b4`, the peephole, is
> in C and rewrites opcodes in place across 18 arms.

> **The order-changing set is 4–22 of 34 passes. F0's enumeration prices 1.**
> Even at the bracket's floor, **two of the four confirmed splicers
> (`0x10b3668d`, `0x10b35c78`) are in stages F0 prices at one lane and zero
> lanes respectively.**

---

## 5. The eight sub-items, resolved — with the denominator on every row

Every address in `WB_ITEMF_FINDINGS.md` §6.1's F0 cell, checked against
`ref/FUNCS.tsv` **and** the pinned image, entry *and* size (`README.md` §5.4).
Verdicts from the closed set registered in the prereg §6.

| # | what §6.1 counts | address · size | resolves | read state today | §6.1 | this lane |
|---:|---|---|---|---|---:|---|
| **1** | a tuple-level IR below item A | — *(no address)* | n/a | **UNREAD as a design.** It is `ARCHITECTURE_PROPOSAL` row **4b** ("IR3 gets its own step"), a proposal **step**, and probe C `#3354` measured the projection **undefined**, not unequal. `STEP5` §3 concedes it: *"Probe C's residue is F0's residue"* | 1 | **UNPRICED** — 1 is a placeholder for a proposal step, not a lane estimate |
| **2** | region finder + DAG builder | `0x10be5d4b` 101 B · `0x10b328da` 2,231 B | ✅ both | region finder **`[O]` 1,461/1,461** (R7) *and its rule corrected* — 3 categories stop inclusive, 1 exclusive, plus an undocumented head special case firing on 1,121/1,461. DAG builder **`[R]`** | 1 | **1** — characterization half discharged, construction untouched |
| **3** | machine model — latencies, priority, issue | `0x10c1c1d4` 380 B · `0x10be5df6` 453 B · `LAB_10c1bfe2` | 2 of 3 are function entries; the third is a **label inside** a function, as `P_DAG` §2 already flags | **READ by R7** → `SCHED_LATENCY.tsv`, 10/10 from raw bytes. **And the read enlarged the job**: matrix cells are **TAGS** (6 negative tags dispatch on 4 inputs); the weight table is **8** entries not 7, with 2 live terms absent from the published formula | 1 | **1, and the cell's premise is refuted** — it says *"the grid shows the consequences and not the values"*; the values are now read, and they are not values |
| **4** | cycle loop · ready list · `node+0x44` | `0x10be60c0` 428 B · `0x10be5cea` 28 B · `0x10b327cd` 158 B | ✅ all | `[R]` + **four R7 corrections**: the list is re-priced and **fully re-sorted every cycle**; `node+0x44` is truncated to **16 bits**; the issue test is `<= cycle + slack(unit)`, not `<= cycle`; the compared field is `+0x3c` not `+0x38` | 1 | **1 + UNPRICED verification.** `#3435`: no population on this corpus can grade a scheduler model; building one is ≈1 d and is in neither published figure |
| **5** | K1 / K2 | `0x10b3b167` 692 B · `0x10b3b41b` 482 B | ✅ both | **READ** with complete gates (`w-dagclients`, `w-merger4`) — and both are **`mode == 2` only** | 1 | **1**, but see §3.3: it prices 2 of `0x10b3c2cc`'s **27** clients |
| **6** | M4 | `0x10b3baa8` 205 B → `0x10b3a790` 1,014 B | ✅ both | **READ, `[O]`** with ablation cells (`WB_MERGER4_FINDINGS.md`) | 1 | **1** |
| **7** | the lowering band, *"three passes, unread by any lane"* | `0x10b7dd2c` 211 B · `0x10b7ddff` 75 B · `0x10b7de4a` 139 B | ✅ all three, sizes exact | ⛔ **WRONG TWICE** (§3): they are **drivers with no transformation logic**, and `0x10b7dd2c` is **labelled** in `ADDR.tsv` by `w-select`. They drive **15 passes / 3,101 B / 11 TUs** (11 live, 2,684 B, 9 TUs) | 1 | **≥ 3, floor** — one per driver is already generous against 11 live passes in 9 TUs; not priceable at lane granularity without reading them |
| **8** | the four-pass interleave with globregs | `0x10b7dc51` 219 B · `0x10b57633` 541 B | ✅ both | **READ** (`P_DAG` §1, `P_REGALLOC` §2), with the correction that the `/Og` flag is **necessary, not sufficient** — a second gate on `func+0x1c` bit 0 | 1 | **1** |
| | | | | | **8** | **≥ 10 + 2 UNPRICED** |

**Denominator, stated plainly: 7 of 8 sub-items are priced here; 1 (the IR) is
UNPRICED because its unit is a proposal step. Sub-item 4 carries a second
UNPRICED term (its verification population). Beyond the eight, stage S7 and the
two mode-0 merger runs are UNPRICED because no sub-item names them.**

---

## 6. The price, with its derivation

**Do not quote a scalar from this section without §6.2.**

### 6.1 On the enumeration's own scope

```
  sub-item 1  the tuple-level IR                    UNPRICED  (a proposal STEP, not a lane)
  sub-item 2  region finder + DAG builder                  1
  sub-item 3  the machine model                            1
  sub-item 4  cycle loop / ready list / tie-break           1  + UNPRICED verification (~1 d, no population)
  sub-item 5  K1 / K2                                      1
  sub-item 6  M4                                           1
  sub-item 7  the lowering band          1  ->        >=   3   (11 live passes, 9 TUs)
  sub-item 8  the four-pass interleave                     1
                                                     ---------
                                        F0  >=  10 raw sub-lanes, + 2 UNPRICED terms
```

### 6.2 And the scope is the larger error

The 10 is a floor **inside a boundary that is itself too small**. Against the
pipeline measured in §4:

* **34** depth-1 passes downstream of the allocator; F0 names **1**;
* **27 of 34** are `cover=none` — unmentioned anywhere in this repository;
* the order-changing subset is bracketed at **4–22 of 34**, and **2 of the 4
  confirmed splicers sit in stages F0 prices at 1 lane and 0 lanes**;
* **stage S7 (`0x10b7e032`, 10 passes, 2,489 B) has no sub-item at all**, and
  it contains a live merger run, a direct tuple splicer that is another read's
  named open question, and the emit walk.

**So: F0 is more expensive than both published numbers, and the number is not
the deliverable.** Anyone re-pricing F0 should re-run
`scripts/f0_pipeline.py --stages --splice` and price against **34**, not
against **8**.

### 6.3 What this does NOT re-price

* **Not item F.** F0 is one of seven steps and the other six are untouched.
* **Not I1 / I2** — `WHITEBOX_LEVERAGE` §3.1(1) forbids it, and sub-item 1 is
  where F0 touches them.
* **Not `CEILING` §6.1's table**, which stays as measured; it is a correct
  black-box price and quotable as one (`w-readdocs`' 2026-08-22 annotation).
* **Not the lane→time conversion.** `WHITEBOX_LEVERAGE` §3.1 measured that
  *"every forward-cost figure in this program is denominated in a unit the
  program does not spend"* — R2 priced 2–4 d ran in 1 h 36 m, R5 priced 15–25 d
  ran in 30 min. **A floor of 10 sub-lanes says nothing about wall clock**, and
  the two are not convertible here. This lane declines to invent the conversion,
  in the same spirit as `#3355`'s E3.

---

## 7. The prereg, scored — 4 HIT · 2 MISS · 1 PARTIAL

| # | prediction | p | result |
|---|---|---:|---|
| **P1** | the 8 and the 4 are the same nominal unit and not the same quantity; published-to-published the "reduction" is an increase | 0.60 | **PARTIAL.** The units observation holds and is §1.1 (8 ceiling vs 20 calibrated are the only published pair). But P1 reached for a *units* explanation when the simpler and stronger one is **staleness** (§1) — the 4 supersedes the 8 by date and one page did not receive it. Scored PARTIAL because the registered framing found the second-best answer |
| **P2** | *"the 4 that leave are search lanes"* cannot be mapped onto the eight sub-items; at most **2** are search lanes | 0.70 | **HIT.** Exactly **1** (sub-item 3), arguably 2 counting part of sub-item 4 |
| **P3** | the three band entries total **> 2,000 B** | 0.55 | **MISS — 425 B.** §3's box: the miss is the finding |
| **P4** | the band's entries reach **> 40** distinct direct callees | 0.50 | **MISS — 15** (18 with the abort poll). Same box; the depth-2 closure is 92 functions / 24,686 B, but the *entries* are shallow, which is exactly what makes the cheap check mislead |
| **P5** | at least one sub-item names an address that is not what its cell says | 0.60 | **HIT, twice.** Sub-item 7's *"three passes, unread by any lane"* is wrong on both clauses; sub-item 3's `LAB_10c1bfe2` is a label inside a function, not an entry |
| **P6** | the honest price exceeds 8 sub-lanes | 0.50 | **HIT** — ≥ 10 on the enumeration's own scope, and the scope is short by a whole stage |
| **P7** | the answer is a **split** price (characterization owed vs construction owed), not a scalar | 0.65 | **HIT.** Sub-items 3, 5, 6 have characterization discharged and construction untouched; 2 is half discharged; 4's *verification* has no population at all |

**The two MISSES are reported as misses and they carry the lane's most
transferable result** (§3's box): a driver's byte size is anti-correlated with
the work it drives, so the cheap check the origin lane proposed would have moved
F0 **down**. This lane's own prereg made the same mistake — P3 and P4 both bet
on *size* — and only the enumeration discipline of §5 caught it.

## 8. The `#3505` check this lane owes

The eight sub-items are a **published enumeration** (`WB_ITEMF_FINDINGS.md`
§6.1), not a ranking this lane built, and **no conclusion here depends on their
order** — §6.1's price is a sum over a set, and §5's table is presented in the
source document's own order for diffability. The one partition this lane *did*
construct (§4.2's A/B/C splice groups) is a **partition by a read predicate**,
not a ranking by a proxy; its conclusion is a bracket `4–22 of 34` and is
invariant to ordering within groups. `--splice` re-derives it.

**Where a ranking *did* bite is §7's P3/P4**: byte size, used as a proxy for
work, and it was an artifact. Fifth instance of the family, and the first one
where the lane's own prereg supplied the bad instrument.

## 9. What this lane did NOT establish

* **Which of the 34 passes actually move tuples.** §4.2 is a bracket derived
  from call-graph reach, not a behavioural measurement. Closing it needs either
  reading the 22 in groups A+B, or a tap comparing the tuple list across each
  stage boundary — the instrument `w-restim` built for six brackets, extended.
* **Whether group C is closed.** The premise that all reordering goes through
  the four splice primitives is stated and **unverified** (§4.2's box).
* **What the 11 live band passes do.** This lane read the three **drivers**
  whole and none of the 11. Sub-item 7's `≥ 3` is a floor derived from
  structure (11 passes, 9 TUs), not from their content.
* **Any wall-clock conversion** (§6.3).
* **Nothing in `crates/`.** `git diff master..HEAD -- crates fixtures` is empty;
  no `DISCLOSURE.md` row is owed, because this lane adopts no constant — it
  withdraws a price.
