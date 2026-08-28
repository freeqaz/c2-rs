# Step 5, re-priced per stage — `w-restim`, 2026-08-21

The re-estimation `docs/ARCHITECTURE_PROPOSAL_2026-08-20.md` §6 promised
(*"re-estimate after the stage oracle and after the plan manifest"*) and
`docs/ARCH_REVIEW_2026-08-21.md` consequence 3 dispatched. Every number below
is derived from a log committed under `work/w-restim/`; predictions were frozen
in `work/w-restim/PREREG.md` before any probe ran.

**Read every forward cost figure here as a LOWER BOUND.** `docs/CEILING.md` §5:
the project's pre-registered estimates miss **optimistic roughly 5:1**, and the
misses are specifically on *forward cost*, not on measurement. That calibration
is **applied** below (each row carries a raw figure and a ×5 lower bound), not
merely cited — which was one of the review's amendments.

---

## 0. The headline, and it is a split verdict

| question | answer | changed by |
|---|---|---|
| Is COLOR gradeable **against real c2**? | **YES.** Its output is the operand's symbol pointer being rewritten from a candidate record to a physical-register record, and the register number is directly readable | probe A |
| Is the final schedule (run 4) gradeable **against real c2**? | **YES.** It moves the list on 149 of 2,946 functions, and that was observed nowhere before | probe B |
| Can a **PORT** be graded per-stage against those observations? | **NO**, and not by a margin — the two have no common coordinate. c2 cuts regions in *tuple index at a pre-lowering phase*; on four byte-exact functions c2 has 19–22 tuples where the port emits 4–5 instructions | probe C |

So step 5's grading premise fails **in one direction only**, and that is the
whole re-pricing. Reading c2's per-stage behaviour is now **cheap and
mechanised**. Grading a ported pass per-stage still requires reproducing c2's
region decomposition and its tuple coordinate system — arch review finding 1's
last bullet, confirmed, and finding 4 (a)/(c) turned into a measured ratio.

**The consequence for the plan is not "step 5 is impossible". It is that
per-stage observability buys CHARACTERIZATION, not a per-stage differential
grade** — and characterization was already the expensive half (`CEILING` §6.1
item F0 is 8 of item F's 17 lanes and is exactly *"the order that decided the
registers does not appear in the obj"*).

> ### ✔ 2026-08-21, HOURS LATER — **THAT HEADLINE IS A DIRECT HIT, NOT A CONSOLATION PRIZE. THE MEASUREMENT IS UNCHANGED; ONLY ITS SIGN IS.**
> *Owner's goal decision,
> [`GOAL_DECISION_2026-08-21.md`](GOAL_DECISION_2026-08-21.md), which cites
> this page's headline by name. Annotated by lane `w-goaldocs`; **nothing on
> this page is edited, re-priced or withdrawn**.*
>
> Goal (1) is *"the perfect reproduction that gives us a clear understanding of
> the MSVC internals, to help us with decomp."* **And the owner ranked it
> PRIMARY later the same day** (`GOAL_DECISION` § "AMENDED"; noted here
> 2026-08-22 by lane `w-readdocs`), which **sharpens this annotation rather
> than qualifying it**: the hit is on the *primary* goal, and the miss is on
> the goal that is now both an end and a means to it. A mechanised, addressed account
> of what c2's middle actually does **is the deliverable** — whether or not a
> ported pass can be graded against it today. So *"buys characterization, not a
> grade"* reports a **hit on goal (1)** and a **miss on goal (2)**, and
> `ARCH_REVIEW_2026-08-21.md` reported it as a downgrade only because the
> standing thesis at the time was verifier throughput.
>
> **Three things this does NOT license.** (a) The judge is untouched — reading
> c2's internals is not permission to grade the port against c2's internal
> state, and §8's standing bound on the stage oracle is what keeps snapshot
> equality out of `crates/`. (b) Probe C's verdict is unchanged: the port→tuple
> projection is **undefined**, not merely unequal. (c) Every forward cost
> figure here is still a **lower bound** under CEILING §5's ~5:1, and the
> 15–45 engineer-month price on the integration prerequisites is unaffected —
> what the goal decision changes is *which end* that price is weighed against
> (§5), not the price.

---

## 1. The per-stage table — measured, 384 fixtures, 2,946 function-pairs per bracket

`work/w-restim/probeAB_all_fixtures.log`, at the eight-site table with the
operand walk and the whole-function walk on. Every bracket is *the same
traversal read at two adjacent phases*; SPINE is opcode/category/flags/cc,
FULL adds the operand and symbol/candidate records.

| stage | bracket | spine moves | operand-only moves | total | gradeable vs real c2 |
|---|---|---:|---:|---:|---|
| scheduler run 1 | `sched1`→`globregs` | 453 (15.4%) | 35 (1.2%) | **488 (16.6%)** | YES |
| **globregs** | `globregs`→`sched2` | 2,766 (93.9%) | 36 (1.2%) | **2,802 (95.1%)** | YES |
| scheduler run 2 | `sched2`→`color` | 171 (5.8%) | 4 (0.1%) | **175 (5.9%)** | YES |
| **COLOR** | `color`→`sched3` | 130 (4.4%) | **771 (26.2%)** | **901 (30.6%)** | **YES — 85.6% of its visible footprint is operand-only and was invisible before this lane** |
| run 3 **+ the lowering band** | `sched3`→`sched0` | 2,946 (100%) | 0 | **2,946 (100%)** | YES but **NOT SEPARABLE** — no tap site exists between them |
| **final schedule (run 4)** | `sched0`→`after0` | 118 (4.0%) | 31 (1.1%) | **149 (5.1%)** | **YES — newly observable, site `0x10b7e701`** |

Two readings that are NOT in this table and would be wrong:

* **"COLOR is a no-op on 69.4% of functions"** is what the IDENTICAL column
  says, and it is a fact about the *fixtures*, not about the allocator: a
  function whose every value already sits in its arrival register has nothing
  for the allocator to write.
* **`#3323`'s "IDENTICAL 7/7 · offsets COLOR writes: NONE"** is unmoved and
  reproduces exactly at this tip — on `il_call_perm.cpp`, where COLOR really is
  a no-op on all seven functions. It does not generalise, and the sweep is how
  we know: 105 of the 292 fixtures with an aligned raw window name at least one
  offset (`work/w-restim/snap_all_fixtures_verdicts.log`).

### 1.1 What COLOR's write actually IS

Before (`color` site, i.e. entering the allocator) and after (`sched3` site),
`add3.cpp` fn1, verbatim from `work/w-restim/probeA_add3.log`:

    BEFORE: 00000272 0d 01 04 | OP D 0 01 1004 01 00000004 unread 00000004 | OP S 0 01 1004 02 00000002 none 00000002
    AFTER : 00000272 0d 01 04 | OP D 0 01 1004 01 00000004 unread 00000004 | OP S 0 01 1004 01 00000004 unread 00000004

The source operand's symbol goes from **kind `02`** (a candidate, id 2, assigned
descriptor `none`) to **kind `01`** (a physical register record, value `4`).
Register `4` is `r3` on the `0x10b181c0` table in the `n = r+1` encoding. On the
last differing row of the same function, candidates `{2,3,4}` become registers
`{4,5,0xc}` = `r3, r4, r11`.

**This answers `docs/rungs/2026-08-20-stageoracle.md` §6.1 q1 in full**, and it
corrects a reading on the way: `P_REGALLOC.md` §4.1's `+0x1c` is the candidate
**id**, not a register — the register is one further hop, and the allocator does
not write it into the candidate the operand already points at. It re-points the
operand.

---

## 2. The two unbudgeted integration prerequisites, priced as their own rows

Arch review finding 3. Neither appears in any `§5` row of the proposal, and both
are on the critical path of **any byte-judged output** from a ported pass.

> **⛔ RE-PRICED 2026-08-26 — READ §2.1 BEFORE QUOTING ANY CELL OF THIS TABLE.**
> The digits below are unchanged and stand. What §2.1 adds is each figure's
> **derivation** — and it establishes that **none of the three inputs wave 11
> corrected was ever an input to this table**, that the corrections' net
> direction is **UP** and they do **not** cancel, and that the ~~"×5 lower
> bound"~~ column head is **`raw × 5` where the 5 was fitted on rung counts,
> not on time** (§2.1(e)). Lane `w-price4a`, board **#3603**–**#3608**.

| # | row | why it is a prerequisite | raw estimate | ~~**×5 lower bound**~~ **`raw × 5` — §2.1(e)** |
|---|---|---|---:|---:|
| **I1** | **general op-level IL decode** | IR0 stops at a two-variant byte framing; `BodyShape`'s 35 whole-function grammars are simultaneously the admission gate. A ported COLOR consumes a *semantic middle* that does not exist. Probe A sharpens this: c2's own middle is a tuple list with per-tuple categories and two operand lists, and the port has neither — probe C measures the gap at **4.4–4.75× more tuples than emitted instructions** on four byte-exact functions | 1.5–4.5 eng-months (reviewer's 3–9 for the pair, split by the reviewer's own ordering) | **7.5–22.5 eng-months** |
| **I2** | **general lowering to `coff::Function`** | today a 35-arm per-shape dispatch (`comdat.rs`, 43 `Selected::` refs). Without it a ported pass's only progress signal is stage-parity — **#3336 at program scale**, an instrument with no emit-path consumer | 1.5–4.5 eng-months | **7.5–22.5 eng-months** |
| | **I1 + I2** | | **3–9 eng-months** | **15–45 eng-months** |

> ### ⚠ 2026-08-22 — **BOTH ROWS NOW HAVE A NAMED, SIZED, MECHANICAL READ, AND THE PRICES ABOVE ARE THE *BLACK-BOX* PRICES.**
> *Lane `w-readdocs`, propagating `whitebox/READ_PLAN_2026-08-21.md` §3 and
> `WHITEBOX_LEVERAGE_2026-08-21.md` §1 (read before probe;
> `ROADMAP_SLICING_2026-08-21.md` §6 rule 6). Board **#3367**. **No number in
> the table above is edited, re-priced or withdrawn** — this is the single most
> consequential row of that propagation and it is stated as an annotation on
> purpose.*
>
> | this row | the read | days | what the read produces |
> |---|---|---:|---|
> | **I2** general lowering to `coff::Function` — 1.5–4.5 eng-mo raw / 7.5–22.5 calibrated | **R2**: extend `dump_opcode_tables.py` to the base-word table `0x10c3a578` and the encode-form table `0x10c39b18`, then read the **79 distinct arms** of the jump table at `0x10bfae2d` (111 entries, 79 targets, **all inside `FUN_10bf9f15`'s 3,861 B** — coordinator-verified from the pinned image) plus 4 helper sites and `DAT_10c2e978` | **2–4** | `encode(tuple) → u32` as a **total function** — the opcode table `0x001..0x294` plus 79 field-composition rules, replacing `encode.rs:207`'s black-box re-derivation of exactly what those two tables state plainly |
> | **I1** general op-level IL decode — 1.5–4.5 eng-mo raw / 7.5–22.5 calibrated | **R5**: read `FUN_10bc2d7a` (5,080 B), the ~~**189-arm**~~ **61-real-arm (95 opcodes + 94 refusals — "189" is the OPCODE count; [`WB_ILARMS_MAP.md`](whitebox/WB_ILARMS_MAP.md) §1, decision 13)** IL-record→codegen-tuple dispatch, jump table `0x10bc4152`, ops `0x01..0xBD`, **zero arms read today** | **15–25** | the semantic map for the ten residue constructs, and the **shared input to all ten Phase-1 construct slices**. Also locates the *select*-vs-*decode* boundary per arm — which is the I1/I2 split the whole 15–45 eng-mo estimate rests on and has never been located |
>
> **The order is not free-choice.** `READ_PLAN` §3's dispatch order is
> **R1 → R2 → R3**, with **R5 gated on R2** — R2 proves the arm-reading method
> on 79 bounded bodies before R5 spends 15–25 days on 189.
>
> **Four things this annotation does NOT do, stated because a days-vs-months
> comparison invites all four:**
>
> 1. **It does not re-price I1 or I2.** A read produces a **spec**; I1 and I2
>    are *implementations* in `crates/`, which a spec makes tractable and does
>    not make free. The honest claim is that the reads remove the *discovery*
>    cost, which is the part these estimates were carrying implicitly.
> 2. **It does not make step 5 GO.** §5 and
>    `ARCHITECTURE_PROPOSAL_2026-08-20.md` §8 decision 0 are the owner's, and
>    they stay open. The read-plan is new *information* bearing on that
>    decision, not a decision.
> 3. **R5's own price is uncertain in the direction that matters.** 15–25 days
>    is the survey's estimate for a body with **zero arms read**; R2 exists in
>    the order partly to test the method's per-arm rate before that bet is made.
> 4. **`[R]` is a hypothesis** (`READ_PLAN` §5.3) — *"the instructions were read
>    correctly"*, not *"this is what c2 does"*; the `.bss` bump rule was read
>    correctly out of a clean function and was wrong about c2. Every read ends
>    in a confirmation probe, and the byte judge is untouched by all of it.

### 2.1 ⛔ 2026-08-26 — **I1 RE-PRICED. THE DIRECTION IS UP, THE MAGNITUDE IS NOT RE-PRICEABLE BY ARITHMETIC, AND THE REASON IS THAT NONE OF THE CORRECTED INPUTS WAS EVER AN INPUT TO THIS TABLE.**

> *Lane `w-price4a`, wave 12, funded by
> [`DECISIONS_2026-08-22.md`](DECISIONS_2026-08-22.md) **decision 14**. Board
> **#3603**–**#3608**; rung [`rungs/2026-08-26-w-price4a.md`](rungs/2026-08-26-w-price4a.md).
> **This is the canonical re-price block; every other pricing surface in the
> tree points here rather than restating it.** No digit in §2's table above or
> §4's table below is edited — what is added is each figure's **derivation**,
> which is what none of them carried, and the direction of the residual.*

**Every number below was reproduced in this lane, not inherited.** The whole
reason the lane exists is that a published figure was wrong three ways.

#### (a) The five inputs, and the direction each pushes

| # | input | as the price was written | as landed at `f202268f6` | reproduced how, in this lane | on I1's **BUILD** | on R5's **READ** |
|---|---|---|---|---|:-:|:-:|
| 1 | dispatch size | **189 arms** | **61 real arms serving 95 opcodes**, plus one refusal arm serving 94; 62 targets, **62 distinct** | `dump_ilarms.py --verify` on sha256 `c80981c0…a66258`: domain `0x01..0xbd`, arm table 62 entries, 62 distinct, arm 61 = the `ja` target, 94 routed there | ↓ unquantified | ↓ ~3× — **and already SPENT** |
| 2 | off-model share | **83.5 %** | **88.61 %** | one line: §3's ten rows sum to **2,036,842**, and `0.835 × 2,404,438 = 2,007,706` — **smaller than the rows it contains** (101.45 % of its own population) | ↑ | — |
| 3 | decode reach | **98.2 %** | that is **FRAME** reach (98.25 %); **MODEL** reach is **11.39 %** of bodies, **5.47 %** by byte | the standing `decode-reach-*` keys, quoted with their strength and stamp; `inmodel` **275,295** equals `reached − modeled`'s complement exactly | ↑ **— and see (b)** | — |
| 4 | port-side mapping | never stated | 41 of 61 arms have ≥1 port reader, 68 of 95 opcodes — **but 24 of those 68 carry no operand field at all**, and **0 production sites mint an IR node** | `scan_port_opcodes.py --coverage` (41/61, 20/61, 68/95, 24 WIDTH-or-NAME-ONLY) + an independent hex scan of **202,610** Rust lines for the ≥ `0x2af` node space | ↑ | — |
| 5 | layer count | 1 fused layer (decode ∧ admission) | **3** — `w-unfuse` split decode from admission, and symbol binding sits under both: **4,001** bodies at `shape_to_function`, every one `:eof` | read from the landed `AdmissionPolicy` seam and `grammar-not-admitted` | ↑ | — |

#### (b) They do NOT cancel, and the four are three

**They cannot cancel, structurally.** The one input that pushes *down* applies
to a **different row** — R5's read — from the four that push *up*, which apply
to I1's build. This page's own annotation note 1 above says why that is not a
transfer: *"a read produces a **spec**; I1 and I2 are **implementations**."*

**And inputs 2 and 3 are one measurement counted twice.** `decode-reach-inmodel`
and the complement of `decode-reach-offmodel` are **both 275,295** — checked,
not asserted. The count of *independent* corrections to this row is **three**,
not four.

#### (c) The magnitude: NOT re-priceable, and here is the derivation that shows why

| figure | its actual derivation | does it read input 1, 2, 3, 4 or 5? |
|---|---|:-:|
| I1 raw **1.5–4.5 eng-mo** | this page's own §2 cell: *"(reviewer's **3–9** for the pair, split by the reviewer's own ordering)"* — `ARCH_REVIEW_2026-08-21.md:116`, halved | **no** |
| I1 **7.5–22.5** | `1.5–4.5 × 5` | **no** |
| 4a whole **15–45** | `3–9 × 5` | **no** |
| bottom-up **31–59** (`ROADMAP_SLICING` §1) | `n_slices × flat 2–4 wk × 5` + the characterization rows — **every Phase-1 slice is priced 2–4 wk regardless of its mass**, so C1's 696,164 bodies and a pooled row cost the same | **no** |

**All four published figures are input-free.** Correcting 189 → 61,
83.5 → 88.61 and frame → model reach therefore cannot move any of them: the
corrections are not corrections *to* these numbers, because these numbers never
read them. **A figure whose inputs cannot be corrected cannot be re-priced. It
can only be withdrawn, or left standing with its derivation printed beside
it** — which is what none of them had until this block. They are left standing.

**This is not "unresolved".** It is a specific and checkable claim: *the figures
are not estimates of the thing they are quoted for.* What is genuinely
unresolved is the magnitude, and (e) names what would resolve it.

#### (d) What DID move: the denominator, by 50.5×

A lane measured on **FRAME** reach begins at 98.25 % with **1.75 points** and
**42,404 bodies** of residue. A lane measured on **MODEL** reach — the strength
a *general decode* is defined by — begins at 11.39 % with **88.61 points** and
**2,142,499 bodies** of residue.

| | FRAME residue | MODEL residue | ratio |
|---|---:|---:|---:|
| bodies | 42,404 | 2,142,499 | **50.5×** |
| bytes | 24,397,092 | 611,476,251 | **25.1×** |

**`IL_DECODE_REACH.md`'s banner ratios (8.6× / 17.6×) are ratios of *reach*.**
A cost estimate is proportional to *residue*, and the residue ratios are the two
above. **They are published here for the first time.**

> **⚠ AND THEY ARE NOT A COST MULTIPLIER.** Cost is not linear in residue and
> nobody has measured how it scales. 50.5× is the honest correction to the
> **denominator** a re-price would use *if either published figure were
> denominated in anything*, and (c) establishes that neither is. Quoting
> "50.5× more expensive" would be this page's own §2 error inverted.

#### (e) The calibration is outside its fitted domain — in two directions at once, and one is now measured

**(i) `CEILING.md` §5's ~5:1 was fitted on WORK QUANTITY, not on time.** Its own
words: *"the misses are specifically on **FORWARD COST** — frontier depth,
refusal counts, rung counts."* §4's last row of this page already states that
applying it to a **read** is a units error. Applying it to an **engineer-month**
figure is outside the same fitted domain — and this page does that two rows up.
**Neither 7.5–22.5 nor 15–45 is "calibrated" in the sense `CEILING` §5
licenses; both are `raw × 5` where the 5 was measured on rung counts.**

**(ii) Three rows of this program have since been EXECUTED, and all three
missed PESSIMISTIC on the unit actually spent.** Spans read from git, not
estimated:

| row | priced at | prereg → rung (git) | direction |
|---|---|---|---|
| **R2** — the encoder read, → I2 | 2–4 days | `f663fd27b` 10:26:26 → `c0a9e596d` 12:02:34, **1 h 36 m** | pessimistic ~30–60× |
| **R5** — `FUN_10bc2d7a`, → **I1** | **15–25 days** | funded `1cb1526b9` 00:11:43 → rung `8ff81967b` 00:41:35, **30 min** | pessimistic ~700–1,200× |
| **C1** — Phase 1's largest construct | 2–4 wk raw / 10–20 wk LB | `fef66f750` 07:05:04 → `c563744e9` 08:23:10, **1 h 18 m**; its own rung says ~0.2 engineer-days | pessimistic ~50–100× on raw |

These are **agent-lane wall clock, not human effort**, and that is exactly the
finding: **this program's forward-cost figures are denominated in a unit the
program does not spend.** `CEILING` §5's 5:1 *optimistic* was fitted on *how
many rungs*; these three are *how long a rung takes*. Both can be true at once,
they compose in **opposite** directions, and **nobody has ever multiplied them
out.**

**The one thing all three executed rows share: none converted anything.** C1
bought **+8 emitted functions, 0 TUs**; R5 bought a spec and its own
`P_ILRECORD.md` §8.6 **declines to re-price**; S0 bought a decline (**#3393**).
So the executed evidence says the **construction** is cheap and the
**conversion** is not — which is a statement about *which term* these estimates
should have been carrying.

#### (f) The upward pressure that appears in NO published price, and it is the largest

`whitebox/ref/P_ILRECORD.md` §8 item 5, verbatim: *"**This seam's output never
appears in any artifact** — it is an in-memory tree in a private opcode range.
`READ_PLAN` §3's 'the tap cannot see this seam' is not a tooling gap; it is
structural."*

Corroborated from the port side by this lane: the port's **entire** contact with
c2's ≥ `0x2af` node-opcode space is **two constants in one test file**
(`crates/c2-harness/tests/pwords_bijection.rs:57`,
`const OP_PROLOGUE: [u32; 2] = [0x2f0, 0x2f4]`) over the **nine words** of a
fenced three-function bijection — and **zero production sites**.
*(`WB_ILARMS_MAP.md` §0 says "zero non-comment hits"; the exact count is **one**,
and the one is a test. The substance — no production site mints an IR node —
stands unchanged. Reported here rather than amended there: `docs/whitebox/` is
`w-opclass`'s fence this wave.)*

**Consequence: 4a(i) funded alone inherits row 4a's own risk column.** Row 4a
exists because a step-5 lane's only progress signal would be an instrument with
**no emit-path consumer** — `#3336` at program scale with no contrast case. I1
built alone has exactly that property **at its own boundary**: its output has no
observable, so its correctness first becomes visible at the far end of
I2 + lowering + emit, through the byte judge. **The row created to prevent
unconsumable instruments is, in its first-funded half, one.** That is not an
argument against funding it. It is a term its price does not contain.

#### (g) What would resolve the magnitude — named, and priced

1. **The depth-2 read.** R5 read **depth-1 only**: 61 arms route into **76
   distinct direct callees over 174 call sites**, and **19 of 61 arms are
   DEFER** — their semantics entirely below R5's bound. `P_ILRECORD.md` §8.1
   names the highest-value target itself: **`0x10bbfebb`, 256 B**, the C1
   `off-add` builder covering **33.3 %** of the residue. Priced off R5's own
   *measured* rate (5,080 B, 62 arms and three tables in one lane), the four
   largest callees total **7,347 B** ≈ 1.4× R5's body: **1–2 characterization
   lanes.** This is the read-before-probe answer and the cheapest item here.
   *(It belongs in `whitebox/READ_PLAN_2026-08-21.md` §3 as a new row; this
   lane may not write `docs/whitebox/` and **reports it rather than writing
   it**.)*
2. **The one experiment that gives I1 an observable: a single-arm end-to-end
   slice.** Decode one arm generally → tuple → lowering → obj, **byte-judged by
   real c2**. This is the experiment **#3393** recorded that S0 *had not run* —
   S0 relaxed an **admission** gate, and **113,165 of 113,557** of its
   population (**99.66 %**) never reached the lowering at all (`no-decode`).
   **1 construct rung**, required-zero on the incumbent 21-row gate table, with
   `ROADMAP_SLICING` §5's registered stop condition unchanged: *if
   `fnbyte-exact` moves at all, the pricing basis is void.* **This is the only
   item here that can move the magnitude in either direction.**
3. **Publish the unit conversion. It costs one table and it is (e)(ii) above.**
   §4 declines to convert lanes into engineer-months rather than invent a rate,
   and that was right when it was written. Three rows have executed since. What
   they yield is a **distribution, not a rate** — which is the point: the
   estimates and the outcomes are not in the same units. **Cost: 0. Done.**

**What none of the three resolves**, stated so absence is not read as coverage:
`ROADMAP_SLICING` §4's question — *is the port's byte-exactness a model or a
fit* — which that section says decides whether 4a is one program or two and, if
a fit, makes it **unbounded**. **Every correction in this lane leaves it
exactly where it was.**

#### (h) So the figures, as they should now be quoted

| figure | amended reading |
|---|---|
| **R5, 15–25 days** | ~~an estimate~~ **SPENT — it executed** (30 min, `8ff81967b`). Quote it as an outcome. Its successor is (g)1, **1–2 lanes** |
| **I1 raw 1.5–4.5 eng-mo** | stands, **with its derivation printed**: `ARCH_REVIEW:116`'s top-down 3–9 for the pair, halved by ordering. Reads none of the corrected inputs. **Residual direction: UP** |
| **I1 7.5–22.5 eng-mo** | digits stand; **the word "calibrated" is WITHDRAWN.** It is `raw × 5` where the 5 was fitted on rung counts — see (e) |
| **4a whole, 15–45 eng-mo** | identical treatment. Digits stand, basis printed |
| **bottom-up 31–59** (`ROADMAP_SLICING` §1) | digits stand; its **flat 2–4 wk per slice** basis is refuted on its one executed row (C1) by 50–100×, pessimistic, at a conversion buy of **0 TUs** |

#### (i) It bears on Phase 1, and this lane STOPS there

`ROADMAP_SLICING` §3's ten constructs **are** Phase 1's ten slices, and they are
decomposed from the very table whose denominator was refuted. **The slice
structure survives the correction**: the ten rows sum to exactly
`reached − modeled` (**2,100,095 = 2,375,390 − 275,295**, checked in this lane),
so the decomposition is intact and only its denominator moved. What the
correction does **not** touch is **#3529**'s Phase-1 TU reach **0** or
`w-joint3`'s **97.2 %** per-TU construct floor.

**Decision 11's hold is the owner's. Decisions 13 and 14 each explicitly
declined to lift it. This lane does not either — it reports that its result
bears on the question and stops.**

#### (j) Live pricing surfaces OUTSIDE this lane's fence — reported, not written

Decision 14 fences `w-price4a` to five docs and forbids `docs/whitebox/`. A
**by-enumeration** sweep of 1,183 files (not a topic grep — `w-ilarms` found a
banner-named consumer list short by two, and every *other* `98.2 %` in this tree
is a different 98.2 % at four unrelated sites) found these carrying 4a(i)'s
price and reachable by **no** token this lane could search on:

* **`SHIPPING_ROADMAP_2026-08-22.md:778`** — a price-table row reading
  `| row 4a … | 15–45 engineer-months | … The critical path for goal (2) |`,
  with `:779` carrying 31–59. **The single most quotable statement of the price
  in the tree, and it is in neither `w-decodereach`'s 11-file list nor decision
  14's fence.**
* **`CEILING.md`** — carries **no** token at all (`4a`, `I1`, `15–45`,
  `1.5–4.5`: zero occurrences), yet its **§5 is the ~5:1 multiplier that
  produces 15–45 from 3–9**. Six live documents cite it *by reference*. The
  arithmetic lives in a file no token search reaches.
* **`crates/c2-harness/tests/stage_region_trace.rs:196`** — a **live assertion
  message** that fails *into* a re-pricing instruction (*"…the cost curve in
  `docs/STEP5_PRICING_2026-08-21.md` must be re-priced rather than this
  assertion relaxed"*), with no token at all.
* **`crates/c2-harness/src/gap/decode.rs:348`** — quotes the **refuted 83.5 %**
  as row 4a(i)'s basis, in a live doc comment.
* **`crates/c2-il/src/func/body/decode.rs:78-83`** — a verbatim restatement of
  `1.5–4.5` / `15–45` in a Rust doc comment.
* **`docs/rungs/README.md:279`** — live standing doctrine (*"STEP5's I1/I2
  eng-month prices are BLACK-BOX numbers and may not be quoted as the cost of
  the facts"*) on the `docs/rungs/` shelf, which every "pricing doc"
  enumeration in this tree treats as dated-record territory. **It is not.**
* **`docs/README.md:46-47`** — index rows paraphrasing the price as
  *"deliverables shorter than **45 months**"*, which matches neither `15-45`
  nor `15–45`.
* **`whitebox/ref/P_ILRECORD.md`**, **`ref/P_ENCODE.md:624`**,
  **`ref/P_BLOCKORDER.md:202`**, **`C2_MAP.md:1012`**, **`WB_ILARMS_MAP.md`**,
  **`READ_PLAN_2026-08-21.md` §3** — inside the `docs/whitebox/` write-fence.

**Both enumerations were short.** `w-decodereach` §12.4's 11-file list and
decision 14's 5-file fence each miss live surfaces, and the `docs/whitebox/ref/`
shelf is short *by fence* this time rather than by oversight — which means the
wave lands a re-price whose `ref/` restatements still read 15–45 unless
`w-opclass` carries them.

---

**Priced two-sided, per `CLAUDE.md`.** The cost of *not* doing them is not zero
and is not "step 5 is slower": it is that every step-5 lane lands an
unconsumable instrument, and the project has a measured precedent for exactly
that failure at one-lane scale (**#3336**, `ir0`: a required-zero byte delta
that held *by construction* because the tree had no production caller). At
program scale the same shape has no contrast case to catch it. The review's
prophylactic — *every step-5 lane names in its rung header which
`coff::Function` field its pass would eventually write* — is adopted here as the
minimum, and it is a smoke alarm, not a substitute.

> **↳ 2026-08-26 — AND §2.1(f) TURNS THIS PARAGRAPH ON I1 ITSELF.** The
> *"unconsumable instrument"* argument was written about step-5 lanes that
> would land *behind* 4a. `P_ILRECORD.md` §8.5 establishes that I1's own output
> is structurally unobservable, so the same argument applies to 4a(i) built
> alone. The paragraph is unchanged and correct; what is new is that its scope
> reaches one row further back than it was written for.

---

## 3. Characterization cost, re-priced against what the probes now cost

The unit is a **lane** (one worktree, one rung). Anchor: `CEILING` §6.2 — the
last five TU conversions cost **~17 landed rungs each**.

| stage | characterization still owed | lanes (raw) | **×5 lower bound** |
|---|---|---:|---:|
| scheduler runs 1–4 | **0 for observability** — this lane built it. Still owed: the *rule* (priority, ready-list order, cycle model) which `P_DAG.md` §3/§5 already holds at `[R]`, now checkable against a live pre/post pair | 0 + 1 to convert `P_DAG` §3/§5 from `[R]` to `[O]` | **5** |
| **the lowering band** | **1 lane, and it is the one gap this lane leaves open.** `sched3`→`sched0` conflates scheduler run 3 with the whole lowering band; there is no site between them, so 100% of that bracket moves and none of it is attributable | 1 | **5** |
| **globregs** | `P_REGALLOC.md` §7: **F1**, `0x10b55732`'s promotion policy, **unread**. The bracket moves on 95.1% of functions, so the observable is loud — the reading is what is missing | 2 (`CEILING` §6.1's own F1 price) | **10** |
| **COLOR** | **0 for observability** — probe A. Still owed: the candidate ORDER, which `CEILING` §6.1 prices at F5 = 2 and states is **not separable from F0** | 2 | **10** |
| | | **5–6** | **25–30** |

**What probes A and B did to F0.** `CEILING` §6.1 prices **F0 — "the order the
allocator is handed, and the four stages after it" — at 8 lanes**, the largest
single line in item F, and its justification is exactly *"the order that decided
the registers does not appear in the obj"*. That order is now **directly
readable at six phases plus after run 4**, and the register assignment is
readable beside it. F0's cost changes in KIND: from a black-box search over
obj-visible consequences to a differential read against a live trace.

**It does not go to zero, and pretending otherwise is the optimism §5 warns
about.** Two things survive:

1. Reading a trace is not deriving a rule. `P_DAG.md` §3's priority formula and
   §5's machine model are still `[R]`, and turning them `[O]` is what the lane
   above prices.
2. **Probe C's residue is F0's residue.** A rule read off c2's trace still has
   to be *implemented in the port*, and the port has no tuple, no category and
   no region — so the implementation lands in I1/I2's coordinate system, not in
   this one.

Re-priced honestly: **F0 8 → 4 lanes raw (×5 = 20)**, and the 4 that leave are
search lanes, not construction lanes. Item F's total goes **17 → 13 lanes raw
(×5 = 65)**.

> ### ⛔ 2026-08-28 — **THE `4` IS SUPERSEDED AND SO IS THE `13 / 65`. F0 IS A FLOOR OF ≥ 9 RAW AND ITEM F RE-DERIVES TO ≥ 17 RAW — THE SAME DIGITS AS THE 2026-08-15 FIGURE, WITH THE BOUND DIRECTION INVERTED.**
> *Lane `w-floor`, wave 17, decision 21 §3. Read + derivation:
> [`REPRICE_2026-08-28.md`](REPRICE_2026-08-28.md) §2–§4; board **#3751**,
> **#3752**, **#3753**. **Amended beside — no digit above is edited**, and the
> paragraph above is correct as the black-box statement it was written as.*
>
> * **F0.** `w-f0price` (2026-08-27) priced F0 sub-item by sub-item against
>   `WB_ITEMF` §6.1's own enumeration and published **≥ 10 + 2 UNPRICED
>   terms**. Its addends sum to **≥ 9** — the withdrawn sub-item 1 was not
>   subtracted (`REPRICE` §2.1). Either way the raw-to-raw comparison with the
>   `4` above is an **increase of ≥ 2.25×**, and the floor now exceeds the `8`
>   this paragraph re-priced *down* from.
> * **Item F.** `≥9 + 2 + 1 + 1 + 2 + 1 + 1 = ≥ 17 raw all-in` (F4 is 1, not 2 —
>   `#3710`), **≥ 16 remaining** once R4's spent F1 characterization lane comes
>   out. Against the `13` that is **≥ 1.23×**.
> * **`65` is withdrawn as an arithmetic descendant of the `13` and must NOT be
>   re-derived as `85`.** §2.1(e) of this page already says why: the ×5 was
>   fitted on rung counts, it is an *optimism* correction, and this figure is an
>   enumerated floor.
> * **And §4's curve does not add** — 5 of its 5–6 characterization lanes are
>   counted a second time in the same table, including this section's own
>   globregs row (F1) and COLOR row (F5). `REPRICE` §4.

> ### ⭑ 2026-08-28 — **THIS SECTION'S OWN TABLE HAS DECAYED IN TWO CELLS.**
> *Lane `w-floor`; board **#3752**. Reported beside, not rewritten.*
> The **globregs** row's *"`0x10b55732`'s promotion policy, **unread**"* was
> read on **2026-08-23 by R4**, and at a different address —
> `FUN_10b550e5` ([`whitebox/ref/P_GLOBREGS.md`](whitebox/ref/P_GLOBREGS.md)
> §3). Its 2 lanes are `WB_ITEMF` §6.1's F1 price, which that page decomposes as
> *"1 characterization + 1 construct rung"*, so **the characterization half is
> spent and the construction half is item F's, not this table's.** The **COLOR**
> row's 2 is F5's, decomposed the same way. See `REPRICE` §4 for the corrected
> reading of the whole table.

> **⚠ 2026-08-22 — 13/65 IS A BLACK-BOX NUMBER AND MUST NOT BE QUOTED AS THE
> COST OF THE FACTS.** *`ROADMAP_SLICING_2026-08-21.md` §6 rule 6, verbatim:
> "no slice may quote it as the cost of the fact when the cost of reading the
> fact has not been priced." Propagated by lane `w-readdocs`; the number above
> is correct as a black-box number and is unchanged.* **R7** re-does this
> paragraph's own move without new reading (confront `P_DAG.md`'s priority
> formula and latency matrix against the tap, `[R]` → `[O]`) at **3–5 days**,
> and **R4** reads `FUN_10b55732` for item **F1** at **3–5 days** against F1's
> 2 raw / 10 calibrated lanes. `CEILING.md` §6.1's 17-lane table carries the
> matching annotation.

---

## 4. The whole curve

> ### ⛔ 2026-08-28 — **THIS TABLE'S ROWS ARE EACH DEFENSIBLE AND THE COLUMN IS NOT A TOTAL: 5 OF THE 5–6 CHARACTERIZATION LANES ARE COUNTED A SECOND TIME IN THE SAME TABLE.**
> *Lane `w-floor`, decision 21 §3; board **#3752**, **#3754**.
> [`REPRICE_2026-08-28.md`](REPRICE_2026-08-28.md) §4. **No digit here is
> edited.*** §3's globregs cell (2) and COLOR cell (2) name themselves as
> `CEILING` §6.1's **F1** and **F5** prices, both of which are inside the item-F
> row's 13; §3's lowering-band lane (1) is this table's own next row. In the
> `× 5` column that is **25 of the 25–30**. And the **item F row is
> superseded** — `≥ 17 raw all-in / ≥ 16 remaining` + 3 UNPRICED terms, the
> `65` withdrawn rather than re-derived (`REPRICE` §3).

> **⛔ 2026-08-26 — the `I1` row of this table is RE-PRICED at §2.1, and the
> re-price does not change a digit here.** Direction **UP**, magnitude **not
> re-priceable by arithmetic** (none of the corrected inputs is an input to the
> figure), denominator **50.5× larger in bodies / 25.1× by byte** than the
> published `98.2 %` implied. **And this table's own last row already knows the
> units problem** — *"applying it to a read is a **units error**"* — while its
> first two rows apply the same multiplier to engineer-months. Lane `w-price4a`.

| row | raw | ~~**×5 lower bound**~~ **`raw × 5` — §2.1(e)** | on the critical path of a byte-judged output? |
|---|---:|---:|---|
| **I1** general op-level IL decode | 1.5–4.5 eng-mo | **7.5–22.5 eng-mo** *(re-priced §2.1)* | **YES** |
| **I2** general lowering to `coff::Function` | 1.5–4.5 eng-mo | **7.5–22.5 eng-mo** | **YES** |
| characterization, all stages (§3) | 5–6 lanes | **25–30 lanes** | no — it is the input to the construct rows |
| item F construct, re-priced (§3) | 13 lanes | **65 lanes** | YES |
| the lowering-band tap site | 1 lane | **5 lanes** | no |
| **the READS that spec the rows above** *(added 2026-08-22, `whitebox/READ_PLAN_2026-08-21.md` §3)* — R5→I1, R2→I2, R4/R7→item F, and six more | **≈6–10 engineer-WEEKS for all nine** | not calibrated: `CEILING` §5's ~5:1 was fitted on lane-shaped construction work, and applying it to a read is a **units error**. Quote the raw days | **prerequisite in the sense that it produces the SPEC — never a substitute for the row it specs** |

**The critical path is INTEGRATION, not any single pass** — registered as E2 at
0.70 and it holds. I1+I2 alone are 15–45 engineer-months at the lower bound,
against which item F's 65 calibrated lanes are the *second* cost, and every
characterization lane in §3 is cheap by comparison.

**Against the proposal's raw 12–24 months for step 5**: the figure is not
obviously wrong in magnitude, but it is wrong in **composition** — it was
written for the passes, and the passes are not what dominates. E3 (registered
0.55, *"the calibrated total exceeds 12–24 months"*) is **not** cleanly
resolved by this table because the two rows are in different units (engineer-
months and lanes) and this lane declines to invent a conversion. What it does
resolve: **the 12–24 figure does not cover I1 and I2 at all**, and those two
rows alone are 15–45 engineer-months at the lower bound.

---

## 5. What this does NOT price, stated so absence is not read as coverage

* **No conversion.** Predicted reach 0, delivered 0. No fixture is claimed, no
  census number moves, `match 26 / mismatch 0` is unmoved by construction.
* **The thesis-vs-870 goal question.** ~~`STRATEGY_REVIEW_2026-08-13.md:251` —
  *"The question is currently owned by nobody"* — is still true, and this
  document does not touch it. A cost curve is an input to that decision and
  never a substitute for it. Arch review consequence 1(c) stands.~~
  **ANSWERED THE SAME DAY, AND THIS BULLET WAS RIGHT ABOUT ITS OWN ROLE**: the
  goal is full reproduction, for understanding MSVC's internals and for parity
  (`GOAL_DECISION_2026-08-21.md`) — decided by the owner, not derived from this
  cost curve. Arch review consequence **1(c) is discharged**; 1(a) and 1(b)
  still gate step 5. **This page's numbers are unaffected.** What moved is the
  weighing: goal (1) is served by characterization alone, goal (2) is not
  reachable without the 15–45 engineer-month integration rows, and that is the
  open decision (`ARCHITECTURE_PROPOSAL_2026-08-20.md` §8 decision 0).
* **IR3.** Arch review finding 4's minimal repair (give IR3 its own step in
  tuple/region coordinates) is the amendment lane's, not this one's. Probe C is
  evidence *for* it: the port→snapshot projection is undefined today, and
  probe C measures by how much.
* **A per-stage grade for a ported pass.** Probe C says NO on measured
  evidence. Any step-5 row that assumes one is mispriced, and the standing bound
  (`docs/rungs/2026-08-20-stageoracle.md` §8 — no `crates/` rule enters on
  snapshot equality) is what keeps that mistake out of the judge.
