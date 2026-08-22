# Architecture review — 2026-08-21

> **AMENDED 2026-08-21 by lane `w-restim` (merge `b6fd2bf48`), which this
> review dispatched.** Finding 1 moves in BOTH directions and the reader
> should take the amendment, not the original:
>
> - **COLOR *is* gradeable against real c2 — finding 1's headline is
>   overturned.** Its output is the operand's symbol pointer being re-pointed
>   from a candidate record to a physical-register record, readable directly
>   (candidate id 2 → `r3`); **771 of 2,946** function-pairs move there while
>   the tuple spine is byte-identical. 85.6% of COLOR's visible footprint was
>   invisible to every prior measurement, including this review's. Board
>   #3323 reproduces exactly and is unmoved — it never generalised, because
>   its fixture is one where COLOR is a no-op 7/7.
> - **The final schedule is gradeable too** — new site `0x10b7e701`, run 4
>   moves the list on 149 of 2,946 functions.
> - **What survives, and is now THE binding constraint: a PORT still cannot
>   be graded per-stage.** Probe C measured it — on a 4/4 byte-exact fixture
>   c2 carries 19–22 tuples and 29 regions where the port emits 4–5
>   instructions. The two have **no common coordinate**; the projection is
>   *undefined*, not merely unequal. That is finding 4's F1 (the unnamed,
>   ungraded port→tuple translation layer) turned into a measured ratio.
> - **Net:** per-stage observability buys **characterization, not a
>   per-stage differential grade**. See `docs/STEP5_PRICING_2026-08-21.md`,
>   which applies CEILING §5's ~5:1 calibration per row and prices the two
>   integration prerequisites at **15–45 engineer-months** as a lower bound.
>
> This is what the review was for: it was dispatched to be tested, and one
> of its load-bearing findings did not survive contact with a measurement.

Seven independent review lenses over `docs/ARCHITECTURE_PROPOSAL_2026-08-20.md`,
run after steps 0–4 of its §5 migration landed (master `d7be7aadc`). Every
reviewer was read-only, forbidden to edit or merge, told that "no defect found"
is a valid answer, and required to mark each claim READ (found in a doc) vs
MEASURED (verified against code/artifacts). The coordinator independently
re-verified the load-bearing measured claims marked ✅ below before adopting
them; claims marked (reviewer) rest on the reviewer's own measurement, each of
which shipped with a control or an independent reproduction.

**Verdict: the diagnosis is right, the migration discipline worked, and steps
0–4 are honest, valuable scaffolding — but step 5 must not be dispatched with
the proposal as its authority.** The proposal is stale against the measurements
of the very lanes it commissioned, step 5's grading premise fails for both of
its named targets, two hard prerequisites appear in no row and no budget, IR3
is specified in a coordinate system c2's middle does not use, and the strategic
justification is split between two goals whose choice is the owner's and has
~~been unowned since 2026-08-13~~ **— OWNED AND ANSWERED THE SAME DAY THIS
REVIEW LANDED: the goal is full reproduction, for understanding MSVC's
internals and for parity ([`GOAL_DECISION_2026-08-21.md`](GOAL_DECISION_2026-08-21.md)).
Read §7 with its banner: the split collapses onto §7's second bullet, and the
first bullet's economics are *superseded, not satisfied.* The other four
defects in this verdict are untouched by that and still stand.**

## Convergent findings

Ordered by how many independent lenses reached them.

### 1. Step 5's per-stage grading premise fails for COLOR and the final schedule
*(four lenses independently: gradeability, history, drift, IR3)*

- The stage oracle observes state at exactly one site (the region finder); the
  `color` and `globregs` sites contribute hit counts and zero bytes of state.
  ✅ `c2host/stagetap.c:369`.
- Measured over 384 fixtures / 2,949 functions (reviewer, with controls): the
  COLOR bracket is trivially-equal on 75.0% of functions; board #3323's
  "IDENTICAL 7/7" does not generalize — 736/2,949 pairs differ, and 649 of
  those are *length* changes: COLOR's visible footprint is tuple deletion
  (coalescing), never register assignment.
- The final schedule (`sched0`, the run that fixes emitted instruction order)
  has its output observed **nowhere** — walks fire at region-finder entry, and
  run 4 has no successor run.
- 65.1% of the payload is suffix re-reads (walk terminates on `next == 0`, end
  of list not end of region ✅ `stagetap.c:343`); 58.7% of rows are
  indistinguishable from a same-region sibling (no operands in the row).
- Live wrong-but-green instance: the raw-window verdict skips length-mismatched
  functions and tests only `hot.is_empty()`, so it prints "offsets COLOR
  writes: NONE — the allocator wrote nothing in this window" over zero
  comparisons. ✅ `crates/c2-harness/src/cli/stage.rs:624,638`. That is the
  sentence #3323 rests on (the #3323 instance itself had 83 real aligned
  pairs, so the board row is not vacuous — the *instrument* is).
- Because walks are interleaved mid-pass, matching the stream at all may
  require a port to reproduce c2's region decomposition and per-region relink
  schedule — whether a port could ever emit this stream is unmeasured. The
  deciding probe is cheap: have `PortC2` emit a region-boundary trace on one
  already-byte-exact fixture and diff it against the tap's.

### 2. Three of the four staged IRs have zero production callers; IR3 does not exist
*(wiring by compiler-checked amputation; drift by caller sweep — same answer)*

- Deleting `c2-il/src/stream/` (IR0), `c2-core/src/plan/` (IR2) and
  `c2-core/src/passes/` from a scratch copy of master builds clean and passes
  1,230 unit tests; a control amputation of `block_ir.rs` correctly fails with
  14 errors (reviewer). Only IR1 (`bind.rs`) is on the emit path — and step
  4's own delta was instrument-scoped (census + diag), which its commit says.
  ✅ `Bindings::census` has exactly two non-test callers.
- IR2 is not merely unwired, it is *fenced off* the emit path by its own
  BANNED-list test (`plan/mod.rs:483`) — wiring it in requires deleting an
  invariant that exists for a documented reason (#3237).
- The IR0 production switch survives in **no commit** — the "reverted" commit
  is 47 doc-comment lines ✅ `git show 09d42b15f`. Redoing it starts from zero
  across eleven call sites.
- `passes/mod.rs` is a 30-line comment ✅; `sym/`, `sem/`, `mir/`, `lower/`,
  `emit/` do not exist ✅. §3.3's claims ledger is unbuilt and no step builds
  it, yet step 5 admits work "behind claim 4".
- None of this is concealed — the source *declares* each layer's inertness.
  This is honest scaffolding, not the wrong-but-green family. The defect is in
  the plan's shape, not the work.

### 3. Step 5 has two unbudgeted prerequisites on its critical path *(wiring)*

For a ported pass to reach the byte judge it needs (a) a **general op-level IL
decode** — IR0 stops at a two-variant byte framing, `BodyShape` starts at 35
whole-function grammars that are simultaneously the admission gate; the
semantic middle a COLOR pass would consume is missing — and (b) a **general
lowering to `coff::Function`** — today a 35-arm per-shape dispatch
(`comdat.rs`, 43 `Selected::` refs). Reviewer's estimate 3–9 engineer-months
*before* the ~5:1 optimism calibration, in no §5 row. Without it, a step-5
program's only progress signal is stage-parity — an instrument with no
emit-path consumer, i.e. **#3336 at program scale**, with no contrast case to
catch it. Cheap prophylactic adopted below: every step-5 lane names in its
rung header which `coff::Function` field its pass would eventually write.

### 4. IR3 is specified in the wrong coordinate system *(IR3 lens)*

The proposal contains **zero** occurrences of "tuple" and zero of "region"
✅ — the words that name c2's actual IR (`WB_REGALLOC_FINDINGS.md`: a tuple
list, not a tree) and the scheduler's unit of work. Consequences, each making
the per-stage grade ill-defined by a different route: (a) the port→tuple
projection is unnamed/ungraded/unowned; (b) c2's opcode numbering (`li`=624 vs
`addi`=11) is already erased by the port's encoded-byte blocks, so the map is
not a function; (c) regions cut through shipped block boundaries (a call ends
a region in c2, sits mid-block in the port); (d) COLOR's real state lives in a
*third* representation (0x48-byte candidate records with a phase-overloaded
field) that IR3 has no concept for. IR3 is also the only IR with no migration
step, and its one novel component (item F) carries a 17-lane price
(`CFG_SHAPE.md`) the proposal never quotes, under a name ("cross-block
liveness") the repo has itself refuted. **Minimal repair, not a rewrite**:
give IR3 its own step; define it to *carry* c2's tuple identity (opcode
number, category, monotonic index) and region partition as first-class fields
— the `Terminator::Bc` carries-`(BO,BI)` pattern, this repo's own precedent —
and make the port→snapshot projection a graded artifact with its own
required-zero criterion.

### 5. The proposal has zero green curves and its three curve properties are false at tip
*(conjunction + history)*

- The only curve component ever measured — emit-set closure from the `.gl`
  `0x20` seed — disagreed with real c2 on **821 of 854 TUs (96.1%)** and was
  withdrawn to `Unknown` by its own prereg. Correctly published; but §6's
  "step 3 gives the first honest continuous curve toward 870" is false at
  master. (That the design *caught this early and cheaply* is a genuine point
  in the methodology's favor.)
- §6's "each gradeable today / none can silently regress under the existing
  gate / monotone": two of four curves are not gradeable today; the gate
  contains zero references to `plan-*` / `gap-metric` keys ✅ (5,854 lines,
  0 hits) so it cannot protect them; and `gap-metric` keys move with the live
  corpus (#3306, #3311: 82–105 of ~395 keys moved on lanes with empty
  `crates/` diffs), so they are not monotone.
- Predictor denominator ceiling is 854, not the promised 870 (16 TUs refused
  by `gl_function_attrs`).
- **Stale denominators quoted as §5 step-3 lane sizes**, re-measured at
  `d7be7aadc`: `845` → **844**; COMDAT synthesis `450` → **399** (11% off;
  the scan asserts `repaired + wall == graded`, 471+399=870); weak-external
  TUs `675` → **674**. `docs/CEILING.md:280` separately warns that quoting
  "the 450 wall" as a figure the project might *attain* inverts its sign.
- **The counterfactual is symmetric and both halves are ≈0**: perfect
  curves with step 5 frozen converts **1** TU; perfect step 5 with curves
  frozen converts **2** (= `gap-metric frontier`). Neither half pays without
  the other — which is why the *diagnosis* survives review even though the
  *remedy's* framing does not.

### 6. The proposal text is behind its own commissioned measurements *(history + drift)*

- §4 still claims "a ported COLOR is graded against COLOR's own output" —
  refuted by the stage-oracle rung it commissioned (rung §6.1: "a ported COLOR
  has no per-stage grade"). The step-0 and step-2 rows carry no landing
  annotations; only step 1 self-corrected.
- §1.2 item 1's headline "there is no lossless decode layer" is refuted by K1
  ✅ (`codec.rs:13`, fail-closed round-trip; #3332: 870/870 on the workload).
  The misdiagnosis had a measured cost: ir0 built a second, weaker framer and
  reverted it; the stronger invariant needed one `if`.
- §7.1's fork-server dependency is inverted by measurement (#3262: spawn was
  never the lever; the oracle rung explicitly declines residency and says
  residency should be fenced *by* the snapshots).
- §4's stop condition is binary ("stable stage observations or re-price as
  black-box") but reality is a **partial fire**: the mechanism is green and
  one pass is unobservable. The proposal has no language for this state — the
  single highest-value amendment.
- **Two citation slips, both verified, both also present verbatim in
  `STRATEGY_REVIEW_2026-08-13.md:485-489` and copied forward**: "a perfect
  reader converts 2" cites #3191 at `:113` and `:141` — the row is **#3190**
  ✅; and "523 of 845 fail A,B,C simultaneously" at `:112-113` is credited to
  `w-871`, which crosses by *mechanism* and states no such figure — it is
  `w-vocabgap` / **#3189**, whose headline says it in those words ✅.
- **A∧B∧C is measurably not *necessary*, and the proposal restates it as if
  it were**: `src/system/decomp_pch.cpp` is a `match` with A false
  (`-BCDE`), falsifying the assertion at
  `crates/c2-harness/src/gap/factors.rs:670` (`:668`, which this review
  first cited, is the D column — corrected by `w-archamend`). Recorded at
  `REFACTOR_REVIEW_2026-08-20.md:127-140` ✅ and not carried across. One
  named exception — it does not weaken the conjunction argument, which
  survived review intact.
- Sizing/anchors stale: "~15 fields" (`:100`) was **34** — cited to
  `docs/rungs/2026-08-21-w8sum.md:1,24`, **not** board #844, which is
  `w-alloc2` (store-run emitter leaf-only) and contains no occurrence of
  "Option" ✅. **Corrected again by `w-archamend`: this was not a reviewer's
  invention.** Board #3345's own headline opens *"BOARD #844's '34
  MUTUALLY-EXCLUSIVE `Option<Shape>` FIELDS…'"* and the w8sum rung repeats
  it, so the mis-attribution is in the repo's board and rung; this review's
  first correction blamed the wrong source. #844 is right for the half-emit
  hazard and wrong for the count, and the proposal now separates them. Cited
  line numbers drifted
  twice more; ~~`docs/GAPS.md:2601`~~ **`docs/GAPS.md` §8** (re-anchored
  2026-08-21 — the line number had drifted; §8 now carries the amendment in
  place) and `gap/fnbytes.rs:650` still describe the
  deleted `Bindings::positional` in the present tense ✅.

### 7. The economics are split between two goals, and the choice is unowned
*(cost/benefit)*

> ### ✔ 2026-08-21, SAME DAY — **THE CHOICE IS NO LONGER UNOWNED. THE OWNER TOOK IT, AND THIS SECTION'S FIRST BULLET IS SUPERSEDED — NOT SATISFIED.**
> *[`GOAL_DECISION_2026-08-21.md`](GOAL_DECISION_2026-08-21.md); `CLAUDE.md`
> § "The goal". Annotated in place by lane `w-goaldocs`; **no measurement below
> is withdrawn or edited**.*
>
> The goal is **perfect reproduction**, for two ends ~~ranked equally~~ — (1) a
> clear understanding of MSVC's internals in service of decomp, and (2) parity,
> a 100 % open-source implementation. The **verifier-throughput thesis is
> retired**, demoted to a property of a native port.
>
> > **↳ THE RANKING, added 2026-08-22 by lane `w-readdocs`; this page is a
> > DATED RECORD and nothing in it is rewritten.** *The owner amended the goal
> > statement later on 2026-08-21 (`GOAL_DECISION_2026-08-21.md` §
> > "AMENDED"): **goal (1) is primary**; goal (2) remains a real end and is
> > **additionally instrumental to (1)**, the port being an executable,
> > tweakable model of c2 that emits signals about compiler state the opaque
> > binary cannot. **The annotation above is unaffected in every particular** —
> > it turns on the throughput thesis being retired, which is unchanged. The
> > ranking only makes this section's collapse-onto-the-second-bullet reading
> > *more* secure, since the surviving branch is the one the primary goal is
> > read off.*
>
> **So the split this section diagnoses collapses onto its second bullet.**
> Step 5 is priced against Option A / 870, and against Option A it is *"the
> right and only trade"* — this section's own words, unchanged.
>
> **The first bullet's evidence stands; its force does not.** ≈2.4×, the
> bottleneck having moved off compilation onto generation, `C2RS_PREFILTER`
> never once enabled, 46 of 162,147 emitted functions, the hot-TU-first and
> c1xx-instead refutations — every one is a real measurement and every one
> stays quotable *as a measurement*. What is gone is the conclusion: it was an
> argument against a goal this project does not hold. **This is the specific
> misreading to avoid: it is SUPERSEDED, not SATISFIED.** Nobody met the
> 2026-08-13 NO-GO's two-clause reopen tripwire; the tripwire is guarding a
> question that is no longer asked. **No lane may be declined on these grounds
> again**, and — the half worth not losing — none may be *justified* on them
> either.
>
> Consequence **1(c)** below (*"the owner re-owning the thesis-vs-870 goal
> question"*) is therefore **DISCHARGED**. Consequences 1(a) and 1(b) are
> untouched and still gate step 5. The **free option** in consequence 5
> (enable `C2RS_PREFILTER=1` on one wave) is still free and still the owner's,
> but it is now a curiosity about a property rather than an input to a funding
> decision.

- Against the **verifier-throughput thesis**, step 5 is a WRONG-TRADE on
  measured evidence: the only real consumer (decomp-synth) is source-space
  (full `cl.exe` per candidate), capped at **≈2.4×** even with an infinitely
  fast c2 (c2 = 142ms of a 245ms PCH-warm compile); its wall-clock bound has
  **moved off compilation to generation** ✅ (verbatim, in the sibling repo:
  `../decomp-synth/docs/plans/il-witness/g2_push/SCORING_SPEEDUP_PLAN.md:410`
  — "scoring is no longer the wall-clock bound on any wave — generation
  is"); the uncapped IL-space regime was stood down in both repos (1
  attributable result, by source fix); the shipped hybrid prefilter has
  **never been enabled once** ✅ (`C2RS_PREFILTER` in no run script or job
  env); and work-weighted coverage is 46 of 162,147 emitted functions
  (0.028%), making the published 1.03× hybrid payoff ~100× optimistic.
  Hot-TU-first is refuted by the consumer's own data (top-10 TUs = 17–21% of
  compiles across 226–486 distinct TUs at 1–2 functions each — no
  concentration to exploit). c1xx-instead is also refuted: with PCH, c2 is
  58% of the compile and c1xx 18% — c2 *is* the right subsystem.
- Against **Option A** (reproduce c2 fully, 870/878 — the owner's decided
  goal per STRATEGY_REVIEW §8.1), step 5 is the right and *only* trade:
  the middle (D∨E) is needed by 843 of 845 refused TUs, its yield is
  multiplied 2→120 by step 3's factor A, and nothing substitutes.
- `STRATEGY_REVIEW_2026-08-13.md:251`: "The question is currently owned by
  nobody." Still true. **This is the decision that gates the port program,
  and it is the owner's, not an engineering call.**
- The hybrid fallback mechanism itself is SOUND and shipped: `PortC2` decides
  accept/refuse from the IL alone, before and independent of emission — there
  is no cliff in principle; there is just almost no coverage weight yet.

## What survives review (checked, clean)

- **The judge.** Untouched everywhere; no second judge proposed; the stage
  oracle ships with a standing bound that no `crates/` rule enters on
  snapshot equality.
- **The conjunction argument** (§1.2 item 4 / H3) — accurate citation-for-
  citation; the strongest part of the document. (One slip: "perfect reader
  converts 2" is #3190, not #3191.)
- **Steps 0–4 as landed.** Real machinery, honestly graded, nothing
  concealed. Step 2's zero delta is the one that is not a tautology (live
  dispatcher). Step 4's numerator retirement (74,033→0 by declining-to-name)
  is honest residue.
- **The construct-rung vehicle** — landed five times; needs a cost clause
  (#3336, named and unfixed in `docs/rungs/README.md`) but not a rewrite.
- **The correct positive case for step 0 exists and is stronger than the one
  §4 makes**: the snapshot genuinely captures the tuple order *entering*
  COLOR — item F0's input half, the largest single CEILING §6.1 line (8 of
  17 lanes) and exactly the black-box blind spot ("the order that decided the
  registers does not appear in the obj").

## Consequences (dispatched 2026-08-21)

1. **Step 5 as written: NO-GO.** Task #8 is gated on (a) the amendments
   below, (b) the re-estimation lane's cost curve, and ~~(c) the owner
   re-owning the thesis-vs-870 goal question~~ **(c) — DISCHARGED the same
   day: the owner answered it, and the answer is full reproduction
   (`GOAL_DECISION_2026-08-21.md`). (a) and (b) still gate.**
2. **Proposal amendment lane** — apply the required corrections (§4 partial-
   fire language + F0-based positive case; §6 calibration applied not cited,
   scorable step definitions, three curve predicates deleted/qualified; §1.2
   item 1 retracted in favor of the real complaint; §5 step 3 re-scoped per
   objplan §5; §7.1 reversed; explicit integration row between 4 and 5 priced
   two-sided; IR3 given its own step in tuple/region coordinates; stale
   anchors/counts; `GAPS.md` §8 and `gap/fnbytes.rs` doc rot).
3. **Re-estimation characterization lane** (the user's standing option 1),
   now with concrete deciding probes named by the review: operand/candidate-
   record walk (stageoracle §6.1 q1), a `sched0`-output probe (q2 + F2), the
   port-side region-trace feasibility probe, and a per-stage cost curve
   re-priced against *actual* snapshot coverage (375/410 armed-and-fired,
   ~20 non-empty workload TUs, 7 sites).
4. **Instrument fixes**: guard the raw-window verdict on `compared > 0`;
   publish a distinct-row count beside `stage-snap-tuples`. One-condition
   class, priced in lines.
5. **Free option for the owner** (decomp-synth side, not dispatched from this
   repo): enable `C2RS_PREFILTER=1` on one wave and wire it into the pool
   backend — hours of work that converts the hybrid's value from assumption
   to measurement.

Reviewer scratch (verbatim reports, repro scripts) is under gitignored
`work/reviews/arch/`; this file is the durable record.
