# Chopping row 4a — 2026-08-21

The owner asked, after settling the goal
([`GOAL_DECISION_2026-08-21.md`](GOAL_DECISION_2026-08-21.md)):

> *"how can we chop the roadmap into concrete deliverables in fewer than 45
> months (a crazy length of time)?"*

Four research lenses were dispatched against that question. Every number below
that carries **✅** was **recomputed by the coordinator** from a fresh
878-TU scan at master `b3f131b31` (workload `2f666acc8aa2`, wibo
`1.2.0-c2rs.1`, cl/c2/c1xx `16.00.11886.00`) or read from source — not taken
from a lane's own arithmetic.

---

## 1. The answer, stated plainly

**Slicing does not make the total smaller. Enumerated honestly, it reads
larger.** The decode lens priced the same work bottom-up as **14 slices,
31–59 engineer-months** with `CEILING` §5's ~5:1 calibration applied per row —
against the review's top-down **15–45**. The review's figure lands at the
optimistic end of the enumeration, which is what a lower bound is supposed to
do; **the direction is the finding.**

Three mechanisms make the enumeration read higher, and all three are real:

1. **Enumeration exposes rows a layer-level estimate absorbs.** The review
   priced 4a as two rows (a general decode, a general lowering). The shipped
   instrument names **ten constructs carrying 97.3% of the residue** ✅, each
   needing its own value variant, its own type resolution, and its own
   lowering.
2. **Super-additivity is an overhead *of* slicing, not a saving.** 31,608
   bodies contain two constructs where neither alone moves one; a two-rung
   schedule under-priced the pair by 17%. Cutting by construct *creates* those
   pairs.
3. **No slice pays in `match` until many have landed.** Coverage is the
   scoreboard under goal (2), and coverage is superlinear in slices.

**So the honest reframe is the one worth taking:** not *"how do we make it
shorter"* but ***"how do we make it a sequence of 2–4 week byte-judged
deliverables, each of which banks value if the program stops right after
it, and which makes the 45-month figure itself falsifiable inside 8
weeks."*** That is achievable, and §5 is the sequence.

---

## 2. The three things that could have made it shorter — all refuted

### 2.1 A near-miss TU population — **does not exist** ✅

Over the 843 unconverted TUs with residue, the fraction of functions that are
byte-wrong is:

| min | p5 | p25 | **p50** | p75 | p95 | max |
|---|---|---|---|---|---|---|
| **0.200** | 0.632 | 0.745 | **0.781** | 0.815 | 0.947 | 1.000 |

Band counts: `[0.90,1.01)` → **0 TUs**. `[0.80,0.90)` → 1. `[0.50,0.80)` → 10.
The best unconverted TU in the workload is `src/link_glue.cpp` at **44/55**.
The distribution has a hard floor and is far too tight to be a mixture of
"nearly done" and "not started" — **every** unconverted TU is wrong on 63–95%
of its functions. There is no head to attack and no queue of TUs one push from
converting.

### 2.2 A big first-refusal win — **converts exactly zero** ✅

`gl-stop-26-introduced` is the first refusal on **818 of 844** ✅ — a two-line
clause at `crates/c2-il/src/func/gl.rs:885-887` firing when the byte before a
`.gl` record's name run is `0x26` rather than `0x00`. It is genuinely one
mechanism: 818 of 818 trip on a `??_G` scalar deleting destructor, 797 of them
on the *same* STLport `std::exception` symbol.

The lens lifted it in a scratch build at three depths, with an identity control
first (lift level 0 reproduced the pristine scan on all 878 TUs, 0 class diffs,
0 `gate_cause` diffs). **`match` held at 26, `mismatch` at 0, and 0 of 878
verdicts moved.** The wall re-forms one record later; two of its three
successors are the same walk on the next record.

And the fact that settles it, verified directly: **all 818 carry the identical
`gate_causes` pair `('gl-stop-26-introduced', 'body-out-of-class')` — one
combination, no variance** ✅. `body-out-of-class` is evaluated independently
of the binding, only **29.42%** of bodies are in the modelled class ✅, and it
is **row 4a's first half verbatim**. 4a(i) stands behind every one of the 818
at every lift depth.

> This is the sixth-plus instance of *ranking instruments measure themselves*.
> **No lane may be dispatched off that histogram.** Corroborating and
> independent: `fence-blocks-sole` is **0 for all 21 causes** ✅ — no TU in the
> workload is held out by exactly one fence.

### 2.3 Composing port output with real-c2 output — **refuted four ways**

This was the one that could have changed the total rather than the increments:
if partial function coverage could compose into a byte-exact obj, the program
would pay at TU granularity immediately. It cannot.

- **Per-function independence HOLDS** — 32 probe cells (4 shapes × 8 TU
  contexts up to 400 preceding functions) are byte-identical in payload, size
  and relocations. So independence is *not* the blocker.
- **It has nothing left to buy.** **18 TUs are per-function-complete and 17 of
  them already `match` the whole obj** ✅ — a figure I computed independently
  before the lens reported it. The single exception, `src/system/math/vec.cpp`,
  fails in the **IL reader** (`gl-stop-26-introduced`), not the COFF writer.
  The obj-global layer — labels, symbol order, section order, relocation
  numbering, string table — **costs zero TUs at the frontier.** Controls agree:
  `fnbyte-match-tu-differs 0`, `fnbyte-match-tu-reloc-differs 0` ✅.
- **The label plan is not derivable.** `$M`/`$T` stride is body-class
  dependent (measured: trivial leaf +7, 2-call framed +8, float leaf +11,
  counted loop +15, 11-local leaf +23). A function the port *refuses* has no
  class, so the port cannot number the labels of anything that follows it. A
  composed obj would have to take the whole label plan and symbol table from
  c2's obj — at which point it *is* c2's obj.

  > **AMENDED 2026-08-21 (read-plan survey, caveat 5.1): this ground's
  > evidence is struck; the refutation stands on the other three.** The
  > strides quoted above are *counterfactual displacements*
  > (`Δseed + Δcharge`), and `LABEL_COUNTER.md`'s own banner records that
  > four lanes measured them with the wrong instrument (`w-bdnz`'s +7
  > reproduces to the digit with a true charge of **+2**; eight unused
  > declarations move the counterfactual by +16 while the true charge stays
  > 1 — `WB_LABEL_FINDINGS.md:29-34,206-219`). Worse for the ground as
  > stated: the label *mechanism* is read (`WB_LABEL_FINDINGS.md` §1 — one
  > increment instruction, a TU-global counter, 31+132 enumerable call
  > sites), so "not derivable" is actually **"unread, and enumerable —
  > closed by construction"** (read-plan **R3**, 2–4 days). The composition
  > refusal does not move: grounds (a) nothing left to buy, (c) anti-safe
  > under `PROGRESS_METRIC.md`, and the open-source disqualifier are each
  > independently sufficient, and (c) alone is decisive. What changes is
  > the *lesson*: a four-ground refusal was one bad citation from being a
  > three-ground refusal, and only the habit of stacking independent
  > grounds kept the conclusion safe.
- **It is anti-safe under `PROGRESS_METRIC.md`, and this is decisive.** There
  are **2,490 functions the port lowers completely and gets wrong** ✅
  (`fnbyte-differs 1960` + `fnbyte-reloc-differs 530`), held back *only*
  because `IlBundle::functions()` refuses the whole TU. `fnbyte-decline-selector`
  is **0** ✅ — the per-function gate accepts every one of them. A per-function
  composition scheme is precisely a scheme for admitting a TU on that gate, so
  on its first run it converts 2,490 refusals into 2,490 wrong emits, each
  scoring strictly below the refusal it replaced. The concentration is exactly
  wrong: **100% of the 2,490 sit in the two call-composing shapes** ✅
  (`tail` 1090 + `seq` 870 + `tail-reloc` 530), the class that is at 0.000.

Also: an artifact that consumes real-c2 output for the remainder is **not a
100% open-source implementation**, so it cannot serve goal (2) even if it
worked.

---

## 3. What is actually true about 4a's two halves

The review priced 4a as *(i)* a general op-level IL decode and *(ii)* a general
lowering. Both halves are mis-shaped in the proposal.

**(i) is not "write a decoder."** A decode-only walker already reaches
**2,362,034 of 2,404,438 bodies (98.2%)**, corroborated by the `step5`
partition I verified: only **41,657 bodies (1.73%) are undecoded** ✅, one of
whose four rows is EH in disguise. What the walk lacks is a *return value* —
it knows widths and construct names and discards them into a `&'static str`
counter. The real hole is **83.5% of bodies having ≥1 operand outside the
semantic model** ✅, decomposing into ten named constructs ✅:

| construct | bodies | cum. |
|---|---:|---:|
| off-add (`0x27`) | 696,164 | 33.3% |
| intrinsic (`0x40`) | 464,172 | 55.4% |
| bind (`0x99`/`9A`/`9B`) | 413,626 | 75.2% |
| load-type | 221,583 | 85.8% |
| temp · lit-type · compare · bitwise · materialize-64 · virtual-slot | 241,297 | 97.3% |

**(ii) is not 35 bespoke lowerings.** `coff::Function`'s construction site is
one struct literal (`crates/c2-core/src/lib.rs:703-715` ✅) whose contract is
**seven values, four load-bearing**. The obj-global assembly is already general
over an arbitrary function list, and 13 of the 35 shape emitters already share
one core (`block_ir`), calling it identically. Concentration is extreme:
**`Plain` + `Tail` are 34,622 of the 35,894 byte-exact functions — 96.5%** ✅,
with the other 28 named shapes accounting for 17 functions in total.

**And one proposal claim is false as written** ✅: step 5 offers "existing
`alloc`/`order`/`schedule` rules as regression fences." Every production caller
is inside `crates/c2-core/src/codegen/leaf/store.rs`; every reference elsewhere
in `crates/` is a **comment**, not a call site. They fence store runs and
nothing else. That sentence should be struck.

> **Corollary that changes the scoreboard.** CFG structure is not the
> constraint: **66.2% of the 113,565 refused emitted functions are already a
> single basic block** ✅, and a block IR alone converts 717 bodies / 9 emitted
> functions. The number that predicts conversion is **TUs at per-function
> completeness** — 18, of which 17 convert. It is already instrumented; it is
> simply not the headline.

---

## 4. The riskiest assumption, which nothing in the tree has ever measured

Both lenses arrived at the same one from opposite directions, and it is the
reason the 8-week experiment in §5 is worth more than any amount of further
planning:

> **Is the port's byte-exactness a MODEL, or a FIT?**

`select_function` is **never called** for a parse-refused function. So the
lowering's apparent 91.2% success rate is **measuring the catalogue against its
own admission gate** — the true reach of the existing lowering behind a general
decode is *unmeasured, and the current tree cannot produce it.* That single
number decides whether 4a is one program or two.

The concern is not hypothetical. `codegen::alloc`'s clauses are, in this
repo's own words, a *"fitted stand-in"* for c2's unread worklist order, with
**clause 2 refuted on 7 of 56 fresh-holdout cells under a preregistered
52,416-configuration search**; `codegen::schedule` carries a 13,104-configuration
residual, also fitted against emitted bytes. If the incumbent bytes are a fit
to the shapes' populations, every "general" layer is a re-fit and 4a is not
15–45 months — **it is unbounded.**

---

## 5. The sequence — 2–4 week slices, each banking value

`raw` is the nominal estimate; `LB` applies `CEILING` §5's ~5:1 optimism
calibration and is a **lower bound**, not a range.

### Phase 0 — make the 45-month figure falsifiable (8 weeks raw)

| # | ships | graded by | raw | LB |
|---|---|---|---|---|
| **S0** | **The blind-reach instrument.** For each of the 113,565 parse-refused functions, build a candidate body from a relaxed decode, run it through the one composition (`comdat::comdat_function_body`), grade against **real c2's own COMDAT bytes**. Publish `fnbyte-blind-{exact,differs,unlowerable}` beside FBM, under FBM's separation rule (never in `gate.sh`, licenses no emit). | byte-judged per function + required-zero identity (emit-path diff 0, 18-lane gate diff 0 lines, `match 26 / mismatch 0 / fnbyte-exact 35894` unmoved) | 2–4 wk | 10–20 wk |
| **S1** | **General `Plain` + `Tail` lowering**, `Selected::Plain`/`Tail` deleted and re-added only as `#[cfg(test)]` cross-checks, driven from a per-op value carrying c2's opcode number. | required-zero byte delta on the **live** dispatcher (#3346's condition) + a **pre-registered cost criterion** (#3336; reuse ir0's protocol — per-fixture minimum over 8 rounds with a byte-identical null arm) + the `after0` opcode agreement **as a ratio measurement** (see the amendment below) — instrument, never a gate | 2–4 wk | 10–20 wk |

> **AMENDED 2026-08-21, by the lane that owns the bijection.** An earlier
> draft of this row proposed promoting `w-ildecode`'s
> `the_final_tuple_order_reproduces_the_text_words` from 3 functions / 9 words
> to the whole fixture corpus **unchanged**. That is wrong and would have
> produced a false alarm. The test *asserts equality*, and the lane names in
> advance where equality breaks: on any **framed** function, because the final
> expansion switch rewrites the prologue pseudo-op in situ into many words
> (`WB_REGALLOC_FINDINGS.md` §4 item 2, `0x2f4`/`0x2f0` → `0x10bff95c`).
> Pointed at the corpus it goes **red on the first framed function** and reads
> as an instrument defect rather than a property of c2.
>
> Promote it instead as a **per-function ratio that reports its
> distribution** — *"bijection on N of M, and here is the shape of M − N"* —
> which is the number a general lowerer's price actually needs.
>
> The bijection's population, fenced by the lane at its own request (#1459's
> shape, and this project's most repeated error): **three functions, nine
> words**, all leaf, **frameless**, call-free, branch-free, memory-free,
> constant-free, **zero `.text` relocations**, one block, `int`, already
> `Port=Match`, at `/Ox /GS- /c`. Every one of those properties is shared
> across the three and **none was varied**. The three graded functions are
> precisely the ones that *cannot* exhibit the framed-prologue expansion.

**Why these two first.** They attack the assumption in §4 rather than the
easiest work, and between them they can move the 15–45 figure **in either
direction** inside 8 weeks:

- **S0 large `blind-exact`** → the catalogue generalises past its gate; 4a is
  decode-first and (ii) is nearly free. The review's coupling is wrong.
- **S0 ≈ 0** → the halves are **not separable**; a general decode buys nothing
  without a matching general lowering per construct, and **15–45 is
  optimistic, not conservative.**
- **S0 large `blind-differs`** → the sharpest of the three: a direct price on
  how many wrong emits the *next* `functions()` widening would ship. That is
  the hazard `STATUS.md` has carried unpriced since 2026-08-06 and the reason
  §2.3 is anti-safe.
- **S1's delta cannot be held** → I2 is not the smaller half; 15–45 moves up.
- **S1 delta holds but workload `fnbyte-exact 35,894` moves at all** → the
  byte-exactness is a **fit**, the pricing basis is void, **and the program
  should stop.** No smaller slice can produce this result.

Even if the program halts after Phase 0, the tree keeps: a standing instrument
that prices every future decode widening *before* it ships — the two-sided
pricing control (#1042, NC-5/#2691) the decode side has never had — and, from
S1, the proposal's own step 6 delivered for 96.5% of the port's byte-exact
output.

### Phase 1 — the constructs (10 slices, 97.3% of the residue)

C1 off-add · C2 intrinsic · C3 bind · C4 load-type · C5 temp · C6 lit-type ·
C7 compare · C8 bitwise · C9 materialize-64 · C10 virtual-slot. Each ships one
value-type variant + its type resolution + one general lowering; each is
**byte-judged on new fixtures**, plus a required-zero identity diff on the
incumbent gate table. **2–4 wk raw / 10–20 wk LB each.**

Two orderings that differ from naive mass order:

- **C1 is a promotion, not a construction** — `designator.rs` already resolves
  the offset and width for four consumers. Largest row, cheapest work; take it
  first.
- **C4 is the inverse trap.** Its type gate is worth 3.2× on *decode* reach,
  but the repo's own grammar notes say its *emission* — lowering a `double`, a
  pointer, a 64-bit pair — "is far more work than its decode." **Do not
  schedule C4 off its decode number.**

### Phase 2 — the genuinely-undecoded residue (1 slice)

`0x59` / `0x08` / `0xBC` / `cf-offadd-type-0x86` / `cf-no-body` = 31,928
bodies. 1–2 wk raw. **This is the only slice in the whole program that is
literally "write more decoder," and it is 1.3% of the mass.**

### Running alongside — characterization, which is product under goal (1)

- **The encoder table — re-scoped 2026-08-21, and the correction is the
  lane's.** An earlier draft of this row priced "dump all 111 encode-form arms
  of `0x10c39b18` plus the base-word table `0x10c3a578`" at **1 wk raw**. That
  estimate was the coordinator's, not measured, and it **priced three jobs as
  one**. `w-ildecode`, which holds 2 of the 111 arms, splits it:
  - **(a) dump both tables + histogram `form` over the workload's emitted
    opcodes — half a day.** `docs/whitebox/scripts/dump_opcode_tables.py`
    already exists and already reads both tables; this is a loop and a counter.
  - **(b) read the arms — ~2 days** for the stereotyped majority, and
    **unbounded** for the tail that branches on `DAT_10c2e978` or calls
    `0x10bf983a`/`0x10bf98ec`. None of that tail has been read.
  - **(c) grade them — not priceable until (a) runs.** Pricing it first repeats
    `CEILING` §6.1 item F's error: a figure against an unmeasured denominator.

  **This row is therefore scoped to (a) alone**, with (b) and (c) priced off
  the resulting histogram. Registered so it can be scored: the lane expects
  **20–40 forms to cover ≥99% of emitted words**.

  > **AMENDED 2026-08-21 (read-plan survey):** the arms live at
  > **`0x10bfae2d`** (the jump table), not `0x10c39b18` (that is the form
  > *table*), and the 111 entries collapse to **79 distinct arms** ✅ —
  > coordinator re-measured from the pinned image (sha256-matched; all 79
  > targets inside the encoder's 3,861 bytes; busiest arm shared by 12
  > forms). The tail called "unbounded" in (b) is **bounded**: 3 call
  > sites of `0x10bf983a`, 1 of `0x10bf98ec`, 12 references to
  > `DAT_10c2e978` (survey-measured). (b) is ~29% smaller than written
  > and has no unbounded component. Full plan:
  > `whitebox/READ_PLAN_2026-08-21.md`. Note also that the row's
  subject **excludes relocations entirely** (§5.6, zero cells) — so it prices
  *part of* interface 2, not interface 2.
- **Resolve the `DAT_10c400d4` contradiction.** The repo asserts both
  *"function-scoped"* (`WB_LIVE_FINDINGS.md:258-260`) and
  *"compilation-global"* (`P_REGALLOC.md:62,188`) for the same counter — and
  that question is exactly what `select_function(func, mode)`'s
  no-TU-context signature rests on. The composition probe measured the
  observable consequence null on 4 shapes; a wider grid plus reading
  `0x10b2c1f1` and `0x10b55732` settles it and earns a `DISCLOSURE.md` row.
  1–2 wk raw.

### Not sliced this way — named so absence is not read as coverage

Item F (register allocation / scheduling — 13 raw lanes, **65 calibrated**),
EH (phase 4), weak externals (phase 5 — 674 TUs, costs 0 today and everything
at 871), COMDAT synthesis (phase 6 — the 399-TU wall), the inliner (phase 2).
A slicing that silently absorbed these would repeat the proposal's own error.

---

## 6. Standing rules for every slice

1. **Byte-judged by real c2, or it does not count.** No new predicate stands in
   for the compiler judge.
2. **Two-sided pricing.** Every admission widening is priced with both its
   hold-out *and* the wrong emits it would admit — Phase 1 is gated on S0's
   `blind-differs` number for exactly this reason.
3. **Per-symbol movement, never subtracted totals.** `w-empty`'s first attempt
   read `+0/−14`; an aggregate cannot distinguish `+1,400/−27` from
   `+1,373/−0`.
4. **Publish the denominator in the sentence that states a null** (#3356).
5. **No slice dispatched off a blocked-key size ranking** — `fence-blocks-sole`
   is 0 for every cause ✅, and this failure family is now at six instances.
6. **Read before probe** *(added 2026-08-21 with the owner's goal
   re-ranking — `GOAL_DECISION_2026-08-21.md` § "AMENDED",
   `WHITEBOX_LEVERAGE_2026-08-21.md`)*. Before any slice budgets a probe
   grid or a fitted-parameter search, price the whitebox read that would
   answer the same question and prefer it. Item F's 13-raw/65-calibrated
   pricing is the black-box number; no slice may quote it as the cost of the
   *fact* when the cost of *reading* the fact has not been priced.
7. **Expose the decision surface** *(same amendment)*. Every general layer
   ships its arbitrary choices — allocation order, scheduling tie-breaks,
   label counters — as named, enumerable parameters, not baked constants.
   The permuter and the training-signal pipeline consume that surface; a
   baked constant is a fit the next population re-opens.

---

## 7. Corrections this round produced

- **`fnbyte-tus-full 21` decomposes as 18 raw + 3 whole-TU credit**, not
  "20 match + 1" ✅. Matched-TU FBM is **43/46**, not 46/46 ✅.
- **FBM is not a floor for `match` in either direction.** Three TUs —
  `src/Main.cpp`, `TomCryptLicense.cpp`, `ZlibLicense.cpp` — **match byte-exact
  at the obj level while FBM refuses their body** ✅. The docs carry only the
  converse caveat.
- **`docs/STATUS.md` is stale on the FBM row**: it quotes `fnbyte-differs
  3,195` and the "861 exact relocating against a symbol c2 does not name"
  caveat; the tree reads **1,960** and **530** ✅, that caveat having been
  closed by construction at `w-relo`/#884.
- **Proposal step 5's `alloc`/`order`/`schedule` regression-fence claim is
  false as written** ✅ — store-run scope only.
