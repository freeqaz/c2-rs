# PARKED LANES — branches that are built, reviewed, and deliberately not merged

A lane that finishes is not automatically a lane that lands. This file is the
record of branches carrying real work whose **review returned `land-with-fixes`**,
so a future session finds them instead of re-doing them or, worse, re-minting
their board numbers.

**A parked branch is not a failed one.** Each of the three below produced
shippable machinery *and* found a defect in its own grading. The findings are
the reason to hold, and they are also the most valuable thing each lane
produced.

Maintenance rule: a row leaves this file when its branch is merged **or** when
the branch is abandoned with a commit saying why. A row that has been here
across two sessions without either is itself a finding.

---

## Status at 2026-08-20

All three were dispatched as wave 1 of the migration in
`ARCHITECTURE_PROPOSAL_2026-08-20.md` §5. All three were based on `c277d3bb0`
(the proposal commit). Findings are staged in
`work/reviews/{oracle,ir0,objplan}-{review,impl}.json`.

**No lanes are currently parked.** All three wave-1 branches from
`ARCHITECTURE_PROPOSAL_2026-08-20.md` §5 completed their fix rounds and landed
on `master` 2026-08-20 (order: refrev → objplan → stageoracle → ir0). Their
departure notes are below; the maintenance rule (a row leaves when its branch
merges) means this table is now empty by design, not by neglect.

`wt-w-objplan` left this file 2026-08-20: fix round completed all 3 majors
(both manifest components re-shipped as `Unknown` under the registered rule;
the control's blind spots became measured keys; the deciding probe found the
alternative explanation true — `gl_function_attrs` skips records), rebased,
and merged to master. See the merge commit and
`docs/rungs/` (w-objplan) for the fix-round record.

`wt-w-stageoracle` left this file 2026-08-20: fix round completed all 5
majors (the neutrality CLI now asserts armed-and-fired or prints `G1 IS
VACUOUS`; the walk-refusal zero is re-derived from a log that can contain
it; the denominator is 375 armed-and-fired of 410; the `DAT_10c2e2fc`-only
claim corrected in five whitebox places; the category cross-derivation
retracted and re-derived), rebased, and merged to master. Step 0 is GO —
step 5 (COLOR, the DAG scheduler, item F liveness) is unblocked, with the
fix round's own measurement that a ported COLOR cannot be graded against
the tuple spine (board #3323). See the merge commit and `docs/rungs/`
(w-stageoracle) for the fix-round record.

`wt-w-ir0` left this file 2026-08-20: fix round addressed all three majors —
the "8–14 %" figure was the measurement's own null (the switch costs ~2 %, the
isolation claim is REFUTED, and the switch is in no commit); the two guards
nobody had watched were shown to fire; and the rung now states outright that
the primary grading criterion could not fail (a purely additive tree with no
production caller — the framer frames `.ex` and never decodes or admits).
Rebased onto post-wave master, the shared `gap/` seam reconciled with objplan's
grader (disjoint key families, lazy marker index preserved, still no production
caller), re-gated 18/18 with match 26 / mismatch 0 unmoved, and merged. See the
merge commit and `docs/rungs/2026-08-20-ir0.md`.

---

## Board numbers held by these branches

**None held — all landed.** `#3322`–`#3326` landed with `wt-w-stageoracle`,
`#3327`–`#3331` with `wt-w-objplan`, and `#3332`–`#3336` with `wt-w-ir0`. The
wave spent the last reserved range; `BOARD.md`'s RESERVED block is now empty
(kept, not deleted — its reason is a property of `board_audit.sh`, which cannot
see a row on an unmerged branch, so the next wave still needs it).
