# w-deaccept — PREREG

Frozen and committed as this lane's **first commit**, before the first byte of
`crates/` changed and before the first sink/widening measurement was run. The
only thing measured before this file was frozen is the **base scan** itself
(no `crates/` delta, no env var) and a read of `base.jsonl`'s
`emit_blockers` / `fn_blockers` histograms — recorded in §0 so that what I knew
when I registered is on the record.

    Lane:   w-deaccept
    Base:   master a238180b, worktree branch wt-w-deaccept
    Kind:   construct rung (required-zero byte delta)
    Date:   2026-08-14

---

## 0. What was known at freezing time

From `work/w-deaccept/base-gap.log` + `base.jsonl` (my own base, re-measured,
**372** `gap-metric` keys — not w-readphase's 370; master advanced):

| | value |
|---|---:|
| `match` · `mismatch` · `codegen-gap` · `port-error` | **25 · 0 · 0 · 0** |
| `vocab-gap` · `capture-fail` · `frontier` | **845 · 8 · 2** |
| `gap-metric` keys · jsonl rows | **372 · 878** |
| `fnbyte-exact` / `fnbyte-denominator` | **35,734 / 162,049** |
| `fnbyte-refused-parse` / `-codegen` | **113,612 / 949** |
| `emit_blockers` | 615 keys summing 113,612 |
| `fn_blockers` | 635 keys summing 1,705,627 |

And the fact that reframes this lane, read out of the base histograms before
anything was registered:

> **At base, `expr-op-0x5D` and `expr-op-0x5E` do not exist as keys — in either
> histogram.** The only `5D`/`5E` row anywhere is **`body-0x5D` = 8** emitted
> functions (8 in `fn_blockers` too), which is the **statement**-position
> dispatch in `body/mod.rs`, not `parse_expr`. w-readphase's **13,158** is a
> number measured **at the sink ceiling**, with all 47 pinned opcodes + `type` +
> `convert` + `intrinsic` granted. It is not a count of anything reachable at
> base.

Prior art found and read before freezing (this is the "check the crate's own
constants" step, and it changed the design):

* **`shapes/control_flow.rs:855`** already has a `0x5D | 0x5E` arm reading
  `<varint> <varint>` — `cf-eh-count` / `cf-eh-count-state` — with the
  `eh_live`/`eh.count` bookkeeping. **A reader of this shape already exists**;
  any new one must not redefine it.
* **`docs/whitebox/WB_READER_FINDINGS.md` §3.1/§3.4** puts `5D 5E` in c2's own
  reader **class 14** and lists `0x5D`/`0x5E <i32c> <i32c>` among the
  *agreements* between c2's disassembly and this tree. The width is not in doubt
  from any direction.
* **`WB_EH_FINDINGS.md` §6 / board #2263, #2976**: the operand-position clause is
  already named — **R16 (`op-0x5E`)** — and recorded as **UNPAID**.
* `parse_expr` is called with `stop` ∈ {`0x41`, `0x32`, `0x55`} at four sites
  (`mod.rs:2815`, `assign.rs:150`, `assign.rs:214`, `calls.rs:1255`). **Never
  `0x5D`/`0x5E`.**

---

## 1. What is being built

Two arms, deliberately separable, so that "poisoned" and "real" are the same
token in the same tree:

* **(I) the instrument arm** — `SkipForm::VarintVarint` + a
  `0x5D | 0x5E => VarintVarint` row in `chain_skip_form`. Env-gated
  (`C2RS_SINK_CHAIN`), poisoned, off on every default scan and every gate lane.
  This is w-readphase's found-and-not-taken #1 exactly as it described it.
* **(R) the real arm** — an unpoisoned `0x5D`/`0x5E` arm in
  `parse_expr_classed` consuming `<varint> <varint>`, pushing no `IlOp`, setting
  no poison flag, on by default with no env var. This is the widening.
* **(R2) contingent** — a statement-position arm in `body/mod.rs` for the
  `body-0x5D` population (8 emitted functions), built **only if R's measured
  reach is 0** and only as a scratch measurement. Registered here so that
  building it later is not an unregistered extension.

---

## 2. Registered predictions

Probabilities are my honest credences. Every count is a **ceiling with no
discount factor**.

### The central claim of the lane

| id | registered | p |
|---|---|---:|
| **C1** | **w-readphase's poisoned-sink numbers do NOT reproduce on a real widening of `5D`/`5E`** — because the two are not comparable populations, not because poisoning is benign. Specifically: the real arm's Δ`match` and Δ`fnbyte-exact` are both **0**, while the poisoned `op:41` run reproduces its published −1 / −2,694 | **0.70** |
| **C2** | The reason C1 holds is **reach, not neutrality**: the real arm is reached by **0** emitted functions at base, so it can neither gain nor de-accept. I.e. this lane's answer to "artifact or real" is **"neither — untested, because the vehicle has no reach at base"** unless R2 is built | **0.65** |
| **C3** | Sinking `op:5D,op:5E` (arm I) at base moves `match` by **0** and `fnbyte-exact` by **0** — the poison does NOT de-accept on this token, unlike `op:41`. The mechanism behind `op:41`'s −2,694 is that `41` is `parse_expr`'s own **stop byte**, and `5D`/`5E` are not a stop byte at any of the four call sites | **0.80** |

### The real arm (R) — the required-zero columns

| id | registered | p |
|---|---|---:|
| **R-a** | Δ`match` = **0** (25 → 25) | 0.88 |
| **R-b** | Δ`mismatch` = **0** (stays 0) | 0.97 |
| **R-c** | Δ`fnbyte-exact` = **0** (35,734 → 35,734) | 0.80 |
| **R-d** | Δ`fnbyte-refused-parse` = **0** (113,612 → 113,612) | 0.75 |
| **R-e** | **de-acceptance count = 0 functions** — no function that is `FnByte::Exact` at base is non-exact after | 0.85 |
| **R-f** | The identity diff is **0 of 372 `gap-metric` keys differing** and **0 of 878 verdict lines differing** | 0.72 |
| **R-g** | R is therefore **shippable** (neutral, not negative) | 0.78 |

### Reproduction of w-readphase (arm I, and the control)

| id | registered | p |
|---|---|---:|
| **W-a** | `C2RS_SINK_CHAIN=op:41` at my base reproduces `match` 25 → **24** | 0.85 |
| **W-b** | …and `fnbyte-exact` 35,734 → **33,040** (−2,694) exactly | 0.60 |
| **W-c** | Of the functions `op:41` de-accepts, **the majority land on a NON-poison key** (`expr-jump` and friends) rather than on `expr-chain-sink-poison` — i.e. #3094's mechanism is **pre-emption**, which a real widening shares, and not the poison flag, which it does not | 0.55 |

### The ceiling claim (arm I at the ceiling)

| id | registered | p |
|---|---|---:|
| **K-a** | Adding `op:5D,op:5E` to w-readphase's ceiling spec removes the `expr-op-0x5D` (6,815) + `expr-op-0x5E` (6,343) = **13,158** residue rows | 0.85 |
| **K-b** | …and the function-tail reach rises by **≤ 13,158** and **> 0**. Ceiling with no discount: **+13,158**; my point estimate is that it comes in **below** it, because a body cleared past a `5D` lands on the next residue item | 0.80 |
| **K-c** | The ceiling run's `match`/`fnbyte-exact` are **worse than base** (the whole ceiling is poisoned), so no required-zero column can be read off it | 0.95 |

### Tests

| id | registered | p |
|---|---|---:|
| **T-a** | Base `cargo test --workspace --release --no-fail-fast` = **1,567 passed / 0 failed / 42 targets** | 0.90 |
| **T-b** | Test-count delta = **+4** (target **1,571 / 42**): one for the new `SkipForm` width, one for the real arm's acceptance, one for the arm NOT firing on a truncated stream, one updating the existing `the_unpinned_opcodes_refuse_rather_than_guess_a_width` assertion (which asserts `chain_skip_form(0x5D) == None` today and must be **moved**, not deleted) | 0.55 |
| **T-c** | No target count change (42) | 0.95 |

### The mutation control

| id | registered | p |
|---|---|---:|
| **M-a** | A mutant that makes the real arm read **one** varint instead of two goes **red** on at least one test built from real IL bytes | 0.85 |
| **M-b** | A mutant that makes the real arm consume `5D`/`5E` **unconditionally without reading the trailing fields** also goes red | 0.80 |

---

## 3. The decision rule, fixed in advance

* `mismatch` ≠ 0 at any point ⇒ **revert immediately**, report it as the
  headline, `Outcome: declined` or `FAILED`. Non-negotiable.
* Δ`match` < 0 **or** Δ`fnbyte-exact` < 0 ⇒ **do not ship R**. Quantify the
  de-acceptance, name the mechanism, revert to a zero delta, land the
  measurement as the deliverable.
* Δ`match` ≥ 0 and Δ`fnbyte-exact` ≥ 0 and `mismatch` == 0 ⇒ ship R, and say so:
  the reader's negative price is then **not** a property of this real widening.
* Arm I ships **regardless** of R's outcome if and only if its own delta is
  exactly zero on all four columns with the sink off — it is env-gated
  instrumentation and its zero-delta is checkable independently.
* **A wrong emit is strictly worse than a gap.** No column is worth a
  `mismatch`.

---

## 4. What this lane will NOT claim

* It will not claim the reader verdict is overturned on the strength of a
  zero-reach widening. If R's reach is 0, the honest statement is that this
  vehicle **does not test** the mechanism, and that is the finding.
* It will not read `match`/`fnbyte-exact` off any run with a sink token set.
* It will not treat `13,158` as a base-reachable number anywhere in the write-up.

---

## 5. ADDENDUM — stage-2 prereg, frozen after stage 1 and before its own measurement

Stage 1 (the `op:41` control, no `crates/` change) is measured and scored in §6
below. It came in **exactly** as w-readphase published it and the decomposition
came in on the informative side of W-c: **all 2,694 de-accepted functions land on
`expr-jump`, a real refusal key, and ZERO land on `expr-chain-sink-poison`.**

That produces a mechanism hypothesis §2 did not contain, and it is testable
without any `crates/` change, so it is registered here **before** it is run
rather than reported as if it had been predicted:

> **S1 — THE SINK DE-ACCEPTS IF AND ONLY IF THE SUNK TOKEN IS ONE OF
> `parse_expr`'s THREE STOP BYTES.** `chain_sink()` is consulted **before** the
> `b == stop` check (`expr.rs:1568`, deliberately, board #663), so sinking a stop
> byte makes every accepted walk run past the end of its own expression. The four
> `parse_expr` call sites use `stop` ∈ {`0x41`, `0x32`, `0x55`}. If the
> hypothesis holds, sinking a **non-stop** token de-accepts **0** and sinking a
> stop byte de-accepts a lot.

| id | registered | p |
|---|---|---:|
| **S1a** | `op:9B` (non-stop, `TypeTok`) — Δ`match` **0**, Δ`fnbyte-exact` **0** | 0.80 |
| **S1b** | `op:30` (non-stop, `Type`) — Δ`match` **0**, Δ`fnbyte-exact` **0** | 0.78 |
| **S1c** | `op:32` (**stop** at `assign.rs:150`) — Δ`fnbyte-exact` **< 0** | 0.70 |
| **S1d** | `op:55` (**stop** at `calls.rs:1255`) — Δ`fnbyte-exact` **< 0** | 0.65 |
| **S1e** | Every non-stop token in w-readphase's 9-token SCAFFOLD
  (`4F 53 54 4B 29 38 39 3A`) is individually neutral, so the SCAFFOLD's whole
  −2,694 / −1 is attributable to **`op:41` alone** | 0.60 |
| **S1f** | If S1 holds, then **#3094's "the poisoned sink is not emission-neutral"
  is true but its named mechanism is wrong**: it is not the poison flag and it is
  not a wider grammar pre-empting a recognizer — it is the sink's **stop-byte
  override**, a construct that exists only in the instrument and that **no real
  widening has**. That would make the reader's negative price an **artifact** | 0.60 |

**What S1 would NOT establish.** It says nothing about whether a real widening
with real reach de-accepts. That is still R's job, and R's reach at base is
predicted 0 (C2). If both land, the honest headline is *"the published negative
is an instrument artifact, and the real widening that would test the other half
has no reach at base"* — two findings, neither of them a licence.
