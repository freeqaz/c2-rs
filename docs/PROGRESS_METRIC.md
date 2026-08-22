# PROGRESS_METRIC — the progress mass, and why it is not a fuzzy match

**Status: adopted 2026-08-04 (lane `w-metric`). One number, printed by every
`c2rs gap` scan, machine-readable as `gap-metric progress-mass`.**

This document answers one question the project could not answer on 2026-08-04:
*a day moved factor C by 55 TUs, closed three live wrong-emit families and grew
the graded corpus by 1,680 cases, and the headline metric read 8/878 before and
after — which lane moved more?* It defines the **progress mass**, the survey
that chose it, the candidates it beat, the backfill that validates it, and —
first, because it is load-bearing — the wall between it and correctness.

---

## 0. The separation rule (read this even if you read nothing else)

> **The progress mass is a PROGRESS metric and never a correctness criterion.
> The real `c2` under wibo plus the byte-exact obj compare is the SOLE judge of
> the port (`CLAUDE.md`). A scan whose progress mass reads 0.97 and whose
> `mismatch` count reads 1 is a FAILING scan.**

The separation is structural, not editorial:

* **It never appears in `scripts/gate.sh`** and must never be added there. The
  gate compares byte-exact verdict counts; the progress mass is not a verdict.
* **It prints in its own block**, headed by the disclaimer, after the
  factorization and apart from the class table that carries `match`/`mismatch`.
* **Its keys are namespaced** (`progress-*`), so no collector can confuse them
  with the verdict counters.
* **A wrong emit scores below a refusal, always** (§5.2) — so the metric cannot
  pay a lane to emit *something*, which is the second-order failure a
  partial-credit score invites.
* **It is unrepresentable over an empty scan** (§5.3) — `NO-RESULT`, no key,
  never 100 % over zero comparisons.

> ### ✔ 2026-08-22 — **THE OWNER ASKED WHETHER THE JUDGE CAN CARRY A SLIDING SCORE. IT CANNOT, AND §5.2 IS THE LOAD-BEARING REASON.**
> *`docs/WHITEBOX_LEVERAGE_2026-08-21.md` §5(a), on the owner's question, the
> same day as the goal re-ranking (`GOAL_DECISION_2026-08-21.md` § "AMENDED").
> Propagated by lane `w-readdocs`; **nothing on this page is edited, re-scored
> or withdrawn**.*
>
> **The gate stays binary.** A 90 %-matching obj *shipped* is a wrong emit, and
> **§5.2's rule — a wrong emit scores strictly below the refusal it replaced —
> is what the answer rests on.** The scale of what it protects against was
> measured after this page was written: **2,490 wrong functions** (board
> **#3363**, which turns on this page's rule and calls the alternative
> *anti-safe*). Nothing in the goal decision or its re-ranking relaxes it, and
> the re-ranking's new consumers do not either — a permuter and a training
> pipeline consume **instruments**, and an instrument that could license an
> emit would stop being one.
>
> **Gradients are welcome and go beside the judge, never inside it.** There are
> three: this page's progress mass, [`FUNCTION_BYTE_MATCH.md`](FUNCTION_BYTE_MATCH.md)
> (whose §0 is the standing **template** for any gradient added after it), and
> [`DIFF_STRUCTURE.md`](DIFF_STRUCTURE.md) / `gap/fndiff.rs`, which classifies
> the disagreement *inside* each differing body and is printed on every scan.
> All three carry §0's five properties. **A fourth would have to carry them
> too**, which is the check a 2026-08-21 planning doc skipped when it proposed
> re-building the third one (board **#3369**).

## 1. Diagnosis — why TU match is stuck at 8, measured

A TU matches only if **every byte of the whole obj** matches: the verdict is a
conjunction over the emit set, the binding, every section, every COMDAT body
and the section/symbol tables. The factorization (`c2rs gap`, ROADMAP
§10.19/§10.21) states the necessary conditions: `A ∧ B ∧ C ∧ (D ∨ E)`, and at
the tree this document was written on (`7b1cecd`, scan real: `capture-fail 7`)
they read A 28, B 338, C 169, `A∧B∧C` 27, FRONTIER 19 over 871 graded TUs
(re-verified unchanged after the rebase onto `d8aa8e3`, which added w-reloc,
w-fork and the wibo resync). So
the binary metric is capped at **28** before Phase 7 exists, 8 are taken, and
each of the 19 frontier TUs is priced at ≥ 6 independent codegen facts (board
#269). TU match is stuck because *it is supposed to be stuck*: it moves only
when a TU's **last** defect closes, and no TU's last defect was close today
except the two `??__E` conversions that did move it 6 → 8.

**The near-miss distribution, measured** (this lane, tip scan
`work/w-metric/gap-tip.jsonl`, 871 graded, 863 failing):

| | value |
|---|---|
| failing TUs with an emitted-function denominator | 862 (1 emits no functions) |
| blocked-emitted-fraction deciles (p10 … p90) | 0.71 · 0.75 · 0.77 · 0.78 · **0.80** · 0.81 · 0.82 · 0.85 · 0.92 |
| failing TUs with < 10 % of emitted fns blocked | **1** (0.1 %) |
| failing TUs with 100 % of emitted fns blocked | 42 (4.9 %) |

**The median failing TU has 80 % of its emitted functions out of class, and 90 %
of failing TUs are above 71 %.** There is no near-miss bulge for a fuzzy metric
to reveal. Any per-function or per-byte similarity score over this workload is
a rediscovery of the emitted census (21.49 %), TU by TU.

## 2. What objdiff's fuzzy match actually computes

Read from the sibling checkout (`../objdiff`, MIT/Apache-2.0), which is the
fork `../rb3-xenon` drives:

* **Per symbol pair, instruction-level.** `diff_code` scans both functions into
  instruction lists, aligns them with a Patience diff
  (`objdiff-core/src/diff/code.rs:673`), and scores each aligned row:
  insert/delete 100, replace 60, register-diff 5, immediate-diff 1
  (`code.rs:53-56`). `match_percent = (1 − score/max) × 100` with
  `max = rows × 100` (`code.rs:276-291`).
* **Normalized on purpose.** Relocations compare by *target symbol name* and
  offset, never by address (`code.rs:122-129`); a second
  `match_percent_normalized` additionally forgives register/offset swaps. The
  local fork adds `masked_equal` disclosure counters and an FP-anchor
  compensation that scores semantically-equal frame-anchor rows as equal —
  i.e. the whole instrument is built to *ignore chosen classes of real byte
  differences*.
* **Aggregation is size-weighted** over per-symbol percents
  (`bindings/report.rs:248-296`), and — the detail that matters most here —
  **`calc_fuzzy_match_percent` returns 100.0 when `total_code == 0`**
  (`report.rs:249-250`): zero comparisons read as perfection.

### Why it does not transfer as this project's progress metric

1. **It needs two objects per unit, and this port produces one on 8 of 871
   graded TUs** (match 8 + mismatch 0, tip scan). objdiff grades a decomp
   whose build always emits *something* for every function; `PortC2` refuses
   whole TUs — honestly, by design — on 99.1 % of the workload. A fuzzy match
   over c2-rs is **undefined on 863 TUs and reads exactly 100.0 on the other
   8**. It carries zero bits of information today, and § 1 shows the near-miss
   structure would keep it census-shaped even after Phase 7.
2. **Where it is defined, it rewards the wrong direction.** The #232 repair
   turned a wrong emit back into a refusal; #259/#263/#276 changed wrong bytes
   into right ones or refusals. On a partial-credit output-similarity score,
   restoring a refusal *removes an object scoring ~90 %* — the metric goes
   **down** on the fix and was **up** on the defect. That is the exact
   inversion `CLAUDE.md`'s rule exists to forbid, and no weighting fixes it,
   because the defect is in what the metric is *of*.
3. **Its tolerance model is this project's negation.** objdiff normalizes away
   register swaps, reloc addends and anchor slips because a decomp's judge is
   eventual semantic equivalence. Here the judge is byte equality; a metric
   whose 100 % is not the judge's 100 % would mint a second, softer definition
   of "done" and put it in every report.

What *does* transfer: the discipline of publishing the score only next to its
denominator, and the negative lesson of `total_code == 0` → 100 %. The progress
mass inverts that: empty scan → `NO-RESULT`, no number at all.

objdiff-style byte similarity remains legitimate as a **forensic aid on the
`mismatch` class** — "how far off was this wrong emit" — which is currently
empty; the scan already records first-divergence offsets there. It must never
be aggregated into a headline.

## 3. The candidate survey

Each candidate faces the two adversarial questions: **(a)** can a lane game it
upward without real progress, **(b)** can it rise while the port gets worse?

| candidate | verdict | (a) gameable | (b) up-while-worse | notes |
|---|---|---|---|---|
| per-COMDAT match rate (port obj vs ref obj, per function) | **rejected** | — | — | needs a port obj; undefined on 863/871 TUs (§ 2.1) |
| byte/instruction similarity within sections | **rejected** | emit garbage that half-matches | **yes — scores the #232 defect above its fix** | § 2.2; the disqualifying pair |
| fraction of workload **bytes** reproducible | **rejected** | accept the biggest functions first, wrongly | yes (acceptance is a parser claim, byte-weighted amplifies it) | also unmeasurable without per-function sizes the scan does not record; weights the metric toward a few huge TUs |
| refusal depth (how far into a TU before refusing) | **rejected** | reorder/delay refusal without new capability | yes — depth can grow with zero new correct output | also ill-defined: decode is not sequential per TU |
| relocation-vocabulary coverage | **rejected** | — | — | already complete: w-reloc measured all 1,819,168 workload records as exactly the five base types the writer emits, so the axis reads 100 % forever — a metric pinned at its ceiling measures nothing |
| blocked-reason histogram as burndown | **kept, supporting** | shrink a row by renaming its key | mostly no (it is not a scalar) | already printed; it is the widening *order*, not a level |
| emitted census alone (21.49 %) | **kept, inside P** | in-class inflation on never-graded bodies (trap 2) | yes, within its documented trap | the project's accepted driver; moves on codegen days, flat today |
| factor counts alone (C 169, B∧C 151…) | **kept, inside P** | C: teach the writer a name with a wrong emitter | weakly (C is bounded: 3 names to closure, then inert) | move on section/binding days, flat on codegen days |
| **weighted composite of the above = progress mass** | **adopted** | inherits only its terms' documented modes | inherits trap 2 through `f`; guarded against the emit direction | § 4 |

The decisive observation: the workload has **two kinds of progress days** —
codegen-widening days (census/`f` moves, factors flat) and
reachability days (factors move, census flat: today) — and every single-term
candidate is blind to one kind. The composite is not cleverness; it is the
smallest set of terms that covers both, with equal weights because all four are
*necessary* and none has a principled price (any unequal weighting is a claim
about relative cost that board #269's pricing says we cannot make).

## 4. The metric: PROGRESS MASS

Over one `c2rs gap` scan of the workload:

```
P = ( a + b + c + f ) / 4          with
a = |A| / graded                    emit-set reachable (gate-anchored 4F 1F)
b = |B| / graded                    every emitted symbol binds
c = |C| / graded                    obj section set within the writer's vocabulary
f = emitted-in-class / emitted      the emitted-function census ratio
```

**Denominator.** `graded` = every captured TU (871 of 878 here; `capture-fail`
TUs were never measured and are excluded exactly as the factorization excludes
them). `f`'s denominator is **emitted functions** (178,975) — the population
the goal is written in — not IL bodies (2.46 M, 92.8 % of which c2 never
emits) and not bytes (unmeasured, and § 3 rejects the weighting). These are
the same denominators the factor block and the emitted census already publish,
which is what makes the metric backfillable from any recorded scan.

`P` is denominated in *discharged necessary conditions*, averaged. It is not a
count of TUs, and `P = 0.21` does not mean "21 % done": the four terms are
necessary, not sufficient, and the frontier pricing (board #269) says the last
term's remaining distance is the expensive part.

### 4.1 What P rewards — and the second-order design

`P`'s terms are **preconditions proven against the reference obj** (A, B, C are
facts about the emit set, the binding and the section vocabulary, measurable
whether or not the port emits) and **honest acceptance** (`f`, the census's
in-class verdict on emitted functions). **No term is a function of the port's
output bytes.** A lane that makes the port emit garbage moves nothing; a lane
that widens acceptance moves `f` by exactly what the census grants; a lane
that teaches the writer a real section moves `c`. Refusal is the zero point,
never penalized relative to wrongness — which answers the inversion risk: a
metric that punished refusal more than wrongness would turn the port's honest
`NotImplemented` boundary from a feature into a cost, and lanes would pay it.

### 4.2 The two structural guards (both unit-tested)

* **Mismatch zeroing.** A TU graded `mismatch` contributes 0 to every
  numerator and stays in every denominator
  (`a_wrong_emit_scores_strictly_below_the_refusal_it_replaced`). Turning a
  refusal into a wrong emit strictly decreases P; the count of zeroed TUs is
  printed, so a reduced P cannot pass as clean.
* **Empty-scan refusal.** `progress_mass()` is `None` when nothing graded or
  nothing emitted; the print says `NO-RESULT` and the `gap-metric` key is
  absent (`progress_mass_is_unrepresentable_over_an_empty_scan`). A bad
  `--cwd` run (capture-fail 878) therefore produces **no** progress number.

### 4.3 What P does NOT measure — its traps, in the STATUS tradition

1. **P is not correctness, and `f` inherits STATUS trap 2 whole.** `f`'s
   numerator is a parse-time claim never graded by the differential for a TU
   that does not match. A widening wrong in a way no standing instrument sees
   (board #232's shape) raises P exactly as it raises the census. The gate,
   the sweep and the mismatch alarm are the warranty; P is a speedometer with
   no brakes.
2. **Factor C can in principle be inflated** by adding a section name to
   `PORT_WRITER_SECTIONS` with a wrong emitter behind it — the known-answer
   control catches that only where matches exist. Mitigations: C is bounded
   (13 workload names, 3 unlearned; after closure the term is inert), and any
   wrong emitter that ever runs lands in `mismatch` and zeroes its TU.
3. **P is workload-denominated, so warranty and instrument work move it by
   zero** — the four wrong-emit closures and the corpus growth of 2026-08-04
   are invisible to it, *predicted in advance* (PREREG P4) and confirmed. The
   supporting metrics below carry those axes; folding them in would average
   incommensurable units.
4. **Equal weights are a modelling choice.** They encode "all four are
   necessary and unpriceable", not "all four are equally hard". Do not read
   ΔP across different terms as comparable effort; read the four terms, which
   are always printed beside it.
5. **P will sit near ~0.2 for a long time and then be dominated by `f`.**
   `a`+`b`+`c` saturate at (28+338+169+…)/871 each ≤ 1; the long middle of
   this project is `f` climbing. When `b` and `c` close, P compresses to
   ≈ (2 + f)/4 + a/4 — still monotone in the work, but read the terms.

## 5. Measured values and the backfill

All scans real, `capture-fail 7 / graded 871` on every row; each historical
point is that tree's own binary rerun on the shared capture cache
(`work/w-metric/gap-*.txt` on branch `wt-w-metric`); the offline computation
(`work/w-metric/analyze.py`) agrees with the shipped implementation at the tip
to all printed digits.

| point (2026-08-04) | tree | match | A | B | C | f numerator | **P** | Δ |
|---|---|---|---|---|---|---|---|---|
| pre-w-r1c | `68bdbf8` | 6 | 28 | 338 | 84 | 38,455/178,975 | **0.18288** | — |
| post-w-r1c (match 6→8, writer +3 names) | `3b00093` | 8 | 28 | 338 | 114 | 38,455/178,975 | **0.19149** | +0.86 pp |
| post-w-sect (`.data`/`.bss` writer) | `a4a6ad8` | 8 | 28 | 338 | 169 | 38,458/178,975 | **0.20728** | +1.58 pp |
| tip (after w-label, w-shapes, w-book4, w-prior, w-llvm, w-bc) | `7b1cecd` | 8 | 28 | 338 | 169 | 38,458/178,975 | **0.20728** | +0.00 |

**The backfill answers the commissioning question.** The binary metric read
8 all afternoon; P moved **+2.44 pp across the day**, and the largest single
step (+1.58 pp) was w-sect — a merge whose TU-match delta was exactly zero.
Ranking the day's lanes by ΔP: w-sect > w-r1c > everything else at 0.00 — a
ranking the binary metric could not produce. The zero rows are equally
load-bearing. The seven merges between `a4a6ad8` and the tip moved P by
0.00000 (`33cbdbe`'s saved scan reads the identical A 28 / B 338 / C 169 /
38,458 inputs, `work/gap-33cbdbe.txt`): the measurement-only merges (w-book4,
w-prior, w-llvm, w-bc) show the metric does not drift under observation, and
w-label — real codegen work whose effect lands on the *fixture* gate
(106 → 118 Match), not the workload — moved P by zero, confirming trap 3
rather than surprising it.

**The oracle resync is an accidental robustness control.** Lane w-wibo
replaced wibo `1.0.1-23-g4a9dd6f` with `1.2.0-27-geab90f0` at 22:16:37, which
re-keys the capture cache; the backfill scans and the final tip scan therefore
re-captured the workload **cold, from scratch, under the new oracle** (the
first tip scan had run warm under the old one — both provenance headers are in
`work/w-metric/`). Every P input — the factor counts, the emitted census, the
match set, `capture-fail 7` — is digit-identical across the swap, consistent
with w-wibo's byte-identity verification. P is a function of oracle *output*,
so an oracle rebuild that preserves output moves it by exactly zero, and now
that has been observed rather than assumed.

Predictions registered before measurement (`work/w-metric/PREREG.md`,
committed `ae692fa`, before any scan ran): P1 near-miss median ≥ 0.60 —
realized 0.80; P2 output-similarity undefined on ≥ 95 % — realized 99.1 %,
killing the objdiff transfer as registered; P3 predicted P values 0.1829 /
0.1915 / 0.2073 / 0.2073 — realized 0.18288 / 0.19149 / 0.20728 / 0.20728;
P4 predicted zero movement from warranty/corpus merges — realized 0.00000.

## 6. Supporting metrics (two, no more)

1. **The emitted-only blocked histogram** (already printed by every scan) — the
   burndown *list*. P says how much mass moved; the histogram says what to move
   next. It is kept a list, not a scalar, because collapsing it invites the
   renaming game (§ 3).
2. **Graded-verdict breadth of the standing instruments** — the gate's own
   verdict count (`2,940` at `33cbdbe`) plus the sweep's `reached/graded` and
   the mode cross's `selected/graded`. This is the warranty axis P deliberately
   does not carry (trap 3): the 2026-08-04 wrong-emit closures and corpus
   growth live here. Quote it from `gate.sh`'s own summary, never from memory.

## 7. Operational notes

* Printed by `c2rs gap` after the factorization block, before `GAP-METRICS`;
  implementation `GapReport::progress_mass` (`crates/c2-harness/src/gap.rs`),
  pure over `results`, unit-tested without a toolchain.
* Machine-readable: `gap-metric progress-mass 0.20728` plus
  `progress-emitted-in-class` / `progress-emitted-total` /
  `progress-mismatch-zeroed`. Keys are an interface; absence means NO-RESULT.
* To recompute anywhere: run the workload scan exactly as `STATUS.md`
  documents and quote the `capture-fail` line with the number. A scan that
  captured nothing prints `NO-RESULT` here by design.
* `scripts/status.sh` does not yet collect `gap-metric` keys (the collector
  change is specified in `rungs/2026-08-04-w-bc.md` §5.1 and not made). When
  it lands, `progress-mass` rides along for free.
