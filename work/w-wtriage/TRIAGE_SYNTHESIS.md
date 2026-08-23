# Unlanded-branch triage, round 2 — the rebase adjudication (2026-08-23, lane `w-wtriage`)

`work/w-wtreap/TRIAGE.md` (board `#3464`) adjudicated the 17 unlanded branches
by three mechanical probes and flagged three "worth an owner look". This round
replaced the probes with **an actual rebase per branch plus one Opus assessor
per branch**, anchored in the CURRENT goal rather than the goal that was live
when each branch was abandoned. It corrects two of `#3464`'s claims and settles
the queue.

## Method, and why the rebase is the instrument

Every unlanded branch got its own worktree and a real `git rebase master`.
A branch whose work landed by another path **replays to zero commits** — that
turns "probably superseded" into proof rather than inference. 19 branches
(4 live peer lanes skipped by reflog age); every pre-rebase tip is tagged
`pretriage/<branch>` and recorded in `originals.tsv`, so nothing rests on
reflog survival. Only the 6 clean rebases moved a ref; the 13 that conflicted
were aborted and sit untouched at their original tips.

Then 17 Opus assessors (one per branch, `~1.69 M` tokens, 0 errors) each read
`GOAL_DECISION_2026-08-21.md`, the `ARCH_REVIEW` banner and
`DECISIONS_2026-08-22.md` before judging, followed by one synthesis pass that
re-verified every load-bearing supersession claim by hand.

**Mechanical outcome:** 2 EMPTY-SUPERSEDED (proven — commits replayed to zero),
4 rebased clean, 13 conflicted.
**Adjudicated outcome: 3 land, 14 abandon, 4 owner decisions.**

## The landing queue

| # | branch | action | cost |
|---|---|---|---|
| 1 | `worktree-agent-ac6428ec202a75930` | cherry-pick the **doc commit `e0a153ac` only** | minutes, docs-only |
| 2 | `wt-w-nc` | land code commit `f1b85893`, take master's rung | 1 doc conflict + 2 preconditions |
| 3 | `w-keygen` | **re-dispatch as a lane**, not a merge | session-scale, serialize behind the wave |

**Slot 1 is done in this lane.** `docs/rungs/_2026-08-01-w-adopt-prereg.md`
exists on no other ref and is cited three times by landed docs of record
(`ROADMAP.md:7995`, `2026-08-02-w-adopt.md:8,53`) to substantiate a
"committed before the first measurement" audit claim. Master tracks 82
`*-prereg.md` files; this was the hole. Its sibling code commit `6d1c8599` is
a strict regression against master's `gl_defined_names_with(gl, true)` and is
deliberately **not** taken — which is exactly the doctrine's single surviving
cherry-pick exception, salvaging one commit from a branch whose remainder is
abandoned.

**Slots 2 and 3 are NOT taken here, on timing.** Decision 8 dispatched four
concurrent lanes with two editing `crates/` at once — "which no previous wave
allowed" — accepted only with disjoint fences, serial merges and a full armed
re-gate at each landing. `wt-w-nc` touches `c2-harness/{gap,cli}` and may fence
against `w-pwords`; `w-keygen` touches `c2-core/coff/` (`w-s1c2`'s crate) and
`c2-il/func/` (`w-4f01`'s), from a 911-commit-stale base. A third concurrent
`crates/` lane is not the exception decision 8 granted.

`wt-w-nc`'s two merge preconditions, both cheap and both unmet: **P8 has never
been seen to fire** (the pre-armed positive control — strip `_fltused`, confirm
first divergence classifies to `symtab`), and the base-vs-tip metric-neutrality
scan has never been run. Per #1236 a guard never seen firing is decoration, and
per decision 5's third correction (#3428) a metric-surface move during Phase 0
is expensive to disentangle. **Do not quote any `nc-*` number until P8 fires.**

## What the goal decision actually re-priced

The honest split, negatives included, because the rehabilitation claim is easy
to oversell.

**Value genuinely changed (2).** `wt-w-nc`'s own PREREG says *"the lane's value
is the instrument and the per-class pricing, not a metric move"* and registers
`fnbyte-exact` delta 0 — under the old thesis a confession, under ruling 2 a
direct hit. Ruling 3 disarms both natural objections (per-obj walk cost,
~900 lines for a 2-TU population) because both are throughput arguments, and
those can no longer forbid work either. `w-keygen` was abandoned as the losing
arm of a commission scored on `match 25 → 26`; under ruling 4 its mutation grid
is now the *preferred* shape — an enumerated decision surface with a grading
cell per point, publishing its two GREEN mutants **with their reasons** rather
than dropping them.

**Changed in category, not in outcome (5).** `w-reach`, `wt-w-cflow`, `w-scfl`,
`ad62a3e829`, `wt-w-gate3048-final` are characterization lanes that ruling 2
rehabilitates *as a class* — and they still abandon, because ruling 2 promotes
characterization **output**, and each produced none or one master holds better.
`w-reach`'s instrument was never run (E1–E13 unscored); `ad62a3e829`'s Ghidra
run aborted with `ERROR Abort due to Headless analyzer error`.

**Did not change (7).** Named so the claim is not inflated. `a9b46436` is pure
perf — ruling 3 is symmetric and nets to exactly zero. `wt-w-rank` is the one
ruling 4 points *against*: master has the named enumerable surface
(`prod_tag("tail-recv-not-b9-load")`), the branch has `line!()` integers and
bit-packing, and master's own doc comment rejects line numbers by name.
`ac6428ec` is in the landing queue for **provenance hygiene, not the goal
decision** — a landed audit claim whose evidence file is missing was as broken
in August as it is now.

## The dominant shape, and the lesson for the next triage

Eight of fourteen abandons share one shape: **the branch is the pre-landing
draft and master is the reconciled final.** Resolving those conflicts means
taking master's side on every hunk — the resolution *is* the abandonment. Two
branches (`w-seam`, `wt-w-fltret`, and `wt-w-gate3048-final`) had already
landed under a different ref; `wt-w-fltret`'s content sits on master under
`work/w-fltret2/` while `work/w-fltret/` belongs to a *different* lane.

`#3464`'s capability-grep probe produced **two false negatives**, both corrected
in `work/w-wtreap/TRIAGE.md` in place: a grep keyed on the *branch's* vocabulary
cannot see the same idea landed under another lane's names. `w-reltgt`'s
"require relocation-target agreement" is on master as `w-relo`'s
`RelocDiffers(RelocKind)`, landed four days before the branch's own salvage
commit. Establish supersession by reading the campaign, not by grepping the
branch's own words.

## Residuals worth re-homing (~20–60 lines each), verified absent from master

Cheaper to write fresh than to salvage; none requires landing a branch.

- `1.0000000000000284` = `0x3FF0000000000080` — an FP literal whose payload
  *begins* with the integer escape byte `0x80`, refuting any marker-keyed width
  rule. Belongs in `docs/IL_CAST_CONVERT.md` §3.1.
- `long double` is `88 8a 41`, the same IL triple as `double`. Same table.
- EH + frame-class axes restricted to the **emitted/COMDAT** population,
  including the joint `(eh, frame)` cell — master crosses them over all bodies,
  which is the wrong denominator. ~15–20 lines at `gap/scan.rs:~750` beside the
  existing `emit-cflow-shape|` increments, with an `emit-eh-accounted` control.

## Open for the owner

1. **`wt-w-nc`** — carry an unfired instrument in the scan path during Phase 0,
   or defer until P8 fires? Recommendation: make P8 + the neutrality scan
   preconditions of the merge rather than a decision.
2. **`w-keygen`** — fund the re-dispatch now or park it behind Phase 0? It buys
   refusal 15 (20 → 19) and closes a live wrong emit. It does **not** buy +90
   TU reach (see the correction above).
3. **`w-biquad`** — the one place "abandon = keep the ref" has a real cost:
   129.5 MB of otherwise-gc-able blobs pinned into every clone forever, carrying
   unscrubbed absolute machine paths that master's newest commit at that path
   (`6ad9e852a`) exists to remove. Recommendation: **drop this one ref.** The
   base half is exactly re-derivable (tracked `scan.sh`, tracked base binary +
   sha256, pinned workload); the tip half came from a dirty non-ancestor tree
   and is unreproducible, which is why the rung re-measured against a rebased
   base instead.
4. **Standing policy** — should ground-truth `.obj` files be tracked? `*.obj` is
   gitignored but eleven are already force-added on master. If yes, regenerate
   all nine GRID B objs fresh rather than landing a two-of-nine salvage.
