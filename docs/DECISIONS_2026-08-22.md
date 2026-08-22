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
