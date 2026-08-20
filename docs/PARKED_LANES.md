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

| branch | tip | base | ahead | step | outcome | review | findings |
|---|---|---|---|---|---|---|---|
| `wt-w-ir0` | `a0066b692` | `c277d3bb0` | 10 | §5 step 1 | `built` | `land-with-fixes` | 3 major / 12 |

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

---

## Why each is held

### `wt-w-ir0` — the number that justifies the deferral does not cover its data

The lossless framer is built (`c2-il/src/stream/`), additive, no production
caller. Held on:

* **The "8–14 %" cost figure excludes its own low end.** The five paired trials
  are −4.4, −13.5, −5.7, −14.2, −8.8 % (geomean −9.4 %). Two of five are below
  8 %, and −4.4 % is *inside* the "< 5 %" threshold the lane declared REFUTED.
  That string is now in four places, including the proposal's step-1 row, where
  it would govern a scheduling decision.
* **The isolation claim is stated as established cause and is not.**
* **The primary grading criterion could not fail** — a purely additive tree
  with no production caller — and the rung does not say so.

---

## Board numbers held by these branches

`#3332`–`#3336` are minted on the branch above and **do not exist on
`master`**. See `BOARD.md`'s reservation block. Do not re-allocate them.
(`#3327`–`#3331` landed with `wt-w-objplan`; `#3322`–`#3326` landed with
`wt-w-stageoracle`.)
