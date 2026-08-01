# W-SLOTARG — pre-registration (board #149/#150)

Written **before** the capture grid was generated or any code was changed, and
committed before the first `c2rs listing` run over it. Base `74d0744`
(worktree branched from a stale `4ea415a`, **609 commits behind**, and was reset
to `master` before any work — the fifth lane this week to meet that).

## What was already measured before this file was written, and is disclosed

Two things are *inputs* to the registration rather than predictions, so they are
stated here rather than scored:

1. **The 356 re-measured at this base.** Board #149's quantity does not age:
   878-TU dc3 scan, base emitted **36,059 / 178,968 (20.15 %)**, under
   `C2RS_SINK_OFF_ADD_ARG=ceiling` **36,415 / 178,968 (20.35 %)**,
   **Δ = +356**, identical to §9.17.5. Both runs read 6 match / 0 mismatch /
   census-gate disagreement 0, reproducing §9.17.7's blind spot.
2. **The `Mismatch @ offset 8` diagnosis**, and the three-cell probe
   (`work/probe_offadd.cpp`) whose listing reproduces §9.17.5's `?a1`/`?a3`
   lowering exactly. See §D below — it is a *finding*, not a prediction, because
   it was run to decide what to build.

## The structural constraint, stated up front

Board #149's rung needs a `SlotArg` variant. **`SlotArg` is declared in
`crates/c2-il`** (`func/mod.rs:313`, the resolved twin `func/body/mod.rs:278`),
and this lane is instructed to stay out of `crates/c2-il` — lane `w-emitset` is
live there. So the shipped port's Δ emitted from this lane is **0 by
construction**, not by measurement, and registering a Δ would be dishonest.

What *is* in `crates/c2-core` is the half board #149 calls the risky half: "a
`SlotArg` variant for `base + k` **and its position in the permutation walk**".
§9.13.1's ALARM is that the position rule is exactly where a wrong lowering
ships green. So this lane registers against **the rule**, and the deliverable is
the rule measured, pinned portably, and wired to a c2-core-local input type that
a five-line `c2-il` change later feeds.

## Predictions

### The capture grid (c2's own listing, non-perturbing)

Grid = (designator steps 1,2) × (offset 0, 8, 0x8000, 0x10000) ×
(arity 1..5 × address slot) × (free caller, member caller) — **crossed, not
sampled**, per board #149 and §9.13.1's third consequence.

| | registered | interval |
|---|---|---|
| **G1** | c2 emits every grid cell | ≥ 200 of 240 |
| **G2** | the computed-address `addi` has **no single fixed position**; ≥ 2 distinct schedule shapes across the grid | 4 shapes, [2, 8] |
| **G3** | ≥ 1 cell **pre-saves the base** (`mr r11,rB`) before the walk, because unlike `SymAddr` the address READS a formal that the walk may clobber | 30 of 240, [10, 150] |
| **G4** | at k ≥ 0x8000 c2 does **not** emit one `addi` (the immediate is signed 16-bit) | YES |
| **G5** | at k = 0 with the base already in the destination slot, c2 emits **nothing** for the address | YES |
| **G6** | at walk length ≥ 2 the address's position **differs** between the address-at-slot-0 and address-at-last arrangements | YES |

**G2 is the one that matters.** `SymAddr`'s rule (§9.13.1) is "second, after
exactly one word of the descending walk", and `sym_slots_text` refuses the
shifting-formal case outright. The off-add's address has a *register input*, so
it is both a source and a destination of the permutation and cannot be scheduled
independently of it. Registering "same rule as `SymAddr`" at **25 %**.

### The rung

| | registered | interval |
|---|---|---|
| **S1** | shipped-port Δ emitted **from this lane** | **0** — a constraint (see above), not a measurement |
| **S2** | a naive **address-last** wiring (WR1's superseded rule) would mis-emit ≥ 50 % of the grid cells that have a walk | 65 %, [20 %, 90 %] |
| **S3** | portable assertions added to `crates/c2-core` | 5, [3, 10] |
| **S4** | the **control stays green**: every existing `SymAddr`/`Lit`/`Formal` ordering test passes unchanged under every mutation this lane makes | PASS |
| **S5** | tip workload scan **identical to base** on every headline number (the addition is unreachable until `c2-il` supplies the variant) | identical |
| **S6** | `gate.sh`, `selftest`, `expr_sweep`, `cross_sweep` all unchanged from base | unchanged |
| **S7** | the verdict | **partial: rule shipped, conversion DECLINED** |

### The controls that could fire

* **S4 is the §9.12 pin.** A mutation that reds every cell identifies nothing;
  the `SymAddr` and pure-literal cells must stay green while the off-add cells
  move, or the grid is not discriminating.
* **S2 is scored against a real alternative**, not against nothing: the grid is
  replayed through both candidate rules and the disagreement counted, so
  "address-last is wrong" is a number rather than an assertion.
* **G1 is the totality control** — a grid cell c2 declines to emit is a cell
  that proves nothing, and #144 says residue 0 is not itself a control, so the
  per-cell schedules are compared to each other rather than merely counted.
* **The 878-TU differential CANNOT grade this** (#149, §9.17.7) and is therefore
  registered as S5, an **inertness** control, never as evidence of correctness.

## Decline rule, registered in advance

If the measured rule cannot be stated as a total function over the grid — i.e.
if any grid cell's schedule is not derivable from (walk length, address slot,
offset, base-clobbered) — the rule is **refused, not fitted**, and this lane
reports the stock unconverted. A rule fitted to 240 cells with a free parameter
per cell is §9.14.7's disease.
