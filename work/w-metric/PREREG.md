# w-metric — pre-registration (written BEFORE any fresh measurement)

Branch `wt-w-metric` off master `7b1cecd`. Question: the headline TU-match
metric is binary and stuck at 8/878 while real work moves; the user suggests
objdiff-style fuzzy matching as a progress measure. Diagnose, survey, recommend
ONE primary progress metric, prototype it, backfill it across today's merges.

Ground rules accepted up front:

* Whatever ships is a PROGRESS metric, never a correctness criterion. It must
  not enter `scripts/gate.sh`, must never present a partial score where the
  byte-exact verdict belongs, and a wrong emit must never score above a refusal.
* This lane changes no codegen; TU match will read 8 before and after.

## Predictions, registered before measuring

**P1 — near-miss distribution (fresh tip scan, per-TU JSONL).** Over the
graded-but-not-matching TUs, the distribution of per-TU *blocked emitted
fraction* (blocked emitted fns / emitted fns) is far from the near-miss story a
fuzzy metric wants: median ≥ 0.60, and fewer than 5 % of failing TUs have
< 10 % of their emitted functions blocked. (Prior: the aggregate emitted
census reads 21.49 % in class; the saved 33cbdbe scan's coarse histogram reads
≤10: 82, ≤100: 403 of 871.)

**P2 — the objdiff transfer dies on the denominator.** An output-similarity
metric needs two objects per TU. Prediction: the port emits an object on
exactly 8 of 871 graded TUs at the tip (match 8 + mismatch 0); on the other
863 there is no port object to compare, so objdiff-style fuzzy match is
undefined on ≈ 99.1 % of the workload and reads exactly 100.0 on the rest —
zero information. This is the registered kill-criterion for the lane's
starting idea *as the primary metric* (it can survive as a forensic aid on
the mismatch class, which is currently empty).

**P3 — the candidate primary registers today's progress where TU match did
not.** Candidate (defined now, measured after): **progress mass**

    P = mean of four fractions over the graded workload
        a = |A| / graded          (emit-set reachable, gate-anchored)
        b = |B| / graded          (every emitted symbol binds)
        c = |C| / graded          (obj sections within writer vocabulary)
        f = emitted-in-class / emitted   (the emitted census ratio)
    with a mismatch guard: a TU graded `mismatch` contributes to NO numerator.

Predicted values (from recorded aggregates; A=28, B=338, graded=871,
f=38458/178975=0.21489 assumed flat across today):

| point | tree | C | predicted P |
|---|---|---|---|
| pre-w-r1c | `3b00093^` | 84 | (28+338+84)/871/4 + f/4 = 0.1829 |
| post-w-r1c | `3b00093` | 114 | 0.1915 |
| post-w-sect | `a4a6ad8` | 169 | 0.2073 |
| tip | `7b1cecd` | 169 | 0.2073 |

i.e. **+2.4 pp across a day in which TU match moved 6 → 8 and then froze** —
the metric registers w-r1c and w-sect, and registers *nothing* for the
measurement-only merges between `a4a6ad8` and `7b1cecd` (w-book4, w-prior,
w-llvm, w-bc): P at 33cbdbe must equal P at 7b1cecd to 4 decimal places.
Uncertainty: A and B at `3b00093^` are assumed 28/338; if either differs the
P0 row moves but the direction of the day does not.

**P4 — what P will NOT register (predicted misses, stated in advance).** The
wrong-emit closures (#232, #259, #263, #276) and the corpus growth
14,484 → 16,164 will move P by zero: warranty and instrument-breadth work is
invisible to any workload-denominated port metric. Supporting metrics must
carry those axes; folding them into P would make one number out of
incommensurable units.

## What would make me recommend AGAINST adopting each candidate

* Against objdiff fuzzy transfer: P2 as stated (undefined on ≥ 95 % of TUs).
* Against progress mass: if the backfill shows it flat (< 0.5 pp) across
  today's behavioural merges, it is no better than TU match and the honest
  deliverable is "no new metric; surface the existing factor block".
* Against ANY candidate: a demonstration that it scores a wrong emit above a
  refusal, or that it can print a healthy number over zero graded TUs.

## Evidence standards

Every scan quoted with its `capture-fail` line (a bad `--cwd` reads
`capture-fail 878 / match 0` and looks ordinary). Backfill points are rebuilt
trees running their own binary; the metric is computed by one script applied
to every point, cross-checked at the tip against the shipped Rust
implementation (must agree exactly).
