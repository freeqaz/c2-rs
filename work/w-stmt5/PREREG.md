# `w-stmt5` — PREREG

**Frozen before the first `crates/` byte changed.** Committed as this lane's first
commit on `wt-w-stmt5`, off master **`5a25656a`**.

Brief: pay `IL_STMT_GRAMMAR.md` §14.2 **step 5**'s fail-closed boundary —
`38`/`39` conditional branches, `3A` jump, `29 <tok>` label definitions, with a
token→position map built in a first pass — as a **phase**, not a rung
(`w-read2` **#3132**).

---

## 0. Populations, named, with denominators registered

`match` has three meanings (**#3125**). Every number on this page is labelled
with which population it is counted over, and the lane's grading unit uses
exactly one of them.

| population | denominator | how |
|---|---:|---|
| **the 878-TU workload scan** — THE GRADING POPULATION | `fnbyte-denominator` **162,049** emitted `.text` COMDAT leaders | `c2rs gap --list work/dc3-workload/files.txt --flags-file …/flags.txt --cwd ../dc3-decomp --jobs 16`, one scan, **no obj emitted** |
| the 381×18 fixture gate | 381 fixtures × 18 lanes = 6,858 verdicts | `scripts/gate.sh --jobs 4 --require-graded` |
| `c2rs perf`'s `/Ox` gate | its own | not used to grade this lane |
| the **bodies** population (a driver, NOT the target) | `fn_cflow` sums to **7,151,296** rows over ~1.7M bodies | the same scan's `fn_cflow` histogram |

**The grading unit is `w-readphase` §6, on the 878-TU workload scan only:**

> `fnbyte-refused-parse` **must FALL**, with THREE REQUIRED-ZEROS —
> `fnbyte-exact` **non-decreasing**, `match` (878-TU workload) **non-decreasing**,
> `mismatch == 0`.

**TU conversion is NOT the grading unit and this lane does not chase one.**
`w-readpx` (**#2282**) measured that no reader rung converts a frontier TU;
`w-readphase` measured the whole `.gl` walk at `match` +0 and `fnbyte-exact`
**−65**. A conversion here would be a **signal to check the work**, not success.

## 1. The base, measured on this branch's own tree before any change

`./work/w-stmt5/scan.sh base` at `5a25656a`, 878 TUs, jobs 16:

```
gap-metric match                    25
gap-metric mismatch                  0
gap-metric codegen-gap               0
gap-metric vocab-gap               845
gap-metric port-error                0
gap-metric capture-fail              8
gap-metric frontier                  2
gap-metric fnbyte-exact         35,734
gap-metric fnbyte-denominator  162,049
gap-metric fnbyte-refused-parse 113,612
gap-metric fnbyte-refused-codegen  949
```

Every digit agrees with the brief and with `w-read2` §9's rebased base. The
`emit_blockers` map sums to **113,612 over 615 keys on 845 TUs** — i.e. the
blocked-emitted population **is** `fnbyte-refused-parse`'s population, which is
the identity the ceiling below is computed over.

Workspace test base to beat: **1,619 passed / 0 failed / 42 targets**
(`docs/STATUS.md`'s generated block). **Target: 1,619 + the tests this lane
adds, and the delta is stated as a count, not as "a few".**

## 2. THE CEILING — no discount factor, and it is read off an instrument that already ships

`w-readphase` §6's rule: *"the ceiling IS the estimate, with no discount"*.

Step 5's decode can only move `fnbyte-refused-parse` for a body it can
**model**, not merely walk. The tree's own name for that is
`CfResidue::Modeled` (`control_flow.rs`), and its emitted-column cross is
`gap-metric cflow-emitted-modeled`, published on every scan since **#1343**.

    cflow-emitted-branchy   35,099   blocked emitted fns a block IR must serve
    cflow-emitted-modeled        9   …and whose operand vocabulary is modeled

**PREDICTION P1 (the ceiling, no discount):** the whole control-flow reader
phase — every one of §14.2 step 5's four opcodes, granted at once — can move
`fnbyte-refused-parse` by **at most 9 of 113,612 (0.0079 %)** at today's
expression layer. Not 7,911, not 5,184, not 9,990.

**PREDICTION P2 (the series, not the point).** `w-slots` (**#3147**) is why: a
number read off one cell is right for that cell and wrong as a rule; only
varying the structural count separates the laws. The structural count here is
**the CFG shape**, and the series is the seven `CfShape` values × the two
`CfResidue` values on the emitted population. I predict, before measuring it,
that the emitted series is **concentrated in at most three shapes** and that
`multi-exit`, `if-2`, `if-n` and `loop` each read **0 modeled**.

**PREDICTION P3 (the composition of step 5's own population).** Measured on the
**bodies** population at base (`fn_cflow` cross, no code change needed), the two
statement-layer `0x29` keys are:

| key | bodies | modeled | shapes |
|---|---:|---:|---|
| `body-cflow-label` | 34,852 | **0** | `loop` 34,812 · `switch` 40 |
| `return-scope-close-cflow-label` | 13,182 | **0** | `loop` 13,182 |

I predict the **emitted** column agrees categorically: **`body-cflow-label` and
`return-scope-close-cflow-label` contribute 0 of the 9.** If the emitted cross
disagrees with the bodies cross, **the emitted measurement wins** and the
disagreement is the lane's headline (#3107's rule, which is why this cross is
being built rather than inferred).

**PREDICTION P4 (the boundary is total).** §14.2 step 5's own fail-closed clause
is *"refuse any body where a label is targeted before it is defined"* — a back
edge, i.e. a loop. P3 says step 5's reachable population **is** loops. I predict
the boundary and the population are the **same set**, so the clause refuses
**100 %** of what the step reaches, and the step's marginal worth behind its own
boundary is **0**.

## 3. What this lane will do

1. **PREREG** (this file), committed first.
2. **Instrument.** Publish the emitted-population cflow cross as a **series** —
   one key per `CfShape` × `CfResidue` — replacing the two-key
   `branchy`/`branchy-modeled` collapse that is all the emitted column has
   carried since #1343. Nobody has crossed the grading unit's own population
   with the full axis; `cflow-straight` is excluded from `branchy` by
   `cflow_needs_block_ir`, so the emitted column literally cannot see the shape
   §14.2 step 5's `29` production serves most often.
3. **Pay the boundary.** State §14.2 step 5's fail-closed boundary as a
   **decidable pre-emission predicate** in the shape `CFG_SHAPE.md` §6.3's
   2026-08-13 amendment requires and `codegen/fold.rs`'s `FoldShape::admit`
   demonstrates — a checked rule, not an enumerated list of function names —
   and grade it against the shapes the tree actually decodes.
4. **Report** what step 5 buys behind its own boundary, in the grading unit's
   units, with the series behind every number.

**What this lane will NOT do:** widen `parse_expr`. **#3129**: every sink is
consulted from inside `parse_expr`, and `w-readphase` §7 priced a widening there
at `match` **−7** and `fnbyte-exact` **−5,949** — the mechanism is pre-emption of
a shape recognizer that was already byte-exact. Any acceptance this lane adds is
a **last-resort** arm, reached only where every existing recognizer has already
declined, so it cannot pre-empt one.

## 4. Predicted outcome on the four graded columns

| column | base | predicted tip | why |
|---|---:|---:|---|
| `fnbyte-refused-parse` | 113,612 | **113,612** (Δ **0**) | P1+P4: the ceiling is 9, and the boundary refuses all of it |
| `fnbyte-exact` | 35,734 | **35,734** (Δ **0**, required) | no acceptance widening, no `parse_expr` change |
| `match` (878-TU workload) | 25 | **25** (Δ **0**, required) | as above |
| `mismatch` | 0 | **0** (required, the alarm) | as above |
| `fnbyte-refused-codegen` | 949 | **949** | nothing moves from parse to codegen |
| `cflow-emitted-modeled` | 9 | **9** | the instrument reports it, it does not move it |
| `gap-metric` key count | 372 | **372 + the series** | stated as a count at the tip |

**A tip that reports `fnbyte-refused-parse` FALLING is therefore a PREREG MISS
and the lane says so in that word** — and, per §0, it is first checked for being
a metric artifact before it is reported as a gain. A tip that reports it falling
by more than **9** is an over-accept and the lane must find the bug before it
publishes anything.

**By the grading unit as written, `Δfnbyte-refused-parse < 0` is required to
PASS.** If P1–P4 hold, this lane **cannot** pass that gate, and the honest
outcome word is then **`declined`** with the price above, or **`instrument`** if
the series is what it lands — **never a compound headline**
(`rungs/README.md` § "Outcome, one word").

## 5. Mutation controls, registered in advance

`w-item-d` / `w-layout` / `w-fencea`'s bar: a zero-delta is a measurement only if
the guard producing it can be made to fail. **Every mutant below is registered
with its predicted colour BEFORE it is run**, `w-fencea`-style, including the
greens.

| # | mutant | predicted |
|---|---|---|
| M1 | the emitted series' partition control (every blocked emitted fn lands in exactly one series bucket) broken by dropping the undecoded bucket | **RED** |
| M2 | the series' `+expr-modeled` suffix test replaced by `contains` instead of `ends_with` | **RED** |
| M3 | the step-5 predicate admits a body with a backward label | **RED** |
| M4 | the step-5 predicate's shape clause dropped, so a `switch` is admitted | **RED** |
| M5 | the step-5 predicate's residue clause dropped | **RED** |
| M6 | the step-5 predicate asked about a body it has no decode for | **RED** (refuses, not admits) |
| M7 | reordering two independent clauses of the predicate | **GREEN — registered green in advance.** A predicate whose every permutation is red is order-dependent and that would be the finding |

## 6. Hygiene, registered

* Single writer: gates run in the foreground of one job; no `crates/` edit while
  a gate is in flight (**#3075**, **#3117**, **#3128**).
* Scratch in `work/w-stmt5/` only. **No `git add -f`** — `.gitignore` carries
  both `*.obj` and `/work`, and the `-f` that lands text evidence silences the
  artifact rule in the same stroke (**#3156**, 19 tracked `.obj` files).
  `.jsonl` scan streams are **not** tracked (`w-readphase` committed 1.85 GB by
  accident).
* Board rows left **UNNUMBERED** for the coordinator; next free is **#3165**,
  two peers in flight. A merged-but-unallocated block is invisible to
  `board_audit.sh` (**#3161**–**#3164**).
* `is_statement_layer` has `5C` and not `5D`/`5E` — a shared predicate with a
  currently-zero population (`w-deaccept`, `w-read2` found-and-not-taken #4).
  **This lane does not touch it.** If this lane's work makes that population
  non-zero, that is a **finding to report, not a thing to quietly fix.**
