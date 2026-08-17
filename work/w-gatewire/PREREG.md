# PREREG — lane `w-gatewire`

Frozen at base `7e541a54` (master), before any probe, build, or measurement.
No discount factor. Every row is a claim I can be wrong about, in the units the
deliverable is written in.

**Mission (a decision the user has made):** wire `scripts/debug_lane.sh` into
`scripts/gate.sh` as a proper gate row, so a debug-profile panic fails the gate
and therefore blocks a merge. Preserve clean degradation. Re-measure the cost.
Prove the row can go red.

**Kind:** construct rung. `Fixtures: none`. `Census: +0`. Required-zero byte
delta. `scripts/` is in `GRADED_DIRS`, so the graded tree hash MOVES by
construction and that is expected, not a defect (#3215 applies to
revert-everything lanes only).

---

## What I already knew at freeze time (declared, not predicted)

Read before this file was written; recording it so the prereg cannot be
credited with discovering it later.

* **F1. The brief's "0.65 s warm" is mis-attributed.**
  `docs/rungs/2026-08-14-dbgassert.md` prices two candidates, not one:
  * **(b)** a *debug unit row* — `cargo test --workspace --lib`, no toolchain —
    at **6 s cold / 0.65 s warm**, 1,386 tests;
  * **(c)** `scripts/debug_lane.sh`, 18 lanes, at **125 s warm**, needs the
    toolchain.
  0.65 s is **(b)**. The thing I am wiring is **(c)**, and its recorded price is
  **~190x** the figure in my brief. I am wiring (c) anyway — the user approved
  `debug_lane.sh` by name and the decision is a *policy* decision, not a cost
  decision — but the correction is a first-class deliverable of this lane, and
  the re-measurement below is against 125 s, not 0.65 s.
* **F2. The row is toolchain-BOUND.** Unlike `hatch-red` / `ladder-red` (which
  need no toolchain and run on the portable lane), `debug_lane.sh` drives
  `c2rs gap` against real c2. So it belongs with `expr-sweep` / `mode-cross`
  under the skip rules, not with the instrument rows.
* **F3. `debug_lane.sh` reads `scripts/lanes.txt` at a hardcoded path** and has
  no `C2RS_LANES` override, unlike `gate.sh`.

## Predictions

### The row itself

| # | prediction | P |
|---|---|---|
| P1 | The row can be made to **SKIP, not FAIL**, with the toolchain hidden, and `GATE: SKIPPED` still exits 0 (and exits 1 under `--require-graded`). | 0.90 |
| P2 | Verified **live** by hiding the toolchain via an env override, not by reading code. | 0.95 |
| P3 | A deliberate debug-only panic (integer overflow or `debug_assert!`) injected in a scratch commit makes the gate print **`GATE: FAIL`** with a message that **names the lane** whose debug run panicked. | 0.90 |
| P4 | The same injection leaves the **release** gate rows green — i.e. the row catches a fault every other row is structurally blind to, demonstrated rather than asserted. | 0.85 |
| P5 | Outcome word: **`built`**. | 0.85 |

### Cost (the figure I owe the user)

Measured on this box, in this worktree, warm capture cache. The gate's own
default is `--jobs 16`; the lane brief's verification command is `--jobs 4`, so
I register both.

| # | prediction | interval | P |
|---|---|---|---|
| P6 | `debug_lane.sh` alone, **warm**, 18 lanes, `C2RS_JOBS=16` | **60–110 s** (point 80 s) | 0.70 |
| P7 | `debug_lane.sh` alone, **cold** (no `target/debug`) — the delta over warm is a from-scratch debug build of 5 crates with zero external deps | **+5–25 s** (point +10 s) | 0.75 |
| P8 | Gate total **before**, `--jobs 4 --require-graded` | **270–360 s** | 0.70 |
| P9 | Gate total **after**, `--jobs 4 --require-graded` | **480–700 s** (point 560 s) | 0.65 |
| P10 | The added row is the **single largest** line item in the gate at `--jobs 16`, larger than the sweep's 82 s | — | 0.45 |

### Non-perturbation (the discriminating evidence)

| # | prediction | P |
|---|---|---|
| P11 | The gate's **per-lane counts are identical** before and after — all 18 lanes, `graded/total/match/mismatch` digit for digit. The row is additive. | 0.97 |
| P12 | **Release-binary sha256 identical** before and after (I touch no `crates/`). | 0.90 |
| P13 | **878-TU scan identity diff: 0 deltas over all 394 anchored keys.** | 0.97 |
| P14 | `cargo test --workspace --release --no-fail-fast` stays **1,648 passed / 0 failed / 42 targets**. | 0.95 |
| P15 | `scripts/board_audit.sh` all-zero; `rung_registry` 2/2. | 0.95 |
| P16 | **No `crates/` change is needed.** If one appears necessary, that is a finding and I stop. | 0.90 |

### Shape of the work

| # | prediction | P |
|---|---|---|
| P17 | I will have to teach `debug_lane.sh` to honour `C2RS_LANES` (F3), so a `--lane`-filtered gate run filters this row too and the row's declared-lane count matches the registry the gate walked. | 0.80 |
| P18 | New `--selftest` cases added: **+18**, interval **12–26**. Every existing `decide` call site (12 of them) needs a 9th argument. | 0.60 |
| P19 | The row will **not** be folded into `--require-graded`'s unit sum — it re-grades the same fixtures the release lanes already counted, so folding it in would double-count. | 0.85 |
| P20 | `debug_lane.sh` finds a **new** live defect (panic or overflow) at `7e541a54` that master does not know about. | 0.15 |
| P21 | All 18 debug lanes are clean at `7e541a54` (0 panics, 0 mismatch, per-lane `match` equal to the release lanes'). | 0.85 |
| P22 | I add **no** sampling knob (`--debug-lanes N`) and **no** opt-out flag: the house rule is that an omittable check is an omitted check. | 0.80 |
| P23 | Board rows: at least one of #3218's neighbours is already spent by a live peer, so I mint **0 new rows** and leave mine UNNUMBERED. | 0.35 |

## What would make this lane FAILED

* The row cannot be made to SKIP cleanly (P1 wrong) — then wiring it breaks the
  portable lane and CLAUDE.md's degrade-cleanly rule, and the honest report is
  that the wiring is refused with a reason.
* The injected panic does not redden the gate (P3 wrong) — a row that cannot go
  red is worse than no row, and shipping it would be this repo's defining
  defect family with a new face.
* Any perturbation of the existing 18 lanes' counts (P11 wrong).
