# `w-restim` PREREG — frozen before the first probe runs

    Lane:   w-restim (step-5 re-estimation; characterization)
    Base:   master 309c4c989
    Brief:  work/coordinator/restim-brief.md
    Date:   2026-08-21

Committed **before** any probe measurement. What had already run at the time
this file was written, and is therefore NOT covered by it:

* the two instrument fixes (`79e8bdfba`) and their 384-fixture re-run
  (`work/w-restim/snap_all_fixtures_verdicts.log`). Those are brief
  deliverable 1 — a fix and its demonstration, not a prediction. They are
  scored in the rung as measurements, never as hits.

Nothing below has been run. Every probe's site table, offsets and comparison
are named here first so a result cannot be re-described after the fact.

---

## 0. Invalidation rules (adopted verbatim from `w-stageoracle`'s PREREG)

1. **A workload-stamp inequality voids any workload-derived table**; the
   response is to re-read the base at the current stamp, never to explain the
   delta. This lane takes no workload scan unless the cost curve needs one.
2. **A number not derived from a log committed under `work/w-restim/` is not
   published.**
3. **Every foreign-memory read is fail-closed**: a pointer that fails the
   `plausible()` filter is printed as a refusal, never as a zero. An
   instrument that renders an unreadable field and a zero field identically
   cannot support the null it would then publish (this is exactly finding 1's
   defect in a new place).
4. **The obj is required-zero.** If `stage-tap-obj-differs` moves off 0 at
   the new site table, every probe result on this branch is void and the
   lane reports `FAILED`.

---

## 1. Probe A — the operand and candidate records

### What will be built

`tap_walk_tuples` currently emits one `TU` row per tuple, reading
`tuple+0x0 next`, `+0x4 opcode`, `+0x8 category`, `+0x9 flags`, `+0xa cc`.
Probe A adds a bounded walk of the two **operand lists** the tuple points to
and, from each operand, the **symbol / candidate record**:

| read | offset | source of the reading |
|---|---|---|
| tuple → operand list D | `tuple+0x28` | `FUN_10b2ceb7` walks `piVar13[10]`; `FUN_10b2e7f8`'s neighbour pass walks `piVar6[10]` |
| tuple → operand list S | `tuple+0x2c` | `FUN_10b2ceb7` walks `piVar13[0xb]`; `FUN_10bfebf7` walks `puVar1[0xb]` |
| operand → next | `op+0x0` | all three of the above |
| operand kind | `op+0x8` (byte) | `*(char *)(puVar3 + 2) == '\x01'` in `FUN_10bfebf7` |
| operand type word | `op+0xa` (u16) | class nibble is `(u16 at op+0xa) >> 12`, indexed into `DAT_10b022cc` |
| operand → symbol | `op+0x1c` | `piVar18[7]` / `puVar3[7]` |
| symbol kind | `sym+0x4` (byte) | `FUN_10b2ceb7`: `cVar1 = *(char *)(iVar9 + 4)`, values 1, 2, 3 |
| candidate id | `sym+0x1c` (u32) | kind-2 arm: `uVar15 = *(uint *)(iVar9 + 0x1c)`, fed to `FUN_10b2c21d` (candidate lookup by id) |
| assigned-register descriptor | `sym+0x10` → `+0x1c` | `FUN_10b31ac9`: `*(uint *)(*(int *)(param_3 + 0x10) + 0x1c)`; `P_REGALLOC.md` §4.1 `+0x10` = "assigned register descriptor" |
| physical-register descriptor | `sym+0x8` → `+0x1c` | `FUN_10b2ceb7` kind-1 arm: `*(uint *)(*(int *)(iVar9 + 8) + 0x1c)`, bounded `0x0f..0x20` by `FUN_10bfebf7` = r14…r31 in the `n = r+1` encoding |

All eight are `[R]`. Each gets a `DISCLOSURE.md` row in the commit that
adopts it.

### Predictions

| # | prediction | p |
|---|---|---:|
| **A1** | the operand-level pre/post-COLOR pair **DIFFERS** on ≥ 1 function of `il_call_perm.cpp` — i.e. COLOR's output IS reachable from the tuple | 0.85 |
| **A2** | the differing field is the register number reached **through** the symbol pointer (`sym+0x10 → +0x1c`, or `sym+0x8 → +0x1c`), not any byte of the operand record itself | 0.70 |
| **A3** | at `sched2` the assigned-register descriptor is **absent** (NULL, printed as a refusal) for a majority of kind-2 symbols, and present at `sched3` — COLOR's write is `none → r`, not `r → r'` | 0.55 |
| **A4** | the candidate id `sym+0x1c` is **stable** across 10 runs, so operand rows can enter the canonical stream without breaking G2 | 0.80 |
| **A5** | ≥ 1 operand pointer fails `plausible()` somewhere in the 384-fixture population, i.e. the fail-closed path is REACHED and is not decorative | 0.45 |
| **A6** | probe A's verdict is **"COLOR is gradeable"** — at least one field that COLOR writes is observable from the tuple spine at both phases | 0.80 |

A refutation of A1 promotes `#3323`'s null from "the tuple record" to "the
tuple record and everything one dereference from it", which is a *stronger*
finding and is to be reported in those words.

---

## 2. Probe B — the final schedule's output

### What will be built

Two things, and the second is the one that matters:

1. **An eighth tap site.** `0x10b7e701: e8 2c f9 ff ff  call 0x10b7e032`,
   inside `0x10b7e6af` (the per-function phase orchestrator), which is the
   **first call after `0x10b7df57` returns** — and `0x10b7df57` is where
   run 4 (`sched0`, site `0x10b7e00c`) lives. `ecx = esi` = the function
   record at that site (`10b7e6ff: 8b ce mov ecx,esi`). Site name `after0`.
2. **A whole-function tuple walk from the function record**, which the
   region tap structurally cannot do:
   `func+0x8` → header, `header+0x4` → first block, `block+0x4` → next
   block, `block+0x20` → first tuple, `block+0x1c` → one-past-the-end tuple
   (`FUN_10b2ceb7`: `iVar14 = *(int *)(*(int *)(param_1 + 8) + 4)`,
   `piVar13 = *(int **)(iVar14 + 0x20)`,
   `while (piVar13 != *(int **)(iVar14 + 0x1c))`,
   `iVar14 = *(int *)(iVar14 + 4)`). This answers `w-stageoracle`'s **P10**
   — the function-record → tuple-list-head offset it registered at 0.20 and
   reported NOT NEEDED — in the affirmative, and it removes the suffix
   re-read inflation the instrument fix above only *measures*.

### Predictions

| # | prediction | p |
|---|---|---:|
| **B1** | the `after0` site passes both fail-closed checks and fires **exactly once per function** (count equal to `sched1`'s) | 0.75 |
| **B2** | the whole-function walk's tuple set at `sched2` **contains** every distinct row the region walk reports at `sched2` — the cross-derivation that licenses the record-layout reading | 0.65 |
| **B3** | the `after0` walk **DIFFERS** from the `sched0`-entry walk on ≥ 1 function of `il_call_perm.cpp` — run 4 really does reorder, and its output really was unobserved | 0.60 |
| **B4** | the `after0` walk's real-instruction tuple count equals the `.cod` listing's instruction count for the same function, on ≥ 1 function | 0.30 |
| **B5** | probe B's verdict is **"the final schedule IS gradeable"** with the new site | 0.65 |

B4 is registered low deliberately: the `.cod` carries directives, labels and
data the tuple list need not, and a count equality would be a windfall rather
than the expected outcome. **B4's failure does not touch B3 or B5.**

---

## 3. Probe C — port-side region-trace feasibility

### What will be measured

One already-byte-exact fixture. Ask `PortC2` (or a test-only shim beside it)
to emit the same region-boundary trace the tap emits — *"region 1 starts
here, ends here; region 2 …"* — and diff against the tap's.

### Predictions

| # | prediction | p |
|---|---|---:|
| **C1** | the port can emit a matching region-boundary trace on the chosen fixture **without new machinery** | 0.20 |
| **C2** | the port has **no representation** in which c2's region rule (`≤ 0x50` tuples; end at category `0x12`/`0x14`/`0x19`/`0x1b`, or `0x17` with opcode `0x30f`) is even expressible, because the port's function bodies are encoded instruction bytes selected per whole-function shape, not a tuple list with categories | 0.85 |
| **C3** | probe C's verdict is **"per-stage grading requires reproducing c2's region decomposition"**, i.e. arch review finding 1's last bullet is confirmed | 0.80 |

C1 and C2 are not complements: C1 could fail because the port's *boundaries*
disagree while a representation exists.

---

## 4. Neutrality, re-run at the new table (brief's explicit requirement)

`w-stageoracle`'s `stage-tap-obj-differs 0` is a statement about **seven**
sites and a shallow payload. This lane arms **eight** and reads three levels
of foreign pointer deeper.

| # | prediction | p |
|---|---|---:|
| **G1′** | `stage-tap-obj-differs` is **0** over `fixtures/cpp`, armed-and-fired denominator, payload + operands ON | 0.75 |
| **G1″** | the armed-and-fired denominator at eight sites is **≥ 355** (the seven-site figure), i.e. the new site does not cost coverage | 0.70 |

G1′ is registered at 0.75 rather than 0.9 because the deeper walk touches
memory the seven-site payload never read, and a fault there is a crash, not a
byte difference — which would show as an ERROR, not a DIFFERS.

---

## 5. The cost curve (deliverable 5) — what is registered about it

| # | prediction | p |
|---|---|---:|
| **E1** | the per-stage curve will find **at least one** of {COLOR, sched runs 1–4, globregs, lowering band} **not gradeable** even after probes A and B | 0.60 |
| **E2** | the two unbudgeted integration prerequisites price **larger** than the sum of the per-stage construct costs — i.e. the critical path is integration, not any single pass | 0.70 |
| **E3** | the calibrated step-5 total is **larger** than the proposal's raw 12–24 month figure once CEILING §5's ~5:1 is applied as a lower bound | 0.55 |

---

## 6. What this lane will NOT do, registered so absence is not read as coverage

* No `crates/` emit rule, refusal predicate or census rule will read a
  `stage-*` key. The standing bound (`stageoracle` §8) is unchanged.
* No workload scan is planned; if one is taken it comes with a stamp read
  before and after.
* The 57 phase-beacon sites stay unarmed (`w-stageoracle` §6.1 q3).
* No fixture is claimed and no census number moves. Predicted reach **0**.
