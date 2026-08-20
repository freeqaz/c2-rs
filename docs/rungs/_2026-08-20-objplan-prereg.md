# PREREG — w-objplan, the ObjPlan structural manifest (ARCHITECTURE_PROPOSAL §5 step 3, instrument half)

    Slug:      objplan
    Date:      2026-08-20
    Kind:      construct rung + instrument
    Frozen at: tree c277d3bb09ae0aa4a57e7b852682400304c2a638 (clean), branch
               wt-w-objplan off master c277d3bb0
    Status:    FROZEN. This file is written BEFORE any edit to crates/ and
               before any probe of the mechanism it registers predictions about.

---

## 0. Base facts, MEASURED at this base — never carried from the brief

Every one of these was read at this tree, in this worktree, on 2026-08-20.
The brief that dispatched this lane carried three of them and **two were
already stale** (see §0.1).

| fact | measured value | how |
|---|---|---|
| tree sha | `c277d3bb09ae0aa4a57e7b852682400304c2a638` | `git rev-parse HEAD` |
| **workload stamp** | **`3df8fd5412c2` (clean)** `/home/free/code/milohax/dc3-decomp` | the `workload` line `c2rs gap` prints |
| `gap-metric` key count | **395** | `grep -cE '^ *gap-metric \S+ \S+$' work/manifest/gap-base.log` |
| `tu-total` / `graded` | 878 / 870 | scan |
| `match` / `mismatch` | **26** / **0** | scan |
| `codegen-gap` / `vocab-gap` / `capture-fail` | 0 / 844 / 8 | scan |
| `factor-a` / `-a-lo` | 28 / 27 | scan |
| `factor-b` | **325** | scan |
| `factor-c` | 169 | scan |
| `b-and-c` | **149** | scan |
| `a-and-b-and-c` | **27** | scan |
| `frontier` | 2 | scan |
| control set | the 26 `match` source paths, `docs/plan/CONTROL_TUS.txt` | scan |

Base scan log: `work/manifest/gap-base.log` (untracked; `work/` is gitignored).
The worktree was provisioned with `scripts/configure_existing_worktree.sh`,
which asserted `fixtures/cpp/w5_chain.cpp -> 4/4 functions in class` — i.e. the
toolchain resolves and the differential is not silently SKIPping.

The base **suite** and **gate** are measured immediately after this commit and
recorded in the next one. They are deliberately not held hostage to the freeze:
no prediction below depends on their values, and the freeze is what must come
first.

### 0.1 DISPATCH DEFECT — two carried numbers were stale at this base

The brief states the state "measured 2026-08-19" as `factor-b`-adjacent facts
and `docs/STATUS.md`'s generated block (collected 2026-08-19, workload
`49ad7cfd5`) reads `factor-b 324`, `b-and-c 148`, workload `49ad7cfd5`.

At this base, one day later, the workload stamp is **`3df8fd5412c2`** — a
*fourth* stamp after 2026-08-19's three (`897d0220fd1d` → `49ad7cfd5d26` →
`eda64e956c87`) — and `factor-b` reads **325**, `b-and-c` reads **149**.
`match 26 / mismatch 0 / a-and-b-and-c 27 / factor-a 28 / factor-c 169 /
frontier 2` all reproduce. This is #3306/#3311's failure mode arriving on
schedule and is recorded here as a **measurement offered with no cause**
(rungs/README: a lane that finds an unexpected delta owes a measurement before
it owes a cause).

**Consequence for this lane:** every `plan-*` figure it publishes is a figure
at stamp `3df8fd5412c2`, and the lane asserts its own two ends read the SAME
stamp by diffing the two strings the scan itself emits.

---

## 1. What is being built, in one paragraph

Today progress is binary per TU: a TU counts only when the reader, the emit
set, the sections and the codegen all succeed. `a-and-b-and-c = 27` and
`match = 26` — **we have converted 26 of the 27 TUs that currently satisfy
every factor**, and the other 844 fail several ways at once, so every
single-stage improvement scores zero. This lane grades the **object plan**:
everything about the output obj that is independent of the instruction bytes.
That is gradeable against real c2 on all 870 graded TUs *today*, including the
844 that will never parse this year, so the conjunction becomes a set of
independently movable curves.

### 1.1 The tautology in the brief, and the fix (adopted from the plan)

Deliverable 3 as literally written — "the manifest must be identical on a TU we
already reproduce byte-exactly" — is **vacuous** if the port side of the
manifest is `observe(port_obj)`: on a `match` TU `port_bytes == ref_bytes`
(`ObjImage::diff`, timestamp-normalized), so `manifest(port) == manifest(ref)`
for *any* pure `manifest`. Worse, on the other 844 the port emits no obj, so the
port side is undefined and there is no curve. The naive reading delivers neither
deliverable 3 nor deliverable 4.

So the manifest has **two independent producers**:

* `ObjPlan::observe(&ObjImage) -> Option<ObjPlan>` — ground truth, read off the
  **reference** obj. Available on all graded TUs.
* `predict(&IlBundle, &PlanInputs) -> PredictedPlan` — the port's plan,
  computed **from IL, without emitting**, and forbidden by a source-level test
  from reaching the emitter.

Required-exact on the 26 is then a real constraint on `predict`, and the 844
get a curve because `predict` is defined on them.

### 1.2 The load-bearing constraint: `predict` must not route through `functions()`

`IlBundle::functions()` is the admission gate and returns `None` on the 844. A
`predict` that asks it for the emit set publishes `known ≈ 30 of 870` and every
one of the 844 reads `unknown` — that is not a curve, it is the reader's
refusal mass wearing new keys, which is trap **#3237** exactly ("returns 0
because it did not look" is indistinguishable from "returns 0 because there was
nothing to find"). The walk-free route is `c2_il::gl_function_attrs`
(`crates/c2-il/src/func/gl.rs:1527`) + `c2_il::mangled_names`
(`gl.rs:35`), both whole-file and both already computed on every TU.

---

## 2. Registered predictions

Probabilities are this lane's. Intervals are 80% credible unless stated.
**Ceilings carry no discount factor.**

### Reference side (the extractor)

* **R1** `ObjPlan::observe` returns `Some` on **≥ 860 of 870** graded TUs.
  p = 0.85, interval [852, 870]. *If < 800, the instrument's denominator is
  dishonest and the walk is fixed before any curve is published.*
* **R2** The agreement control is **100%** against `text_comdat_functions`,
  `section_names`, `weak_externals` and `text_comdat_relocs_named` on every TU
  where both sides decode. p = 0.93. *Any disagreement is a finding about the
  existing accessor or the new walk and is published either way.*
* **R3** `observe` is invariant under mutation of `.text` raw bytes on a
  synthetic image (body-independence). p = 0.97.

### The control (deliverable 3)

* **C1** At the tip, all **26** control TUs read `Exact` on every **shipped**
  component. p = 0.75.
* **C2** **≥ 3** of the 26 fail on the *first* implementation and the control
  catches a real extractor-or-predictor bug. p = 0.65. *A first-try 26/26 is
  weak evidence the manifest is too coarse to distinguish anything.*
* **C3** The named set found by the tip scan equals `docs/plan/CONTROL_TUS.txt`
  exactly. p = 0.90. *A difference is reported before anything else.*

### The curve (deliverable 4) — numbers that do not exist today

* **P1** `plan-emitset-members-known ≥ 700` of 870 via the `.gl`-attrs route.
  p = 0.60, interval [400, 860]. **Go/no-go for the whole step.** *If < 300 the
  predictor is routing through the reader's refusal mass and the curve is fake
  — see §4.*
* **P2** `plan-emitset-members-exact` (seed set == observed emit set) over 870:
  point **60**, interval **[10, 300]**, p = 0.70 that it lands in the interval.
  *§3E's own warning is that a seed-only rule over-deletes on real TUs, so a
  low number is the expected, informative result and it prices the closure lane.*
* **P3** `plan-emitset-seed-subset` (seed ⊆ observed — the seed never
  over-claims) ≥ **0.90** of `known`. p = 0.75. *Over-claiming would be a
  finding about §3E's bit, not about the port.*
* **P4** `plan-emitset-order-exact < plan-emitset-members-exact` strictly, by
  ≥ 10 TUs. p = 0.70. *If equal, order is free on this workload and must be
  labelled free.*
* **P5** `plan-emitset-members-exact` ≠ `factor-a` (28), by ≥ 5 in either
  direction. p = 0.85. *Factor A is a **count** identity and can pass by
  coincidence; a set identity cannot.*
* **P6** ≥ 1 TU is `factor-a == true` and `plan-emitset-members` `Differs` —
  a count identity that is set-false. p = 0.60. *A zero is also publishable: it
  would say the count identity has been an honest proxy all along.*
* **P7** `plan-sections-attrs-exact` over 870 is **< 100**, well under
  `factor-c` (169). p = 0.85, interval [5, 120]. *C is a subset test against a
  10-name vocabulary; this is an ordered equality including characteristics
  and alignment.*
* **P8** `plan-weak-known ≤ 30` at the end of the lane. p = 0.90. *The port
  models no weak externals. It must publish as `Unknown`, never as `Differs`
  and never as a 0-of-0 `Exact`.*
* **P9** At least **3** components read `distinct == 1` across the workload
  (free wins) and are excluded from the headline. p = 0.55. Candidate:
  `.drectve`.

### Identity / cost (the construct half)

* **I1** Every pre-existing `gap-metric` key value is byte-identical before and
  after, key-for-key. p = 0.95. **REQUIRED-ZERO.**
* **I2** `match 26 / mismatch 0`, gate 18/18, sweep and cross counts unchanged.
  p = 0.97. **REQUIRED-ZERO.**
* **I3** The added walk raises `c2rs gap` cost by **< 15%**, measured as a
  **CPU-time ratio against a control run in the same session** — never as
  absolute seconds. p = 0.70.
* **I4** `plan-bounds-violations == 0` on every run. p = 0.95.
  **REQUIRED-ZERO, published as a COUNT and not as a status.**

---

## 3. The grading criterion

**Primary (required-exact, the control).** On the TUs named in
`docs/plan/CONTROL_TUS.txt`, every **shipped** component must read `Exact`.
`Unknown` on a control TU is a **failure**, not a neutral — the port
demonstrably had the information, since it reproduced those objs
byte-for-byte. `Differs` on a control TU means the extractor or the predictor is
wrong and the component **does not ship**. A component whose control is red
ships as `Unknown`, never as `Differs`: the manifest must not claim to disagree
with the reference on a TU the byte judge has already called equal.

Every run prints: the named set found this run, the **identity diff** against
the committed file (entered / left, by name), and the per-TU verdict for each
control TU with the differing component named.

**Environment, asserted not assumed.** In every environment: the `compilers/`
symlink present, the **executed-test count** (not the exit code), and
`census_gate`'s **duration non-zero** with **0** `SKIP: toolchain absent`.

**Secondary (required-zero).** Every existing `gap-metric` key identical
key-for-key; census numerator, blocker histogram, `census/gate disagreement`,
`match 26 / mismatch 0`, gate 18/18, sweep and cross counts unchanged. The only
expected non-identity is the scan JSONL row bytes (new fields), accounted
explicitly.

**Tertiary (agreement control).** `observe` must agree with each existing
`c2-obj` accessor over the whole workload, TU by TU: `emit_set` vs
`text_comdat_functions()`; section names in order vs `section_names()`; `weak`
vs `weak_externals()`; `relocs` (type + target) vs `text_comdat_relocs_named()`.
Disagreement anywhere is a red. **The existing accessors are not replaced** —
a new walk plus an agreement assertion is cheaper, uncontended, and catches the
same drift.

**#3288 — every published count derived a second, differently-built way.** The
`plan-*` metrics come from `GapReport::metrics()` over the live results; a
parser over the `--plan-tsv` rows re-derives each one offline, in `sets.rs`'s
two-producers-one-definition shape, and a `check_metrics`-style block prints
every row's agreement.

**Workload stamp, both ends.** Read at base (§0) and again at the tip, and
asserted EQUAL by diffing the two strings the scan emits.

**NOT A GATE.** `plan-*` never fails `gate.sh`, never gates an emit, never
appears in a refusal predicate. The only `plan-*` figures allowed to be a hard
red are `plan-bounds-violations` (a containment invariant of the instrument
itself) and the named control. **The compiler remains the sole judge, and
`plan-exact` is NECESSARY but NOT SUFFICIENT for `match`** — a TU can be
plan-exact and mismatch on every byte.

---

## 4. What would justify DECLINING, registered in advance

* **P1 fails hard**: `plan-emitset-members-known < 300` of 870 and no walk-free
  route raises it. Then the plan curve is the reader's curve wearing new keys.
  Honest outcome: `declined`, with the measured whole-file refusal rate of
  `gl_function_attrs` as the deliverable.
* **No dynamic range**: if, after the first published keys, **no component's
  `differs` count exceeds 20 TUs**, the manifest distinguishes nothing. A flat
  line is not a curve; it is #3237 with more keys. Decline rather than publish.
* **The control cannot be made green**: if ≥ 5 of the 26 keep failing
  required-exact after both sides are audited, ship the components that pass,
  `Unknown` the rest, and say so.
* **Required-zero breaks and cannot be accounted**: revert, commit the revert
  with its reasoning, report **FAILED** in that word.

---

## 5. What the manifest CANNOT see — stated because #3237

* Every instruction byte, and therefore: `.text` section sizes, COMDAT aux
  `CheckSum` for code sections, symbol `Value` for in-body labels, relocation
  `VirtualAddress` inside code, `.pdata` prolog/epilog fields, and the
  `.debug$S` subsections keyed to code offsets.
* The 8 `capture-fail` TUs. No reference obj → not a row, never a false row.
* Any statement about correctness (see §3's last paragraph).
* Whether a difference is the port's fault or the extractor's — that is what
  the named control on the 26 is for.

**Denominator, chosen deliberately: TUs, not functions and not bytes.**
Option A's target is stated in TUs (870/878), so the curve's denominator is the
goal's denominator. This sidesteps #3254 entirely: the `fnbyte` denominator is
71.2% bodies the shipped image never contains because `/Gy` COMDATs get
discarded by the linker — a *function*-level denominator problem. A TU is a TU;
there is no discard. Each component nonetheless publishes **three** denominators
and never a bare ratio: `observable` ⊇ `known` ⊇ `exact`, with `differs`
derived in `metrics()` and never by the reader (board #213).

---

## 6. Unknowns this lane must MEASURE before publishing anything that rests on them

1. The workload stamp and the `gap-metric` key count at base **and** at tip.
   (Base: done, §0.)
2. `observe`'s decode rate over the 870 reference objs (R1).
3. **`gl_function_attrs`'s whole-file refusal rate over the 870.** This is P1's
   ceiling and the single most decision-relevant unknown. It refuses the entire
   file on an unrecognized `SRCPOS`/`SIZE` encoding and refused 60 of 870
   before lane `w-glattrs`. Measure it BEFORE the grader is written.
4. **Whether bit `0x20` in the byte `gl_function_attrs` returns is in fact
   §3E's emit bit on this workload.** The reader's doc says only bit 6
   (`FN_FLAG_INLINABLE`) is *consumed*. The cheap check: on the 26 control TUs,
   is `{name : attr & 0x20}` ⊆ the observed emit set? **If it over-claims
   there, the identification is wrong and the DISCLOSURE row must not be
   written.**
5. Whether `.gl` record order relates to COMDAT section order at all (P4's
   baseline).
6. How many of the 870 actually carry weak externals, COMDAT selections beyond
   `NODUPLICATES`, and out-of-vocabulary sections. The brief's "675 TUs" and
   "450 TUs" are CARRIED figures — re-derive, do not quote.
7. Which components are free (`distinct == 1`).
8. The relationship between `factor-a` (28) and `plan-emitset-members-exact`
   (P5/P6).
9. The cost of the extra walk (I3), as a CPU-time ratio in one session.

## 7. Retroactivity (rungs/README rule 2)

The results table is **derived from the scan logs and the `--plan-tsv` files**,
never accumulated. Every classification (`Exact`/`Differs`/`Unknown`/
`Unobservable`, and the free-component labelling) must be re-appliable
retroactively to a stored log if the rule turns out wrong — it was wrong three
times in `w-mutcensus`.
