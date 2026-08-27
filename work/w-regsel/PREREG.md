# PREREG — `w-regsel`, the selector as a settable module

    Lane:      w-regsel
    Kind:      construct rung
    Date:      2026-08-27
    Base:      master `42f76b849` (decision 20's own commit)
    Board:     #3694–#3699 (mine, and only these)
    Wave:      16 — `docs/DECISIONS_2026-08-22.md` § Decision 20
    Brief:     `docs/REGALLOC_BRIEF_2026-08-27.md` §3 L1

**Frozen before the first edit to `crates/` and before the first measurement.**
This file is the first commit on `wt-w-regsel`.

---

## 1. The deliverable, stated so it can be graded

`codegen::regalloc::select` — c2's selector at **`0x10b2e7f8`**, expressed as
executable code: the minimum-cost walk over a per-class **ordered** register
list, strict `<`, so a tie goes to the earliest entry of the order. The order
is a **named, settable parameter** (decision 15 / `rungs/README.md`'s
decision-surface clause), not a baked constant, and its **default reproduces c2
byte-exactly**.

Graded as a **construct rung**: `Fixtures: none`, `Census: +0`,
**required-zero byte delta**, identity diff over the 21 gate rows
(`gate_identity_diff.sh`, board `#290`'s pattern, `#3579`'s instrument).

**The re-expression** — a construct rung is graded by re-expressing an
**already byte-exact** class through the new machinery. The class chosen is the
store run's producer allocation, `codegen::alloc::allocate`, whose pool walk

```rust
.map(|(i, p)| (p.id, POOL_TOP - i as u8))          // POOL_TOP = 11, board #543
```

is *hypothesised* (P1 below) to be exactly the **zero-cost case** of the
selector over the read GPR order.

## 2. The caveat carried IN, not discovered

`P_REGALLOC.md` §3's correction box: on all 10 cells of `wb-live`'s grid and
all 15 of `wb-regalloc`'s, **every cost array is uniformly zero over its
allowed set**, so the answer is decided entirely by list order. Therefore:

* the **cost arithmetic** this module implements is `[R]` and **stays `[R]`**;
* what is made executable and testable is the **ORDER**, which is `[O]`
  (cells G1–G4, P1, `WB_REGALLOC_FINDINGS.md` §7.1, 6/6);
* **this lane will not report "the cost model is confirmed"** in any form. If
  the rung says that, the rung is wrong and this paragraph is the receipt.

## 3. Predictions, with confidence, frozen

| # | prediction | conf. | discriminator |
|---|---|---:|---|
| **P1** | `alloc::allocate`'s descending pool walk is **exactly** the zero-cost selector over `0x10c37de0`'s order restricted to the run's allowed set, on **every** input in the enumerated `(n, pool_floor)` grid — same registers, same refusals | 0.90 | exhaustive equality of the assignment vector AND of the refusal predicate over `pool_floor ∈ 0..=20`, `n ∈ 0..=4`, all producer kind/use-count shapes the module admits. **One disagreeing cell refutes it** |
| **P2** | Board **`#543`** — *"`r12` is never used — recorded, not explained"* — is **EXPLAINED by the read**: c2 index `13` (= `r12`) appears in **none** of the three GPR order arrays, so `r12` is excluded by the ORDER, not by a cap | 0.85 | enumerate `0x10c37de0`/`0x10c37e50`/`0x10c37eb8` as transcribed in `WB_REGALLOC_FINDINGS.md` §3.1. If index `13` appears anywhere, refuted |
| **P3** | Required-zero byte delta holds: identity diff **0 lines over 21 rows**, `mismatch 0` at both ends | 0.95 | `scripts/gate_identity_diff.sh BASE.txt TIP.txt` |
| **P4** | The re-expression needs **no widening of any refusal**. `POOL_TOP`'s cap can be **removed** and replaced by the order's own exclusion of `r12` with the refusal domain unchanged | 0.75 | the refusal-domain table of P1. If a refusal moves, the lane **reverts the cap removal** and says so — it does not widen an emit to make a module look general |
| **P5** | No production call site supplies a **non-zero** cost. The only cost array the port constructs is `Costs::ZERO` | 0.95 | a `#[cfg(test)]` enumeration of every `select`/`select_sequence` call outside tests, asserting the cost argument is the zero array |
| **P6** | Predicted reach **0**, census **+0**, `match` unchanged | 0.97 | the identity diff and `c2rs census` |

**P4 is the one I expect to be wrong**, and it is registered at 0.75 rather
than 0.9 for that reason.

## 4. The fail axis — `#3336`, named before starting

A required-zero **byte** delta is silent about everything that is not a byte.
The axis on which this rung **can** fail with every byte identical:

> **THE REFUSAL DOMAIN.** The exact set of `(producers, pool_floor)` on which
> `alloc::allocate` returns `None`. Today's fixtures reach a *narrow* band of
> `pool_floor`, so a re-expression could widen or narrow the refusal outside
> that band and **no gate row, no census count and no byte would move**. It is
> enumerable, so it is measured: the full grid of §3 P1 is tabulated at base
> and at tip and required to be **line-for-line identical**.

Second axis, weaker but real: **the register order's tail**. The read order
continues `r3 → r31 → r30 → … → r14` past the volatiles. That tail is
**unreachable** from every production call site (the allowed set is capped at
the volatiles) and must **stay** unreachable — a rung that quietly made it
reachable would widen the port's emit with no fixture to catch it. Measured by
a test asserting the selected register is always a volatile on the whole grid.

## 5. Controls — `#3336`, "a control you have never watched FAIL is decoration"

Each control below is **planted, watched red, and recorded** with the failure
text before the defect is reverted. A control not recorded as having gone red
does not ship.

| C | planted defect | must go red in |
|---|---|---|
| **C1** | reverse `GPR_DEFAULT_ORDER` | the P1 equivalence test |
| **C2** | insert `r12` into `GPR_DEFAULT_ORDER` at the head | the `#543` test and the equivalence test |
| **C3** | strict `<` → `<=` in the selector's walk | the tie test (zero costs ⇒ the LAST allowed entry wins instead of the first) |
| **C4** | the sequential walk stops removing the chosen register | the "distinct registers" test |
| **C5** | drop the "allowed set" check from the walk | the refusal-domain table |

## 6. What this lane will NOT do

* **No full register allocator.** Decision 20 §2 — F5 is not separable from F0,
  the port schedules nothing. If the work starts needing live-range versions or
  the backward walk over the lowered tuple list, the lane **stops and reports
  that as the finding**.
* **No `ported` numerator for regalloc.** A site-level numerator is NOT YET
  DEFINED for this subsystem; constructing a denominator to move a percentage
  is `#3505`, four for four.
* **No new count-bearing `gate.sh` row.** A 22nd row makes
  `gate_identity_diff.sh` exit 2, refusing to diff at all, for every live lane
  (`#3691`).
* **No re-taking of `#3534`.**
* **No claim about the cost model.** §2.

## 7. What would make this lane say `FAILED`

Any one of:

* P1 refuted and the divergence is **not** explainable as a read error on my
  side — i.e. the port's fitted walk and c2's read order genuinely disagree
  somewhere reachable;
* a non-zero byte delta that cannot be reverted;
* the deliverable not landing as a settable parameter (a baked constant with a
  new name is not a parameter);
* the controls not watched red.

## 8. Provenance

The order arrays are **copied numbers**, so `WB_REGALLOC_FINDINGS.md`'s
adoption-ready row **`W-REGALLOC-1`** — *"carry this row only if the numbers or
the variant arrays are copied"* — becomes due **in the same commit** that puts
them in `crates/`, with a `PROV[R]`/`PROV[O]` marker at the site and the
citation written qualified (`DISCLOSURE W-REGSEL-1`, never bare).
