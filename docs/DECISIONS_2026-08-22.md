# Owner decisions — 2026-08-22

Recorded by the coordinator the same day, from the owner's review of the
2026-08-22 plain-language brief (the brief itself is conversation, not a doc;
everything it asserted that matters is already in the tree — `docs/whitebox/
READ_PLAN_2026-08-21.md`, `docs/GOAL_DECISION_2026-08-21.md` § AMENDED, board
#3367–#3370). The owner's words, verbatim, because the numbering below maps
onto the brief's three asks:

> *"#1 funded for option 4. #2 rewrite the doc and get it aligned with our
> earlier goal statement. #3 you have permission to push"*

followed in the same session by:

> *"additionally, please lay out the structure to make finding information
> easier. look at ../decomp-synth for inspiration on the structure. hand this
> an opus subagent"*

## Decision 1 — the reads are FUNDED ("option 4")

**What "option 4" is.** The brief presented `ARCHITECTURE_PROPOSAL_2026-08-20.md`
§8 decision 0 as having grown a fourth option beyond (fund 4a now / declare
characterization-only / run the 8-week Phase 0 first): **fund the read-plan's
R1→R2→R3 first (~5–9 days), then decide.** That is the option funded.

**What this decides.** R1 (the `DAT_10c400d4` scope read, 0.5 d), R2 (the
encoder — two tables plus the 79 arms off `0x10bfae2d`, 2–4 d) and R3 (the
label charge, 2–4 d) are authorized and dispatched as characterization lanes
under prereg (`docs/whitebox/READ_PLAN_2026-08-21.md` §3–§4 are the specs).

**What this does NOT decide — read this before citing the decision.**
Decision 0's branch choice — approve 4a/4b, declare step 5
characterization-only, or run Phase 0 — remains **open**. "Funded for option
4" funds the *reads before the choice*, and the choice is deferred until
R1–R3 report. Nothing here approves 4a, prices it lower (a read produces a
spec, not an implementation — the proposal's own units warning), or waives
Phase 0's stop condition. Row 5's NO-GO clauses (a)–(c) stand.

## Decision 2 — the shipping roadmap is rewritten against the goals

`docs/SHIPPING_ROADMAP_2026-08-19.md`'s headline (track 1: a vendor-DLL-backed
compatibility service) tripped its own §1 escape clause when the owner stated
goal (2) — a 100 % open-source implementation — and two lanes (`w-goaldocs`,
`w-readdocs`) bannered the contradiction without resolving it, because
re-ranking a product plan is the owner's call. **The owner has now made it:
rewrite the doc, aligned with the goal statement** (goal (1) primary — 
understanding MSVC's internals in service of decomp; goal (2) parity, real
and instrumental to (1); `docs/GOAL_DECISION_2026-08-21.md` § AMENDED).

Execution note: the 2026-08-19 text is a dated record and stays on disk as
one, bannered as superseded; the rewrite lands as the superseding roadmap.
What survives on its own merits and should be carried forward rather than
rewritten: §4's "three meanings of 100 %", §6's operating-model items 3–6,
and every measurement (kept as measurements that neither justify nor forbid).

## Decision 3 — push is authorized

Standing conflict resolved: the repo `CLAUDE.md` says *"Never push unless
asked"*; a stored session memory said this fork
(`git@github.com:freeqaz/c2-rs.git`, the owner's own) was push-authorized.
The coordinator held 46 commits on the conservative reading and asked. **The
owner has now said it in this session** — the commits push today, and the
memory's claim is confirmed rather than merely stored. The repo rule stays as
written; this is the "unless asked" branch of it being exercised, plus a
standing confirmation that this remote is the owner's fork.

## Decision 4 — a documentation structure pass, on an Opus lane

The owner ordered a structure pass so information can be found: 69 top-level
files in `docs/` plus `docs/whitebox/`, with `../decomp-synth`'s layout
(topical subdirectories, `DOC_CONVENTIONS.md`, purpose-first README) named as
the model. Dispatched as its own lane with one hard constraint stated at
dispatch: **`file.md:NNN` citations are load-bearing across this tree and
`board_audit.sh` cannot see cross-doc staleness** (#3367's consequence, the
detector lane named at the `w-goaldocs` merge and still unbuilt) — so the
lane prices citation breakage before moving any file, and a navigation layer
that moves nothing is the default shape.

## Recorded from the same brief, not owner-decided

Two recommendations the brief made that the owner's reply did not rule on;
they stand as the coordinator's, recorded here so they are in the tree rather
than in a conversation:

- **The permuter wants a measurement before a build.** The measured
  wrong-body population is one mechanism — c2 inlined a callee where the port
  emitted a call (`docs/DIFF_STRUCTURE.md`: 0 pure reorderings, 5,173 of
  5,189 substituted words differ in opcode, 94.3 % wrong at word 0) — and it
  is the **port's own** failure population, which nothing has measured
  against the owner's actual permuter case, hand-written decomp near-misses
  (#3369's conflation). One day of measuring whether decomp near-misses are
  also inlining-dominated decides which permuter to build; building first
  risks weeks on the wrong search space. `splice.rs:57-60`'s 0.9716 inline
  cost model is the candidate knob if inlining dominates.
- **`docs/DIFF_STRUCTURE.md` needs a rescan, not an edit.** Its numbers are
  from tree `0c8a185` (3,195 wrong bodies against today's 1,960 + 530) and
  its own banner marks §3.2 refuted. Cheap, and load-bearing for the
  measurement above.

---

## Decision 5 — decision 0's branch is chosen: run Phase 0, and build while it runs (later the same day, after R1–R3 landed)

The owner, verbatim, after the three funded reads were merged and pushed
(`071864537` / `299aea369` / `3d507bc6e`, master `02f8a8af0`):

> *"please review and let's switch to implementation now. update the docs
> first. what do you need my answers for, if anything? We should build what
> we can that is high leverage to get us results + validation of our design.
> The 8 week experiment is reasonable to start work on now via opus
> subagents."*

**What this decides.** `ARCHITECTURE_PROPOSAL_2026-08-20.md` §8 decision 0's
branch choice — deferred at decision 1 until the reads reported — is
**branch (c): the 8-week Phase 0** (`ROADMAP_SLICING_2026-08-21.md` §5: S0,
the blind-reach instrument, and S1, the general `Plain`+`Tail` lowering as a
required-zero re-expression on the live dispatcher). It starts now, dispatched
as lanes. The owner's framing adds a standing selection rule for concurrent
work: **build what is high-leverage toward results plus validation of the
design** — which is Phase 0's own purpose stated as a dispatch criterion.

**What this does NOT decide.** Rows **4a**/**4b** are still not approved —
Phase 0 exists to price them on evidence, and one of its registered outcomes
(*S1's delta holds but workload `fnbyte-exact 35,894` moves*) voids the
pricing basis and should stop the program.
**Re-anchored at the w-s0 merge (#3396): the 35,894 was stale** — the
workload tree moved to `e5aef017d456`, where the base reads
**`fnbyte-exact 35,891`** (measured by w-s0 at `9b9530791`). The stop
condition binds against **the S1 lane's own base-tree measurement with its
workload stamp recorded beside it**, never against a filed snapshot — as
written above it would have false-triggered a program stop on a
workload-tree difference.
**AND THAT RE-ANCHORING WAS ITSELF INSUFFICIENT — corrected again at the
`w-s1bc` merge (2026-08-23, board #3428).** The condition is not merely
mis-anchored, it is **unquotable without naming its corpus**: one *unchanged*
binary produced **35,885 / 35,890 / 35,905 / 35,898** across four corpora
inside ~2 hours, and `dc3-decomp` moved **seven times** during that single
lane. Scoring that lane's tip against the filed 35,894 reads *"moved by 4"*
and **would have stopped the program with byte-identical output throughout**.
So *"re-measure on your own base tree"* is **not enough** — a base and a tip
measured minutes apart can straddle a corpus change. **The standing form is
now: the condition binds only on a base and tip measured against the SAME
PINNED BINARIES and the SAME workload stamp, run back-to-back, with the stamp
quoted beside the number.** A threshold that does not name the corpus it was
taken on is not a threshold. Third correction to one registered condition in
two days, each found by the lane the condition was pointed at. Step 5's NO-GO clauses (a)–(c)
stand. Nothing here re-prices anything, and the byte judge is untouched.

**The evidence the choice was made on** (discharging the coordinator's
promised decision-0 synthesis; this section *is* that report):

- **Two of the three funded reads were subtractions.** R1 removed
  `P_REGALLOC.md` consequence 3's premise, so the ten refuted allocation keys
  are back to UNEXPLAINED and the question passes whole to R4 (unfunded).
  R3 struck `READ_PLAN` §3 row R3's *"closed by construction"* in half: the
  site population is closed, the **charge is not** — 42 of 163 sites sit on
  loop back edges, so `charge(TU) = the number of objects c2 constructs
  itself`, and "replace two fitted constants" inherits "reproduce c2's object
  population" (#3387).
- **R2 succeeded totally and still declined to re-price I2** — 79/79 arms,
  both tables, and `P_ENCODE.md` §9's six reasons the spec is not yet
  buildable, first among them that it starts at a finished machine tuple and
  building the tuple is R5 (189 arms, unstarted, 15–25 d).
- **R3 found a latent defect in shipped code** (#3388):
  `crates/c2-core/src/coff/label.rs`'s `LABEL_SEED_GAP = 9` is
  `7 + 2·[/Og] + 1·[/GF ∧ string pooled in the data phase]`, and the `/Od`
  lane is green only because its 21 matching TUs contain zero function
  definitions. **That is the model-or-fit failure mode Phase 0 exists to
  test, already present in miniature in the tree** — a fitted constant
  passing because the control is structurally incapable of exercising it.
  It is the single strongest piece of evidence that Phase 0's question is
  live, which is why the reads strengthened the case for branch (c) rather
  than for approving 4a blind.

**Dispatched under this decision**, three lanes, all Opus:

1. **S0** — the blind-reach instrument (`ROADMAP_SLICING` §5 row S0):
   `fnbyte-blind-{exact,differs,unlowerable}` over the 113,565 parse-refused
   functions, graded against real c2's COMDAT bytes, published beside FBM
   under FBM's separation rule — never in `gate.sh`, licenses no emit —
   with the required-zero identity (emit-path diff 0, 18-lane gate diff 0,
   `match 26 / mismatch 0 / fnbyte-exact 35894` unmoved).
2. **S1** — the general `Plain`+`Tail` lowering (row S1, with its 2026-08-21
   amendment: the bijection promoted as a per-function **ratio**, never an
   equality gate), required-zero byte delta on the live dispatcher (#3346),
   pre-registered cost criterion (#3336), `after0` opcode agreement as a
   ratio measurement.
3. **The `LABEL_SEED_GAP` repair** — the first build selected under the
   high-leverage rule: replace the fitted `9` with R3's read formula,
   exposed as a named decision point per `GOAL_DECISION` § AMENDED, priced
   two-sided, byte-judged against the reference on the very shapes that
   today return `NotImplemented` where the grid can reach them, and with the
   required-zero identity on everything currently matching.

**What still needs the owner** (asked in the same turn, recorded here):
whether to fund **R4** (globregs mint order, 3–5 d — now the destination of
R1's unanswered question) and **R5** (the 189-arm IL→codegen dispatch,
15–25 d — the shared input to all ten Phase-1 slices and the closure of
R2's `[R]` bound). Decision 1 funded R1–R3 only; R5 is material spend and is
**not** started under this decision. Nothing else blocks on an answer.

---

## Decision 6 — R4 and R5 are funded (2026-08-23)

*This file is the continuous owner-decision record opened 2026-08-22; this
decision was taken the following day, in the same session, and is dated
explicitly rather than split into a second file.*

The owner, verbatim, on being asked the one open spend question at the end of
the Phase 0 wave:

> *"fund R4 and R5 now. drive this with opus subagents please. your role is
> coordinator"*

**What this decides.** The two reads left unfunded by decision 1 —
`docs/whitebox/READ_PLAN_2026-08-21.md` §3 rows **R4** and **R5** — are
funded and dispatched as characterization lanes under prereg. Decision 1
funded R1→R3 only; decision 5 explicitly put R4/R5 back to the owner as the
sole open spend question. It is now answered.

- **R4** — `FUN_10b55732` (1,676 B), the globregs mint/merge, **3–5 days**.
  This is where R1's subtraction sent its question: R1 proved the candidate
  counter is **function-scoped**, which removed `P_REGALLOC.md` consequence
  3's premise, so **the ten fitted-then-refuted allocation keys
  (`alloc.rs:103-539`, "wrong on 5 to 42 each") have no explanation on that
  mechanism at all**. R4 reads the mint order and merge rule as an ordered
  algorithm — the missing input to the already-read comparator, and the
  standing account owed to the 52,416-configuration null.
- **R5** — `FUN_10bc2d7a` (5,080 B), the **189-arm** IL-record → codegen
  dispatch, **15–25 days**. The largest row on the critical path: it is
  I1 (the general decode), the shared input to all ten Phase-1 construct
  slices, and the closure of the `[R]` bound R2 named in `P_ENCODE.md` §9 —
  an encoder is a total function of a tuple **nobody can yet build**, and R5
  is how the tuple gets built. Its gate was discharged by R2: the
  arm-reading method works on 79 bounded bodies.

**Why now, and what it does NOT decide.** R5 is material spend and it is
**still not an approval of rows 4a/4b** — a read produces a spec, not an
implementation, and quoting "the dispatch is read" as "I1 is cheaper" would
be the units error `READ_PLAN` §5 warns about. `CEILING` §5's ~5:1
calibration was fitted on lane-shaped construction work and must not be
applied to a read. Phase 0 continues in parallel and still prices 4a/4b on
evidence; step 5's NO-GO clauses stand; the byte judge is untouched.

**The standing caveat, unchanged:** `[R]` means *"the instructions were read
correctly"*, not *"this is what c2 does"* — the `.bss` bump rule was read
correctly out of a clean function and was wrong about c2. Every read lane
ends in a confirmation probe.

**Dispatched under this decision**, both Opus, both characterization lanes
(`Fixtures: none`, `Census: +0`, predicted reach 0, prereg as the first
commit): **`w-read-r4`** (board **#3410**–**#3414**) and **`w-read-r5`**
(board **#3415**–**#3421**).

**Nothing is currently waiting on the owner.**

---

## Decision 7 — the remaining Phase 0 slices are dispatched and R6–R9 are funded (2026-08-23)

The owner, verbatim:

> *"okay please continue and dispatch the remaining slices and fund R6+. do
> this via opus subagents"*

**What this decides.** Two things, both dispatched as Opus lanes:

1. **The remaining Phase 0 slices — S1b and S1c** (`ROADMAP_SLICING_2026-08-21.md`
   §5 row S1; priced by the lane that landed S1a in
   `docs/rungs/2026-08-22-w-s1.md` §6). S1a shipped the per-op value and the
   one general composition; S1b collapses `Selected::{Plain,Tail,MemcpyTail}`
   on the live path (~10 edit sites, mechanical, one full base+tip gate pair),
   and S1c converts the 18 `Plain` + 4 `Tail` producers to build a
   `Vec<MachineOp>` natively — **the bulk of S1's 2–4 weeks**. Dispatched as
   ONE lane, not two: both edit `Selected` and its consumers, and this repo's
   own record is that concurrent lanes erase each other through shared
   predicates with no textual conflict and no red gate.
2. **Reads R6, R7, R8 and R9 — the entire unfunded remainder** of
   `docs/whitebox/READ_PLAN_2026-08-21.md` §3. With R1–R5 spent, funding these
   closes the read plan. Four separate lanes: the subsystems are disjoint and
   the read plan itself says so.

| lane | read | price (days) | why it is worth its price |
|---|---|---:|---|
| `w-read-r6` | the final-expansion switches — `FUN_10c0d57e` (3,899 B), `FUN_10c182b4` (426 B, 18 arms), the `0x2f4`/`0x2f0` prologue arms | **4–6** | **it de-risks the lane dispatched beside it**: S1's bijection instrument is *known* to go red on framed functions without this, because the final expansion rewrites the prologue pseudo-op in situ into many words |
| `w-read-r7` | scheduler `[R]` → `[O]` — **no new reading**; confront the read priority/latency model with the live tap | **3–5** | re-prices F0 8 → 4 raw and confronts the **13,104-configuration residual** with c2's actual priority function |
| `w-read-r8` | block emission order — `fg.c` `0x10b36133`, `factor.c` `0x10b34a89`, `0x10b968b0` | **5–10, uncertain** | `CEILING` phase 1, the one **unserved** phase — *"a port cannot place labels"*. It is **the other half of R3** and R3 is spent, which is what unblocks it |
| `w-read-r9` | the `0x4F` sub-record switch `FUN_10b9761e` (~14 arms) | **1–2** | the one transcribed width in the port, and every other sub-opcode's refusal. ~~**R5 supplied its entry points** (arms 48/49)~~ — **struck 2026-08-23 by the lane that used it: R5 handed R9 *arm 32* -> `0x10bbe561`; arms 48/49 are R6's.** `docs/rungs/2026-08-23-w-read-r5.md:195-198` is a compound sentence naming two reads and was mis-transcribed here. **R9 has RUN** — [`docs/whitebox/ref/P_SUB4F.md`](whitebox/ref/P_SUB4F.md), board **#3442**-**#3444**; and *"the one transcribed width"* is itself wrong, there are five readers carrying two inconsistent widths |

**The riskiest row is named as such**: R8 is the only row in the plan **with no
known address for the rule it is looking for**, and its 5–10 is explicitly
uncertain. A priced decline is an acceptable outcome there.

**What this does NOT decide, and it matters.** **Phase 1 is NOT dispatched and
is not unlocked.** `ROADMAP_SLICING` §6 rule 2 gates Phase 1 on S0's
`blind-differs` number. That number now exists — and it is **373 of 388
reached, 96.1 % byte-wrong** (#3392–#3396). The gate has been satisfied in the
sense that the number was produced; it has **not** been satisfied in the sense
the rule intended, for two independent reasons:

- **S0 declined all three of §5's registered readings**, under a rule frozen in
  its prereg before measuring: its ladder relaxed an *admission* gate, while
  §5's outcomes are about a *general decode*. 99.66 % of the population never
  reached the lowering. **S0 has not yet asked §5's question**, and the lane
  said so about its own headline.
- **The number that does exist argues against Phase 1's naive form**, not for
  it. 96.1 % wrong emits is a direct price on what the next `functions()`
  widening would ship, and under `PROGRESS_METRIC.md` a wrong emit scores
  strictly below the refusal it replaces.

So Phase 1 stays closed, and the honest statement is that **the cheap path to
coverage is now measured and it is bad**, rather than that it is untested.
Rows 4a/4b remain unapproved; step 5's NO-GO clauses stand; the byte judge is
untouched.

**Board:** `#3423` this decision · `#3424`–`#3428` `w-s1bc` ·
`#3429`–`#3432` `w-read-r6` · `#3433`–`#3436` `w-read-r7` ·
`#3437`–`#3441` `w-read-r8` · `#3442`–`#3444` `w-read-r9`.

**Nothing is waiting on the owner.**
