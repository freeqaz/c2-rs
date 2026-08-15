# w-vocabgap — PREREG

Frozen as this lane's **first commit**, before the first statistic of the
lane's own question was computed. The 878-TU base scan
(`./work/w-vocabgap/scan.sh base`) was run **before** this file, because it is
the lane's required identity **control** and because §0 below is a correction
to a handed-down number that comes out of that control. **No per-TU
blocker-set statistic — no cardinality, no coverage, no set — was read before
this file was committed.**

---

## 0. A handed-down number, checked against the control BEFORE predicting anything

The dispatch brief requires the identity diff to read **`370` anchored
`gap-metric` keys**. Measured at master **`55933035`**, with `w-stmt5`'s own
anchored collector (`^ *gap-metric [^ ]+ `, board **#3181**):

```
anchored gap-metric keys: 394
```

**The brief's 370 is stale, and it is stale by exactly `w-stmt5`'s own +24.**
`#3181` published `370 → 394: 24 NEW, 0 GONE, 0 MOVED`, and `w-stmt5` is
merged at this master (`868c7aba`). 370 was the count on the tree *before* the
lane that corrected 372 → 370 landed its own keys. **The measurement wins**
(brief's own rule). Every other figure the brief pins reproduces exactly:
`match 25 · mismatch 0 · codegen-gap 0 · vocab-gap 845 · capture-fail 8 ·
port-error 0 · frontier 2 · fnbyte-exact 35,734 · fnbyte-denominator 162,049 ·
fnbyte-refused-parse 113,612 · fnbyte-refused-codegen 949`, 878 verdict rows.

This is the **eighth** handed-down number a lane has had to re-derive in this
wave, and the first where the stale figure is the *corrected* one quoted one
merge too early.

---

## 1. THE POPULATIONS, registered by name with their denominators

Every number this lane publishes names one of these. **`match` has three
meanings (#3125)** and the one used here is registered as D1.

| id | population | denominator | real or counterfactual |
|---|---|---:|---|
| **D1** | **`vocab-gap` TUs on the 878-TU dc3 workload** — *not* the 381×18 fixture gate, *not* `c2rs perf`'s `/Ox` gate | **845** of 878 scanned (870 graded; 25 `match`, 8 `capture-fail`) | **REAL** |
| **D2** | emitted functions refused by the **reader** — `emit_blockers`' own sum | **113,612** = `fnbyte-refused-parse`, of `fnbyte-denominator` **162,049** | **REAL** |
| **D3** | emitted functions the port already emits **byte-exactly** | **35,734** of 162,049 | **REAL — the only one that ships** |
| **D4** | distinct `emit_blockers` **keys** at base | **615** (`readphase` §0) — re-measured, not assumed | REAL |
| **D5** | **reach** (`expr-chain-fntail` + friends, through the committed sinks) | 88,806 / 93,990 of 120,456 | **COUNTERFACTUAL.** Used in §4's ladder only, and **never as the numerator of a TU claim** |

**#3138's rule, adopted verbatim:** reach and shipped bytes are not two views
of one quantity. `op:33`+`op:B9` are 7,124 **real** `fnbyte-exact` functions at
**0** reach; `op:29`+`op:3A`+`op:4F` are 266,417 of **reach** at **0** real
functions. Every number below carries its `D`-id.

---

## 2. THE INSTRUMENT, defined before it is run

For a TU `t`, **`S(t)` is the SET of distinct `emit_blockers` keys on `t`** —
the set, not the mass. A granted key set `G` **key-covers** `t` iff
`S(t) ⊆ G`.

**Why the SET and not the mass, stated as the lane's whole thesis.** A TU
leaves `vocab-gap` only when the reader stops refusing **every** function of
it. So the TU-quantity question is a **conjunction over each TU's own set** —
a covering problem — and every published ranking of this population is a
**sum over a mass**. A mass ranking answers "which key blocks the most
functions"; it cannot answer "which TU is closest", because those are
different quantities and the second one is the goal's.

### 2.1 Three bounds this instrument has, registered before it produces a number

1. **`S(t)` is a FIRST-blocker set, so key-covering `t` is NECESSARY and NOT
   SUFFICIENT.** `#3095`: `decode_causes` under-reports the head class's
   arity by up to **725×**, and rung 3 of that ladder is *not a clause at all*
   — it is a 615-key space. Every coverage number below is therefore a
   **CEILING on TU yield with NO discount factor**, and simultaneously a
   **LOWER BOUND on the work**. It is the 1.0002× kind of counterfactual — a
   counterfactual of the production being widened — so per ROADMAP's own rule
   the ceiling **is** the estimate.
2. **Key-covering `t` does not convert `t` even in the limit.** A byte-exact
   obj requires **A ∧ B ∧ C ∧ (D ∨ E)** (`factors.rs`). Reader work moves the
   route to **D**; it does not move A, B or C. §5 measures that ceiling
   separately and it binds *above* §3's.
3. **`emit_blockers` is itself an emitted-population instrument**, and
   `#3107`/`w-read2` §2 showed a published residue ranking of it can be
   counterfactual key-for-key. Base readings only; no ceiling reading is
   quoted as a base one.

---

## 3. PREDICTIONS — probability form, numerator and denominator together

**Breadth of the per-TU blocker set (D1 = 845 TUs, D4 = keys):**

| id | registered | p |
|---|---|---:|
| **P-A** | **median `\|S(t)\|` ≥ 10 keys**, over the 845 | 0.80 |
| **P-B** | **max `\|S(t)\|` ≥ 100 keys**, over the 845 | 0.70 |
| **P-C** | **≤ 5 of 845** TUs have `\|S(t)\| = 1` (a singleton blocker set) | 0.75 |

**The TU-marginal of a key, and its subset structure (D1 = 845):**

| id | registered | p |
|---|---|---:|
| **P-D** | **max single-key TU-coverage = 0 of 845**: no key, granted alone, key-covers even one TU | 0.60 |
| **P-E** | **the TU-quantity mirror of #3139**: the sum of all 615 single-key TU-marginals is **0 of 845**, while some set of **≤ 50** keys key-covers **≥ 1 of 845**. Zeros that do not compose, in the goal's own unit | 0.65 |
| **P-F** | greedy max-coverage needs **≥ 150 keys** to key-cover **100 of 845** | 0.60 |

**The published mass order, scored in the TU quantity (D1 = 845, D2 = 113,612):**

| id | registered | p |
|---|---|---:|
| **P-G** | the **top 20 keys by mass** — the head of `CEILING` §3.1 / `readphase` §0's widening order, 19.0 % of which `#3137` showed is not even token-addressable — key-cover **0 of 845** | 0.85 |
| **P-H** | at the first K where the greedy coverage order reaches **10 of 845**, the mass order needs **≥ 3×** as many keys to reach the same 10 | 0.55 |

**The factor ceiling — the bound that sits ABOVE all reader work (D1 = 845):**

| id | registered | p |
|---|---|---:|
| **P-I** | **≤ 200 of 845** `vocab-gap` TUs have **A ∧ B ∧ C** all true. A byte-exact obj needs `A∧B∧C∧(D∨E)`; the reader moves the route to D and moves neither A, B nor C, so this is a ceiling on what *every* reader phase together can convert | 0.60 |
| **P-J** | **0 of 845** TUs have `\|S(t)\| = 0` at base — a `vocab-gap` TU with no first blocker at all would be a contradiction, and the count is printed as a **discriminating-cell control**, not assumed | 0.90 |

**The ladder — three scans, because a first-blocker count is not a distance:**

| id | registered | p |
|---|---|---:|
| **P-K** | at the expression ceiling + statement sink, **median `\|S(t)\|` FALLS** relative to base. Registered in the naive direction on purpose: `#3095`'s masking says it may **rise**, and either way the answer is informative | 0.55 |
| **P-L** | at that ceiling, **≥ 1 of 845** TUs reaches `\|S(t)\| = 0` | 0.50 |

**Required-zero and identity (Z):**

| id | registered | p |
|---|---|---:|
| **Z1** | `git diff master..HEAD -- crates/ scripts/` is **empty**, 0 lines — stronger than an identity diff of counts | 0.95 |
| **Z2** | base and tip scans agree on **394 of 394** anchored keys and **878 of 878** verdict lines | 0.97 |
| **Z3** | `match` 25 · `mismatch` 0 · `codegen-gap` 0 · `vocab-gap` 845 · `capture-fail` 8 · `fnbyte-exact` 35,734 at both ends | 0.97 |
| **Z4** | `cargo test --workspace --release` **1,643 / 0 / 42**, delta **+0** — this lane adds no code | 0.90 |

---

## 4. MUTANTS — every colour registered BEFORE the mutant runs

`w-loo`'s cautionary case is the reason M5/M6 exist: **without its zero-reach
guard its mutant printed 52 margins of 0 and read as a clean null.** The
analogous failure here is worse — an analyzer whose `S(t)` came back empty
would report **845 of 845 key-covered by the empty set**, which is the most
optimistic possible answer and looks like a triumph.

| # | mutant | predicted |
|---|---|---|
| **M1** | coverage test `S(t) ⊆ G` → `S(t) ∩ G ≠ ∅` | **RED** — the curve must move |
| **M2** | the analyzer scores all 878 rows instead of the 845 `vocab-gap` ones | **RED** — the denominator moves |
| **M3** | `S(t)` built as a mass-weighted multiset rather than a set | **RED** — the cardinality distribution moves |
| **M4** | a comment-only edit inside the analyzer | **GREEN** |
| **M5** | the totality guard broken: Σ over TUs of Σ `emit_blockers` ≠ 113,612 | **REFUSE**, exit non-zero |
| **M6** | every `vocab-gap` row's `emit_blockers` emptied | **REFUSE**, exit non-zero — must NOT print "845 of 845 covered" |
| **M7** | the graded-TU floor (< 800 rows) removed and a truncated jsonl fed in | **REFUSE**, exit non-zero |

**Every check reports a positive count.** *"The re-measure covered N > 0"*,
never *"found no discrepancies"* — the count of **discriminating cells** is
printed on every table.

---

## 5. DECLINED IN ADVANCE

| id | registered |
|---|---|
| **N1** | **No dispatch ranking, and no recommendation of one.** This repo is **0-for-4** on lanes dispatched off a mass ranking, with an 11-refuted-selector / 12-refuted-placement-rule / 12-deep allocation-key graveyard. If this lane's coverage order looks actionable, it states `w-loo` §7's five conditions and notes that **no ranking in this repo has ever forecast a conversion** — including this one |
| **N2** | **No duplication of peer `w-bind`.** `#3177` already named the reachable head as call-argument / data-symbol **binding** (1,825 of 3,062, 59.6 %). That lane owns the *mechanism* in `crates/c2-il`; this lane owns the *distribution* and touches no `crates/` byte |
| **N3** | **No re-derivation of `w-loo`'s token LOO.** Its zeros-do-not-compose bound is adopted as published, not re-run. This lane's subset structure is in the **TU** quantity, which that instrument cannot express |
| **N4** | `coff/` is off-limits and is not read |
