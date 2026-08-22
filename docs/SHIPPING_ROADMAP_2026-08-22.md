# Roadmap — a compiler-backend reconstruction program

**Status:** the superseding roadmap. Written under an explicit owner order to
rewrite (`docs/DECISIONS_2026-08-22.md` decision 2), aligned with the owner's
goal statement.
**Date:** 2026-08-22
**Supersedes:** [`SHIPPING_ROADMAP_2026-08-19.md`](SHIPPING_ROADMAP_2026-08-19.md),
which stays on disk as a dated record with every word intact and now carries a
SUPERSEDED banner pointing here.
**Decision requested:** §9. Two of its rows are already the owner's and are
recorded as decided; the rest are open and are posed as proposals with
alternatives, never as choices this page has made.

> **What is different, in one paragraph.** The 08-19 page recommended a
> **two-track product strategy** whose headline (track 1) was a vendor-DLL-backed
> compatibility service, and it wrote its own escape clause: *"If the product
> requirement forbids using the vendor DLL, this is not a near-term shipping
> project. It is a compiler-backend reconstruction program and should be staffed,
> budgeted and reviewed as one."* The owner's goal statement made that clause
> **true**. Two lanes (`w-goaldocs`, `w-readdocs`) bannered the contradiction and
> deliberately did not resolve it, because re-ranking a product plan is the
> owner's call; the owner has now made it. **This page is that page's own
> consequence, written out.** Nothing in the 08-19 text was measured wrong — its
> measurements are carried forward here unchanged — and what changed is which of
> them a decision may rest on.

---

## 1. What this program is

### 1.1 The goal, quoted

Quoted rather than summarized, because the wording carries the ranking and
this page may not strengthen or weaken it. Both quotations are the owner's,
2026-08-21, and both live in
[`GOAL_DECISION_2026-08-21.md`](GOAL_DECISION_2026-08-21.md).

> *"the goal is: the perfect reproduction that gives us a clear understanding
> of the MSVC internals, to help us with decomp, and also to get parity so
> that we have a 100% open source implementation."*

and, later the same day, that doc's § "AMENDED", which **ranks** them:

> *"Goal #1 is definitely the biggest. #2 is also very valuable and helps #1
> by giving us not just docs, but actual code we can tweak to instrument +
> help produce signals about the compiler's state. this is especially
> valuable for training AI models to reverse the compiler and give us a
> matching pretext. (and build a better permuter to 'brute force' fixing code
> that is close, but wrong because of opaque compiler internal state)"*

So: **(1) a clear understanding of MSVC's internals in service of decomp —
PRIMARY. (2) parity, a 100 % open-source implementation — a real end, and
additionally instrumental to (1).** `GOAL_DECISION`'s opening paragraph still
carries a struck *"ranked equally"* clause with an inline pointer to the section
that supersedes it; that strike is deliberate house style and is not a live
claim. **Do not quote the opening without the AMENDED section** — copying
"ranked equally" into a lane brief is itself a registered dispatch defect
(`docs/rungs/README.md` § "Standing facts", board #3370).

### 1.2 What follows, and it is the whole shape of this page

- **This is not a near-term shipping project in the 08-19 page's sense**, and
  that page said so itself. It is a compiler-backend reconstruction program.
  Milestones below are therefore **evidence-gated, not calendar-gated** (§5).
- **Characterization is a first-class deliverable, not overhead.** Under goal
  (1) a mechanised, addressed account of what c2's middle does *is* the
  product. `docs/whitebox/` is product. A characterization lane owes no
  conversion story and predicted reach 0 is not a mark against it
  (`docs/rungs/README.md` § "Lane kinds", promotion of 2026-08-21).
- **Coverage is goal (2)'s scoreboard and it is the hard one.** `match` →
  870/878. Partial coverage does not pay in proportion, and a wrong emit
  scores strictly below the refusal it replaced
  ([`PROGRESS_METRIC.md`](PROGRESS_METRIC.md) §0 bullet 4 and §4.2 — unchanged,
  and the reason the judge cannot carry a sliding score. **Cited as "§5.2" in
  three places in this tree, including that page's own §0 and its 2026-08-22
  banner; there is no §5.2 — the rule is §0's fourth bullet and §4.2's
  mismatch-zeroing guard. Not amended by this lane; reported as a dangling
  citation, and it is the class `board_audit.sh` cannot see, board #3367.**)
- **The verifier-throughput thesis is retired.** Throughput is a property. It
  may not justify a lane and — symmetrically, and this half is the one that
  gets forgotten — **it may not forbid one either.** Every throughput figure
  on the 08-19 page and on this one is carried as a *measurement*.
- **Read before probe** is standing doctrine
  ([`WHITEBOX_LEVERAGE_2026-08-21.md`](WHITEBOX_LEVERAGE_2026-08-21.md)): before
  a lane budgets a probe grid or a fitted-parameter search, price the binary
  read that would answer the same question and prefer it. The enumerated
  targets are [`whitebox/READ_PLAN_2026-08-21.md`](whitebox/READ_PLAN_2026-08-21.md)
  §3 (R1–R9).
- **General layers expose their decision surface.** From S1 on, a general layer
  ships allocation order, scheduling tie-breaks and label counters as **named,
  enumerable parameters whose DEFAULT reproduces c2 byte-exactly**, not as baked
  constants. A baked constant serves goal (2) only; a named decision point
  serves goal (2), the permuter and the training pipeline at the same
  correctness cost. **The judge grades the default and nothing else**; every
  non-default configuration is an instrument state and licenses no emit.

### 1.3 What the correctness rule still is — unchanged, and never in question

```text
normalize_timestamp(PortC2(IL, argv)) == normalize_timestamp(c2.dll(IL, argv))
```

The real `c2.dll` under wibo plus a byte-exact obj compare is the **sole**
judge. Understanding c2's internals is **not** a licence to grade the port
against c2's internal state; the stage oracle's snapshots are an instrument and
`ARCH_REVIEW`'s probe C measured that the port→tuple projection is undefined
anyway. Verification here is coverage-bounded differential testing: a green run
is sound only on the IL it ran against, never a total proof.

### 1.4 What is being implemented, and what is not

The target is the MSVC Xbox 360 **back-end DLL**, `c2.dll` from XDK
`16.00.11886.00`. The operative interface is a five-file IL bundle plus compiler
arguments in, and a PPC COFF object plus diagnostics and exit status out. This
does **not** implement the complete `cl.exe` source-to-object pipeline; that
would also require a native replacement for `c1xx.dll`, and `c1host` proves the
front-end replay seam without porting the front end.

---

## 2. Three meanings of 100 % — CARRIED FORWARD from the 08-19 page §4

**This is the most durable thing the 08-19 page contained and it survives the
goal decision intact.** It is reproduced here rather than rewritten, because
goal (2) is stated in its terms and because a project that ships one number
called "100 %" without saying which one is the one that will be misread.

### 2.1 Compatibility-product completeness

Every request is handled by the real pinned `c2.dll`, either directly or as the
fallback behind the native fast path. The service reproduces output bytes,
diagnostics and failure behavior and is validated against ordinary
spawn-per-request execution.

This provides 100 % **behavioral coverage through the vendor implementation**.
It is shippable, and it is not a vendor-free reimplementation. **It is not goal
(2), and it cannot become goal (2)** — see §7.

### 2.2 Native dc3 parity

`PortC2` produces a normalized byte-exact object for all **870** dc3 TUs that
the reference compiler successfully compiles, using their pinned real flags.

The target is 870, not 878: the remaining eight fail in the reference pipeline,
so no object exists for a byte-exact port to reproduce.

This is a meaningful milestone. **Its claim is limited to one game, one compiler
build and the observed configuration** — `870/870` on the pinned dc3 workload is
*native dc3 parity*, not proof of all possible `c2.dll` behavior.

**This is the sense in which goal (2) is currently scored**: `match` → 870/878
is the scoreboard, and it reads **26** today (§3).

### 2.3 General native backend parity

No `NotImplemented` remains inside a declared matrix containing at least:

- compiler DLL hash and host contract;
- supported IL bundle files and record versions;
- `/O1`, `/O2`, `/Ox`, `/Od` and function-level-linking behavior;
- EH, RTTI, generated symbols and static initialization;
- scalar PPC and VMX128 lowering;
- data, COMDAT, weak-external, relocation and section behavior;
- debug/listing output where in scope;
- failure results and diagnostics;
- PGO/LTCG explicitly included or explicitly excluded.

The declaration must be supported by pinned production corpora, independently
held-out projects and generated cross-mode tests. **Differential testing cannot
prove behavior outside that surface.**

### 2.4 Adopt the three as language

[`ARCHITECTURE_PROPOSAL_2026-08-20.md`](ARCHITECTURE_PROPOSAL_2026-08-20.md) §7
already recommends adopting these three as standing language so no future claim
of "done" is ambiguous. That recommendation is repeated here and is not
contingent on anything else on this page.

---

## 3. Where the program is — measurements, carried as measurements

Every figure below is kept because it was measured. **None of them justifies a
lane and none of them forbids one.** Quote them from a scan, never from this
page (the standing instruction in [`STATUS.md`](STATUS.md), which is the
one-page answer to "where is this project" and whose metric block is generated).

### 3.1 The scoreboard and its neighbours

From the generated block in `STATUS.md` (collected 2026-08-19, tree `977827d78`,
workload `49ad7cfd5`) — the same strategic position the 08-19 page reported at
its own reviewed tree:

| metric | value |
|---|---:|
| workload TUs | 878 |
| graded by the reference compiler | 870 |
| reference capture failures | 8 |
| whole-TU byte matches (`match`) | **26** |
| whole-TU mismatches | 0 |
| `vocab-gap` | 844 |
| `codegen-gap` | 0 |
| emitted-function census (driver, not target) | 39,332/162,063 (24.27 %) |
| per-function census (driver, not target) | 706,491/2,411,388 (29.30 %) |
| FUNCTION BYTE MATCH (driver, not target) | FBM 0.22145 |
| PROGRESS MASS (driver, not target) | P 0.21039 |

**`mismatch 0` is not evidence of correctness**, and this is the standing trap,
restated because it is the number most likely to be quoted as reassurance. Most
of the workload refuses before the emitter is consulted, so the scan cannot see
a codegen or binding defect in it. The demonstration is on record: board **#232**
was a live wrong emit on master for 255 commits while every gate read
`mismatch 0`, because no standing instrument could generate the shape. **A green
gate is a statement about the instruments, and widening the instruments is how
you find out which.**

### 3.2 The active wall, and what it is not

The first TU-level refusal is `gl-stop-26-introduced` for roughly 818 of the 844
vocabulary-gap TUs, and almost all refusal mass sits on the reader/model side
rather than the final emitter. **This is not one missing byte-width rule.** The
coupled problems are lossless parsing versus positive whole-function
recognition; `.gl` record traversal and body/name binding; the emit-set closure
and generated-symbol rules; typed expression and control-flow semantics; and
whether the resulting body can be lowered by a general backend. Removing the
first fence without implementing that chain changes the name of the next refusal
and risks admitting a wrong object.

**And the 818 has since been measured directly rather than argued**: lifting
that first refusal at three depths converts exactly **zero**, with an identity
control, and `body-out-of-class` — the first half of the integration row 4a — is
co-resident on **818 of 818** (board #3362,
[`ROADMAP_SLICING_2026-08-21.md`](ROADMAP_SLICING_2026-08-21.md)).

### 3.3 The architecture position

The native path is still predominantly a catalog of recognized shapes: the
central selector has about 38 success returns and dispatches to named lowerers
(`json_utf8_copy`, `xtea_encrypt_loop`, `pool_ctor_chain`, `guard_ret_chain`,
`ptr_walk_loop`). **These are valuable behavioral specimens and should remain as
regression tests. They are not a scalable architecture for arbitrary compiler
input**, and under §6 item 6 they may not remain production dispatch arms
indefinitely.

The block IR is a correct shared foundation, and its own contract records the
missing general mechanisms: code motion, scheduling, condition/value modeling,
cross-block liveness and the machine cost model.

**One thing nothing in this tree has ever measured, and it is the sharpest open
question about the position above:** whether the port's byte-exactness is *a
model or a fit*. `select_function` is never called for a parse-refused function,
so its 91.2 % is the catalogue graded against its own admission gate. The
concern is concrete rather than hypothetical — `codegen::alloc`'s clauses are
this repo's own fitted stand-in for c2's unread worklist order, and clause 2 was
refuted on 7 of 56 fresh-holdout cells under a preregistered
52,416-configuration search. **If the incumbent bytes are a fit, every "general"
layer is a re-fit.** §4.3 is where this question is currently parked.

### 3.4 Two structural findings that bound what can be promised

- **No near-miss TU population exists.** The 90–100 % FBM band is empty, hard
  floor 0.200 (board #3361). There is no bulge of nearly-matching TUs to
  harvest, and `PROGRESS_METRIC.md` §1 measured the same thing from the other
  side: the median failing TU has 80 % of its emitted functions out of class.
- **Composition is refuted four ways**, decisively because it is *anti-safe*
  under `PROGRESS_METRIC.md` (board #3363): shipping a 90 %-matching obj is a
  wrong emit, and the scale it would protect against was measured at **2,490
  wrong functions**.

### 3.5 Process, and one still-live release-hygiene defect

The repository holds hundreds of dated rung documents and roughly 20,000 lines
across `ROADMAP.md`, `BOARD.md`, `CEILING.md`, `STATUS.md` and the strategy
reviews. **This record has found real defects and should be retained.** Whether
it should remain the day-to-day backlog is §6 item 1, which is open.

**The rung-index locale defect the 08-19 page reported at §3.5 is still live at
this tree, re-measured for this page rather than carried on trust.**
`scripts/gen_rung_index.sh` consumes a shell glob and imposes no stable
collation order. Measured 2026-08-22 at `0636051e9`: the committed
`docs/rungs/INDEX.md` is byte-identical to the script's output under
`LC_ALL=en_US.UTF-8` and differs from its output under `LC_ALL=C` by **93 diff
lines**. So `rung_index_is_generated_and_current` is red for anyone whose
environment collates as `C`. It is a one-line fix (`LC_ALL=C` in the script plus
one regeneration) and it is deliberately **not** made by this lane, which is a
docs lane and would be changing a gate-visible generated file on a branch with
three peers in flight. Carried as an open item, §5 M0.

---

## 4. What is funded, what is open

### 4.1 FUNDED — the reads-first path, R1 → R2 → R3

**Owner-decided 2026-08-22** (`docs/DECISIONS_2026-08-22.md` decision 1, board
**#3371**): *"#1 funded for option 4"*. Option 4 is the fourth branch that the
2026-08-22 survey added to `ARCHITECTURE_PROPOSAL` §8 decision 0 — **fund the
read-plan's R1→R2→R3 first (≈5–9 days), then decide.**

| read | subject | replaces / de-risks | price |
|---|---|---|---:|
| **R1** | `DAT_10c400d4`'s scope — `0x10b54d32` (mint), `0x10b2c1f1` (table clear), `0x10b2c21d` (hash) | a 1–2 wk black-box settle; soundness of `select_function`'s no-TU-context signature; **whether the ten refuted alloc keys have an explanation at all** | **0.5 d** |
| **R2** | the encoder — two tables (`0x10c3a578`, `0x10c39b18`) plus the **79 arms** off the jump table at `0x10bfae2d`, all inside `FUN_10bf9f15`'s 3,861 B | **I2 general lowering**, whose black-box price is 1.5–4.5 eng-mo raw / 7.5–22.5 calibrated | **2–4 d** |
| **R3** | the label charge — `FUN_10b97dd0`'s 31 sites, `FUN_10b9a455`'s 132, the formatter's switch | the *"label plan is not derivable"* premise; four lanes' wrong-instrument stride measurements | **2–4 d** |

R2 proves the arm-reading method on 79 bounded bodies **before** R5 spends
15–25 days on 189. The sum of all nine ranked reads is **≈6–10 engineer-weeks**.

Three things this funding does **not** do, stated because the decision record
states them: it approves no row of 4a, it re-prices nothing, and it waives no
NO-GO clause. And the standing caveat that keeps a read honest: **`[R]` means
"the instructions were read correctly", not "this is what c2 does"** — the
`.bss` bump rule was read correctly out of a clean function and was wrong about
c2. **Every read lane still ends in a confirmation probe**, and
`docs/whitebox/DISCLOSURE.md`'s adoption rule is unchanged.

### 4.2 OPEN — `ARCHITECTURE_PROPOSAL` §8 decision 0's branch choice

**Explicitly deferred until R1–R3 report.** The three branches, as that section
poses them:

- **(a) approve rows 4a + 4b.** 4a is the integration prerequisite — a general
  op-level IL decode and a general lowering to `coff::Function` — priced
  **15–45 engineer-months as a lower bound**. 4b gives IR3 its own step,
  defined in c2's tuple/region coordinates, and is what makes a per-stage grade
  for a port *definable at all*.
- **(b) declare step 5 characterization-only** — in writing, as a program with
  no path to the byte judge.
- **(c) run the 8-week Phase 0 first** — S0 (blind-reach measurement) and S1 (a
  general `Plain`+`Tail` lowering as a required-zero re-expression on the live
  dispatcher), which decides (a) vs (b) on evidence and banks a standing
  instrument either way. **One of its outcomes should stop the program**: if S1
  holds its required-zero delta but workload `fnbyte-exact` moves at all, the
  pricing basis is void.

**Goal (2) is unreachable without 4a** — a ported pass with no route to
`coff::Function` cannot move one obj byte. **Goal (1) is served by
characterization alone.** That is the shape of the choice, and it is the
owner's.

Whether R1→R2→R3 is a cheaper Phase 0, a prerequisite *to* Phase 0, or an
orthogonal third thing is a scheduling judgement **no lane has made and this
page does not make either**. What is known: the reads produce the spec 4a would
be built from, and **a read produces a spec, not an implementation** — quoting
the reads as a reduction in 4a's price would be exactly the units error the
read-plan warns about, and `CEILING` §5's ~5:1 calibration was fitted on
lane-shaped construction work and must not be applied to a read.

### 4.3 OPEN — the two named consumers, and the measurement one of them wants first

`GOAL_DECISION` § "AMENDED" names two downstream consumers of goal (2), and
lanes may be priced against them:

- **Training data for AI models that reverse the compiler** — producing a
  *matching pretext*: source that recompiles to the target bytes. The port
  supplies what the binary cannot: aligned `(IL, internal state, bytes)` triples
  at every pipeline stage, in unlimited volume. This is what the decision-surface
  rule in §1.2 exists to keep possible.
- **A better permuter** — when candidate code is close but wrong because of
  opaque internal compiler state, a search over the port's exposed decision
  points can find the configuration that lands the bytes. The repo has already
  run ad-hoc versions: the 52,416- and 13,104-configuration searches behind
  `codegen::alloc`/`schedule` **are** permuter runs against fitted constants.

> **THE PERMUTER WANTS A POPULATION MEASUREMENT BEFORE A BUILD, and this is
> recorded as the coordinator's recommendation rather than an owner decision**
> (`docs/DECISIONS_2026-08-22.md` § "Recorded from the same brief, not
> owner-decided"; board **#3369**).
>
> The measured wrong-body population in this repo is **one mechanism**: c2
> inlined a callee where the port emitted a call. `docs/DIFF_STRUCTURE.md`
> reads **0 pure reorderings** — the permutation→scheduling class is *empty* —
> **2** field-only words and **2** immediate-only words in **5,189** substituted
> words, **5,173 (99.7 %) differing in opcode**, and **94.3 % of bodies wrong at
> word 0**. A fitness gradient over allocation or scheduling would point at
> nothing.
>
> **But that is the *port's own* failure population, and the owner's permuter
> case is hand-written decomp near-misses — a different population that nothing
> in this tree has measured against it.** One day of measuring whether decomp
> near-misses are also inlining-dominated decides which permuter to build;
> building first risks weeks on the wrong search space.
> `crates/c2-core/src/splice.rs:57-60`'s **0.9716** inline cost model, with its
> **2.84 %** NOT-MODELLED residual that no emitter consults, is the candidate
> knob if inlining dominates.
>
> One prerequisite, also recorded and also cheap: **`DIFF_STRUCTURE.md` needs a
> rescan, not an edit** — its numbers are from tree `0c8a185` (3,195 wrong
> bodies against today's 1,960 + 530) and its own banner marks §3.2 refuted.

---

## 5. Milestones — evidence-gated, not calendar-gated

> **Why there are no dates on this page, and it is a measurement rather than
> caution.** `CEILING.md` §5 calibrates every forward figure in this repo
> against board #770's prereg record: **optimism dominates roughly 5:1** (ten or
> eleven optimistic misses against one or two pessimistic), **the misses are
> specifically on forward cost** — frontier depth, refusal counts, rung counts —
> and not on measurement, where the same preregs routinely hit known-answer
> controls to the digit. **Therefore: read every forward cost figure here as a
> LOWER BOUND.** `CEILING` publishes no point estimate of when TU match reaches
> 871 and registers that refusal deliberately; this page inherits it. The 08-19
> page's *"indicative duration"* lines are not carried forward.

Each milestone below is therefore stated as **an exit gate that can be observed
to have fired**, with its dependencies named. They are partially ordered, not
sequential: M0 and the R-reads are independent of each other, and M-CHAR runs
continuously.

### M0 — Reproducible baseline

*Carried forward from the 08-19 page and narrowed to what is still open.*

- fix deterministic rung-index generation (§3.5 — measured still live at
  `0636051e9`, 93 diff lines between `C` and `en_US.UTF-8`);
- pin compiler, wibo, workload revision, environment and exact flag matrix;
- regenerate `STATUS.md` from the pinned state (`scripts/status.sh --write` —
  the block is generated and never hand-edited);
- state the three completion claims of §2 in user-facing language;
- one release command and one machine-readable result manifest.

**Exit gate:** a clean clone produces the same generated files and metrics under
supported locales; the complete release test command passes; the worktree and
workload identities are recorded in the report.

**Why it is still first even without a product track:** every number this
program reports is read off a tree, and `fnbyte-*` is already known not to be a
pure function of the commit — it reads (commit × capture-cache state ×
untracked workload). A baseline that does not pin those is a baseline that
cannot be re-read.

### M-CHAR — Characterization (continuous, and it is a deliverable)

This is goal (1)'s output and it does not wait on any decision below. It is the
funded work today (§4.1) and it is graded the way characterization lanes are
graded: prereg frozen before the first probe, every load-bearing claim citing an
address or a grid cell, a `DISCLOSURE.md` row in the same commit that adopts a
disassembly-derived constant into `crates/`, and a confirmation probe at the
end.

**Exit gate for each read**, not for the milestone as a whole — the milestone
has no terminal state and is not supposed to:

- the read's named spec artifact exists under `docs/whitebox/` in the shape
  `READ_PLAN` §4 specifies (R2 → `ref/P_ENCODE.md`, R5 → `ref/P_ILRECORD.md`,
  R3 → an amendment to `LABEL_COUNTER.md`);
- a confirmation probe was run and is reported, including if it refuted the
  read;
- a `DISCLOSURE.md` row names the address;
- the roadmap premise the read was ranked against is explicitly **confirmed or
  refuted** in the rung.

### M-ORACLE — Intermediate-state reference oracle — **BUILT, and re-priced**

The 08-19 page called this M2 and *"the go/no-go milestone for renewed native
investment"*. **Two things have happened to it.**

1. **It has been built and graded** (`docs/rungs/2026-08-20-stageoracle.md`,
   board #3322) and re-priced (`docs/STEP5_PRICING_2026-08-21.md`). Its
   stop-condition turned out to need a third state — the fire is a *partial*
   fire, and the fire line runs between c2's side and the port's rather than
   between boundaries (`ARCHITECTURE_PROPOSAL` §4.2).
2. **Its framing is inverted by the goal decision.** `STEP5_PRICING`'s headline
   — *"per-stage observability buys CHARACTERIZATION, not a per-stage
   differential grade"* — read as a downgrade under the retired thesis. Under
   goal (1) it is a direct hit: the snapshots and the whitebox record are **the
   deliverable**, not the gate in front of one.

**Its standing bound is unchanged and is what keeps it an instrument:** snapshot
equality is not a grade, no `plan-*` or snapshot key gates an emit, and the byte
judge is untouched.

### M-READER / M-PLANNER — Lossless IL + symbol frontend, and the exact object planner

*Carried forward from the 08-19 page's M3 and M4, whose content stands.* They
are stated together because symbol parsing and object planning share facts;
their acceptance reports stay separate **so a good writer cannot hide a bad
reader**.

M-READER deliverables: decode every record in all five bundle streams without
requiring a known whole-function shape; retain unknown semantics as typed or
opaque nodes with their exact source bytes and boundaries; separate framing,
binding, semantic understanding and lowering results; reproduce `.gl` traversal,
body/name binding and generated-symbol handling; port the emit-set reachability
closure from the real implementation rather than reviving the invalid fitted
predicate.

**M-READER exit gate:** every one of the 870 graded dc3 bundles is framed
losslessly; zero functions are refused merely because a complete function/body
boundary cannot be represented; decoded function, symbol and binding inventories
match the stage oracle; malformed input behavior is covered by negative tests.

M-PLANNER deliverables: function and section emission order; packed and `/Gy`
COMDAT layouts; weak externals and default-symbol recursion; section selection,
associative relationships and checksums; relocation inventory and addends;
`.data`, `.bss`, `.rdata`, debug and directive-section planning; EH region and
record planning independent of final instruction bytes.

**M-PLANNER exit gate:** structural object manifests match the reference on all
870 graded TUs — section order and attributes, symbol order and fields,
relocation locations and types, COMDAT associations and EH record inventory;
remaining differences are attributable to body bytes or explicitly deferred
content encoders.

> **The instrument for this exists now and carries its own trap.** `plan-*`
> (lane `w-objplan`, `STATUS.md` trap 0b) grades everything about the output obj
> that is independent of the instruction bytes, on all 870 graded TUs including
> the 844 that produce no IR at all. **It is necessary and not sufficient**: a
> TU can be plan-exact on every component and mismatch on every instruction
> byte. **Read the three denominators, never the ratio** — `observable ⊇ known ⊇
> exact` — and note that `emitset-members` and `emitset-order` both currently
> read `known 0` because their named control is red, which is the ship rule and
> not a regression.

### M-IR — General typed IR and scalar backend

*Carried forward from M5.* Typed values and operations rather than
whole-function shape fields; explicit CFG, block order, terminators and
condition producers; ABI-aware call, return, frame, stack and aggregate
handling; general scalar instruction selection; dataflow, liveness and
cross-block value tracking; the optimizer passes the pinned workload requires,
in observed order; dependence-DAG scheduling and the Xenon machine model;
register allocation across branches and backedges.

The existing named lowerers become regression fixtures and byte-exact witnesses
and progressively stop being production dispatch arms as their semantics move
into the general pipeline.

**Exit gates are byte-weighted:** 25 %, 50 %, 75 %, 90 %, 99 %, then 100 %
function bytes exact; relocation-correct and relocation-unknown populations
reported separately; **no wrong whole-object emit may replace a refusal**; every
threshold validated on DEV and then checked **once** on HELDOUT.

**Amended for the decision surface** (§1.2): every layer landed here ships its
arbitrary choices as named, settable parameters whose default reproduces c2, and
is graded **at the default and nowhere else**. A layer that bakes a fitted
constant owes a pointer to the read that would replace it (`READ_PLAN` §2 is the
index).

### M-DC3 — Native dc3 parity (this is goal (2)'s scoreboard reaching its target)

*Carried forward from M6.* EH and unwind state; RTTI and read-only-data COMDAT
families; static initialization and compiler-generated functions; VMX128; all
observed optimization modes and pragma mode changes; debug and listing data;
exact diagnostics and failure behavior where claimed.

**Exit gate:** `PortC2` is normalized-byte-exact on **870/870** graded dc3 TUs;
zero `NotImplemented` within the dc3 contract; zero mismatch on the held-out
project corpus and generated cross-mode suite; real-TU native performance is
reported — fixture microbenchmarks are not a substitute.

### M-GENERAL — Declared-surface general parity

*Carried forward from M7.* Expand the declared surface one feature/mode family
at a time. Each expansion includes a corpus, held-out cases, reference snapshots
and final object comparison. PGO, LTCG, unusual debug/listing modes and
malformed-input parity are **separate** release criteria, not implied by dc3
parity.

**Exit gate:** the support matrix has no unknown or untested cell advertised as
supported; the vendor fallback can be disabled for the declared surface without
changing results; packaging, API stability and consumer integration are ready.

### 5.1 PROPOSAL — what a "shippable" milestone even is for a reconstruction program

**The owner has not decided this and this page does not decide it.** The 08-19
page had an easy answer because its headline was a service; with the service
subordinate (§7), the question is genuinely open, and it matters because
"shippable" is what an outside reader will look for. Three candidate answers,
with what each costs and what each would mislead about:

- **(A) Nothing ships until M-DC3.** Honest and simple; goal (2)'s scoreboard is
  binary and this is the only definition that matches it exactly. Cost: the
  program has no externally legible output for a very long time, and §5's
  calibration says nobody may say how long.
- **(B) The characterization record is the shipping artifact.** Each read lands
  a spec under `docs/whitebox/` that stands on its own and is directly
  consumable by decomp work — which is goal (1), the primary goal, so this is
  the definition most aligned with the ranking. Cost: it is a documentation
  release, and it must not be allowed to read as a claim about the port. The
  discipline that keeps it honest already exists (prereg, address citations,
  confirmation probes, `DISCLOSURE.md`).
- **(C) The port ships as an instrument before it ships as a compiler** —
  released for the two named consumers (§4.3) at whatever coverage it has, with
  its refusal boundary as the advertised interface. This is the definition goal
  (2)'s *instrumental* half suggests. Cost: it needs the decision surface to
  exist first (§1.2, S1 onward), and it needs the permuter population
  measurement (§4.3) to know whether anyone can use it; shipping it earlier
  would advertise a search space that `DIFF_STRUCTURE` says may point at
  nothing.

**These are not exclusive** — (B) is already happening under M-CHAR, and (C)
becomes available once S1-style layers exist. What needs deciding is which one
the program *reports against*, because that is what sets what a lane is allowed
to call done. §9 item 3.

---

## 6. Operating model

### 6.1 Items 3–6 — CARRIED FORWARD unchanged

The 08-19 page's items 3, 4, 5 and 6 are about **honest measurement of
reproduction**. They stand on their own, they are unaffected by the goal
decision, and several are strengthened by it.

3. **Stop using whole-TU match as the only progress view.** Publish separate
   metrics for lossless decode, binding, object planning, IR translation,
   codegen, relocation correctness and final byte match. *(Strengthened: this is
   most of what the `plan-*` family and FBM now do, and under goal (1) a
   per-stage account is a deliverable in itself. What it may never become is a
   gate — every gradient goes beside the judge, never inside it,
   `FUNCTION_BYTE_MATCH.md` §0 being the standing template.)*
4. **Use byte-weighted coverage as the main native-progress curve.** Historical
   measurement: function counts can overstate byte coverage by roughly 9×.
   *(Carried with its own live caveat: `fnbyte-*` denominators are 71.2 %
   bodies the shipped image never contains — `/Gy` COMDATs the linker discards
   — so a `fnbyte` ratio is progress over what c2 emits, never over the game.)*
5. **Freeze a DEV corpus and preserve a genuinely unseen HELDOUT corpus.** Do
   not repeatedly fit on the final acceptance population. *(Strengthened, and
   §3.3's model-or-fit question is exactly why. `w-sizebracket`'s finding is
   the sharpest argument for it in the tree: a predicate can be 39.6 % wrong
   about c2 and **free** in the metric used to choose it, and a split-half
   validation is blind by construction because both halves share the blind
   spot.)*
6. **Ban production dispatch keyed to source paths, symbol identities or named
   application functions.** Specialized fixtures may remain; specialized
   compiler paths do not count toward the general architecture. *(Unchanged,
   and §3.3 is what it binds.)*

### 6.2 Item 2 — carried, and already in force

*"Permit a new fixture/rung only when it is a minimized regression, answers a
pre-registered behavior question, or advances a named phase exit gate."* This is
substantially what `docs/rungs/README.md`'s three lane kinds already enforce,
plus the one-word `Outcome:` field. The one amendment the goal decision makes:
**a characterization lane satisfies the middle clause on its own and owes no
conversion story.**

### 6.3 Item 7 — carried

*"Require a phase to end in one of `complete`, `failed` or `stopped by
decision`. An instrument or document is an input to a phase, not completion of
it."* Carried with one reconciliation: under goal (1) a characterization lane's
**document is its deliverable**, and that is not a contradiction — the lane
completes, the *phase* does not. The rung `Outcome:` vocabulary already draws
this line (`built` for a lane that landed what it preregistered; `FAILED`, in
that word, for one that did not).

### 6.4 Item 1 — **OPEN, and deliberately not resolved here**

*"Replace the active board with 6–8 subsystem epics. Keep dated rungs as a
research archive and provenance record."*

**This is a live question and this page does not settle it.** It is carried
forward as open, exactly as the 2026-08-21 annotation on the 08-19 page left
it, because it is a governance change with costs measured on both sides:

- **For:** §3.5's volume argument is real, and the 08-19 page's reading — that
  the record makes current commitments, superseded measurements and actual
  completion gates hard to distinguish — is corroborated repeatedly in the tree
  (a coordinator dispatched phase work off a `CEILING` §6.1 prose row that a
  scan read as converting **zero**).
- **Against:** the board is load-bearing machinery, not just prose.
  `scripts/board_audit.sh` checks it; `#N` citations run through `ROADMAP.md`,
  every rung and much of `docs/`; and MEMORY records *"check the board before
  dispatching"* as the thing that stops re-measuring an answered question — five
  rows re-entered a ranking after already measuring zero. **Retiring it without
  a replacement for the "has this been measured already" query would remove a
  check nothing else performs.**
- **And the cost of moving anything is measured**: `file.md:NNN` citations are
  load-bearing across this tree and `board_audit.sh` **cannot see cross-doc
  staleness** (board #3367's consequence; the detector lane named at the
  `w-goaldocs` merge is still unbuilt). Any restructuring prices citation
  breakage before it moves a file.

The docs-structure lane the owner ordered the same day
(`DECISIONS_2026-08-22.md` decision 4) is scoped as **navigation**, not
retirement, for exactly this reason. §9 item 4.

---

## 7. The vendor-DLL-backed service — an explicitly subordinate option

**It appears here only as an option, and only as a subordinate one.** It was the
08-19 page's headline; it is not this page's, and the reason is not that it was
measured wrong.

### 7.1 Its two disqualifications, stated

1. **It moves the parity scoreboard by zero.** Goal (2) is a 100 %
   *open-source* implementation, scored `match` → 870/878. A service that keeps
   the real pinned `c2.dll` resident and answers every request from it adds
   exactly nothing to that number, and it **depends on the binary parity is
   defined as replacing**. In §2's language it is 2.1, and 2.1 is not 2.2.
2. **It is the opaque binary, so it emits none of the signals goal (2) is
   valued for.** The ranking's second half is that the port is *"actual code we
   can tweak to instrument + help produce signals about the compiler's state"* —
   for training models to reverse the compiler and for a permuter over exposed
   decision points. **A vendor-backed service cannot be made to emit those.**
   It moves the parity scoreboard by zero *and* the instrument story by zero.

**And a third thing it may not be ranked on, in either direction:** throughput.
The verifier-throughput thesis is retired, so the service may not be ranked
ahead of native work *because* it is faster to ship or faster to run — and it
may not be declined on throughput grounds either. **The measurements below are
kept as measurements.**

### 7.2 The measurements, carried forward as measurements

From the fork-server prototype (`docs/rungs/2026-08-04-w-fork.md`) and the 08-19
page §5:

- post-`LoadLibrary` fork state is safe for `c2.dll` on the tested population;
- **10,580 comparisons produced zero differing objects**;
- small IL bundles improve by **2.76×**; large dc3 TUs by **1.01×**, because
  their time is real compiler work; the mixed capture population weighted out at
  **1.44×**;
- for a native-hit fraction `p`, the idealized c2-stage speedup of a
  native-first / vendor-fallback hybrid is approximately `1/(1−p)`: at 26 of 870
  that is about **1.03×**; 2× needs ~50 % native coverage, 10× needs ~90 %;
- the source pipeline pays roughly **25 ms** of fixed work, and `c1host` already
  invokes `c1xx.dll` standalone — but **a state-safety probe against `c1xx.dll`
  is required before committing to that route and the speedup has not been
  demonstrated**; it must not be promised before the probe;
- **static recompilation of `c2.dll` is rejected on a mechanism, not a
  preference**: it is a 32-bit x86 PE and wibo is a loader, not an ISA emulator,
  so the host CPU already executes the compiler's instructions natively. There
  is no interpretation layer to remove, and x86 PE → x86 ELF introduces TLS,
  exception, indirect-dispatch and ABI risk without addressing compiler-work
  cost.

**Two later measurements cut against the service's supporting case and belong
next to the ones above:**

- **The dependency the 08-19 page assumed runs the other way.** M2 was said to
  share M1's resident-c2 plumbing. `w-stageoracle` declined residency **in
  writing** — *"one process per compile is precisely what makes snapshot
  determinism testable"* — so building residency "for free" would have been
  building the thing most likely to break the oracle's load-bearing property.
  What a future service reuses is **already built**; what it adds is a loop and
  a socket, and it should be graded **against the oracle's snapshots** as its
  regression fence (`ARCHITECTURE_PROPOSAL` §7.1).
- **Spawn was never the lever.** Counted rather than inferred (board #3262):
  `c2rs` process startup is **under 1 ms**, and the expensive spawn is `cl.exe`
  under wibo, one in six. **A fork server buys under 2 %** on that path.

### 7.3 So what remains true of it

It is **product architecture, not compiler architecture**. If a consumer exists
who needs exact `c2` behavior at volume today, the service is the cheapest way
to give it to them, its exactness evidence is real, and the hybrid remains the
right *shape* for such a product because it preserves exact coverage while
native support grows. **Ship it as a product decision on its own merits; do not
let it reorder the reconstruction program, and do not let the reconstruction
program block it.** What it may not do is stand in for goal (2) or be reported
as progress toward it.

---

## 8. Cost and staffing — lower bounds, and nothing else

**Every figure in this section is a LOWER BOUND** (§5's calibration box). They
are program-sizing ranges, not delivery promises, and they carry no dates.

| deliverable | figure | provenance and what it is a bound on |
|---|---:|---|
| the nine ranked reads (R1–R9) | **≈6–10 engineer-weeks** | `READ_PLAN` §3, priced per read against a named black-box row it displaces. R1→R2→R3 alone is **≈5–9 days** |
| Phase 0 (S0 + S1) | **8 weeks** | `ROADMAP_SLICING` §5. Decides §4.2 (a) vs (b) on evidence and banks an instrument either way |
| row 4a (integration: general IL decode + general lowering to `coff::Function`) | **15–45 engineer-months** | `STEP5_PRICING` §2/§4, top-down, explicitly a lower bound. **The critical path for goal (2)** |
| the same work enumerated bottom-up | **31–59 engineer-months** | `ROADMAP_SLICING`, `CEILING` §5's ~5:1 applied per row. **The direction is the finding**: slicing does **not** shorten it |
| broad general c2 parity (§2.3) | **potentially multiple engineer-years** | 08-19 page §8, carried |

**Two things about these numbers that are more useful than the numbers.**

- **Slicing was tested as a shortcut and refuted.** Ten constructs carrying
  **97.3 %** of the residue collapsed into two rows, and super-additivity turned
  out to be an overhead *of* slicing rather than a saving (board #3365).
  Alongside the two other refuted shortcuts (§3.2's 818-TU lift converting zero;
  §3.4's empty near-miss band and anti-safe composition), **all three candidate
  ways to make this cheaper have been measured and none of them works.**
- **4a's price rests on exactly two rows, I1 and I2, and each now has a named,
  addressed, sized, mechanical read** (R5 and R2). What a read removes is the
  *discovery* cost those estimates carried implicitly. **It does not lower the
  implementation price**, and saying it does would be a units error.

If multiple engineers are available, the useful split is **by subsystem, not by
fixture**: reader/types/symbol binding; object planning, COMDAT, EH and debug
records; typed IR, optimizer and lowering; reference instrumentation, corpora
and release infrastructure.

---

## 9. Decisions

### Already decided by the owner — recorded, not requested

- **The goal and its ranking** (2026-08-21, `GOAL_DECISION_2026-08-21.md`
  including § "AMENDED"). Quoted at §1.1.
- **The reads are funded** (2026-08-22, `DECISIONS_2026-08-22.md` decision 1):
  R1→R2→R3 before the §4.2 branch choice. §4.1.
- **This rewrite** (2026-08-22, decision 2). This page.

### Open — posed as proposals with alternatives, in the order they bind

1. **`ARCHITECTURE_PROPOSAL` §8 decision 0's branch: (a) approve 4a + 4b, (b)
   declare step 5 characterization-only, or (c) run the 8-week Phase 0.**
   Deferred by the owner until R1–R3 report; §4.2 states what each branch
   claims and what goal (1) and goal (2) each need. **Nothing on this page
   recommends one.**
2. **Whether to spend one day measuring the permuter's real population before
   building a permuter.** §4.3. The coordinator recommends yes; it is not an
   owner decision yet. It carries a cheap prerequisite (`DIFF_STRUCTURE.md`
   wants a rescan, not an edit).
3. **What the program reports "shippable" against**: §5.1's (A) nothing until
   M-DC3, (B) the characterization record, or (C) the port as an instrument for
   the two named consumers. This page presents three and picks none.
4. **Operating-model item 1 — whether the active board is retired in favour of
   subsystem epics.** §6.4, carried forward as **open** with the costs on both
   sides stated. Note the hard constraint any restructuring inherits:
   `file.md:NNN` citations are load-bearing and `board_audit.sh` cannot see
   cross-doc staleness, so moves are priced before they are made.
5. **Whether the vendor-DLL service is built at all, as a subordinate product.**
   §7. If it is, it is reported under §2.1's definition of 100 % and never under
   §2.2's, and it is graded against the stage oracle's snapshots.

### Adopt without further review (they were never contested)

- **§2's three definitions of 100 % as standing language.**
- **§6.1's items 3–6.**

---

## 10. Immediate next actions

In dependency order, and each one is either already funded or is cheap enough
that its cost is not the question:

1. **Run R1, R2, R3** and land their specs under `docs/whitebox/` with
   confirmation probes and `DISCLOSURE.md` rows (funded, §4.1).
2. **Fix deterministic rung-index generation** (§3.5 — one line plus one
   regeneration; measured still live at `0636051e9`).
3. **Rescan `DIFF_STRUCTURE.md`** — a rescan, not an edit (§4.3).
4. **Measure whether hand-written decomp near-misses are inlining-dominated**,
   which is the one-day input to decision 2 above.
5. **Regenerate `STATUS.md` from a pinned state** and record the workload stamp
   and capture-cache state with it (§5 M0).
6. **Report R1–R3 into `ARCHITECTURE_PROPOSAL` §8 decision 0** — the reads exist
   to be read against that choice, and the choice is the owner's.
