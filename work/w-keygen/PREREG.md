# PREREG — lane `w-keygen` (board task #219)

**Frozen 2026-08-13, before the first hypothesis probe.** Committed on branch
`wt-w-keygen` before any `crates/` edit, any fixture, any `/Ob0` compilation and
any re-run of `work/w-inline/grade_pair.py`.

## 0. What was ALREADY MEASURED before this file was frozen, and why that is not a probe

Deliverable 1 of this lane's brief is *"re-derive `keygen_xbox.cpp`'s price from
the object code"*. That is a **read of the tree's own standing instruments plus
one reference compilation**, not a test of a hypothesis this lane holds. It was
done first, deliberately, because a PREREG written without it would be
predicting against inherited numbers the brief already flags as underived.

Declared in full, so nothing below can be mistaken for a prediction:

| already read | value | instrument |
|---|---|---|
| `c2rs gap` on the 1-TU list | frontier, `19 \| 20`, `8/1440` bytes accepted | `gap --list work/w-keygen/one.txt` |
| CODEGEN column for this TU | `den 20 · exact 1 · wrong 0 · cg-ref 0 · reader 19 · ungraded 0` | same scan |
| per-function census | 1/20 in class; 11 distinct first-blocker productions | `c2rs census` |
| the `.cod` at workload flags | 20 procs, 360 words, per-proc `bl` targets | `cl /FAsc` under wibo |

**Two of those already contradict published figures** and are recorded here as
*corrections*, not predictions: the brief's inherited CODEGEN reading (*"21 of
its 20 emitted functions are behind the reader (91%), 0 measurable, 2 already
byte-exact"*) reads **19 of 20 (95%), 0 measurable, 1 byte-exact** at HEAD.

**Question (c) is RETRODICTED, not predicted.** The `.cod` call-edge count was
taken before this file existed, so this lane's answer to *"is the 14-word
`?shuffle2` gap an inline decision"* is a **retrodiction** and is scored as one.
No credit is claimed for it as a forecast.

## 1. The registered PRICE prediction (deliverable 1)

`CEILING.md` §6.1 and board **#1792** both price this TU at **"≥ 36"**, and #1792
admits no row in it was compiled or disassembled.

> **P1.** Counting **independent** refusals — *"what varies between these? if
> nothing, it is one refusal"* — the honest re-derived price of
> `keygen_xbox.cpp` is in **[18, 30]**. **p = 0.70**
>
> **P1a.** The re-derived count is **strictly less than 36**. **p = 0.75**
>
> **P1b.** At least **one** refusal class appears in the re-derived price that
> is named in **neither** `CEILING.md` §6.1 nor board #1792. **p = 0.80**

## 2. The registered INLINER predictions (deliverable 2)

> **P2 (question a).** Re-running `work/w-inline/grade_pair.py` on the frozen
> 100-TU `sample_b.txt` hold-out at today's tree reproduces mechanism I's
> accuracy **within ±0.010 of 0.9716**. **p = 0.80**
> *(The instrument is obj-side only and reads nothing from `crates/`, so the
> only way it can move is if the workload sources moved.)*
>
> **P2a.** The graded callee population is within ±5 % of the published
> **9,993**. **p = 0.75**
>
> **P3 (question b).** Shipping mechanism I's decision rule **fenced** — admit a
> caller only where `INLINE-P` is categorical, never guess an inline — moves
> **`match` by 0**. **p = 0.95**
>
> **P3a.** The same, fenced, moves **`fnbyte-exact` by 0**, because the fence's
> own precondition (the port must be able to lower the callee to know `s`) is
> exactly what today's reader denies on the population the fence would reach.
> **p = 0.70**
>
> **P3b.** There exists at least one measured case in this workload where
> `INLINE-P`'s **false-decline** direction (predicts DECLINED, c2 INLINED) would
> put a **wrong emit** on the accept side of any decline-side fence.
> **p = 0.85**

## 3. The registered DELTAS (the scored quantities — CEILING §10)

Census-only predictions are UNSCORED. These are the scored ones.

### `match` DELTA (878-TU scan, base **25**)

| outcome | p |
|---|---:|
| **+0** | **0.94** |
| +1 (`keygen_xbox.cpp` converts) | 0.02 |
| +1 (some *other* TU converts as a side effect of a class-wide widening) | 0.04 |
| any negative | 0.00 — a regression is an alarm, not an outcome |

### `fnbyte-exact` DELTA (878-TU scan)

This lane may ship at most the cheap reader widenings its price derivation
isolates (`expr-op-0x0F`, `expr-op-0x30`, `expr-lit-type-9641` are the three
candidates at ≤ 6 emitted words). **Calibration taken from `w-vsnprnc`: a
widening reaches a CLASS, not a TU** — it predicted +2 and got +17. The TU's own
blocker and the class's reach are priced separately, as the brief requires:
this TU's own take from any such widening is **at most +3 functions** (`?roll`,
`?swap`, `?opaquePredicate`), and everything above that is class reach.

| outcome | p |
|---|---:|
| **+0 (nothing ships — the priced decline)** | **0.45** |
| +1 … +50 | 0.20 |
| +51 … +500 | 0.20 |
| > +500 | 0.15 |

**`fnbyte-differs` must not increase.** Any increase is a regression to
investigate, not a result. **p(no increase) = 0.99**

### TEST-COUNT DELTA (`cargo test --workspace --release`)

Base measured on **this lane's own tree before any edit** (recorded in §5 of the
rung, never inherited).

| outcome | p |
|---|---:|
| +0 (nothing ships) | 0.40 |
| +1 … +10 | 0.35 |
| +11 … +30 | 0.20 |
| > +30 | 0.05 |

**TARGET count must not shrink.** A shrunken target count is an earlier target
failing to build, not a partial run. **p(targets ≥ base) = 0.99**

## 4. Standing constraints this lane accepts in advance

* **`mismatch` stays 0** on the 878-TU scan and everywhere in `gate.sh`. An
  increase voids the lane.
* **A refusal becoming a wrong emit is strictly worse than a gap.** If any
  candidate widening cannot be graded N/N by real `c2` on a fixture whose
  structural axes are varied (pointee type, arity, operand position, count),
  it does **not** ship.
* **No shared predicate is narrowed or shadowed.** `FnByte::Exact`,
  `chain_skip_form`, the inline fence in `IlBundle::functions`, census
  acceptance vocabulary and the scan keys are **wideners only**, and every one
  touched is named in the report.
* **Peer-owned, not touched:** `crates/c2-core/src/codegen/coff.rs`,
  `codegen/labels.rs`, `fixtures/cpp/*wordwrap*`.
* **Board #2343 trap accepted in advance:** any ladder depth read past a
  floating-point literal is a corrupted stream (`noform-0xNN` is
  indistinguishable from an unpinned opcode). This TU contains **no** FP
  literal — checked against the source before this file was frozen — so no
  depth read here is exposed to it. If that check is wrong, every depth number
  in the rung is void.
* **A priced decline is a successful rung.** This lane will not manufacture a
  conversion.
