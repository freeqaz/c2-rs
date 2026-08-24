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
