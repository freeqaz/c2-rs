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
`ARCHITECTURE_PROPOSAL_2026-08-20.md` §5. All three are based on `c277d3bb0`
(the proposal commit) and need a rebase before landing. Findings are staged in
`work/reviews/{oracle,ir0,objplan}-{review,impl}.json`.

| branch | tip | base | ahead | step | outcome | review | findings |
|---|---|---|---|---|---|---|---|
| `wt-w-stageoracle` | `8b338b39e` | `c277d3bb0` | 15 | §5 step 0 | `instrument` | `land-with-fixes` | 5 major / 14 |
| `wt-w-ir0` | `a0066b692` | `c277d3bb0` | 10 | §5 step 1 | `built` | `land-with-fixes` | 3 major / 12 |
| `wt-w-objplan` | `2350c118c` | `c277d3bb0` | 14 | §5 step 3 | `instrument` | `land-with-fixes` | 3 major / 12 |

**`wt-w-stageoracle` is on the critical path.** Proposal §5 puts step 5 —
porting the middle: COLOR, the DAG scheduler, item F liveness — *strictly*
behind step 0. Steps 1 and 2 are not, and can proceed while it is parked.

---

## Why each is held

### `wt-w-stageoracle` — the go/no-go number is not yet trustworthy

The lane built the call-site detour taps at c2's seven per-function phase
boundaries and reported **GO**. The review found the number backing that GO is
not guarded:

* **`c2rs stage neutrality` never asserts the tap armed.** Its required-zero
  can print `G1 HOLDS over N graded fixtures` over a population where not one
  byte of `c2.dll` was patched — armed-vs-disarmed objs are trivially identical
  when nothing armed. This is **the repo's signature defect inside the lane's
  own required-zero**. `cmd_counts` and `cmd_determinism`, in the same file,
  both *do* check; the integration test checks; the CLI whose output the rung
  quotes does not.
* **A zero was read from a log that structurally cannot contain the line.**
  "Zero walk refusals over the whole campaign" — no `walk-overrun`,
  `walk-span`, `walk-implausible-*`, `arena-full` — over 410 objs.
* **G1's denominator counts 35 of 410 objs on which the tap fired zero times.**
  The byte-identity is free on those. Board `#3322`'s headline says "OVER 410
  GRADED OBJS WITH THE PAYLOAD ARMED".
* **A disassembly-derived justification is refuted by the disassembly.** "The
  four scheduler runs are gated *only* by `DAT_10c2e2fc`" is false — three of
  the four sites carry a second per-function gate and `sched0` carries three
  more. The `/Od`→0 direction still holds.
* **A cross-derivation is a category error.** The observed tuple categories are
  every category in a region *body*; the finder branches on region
  *terminators*.

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

### `wt-w-objplan` — the shipped component is self-declared RED on every run

The `ObjPlan` extractor and the structural-manifest instrument are built. Held
on:

* **The registered ship/no-ship rule was applied asymmetrically.** Prereg §3:
  a component whose control is red ships as `Unknown`, never `Differs`.
  `emitset-order` (12 of 26 control TUs differ) was withdrawn under that rule;
  `emitset-members` (2 of 26 differ) shipped as the headline and prints its own
  red on every scan. The post-hoc justification applies verbatim to the
  withdrawn one. The effective rule was **"12 is too many, 2 is fine"**, which
  was not registered.
* **The named control's headline is vacuous** in exactly the way the lane
  diagnosed and fixed elsewhere in the same rung.
* **The deciding probe rules out one explanation and never tests the other.**

---

## Board numbers held by these branches

`#3322`–`#3336` are minted on the branches above and **do not exist on
`master`**. See `BOARD.md`'s reservation block. Do not re-allocate them.
