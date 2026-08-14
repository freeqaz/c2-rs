# PREREG — lane `w-readphase`

**Frozen before the first measurement.** Committed as this lane's first commit on
`wt-w-readphase`, base master **`6f2c7c41`**. Nothing below was written after a
scan, a probe compile or a `crates/` edit. Scored in the rung doc.

## 0. What this lane is

A **PRICING** lane on the IL reader. Deliverables: (1) a **ladder-based** size for
the top reader blockers, (2) a re-read of `IL_STMT_GRAMMAR.md` §14.2's decode
order, (3) a proposed **offline grading unit** for a reader phase, (4) a
**two-sided price**. It ships no reader widening and no emission widening.
`crates/` byte delta is required **zero**.

## 1. The base I am pricing against (declared, not yet measured)

Taken from the dispatch brief and to be confirmed by this lane's own base scan:

    match 25 · mismatch 0 · codegen-gap 0 · vocab-gap 845 · capture-fail 8 · frontier 2
    ~372 gap-metric keys · 878 verdict lines

If my base scan disagrees with any of these, **that disagreement is the first
finding** and is reported before anything else.

## 2. Prior art I am registering as already-known, so I cannot claim it

* `w-vec` board **#2505**: 811 of 851 refused TUs stop at `gl-stop-26-introduced`
  and *"repairing it converts ZERO TUs"*; `body-out-of-class` fires on **851 of
  851**; there is **not one TU** whose only refusal is `gl-stop-26-introduced`.
* `w-fencecount` board **#3062**: `sole` 0 and `exact` 0 across all 23 causes;
  `gl-stop-26-introduced` `first-of-multi` = **819**; 845 held TUs carry 1,716
  cause firings.
* `w-readpx`: seven one-function transcriptions = **+7** fnbyte-exact / **+7** TU;
  one 444-wide admission = **+0 / +0**.
* `IL_DECODE_REACH.md` §3: decoding `0x67` **and nothing else** moves decode reach
  by **ZERO** — the 45,631-body `cf-expr-0x67` row becomes a 45,631-body
  `cf-expr-0x9A` row **two tokens later**. This is the ladder shape I expect.
* `CEILING.md` §3.1: the emitted-code widening order is **648 keys summing
  130,575**, head `expr-op-0x27` at **22,373**; board #150 closed `expr-op-0x27`
  at **6 emitted functions converted**, `w-op27` re-measured **8** (board #1337).
* `IL_STMT_GRAMMAR.md` §14.1: statement layer alone **+7 bodies**; with a full
  expression layer **+812**; the operand TYPE gate is worth **3.2×** and the rest
  of the plain operator table **exactly zero**.
* Ranking ratios on this project (row → realized): **67× · 67.8× · 13.4×**
  (first-blocker rows), **2.62×** (a `-whole` first-blocker key), **1.45×** (a
  counterfactual successor), **1.0002×** (a counterfactual *of the production
  being widened*). Rule: *when the instrument is a counterfactual of the
  production you are widening, the ceiling IS the estimate.*

## 3. Bias direction, stated in writing

Every size figure below is a **CEILING with no discount factor applied**. The
recorded error direction on this project is **optimism, ~5:1** (`CEILING.md` §5),
and **five of six times a discount was applied here the discount was the error**
(ROADMAP: "multiplying a ceiling by a previous rung's realized fraction without
asking what produced it"). So: no discount, count **independent** refusals, and
before counting two refusals as two, ask **"what varies between these two?"** — a
ceiling on this board once counted one variable read at eight thresholds as eight
independent refusals.

## 4. The registered predictions

Ladder depth is defined as **the number of successive clause lifts for which the
class still reports exactly one nameable clause as its next blocker.** Depth stops
when the successor is an open key space rather than a clause.

| id | prediction | p |
|---|---|---:|
| **L1** | Lifting `gl-stop-26-introduced` in a scratch tree moves **≥ 750 of the 819** first-blocker TUs to `body-out-of-class` as their new first blocker | 0.88 |
| **L1b** | The L1 lift converts **0** TUs: `match` stays 25 | 0.95 |
| **L1c** | The L1 lift moves `fnbyte-exact` by **0** | 0.85 |
| **L1d** | The L1 lift moves `fnbyte-refused-parse` by **0** functions — the `.gl` walk and the body decode are independent | 0.85 |
| **L2** | The **ladder depth of the head class is exactly 2**: L1 → `body-out-of-class`, and `body-out-of-class` is not a clause but an **open key space** of ≥ 500 distinct per-function keys | 0.80 |
| **L2b** | Depth ≥ 3, i.e. a *third* nameable single clause exists between the head class and the open key space | 0.15 |
| **L3** | The emitted-code widening order restricted to the 819-TU head class has **≥ 600 keys summing ≥ 120,000** blocked emitted functions | 0.85 |
| **L4** | The head class's true yield **in TU conversions is 0** | 0.93 |
| **L5** | The head class's true yield **in `fnbyte-exact` functions is 0** (ceiling, no discount) | 0.80 |
| **L6** | **The reader's refusal structure is not a ladder at all**: for the majority of blocked emitted functions the "next blocker" is **not computable without implementing the production**, because the blocking token has unknown width and a walk cannot skip it (`IL_DECODE_REACH` §2.1's desynchronization). Registered as the load-bearing methodological claim | 0.70 |
| **S1** | Of `IL_STMT_GRAMMAR.md` §14.2's six steps, **≥ 4 (steps 1–4) are already done** in `crates/c2-il` today | 0.60 |
| **S2** | §14.2's stated **order is stale in at least one place** — a step's named gain, prerequisite, or fail-closed boundary is contradicted by a measurement landed since it was written | 0.70 |
| **G1** | **FBM cannot grade a decode-only reader phase**: a widening that decodes strictly more and emits strictly nothing new moves `fnbyte-exact` by **0** and moves **no** published `gap-metric` key except the census / blocking-feature histograms. This is the reason the grading unit is "missing" | 0.75 |
| **G2** | There **is** an existing published key that would move under a decode-only phase and is *not* the census: `fnbyte-refused-parse` (and its complement `fnbyte-refused-codegen`) | 0.65 |
| **P1** | The **two-sided price** will come out with the *narrow* side non-zero — i.e. keeping the reader narrow has a stateable cost in the goal's own unit, not merely an opportunity cost | 0.55 |

## 5. Absence is not success

Every check below is positive on content. A grep that finds nothing is **not** a
result; the run must have graded N > 0 and N is quoted.

* The base scan must report `878` verdict lines and a non-zero `fnbyte-denominator`.
* Every ladder step must report the population it graded, and a step that graded 0
  is a FAILED step, not a confirmed null.
* The identity diff at the end must compare a **non-empty** key set (~372 keys)
  and 878 verdict lines by name, and say so.

## 6. What would make this lane FAILED

* No ladder is built (only a histogram is re-quoted).
* The ladder is built but its depth is not measured by an actual lift.
* A number is quoted whose command is not recorded.
* `crates/` byte delta is non-zero at the tip.
