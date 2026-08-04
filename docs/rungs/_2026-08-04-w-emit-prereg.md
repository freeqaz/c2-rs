# w-emit — pre-registration: can §2's emit predicate survive real TUs at all?

Written and committed **before any measurement of this lane's headline
quantity**. Worktree `wt-w-emit`, base master **`c7f7529`** (master moved
`14bd6d7 → 88e5ff6 → c7f7529` while this lane was being set up; lane w-prov is
landing merges).

    Lane:      w-emit, 2026-08-04
    Target:    PHASE7_PLAN.md §2 / board #161 — the fitted emit predicate.
    Question:  does the least-fixpoint ODR-use reachability model transfer from
               172 synthetic cells to the real workload, and specifically can
               it ever meet the fail-closed R3 precision requirement?

## 0. Provenance, frozen now

| | |
|---|---|
| c2-rs base | master **`c7f7529`** |
| **dc3-decomp HEAD BEFORE any run** | **`940d07dcb0960964ad61aa5f025658f993eb46b2`**, `git status --porcelain` empty (`workload_dirty = False`) |
| dc3-decomp HEAD AFTER the run | recorded in the findings; **if it moved, that is a finding, not a footnote** |
| flags | the unmodified workload line, `work/dc3-workload/flags.txt` |
| TU list | `work/dc3-workload/files.txt`, 878 entries |
| scratch | `work/w-emit/` (gitignored); instruments reused from `work/emitpred/` |
| toolchain | X360 `16.00.11886.00` under wibo, as every other lane today |

**Revs other lanes measured this predicate at, none of which are mine:**
`13b583df` (the 172-cell fit and the 871-TU census), `51fb5b73` (w-emitpred's
frozen draw), `9ad5c4c8` (`MAGNITUDE.md`). **No number from those is quoted as
if it were measured here.**

## 1. What is already known, so this lane does not re-run it

`docs/PHASE7_VALIDATION.md` (lane w-emitpred) already returned **DECLINE** on
#161: **V6 = 5** structural axes broken against a registered interval of
**[0, 3]**, R1 internally inconsistent on its face, and a four-item repair list
(§8a). So *"does §2 transfer as written"* is **already answered: no.**

**Two things it did NOT establish, and they are what decide R3:**

1. **Part 1 was never run.** V1 (F1), V2 (vs incumbents), V3 (precision) are
   **unmeasured**. The `pipeline` agent died to an OOM before freezing a
   prediction, so §2's *quantitative* fit on real TUs is unknown.
2. **`MAGNITUDE.md` measured exactly one false-positive class** — the
   virtual-slot class, 649 instances, precision ceiling **0.99629** — and said
   in its own §6.1: *"This class does not break V3. It is not the thing that
   will."* **Nobody has measured what will.**

**This lane measures the thing that will, and it does it without a root model.**

## 2. The measurement — root-model-free, and why it is sound

A root model does not exist and this lane will not invent one (inventing it is
fitting, and `MAGNITUDE.md` §2 already declined it for the same reason). The
bound below needs none.

**Definitions, per non-quarantined TU `t`:**

| | |
|---|---|
| `E(t)` | truth — COMDAT leaders of every section with `IMAGE_SCN_CNT_CODE`, **never** a `.text` name prefix (`magnitude/truth_all.py`) |
| `U(t)` | the candidate universe — names with a `.gl`-named `.ex` body in `t` (`model.named_bodies`) |
| **`26`-edge** | `A → F` where a `.gl`-resolvable operand token for `F` occurs in `A`'s body at a position `p` with `exb[p-1] == 0x26` — the **direct call / reference** form (`MAGNITUDE.md` §3a) |
| `67`-edge | `exb[p-2] == 0x67` — virtual dispatch. **Excluded from the headline by construction.** |
| attribution | **strict** (`.gl`-named segments only) plus the local-static owner channel; **never** the folding rule, which `attrib.py` grades correct on 1 of 14 842 |

**The contradiction set:**

    X = { (t, F) : F ∈ U(t),  F ∉ E(t),
                   ∃ A ∈ E(t) with a 26-edge A → F }

**The dilemma, which needs no roots and admits no third branch.** Take any
`(t, F) ∈ X`. Under §2 the emit set *is* the kept set, so:

* if `A ∈ P(t)` — §2 predicts its own referrer — then §2's Propagation clause
  ("a call anywhere in the pre-optimization body") **adds `F`**, and `F ∉ E`,
  so **`F` is a false positive**;
* otherwise `A ∈ E ∖ P` — **`A` is a false negative**.

So **every element of `X` is an error of §2**, whatever the roots are. The
`67`-edges are excluded, so **`X` survives repair #1 of `PHASE7_VALIDATION.md`
§8a** — this is a bound on the *repaired* predicate, not the refuted one.

**Reported both ways, because the horn matters:**

| quantity | meaning |
|---|---|
| `\|X\|` | contradiction instances (TU × name) |
| `π = \|E\| / (\|E\| + \|X\|)` | **precision ceiling** under the recall-1 horn (every emitted body is predicted) |
| `B` = `\|{(t,A) : A ∈ E(t), A has a 26-edge into X}\|`, and `B/\|E\|` | the **false-negative blame set** — how much of `E` §2 must disown to escape the FP horn |

`X_any` (any non-`67` resolvable token, the loose extractor `model.ref_graph`
uses) and the transitive closure `\|closure₂₆(E) ∩ U\|` are reported as **upper
variants for sensitivity only**; no conclusion rests on them.

## 3. Frozen predictions — incumbents named, never a bare threshold

**Incumbents, computed on the same `U` at scoring time, never asserted:**

| incumbent | predicted set | its precision |
|---|---|---|
| **never-emit** | `∅` | recall 0, F1 0 — the trivial floor |
| **emit-everything** (the port's behaviour today) | `U(t)` | `\|E\|/\|U\|`, expected ≈ **0.116** |
| **§2 + repair #1** | `⊇ E ∪ X` | `π` above |
| w-emitpred's virtual-slot-only ceiling (`MAGNITUDE.md`) | — | **0.99629** at rev `9ad5c4c8` |

| # | quantity | point | interval | what it decides |
|---|---|---:|---|---|
| **W1** | `\|X\|` (strict, `26`-edges) | **250 000** | [50 000, 800 000] | the size of the class nobody measured |
| **W2** | TUs with `\|X(t)\| > 0` | **820** of ~850 | [600, 850] | breadth — is it a construct or a curiosity |
| **W3** | **`π`** | **0.41** | [0.18, 0.78] | **the ship gate.** w-emitpred registered V3 ≥ 0.95 to ship into fail-closed R3 |
| **W4** | `π > \|E\|/\|U\|` (§2 beats emit-everything on precision) | **TRUE**, by ≥ 20 pp | — | §2 is a *real model* even if unshippable. **A two-sided prediction: I expect §2 to be far better than trivial AND far short of shippable.** |
| **W5** | `B/\|E\|` | **0.60** | [0.30, 0.90] | the size of the other horn |
| **W6** | `X_any / X_strict` | **2.0** | [1.0, 5.0] | how much of `X` is the token over-approximation |
| **W7** | share of `X` whose target `F` is `??_`-prefixed (synthesized families, #152's territory) | **0.15** | [0.02, 0.45] | attribution: is `X` really #152's gap wearing #161's clothes |

**W3 is the headline and it is falsifiable in the direction that hurts me:**
if `π ≥ 0.95`, §2-as-repaired **transfers**, this lane's thesis is wrong, and
the correct report is "the fit transferred; build R3".

### Registered before the numbers exist: what `X` will NOT show

Restating board #150's shape (`w-afail` §8, fifth instance) so it cannot be
claimed afterwards:

* **`\|X\|` does not predict TU yield.** A is necessary and not sufficient;
  perfect A converts **0** TUs alone and moves `A∧B∧C` 25 → 107.
* **`\|X\|` is not a work queue.** It is a bound on a model's error, not a list
  of things to fix.
* **A large `X` does not name the correct predicate.** Refuting §2 quantitatively
  says nothing about what replaces it, and this lane will not fit one.

## 4. Known-answer controls — registered before running

| # | control | pass condition |
|---|---|---|
| **KA1** | `magnitude/gate.py` — the `67`-vs-`26` discriminator on 12 designed probe cells | **12/12**, unchanged |
| **KA2** | reproduce `MAGNITUDE.md`'s virtual-slot class with the unmodified `detect.py` at **my** dc3 rev | **649 instances / 289 TUs ± 10 %**. dc3 moved `9ad5c4c8 → 940d07dc`, so an exact match is not expected; a miss > 25 % means my instrument is not w-emitpred's and I say so |
| **KA3** | `E ⊆ U` coverage (`coverage.py`) | **≥ 99.9 %** of emitted names have a `.gl`-named body (w-emitpred: 174 404 / 174 410) |
| **KA4** | the **`26`-edge extractor itself**, on the probe cells: `f2` (`pc->nv(x)`, non-virtual) and `f3` (`pc->C::v(x)`, qualified) must each yield a `26`-edge to the emitted target; `f1` (`pc->v(x)`) must yield **no** `26`-edge to `?v@C` — only a `67`-edge | 3/3. **This is the one control w-emitpred never ran**: its gate validated the `67` side only, and my headline rides on the `26` side |
| **KA5** | incumbent gate on the unmodified tree | `cargo test --workspace --release` **0 failed** / 25 targets; `gate.sh --jobs 6` **12/12, 0 mismatch**; `selftest` **219 PASS 0 FAIL**; `cargo build --release` 0 warnings; **`census/gate disagreement` 0** |
| **KA6** | hand check of `X`, n = 20, seeded uniform draw | ≥ 15/20 confirmed by locating the call in the dc3 sources or by demangling to a plausible header-inline callee. **Below 15/20 fires the decline clause in §5.** |

**Quarantine.** The 21 TUs quarantined by `_2026-08-02-w-emitpred-prereg.md`
(`magnitude/heldout.txt`) remain quarantined. **No c2 output is read for them**;
`truth_all.py` is pointed only at `truthlist.txt`. This lane does **not** spend
the held-out population — it makes no prediction on it — and says so explicitly
so a future lane can still run Part 1 once.

## 5. The decline clause, priced in advance

* **If KA2 misses by > 25 % or KA3 < 99 %:** my instrument is not
  `MAGNITUDE.md`'s. I **decline to publish `X` as a bound** and report the
  instrument disagreement as this lane's finding instead.
* **If `W6 = X_any/X_strict > 5`:** the token over-approximation dominates. I
  **decline to quote `π` as a point** and publish only the strict number with
  the artifact rate beside it.
* **If KA6 < 15/20:** I **decline to quote `π` at all** and publish only the
  order of magnitude with the measured artifact rate.
* **If `π < 0.95` (the expected case): I will NOT implement §2 in `PortC2`,
  and that is a deliverable, not a shortfall.** Stated now so it cannot be read
  later as a failure to deliver. A fail-closed R3 built on a predicate with
  `π < 0.95` has exactly two behaviours: refuse on the construct — which
  `MAGNITUDE.md` §8 already prices at **more than one TU in three** for the
  virtual-slot class *alone* — or emit a wrong obj, i.e. **mismatch > 0**, which
  this project's one correctness rule forbids. **Never widen the gate to make
  something pass.**
* **If `π ≥ 0.95`:** §2-as-repaired transfers. I report that as the headline,
  say my W3 was refuted, and hand R3 a validated precision figure. **No
  implementation is attempted on the strength of one lane's measurement**; the
  prereg'd one-shot Part-1 gate is still owed and its population is still
  unspent.

## 6. What this lane will not do

* **No root model.** Not fitted, not guessed, not "for the denominator".
* **No re-fitting §2.** If `X` names an axis, that axis is reported as a
  characterized boundary; iteration happens on new designed probe cells in a
  future lane (w-emitpred's standing rule).
* **No widening of `PortC2`'s accepted class.** `NotImplemented` outside the
  ported class is the open gate, not a defect.
* **No board numbers minted, no `#N` pinned in code.** Rows are proposed
  lettered (`P-a …`), as w-afail did; `BOARD.md`, `ROADMAP.md` and
  `rungs/INDEX.md` are not edited by this lane.
* **No neutrality / behaviour-preserving classifier as a gate.** The obj
  byte-compare stays the sole judge.

## 7. Declared bias

**Deflationary.** I was briefed that "the emit predicate is the critical path"
and that a refutation of a landed plan is worth more than a partial
implementation — so my incentive is to find `X` large. Guards:

1. **W4 is registered in §2's favour** and scored the same as the rest: I have
   pre-committed to reporting that §2 beats the trivial incumbent by ≥ 20 pp.
   If `π` comes in *below* emit-everything's precision, that is a MISS I must
   publish, and it would mean my extractor is broken, not that §2 is worse than
   nothing.
2. Every bound is reported with the horn it rests on (`B/|E|`), so "§2 is
   wrong" cannot be asserted without saying *which* way it is wrong.
3. The strict/loose split (W6) is registered *before* seeing either, so I
   cannot pick the extractor that flatters the conclusion.
4. TU match is expected to be **8 at both ends of this lane**. A flat payoff
   metric is the pre-registered outcome, not a shortfall.
