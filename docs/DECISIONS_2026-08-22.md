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
  building the tuple is R5 (~~189 arms, unstarted~~ **61 real arms over 95 opcodes + 94 refusals, READ at #3415 — decision 13 below carries this correction; [`WB_ILARMS_MAP.md`](whitebox/WB_ILARMS_MAP.md) §1**, 15–25 d).
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
R1's unanswered question) and **R5** (the ~~189-arm~~ **61-real-arm (95 opcodes + 94 refusals — decision 13 below; [`WB_ILARMS_MAP.md`](whitebox/WB_ILARMS_MAP.md) §1)** IL→codegen dispatch,
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
- **R5** — `FUN_10bc2d7a` (5,080 B), the ~~**189-arm**~~ **61-real-arm (95 opcodes + 94 refusals — decision 13 below; [`WB_ILARMS_MAP.md`](whitebox/WB_ILARMS_MAP.md) §1)** IL-record → codegen
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

## Decision 8 — the implementation wave is dispatched (2026-08-23)

The owner, verbatim: **"Okay let's orchestrate the implementation opus
subagents now."**

The queue this funds is the one reported at the wave-5 close, and the
selection rule is decision 5's standing one — build what is high-leverage
toward results plus validation of the design. Four lanes, dispatched
concurrently on Opus subagents, disjoint file fences:

| lane | kind | what it buys | price carried in |
|---|---|---|---|
| `w-s1c2` | construct | S1c (i)'s remainder: the 63 encoders without a `mop_*` (~2 h mechanical, 5 hand cases) and the producer half — 12 `Selected::plain` producers + 4 `Tail` sites, dominated by `select_text` and `store_leaf_text` (3–6 sessions, the lane's own estimate) | `w-s1bc` rung §6 |
| `w-4f01` | fixture-claim / defect fix | the contradicted `4F 01` width (#3443): the port reads a fixed byte where c2 reads VI32, latent because every fixture sits below source line 128. R9 shipped the rule, the twin grid that fails on demand, and the exact edit sites | `WB_SUB4F_FINDINGS.md` §5 |
| `w-pwords` | construct | the `prolog_words` seam (#3431): feed the oracle-side `.pdata` field into the port-side bijection instrument and measure whether S1's demoted equality is recoverable. Three caveats live; the ratio-not-equality amendment binds until measured | `w-s1bc` rung §7 |
| `w-tailread` | characterization | read `0x10c3afd8` — the 767-opcode class table behind the dispatch tail, R6's top-ranked follow-up; plus the peephole's one unpublished arm (6 = `fmr`) and, if cheap, the `0x10b1d180` index contradiction | R6 rung "Found and not taken" items 1–2 |

**Two lanes edit `crates/` at once, which no previous wave allowed.** Accepted
deliberately: the crates are disjoint (`c2-core` vs `c2-il`+`c2-reference`),
each lane carries its own byte-attribution protocol, and merges stay serial
with a full armed re-gate at each landing. If the fences turn out to touch,
the later merge eats the conflict and the funnel catches it.

**Deliberately not dispatched:** S1c (ii) (`w-s1bc` §6 says it is not
priceable from there — it waits for the producer half); the two +24 % cost
fixtures (a property of the fixtures, parked); `#3444` (bounded, no action
needed); Phase 1 (unchanged — `#3423`'s gate reasoning stands).

**Board:** `#3446` this decision · `#3447`–`#3451` `w-s1c2` ·
`#3452`–`#3455` `w-4f01` · `#3456`–`#3459` `w-pwords` ·
`#3460`–`#3463` `w-tailread`. Next free `#3464`.

**Nothing is waiting on the owner.**

## Decision 9 — wave 7 is funded AND Phase 1 is unlocked (2026-08-24)

The owner, verbatim: **"continue and lets get both done. use opus subagents
and drive coordination."** — in reply to the wave-6 close, whose final
paragraph framed exactly two options: *"fund wave 7 within Phase 0, unlock
Phase 1, or both."* The owner chose both.

**Phase 1 is UNLOCKED, by the owner.** This supersedes `#3423`'s closure.
`#3423`'s two reasons stand as written and as history — S0 never asked §5's
question, and blind-differs 96.1 % prices the naive widening as bad — but the
gate they argued for was a *default*, and the owner has now decided over it.
What does **not** change, because none of it was ever the gate:

- **The one correctness rule.** Byte-judged by real c2 on new fixtures, per
  `ROADMAP_SLICING` §5 Phase 1's own row definition. No blind widening ships:
  a Phase-1 slice earns coverage through fixtures the byte judge grades, and
  outside its fenced class the port still returns `NotImplemented`.
- **A wrong emit scores strictly below the refusal it replaced**
  (`PROGRESS_METRIC.md`) — which is precisely why unlocking Phase 1 does NOT
  mean re-running S0's relaxed decode as an emitter.
- **Required-zero identity on the incumbent gate table** for every slice, and
  **two-sided pricing** (§6 rule 2) for every admission widening inside it.

**Phase 1 opens with C1 alone**, per the slicing doc's own ordering rule:
*"C1 is a promotion, not a construction — `designator.rs` already resolves
the offset and width for four consumers. Largest row, cheapest work; take it
first."* C2–C10 stay undispatched until C1 reports what a slice actually
costs; C4 is additionally fenced by §5's inverse-trap note (never schedule it
off its decode number).

Four lanes, dispatched concurrently on Opus subagents:

| lane | kind | what it buys | price carried in |
|---|---|---|---|
| `w-s1c3` | construct | the S1c (i) remainder as `w-s1c2` §6.2 priced it: 5 producers (~2 sessions: `pool_ctor_chain`, `xtea_round_loop`, `float_walk_loop`, `compare_leaf_text`, `select_text`) + `store_leaf_text` (~1, moves two files) + `permute_args_text` (~1, UNVERIFIED — must be re-priced by reading before converting). S1c (ii) only as a registered stretch (only 2 of 13 `block_ir` arms are `Plain`). May adopt the mop generator + cost harness into `scripts/` as a named, two-sided-priced deliverable closing `#3451` — never smuggled | `w-s1c2` rung §6.2, §6.3 |
| `w-c1` | Phase 1 slice C1 (fixture-claim + construct) | off-add (`0x27`, 33.3 % of the residue): promote `designator.rs`'s existing offset/width resolution to one value-type variant + its type resolution + one general lowering, byte-judged on NEW fixtures, required-zero identity on the incumbent gate table | `ROADMAP_SLICING` §5 Phase 1 · §3 |
| `w-ordid` | construct | close `#3459`: carry a function identity through the funcwalk tap so ordinals stop being paired to `.text` address order on faith — the hazard affects every funcwalk instrument in the tree | `w-pwords` rung §4 |
| `w-r8idiom` | characterization | the `mr r8,r8` idiom (3,792 instances, all r8, unexplained — `w-tailread`'s top-ranked follow-up, obj-visible) plus arm 14's handler `0x10c16d83`, where the self-move exception lives — the same question from its two sides; and, if cheap, the second byte table `0x10c3b270` | `w-tailread` rung "Found and not taken" 1, 6, 3 |

**Concurrency acceptance, sharper than decision 8's.** `w-s1c3` and `w-c1`
both touch `crates/c2-core` — same crate, which decision 8 did not allow. The
fence is file-level and asymmetric: `w-s1c3` owns every existing `codegen/`
file it converts; `w-c1` may create NEW files in `c2-core` and edit
`crates/c2-il` (designator/decode), but may not modify any existing `c2-core`
file — if its lowering needs one (a new opcode in `mop.rs`, a dispatcher arm),
it STOPS and reports, per the `w-seedgap`/`w-pwords` precedent. The repo's own
record (decision 7) is that same-crate concurrent lanes erase each other
through shared predicates with no textual conflict; the stop-and-report rule
plus serial merges with a full re-gate at each landing is the answer this
wave carries.

**Deliberately not dispatched:** C2–C10 (C1 reports first); the two +24 %
cost fixtures (still a property of the fixtures); R8's open items and R6's
`0x2f5` orphan (queued behind the funded characterization); the
`P_EXPAND.md` §3 signed-word-delta re-score (needs the delete oracle,
priceable after `w-r8idiom` reads arm 14); the maintenance debt (`#3381`,
the phantom `PROGRESS_METRIC.md` §5.2 citation, `DIFF_STRUCTURE.md`).

**Board:** `#3466` this decision · `#3467`–`#3471` `w-s1c3` ·
`#3472`–`#3476` `w-c1` · `#3477`–`#3480` `w-ordid` ·
`#3481`–`#3484` `w-r8idiom`. Next free `#3485`.

**Nothing is waiting on the owner.**

## Decision 10 — wave 8, and C7 is dispatched as a COST PROBE because its coverage was already measured at zero (2026-08-24)

The owner, verbatim: **"please continue to the next phases after this
finishes."** — given while wave 7's last lane was re-gating, in reply to a
close that named the live options (C7 as the second Phase 1 slice,
`permute_args_text`, S1c (ii), naming `0x2e4`). This funds wave 8 and carries
decision 9's Phase 1 unlock forward; nothing in decision 9 is superseded.

**Wave 7 landed complete**: `w-ordid` (`built`, `#3459` closed — the funcwalk
hazard is an OFFSET, not a permutation), `w-r8idiom` (`built`, `mr r8,r8` is
`emit 0x7d084378`, a baked literal), `w-s1c3` (`built`, S1c (i) COMPLETE,
required-zero verified by the coordinator against the prior base tree), `w-c1`
(`converted`, Phase 1's first slice: +8 `fnbyte-exact`, 0 new wrong emits, the
`(tu, symbol)` differ-set identical at 1,968). Ledger `#3466`–`#3484` closed.

**THE CORRECTION THAT SHAPES THIS WAVE.** `w-c1` §8 recommends C7 (compare)
as the second slice — *"the only remaining Phase-1 construct with a shipped
sink, though a poisoned one."* That is a **cost** argument and it is right.
It is not a **coverage** argument, and the board had already answered the
coverage question before this wave was designed:

- **`#420`** — `expr-cmp-eq` is a FALL-THROUGH key; the whole relational
  family (3,298 blocked emitted functions) is worth **0 TUs and at most 5
  functions**. That lane's deliverable *was a decline of the rung it was
  asked to build*.
- **`#1593`** — admitting the WHOLE relational family moves
  `frontier-codegen-reader` **48 → 48**: recovered 0, renamed 21.
- **`#423`** — the six relations are **not one family in the guard
  position**; four of six rewrite at exactly `k = 0`, unsigned. A live
  wrong-emit hazard, whose shape `#1788` already caught once (`int size` and
  `unsigned size` emit the identical `22` byte).

This is the fifth time the standing rule *"a lane dispatched off a
blocked-key size ranking finds the ranking was an artifact"* has bound. C7
therefore ships with **coverage ≈ 0 TUs / ≤ 5 functions REGISTERED IN ITS
PREREG BEFORE MEASUREMENT**, so neither a zero nor a surprise can be
reported as a win. What C7 actually buys is the number `w-c1` could not
give: **the cost of a CONSTRUCTION slice.** C1 was a promotion — its own §8
says it is *"the floor of the range, not a sample"* — and the remaining nine
slices are priced at 2–4 wk raw / 10–20 wk LB **each**. Whether Phase 1 is
fundable past C1 is a decision that turns on this one measurement.

Four lanes, dispatched concurrently on Opus subagents:

| lane | kind | what it buys | price carried in |
|---|---|---|---|
| `w-c7` | Phase 1 slice C7 (fixture-claim + construct) | the relational family `0x1F`..`0x24`: one value-type variant + its type resolution + **one real lowering** (unlike C1, there is no acceptance rule to promote — `C2RS_SINK_REL` is measurement-only by construction, pushes no `IlOp`, and any walk containing a relational refuses under `expr-rel-sink-poison`). Deliverable is the **construction-slice cost**; the `_neg` fixture must carry `#423`'s `k = 0` unsigned case | `ROADMAP_SLICING` §5 · `w-c1` §8.2/8.3 · `#420` · `#423` |
| `w-permute` | construct | S1c (i)'s last producer: `permute_args_text` — **6 functions / 342 lines**, re-priced BY READING and VERIFIED by `w-s1c3` (§6.2's four-name enumeration was short by two; no byte-position obstruction; one extra caller at `calls.rs:1360`) | `w-s1c3` rung §6.1 · `#3471` |
| `w-3475` | construct | `eat_int_operands` (`mcall.rs:2917`) still has no `0x27` arm, so the completeness walker mis-names `off-add` for **19,898 bodies**. It feeds `Admit`, so required-zero is a **HYPOTHESIS TO MEASURE, not an assumption** — if it moves bytes, byte-judge and price two-sided | `w-c1` `#3475` |
| `w-2e4` | characterization | name `0x2e4` — the pseudo-opcode that is the sole route to `mr r8,r8` — by reading `fg.c`'s edge construction at `0x10b372ea`/`0x10b39937`. `w-r8idiom` deliberately refused to name it and ranked this its #1 follow-up at one lane | `w-r8idiom` "Ranked follow-ups" 1 |

**Concurrency fences.** `w-c7` and `w-3475` both touch `crates/c2-il` — the
wave-7 asymmetric pattern, now file-level within one crate: `w-c7` owns
`func/body/expr.rs`, `w-3475` owns `func/body/mcall.rs`, and either STOPS and
reports if the other's file must change. `w-permute` owns
`crates/c2-core/src/codegen/calls.rs` and the call-sequence files; `w-c7` may
CREATE new `c2-core` files but may not modify an existing one. `w-2e4` is
docs-only, zero `crates/` bytes.

**Merge order: `w-3475` before `w-c7`.** The required-zero correction lands
first and `w-c7` re-verifies its byte-judged fixtures after rebasing over it —
wave 7's `w-s1c3`-before-`w-c1` rule, which paid for itself (c1's verdicts
went 7,002 → 7,038 = exactly 2 fixtures × 18 lanes, 0 mismatch, measured on
the merged tree rather than assumed).

**Deliberately not dispatched:** C2–C10 other than C7 (C7 reports the
construction cost first; C4 still fenced by §5's inverse-trap note); S1c (ii)
(only 2 of 13 `block_ir` arms are `Plain`); the three `cgintrin.c`
`push 0x7d084378` sites (a *second* idiom, `nop / nop / mr r8,r8`);
`0x10c3b270`; `nopcapenter`/`nopcapexit`; the `P_EXPAND.md` §3 signed
re-score (now priceable — arm 14 has been read); the generated STATUS.md
block's missing workload identity (found this wave, recorded not repaired);
`WB_EXPAND_FINDINGS.md:79` and `#3432`'s stale "unrecorded" sentence — **two
lanes have now declined it for the same reason; it needs budget, not another
flag**; the maintenance debt (`#3381`, the phantom `PROGRESS_METRIC.md` §5.2
citation, `DIFF_STRUCTURE.md`).

**Board:** `#3485` this decision · `#3486`–`#3490` `w-c7` ·
`#3491`–`#3495` `w-permute` · `#3496`–`#3500` `w-3475` ·
`#3501`–`#3504` `w-2e4`. Next free `#3505`.

## Decision 11 — wave 9: measure the JOINT blocker set before ruling on Phase 1 (2026-08-24)

The owner, asked directly whether wave 9 should stop Phase 1, take C7's
one-arm variant, continue as planned, or measure the joint blocker set first,
chose: **measure the joint blocker set first.** Phase 1 is therefore **neither
stopped nor continued** — it is **held pending evidence**, and decision 9's
unlock stands unchanged in the meantime. No Phase-1 slice is dispatched in
this wave.

**Why the question is open rather than already answered.** Wave 8 closed with
two slices measured: C1 (+8 functions, 0 TUs, ~1 session) and C7 (declined —
its lowering was already built, byte-graded on 552 cells, measured `+0` and
reverted on 2026-07-31; ceiling today **6 emitted functions, 0 TUs**). Every
*single*-key closure this project has measured substitutes a successor rather
than converting: `#150` has seven-plus confirmations, `#420`, `#1593`, `#440`,
and `#3094` (which measures **−2,694 byte-exact functions and −1 matching TU**
for admitting one token). That is a strong case for stopping — **and it is
made entirely of marginals.**

**The prior that makes the joint question worth funding is already on the
board and was measured, not guessed.** `w-loo` (`#3135`–`#3140`) published its
own blind spot in the same breath as its result: two tokens each at margin 0,
**both removed = 7,378**, and the **15 margin-0 tokens jointly worth 40,917
(46.1 %)**. **ZEROS DO NOT COMPOSE.** So the grammar steps a marginal re-score
would most confidently demote are exactly the ones the re-scorer is *proven
blind to*. Ten slices each measuring ~0 is consistent with Phase 1 being
worthless **and** with it being worth 46 % jointly; the marginals cannot
separate those, and the whole stop/continue decision turns on which is true.

**The method is prescribed, not invented here.** `w-loo`'s standing
instruction: *"do not dispatch off any ranking — greedy or LOO. If you need to
size work, use a **ladder built by lifting the clause in a scratch tree**
(three compiles) and report **subset structure**, not marginals — and state
the **denominator in the same breath as the numerator**, in the prereg, before
the run."* And the reason a ladder is mandatory rather than a histogram:
**the port stops at the first refusal BY DESIGN, so every blocked body reports
exactly one blocker no matter how many it has** — a first-blocker key is not a
distance, and the coordinator has previously narrated one as "one refusal
left" for hours before `w-mixed`'s ladder found three more beneath it.

Four lanes, dispatched concurrently on Opus subagents:

| lane | kind | what it buys | price carried in |
|---|---|---|---|
| `w-joint` | characterization + instrument (**primary**) | the FULL blocker set per blocked function on the nearest TUs, by **ladder**, reported as **subset structure**. Answers: is there any TU whose complete blocker set lies within a bounded construct set, and does closure COMPOSE? Commits the ladder tool — `w-mixed`'s and `w-loo`'s both died in gitignored `work/`, the third such loss after `#3451` | `w-loo` `#3140` · `#3131` · `w-mixed` · `w-c7` §2 |
| `w-latent` | construct | `#3493` found a surface where **three mutations that gut or poison it are byte-identical across 627 unit tests AND 391 fixtures against real c2**. Hunt for others: which shipped axes can be arbitrarily wrong with nothing able to observe it? | `w-permute` `#3493` |
| `w-relread` | characterization | `WB_RELATION_FINDINGS.md` §5's three ranked reads, the first at ~½ day, which retires `#423`'s 36-cell probe grid entirely — read-before-probe applied to a grid that was already priced | `w-c7` §5 |
| `w-adjacency` | instrument | build the adjacency-balanced rotation `w-permute` diagnosed and deliberately did not build, and pay the re-run it owes to `w-s1bc`, `w-s1c2`, `w-s1c3` | `w-permute` `#3495` |

**`w-joint` must not produce another ranking.** The standing rule has now
bound **five times**, most recently *before* dispatch in decision 10. Its
deliverable is subset structure and a composition verdict, never an ordered
list of keys to work on. It must also interrogate the distance instrument it
starts from: `STATUS.md` reads `TU distance to match, blocked functions ≤0:
17` while `match` is 25, and `#3364` already found `FBM` refusing bodies of
TUs that match byte-exact — so the denominator is suspect before it is used.

**Concurrency fences.** `w-joint` owns `crates/c2-harness` (instrument +
tests) and `scripts/` for its ladder tool. `w-latent` is analysis-first: it
may add tests in NEW files only and must STOP-AND-REPORT before modifying any
existing `crates/` file. `w-adjacency` owns `scripts/cost_arms.py`.
`w-relread` is docs-only, zero `crates/` bytes.

**Deliberately not dispatched:** every Phase-1 slice (held pending `w-joint`);
C7's one-arm `select.rs:435` variant (a live option the owner did not take,
recorded so it stays takeable); `#3492`'s T3 checklist over prior extractions;
corpus pinning (~14 GB CoW, named and priced by `w-permute`, not built);
`0x10c3b270`; `nopcapenter`/`nopcapexit`; the three `cgintrin.c` sites (now
declined by three lanes); `tuple[+9]` bit 3 and `tuple[+0x34]`; the generated
`STATUS.md` block's missing workload identity; `WB_EXPAND_FINDINGS.md:79` and
`#3432`'s stale "unrecorded" sentence — **three lanes have declined it now**;
the maintenance debt (`#3381`, the phantom `PROGRESS_METRIC.md` §5.2 citation,
`DIFF_STRUCTURE.md`).

**Board:** `#3505` this decision · `#3506`–`#3511` `w-joint` ·
`#3512`–`#3516` `w-latent` · `#3517`–`#3520` `w-relread` ·
`#3521`–`#3524` `w-adjacency`. Next free `#3525`.

## Decision 12 — wave 10: the #3509 read is funded, the w-biquad binaries are removed without a history rewrite, and the wave is funded whole (2026-08-25)

The owner, verbatim, in reply to the structural review's three asks:

> *"#1 funded. #2. remove from git and skip rewriting history. #3 fund it.
> all of these are solid. please orchestrate opus subagents"*

**What this decides**, in the order asked:

1. **The `#3509` key→slice read is FUNDED** (0.5–1 day). Phase 1's ten
   constructs have never been mapped to census keys — 3 of 10 carry an
   opcode, 7 are prose — so prices are in **bodies**, arguments are in
   **TUs**, and nothing in the tree converts between them. Until it runs,
   *"can Phase 1 convert a TU"* is not decidable from any measurement here,
   and decision 11's hold cannot be lifted or released on evidence. The
   cheap half is already on disk (`docs/data/census_key_populations.tsv`,
   785 blocker keys, sorted by NAME so it cannot be read as a ranking).
2. **The 21 forbidden binaries under `work/w-biquad/` are removed from
   `HEAD`, and history is NOT rewritten** — done by the coordinator before
   this wave was dispatched (`2c0de2ad4`). `git rm --cached` only: the files
   stay on disk, ignored; every commit hash since 2026-08-09 is untouched.
   **`#3156` undercounted by 10** — it named the 11 `.obj` and never counted
   the 10 `_CL_*` IL captures, which appeared on no row at all. The row is
   corrected and closed from `HEAD`; `work/w-biquad/REMOVED_ARTIFACTS.md`
   carries the regeneration path per class. **The funnel check `#3156`
   prescribes is still not built**, and is item 1 of the hygiene lane below.
3. **Wave 10 is funded whole**, dispatched as Opus subagents under the
   coordinator, per the three tracks of `ROADMAP.md` §11.6.

**What this does NOT decide.** Phase 1 stays **held** — this wave funds the
read that makes the ruling possible, not the ruling. No Phase-1 slice is
dispatched. Rows 4a/4b remain unapproved; step 5's NO-GO clauses stand; the
byte judge is untouched; a wrong emit still scores below the refusal it
replaced.

Four lanes, dispatched concurrently on Opus subagents:

| lane | kind | what it buys | price carried in |
|---|---|---|---|
| `w-keymap` | characterization (**primary**) | **`#3509`**: map Phase 1's ten constructs to census keys by READING the decode sites, and publish the TU-denominated reach of each. The deliverable is the artifact the whole stop/continue debate has been conducted without. **Must not produce a ranking** (`#3505`, bound five times) and must state which of the ten it CANNOT key rather than guessing | `w-joint3` rung §5, §9 item 1 · `#3509` |
| `w-permeasure` | characterization | the **permuter pre-measurement**: are hand-written decomp near-misses inlining-dominated the way the port's own wrong-body population is? `DIFF_STRUCTURE.md` measured the port's failures (0 pure reorderings, 5,173 of 5,189 substituted words differ in opcode); **nothing has measured the owner's actual permuter case**, and `#3369` is that conflation. One day of measuring decides which permuter to build; building first risks weeks on the wrong search space | `DECISIONS_2026-08-22.md` § "Recorded from the same brief" · `#3369` |
| `w-hygiene` | construct | five named defects, each blocking something and each priced: **experiment F** (~15 min, registered-unrun, blocks every historical cost re-run and `#3523`'s attribution); the **`repo_root()` runtime fix** (one function — closes `#3470` AND `#3525` from one site); `scripts/configure_existing_worktree.sh`'s **two** defects (`#3500` silent no-op, `#3516` the `bench` mislabel); the `#3513` NUL fix (one character × 3); and the `#3156` funnel check that is still not built | `w-adjacency` rung §7.3, §7.7 · `w-3475` §10.1 · `w-latent` §8.1, §8.4 |
| `w-relsite` | characterization | `w-relread`'s top-ranked follow-on (~½ day): the IL-opcode → relation-code site, **still unnamed after two lanes**, which closes `w-c7`'s W2. Read-before-probe: this retires a question two lanes argued from tables instead of from the code | `w-relread` rung §9 item 1 |

**Concurrency fences.** `w-hygiene` is the only lane permitted to edit
`crates/` (`c2-reference/src/lib.rs`, `c2-il/.../bundle.rs`) and `scripts/`;
it owns both. `w-keymap`, `w-permeasure` and `w-relsite` are docs-only, zero
`crates/` bytes, and STOP-AND-REPORT if a `crates/` edit looks necessary.
`w-keymap` and `w-relsite` both read `crates/c2-il` decode sites — **reading
is unfenced, writing is not**.

**Deliberately not dispatched:** every Phase-1 slice (still held — this wave
produces the evidence, and the ruling is the owner's); C7's one-arm
`select.rs:435` variant (still takeable); the against-zero emitters (~1 day,
queued behind `w-relsite` which shares its subject); `DIFF_STRUCTURE.md`'s
rescan (cheap, but `w-permeasure` may moot its framing); `#3510`'s diagnosis
(named in `w-hygiene`'s brief as a stretch, not a deliverable); the
`STATUS.md` block regeneration (needs a quiet box, coordinator's); a history
rewrite of the w-biquad binaries (owner declined, deliberately).

**Board:** `#3527` this decision · `#3528`–`#3533` `w-keymap` ·
`#3534`–`#3538` `w-permeasure` · `#3539`–`#3545` `w-hygiene` ·
`#3546`–`#3550` `w-relsite`. Next free `#3551`.

**Nothing is waiting on the owner.**

## Decision 13 — the GENERAL DECODE is funded: row 4a(i) / I1 (2026-08-25)

The owner, verbatim, on the wave-10 close and its Phase-1 recommendation:

> *"okay lets fund general decode now"*

**What this decides.** `ARCHITECTURE_PROPOSAL_2026-08-20.md` §8 row **4a(i)**
— *a general op-level IL decode* — is **funded and started**. This is the
row `STEP5_PRICING_2026-08-21.md` §2/§4 prices at **I1 raw 1.5–4.5
engineer-months, and 15–45 engineer-months as a LOWER BOUND** once
`CEILING` §5's ~5:1 calibration is applied (4a as a whole; I1 is its first
half). It is the **critical path**: without it step 5 cannot reach the byte
judge at all, and 4a's own risk column names the failure mode — every lane
lands an *unconsumable instrument*, `#3336` at program scale with no
contrast case to catch it.

**Why this row and not Phase 1.** The `#3509` read the owner funded as
decision 12 returned **TU reach 0** — per construct, jointly, and granting
every byteless key free (`#3529`); Phase 1's ten constructs move the per-TU
construct floor from median 187 to 147. `codegen-gap` is **0 over all 878
TUs** and every non-matching TU is `vocab-gap`: the port is not blocked on
generating PowerPC, it is blocked on **reading IL**. That is 4a(i)'s
subject.

**What this does NOT decide.** It does not fund **4a(ii)** (the general
lowering to `coff::Function`) or **4b** (IR3 in tuple/region coordinates),
and it does not approve **step 5** — whose NO-GO clauses (a)–(c) stand.
**It does not formally rule on Phase 1**: decision 11's hold is not lifted
here, no Phase-1 slice is dispatched, and if the owner wants Phase 1 closed
in the record that is a one-line decision that has not been taken. What has
changed is the route being funded, not a ruling on the other.

### The correction this decision carries, because two of my own documents repeat it

**"189 arms" IS AN OPCODE COUNT AND THERE ARE 61 ARMS.** `P_ILRECORD.md`'s
own ⛔ banner: the switch is MSVC's **two-level** form (byte index table,
then a DWORD target table), so **189 opcodes → 62 distinct arms**, of which
**94 of the 189 opcodes route to a single arm that raises C1001** — the real
read is **61 arms serving 95 opcodes, plus one refusal**. `READ_PLAN` §3/§4,
`C2_MAP.md:1012`, `STEP5_PRICING:139`, `WHITEBOX_LEVERAGE:89`,
`ARCHITECTURE_PROPOSAL:968` **and decision 6 and decision 12 of this file**
all say 189. The dispatch a general decode must implement is **three times
smaller than every planning document in this tree states**, and that is a
re-pricing input, not a footnote.

### The architectural fact the wave is designed around

4a(i)'s own wording: *"IR0 stops at a two-variant byte framing and
`BodyShape` starts at 35 whole-function grammars **that are simultaneously
the admission gate**, so the semantic middle a COLOR pass would consume does
not exist."* **Decode and admission are fused.** That is why widening reach
has always meant widening emission — and `S0` measured what naive widening
ships: **blind-differs 96.1 %** of what it reached. So the wave's first
move is to **unfuse them**: decode generally, admit exactly as
conservatively as today. Under `PROGRESS_METRIC.md` a wrong emit still
scores strictly below the refusal it replaced, and nothing here relaxes
that.

Four lanes, dispatched concurrently on Opus subagents:

| lane | kind | what it buys | price carried in |
|---|---|---|---|
| `w-unfuse` | construct (**primary**) | separate DECODE from ADMISSION in the `BodyShape` path, so a body can be decoded without thereby being admitted. **Required-zero byte delta** and a required-zero identity diff: today's admitted set must be identical, function for function, after the split. This is the prerequisite every later I1 slice depends on | `ARCHITECTURE_PROPOSAL` row 4a(i) · `S0` blind-differs 96.1 % |
| `w-decodereach` | instrument | the signal I1 has no progress measure without: how many bodies the general decode **reaches**, published beside FBM under FBM's separation rule — **never in `gate.sh`, licenses no emit**. 4a's risk column says an unconsumable instrument is the failure mode; this is the consumer | `w-joint3` §9 item 4 (the parse-layer lifter) · `ROADMAP_SLICING` §5 S0 · `#3336` |
| `w-ilarms` | characterization | reconcile `P_ILRECORD.md`'s **61 arms / 95 opcodes** against the port's existing decode vocabulary and publish the arm → port-site map, with the 94 C1001 opcodes named as out of scope. The artifact every later I1 slice is sized from — and the first chance to catch the 189 error's consequences | `P_ILRECORD.md` §1.1–§1.3 · `#3421` |
| `w-guard` | construct | `#3552`'s reap guard (three consecutive actors destroyed pinned artifacts); explicit toolchain pinning for any re-run of a PRE-`repo_root()`-fix commit (`#3470` bites backwards and cannot be fixed forwards); and `#3510`'s diagnosis, which voids the emit-set ceiling `STATUS.md` publishes | `#3552` · `#3470` · `#3510` |

**Concurrency fences.** `w-unfuse` owns the `crates/c2-il` decode/admission
sites and may not touch `crates/c2-harness`. `w-decodereach` owns
`crates/c2-harness` and may READ `c2-il` but not write it. `w-ilarms` is
docs-only, zero `crates/` bytes. `w-guard` owns `scripts/` and
`crates/c2-harness/tests/`. Reading is unfenced; writing is not. Any lane
that needs a peer's file **STOPS and reports** — the wave-7 precedent.

**Merge order: `w-unfuse` first**, because `w-decodereach` measures the
surface it creates; the instrument rebases over it and re-measures rather
than assuming.

**Deliberately not dispatched:** 4a(ii) and 4b (this decision funds I1
only); any Phase-1 slice; step 5; the against-zero emitters; the three
historical cost re-runs (now known to be **inside build noise** — `#3551`,
so they need the floor quoted beside them, not just a re-run);
`DIFF_STRUCTURE.md`'s edit; the `STATUS.md` block regeneration
(coordinator's, needs a quiet box).

**Board:** `#3553` this decision · `#3554`–`#3560` `w-unfuse` ·
`#3561`–`#3566` `w-decodereach` · `#3567`–`#3572` `w-ilarms` ·
`#3573`–`#3578` `w-guard`. Next free `#3579`.

**Nothing is waiting on the owner.**

---

## Decision 14 — the five follow-ons wave 11 named are funded (2026-08-26)

**Asked and answered in one turn.** Wave 11 closed with four follow-ons
named, specified, and deliberately not taken — each because it crossed a
lane's fence, not because anybody judged it unaffordable — plus one open
row the coordinator filed against its own regeneration (`#3583`). The
owner's instruction was to dig into each and get answers, and that nothing
here needs them. **All five are funded.**

**Nothing in this decision relaxes the correctness rule.** Real `c2` under
wibo plus a byte-exact obj compare is still the sole judge, a wrong emit
still scores strictly below the refusal it replaced, and every lane below
that touches `crates/` carries a **required-zero byte delta** graded by the
21-row identity diff — now with a committed instrument
(`scripts/gate_identity_diff.sh`, `#3579`) rather than a retyped procedure.

### The five lanes

| lane | kind | the question it must answer | why now |
|---|---|---|---|
| `w-opclass` | characterization | **Read the 29 operand-class arms at `0x10b3d954`.** `w-ilarms` published that **65 of its 68 `MATCHED*` rows have limb 2 of the NARROW test genuinely unchecked** — it did not budget these arms. One read closes all 65 at once. It must also adjudicate the two hazards `w-ilarms` reported *beside* its count rather than folding in: `0x28` as `NARROW(fields)`, and arms 17/26's `0x43` escape **that does not exist in the real dispatch** | the cheapest large closure on the board: 65 unchecked rows, one budgeted read, and a live hazard where the port is currently right by coincidence |
| `w-atend` | construct | **Can the admission layer own a refusal reason at all?** `w-unfuse` shipped `AdmissionPolicy` with **one** variant because a `Block` says where the *read* stopped, so a policy refusing a body the decode read whole has no key — and minting one publishes a `:eof` census key **no scan can reach**. The named follow-up is `EXPECTED_AT_END_SITES` 7 → 8. **If the answer is no, `declined` with the two-sided price is the correct outcome** and the lane must say so | it is the one thing blocking `AdmissionPolicy` from being a policy rather than an identity |
| `w-symbind` | instrument | **Is symbol binding a third fused layer, and is it separable the way the grammar layer was?** `w-decodereach` measured `grammar-not-admitted` = **4,001** bodies, all `:eof`, all symbol binding, refused at `shape_to_function` (`census.rs:957`) **downstream of `AdmissionPolicy`**. `w-unfuse` unfused the grammar layer only | 4,001 bodies is the first quantified estimate of what the next unfusing is worth, and nobody has looked at the layer |
| `w-price4a` | characterization | **Re-price row 4a(i) / I1 against inputs that are now known to be wrong.** The live 15–45 engineer-month figure was written against **189 arms** (really **61**), **83.5 % off-model** (arithmetically **refuted**; really 88.61 %), and a **98.2 % reach** that is *framing*, not model reach (really **11.39 %**). Three of its four inputs moved | a price standing on a refuted premise is worse than no price, and this row is the critical path |
| `w-perfstep` | instrument | **Resolve `#3583`** — the published speedup moved 664× → 553× (≈15 %) with a same-session re-run at 571× (spread ≈3 %). Three candidates are named and none chosen: the workload advanced under both arms; `w-hygiene`'s `repo_root()` now resolves at **runtime**; box state across a day | a published metric moved an order beyond its own spread and the tree currently cannot say why |

### Concurrency fences — writing is fenced, reading is not

`w-opclass` owns **`docs/whitebox/**`** (including `READ_PLAN_2026-08-21.md`)
and writes **zero `crates/` bytes**. `w-atend` owns **`crates/c2-il/**`** plus
**`crates/c2-harness/tests/fence_site_census.rs`**. `w-symbind` owns
**`crates/c2-harness/src/gap/**`**. `w-price4a` owns the **top-level pricing
docs** — `ARCHITECTURE_PROPOSAL_2026-08-20.md`, `STEP5_PRICING_2026-08-21.md`,
`ROADMAP_SLICING_2026-08-21.md`, `WHITEBOX_LEVERAGE_2026-08-21.md`, `ROADMAP.md`
— and **may not write `docs/whitebox/`**. `w-perfstep` owns **`scripts/**`** and
**`crates/c2-harness/src/perf.rs`**. Every lane writes its own rung file and
appends its own board rows. **A lane that needs a peer's file STOPS and
reports** — the wave-7 precedent, honoured by `w-unfuse` in wave 11 when it
hit `fence_site_census.rs` and stopped.

**No merge order is imposed.** Wave 11 needed one because the instrument
measured the surface the construct created; these five are independent.
`w-price4a` is told explicitly that `w-opclass` is in flight and may sharpen
limb 2 — it prices off what has **landed**, states that input as of its own
tree, and does not wait.

### What every lane is told, and what none of them may do

- **Prereg first, committed before the first measurement**, and graded
  honestly at the end. Wave 11 produced two lanes whose registered brackets
  MISSED and both said so; that is the standard.
- **Read-before-probe** (`WHITEBOX_LEVERAGE_2026-08-21.md`) — price the
  binary read that would answer the question before any probe grid.
- **No lane widens emission.** Not one of these five is authorized to move
  the admitted set. `w-symbind` in particular measures a refusal population
  and may not convert it.
- **Read the gate's verdict line, never its exit code.** Both failure
  shapes appeared in wave 11: `GATE: REFUSED (DIRTY crates/)` at exit 0,
  and `GATE: PASS` at exit 1 from the tree-movement guard.
- **Do not reap or unlock a peer's worktree**, and check
  `scripts/wt_pin_audit.sh` before reaping your own (`#3552`, three losses
  in three waves).
- **A lane that produced none of its deliverable says `FAILED`** in those
  words. `declined` with a two-sided price is a legitimate outcome for
  `w-atend` and is not a failure.

### Deliberately not dispatched

**Phase 1 stays HELD** — decision 11's hold is the owner's and no lane here
touches it. Also not funded: 4a(ii) / I2, 4b / IR3, step 5, the
against-zero emitters, `DIFF_STRUCTURE.md`'s edit, and any repair of the
emit-set ceiling **predicate** (`#3577` diagnosed it; `STATUS.md` now
renders both ceilings VOID, which is the honest state until somebody funds
the repair).

**Board:** `#3584` this decision · `#3585`–`#3590` `w-opclass` ·
`#3591`–`#3596` `w-atend` · `#3597`–`#3602` `w-symbind` ·
`#3603`–`#3608` `w-price4a` · `#3609`–`#3614` `w-perfstep`.
Next free `#3615`.

**Nothing is waiting on the owner.**

## Decision 15 — OWNER DECISION: the goal is restructured onto per-subsystem scoreboards; the instrument wave that makes them trackable is funded (2026-08-26)

**The owner's words, which this section implements:** *"lets restructure our
goal so that we can get each submodule in shape and have measurements for
that. the overall TU goal is too broad because it is binary. we need a
smarter goal. for example, inliner is extremely valuable to understanding how
that logic works in the compiler. focus your effort on this right now before
we resume the broader goal. focus on building tools we can use to measure our
progress for each unit. we are breaking down a monolithic task into discrete,
smaller subtasks."*

### What changes and what does not

- **The working scoreboard becomes per-subsystem.** Dispatch criterion moves
  from "what moves `match`" to "what moves a named subsystem's tuple".
  `SUBSYS.md` §1's ten subsystems are the unit list; each gets a metric tuple
  with a **published denominator taken from the whitebox read**.
- **The correctness rule is untouched.** Real `c2` under wibo + byte-exact
  obj compare stays the sole judge; a wrong emit still scores below the
  refusal it replaced. Per-subsystem keys are **progress instruments under
  `FUNCTION_BYTE_MATCH.md` §0's separation rule — published beside FBM,
  never in `gate.sh`, licensing no emit.**
- **`match`/`fnbyte-exact` stay measured** as goal (2)'s terminal metric.
  They stop being the *dispatch* criterion, which is what "binary" costs us:
  a subsystem can go from 20% to 90% understood with `match` unchanged, and
  until now that progress was invisible to the roadmap.
- **The broader-goal items are paused, not cancelled** — the one-arm
  byte-judged slice, 4a(ii)/I2, 4b/IR3, step 5 wait behind this wave per the
  owner's "before we resume the broader goal". **Phase 1 stays HELD**
  (decision 11, the owner's hold).

### The metric shape every subsystem gets

A 4-tuple, never a single percentage, each strength with its denominator
printed beside it (`w-decodereach`'s pattern, the proven template):

1. **read** — of the subsystem's enumerable sites in the image, how many the
   port implements (denominator: the P_*.md coverage line, re-measured);
2. **agreement** — of those, how many match the read spec under a
   differential;
3. **exercised** — how many the 878-TU workload reaches (an unexercised site
   is unverified, not done);
4. **byte-owned** — of the bytes the judge grades, how many flow through it.
   **Not re-measured this wave: `#3534` measured it 2026-08-25** (port wrong
   bodies = 99.87% opcode substitutions, 0 reorderings, 92.78% wrong at
   word 0, both sides one tree one day). The framework *cites* it; a
   standalone byte-ownership lane would re-take a read already taken.

**The signal is the CHANGE in each strength, never its distance from 0 or
100.** A green subsystem row is a statement about the population the
instrument can reach — every denominator says which tree it was taken on.

### The three lanes

| lane | kind | deliverable | why |
|---|---|---|---|
| `w-submetric` | instrument | The committed per-subsystem metric instrument: one runnable that prints the tuple per `SUBSYS.md` §1 row with denominators and a workload stamp, rendered to a committed doc; strengths it cannot yet measure print as a **named residue**, never silence | the tool the restructured goal is tracked with; without it the new goal is prose |
| `w-inlmetric` | characterization + instrument | **The inliner's own scoreboard** — the owner's named exemplar. Clause-by-clause conformance table for c2's decision function (`P_INLINE.md` §1–§2: ceiling `16<<k`, legality bits, `__forceinline`/`noinline`, the favor-speed bit, POGO path, depth/budget, the 40-instruction test) → port state (`[R]`-derived / fitted / absent) → graded agreement where measurable. Re-freeze `INLINE-P`'s hold-out **by content hash** (`#3045`'s named fix) and re-grade with denominator beside rate | the richest-read subsystem with a fitted incumbent whose blind axes are already measured (right in fitted class; wrong on a flag axis and a LOOP axis) — the exemplar the other nine copy |
| `w-provenance` | instrument | The **derived-vs-fitted census**: a greppable provenance convention for load-bearing constants/rules in `crates/`, seeded from `DISCLOSURE.md`'s rows, plus the per-module counter script with a positive control watched failing | goal (1)'s scoreboard. Today `crates/` carries **6** provenance markers against a 247-line `DISCLOSURE.md` — the ratio that most directly tracks "understanding MSVC internals" is invisible |

### Concurrency fences — writing is fenced, reading is not

`w-submetric` owns **`crates/c2-harness/**`**, new `scripts/subsys_*` files,
and its rendered doc (new file). `w-inlmetric` owns **`P_INLINE.md`**,
**`INLINE_PREDICATE.md`**, **`WB_INLINE_FINDINGS.md`** (amend-beside only),
and `work/w-inlmetric/**`; it writes **zero `crates/` bytes** and may not
ship any decision rule into the port (`INLINE_PREDICATE.md`'s own standing
rule). `w-provenance` owns **comment-only** edits in `crates/c2-core`,
`c2-il`, `c2-obj`, `c2-reference` (NOT `c2-harness` — that is `w-submetric`'s
this wave), **`DISCLOSURE.md`**, and new `scripts/provenance_*` files. Every
lane writes its own rung file and appends only its own board rows. Owned
surfaces include **predicates, keys and facts, not just files** — two lanes
building a reader for the same quantity is a collision even when git
auto-merges it. A lane that needs a peer's surface STOPS and reports.

### What every lane is told, and what none of them may do

- **Prereg first, committed before the first measurement**, graded honestly.
- **Read-before-probe** stands.
- **No lane widens emission.** No lane moves the admitted set. Lanes
  touching `crates/` (including comment-only edits) carry a
  **required-zero byte delta** graded by the 21-row identity diff
  (`scripts/gate_identity_diff.sh`).
- **Re-measure every denominator on your own tree** with the workload stamp
  recorded beside it; the coordinator has NOT verified the addresses or
  figures quoted in your brief.
- **Read the gate's verdict line, never its exit code**; run the gate
  detached (it can exceed the 600 s cap) and never commit while one runs.
- **Do not reap or unlock a peer's worktree**; check
  `scripts/wt_pin_audit.sh` before reaping your own.
- **A lane that produced none of its deliverable says `FAILED`** in those
  words; a priced `declined` is a legitimate outcome.

### Deliberately not dispatched

A standalone byte-ownership lane (`#3534` already measured it, this week);
any repair of the emit-set ceiling predicate (`#3577`); the
`expr_sweep` pin-by-name lane (`#3614`); everything listed as paused above.

**Board:** `#3616` this decision · `#3617`–`#3622` `w-submetric` ·
`#3623`–`#3628` `w-inlmetric` · `#3629`–`#3634` `w-provenance`.
Next free `#3635`.

**Nothing further is waiting on the owner.**

## Decision 16 — wave 14: the four follow-ups wave 13 named are funded as three lanes (2026-08-26)

**Asked and answered in one turn.** Wave 13 built decision 15's instruments
and each lane named what it deliberately did not take. The owner's
instruction was to do these now. **All four items are funded, as three
lanes** — items 3 and 4 are one lane because they share a fact (the census
and its tagged population), and two lanes building readers over one quantity
is this repo's most-recorded silent merge failure.

**Nothing here relaxes the correctness rule**, and nothing here widens
emission. Every lane touching `crates/` carries a **required-zero byte
delta** graded by `scripts/gate_identity_diff.sh` over the 21 count-bearing
rows. Decision 15's frame is unchanged: these are per-subsystem progress
instruments, published beside FBM under `FUNCTION_BYTE_MATCH.md` §0, never
in `gate.sh`, licensing no emit.

### The three lanes

| lane | kind | the question it must answer | why now |
|---|---|---|---|
| `w-encmap` | characterization + construct | **Convert `ported` from residue to a number on the encoder row** — the cheapest single conversion `w-submetric` named (`#3617`). The enumeration: for each of the **79 arms** / **111 jump-table entries**, which port function composes that form, and which arms nothing implements. **First, adjudicate a possible duplicate reader**: `encode.rs`'s module doc still declares itself *"a black-box re-derivation … nothing here changes"* while `mop.rs` now carries `OPCODES` (base words `0x10c3a578`, forms `0x10c39b18`) tagged `PROV[R]`. Are these two encoders for one fact? **`#3634`'s claim that the black-box re-derivation was "retired" is a CENSUS-of-CONSTANTS reading and may be an over-read of the functions** — the lane adjudicates and corrects whichever side is wrong | `ported` is the strength decision 15 named first and no instrument could reach; the encoder is the best-read subsystem, so it is where the map is cheapest — and the duplicate-reader question is a live correctness hazard, not bookkeeping |
| `w-disclose` | characterization | **File the missing ledger rows** `#3632` found and deliberately did not repair: `codegen/mop.rs`'s **88** read constants on the emit path (c2's opcode indices `0x10b1b260`, base words `0x10c3a578`, forms `0x10c39b18`, field placements) and `EX_CLASS_TABLE = 0x10b25e48`. **`mop.rs`'s module doc asserts DISCLOSURE carries these rows; it does not** — that is a false provenance claim in the file that holds the port's only source of a primary opcode | filing a row is an **adoption decision** and moves `ref/README.md`, which is why the census lane correctly stopped. It is the highest-value hole in the provenance record: the constants nearest the judge are the ones with no ledger |
| `w-provext` | instrument | **Two halves of one fact.** (a) `--since <sha>` two-tree diff mode for `provenance_census.py`, so the **tracked signal is the CHANGE per module**, which is what decision 15 says is the signal and what the tool cannot currently print. (b) Extend tagging to the residue the wave left: `c2-il/src/func/**` (**290**), `c2-obj/src` (**43**), `c2-harness/**` (**214**, fenced to a peer last wave) | the census reads a LEVEL and the goal tracks a DELTA; and `c2-il`'s decode vocabulary is where `[F]` should actually live — the census found `[R]` beating `[F]` 100 to 4 and read that as the instrument measuring its own 18 % coverage |

### Concurrency fences — writing is fenced, reading is not

`w-encmap` owns **`docs/whitebox/ref/P_ENCODE.md`**,
**`crates/c2-harness/src/subsys.rs`** + **`src/cli/subsys.rs`**,
**`docs/SUBSYS_METRICS.md`**, `scripts/subsys_metrics.sh`, and
`work/w-encmap/**`. `w-disclose` owns **`docs/whitebox/DISCLOSURE.md`**,
**`docs/whitebox/ref/README.md`**, comment-only edits in
**`crates/c2-core/src/codegen/**`**, and `work/w-disclose/**`.
`w-provext` owns **`scripts/provenance_census.py`**, comment-only edits in
**`crates/c2-il/**`**, **`crates/c2-obj/**`** and **`crates/c2-harness/**`
EXCEPT `subsys.rs` and `cli/subsys.rs`**, the committed census snapshot, and
`work/w-provext/**`.

**Three shared facts are named, because file-ownership deconfliction is
necessary and NOT sufficient here — it has failed three times:**

1. **The `PROV[X]` marker convention** is `w-provext`'s to apply and
   `w-disclose`'s to cite. Neither may change its grammar. A change to
   `MARK_RE` is a STOP-and-report.
2. **The DISCLOSURE row namespace** is `w-disclose`'s alone. `w-provext`
   tags may cite existing rows and may **not** mint new ones.
3. **The census counts.** `w-provext` will move every per-module number.
   Its `--since` tests must therefore pin **planted-fixture** counts or
   invariants, **never live per-module tree counts** — a peer's tagging must
   not be able to redden them. `w-encmap` must not add a second provenance
   or census reader of any kind.

A lane that needs a peer's surface **STOPS and reports** rather than
resolving it.

### What every lane is told, and what none of them may do

- **Prereg first, committed before the first measurement**, graded honestly
  at the end; a MISS is said in that word.
- **Read-before-probe** stands. **And check the read index's ALREADY-READ
  half, not just the todo half** — a wave-12 lane was funded to take a read
  that had already been taken four times.
- **Grep every board row number this brief cites** before writing prereg;
  corrections live in rows that *cite* the row they correct.
- **No lane widens emission.** Required-zero byte delta on the 21 rows for
  anything touching `crates/`, including comment-only edits.
- **Re-measure every denominator on your own tree** with the stamp beside
  it; the coordinator has NOT verified the figures in these briefs. The
  79 / 111 / 88 / 290 / 43 / 214 numbers are all quoted from other lanes'
  reports and are the lane's to confirm or correct.
- **Read the gate's verdict line, never the exit code**; run it detached
  (it exceeds the 600 s cap) and never commit while one runs.
- **A lane that produced none of its deliverable says `FAILED`** in those
  words; a priced `declined` is legitimate.

### Deliberately not dispatched

`FUNCTION_BYTE_MATCH.md` §0's sixth-gradient cross-link box — named by
`w-submetric` as outside every fence, and it stays outside this wave's too;
it needs an owner, not a squeeze. Also not funded: `P_LABEL`'s unreproducible
`25`; `build_ref.py`'s missing `PAGE_SUBSYS` entries; the `c2rs subsys` EPIPE
cosmetic; `codegen/**`'s 205 untagged per-fixture register numbers (expect
`[O]`, expect tedium — it is volume, not a question); the emit-set ceiling
predicate repair (`#3577`); the `expr_sweep` pin-by-name lane (`#3614`).
**Phase 1 stays HELD.**

**Board:** `#3635` this decision · `#3636`–`#3641` `w-encmap` ·
`#3642`–`#3647` `w-disclose` · `#3648`–`#3653` `w-provext`.
Next free `#3654`.

**Nothing is waiting on the owner.**

## Decision 17 — wave 15: the duplicate producer is repaired as a CONSTRUCT rung, section gets `ported`, and the prose-truth gap gets an instrument (2026-08-26)

**Asked and answered in one turn.** Wave 14 closed with three findings that
outrank its own deliverables and two one-line repairs no lane could make
inside a peer's fence. The owner's instruction was to dispatch them. **All
are funded.**

**Nothing here relaxes the correctness rule.** One lane touches the emit
path and it is a **construct rung**: `Fixtures: none`, `Census: +0`,
**required-zero byte delta**, graded by an identity diff of the 21
count-bearing rows. **It may not change one emitted byte.** If byte
neutrality turns out to be impossible, that is a **STOP-and-price**, not a
licence — a wrong emit still scores strictly below the refusal it replaced.

### The three lanes

| lane | kind | the question it must answer | why now |
|---|---|---|---|
| `w-mopfold` | **construct rung** | **Fold the seven second-rule word compositions onto the table that already covers them** — `calls.rs:36/98/102/106` (`b`, `addis`, `addi`, `li`) and `frame.rs:54/57/63/74` (`stw`, `lwz`, `mtspr`, `lwz`), board `#3637`. They agree with `mop` today, verified by hand to the bit. **The durable deliverable is not the fold — it is the CONTROL that makes the next one impossible**: `w-encmap`'s enumeration (every live non-`cfg(test)` word production in `crates/c2-core/src`) must become a test that fails when a new second producer appears. Also repair `#3638`'s three false pledges that `base_word` is the port's only source of a primary opcode | **the gate is structurally blind to a CONCURRING second producer.** `mismatch 0` is silent about it, and it stays silent right up until the day the two rules disagree — at which point the defect is indistinguishable from a lowering bug. This is the only class of defect the project's sole judge cannot see |
| `w-secported` | characterization + construct | **Convert `ported` from residue to a number on the `section` row.** `w-encmap` named it the only other subsystem whose sites are **rules rather than addresses** — the property that made the encoder cost a join instead of a read. The enumeration: `P_SECTION.md`'s `.gl` record dispatcher `0x10b9b8e9`, its **27-entry byte-index table `0x10b9c615`** and 16-entry jump table `0x10b9c5d5`, plus the section-kind decision `0x10b982d6` whose arms are already `[O]` by `.gl` mutation | one of ten rows now has all four strengths as numbers. A second row proves the encoder was not a special case — and if it is a special case, **that is the finding**, and `declined` with the reason is the right outcome |
| `w-provaudit` | instrument | **Build the instrument for the class nothing covers: a COUNT OR CLAIM INSIDE a marker or doc that is FALSE.** `#3643` (a `PROV[R]` marker said "71 distinct opcodes" where the truth is 85, wrong since the file's first commit, correct figure one comment away) and `#3641` (writing prose *about* mark letters moved a census 9/28 → 13/34, because the counter cannot tell a mark from a mention) are one family. **Every control this repo owns fabricates a NUMBER; this defect fabricates none.** Plus the two queued repairs: `#3645` and `#3644` | the census now covers 78 % of constants and can say whether each is tagged. It cannot say whether the tag is **true**. That is the difference between a provenance record and a provenance *claim*, and goal (1) rests on the former |

### Concurrency fences — writing is fenced, reading is not

`w-mopfold` owns **`crates/c2-core/src/codegen/**`** (code and comments —
`calls.rs`, `frame.rs`, `mop.rs`, `encode.rs`) and `work/w-mopfold/**`.
`w-secported` owns **`docs/whitebox/ref/P_SECTION.md`**,
**`crates/c2-harness/src/subsys.rs`** + **`src/cli/subsys.rs`**,
**`docs/SUBSYS_METRICS.md`**, `scripts/subsys_metrics.sh`, and
`work/w-secported/**`. `w-provaudit` owns **root `README.md`**,
**comment-only** edits in `crates/c2-reference/**`,
**`docs/whitebox/DISCLOSURE.md`**, **`docs/whitebox/ref/README.md`**,
**`scripts/provenance_census.py`** and new `scripts/prose_*` files, and
`work/w-provaudit/**`.

**Shared facts, fenced by name — file ownership alone has failed here three
times:**

1. **`w-provaudit` may NOT write `crates/c2-core/src/codegen/**`.** If its
   audit finds a false claim there — and it will, that is where `#3643`
   lived — it **reports** it; `w-mopfold` owns the repair.
2. **The `PROV[X]` marker grammar.** `w-provaudit` owns the script and is
   therefore the one who must not break it: any `MARK_RE` change must be
   proved to leave all **777** existing markers counting identically, and
   said so explicitly.
3. **The census counts.** `w-mopfold`'s comment repairs move them.
   `w-provaudit`'s tests pin **planted-fixture counts or invariants,
   NEVER live per-module tree counts** — the rule that held last wave.
4. **`w-secported` adds no second provenance or census reader**, and
   `w-mopfold` adds no metric key.

### What every lane is told, and what none of them may do

- **Prereg first**, committed before the first measurement, graded honestly;
  a MISS is said in that word. **11 of 11 HIT is a weak result** if the
  measurements predate the prereg — say so, as `w-disclose` did.
- **No lane widens emission.** Required-zero byte delta on the 21 rows for
  anything touching `crates/`, comments included.
- **Re-measure every denominator on your own tree**; the coordinator has NOT
  verified the figures in these briefs — the seven sites, the 27 entries,
  the 777 markers are all quoted from other lanes and are yours to confirm
  or correct. Every brief figure a lane has checked this month has needed
  correcting at least once.
- **A control you have not watched FAIL is decoration.** Plant the defect,
  see red, revert, quote the green.
- **Read the gate's verdict line, never the exit code**; run it detached; do
  not commit while one runs.
- **A lane that produced none of its deliverable says `FAILED`** in those
  words; a priced `declined` is legitimate and, for `w-secported`, is a
  real possible outcome.

### Deliberately not dispatched

`FUNCTION_BYTE_MATCH.md` §0's sixth-gradient box (**third wave outside every
fence — it now needs an owner, not another pass**); wiring the census under
`cargo test` (**two lanes have been comment-only-fenced out of it**;
`w-provaudit` owns the script and may take it if cheap, else it is named);
`codegen/**`'s 205 untagged register numbers; `P_LABEL`'s unreproducible
`25`; the emit-set ceiling predicate (`#3577`); `expr_sweep` pin-by-name
(`#3614`). **Phase 1 stays HELD** — and its gating read has now returned,
which is an owner decision, not a lane's.

**Board:** `#3654` this decision · `#3655`–`#3660` `w-mopfold` ·
`#3661`–`#3666` `w-secported` · `#3667`–`#3672` `w-provaudit`.
**`#3647` remains RESERVED-AND-UNSPENT in the pool and is not reused here.**
Next free `#3673`.

## Decision 18 — OWNER: Phase 1 is CLOSED (overturnable, with the conditions written down), `work/` is a tracked evidence shelf, and the register allocator is next (2026-08-26)

**Four owner answers, given in one turn.** Recorded verbatim where they
decide something.

### 1. Phase 1 — CLOSED, and the owner asked that it be closable BACK

> *"close it but document why so we can overturn that if needed."*

**Phase 1 is closed as `declined-on-measurement`.** It is NOT closed as
"done", "wrong", or "impossible" — it is closed because the read its own
hold was conditioned on came back, and came back negative.

**The record, so an overturn is a decision and not an archaeology
project:**

- **The hold.** Decision 11 (2026-08-24) put Phase 1 on hold *pending the
  `#3509` mapping read*. That is the owner's hold and it was never a
  judgement about Phase 1's value.
- **The read ran.** Lane `w-keymap` executed it. **`#3509` is CLOSED and its
  answer is TU reach 0** — 0 of 845 for each of the ten Phase-1 constructs
  taken separately, 0 for their union, and **0 even when every byteless key
  was granted for free**. The last clause is the load-bearing one: the
  result is not an artifact of a strict key filter.
- **What survives and is not retracted**: the **97.2 % construct-floor**
  result, and every measurement under it. Phase 1's constructs are real
  constructs; what measured zero is their *TU reach on this workload*.

**Overturn conditions — any ONE of these reopens it, and none requires the
owner to re-argue the case:**

1. **The workload changes.** TU reach 0 is a statement about 845 TUs of
   `dc3`. A different corpus, or a materially advanced one, is a different
   denominator and the read must be re-taken before the closure binds.
2. **A downstream construct makes the ten reachable.** Reach was measured
   against today's admitted set. If a later lane widens what the port
   admits such that a Phase-1 construct becomes the *first* blocker on some
   TU, the zero is stale by construction.
3. **The instrument is refuted.** `w-keymap`'s mapping is a measurement and
   this project has refuted its own instruments repeatedly (`#3505`,
   `#3649`). If the key mapping is shown wrong, the reach number falls with
   it.
4. **The goal changes such that reach stops being the criterion.** Phase 1
   was priced against TU conversion. Under decision 15's per-subsystem
   scoreboards, a construct with 0 TU reach can still be worth building
   **for characterization** — and if it is chosen on that basis, this
   closure does not stand in the way. **It never argued the constructs were
   valueless; only that they convert nothing today.**

### 2. `work/` — a tracked evidence shelf. `#3615` is SETTLED.

The owner asked what the files are and whether they can be public. **They
already are public** — `origin` is a public GitHub repository and all 8,114
files have been pushed. So the live question was never "can we publish" but
"should what is already published stay, and under what rule". **Audited by
the coordinator before answering** (tree `0dcfca959`):

| what | count | verdict |
|---|---:|---|
| tracked files under `work/` | **8,114** (21 MB) | the shelf |
| `.cpp` | 3,146 | **ours** — synthetic probe fixtures and generated padding (`int pad14999;`), **not `dc3` source** |
| `.txt` / `.py` / `.log` / `.sh` / `.md` / `.out` | 4,472 | lane evidence: measurements, analysis scripts, prereg records, transcripts |
| captured IL (`_CL_*`, `*.il`) | **0** | clean |
| `*.obj` `*.o` `*.exe` `*.dll` `*.lib` `*.pdb` | **0** | clean |
| files naming `e:/lazer_build_gmc1` / `dc3-decomp` | 366 | **prose about build roots — `CLAUDE.md` says these are INTENTIONAL and must not be scrubbed.** No game source is copied in |
| secret-scanner hits | 5 | **all false positives** (`room_for_token`, `BADTOKEN`, and a sentence asserting *"no secrets"`) |
| **tracked ELF executable** | **1** | **DEFECT — see below** |
| files carrying `/home/free` absolute paths | **488** | **DEFECT — `CLAUDE.md` forbids absolute machine paths in those words** |

**THE RULE, adopted:** `work/` is a **tracked evidence shelf**. Committing
lane evidence there is correct and is not a `.gitignore` violation — it is
what 8,114 files already do and what every citation in `ROADMAP.md` and
`BOARD.md` depends on. **Two carve-outs are absolute and now enforceable:**
**no binaries or build artifacts**, and **no absolute machine paths**.
`.gitignore`'s `/work` line stays (it governs the *default* for untracked
scratch); force-adding evidence under `work/` is sanctioned by this
decision, and `scripts/tracked_artifact_audit.sh`'s scope widens to cover
`work/` so it can enforce the carve-outs it currently cannot see.

**The defect that proves the rule was needed.** `work/w-biquad/c2rs.base`
is a **3,835,864-byte statically-linked ELF executable, tracked at HEAD**.
Decision 12 removed 21 binaries from this very directory on 2026-08-25 and
`work/w-biquad/REMOVED_ARTIFACTS.md` reports that removal as complete —
**`c2rs.base` is not on its list of 21.** A removal documented as total,
with one survivor. **And the reason nothing caught it is on the record
already**: the coordinator's first scan used `grep -P '\x00'`, which
**silently does not fire on NUL** (board `#1236`, known since 2026-08-08);
the `tr -d '\000'` byte-count comparison found it immediately. The guard
that would have caught this is the one that was already known broken.

### 3. Next subsystems — the **register allocator** first, the inliner continued

> *"those are all valuable. register allocator is very valuable. inliner is also."*

All four candidates (register allocator, DAG+scheduler, EH, label numbering)
are ratified as valuable. **Priority: the register allocator**, with the
inliner continued. Both are chosen for **goal (1) — understanding, to help
decomp** — and neither is required to move `match` to be worth funding.
Dispatch follows wave 15; this decision funds no lane by itself.

### 4. The broader goal does **not** resume yet

> *"not yet. we are still mid review and planning here."*

Decision 15's frame stands unchanged: per-subsystem scoreboards are the
working goal, the paused items (the one-arm byte-judged slice, 4a(ii)/I2,
4b/IR3, step 5) stay paused, and no lane resumes parity work.

### The lane this decision dispatches

`w-shelf` — enforce the two carve-outs: remove the surviving ELF (**no
history rewrite**, matching decision 12's precedent, with the removal note
corrected to say what it actually removed), scrub the 488 files, **repair
the NUL guard wherever the broken `grep -P` form is used**, and widen
`tracked_artifact_audit.sh` to cover `work/`. It owns `work/**` hygiene,
`.gitignore`, `scripts/tracked_artifact_audit.sh`, and the wave-15 lanes'
worktrees are **off limits** — three peers are live.

**Board:** `#3673` this decision · `#3674`–`#3679` `w-shelf`.
`#3647` remains reserved-and-unspent in the pool. Next free `#3680`.
