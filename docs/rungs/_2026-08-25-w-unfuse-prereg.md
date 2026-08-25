# PREREG — `w-unfuse`, separate DECODE from ADMISSION in the `BodyShape` path

    Tag:       w-unfuse
    Date:      2026-08-25
    Kind:      construct rung
    Base:      5db186426  (master, clean; `decisions: decision 13 — the GENERAL DECODE (row 4a(i) / I1) is funded`)
    Board:     #3554–#3560 (reserved by the coordinator for this lane)
    Fixtures:  none — construct rung: separates the decode seam from the admission
               predicate by re-expressing the already-byte-exact class through it
    Census:    +0

**Frozen before any measurement of this lane's own changes.** Predictions below
are never edited afterwards. Navigation pointers may be repaired by *amending
beside* (`#3495`'s convention); a number or a probability may not.

---

## 0. What this lane is

Decision 13 (`docs/DECISIONS_2026-08-22.md`) funds row 4a(i) — a general
op-level IL decode — and names the obstacle in `ARCHITECTURE_PROPOSAL_2026-08-20.md`
§8's own wording:

> *"IR0 stops at a two-variant byte framing and `BodyShape` starts at 35
> whole-function grammars **that are simultaneously the admission gate**, so
> the semantic middle a COLOR pass would consume does not exist."*

**Decode and admission are fused.** This lane unfuses them, and **nothing
else**: the admitted set must be identical *function for function* afterwards.

This is a **construct rung**: `Fixtures: none`, `Census: +0`, **required-zero
byte delta**, graded by a strict identity diff of the **21 count-bearing gate
rows** against base `5db186426`.

### 0.1 The fusion, as read at base (an inventory, not a claim)

Read at `5db186426`. Line numbers stated because the brief's were stale and a
future reader will want to know which ones this lane worked from.

| site | file:line at base | role |
|---|---|---|
| `parse_segment` | `crates/c2-il/src/func/body/mod.rs:1917` | **THE FUSED ENTRY.** `Option<BodyShape>`: `None` conflates *"this port cannot read the IL"* with *"this port may not emit it"* |
| `parse_segment_detail` | `crates/c2-il/src/func/body/mod.rs:1924` | the *same* parse with the refusal reason kept. Its own doc: *"`parse_segment` is `.ok()` of this"* |
| `parse_segment_shape` | `crates/c2-il/src/func/body/mod.rs:1948` | the recursive-descent parse itself |
| `BodyShape` (the parser's) | `crates/c2-il/src/func/body/mod.rs:545` | the 35 whole-function grammars that ARE the admission gate |
| `Block` | `crates/c2-il/src/func/body/mod.rs:1088` | the decode's stopping point, already computed on every refusal, and thrown away by the gate |
| `Block::feature` | `crates/c2-il/src/func/body/mod.rs:1607` | the gap key rendered from it |
| `call_tokens` | `crates/c2-il/src/func/body/mod.rs:212` | a decode fact that admission provably does not consult — its own doc says so |

**Five production call sites of the fused predicate**, enumerated (all of
`crates/c2-il/src/`, no other crate can reach it — both entries are
`pub(crate)`):

| # | site | entry it calls | what it wants |
|---|---|---|---|
| 1 | `func/bundle.rs:2105` (`IlBundle::functions`) | `parse_segment` | **admission** |
| 2 | `func/bundle.rs:2536` (`IlBundle::dyninit_tu`) | `parse_segment` | **admission** |
| 3 | `func/diag.rs:398` | `parse_segment` | admission, counted (`bodies_out_of_class`) |
| 4 | `func/census.rs:607` | `parse_segment_detail` | admission (`let Ok(sh) = … else { continue }`) |
| 5 | `func/census.rs:939` | `parse_segment_detail` | **both** — the shape *and* the `Block` |

`c2-core::codegen::select_function` (`crates/c2-core/src/codegen/select.rs:403`)
dispatches on the **other** `BodyShape` — the public `c2_il::func::BodyShape`
that `shape_to_function` produces. It is downstream of admission and this lane
does not touch its dispatch order.

**These are the denominators as believed now: 2 fused entry points, 5
production call sites, 0 out-of-crate callers.** If the inventory is wrong the
lane says so in §7 rather than quietly working from a different one.

### 0.2 The shape of the split (registered before it is built)

1. **`Decoded`** — one type, one constructor, **total over segments**. It
   answers *"what does this IL say"*. It never refuses on admission grounds and
   it carries no admission verdict. It exposes, at minimum: the recognized
   whole-body grammar if the decode reached one; the `Block` if it did not; and
   at least one decode fact that admission provably does not consult
   (`call_tokens`, whose own doc at base reads *"Diagnostic only. Nothing here
   is consulted by the emitter or by acceptance."*).
2. **`AdmissionPolicy` + an `admit`-shaped predicate over `&Decoded`** — it
   answers *"may the port emit this"*, and it is the **only** thing that does.
   `AdmissionPolicy` is a **named, settable parameter** whose default reproduces
   today exactly (`docs/rungs/README.md` § Lane kinds, THE DECISION-SURFACE
   CLAUSE). Every non-default value is an instrument state and licenses no emit.
3. **All five production sites route through (1) then (2).** The fused
   `Option`-returning entry does not survive in production.
4. **`census/gate disagreement: 0` becomes structural rather than conventional.**
   Today it holds because `parse_segment == parse_segment_detail.ok()` by
   construction and a comment says so. After the split both sides read one
   `Decoded` through one `admit`, so it holds because there is only one
   predicate. This lane must not ship two.

**Explicitly NOT in scope**: any general op-level decode (that is 4a(i)'s
15–45 engineer-month body, and this lane is its prerequisite, not a slice of
it); any widening of the admitted set; `select_function`'s dispatch order;
anything under `crates/c2-harness` (peer `w-decodereach`) or `scripts/`
(peer `w-guard`).

### 0.3 THE COST CLAUSE (#3336, as amended 2026-08-21) — the axes this rung can fail on with every byte identical

A required-zero **byte** delta is silent about everything that is not a byte,
and a criterion that cannot fail abstains rather than passes. Three axes,
named before starting:

1. **The census/gate agreement denominator.** The split's whole hazard is
   shipping *two* predicates that agree today and drift tomorrow. Observed by:
   the `census/gate disagreement` line of the 878-TU `c2rs gap` scan, at base
   and at tip, plus the whole `gap-metric` key table diffed key-for-key.
2. **Throughput.** `Decoded` sits on `PortC2::build`'s hottest path — the port
   runs ~922k obj/s on one thread and this is one allocation/indirection per
   function body. Observed by: `c2rs perf` geomean with its fixture count, base
   and tip, back to back on the same box, **quoted beside `#3551`'s measured
   build-to-build cost floor `F = 0.93 %`** — a reading inside that band is
   not a reading.
3. **The census-key population.** `dispatch_reset()` runs inside
   `parse_segment_detail` today, and the `Block` carried out of a refusal is
   what `Block::feature` renders. Moving where either happens moves per-key
   census counts while every emitted byte stays identical (this is trap 0's
   shape). Observed by: the full `gap-metric` key list (name **and** value)
   diffed base vs tip, and the workspace suite's census tests.

---

## 1. Predictions

Each with a probability, frozen. Scored in the rung.

| # | prediction | P |
|---|---|---|
| **P1** | The strict identity diff of the **21 count-bearing gate rows** (18 mode lanes + `expr-sweep` + `mode-cross` + `debug-lane`; `hatch-red`/`ladder-red` excluded as `n/a`), tip against base `5db186426`, is **0 lines** | 0.80 |
| **P2** | `C2RS_REQUIRE_TOOLCHAIN=1 cargo test --workspace --release` passes at tip with **no test that passed at base failing at tip**; the target count is unchanged at 54 and the passing count moves only by the tests this lane adds | 0.88 |
| **P3** | **The diff can fail, control A (a KNOWN signature).** Fabricating `#3515`'s mutation — `c2_core::comdat::INLINE_DECLINE_LOOP_BYTES` 80 → 4096 — against my own base and my own diff procedure reproduces **exactly 7 moved rows / 14 diff lines**: `O1` −1, `O1-EHsc` −1, `O1-Oi` −1, `O1-Oi-EHsc` −1, `O1-Oi-GR` −1, `O1-Oi-EHsc-GR` −1, `debug-lane` −6, with `/Ox`(×4), `/O2`(×2), `/Od`(×4), `expr-sweep`, `mode-cross` unmoved | 0.80 |
| **P4** | **The diff can fail, control B (MY OWN seam).** Making `admit` refuse one `BodyShape` variant that the port emits today — `EmptyBody` — moves **≥ 10 of the 21 rows**, and `debug-lane`'s delta equals the sum of the 18 mode-lane deltas | 0.85 |
| **P5** | `census/gate disagreement` reads **0** on the 878-TU scan at tip, as at base | 0.90 |
| **P6** | Every `gap-metric` key present at base is present at tip **with the same value**, and no key is added or removed by this lane | 0.85 |
| **P7** | `\|Δ c2rs perf geomean\|` between base and tip, same box, back to back, same fixture count, is **≤ 3 %** — reported beside `F = 0.93 %`, and NOT claimed as a null if it lands inside that floor | 0.70 |
| **P8** | Outcome word is **`built`** | 0.72 |

**P3 and P4 are the load-bearing pair and they are not redundant.** P3 grades
the *procedure* (cut to `LANE VERDICT graded/total match`, normalise
`/tmp/c2rs-gate-<pid>` → `RUNDIR`, drop the two `n/a` rows) against a signature
somebody else measured. P4 grades the *seam*: #3346's point is that a
construct rung's zero byte delta is evidence only when the re-expression is on
a path the gate exercises, and a mutation of `admit` that moves nothing would
mean `admit` is not on that path. **A diff I have never seen fail is not a
check, and a seam I have never seen the diff notice is not a seam.**

---

## 2. What would falsify "the split is behaviour-preserving"

Registered so it cannot be renegotiated after the fact. **Any one of these is
a falsification**, not a discussion:

1. Any non-empty line in the 21-row identity diff (base vs tip).
2. `mismatch` > 0 in any gate lane, the sweep, the cross, or the 878-TU scan.
3. `census/gate disagreement` ≠ 0 at tip.
4. Any `gap-metric` key added, removed, or moved in value.
5. Any workspace test green at base and red at tip.
6. The 878-TU scan's `match` moving off its base value in **either** direction.
   **An increase falsifies it too** — this lane is forbidden to widen, so a
   conversion is a bug in the split, not a bonus (§4).
7. Per-symbol movement in the admitted set. Aggregate equality is not equality:
   `w-empty`'s first attempt read `+0 / −14` and an aggregate cannot
   distinguish `+1,400 / −27` from `+1,373 / −0`. **The admitted set is
   compared function-name for function-name, not by count.**

---

## 3. The decline floor

Registered before starting, per `docs/rungs/README.md`:

* **If the split cannot be made without moving a byte**, the lane says
  **`declined`** or **`FAILED`** in those words, ships nothing to `crates/`,
  and reports what was tried and what it cost. It does **not** ship a
  half-split leaving two predicates that can drift apart — duplicate
  implementations of one fact are the merge failure textual conflict detection
  structurally cannot see.
* **Two debugging attempts** at a non-zero identity diff. If the third run of
  the 21-row diff is still non-empty, the lane declines and reports the diff.
* **If the split makes a widening look free or obvious, the lane STOPS and
  reports it as a finding with its two-sided price** (`#1042`, NC-5/`#2691` —
  both times the refusal's own cost was counted and the answer flipped). It
  does not take the widening. That is the whole point of unfusing: widening
  becomes a separate, priceable decision instead of a side effect.
* **If a peer's file is needed** (`crates/c2-harness`, `scripts/`,
  `crates/c2-harness/tests/`) the lane STOPS and reports — the wave-7
  precedent, restated at decision 13's concurrency fences.

---

## 4. The scope fence, stated as a prohibition

**Do not widen what is admitted. Not by one function.** `S0` measured what
naive widening ships: **blind-differs 96.1 %** of what it reached. Under
`docs/PROGRESS_METRIC.md` a wrong emit scores strictly below the refusal it
replaced.

Concretely, this lane may not:

* add a `BodyShape` variant, or admit an existing one at a site that refused it;
* relax any clause of `IlBundle::functions` (the `.drectve` gate, the label
  counter gate, the unclaimed-`.gl` accounting, the varargs gate, the
  string-literal refusal, `callee_defined_here`);
* change `select_function`'s dispatch order (load-bearing, `select.rs:403`);
* change what `Block::feature` renders for any refusal.

---

## 5. The concurrency question, asked explicitly

**After this change, does every existing reader of the fused predicate still
read what it did?** Concurrent lanes in this repo have erased each other
through shared predicates with no textual conflict and no red gate — three
separate mechanisms in one wave. The five readers of §0.1 are enumerated
*before* the change precisely so the question has a denominator; §7 answers it
reader by reader, not in aggregate.

`census/gate disagreement: 0` is a **shared invariant**. This lane must not
break it silently, and P5 is where it is scored.

---

## 6. What gets measured, in order

1. Base gate at `5db186426`: `sh scripts/gate.sh --jobs 16 --require-graded`,
   table saved. Base 878-TU scan, `gap-metric` keys saved. Base `c2rs perf`.
2. Control A (P3): fabricate `#3515`'s mutation on the base tree, gate, diff,
   revert, confirm clean.
3. Build the split. Gate, diff (P1), scan (P5/P6), perf (P7).
4. Control B (P4): mutate `admit` on the tip tree, gate, diff, revert, confirm
   clean.
5. Per-symbol admitted-set comparison, base vs tip (falsifier 7).

## 7. Board rows this lane will mint

`#3554`–`#3560`, minted in `docs/BOARD.md` in the commit that uses them. Not
pre-titled here — a row minted before its finding exists is a row whose
sentence was written before the measurement, which is the thing prereg exists
to prevent. What is registered is the **range** and that no number outside it
is taken.
