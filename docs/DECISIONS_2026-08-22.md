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
workload-tree difference. Step 5's NO-GO clauses (a)–(c)
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
