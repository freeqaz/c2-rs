# `w-guard` — PREREGISTRATION, frozen before the first measurement

    Lane:      w-guard (wave 11, decision 13)
    Kind:      construct rung
    Base:      5db186426 (master), worktree .claude/worktrees/w-guard, branch wt-w-guard
    Date:      2026-08-25
    Fixtures:  none — construct rung: three funnel/instrument guards
    Census:    +0 (no acceptance surface is touched)
    Board:     #3573–#3578 reserved
    Owns:      scripts/ ; crates/c2-harness/tests/
    Rung:      docs/rungs/2026-08-25-w-guard.md

**This file is frozen at its first commit. Predictions are never edited
afterwards — a miss is scored as a miss in §5 of the rung.**

---

## 0. What was read before anything was written

Per the brief, `docs/BOARD.md` was grepped for `#3552`, `#3470`, `#3510`,
`#3156`, `#1236` and the **oldest** hit read **last**:

| row | date | what it says that binds this lane |
|---|---|---|
| **#3552** | 2026-08-25 | the reap that destroyed the pinned arms; *"the reap guard is the missing funnel step and it is not built"*; and `#3470` reproduced at a **fourth** site |
| **#3510** | 2026-08-24 | `emit_set_violations()` reads **1**, `decomp_pch.cpp` `fn_total 1260` vs `emit-emitted 0`; **recorded, NOT diagnosed** |
| **#3470** | 2026-08-24 | a stamp guard correct about its own question is silent about an empty run; **only a denominator can catch an absence** |
| **#3156** | 2026-08-15 | `git add -f` walked past two working ignore rules in two lanes three weeks apart — **a funnel gap, not a lapse** |
| **#1236** (oldest, read last) | 2026-08-08 | *"the standing NUL-free check… reports 0 on this file"* — **a guard nobody has watched fire is not a guard**, and the copied-everywhere `st_size` check is *load-bearing nowhere*: it passes exactly when the writer is busy |

`#1236` is the one that shapes every item below. Each of the three guards
this lane ships is required to be **watched going red on a planted case**
before its green is quoted anywhere, and each plant is reversible.

Also read: `CLAUDE.md`, `docs/STATUS.md`, `docs/rungs/README.md` §"Lane
kinds", `docs/DECISIONS_2026-08-22.md` decision 13,
`work/coordinator/gatebase/HOWTO_DIFF.md`, `scripts/wt_reap.py`,
`scripts/tracked_artifact_audit.sh` + its test (the worked example the brief
names), `scripts/status.sh`, `crates/c2-harness/src/gap/report.rs`
`emit_set_*`.

---

## 1. The axis on which this rung can FAIL with every byte identical

The cost clause (`rungs/README.md`, board `#3336`): a required-zero **byte**
delta is silent about everything that is not a byte, and a criterion that
cannot fail abstains rather than passes. This rung touches no code on
`PortC2::build`'s path at all, so throughput is not the axis. The two axes
named here, before starting, are:

1. **The published-metric surface.** `scripts/status.sh` renders the 27-row
   metric registry that `docs/STATUS.md`'s generated block *is*. Item 3
   changes how two of those rows render. The rung fails on this axis if any
   registry row other than `emit-ceiling` / `emit-ceiling-gate` changes its
   rendering, if `status.sh --check` stops passing, or if a row that should
   read `NO-RESULT` starts reading a number (or the reverse). Observed by
   running `status.sh --check` and by diffing `--raw` output on identical
   inputs before/after.
2. **The guards' own false-positive cost, priced two-sided.** A reap guard
   that refuses trees holding nothing pinned is paid for by every future
   reap; `#3545` measured its own wide prescription at **8,041** and had to
   re-scope. So each guard's **wide number is measured and printed before it
   is scoped**, and the scoped number is justified against it. The rung fails
   on this axis if a guard ships whose at-HEAD false-positive population was
   never counted.

---

## 2. Item 1 — the reap guard (`#3552`)

### 2.1 The design under test

The failure was **not** `scripts/wt_reap.py`: that script never passes
`--force` and says so in its docstring. The destroyer was a **hand-typed
`git worktree remove --force`**. So a guard that lives only inside
`wt_reap.py` cannot fire on the failure it is written for — which is exactly
`#1236`'s shape (a check that passes precisely when it matters).

The design being tested is therefore **git's own refusal**: a worktree
carrying `git worktree lock` is refused by `git worktree remove`, and the
hypothesis is that **plain `--force` is refused as well** and only
`--force --force` gets through. If that holds, the guard is: detect pinned
artifacts → `git worktree lock` with the pin manifest as the reason → the
hand-typed command refuses on its own.

### 2.2 Predictions

| # | prediction | p |
|---|---|---|
| **P1.1** | `git worktree remove --force` **REFUSES** a locked worktree; `--force --force` is required. Watched live, not read from a man page | **0.85** |
| **P1.2** | The wide detector — regular files whose first four bytes are `\x7fELF`, anywhere in a worktree, **outside `target/`** — reads **≤ 20** over the 6 live non-primary worktrees plus the primary, and **0** in the primary repo | **0.50** |
| **P1.3** | The guard's `--self-test` drives **≥ 4 distinct planted classes RED**, control green, each plant reversible | **0.90** |
| **P1.4** | At least one worktree live on this box **right now** holds an unlocked pinned-shaped artifact (a live pre-existing violation, `#3545`'s pattern) | **0.45** |
| **P1.5** | `wt_reap.py` classifies `LOCKED` already and therefore needs **no** new refusal for the locked path — the new refusal it needs is for the **unlocked-but-pinned** path | **0.80** |
| **P1.6** | The guard flags **itself** on its first tracked run in some way (`#3545`'s self-exemption trap recurring) | **0.25** |

### 2.3 Decline floor

If **P1.1 is false** — if `git worktree lock` does not make plain `--force`
refuse — then no in-git enforcement exists for the hand-typed path, and I
**decline to claim a guard**. What ships in that case is the audit plus a
`wt_reap.py` refusal, and the rung states in these words that **the
hand-typed `git worktree remove --force` path remains UNGUARDED**, rather
than shipping a check that cannot fire on the recorded failure.

If **P1.2 misses high** (the wide number is large), the detector is
re-scoped, the wide number is **printed every run** the way
`tracked_artifact_audit.sh` prints its 8,041, and the re-scoping is
justified in the rung rather than performed silently.

---

## 3. Item 2 — `#3470` bites backwards

### 3.1 What is being asserted

Every historical re-run builds a **pre-`repo_root()`-fix** commit, whose
binary resolves `compilers/` relative to its own build tree, prints
`SKIP: toolchain absent`, and **exits 0 having graded nothing**. The fix
cannot reach backwards. The remedy already exists and is re-derived by hand
every time (`w-s1c3`'s explicit `C2RS_COMPILERS` / `C2RS_WIBO` pin), so what
is missing is not a remedy but a **refusal**.

### 3.2 Predictions

| # | prediction | p |
|---|---|---|
| **P2.1** | `scripts/cost_arms.py` today has **no** preflight that would refuse an arm printing `SKIP: toolchain absent`, and no per-arm denominator assertion | **0.75** |
| **P2.2** | A planted arm — an executable that prints `SKIP: toolchain absent` and exits 0 — is accepted by the protocol **as it stands today**, producing a run that graded nothing without a nonzero exit | **0.80** |
| **P2.3** | After the change, the same plant produces a **nonzero exit with a named reason** naming the arm | **0.90** |
| **P2.4** | The preflight also catches the weaker sibling — an arm that runs but grades a **zero denominator** — and this is a distinct planted case from P2.2 | **0.70** |
| **P2.5** | `scripts/scan_pair.sh` already refuses on SKIP (`#3470`'s own repair) and therefore needs no change; the gap is on the **cost-arm** path only | **0.65** |

### 3.3 Decline floor

If **P2.1 is false** — the preflight already exists — this item lands as
`already closed`, the rung says so with the line numbers, and **no second
redundant fence is shipped**. A fence whose only effect is to duplicate an
existing one still costs every future reader a decision about which one is
authoritative.

### 3.4 Recorded if cheap, not measured for its own sake

`f6f56df78` is reachable from **no ref** — it survived in the object store
only. Recording this costs one `git` invocation; it is not a lane.

---

## 4. Item 3 — `#3510`, the voided ceiling `STATUS.md` publishes

### 4.1 The two candidate diagnoses

`emit_set_violations()` counts `match` TUs where `fn_total !=
emit-emitted`; it reads **1** (`src/system/decomp_pch.cpp`, `fn_total` 1260
vs `emit-emitted` 0). Its own doc says a nonzero **voids the ceiling**.
Either:

* **(a)** the defect is `decomp_pch.cpp`'s data — a PCH TU whose
  `emit-emitted 0` is wrong; or
* **(b)** the defect is the **predicate's premise**. `emit_set_reachable_tus`
  argues *"`PortC2::build` takes `il.functions()` … and pushes exactly one
  `.text` COMDAT per entry … `fn_total` is exactly that segment count"* — but
  `fn_total` comes from `census_functions()`, which splits at the **census**
  marker `4C 4F 11`, while `functions()` splits at the **gate** marker
  `4F 1F`. The premise silently identifies two different counts.

### 4.2 Predictions

| # | prediction | p |
|---|---|---|
| **P3.1** | The diagnosis is **(b)**, the predicate's premise, not `decomp_pch.cpp`'s data | **0.75** |
| **P3.2** | `decomp_pch.cpp`'s reference obj genuinely carries **0 `.text` sections** and is ~900 B, so `emit-emitted 0` is CORRECT | **0.85** |
| **P3.3** | The port matches that TU by emitting **0** `.text` COMDATs — i.e. it does not emit one per `fn_total` entry, falsifying the premise directly | **0.60** |
| **P3.4** | `emit_set_violations_gate()` reads **0** violations at HEAD: the LO-anchored control is red and the gate-anchored one is green | **0.55** |
| **P3.5** | The violation count at HEAD, re-measured, is still exactly **1** and still exactly `decomp_pch.cpp` | **0.80** |
| **P3.6** | `scripts/status.sh --check` is wired into **no** `cargo test` target today — a validating flag whose exit status gates nothing | **0.70** |

### 4.3 What ships, and what explicitly does not

**Repairing the predicate is NOT in scope** (the brief; it is a separate
priced decision). What ships is (i) the diagnosis, address- and
line-cited, and (ii) an **assertion**: the violation control must stop being
a printed line. The intended home is `scripts/status.sh` — the generator of
the block that publishes the ceiling — which must render both ceiling rows
as **VOID** rather than as bare numbers whenever the control is nonzero,
plus a parser self-test in `--check` that is itself gated by a `cargo test`
target under `crates/c2-harness/tests/`.

Ownership fence: `crates/c2-harness/src/` belongs to `w-decodereach` this
wave. If the assertion cannot be built without editing a non-test file
there, this lane **STOPS and reports** rather than editing it.

### 4.4 Decline floor

If a scan of `decomp_pch.cpp` cannot be produced in this worktree, the
diagnosis is reported as **UNVERIFIED-FROM-RECORD** in those words and **no
assertion is shipped whose red state I have not watched**. A guard validated
against a population that does not contain the case is not validated
(`#3545`).

---

## 5. The required-zero byte delta and its proof-of-failure

* Identity diff over the **21 count-bearing gate rows** (18 mode lanes +
  `expr-sweep` + `mode-cross` + `debug-lane`; `hatch-red` / `ladder-red`
  excluded as `n/a`), per `work/coordinator/gatebase/HOWTO_DIFF.md` — cut to
  `LANE VERDICT graded/total match`, run dir normalised, base and tip both
  produced in this lane.
* **The diff is proved able to fail before its zero is trusted**: `#3515`'s
  one-TU-refused signature (`O1 186→185`, `O1-EHsc 187→186`, `O1-Oi
  188→187`, `O1-Oi-EHsc 189→188`, `O1-Oi-GR 188→187`, `O1-Oi-EHsc-GR
  189→188`, `debug-lane 2479→2473`) is fabricated into a copy of the base
  table and the procedure is required to print **exactly 14 diff lines over
  7 rows**. A zero from a procedure that has not been seen produce a nonzero
  is not a zero.
* The graded **tree hash WILL move** — this lane changes files under
  `scripts/`, which the gate content-hashes. `HOWTO_DIFF.md` §"The graded
  tree hash is NOT part of the identity diff" covers this, and a hash that
  did **not** move would be the finding.
* `gate.sh` prints `GATE: REFUSED (DIRTY crates/)` and **exits 0**. The
  verdict line is read; a count is compared, never a status.

Predicted: **0 diff lines over 21 rows** at p = **0.90**. A nonzero is a
FAILED construct rung whatever else landed, and will be reported in that word.

---

## 6. Outcome word

Registered now, before building: this rung will report **exactly one** of
`converted | declined | instrument | built | FAILED`. The intended word is
**`built`**. It becomes **`FAILED`** if the byte delta is nonzero, or if
fewer than two of the three items land with a guard whose red state was
watched.
