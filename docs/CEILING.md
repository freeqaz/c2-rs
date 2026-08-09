# CEILING — the arithmetic between the process and TU match 871

**What this doc is.** The standing goal is **TU match 871 of 878**. Nothing in
this repository computed the distance to it honestly: the ceilings were scattered
across three documents at three different anchors (`28`, `450`, "the wall"), the
conversion rate was a folklore figure with no denominator, and the one place the
goal's binding constraint is stated is a board row nobody cites. This page is
that arithmetic, in one place, **regenerated from the instruments rather than
quoted from history**.

**What this doc is NOT.** It is not an argument for continuing and not an
argument for re-scoping. The re-scope decision is the user's; this page exists to
equip it. Where the arithmetic reads badly it is published as arithmetic.

> **Provenance rule for this page.** Every number below states the instrument
> that printed it and the tree it was collected at, or it carries the
> **HAND-COUNT** tag (`BOARD.md` Conventions, board #1476). A number without one
> of those is a defect in this page. **Never quote a hand-count and a scan
> reading in the same sum.**

**Collection stamp for every INSTRUMENT figure on this page unless stated
otherwise:**

| | |
|---|---|
| tree | **`b234d826`** — `git diff b234d826 -- crates scripts fixtures` is **0 bytes** at the tree the scan ran on (`babf8d42`, docs-only commits above the base) |
| binary | `c28cd1a9bdeb` |
| collected | **2026-08-08** |
| command | `c2rs gap --list work/dc3-workload/files.txt --flags-file work/dc3-workload/flags.txt --cwd <dc3-tree> --jobs 16 --factors-tsv <tsv>` |
| second instrument | `c2rs factors --tsv <tsv> --check-metrics <gaplog>` — **13 OK, 0 DISAGREE, 0 ABSENT of 13 keys** |

A concurrent code lane owns `crates/`. If the tree hash above is not an ancestor
of yours, **re-run the two commands** rather than reading this page. §9 says how.

---

## 1. The population ladder

### 1.1 Top of the ladder — 878 to 871

| step | count | instrument |
|---|---:|---|
| TUs in the dc3 workload | **878** | `gap-metric tu-total 878` |
| − TUs the reference toolchain cannot compile | **7** | `gap-metric capture-fail 7` — 3 × `C1083`, 2 × `C1189`, 2 × `C2084` |
| **= graded, the real denominator** | **871** | `gap-metric graded 871` |

**871 is not a target the project chose; it is what is left after the oracle
declines.** A `capture-fail` TU has no obj and no census, so it is **absent**
from the factor listing rather than present as an all-false row — board **#352**,
deliberately, because summing absences as zeros reports every factor tighter than
it is.

### 1.2 The factorization, as a partition

`match = A ∧ B ∧ C ∧ (D ∨ E)` over the 871 (ROADMAP §10.19, §10.21; board
**#179** for the fifth term). Every TU falls in exactly one cell:

> **⚠ 2026-08-08 — the cells and the margins below were collected at `match 11`
> and the tree reads `match 14`.** Three conversions have landed since
> (`negate_test.cpp` at `w-cfgclass`, `Primes.cpp` at `w-data`,
> `vswprnc.cpp` at `w-extdata`), so the row that moves most is the one this page
> is named for: **`frontier` is 13, not 16**, and `frontier-if-a` is **135**, not
> 138. `factor-d` is **14** and `d-or-e` **16**; `factor-a` 28, `factor-b` 338,
> `factor-c` 169, `b-and-c` 151, `a-and-b-and-c` 27 and `mismatch` 0 are
> **unchanged** across all three. Read from `work/w-extdata/scan_tip2.out`
> (`gap-metric` lines), tree `dc962b63`.
>
> The banner is here rather than a rewrite because this page's own provenance
> rule is *"regenerated from the instruments"* — the cells need a fresh
> `gap --factors-tsv` pass, which is a rung of its own. **Quote the frontier from
> a scan, not from this table**, which is `docs/STATUS.md`'s standing instruction
> for exactly these keys.

| cell | TUs | what it is |
|---|---:|---|
| `-----` | **514** | fails every factor |
| `-B---` | **187** | binds, but the writer cannot emit its sections |
| `-BC--` | **122** | binds **and** section-clean, but the emit **set** is wrong |
| `--C--` | **18** | section-clean only |
| `A----` | **1** | emit-set right, nothing else |
| **`ABC--`** | **16** | **the FRONTIER** — codegen breadth is the whole remaining distance |
| `-BCD-` | **2** | the port already accepts the contents; only factor **A** is missing |
| `ABCD-` | **9** | **match** |
| `ABC-E` | **2** | **match**, via a whole-TU recognizer |
| | **871** | |

Derived by `awk -F'\t' '{n[$8]++}'` over `gap --factors-tsv`, and every joint
below re-derives from it. The margins:

| key | value | instrument |
|---|---:|---|
| `factor-a` (`.ex` segments == `.text` COMDATs, gate-anchored `4F 1F`) | **28** | `gap-metric factor-a` (`factor-a-lo` 27 on the *census* anchor — quote the gate one) |
| `factor-b` (every emitted symbol binds) | **338** | `gap-metric factor-b` |
| `factor-c` (obj section set ⊆ writer's names) | **169** | `gap-metric factor-c` |
| `factor-d` (every emitted COMDAT in the per-function codegen class) | **11** | `gap-metric factor-d` |
| `factor-e` (a registered whole-TU recognizer accepts) | **2** | `gap-metric factor-e` |
| `b-and-c` | **151** | `gap-metric b-and-c` |
| `a-and-b-and-c` | **27** | `gap-metric a-and-b-and-c` |
| **`d-or-e`** | **13** | `c2rs factors` set cardinality |
| `frontier` | **16** | `gap-metric frontier` |
| `frontier-if-a` | **138** | `gap-metric frontier-if-a` |
| **`match`** | **11** | `gap-metric match` — the sole judge |
| `mismatch` | **0** | `gap-metric mismatch` — **not evidence of correctness**, see §8 |

---

## 2. The ceilings, each with its provenance

There are five distinct ceilings in this project and they have been quoted
interchangeably. They bound different things and only one of them bounds TU
match.

### 2.1 The frontier ceiling — 27, of which 16 are open

`A∧B∧C` = **27**. Eleven are already matched, so **16 TUs are reachable by
codegen breadth alone** and that is the FRONTIER. This is the hard bound on TU
match *before an emit-set model exists*: on the other 844 the port is wrong about
which functions to emit regardless of how correctly it lowers each body.

**What would move it:** factor A — an emit-set model (Phase 7). Nothing else.

**What it is worth:** 27 − 11 = **16 conversions, ever**, and every one is gated
on codegen that does not exist. The frontier's own codegen column
(`gap-metric frontier-codegen-*`, board #1474) reads, over its **59** emitted
functions:

| bucket | reads |
|---|---:|
| `frontier-codegen-exact` | **10** |
| `frontier-codegen-wrong` | **1** |
| `frontier-codegen-refused` | **0** |
| **`frontier-codegen-reader`** | **48** (81 %) |

**Read the last row first.** 48 of 59 sit behind an IL-parser refusal, so no
codegen question was asked and none *can* be. `frontier-codegen-measured 1` is a
**lower bound of unknown tightness**, never a price. `0` refusals is an alarm that
did not fire, not "codegen is done."

### 2.2 The emit-model reach ceiling — 124 perfect, 110 measured

Board **#213** prices a *perfect* emit predicate as `reach-pool = B∧C ∖ A∧B∧C`
= **124** TUs of reachability (`gap-metric emit-predicate-worth 124`), and as
`frontier-pool = frontier-if-A ∖ FRONTIER` = **122** of frontier. The two are
different sets; their difference is exactly `projection-divergence` = **2**
(`src/system/decomp_pch.cpp`, `src/system/math/vec.cpp`).

**The measured ceiling, not the perfect one** — lane `w-bcgap`,
[`rungs/2026-08-08-w-bcgap.md`](rungs/2026-08-08-w-bcgap.md) §5, collected
2026-08-08 at tree `b027eaad`, **cited not re-measured here**:

| model | reach bought, `\|model ∩ reach-pool\|` | of |
|---|---:|---:|
| `ALIAS_IN` (the best measured predicate) | **110** | 124 |
| `ORACLE` | 110 | 124 |
| `JFP_ALIAS` / `JFP` | 98 | 124 |

**The best measured emit predicate is already worth 89 % of a perfect one.** The
remaining 14 is not a modelling gap worth another channel hunt (w-bcgap §5.5),
and the alias channel's 3.1× move in per-TU exact bought **zero** reach
(board **#1522**).

**And every one of the 110 is still gated on codegen.** Reach is not conversion.

> **⚠ This is the ceiling that supersedes "the 450 wall" as the emit-model
> number to quote.** See §2.3.

### 2.3 The 450-wall — RE-DERIVED, and it is a COUNT OF BLOCKED TUs, not a ceiling

`docs/STATUS.md`'s generated block carries the row
**"Emit-set MODEL ceiling (today / repaired / wall) | 338 today / 421 repaired /
450 wall"**, and `scripts/status.sh`'s registry labels all three with the word
*ceiling*. Re-derived at this tree, from the scan's own line:

```
emit-set MODEL ceiling: 338 of 871 TUs bind every emitted symbol today;
  421 would if `bind.rs` lost none;
  450 carry an emitted symbol with NO `.gl` body record and are a wall
      for any segment-driven model
  nesting invariant, must hold: today <= repaired (true), repaired + wall == graded (true)
```

**All three re-derive. `421 + 450 == 871` is asserted by the instrument itself**
(`crates/c2-harness/src/cli/gap.rs`, the `repaired + wall == graded` control) and
holds.

**Therefore 450 is the complement of 421 inside the graded population — the
number of TUs a segment-driven model can never reach — and the ceiling that
population implies is 421, not 450.** Quoting "the 450 wall" as a figure the
project might attain inverts its sign. The three numbers are one decomposition of
**factor B**:

| | TUs | meaning |
|---|---:|---|
| `today` = **factor B** | **338** | every emitted symbol already binds |
| `repaired − today` | **83** | reachable by a perfect `bind.rs` and nothing else |
| `repaired` | **421** | **the emit-model ceiling** — what a segment-driven model can ever bind |
| `wall` | **450** | ≥1 emitted symbol with **no `.gl` body record at all**; needs COMDAT *synthesis*, which no binding repair reaches |

Supporting counts, same scan: `emit-unbound-has-record` **4,632** (the
instrument-defect half) and `emit-unbound-no-record` **4,585** (the wall half),
summing to the **9,217** symbols FBM reports as `unbound`.

**Status: the 450 figure SURVIVES as a measurement and is SUPERSEDED as a
ceiling.** It is not deleted, it is renamed. The emit-model ceiling is **421**;
the emit-model *reach* ceiling on the payoff metric is §2.2's **110 of 124**.

### 2.4 The section ceiling — C is FINITE, and its head step has been declined twice

`factor-c` = **169**. C is the one factor with a closed form: the workload uses
**13** section names, the writer emits **10** (`PORT_WRITER_SECTIONS`), and the
scan prints the exact remaining ladder:

```
GREEDY LADDER — next section name to teach the writer (3 steps from 169 to 871):
  +.rdata$r     C = 590   (+421)
  +.text$yd     C = 804   (+214)
  +.xdata$x     C = 871   (+67)
```

`gap-metric ladder-head .rdata$r` · `ladder-head-c 590`.

**Three writer section names take C from 169 to 871.** That is the good news and
it is smaller than it looks:

* **`.rdata$r` is RTTI, not EH** (ROADMAP §10.20) — 24,163 content symbols, all
  `??_R1..R4`, zero `__ehfuncinfo$`. Phase 5 (EH) moves C by **zero**.
* **The head step has been declined by two independent lanes at two masters.**
  `w-rdata` priced the minimal `.rdata$r` obj at **seven independent refusals**
  (**HAND-COUNT**); `w-rtti` was briefed to ship it anyway, re-derived at
  `9827bcf`, and found **all seven still unpaid** — `factor-c` **169 before and
  after**, all 77 `gap-metric` lines byte-identical.
* **C is necessary and not sufficient.** `C = 871` converts **zero** TUs on its
  own. See §2.5.

### 2.5 THE BINDING CEILING — `|D ∨ E| = 13`, so today's non-codegen headroom is **2 TUs**

This is the number the rest of this page exists to reach, and it is the one that
was scattered.

`match = A ∧ B ∧ C ∧ (D ∨ E)`. `D∨E` is **the port's codegen class** — D
per-function, E the whole-TU registry — and it is **invariant** in A, in B and in
C: no emit-set model, no binding repair and no writer section name changes it,
by construction.

| | value | instrument |
|---|---:|---|
| `d-or-e` | **13** | `c2rs factors`, this tree |
| `match` | **11** | `gap-metric match` |
| **non-codegen headroom** | **13 − 11 = 2** | subtraction of two keys from one run |

**So: perfect factor A, perfect factor B, and C = 871 — all three, together,
today — move TU match from 11 to at most 13.** The two TUs are named by the
scan and are exactly the projection-divergence pair: `src/system/decomp_pch.cpp`
and `src/system/math/vec.cpp`. Both need factor **A** alone.

Board **#361** stated this shape at `|D∨E| = 10` with match 8 (lane `w-joint2`,
2026-08-05) and called it *"the project's entire non-codegen headroom is 2 TUs"*.
**It is still exactly 2, three days and 24 landed rungs later**, with both terms
moved by +3. That invariance is the finding: every non-codegen lever the project
has pulled moved `D∨E` and `match` together.

**To reach 871, `|D ∨ E|` must reach 871.** Everything else on this page is
necessary and jointly worth **+2**.

---

## 3. What actually stands between `B∧C = 151` and 871

`B∧C` is the near-term joint ceiling — perfect emit model, perfect binding, at
*today's* writer vocabulary. From §1.2's partition, the complements are exact:

| | TUs | share of 871 |
|---|---:|---|
| in `B∧C` | **151** | 17.3 % |
| fail **C** only (bind, wrong sections) | **187** | 21.5 % |
| fail **B** only (sections fine, symbols do not bind) | **18** | 2.1 % |
| **fail BOTH** | **515** | **59.1 %** |

So `871 − 151 = 720` TUs must be moved, and **515 of them need work on both
axes**. Restated per factor: **533 TUs fail B** (871 − 338) and **702 fail C**
(871 − 169).

**The two populations decompose differently, and that is the useful part:**

* **C — 702 blocked, closed-form, 3 steps, head step priced at 7 (HAND-COUNT)
  and declined twice.** C's blockers are *section names*, an enumerable
  13-element set, measured **0** occurrences of `#pragma init_seg` in the
  workload's 78,746 source files. The ladder in §2.4 is the whole list.
* **B — 533 blocked, and 450 of them (84 %) are a wall.** §2.3: only **83** of
  the 533 are reachable by repairing `bind.rs`; the other **450** carry an
  emitted symbol with no `.gl` body record and need COMDAT synthesis — a thing
  no phase in the plan builds.

### 3.1 Blocker families by mass — and the standing warning against ranking by them

The scan prints the emitted-code widening order: **648 keys summing 130,575
blocked emitted functions**. The head:

| key | blocked emitted fns | share |
|---|---:|---|
| `expr-op-0x27` | **22,373** | 17.1 % |

**Do not convert that ranking into a plan.** Two measured reasons, both from this
month:

1. **Mass order ≠ yield order.** Lane `w-mass`
   ([`rungs/2026-08-08-w-mass.md`](rungs/2026-08-08-w-mass.md)) ranked
   `emit_blockers` by mass, took the largest family that survives its own
   counterfactual, and **declined at 5,362** — *"the ranking column and the
   counterfactual column disagree about the order of the top three families"*
   (largest family's counterfactual terminals: **2**; third-largest: **5,362**).
2. **Ranking instruments measure themselves.** `expr-op-0x27` is the standing
   case: board **#150** closed it at **6 emitted functions converted** against a
   headline of 22,759, and `w-op27` re-measured it at **8** three days later
   (board **#1337**). A key's size is not its yield and never has been here.

**A mass ranking is a driver. It has never once been a forecast of conversions on
this project.**

---

## 4. Cost per converted TU

### 4.1 The numerator — 5 TUs, 4 mechanisms, all four dated from the rung record

TU match has moved **6 → 11** over the entire measured history of the metric. The
rung record dates every step and the sequence is **monotone** (no conversion has
ever been lost):

| rung | landed | TU match | mechanism |
|---|---|---:|---|
| [`w-r1c`](rungs/2026-08-04-w-r1c.md) | `4233939b`, 2026-08-04 | 6 → **8** | whole-TU `??__E` dyninit recognizer (factor **E**) |
| [`w-tu1`](rungs/2026-08-05-w-tu1.md) (W42/W43) | `dbef1913`, 2026-08-05 | 8 → **9** | first conversion from per-function codegen breadth |
| [`w-hash`](rungs/2026-08-05-w-hash.md) | `92aec237`, 2026-08-05 | 9 → **10** | first conversion needing a control-flow class |
| [`w-lineage`](rungs/2026-08-08-w-lineage.md) | `9b3b45e3`, 2026-08-08 | 10 → **11** | mixed-kind allocation, by refusing the disputed term |

**+5 TUs over 4 conversion events**, because `w-r1c`'s single mechanism converted
two structurally identical license TUs at once. **Count mechanisms, not TUs, when
sizing a channel.**

One near-miss for the record: ROADMAP §9.16-era prose and
[`rungs/2026-08-04-w-sect.md`](rungs/2026-08-04-w-sect.md) contain a *"TU match
8 → 6"* — that is a **counterfactual** (what a rejected rule would have cost), not
a regression. Checked, not assumed.

### 4.2 The denominator — stated, three ways, because the folklore stated none

The folklore figure is **"~5 TUs per ~161 lanes"**, and its one occurrence in the
tree is [`docs/whitebox/CAMPAIGN_2026-08-08.md`](whitebox/CAMPAIGN_2026-08-08.md)
line 22, where it appears **with no window and no definition of "lane"**. Every
denominator below is a `git` count at tree `b234d826`, reproducible from §9.

| window | landed rungs | merge commits | TUs gained | **rungs / TU** | merges / TU |
|---|---:|---:|---:|---:|---:|
| **C — the conversion window** `4233939b^..9b3b45e3` (first conversion through last) | **86** | 140 | **+5** | **17.2** | 28.0 |
| **D — first conversion to this base** `4233939b^..HEAD` | **110** | 165 | **+5** | **22.0** | 33.0 |
| **E — since the last conversion** `9b3b45e3..HEAD` | **24** | 25 | **0** | **∞ so far** | ∞ so far |
| **P — the whole rung record** (project start → this base) | **148** | 281 | 11 (6 unattributed) | 13.5 | 25.5 |

*"Landed rung"* = a non-`_`-prefixed file in `docs/rungs/`, i.e. exactly one row
of the generated `rungs/INDEX.md`. That is the only lane-shaped unit the
repository maintains.

### 4.3 The folklore reconstructs — and its unit is wrong

`161` is **not** any count in the tree at the commit where it was written
(`c34c388c`: 142 landed rungs, 241 rung files, 275 merges). It reconstructs as
one thing and one thing only:

> **merge commits in `4233939b..c34c388c` = 159.**

Off by 2 from "~161", against a "~". So the folklore's denominator was
**merge commits since the first conversion**, presented as *lanes*. Over that
same range the **lane** count is **103**, giving **20.6 lanes/TU** rather than
the implied **31.8**. The folklore is **1.5× pessimistic, and its error is a unit
error** — a merge is not a lane here (docs merges, gate re-runs and coordinator
merges all count).

**This is a reconstruction, not a citation.** No document states the denominator;
the arithmetic identifies it and the residual 2 is not explained.

### 4.4 The honest headline

> **Between the first conversion and the last, 86 landed rungs bought 5 TUs:
> ~17 rungs per converted TU.** Since the last conversion, **24 more rungs have
> bought 0**, so the *marginal* cost is not the average cost and is currently
> unbounded above.

**And the marginal rate is the one to plan against.** The three most recent
codegen-facing lanes all declined at a measured price rather than converting:
`w-rtti` (7 refusals unpaid, `factor-c` +0), `w-heap` (declined at 5 over a
27-cell frozen grid), `w-mass` (declined at 5,362 emitted functions). Board
**#269**'s standing clause — *a frontier TU at ≥ 4 independent refusals is not a
target* — **fires on every frontier TU that has been priced** (**HAND-COUNT**,
and see §5 on how to read it).

### 4.5 What a rung buys, in the unit the goal is written in

TU match is a conjunction, so a rate in TUs is lumpy. The continuous companion is
the **emitted census** — `progress-emitted-in-class 39185` of
`progress-emitted-total 178977` (21.89 %).

| | value |
|---|---:|
| emitted functions in class, 2026-08-04 (`33cbdbe`, STATUS §history) | 38,458 |
| emitted functions in class, this tree | **39,185** |
| gained over window C's 86 rungs | **+727** |
| still needed for `D` on all 871 (`178,977 − 39,185`) | **139,792** |
| **share of the remaining distance bought by 86 rungs** | **0.52 %** |

Straight-lining that rate gives **~16,500 more rungs**. **That number is
arithmetic, not a forecast, and this page says so in the same breath:**

* the blocked mass is heavy-tailed — single rungs in 2026-07-31 moved the
  per-function census by **+58,135** and **+36,684**, so one recognizer could in
  principle move six figures;
* and trap 8 / board **#250** is the counterweight: a **+7.4 pp** micro-F1 move
  closed **0** TUs, because a whole-obj verdict is a conjunction and an average
  is not.

**Both corrections point in opposite directions and neither is quantified.** The
defensible statement is the order of magnitude of the *current channel*: at the
rate the last 86 rungs actually achieved, the remaining emitted-function distance
is **four orders of magnitude larger than what those rungs bought**.

---

## 5. Calibration — what board #770's streak implies for every forward number here

Board **#770** tracks the project's record of pre-registered estimates against
measured outcomes. **The tally is maintained by hand in each lane's own rung and
the rungs disagree**, which is itself worth knowing before quoting it:

| source | stated tally |
|---|---|
| [`whitebox/CAMPAIGN_2026-08-08.md`](whitebox/CAMPAIGN_2026-08-08.md) §Why | ~10 optimistic / 2 pessimistic / 1 hit |
| board **#1459** (`w-5c2`) | ten optimistic, one pessimistic, one hit, one optimistic |
| [`rungs/2026-08-08-wb-memcpy.md`](rungs/2026-08-08-wb-memcpy.md) §2 | ~10 optimistic / 2 pessimistic / **2 hits** |
| [`rungs/2026-08-08-w-bcgap.md`](rungs/2026-08-08-w-bcgap.md) §7 | +1 hit, 0 misses |

**No instrument produces this tally.** It is a HAND-COUNT of prereg scorecards.
What every source agrees on is the shape, and the shape is what calibrates:

1. **Optimism dominates roughly 5:1.** Ten-or-eleven optimistic misses against
   one-or-two pessimistic.
2. **The misses are specifically on FORWARD COST** — frontier depth, refusal
   counts, rung counts. Not on measurement, where the same preregs routinely hit
   known-answer controls to the digit.
3. **Both pessimistic misses were "I thought this was structurally cheap"
   inverted** — board **#1386** (`w-4c`, registered +11, measured +23) and
   **#1420** (`w-clear`, *"the bracket was not the CFG class, it was the guard
   itself"*). Even the exceptions are about mis-modelled structure.
4. **The most recent instructive miss is #1459's**: a number registered from a
   population of one witness and generalised to 810 TUs, 807 of which had a
   different shape. **A registered number is a statement about the population
   that suggested it.**

> ### Therefore: **read every forward cost figure on this page as a LOWER BOUND.**
> §2.4's seven refusals, §4.4's ≥ 4-refusal clause, §4.5's 16,500 rungs, §6's
> phase list — each is what could be enumerated by someone who had already
> decided the work was tractable. This page publishes **no** point estimate of
> when TU match reaches 871, and that refusal is registered rather than
> accidental (`rungs/_2026-08-08-w-ceiling-prereg.md` P15).

---

## 6. What 871 would require under the current model

Stated plainly, from §2.5: **`|D ∨ E|` must go from 13 to 871.** That is 858 TUs
whose *every* emitted COMDAT the port lowers byte-exactly. The supporting
distance, from this scan:

| | value | instrument |
|---|---:|---|
| emitted functions the port must additionally accept | **139,792** | `178,977 − 39,185` |
| of those, blocked at the IL reader | **130,575** | emitted-code widening order total |
| of those, unbound (no census row claims the symbol) | **9,217** | FBM partition |
| TUs within ≤ 10 blocked emitted functions of matching | **82** | `TU distance to matching (blocked EMITTED functions)` |
| TUs within ≤ 100 | **407** | same |
| TUs within ≤ 1000 | **858** | same |
| TUs 100 % byte-exact per emitted function | **7** of 865 | per-TU FBM |
| TUs ≥ 50 % byte-exact per emitted function | **15** of 865 | per-TU FBM |

### 6.1 The phases that would have to exist, and do not

This list is ROADMAP **§10.24**'s re-ordering (2026-08-08), which supersedes
§10.9's, plus the two items §10.24 books as debt. **Each entry is a phase, not a
rung.**

| # | phase | what the instruments say about its size |
|---|---|---|
| 1 | **Emitter CFG classes** — `cflow-loop`, `cflow-if-n`, `cflow-if-2` | covers **33 of the frontier's 48** reader-blocked functions (`wb-reader`). Today `gap-metric cfg-reach-shipped` is **2** of `cfg-reach-top` **16**: 14 of 16 frontier TUs are held by CFG class alone. What shipped for `cflow-loop` is *"a twenty-word transcription of one function class at `/O1`"* — `PORT_CFG_CLASSES` deliberately does not list it (board **#761**) |
| 2 | **An inliner** | `keygen_xbox.cpp` is the one frontier TU whose gap is neither reader nor emit-set. `wb-frame` retracted board #1477 and found the real `?supershuffle` gap is **14 words of uninlined `?shuffle2`** (**HAND-COUNT**, disassembly-derived). `INLINE_PREDICATE.md`'s mechanism I holds at **0.9716** on a 100-TU hold-out and is **not shipped** |
| 3 | **`memset` / selector lowering in `c2-core`** | `w-mass`'s priced decline: **5,021** emitted functions terminal on `memset`. Convert-rate per TU **unknown** |
| 4 | **Exception handling** | ROADMAP §10.20: EH blocks by factor **D** over **740** objs. Board **#283**: the `/EHsc` axis is graded entirely through implicit destructor unwind — **`try`/`throw` have ZERO cases** in the generated corpus |
| 5 | **Weak externals at scale** | `alias-weak-needed-tus` **675** of 871 carry a COFF weak-external symbol record the port's writer cannot emit. **No factor in §10.19 represents it.** `alias-weak-needed-in-b-and-c` **0** and `-in-frontier` **0**, so it costs the payoff metric **nothing today** and everything at 871 |
| 6 | **COMDAT synthesis** | §2.3's **450**: TUs carrying an emitted symbol with no `.gl` body record. No binding repair reaches them and no phase in the plan builds this |
| 7 | **Register allocation and scheduling across a back edge** | board **#770** (**HAND-COUNT**): `Sort.cpp` re-derived at **eleven** refusals, of which loop rotation, memory-reference peeling and cross-back-edge allocation *"are properties of the loop's schedule of values, not of its instruction vocabulary, and no recognizer reading this body alone can derive them"* |

**Five of the seven have never had a rung.** Items 2, 4, 5, 6 and 7 exist as
measurements and declines only.

### 6.2 The one-sentence version

> **TU match 871 requires the port's codegen class to reach 871 (§2.5). The
> emit-set model, the binding repair and the section vocabulary are jointly worth
> +2 TUs today (§2.5) and are prerequisites for the other 858 rather than
> contributors to them. The measured cost of the last five conversions was ~17
> landed rungs each (§4.2), the last 24 rungs bought none (§4.2), and the streak
> calibration says to read every one of those figures as a lower bound (§5).**

Whether that is a reason to continue, to re-scope the goal, or to change channel
is **not this page's call**. What the page asserts is that the arithmetic is now
computable, and that anyone making the call should make it against §2.5 rather
than against `factor-c`, `B∧C`, the census, or the 450.

---

## 7. What this page does NOT claim

Named so absence never reads as success (trap 5).

1. **That any ceiling here is achievable.** Every one is a bound, and the bound
   being 871 does not make 871 reachable.
2. **That the cost-per-TU figure transfers forward.** It is a backward-looking
   average over one window with the window stated; §4.4 and §5 both say to use
   the marginal rate instead.
3. **That the frontier's codegen prices are re-derivable by a scan.** All but one
   are HAND-COUNTs (#1476); `frontier-codegen-measured` is **1**, of 59.
4. **That `|D∨E| = 13` is stable.** It has moved 10 → 13 in three days. Re-read
   `c2rs factors` before quoting it.
5. **That the 878 workload generalizes.** Coverage-bounded differential testing;
   one game, one XDK, one flag set (`/O1 /EHsc /GR`).
6. **Anything about ORDER.** A right emit set in the wrong order is still a
   mismatch, and no set on this page says anything about order (w-bcgap §9.5).

---

## 8. Traps specific to reading THIS page

The general traps are `STATUS.md`'s. These four are the ones that bite on a
ceiling document.

### 8.1 `mismatch 0` is not evidence of correctness

**860 of 878 TUs refuse before the emitter is consulted** (`gap-metric vocab-gap
860`), so the scan *cannot see* a codegen or binding defect in them. `mismatch 0`
means "nothing the scan could grade came out wrong", over a population the scan
mostly cannot grade. The alarm has fired **five** times, and **four of the five
were found by lanes building probe grids for unrelated rungs** — so the rate says
more about how many grids were built than about how many defects remain.
`codegen-gap 0` is the same shape one level in: it is 0 because nothing reaches
codegen, not because codegen is right.

**On this page specifically:** none of §1's factor counts is graded by the
oracle. A, B and C are properties of the *reference obj* compared against the
port's declared capability; only `match` is a byte-exact verdict.

### 8.2 The FBM duplicate-credit caveat — a payoff figure that counts template instantiations

`FBM = 0.20234` and `fnbyte-exact 36,213` are **per-emitted-function** credits,
and function-level credit on this workload is dominated by **template
instantiation duplicates**:

* `w-empty`'s mechanism E closed **1,373** functions — **all 1,373 are one
  STLport template**, `??1?$_STLP_alloc_proxy`, **545 instantiations**
  (board **#925**).
* `w-fix`'s fixpoint closed **143** more — **all 143 are
  `??1?$_Rb_tree_base@…`**, the tier directly above (board **#952**).

**1,516 credited functions are two templates.** A reader who converts an FBM
delta into "progress toward 871" has counted one mechanism 545 times. **TU match
did not move for either lane** (10 → 10 both times), which is the check.

Two further reasons `fnbyte-exact` is not a clean credit, both on every scan:
**861** of the exact bodies relocate against a symbol c2 does not name
(board **#986** — a `/Gy` branch word cannot carry its callee, so a byte test
scores the word equal), and `fnbyte-differs` is **2,111** with `reloc-differs`
**861** beside it. **Quote FBM with `fnbyte-differs`, from a scan.**

### 8.3 HAND-COUNT vs INSTRUMENT (#1476) — and where each lives

| tag | means | on this page |
|---|---|---|
| **INSTRUMENT** | a key a scan prints; names the key, so a successor re-reads it at their own tree | §1, §2.1, §2.3, §2.4, §2.5, §3, §4.5, §6 |
| **HAND-COUNT** | a person enumerated refusals, clauses or bytes by reading the obj beside the IL. Reproducible only by repeating the reading; **does not move when the tree does** and can go stale silently | §2.4's seven · §4.4's ≥ 4 clause · §5's whole tally · §6.1 items 2 and 7 |

**There is exactly one INSTRUMENT codegen number on the frontier and it is `1`**
(`frontier-codegen-wrong`, board #1474), against **48** functions for which only
a hand-count is possible. That is not an argument for distrusting the
hand-counts — they are the only numbers that exist for the 48. It is the reason
for the rule: **never quote a hand-count and a scan reading in the same sum.**

`docs/ROADMAP.md` §10.25 tags that file's own codegen hand-counts.

### 8.4 Zero-by-construction columns

Three zeros on this page are **inert**, not achievements:

| reads | why it is 0 |
|---|---|
| `codegen-gap 0` | every non-matching capturable TU fails at the IL decoder first, so the codegen class is never consulted |
| `frontier-codegen-refused 0` | three of `fnbytes::Decline`'s four stages are zero **by construction**; acceptance lives in the IL parser (board **#1475**) |
| `alias-weak-needed-in-frontier 0` | the frontier is 16 TUs and none of them happens to need a weak external. It says nothing about the other 675 |

The project's canonical instance is board **#1524**: a `!is_match()` guard in
`FRONTIER` survived a must-fail mutation because **the clause has zero
witnesses**. And trap 0 one level up: an accounting identity was `0` for the
entire life of a file because the population it ran over was too small to contain
the shape.

---

## 9. Reproducing every number on this page

```sh
# 1. the scan and the per-TU factor listing  (needs the toolchain + the dc3 tree)
C2RS_DC3=<dc3-tree> ./target/release/c2rs gap \
    --list work/dc3-workload/files.txt --flags-file work/dc3-workload/flags.txt \
    --cwd <dc3-tree> --jobs 16 --factors-tsv factors.tsv > gap.log

# 2. §1.2's partition — one awk over the listing, no toolchain
awk -F'\t' '!/^#/{n[$8]++} END{for(k in n) printf "%5d  %s\n", n[k], k}' factors.tsv | sort -rn

# 3. §1.2's margins + §2.5's |D or E|, with the known-answer control (no toolchain)
./target/release/c2rs factors --tsv factors.tsv --check-metrics gap.log
#    must print `13 OK, 0 DISAGREE, 0 ABSENT of 13 keys`

# 4. §2.3's three-way decomposition and its asserted invariant
grep -A1 'emit-set MODEL ceiling' gap.log

# 5. §2.4's section ladder, §2.1's codegen column, §3.1's blocker head, §6's distances
grep -E 'GREEDY LADDER|frontier-codegen-|widening order OVER EMITTED|TU distance' -A5 gap.log

# 6. §4.2's denominators — git only, no toolchain
rungs() { git log --diff-filter=A --name-only --format='' "$1" -- docs/rungs/ \
          | grep -v '/_' | grep -vE 'INDEX|README' | sort -u | wc -l; }
rungs 4233939b^..9b3b45e3      # window C: 86
rungs 4233939b^..HEAD          # window D: 110
rungs 9b3b45e3..HEAD           # window E: 24
git rev-list --merges --count 4233939b..c34c388c   # §4.3's reconstruction: 159
```

---

## 10. Where the other numbers live

| question | doc |
|---|---|
| where the project is, and what each number is *for* | [`STATUS.md`](STATUS.md) |
| the numbered items this page cites (`#213`, `#361`, `#770`, `#1476`, `#1520`…) | [`BOARD.md`](BOARD.md) |
| the factorization's derivation and the phase plan | [`ROADMAP.md`](ROADMAP.md) §10.19, §10.21, §10.24; **§10.25** for this file's own hand-count tags |
| the set algebra behind §2.2 and §2.5 | [`rungs/2026-08-08-w-bcgap.md`](rungs/2026-08-08-w-bcgap.md); `c2rs factors` |
| what the CFG step must emit | [`CFG_SHAPE.md`](CFG_SHAPE.md) |
| what is inside the differing bodies | [`DIFF_STRUCTURE.md`](DIFF_STRUCTURE.md) |
| the emit-set model's own plan and its held-out validation | [`PHASE7_PLAN.md`](PHASE7_PLAN.md), [`PHASE7_VALIDATION.md`](PHASE7_VALIDATION.md) |

---

## 10. The 2026-08-09 addendum — the distance is measured from three directions now

`CEILING.md` was written on 2026-08-08 to equip the re-scope decision. Three
lanes since have measured the same distance independently, and they agree.

**1. The arithmetic is unchanged and was re-derived live** — first at tree
`5ad60e9e` (`match` 18), then again at `bf7ef653` after `w-blockir`'s conversion
(`match` **19**, `frontier` **8**, `factor-d` **19**, `|D∨E|` **21**,
`A∧B∧C∧D` 17, `frontier-if-a` **130**). `factor-a` **28** · `factor-b` **338** ·
`factor-c` **169** · `b-and-c` **151** · `a-and-b-and-c` **27** are unmoved
across both. Perfect A + perfect B + `C = 871`, together, move match
**19 → 21**.

**The non-codegen headroom has been exactly 2 at every reading**: at match 11 on
2026-08-08, at match 18, and at match 19 — unmoved across eight conversions and
sixteen lanes, with both terms rising together each time. That invariance is the
finding, and it is why **to reach 871, `|D∨E|` must reach 871**: every non-codegen
lever the project has pulled moved `D∨E` and `match` by the same amount, one TU
at a time.

**2. A census gain is not a goal gain, and now that is measured.** `w-fltret`
(#2080–#2087) admitted **+444** emitted functions at 99.3 % of its predicted
population and moved **`fnbyte-exact` by exactly zero** (36,228 → 36,228),
because c2 inlines the callees. `w-inlfence` (#2220–#2227) then fenced the
class and showed the per-function census is **fail-open on 845 of 871 TUs**.
**Only `fnbyte-exact` and per-TU byte-exactness map to the goal.**

**3. There is no multi-TU cluster.** `w-band` (#2240–#2247) priced the two
distance bands `STATUS.md` publishes. The published distance is
three-quarters **not** blocked functions; the `≤1` band is **7 live TUs**, not
21 (13 already match); the `≤10` band is **69**. Of those, **56 carry ≥2
distinct blocking keys**, **8 of the 11 single-key candidates are `-BC--`** and
so fail factor A regardless of codegen, and **exactly one key owns two TUs** —
which dies on inspection, needing block IR rather than reader admission. The
only cleanly-one-TU rung in either band is `src/Main.cpp`.

**What this means for the goal, stated as arithmetic and not as an argument.**
Every remaining TU is an independent unit of function-byte codegen. The
project's demonstrated rate is ~1 TU per lane at its best (2026-08-08: seven
conversions), and the cheap transcription pool is exhausted — the frontier is
**8** and its remaining rows are EH, block IR, and two TUs above the
one-block-plan licence. `WB-I` (§10.27) establishes that a *general* lowering is
derivable (~60 rules), which changes the price of every future class — but its
own predicted first-scan reach is **0**, because the reader gates most of the
frontier's functions before any emitter question is asked. §10.30 then priced
the reader itself and found **no reader rung converts a TU either**; the
frontier's blocked column is **41 of 51**, and its 8 departures were 8
one-function transcriptions with measured reach 1 apiece.

**The rate comparison that settles the shape of the problem** (§10.30):
**seven one-function transcriptions bought +7 `fnbyte-exact` and +7 TU
conversions; one 444-wide admission bought +0 and +0.** Breadth does not convert;
depth converts one at a time. §10.31's conversion is the first driven by a
*derived* generator reading rather than a transcription, and it still converted
exactly one TU — but it cost one lane, and the machinery it used is now reusable,
which is the only thing on the board that could change the rate.

**The re-scope decision remains the user's.** This page still does not argue
for or against continuing. What it now says that it could not say yesterday is
that the distance has been measured from three independent directions and none
of them found a multiplier.

---

## 11. NON-CODEGEN LAST BLOCKERS — the class, its detector, and the checklist

Added 2026-08-09 by lane `w-nc` (#2380–#2399). **Read §11.4 before you conclude
that a body is a codegen problem.** Five conversion lanes have now found that
their *last* blocker was not codegen, each at the end of a lane that had already
paid for codegen — and the fifth added a family this list did not have:

| lane | what the rung says | family |
|---|---|---|
| `w-bdnz` #1980 | *"The unsigned half of the class is byte-exact against real `c2` and was blocked by the reader, not by codegen."* … *"So the emitter was never the question."* — `.sy`'s predicate is `kind == 1 && size == 4 && tid == 0x74`, plain `int` only (**#764**) | NC-3 |
| `w-blockir` #2301 | *"the scan read `fnbyte-exact 4 · fnbyte-differs 0` — **every body byte-exact** — and the whole obj graded `mismatch`, because `IlFunction::touches_floating_point` had no arm for this class and the obj came out **one symbol short**… the thing standing between a byte-exact body and a matching TU was a **TU-level fact**, not an instruction."* | NC-1 |
| `w-main` #2260 | *"`WB_EH_FINDINGS.md` §6 files this as R1, `param-width-undetermined:mid`, `c2-il` formals header. **The key is right and the location is wrong.**"* — the refusal is `func::sy::ex_exit_label` wanting a `3A` byte the `.ex` does not contain | NC-4 |
| `w-front5` #2621 | *"`src/Main.cpp`'s `.gl` carries exactly ONE framed defined record, at body-start 2713, which is exactly its single `.ex` segment's start — the binding is arithmetically perfect. The name is `main`, four bytes, and `INLINE_NAME_MAX` is 8."* So `Bindings::per_record` returns `None` and **`w-main`'s own thirteen mechanisms price the second layer of a two-layer chain** | NC-4 |
| `w-xtea3` #2691 | *"With every body emitting the reference obj's bytes the TU refused at `comdat::fenced_inlined_callee`… the oracle disagrees — `EncryptXTEA.obj` carries the `bl ?Encipher` — and one clause turns `codegen-gap` into `match`."*  `fnbyte-exact 5 of 5`, and the blocker is a FENCE rather than an obligation | **NC-5** |

### 11.1 The class

> **A NON-CODEGEN LAST BLOCKER is an obligation that stands between a TU and a
> byte-exact obj and that is not a question about any function's instruction
> bytes.** Four families — and, since 2026-08-09, a **fifth that is not an
> obligation at all** (NC-5, a refusal the port makes on purpose):

* **NC-1 — a whole-obj SYMBOL obligation.** The obj carries a symbol no function
  body contains, minted by a TU-level predicate. Every one the port's writer can
  owe, enumerated from `crates/c2-core/src/coff/writer.rs` and `function.rs`:

  1. **`_fltused`** — one per TU, emitted immediately after the FIRST float
     function's complete symbol group. Producer
     `c2_il::IlFunction::touches_floating_point`. **This is the one `w-blockir`
     paid.**
  2. **`__real@…` pooled FP constants** — one `.rdata` section symbol + aux +
     one external per *distinct* constant, in first-reference order, deduped
     **across functions**. A per-function byte test sees each body and not the
     TU-wide pool.
  3. **Undefined externals, deduped TU-wide, in REVERSE first-reference order**
     over `callees ∪ data names` (`Function::introduced_externals`). Both the
     dedup and the order are whole-obj facts; emitting one per call site instead
     resolves every relocation and is still the wrong obj.
  4. **`__savegprlr_N` / `__restgprlr_N`** (`Function::helper_externals`) —
     minted from the frame's `saved_gprs`, never named in the IL, and placed
     *after* the `.pdata` group.
  5. **The compiler-label counter** — the `$M`/`$M`/`$T` numbering, seeded from
     `.gl` and advanced per function by a measured stride, **plus one slot
     charged once per TU for the first FP-touching function** (`_fltused`'s).
     A wrong count here is a wrong obj with every body byte-exact.
  6. **`@comp.id` and the 13 fixed shell slots**, and the string table that
     follows from every name above.
  7. **A MINTED INTRINSIC EXTERNAL, and its own once-per-TU label slot.**
     *Added 2026-08-09 by lane `w-ifn` (#2354, #2353), the day after this
     enumeration was written, and it is trap 0's shape applied to the
     enumeration itself: the list was complete over the classes the port then
     emitted.* `memcpy` arrives in the `.ex` as **intrinsic selector 172 on a
     `40` token** and has **no `.gl` record at all** — so it is not a callee in
     the sense item 3 means, and `IlFunction::callees` must NOT name it. It owes
     two things:
     * **placement** — after the FIRST user's `$T` label, on
       `Function::helper_externals` (item 4's slot, reached from an intrinsic
       instead of from the frame), and **not** in the callee region between the
       two `$M`s, where an IL-named callee goes. `work/w-ifn/probe/lab_z.cpp`
       is one obj showing both placements;

       > **⚠ 2026-08-09 — THE PLACEMENT IS A FACT ABOUT THE USER'S FRAME CLASS,
       > NOT ABOUT THE NAME, and the sentence above is true only of a FRAMED
       > user.** Every witness behind it is one. Lane `w-xtea2` (#2663) shipped
       > a **LEAF** user — `?SetKey@XTEABlockEncrypter`, whose whole body is
       > `addi r3,r3,16 · li r5,16 · b memcpy` — which has no `$T` at all, and
       > both objs put `memcpy` in the **callee region**: `work/w-xtea2/ref/
       > xtea.dump` reads `[16] ?SetKey… · [17] memcpy · [18] .text`, and
       > `work/w-xtea2/probe/mcpytail.obj` the same one function over, with its
       > three later users minting **no second symbol**. So the crate carries
       > two placements for one name — `guard_ret_chain` fills
       > `helper_externals`, `memcpy_tail` leaves it empty and lets
       > `introduced_externals` place it — and a class that inherited this
       > paragraph unqualified would resolve every relocation and move two
       > symbol indices.
     * **one compiler-label slot, once per TU**, before the first minting
       function's own triple — item 5's `_fltused` rule, one external over.
       Measured `[framed, sub]` stride **6**, `[framed, sub1, sub2, framed]`
       strides **6, 5, 5**.

     **Both were live wrong emits when `w-ifn`'s class first shipped**, with
     every body byte-exact, which is exactly what T1 below detects. Any future
     class that mints an external the IL does not name inherits both.
* **NC-2 — a SECTION obligation.** The section set (factor **C**: 10 writer
  names of 13 workload names), the section *count* at file offset 2, the section
  *order* (Rule S1's three `.bss` slots, **#1148**/**#1179**), and `.pdata`'s
  existence at all — decided by "does any function in this TU have a frame".
* **NC-3 — a TYPE-LIST gap.** A reader refuses on **list membership**, not on a
  construct it cannot represent: `.sy` admitting `tid == 0x74` and not `0x75`
  (#764) is the recorded instance. Detector: the refusal key ends in a hex
  **type tag** (`expr-load-type-8881`, `assign-store-type-8643`).
* **NC-5 — a whole-TU CONSERVATISM GATE.** *Added 2026-08-09 by lane
  `w-xtea3` (#2691), and it is the first family here that is not an
  **obligation**.* NC-1…NC-4 are all things the obj owes and the port does not
  supply: a symbol, a section, a type-list row, a clause in the right layer.
  This one is a deliberate refusal the port makes **because it cannot prove it
  may not** — right in general, wrong on one TU, and invisible to every
  per-function instrument.

  `src/system/utl/EncryptXTEA.cpp` reached **`fnbyte-exact 5 of 5`** — every
  body emitting the reference obj's own bytes — and graded `codegen-gap`,
  because `c2_core::comdat::fenced_inlined_callee` saw a call to a callee this
  TU defines whose lowered body is **116** bytes against `INLINE_DECLINE_BYTES`'
  **128**, and the fence's rule is *"the port cannot prove c2 KEPT this call"*.
  The oracle disagrees: the reference obj carries the `bl`.

  **The detector is T1**, unchanged — `fnbyte-exact == fnbyte-denominator ∧
  class != match` fires on it exactly as it does on NC-1. What is different is
  the **repair**: there is no missing symbol to mint. The port already knows
  everything it needs; what it lacks is a licence to keep its own call, and the
  licence was sitting unadopted in `docs/whitebox/WB_INLINE_FINDINGS.md` §7's
  MAY table (*"a loop-bodied callee > 80 bytes ⇒ never inlined at `/O1`"*, F9 +
  the anchor, 62 cells, port-side use stated as *"the safe decline side"*).

  > **Before writing an emitter arm for a TU that T1 fires on, read the
  > FENCES the port applies to it, not only the obligations it owes.** Today
  > that is `comdat::fenced_inlined_callee`, `elide`'s mechanism E and
  > `splice`'s S7 — three places where the port refuses a body it can lower.

* **NC-4 — a MISLOCATED reader clause.** The published refusal names a layer
  that is not the one that fails. `w-main`'s R1 is the hand-found instance;
  board **#1416** is the population version — *"`expr-cmp-eq` / `expr-cmp-ne` IS
  A FALL-THROUGH KEY ON THIS FRONTIER — it is the reported first blocker of 6 of
  the 7 blocked EMITTED functions in the four READER-CLEAR TUs and **it names
  none of their refusals**"* — **diagnosed and not repaired.**

  **A THIRD instance, and it is a whole PRICE rather than one key.** *Added
  2026-08-09 by lane `w-mmioclose` (#2405, #2406).* `w-ifn` priced
  `src/xdk/nuispeech/mmio.cpp` at six codegen mechanisms, all of them
  `mmioClose`'s, and closed with *"`mmioClose`'s 124 bytes are the entire
  remaining distance"*. The TU's own scan row reads **`1 .gl names`** against
  **11** `.ex` segments: ten of its functions are `extern "C"` and undecorated,
  `c2_il::mangled_names` is `looks_mangled` = `contains("@@")`, and
  `Bindings::per_record` binds nothing unless the records are 1:1 with the
  segments. **So `IlBundle::functions()` returns `None` before any body is
  looked at, and paying all six converts nothing.**

  The mislocation has a *mechanism* here and it belongs on this list beside the
  key-level one: **the instrument that said the distance was bytes is keyed on a
  different binding from the gate that decides conversion.**
  `frontier-bytefrac-*` and every `fnbyte-*` row are keyed on
  `FnCensus::emit_name`; TU `match` is decided by `Bindings::per_record`; board
  **#918** measured those two disagreeing on **74,955** workload rows. A
  per-function instrument can go 16.8 % → 67.4 % on a TU the gate has never
  bound. **Checklist item 8 below.**

### 11.2 The detector

Three tests, all over the **emitted** population, which `w-readpx` (#2280)
established is the only discriminating one — blocked rows are `fnbyte-refused`
by construction.

| test | catches | how |
|---|---|---|
| **T1 ALL-EXACT-NO-MATCH** | NC-1, NC-2 | from a `c2rs gap --jsonl` scan: `fnbyte-denominator > 0` ∧ `fnbyte-exact == fnbyte-denominator` ∧ `class != match`. This is `w-blockir`'s shape one day before it converted. `work/w-nc/sweep.py`. **`w-ifn` fired it a fourth time the same day, on a FIXTURE rather than a workload TU**: `fnbyte-exact 2 · fnbyte-differs 0` with the whole obj `mismatch`, over the two obligations item 7 adds — so the test works at fixture scale, which is where a conversion lane meets it first |
| **T1b ZERO-BYTE** | NC-1, NC-2 | `fnbyte-denominator == 0` ∧ `class != match` — the reference obj has no code at all, so the *entire* remaining distance is a whole-obj obligation |
| **T2 emitted-side refusal keys** | NC-3, NC-4 | the census verdict key of every `fnbyte-refused` **emitted** function. `work/w-nc/keys.py` + the reverted `C2RS_NC_KEYS` scratch. The published `fn_blockers` is over all 2.4 M bodies and is fail-open on 845 of 871 TUs (#2220); this one sums to `fnbyte-refused` |
| **T3 the type-tag regex** | NC-3 | `(type\|target)-[0-9A-F]{4}` over T2's keys |

**T2 is itself subject to NC-4** and the detector must say so: #1416's
fall-through means a reported key can name none of the refusal. Publish the
fall-through family's size beside any ranking taken off T2 — it is **9,095 of
114,687 emitted refusals (7.93 %)**.

### 11.3 What the sweep found (2026-08-09, dc3 `eb40a361`)

* **T1 = 1**: `src/system/math/vec.cpp` — `??0Vector3@@QAA@MMM@Z` and
  `??0Vector4@@QAA@MMMM@Z`, **both byte-exact**, factors `-BCD-`.
* **T1b = 1**: `src/system/decomp_pch.cpp` — its whole obj is 933 bytes, the
  four-section shell plus one 4-byte `.rdata` COMDAT (`ff ff ff ff`) carrying
  one external, `?npos@?$basic_string@…`. **1,312 `.ex` bodies and zero emitted
  functions.**
* So the **byte-distance-zero population is 2**, and it is exactly the two TUs
  §2.5 and board **#213** already name as *"inside `B∧C`, failing A, already
  accepted by the port"*. What is new is that it is now **byte-verified**: a
  perfect factor A converts these two with **zero codegen**, and that had been a
  reachability claim with no bytes behind it.
* **The frontier 8 carry ZERO all-exact TUs and 8 of 8 carry ≥1
  `fnbyte-refused`.** Its codegen column reads `denominator 47 · exact 10 ·
  wrong 0 · refused 0 · reader 37` — **quote it from a scan; §2.1's copy is a
  stale 59/10/1/0/48.**
* Every one of the 25 declines in the `1–2 functions short` band is
  `Decline::Parse`. **The reader, not the emitter, is the whole band.**

### 11.4 THE CHECKLIST — before you call a body a codegen problem

1. **Ask the BYTE judge, not the census.** `fnbyte-exact` / `differs` /
   `refused` per function, from a scan. The per-function census is fail-open on
   845 of 871 TUs (#2220) and a census gain is not a goal gain (§10.2).
2. **If every body is already `fnbyte-exact`, the blocker is NOT codegen.** Run
   T1. Then walk NC-1's six-item list and NC-2's four before you write an
   emitter arm. `w-blockir` paid `_fltused` in **one line** at the end of a lane
   that had already built a loop.
3. **Read the reference obj's SYMBOL TABLE, not just its `.text`.**
   `scripts/gt_dump.py <obj>` prints it. A symbol with no body — `_fltused`,
   `__real@…`, `__savegprlr_N`, a `$M`/`$T` label — is an obligation no
   per-function byte test can see.
4. **Check whether the refusal is LIST MEMBERSHIP.** A key ending in a hex type
   tag is a positive-list question (#764), not a construct. `.sy` has four
   positive lists; `read_record`'s own 21-cell table carried the `unsigned` row
   **unconsumed** from the lane that measured it until `w-bdnz` used it.
5. **Do not trust the reported key's LAYER.** #1416: `expr-cmp-eq`/`-ne` names
   none of the refusal on this frontier, and #420 measured the whole relational
   family absorbing into branch keys under a sink. Confirm the key against the
   body — `w-nc` found `void WordWrap_SetOption(unsigned o){ g_uOption = o; }`,
   **twelve bytes of PowerPC with no control flow whatsoever**, reported blocked
   at **`expr-jump`**.
6. **Check factor A before pricing any reader or emitter work.** 14 of the 17
   TUs within 2 functions of all-exact fail A, so closing their reader gap
   converts **nothing**, and the 3 that pass A are already in the frontier. `A∧B∧C` minus the match set is the only population
   where codegen alone can convert.
7. **Check the board.** Five rows have re-entered a ranking after already
   measuring zero. `grep` `BOARD.md` for the key before sizing a rung on it.
8. **Before pricing a TU CONVERSION, quote the GATE's number, not a
   per-function one.** *Added by `w-mmioclose` (#2406).* `fn_names` and the
   scan row's own `detail` string say whether `Bindings::per_record` binds this
   TU at all; `frontier-bytefrac-*`, `fnbyte-exact` and the emitted census do
   not, because they are keyed on `FnCensus::emit_name`. A TU can be at 67 % by
   byte with **one** of its eleven names bound. The two bindings disagree on
   74,955 workload rows (#918), and the cheap check is one line of the TU's own
   `gap --jsonl` row.

   **`fn_names` IS NOT THAT LINE, and reading it as one is how the second
   instance was missed for a day.** *Amended by `w-front5` (#2621, #2624).*
   `fn_names` is `c2_il::mangled_names(gl).len()` — the census's loose scan —
   and it can equal `fn_total` on a TU the gate does not bind (`src/Main.cpp`
   reads `fn_names 1, fn_total 1` and binds **nothing**) and disagree with it
   on a TU it does. **The field that answers item 8 is `gate_cause` /
   `gate_causes`**: any `gl-stop-*` or `bind-*` clause in the SET means
   `Bindings::per_record` returned `None` and every emitter question on that
   TU is unasked. For the stop **record** rather than the stop clause — which
   is what tells you how much of the walk survives — run
   `work/w-front5/glwalk.py <bundle.gl> <bundle.ex>`, a transcription of
   `gl_defined_names_framed(gl, true, codec::gl_offset_framed)` over the TU's
   own capture.

   **And do not assume the binding repair is free.** `w-front5` #2622 built the
   one-line widening as a counterfactual: it binds **2 of the 15** TUs that
   stop on `gl-stop-name-not-mangled`, converts **0**, moves **0** of 878 TU
   verdicts — and costs **−1 `fnbyte-exact`**, because `gl_defined_names` is
   also `bind::defined_name_set`, the ground set the inline fence tests callees
   against, where a widening of the walk is a **tightening** of the fence
   (#2623).

9. **If T1 fires, read the port's own FENCES before you read its
   obligations.** *Added 2026-08-09 by lane `w-xtea3` (#2691, NC-5).* A TU whose
   every body is `fnbyte-exact` and which still does not match may be blocked by
   a refusal the port makes on purpose rather than by a symbol it owes.
   `EncryptXTEA.cpp` was, at 5 of 5 bodies exact, and the repair was one clause
   in `comdat::fenced_inlined_callee` — adopted from `WB_INLINE_FINDINGS` §7's
   MAY table, which had five licensed arms of which four were already in the
   code. The three fences to check are `comdat::fenced_inlined_callee`,
   `elide`'s mechanism E, and `splice`'s S7.
