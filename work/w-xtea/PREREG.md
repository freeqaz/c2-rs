# PREREG — lane `w-xtea`

Frozen 2026-08-09, at worktree base `42fe7cb1` (master tip), **before any tree
change, before the ladder climb, and before any codegen probe.**

## 0. What I had already seen when this was frozen (full disclosure)

Honesty beats a fiction of a clean slate. Before freezing I had run *read-only
baseline* measurements only — no tree change, no experiment:

1. `work/w-frame/refobj.sh` on `src/system/utl/EncryptXTEA.cpp` at the
   workload's own `flags.txt`, and `scripts/gt_dump.py` over the result.
   So **I have seen the reference disassembly of all five functions.**
2. One `c2rs gap --list` over that single TU, giving its blocker keys and its
   `bytefrac`/`fnbyte` cells.

Everything below is registered *knowing* the disassembly. That makes the
codegen-price prediction (§3) a **weaker** claim than a blind one — it is a
prediction about what a *derivation* will cost, made after reading the target,
not a prediction about what the target looks like. The reader-ladder prediction
(§3b) is blind: I have not climbed the ladder.

## 1. Baseline, measured at this tree

| cell | value | source |
|---|---|---|
| 878-TU scan | `match 19, mismatch 0, codegen-gap 0, vocab-gap 852, capture-fail 7` | `docs/STATUS.md` generated block |
| `fnbyte-exact` (workload) | 35,982 | `docs/STATUS.md` trap table |
| `cargo test --workspace --release` | 1,387 pass / 38 target | brief |
| this TU: class | `vocab-gap` — `il function decode failed` | single-TU gap, this tree |
| this TU: `bytefrac` | accepted 16 / den 272 / refused 256 = **5.9 %** | ditto |
| this TU: `fnbyte` | `exact 1`, `refused 4`, all four `fnbyte-decline\|parse` | ditto |
| this TU: blockers | `expr-intrinsic-memcpy` 1, `expr-load-type-8882` 1, `expr-op-0x27` 2 | ditto |

## 2. The commission

Convert `src/system/utl/EncryptXTEA.cpp` to byte-exact: **match 19 → 20**.

## 3. Predictions, in probability form

**Outcome predictions (the scored ones — census-only predictions are UNSCORED,
CEILING §10, so every row below is `fnbyte-exact`, TU match, `mismatch`, or
test count):**

| # | claim | P |
|---|---|---|
| R1 | **TU match delta = +1** (19 → 20), i.e. the commission is completed | **0.04** |
| R2 | TU match delta = 0 | 0.96 |
| R3 | `mismatch` stays 0 on the 878-TU scan and every gate | 0.98 |
| R4 | `fnbyte-exact` delta **> 0** on the 878-TU scan | 0.30 |
| R5 | `fnbyte-exact` delta **≥ +2** | 0.12 |
| R6 | `fnbyte-exact` delta **≥ +50** (i.e. whatever I ship generalizes off this TU) | 0.06 |
| R7 | `fnbyte-exact` delta **< 0** (a regression — must be 0) | 0.02 |
| R8 | test-count delta in **[+6, +30]** | 0.75 |
| R9 | test-count delta **≥ +1** and TARGET count **≥ 38** (never shrinks) | 0.97 |
| R10 | the lane's primary shipped artifact is a **priced decline**, not a conversion | 0.80 |

**R8's point estimate: +14 tests.** TARGET count predicted **38 → 38 or 39**.

### 3b. The price prediction (blind on the ladder, sighted on the obj)

| # | claim | P |
|---|---|---|
| P1 | the re-derived price for this TU is **≥ 12 independent facts** | 0.85 |
| P2 | the re-derived price is **≥ 20 independent facts** | 0.55 |
| P3 | the price is **≤ 6** (i.e. board #344's "one body-class question away" survives) | 0.03 |
| P4 | at least one of the five functions is priced at **≥ 6 facts on its own** | 0.80 |
| P5 | #1792's `LIFTED→LIMIT` for this TU is the **TU's** limit, not the instrument's — i.e. the ladder's exit is a real refusal with decode distance, not a `noform` rename | 0.45 |
| P6 | the whole-TU `functions() == None` (`vocab-gap`) is a **separate, additional** blocker on top of the four per-function ones | 0.70 |

**No discount factors are applied anywhere.** Where a blocker class already has
an emitter the ceiling IS the estimate. Facts are counted by the test *"what
varies between these two? if nothing, it is one refusal."*

## 4. Standing refusal clause

If the price lands at **≥ 4 independent facts** for the TU, the standing decline
clause applies and this lane **ships the price, not a conversion**. A
manufactured conversion — widening an emitter past what the obj licenses so one
TU goes green — is the failure this lane is explicitly told not to produce.

## 5. Fences

- No wrong emit. A refusal becoming a wrong emit is strictly worse than a gap.
  Every admission fenced; `mismatch` 0 is the alarm that outranks the mission.
- Call-bearing admissions checked against the obj's **own** relocation/call-edge
  count, never assumed. `Encrypt` has 3 relocations in `.text` and one in
  `.pdata`; `SetKey` has 1. An encryption routine is where intrinsic expansion
  and dead-call elision live.
- Seams: my own new shape module(s) under `crates/c2-il/src/func/body/shapes/`,
  my own codegen module(s) under `crates/c2-core/src/codegen/`, my fixtures, my
  rung. **`coff.rs` is w-ifn's and is not touched.** No shared predicate
  narrowed or shadowed.
- Label charges are **never** taken from `docs/LABEL_COUNTER.md`; measured by
  counterfactual over TUs one body apart, at `/O1` **and** `/Ox`.
