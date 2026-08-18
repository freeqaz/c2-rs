# PREREG — `w-calleeguard`: guard all four raise sites of the `callee-unresolved` key family

    Lane:   w-calleeguard
    Base:   master 44794fa4
    Frozen: BEFORE any probe, any mutant, any `cargo test` run in this worktree.
            This file is the lane's FIRST commit.

`docs/rungs/README.md` § "Two rules a probe must satisfy" (boards #3219 / #3231)
binds this lane in full. Every colour below is registered here and nowhere else;
the results table in the rung doc will be **derived from the run logs**
(`work/w-calleeguard/logs/`), never accumulated.

---

## 1. The frame — what is being guarded, re-located at THIS base

`w-mutcensus` (`docs/rungs/2026-08-17-mutcensus.md` §3, §4.3) measured, at
`3835469c`, that the whole four-site `callee-unresolved` key family is
**unguarded**: `CS5`–`CS8` all GREEN. Its default arm routes
`callee-unresolved-tail-call`, board **#3209**'s key over **1,296** function
bodies on the 878-TU workload.

**The line numbers are re-located here rather than inherited** — `w-fence163`
and `w-npos` both landed in `c2-il` since `3835469c`, and a handed-down line
number is exactly the figure this repo keeps catching wrong. Re-located by
`grep -rn CALLEE_UNRESOLVED crates/` at `44794fa4`:

| census id | site at `3835469c` | **site at `44794fa4`** | arm | key raised |
|---|---|---|---|---|
| `CS5` | `census.rs:1265` | **`crates/c2-il/src/func/census.rs:1308`** | `"framed-call" =>` | `callee-unresolved-framed-call` |
| `CS6` | `census.rs:1267` | **`census.rs:1309–1311`** | `l if l.starts_with("call-sequence") =>` | `callee-unresolved-call-sequence` |
| `CS7` | `census.rs:1270` | **`census.rs:1312–1314`** | `l if l.starts_with("empty-dtor") =>` | `callee-unresolved-dtor-delegation` |
| `CS8` | `census.rs:1272` | **`census.rs:1315`** | `_ =>` (the DEFAULT arm) | `callee-unresolved-tail-call` |

All four are arms of one `match label` inside the `None =>` arm of
`match shape_to_function(...)` — i.e. they fire when the body **parsed** but the
shape could not be turned into a function, with no data-symbol `sym_fail`
pending.

### 1.1 A frame fact this lane must state up front, because it changes the answer to deliverable 5

`grep -rn 'CALLEE_UNRESOLVED_\(TAIL\|FRAMED\|SEQ\|DTOR\)' crates/` at
`44794fa4` returns, outside the `const` declarations in `func/body/mod.rs` and
the `use` at the top of `census.rs`, **exactly one raise site per key**:
`census.rs:1308`, `:1310`, `:1313`, `:1315`.

So for this family **k = 1 for every key**, and `w-mutcensus` F2's mechanism —
*"a key with k raise sites contributes k − 1 unguarded sites by construction"* —
**does not apply to it at all**. These four sites are not unguarded by
construction; they are unguarded because nobody wrote a witness. That is
registered as a *frame fact*, read out of the source before any probe, not as a
prediction. What is registered as a prediction is what follows from it (P1, P12
below).

---

## 2. The instrument — key strings through the public API, exactly `w-guards`' form

`crates/c2-il` is peer `w-dataseam`'s seam this wave. Guards are written from
`crates/c2-harness/src/gap/tests.rs` and reach the sites through the **public**
`IlBundle::census_functions()`, asserting on **`FnVerdict::key()`** — verbatim
what `scan.rs` concatenates into `emit-cflow-modeled-key|{}`.

**On the key STRING, never the constant** (`2026-08-16-guards.md` §2): a guard on
the constant passes a mutation that renames the constant *and* its uses while the
published key moves; a guard on the key string cannot.

### 2.1 The cells

One cell per raise site, each a **whole captured `.ex` function segment**
transcribed into the test (the `DYNINIT` precedent, `gap/tests.rs:3832`), wrapped
in the `4F 1F` segment header, with a `.gl` that **does not name the callee
token** — which is the honest way to make `shape_to_function` return `None` while
`sym_fail` stays `None`.

| cell | transcript source | expected label | site it must route through |
|---|---|---|---|
| **F** | `func/mod.rs::test_fixtures::MVP_FRAMED` — `int f(int a){ return g(a)+1; }` | `framed-call` | `CS5` |
| **Q** | `…::SEQ_TWO_VOID` — `void f(int a){ g1(a); g2(); }` | `call-sequence` | `CS6` |
| **E** | `…::DTOR_DELEGATE` — `Der::~Der() {}` | `empty-dtor-delegation` | `CS7` |
| **T1** | `gap/tests.rs::DYNINIT` + `gl_named(0x02)` (`w-guards`' cell C) | `multiarg-tail-call` | `CS8` (default) |
| **T2** | `…::MVP_CALL` — `void f(){ g(); }` | a void tail call | `CS8` (default), by a **second** label |

**T2 exists because `CS8` is a DEFAULT arm and one witness cannot say so.** A
single cell is consistent with the arm having been keyed on that cell's own label;
two cells with *different* labels reaching the same key is the statement that the
arm is the catch-all.

### 2.2 Cell separation, stated as a minimal difference and asserted

`w-guards`' standard (its `.gl` pair differing in exactly 1 byte, its marker pair
in exactly 2) is carried:

* every cell's `.gl` is built by the same `sym_rec`/`data_rec` helpers already in
  the module, and the **callee-named / callee-unnamed** pair for at least one cell
  differs by exactly the callee's record and nothing else — asserted as a
  byte-level prefix/length identity, not trusted;
* **"exactly one census row" is a NAMED failure**, reusing `key_of`'s existing
  panic path (`gap/tests.rs:3937`), never an `unwrap`;
* the arm table asserts **distinctness of the four keys as a count**, so a
  collapse cannot leave the equalities vacuously satisfied.

---

## 3. Registered predictions — probability form, no discount factor

| id | prediction | P |
|---|---|---:|
| **P1** | **All four** raise sites are guardable from `crates/c2-harness` with **zero bytes landed in `crates/c2-il`** | 0.75 |
| **P2** | Cell **F** (framed, callee unnamed) keys `callee-unresolved-framed-call` (with or without a `:eof` suffix) | 0.60 |
| **P3** | Cell **Q** keys `callee-unresolved-call-sequence` | 0.55 |
| **P4** | Cell **E** keys `callee-unresolved-dtor-delegation` | 0.50 |
| **P5** | Cell **T1** keys `callee-unresolved-tail-call` (`w-guards` §3 measured this string; re-measured here, not inherited) | 0.90 |
| **P6** | Cell **T2** reaches the **same** key as T1 under a **different** label | 0.60 |
| **P7** | The four family keys are pairwise **distinct** as observed strings | 0.90 |
| **P8** | Every cell yields **exactly one** census row | 0.70 |
| **P9** | At least one of the five cells needs a transcript repair (header wrap, module end, `.sy`) before it produces a row at all | 0.65 |
| **P10** | At least one of the four sites proves **unguardable from the harness side** and is recorded as such rather than crossed into `c2-il` | 0.20 |
| **P11** | The re-measured census GREEN count over the 63 sites is **26** (30 − 4) | 0.70 |
| **P12** | A guard binding **all** raise sites of one key — not one witness per key — is **expressible**, and for this family it is *trivially* so because k = 1 (§1.1). The general form (**one witness per raise site**, not per key) is stated as a construction and priced | 0.80 |
| **P13** | The general form is **demonstrated** in this lane on a family where two sites share ONE key (e.g. `leaf_store`'s `STORE_RUN_BIND_GROUP_SHAPE`, 4 sites / 1 key) | 0.20 |
| **P14** | `cargo test --workspace --release` ends at **1,660 + k**, `1 ≤ k ≤ 6` | 0.80 |
| **P15** | 878-TU scan: **0 deltas over 394 `gap-metric` keys**, 878 verdict lines, at both ends | 0.97 |
| **P16** | `scripts/gate.sh --jobs 4 --require-graded` PASS at the tip, with the fourth `debug-lane` row present | 0.92 |

---

## 4. Registered mutant colours — ALL of them, before any run

Probe: `cargo test --workspace --release --no-fail-fast`, run **in this
provisioned worktree** (`compilers/` symlinked; `configure_existing_worktree`'s
own gate `fixtures/cpp/w5_chain.cpp -> 4/4` verified at setup).

**Baseline `N0` = 1,660 passed / 0 failed / 43 targets** (briefed; re-measured as
the lane's first run and treated as INVALID if it does not reproduce).

### 4.1 Phase R — the four sites BEFORE any guard lands

This phase exists because of `w-fence163`'s finding that *a guard that catches a
mutation incidentally is not a guard*, and because two peers landed in `c2-il`
since the census was taken. A site that reads RED here was never this lane's to
close.

| id | mutation at `44794fa4` | registered |
|---|---|---|
| **R5** | `census.rs:1308` `"framed-call" => CALLEE_UNRESOLVED_FRAMED` → `CALLEE_UNRESOLVED_TAIL` | **GREEN** 0.85 |
| **R6** | `census.rs:1310` `CALLEE_UNRESOLVED_SEQ` → `CALLEE_UNRESOLVED_TAIL` | **GREEN** 0.85 |
| **R7** | `census.rs:1313` `CALLEE_UNRESOLVED_DTOR` → `CALLEE_UNRESOLVED_TAIL` | **GREEN** 0.85 |
| **R8** | `census.rs:1315` `_ => CALLEE_UNRESOLVED_TAIL` → `CALLEE_UNRESOLVED_FRAMED` | **GREEN** 0.85 |

P(all four still GREEN at `44794fa4`) = **0.70**; P(at least one has become
incidentally RED since `3835469c`) = **0.30**.

### 4.2 Phase G — the same four with the guards in

| id | registered |
|---|---|
| **G5** | **RED** 0.90 |
| **G6** | **RED** 0.90 |
| **G7** | **RED** 0.90 |
| **G8** | **RED** 0.90 |

A `G` that reads GREEN is a **failed** deliverable for that site and is reported
in that word.

### 4.3 Controls — pinned BY NAME, re-run in every environment (README rule 1)

| id | what | registered |
|---|---|---|
| **N0** | clean tree, guards in | **GREEN**, 1,660 + k / 0 / 43 |
| **C1** | `crates/c2-il/src/func/body/shapes/calls.rs` arity fence `syms > 1` → `syms > 2` | **RED**, and the failing set must be **exactly** `gap::tests::wr1_census_key_guards::the_call_argument_arity_fence_is_a_series_and_admits_exactly_one_symbol` **and** `…::the_two_symbol_thunk_exemption_turns_on_the_bare_body_marker_alone` — the "G1 pair", reproduced by `w-guards` and by `w-mutcensus` in eight worktrees |
| **N1** | with the guards in, the callee-naming `.gl` **restored** on one cell (the input perturbed, **every assertion byte-identical**) | **RED** 0.80 |

**C1 is the environment validator.** Both tests in its failing set are
capture-driven, so a worktree whose captures were skipping cannot reproduce the
pair. It is run **before** the first mutant and **after** the last.

### 4.4 Invalidation rules — counts and durations, never exit codes

A run is **INVALID**, not a colour, if any of:

1. the `census_gate` target's wall-clock is **< 1 s** (a skipping differential is
   0.00 s; a grading one is tens of seconds) — `w-mutcensus` D6;
2. the target count is not **43**;
3. the executed-test count is not `N0 ± (the mutation's own expected delta)`;
4. `git diff --quiet -- crates/` is dirty in a way the runner did not apply, or
   the tree is uncommitted;
5. `C1` in that environment does not reproduce the G1 pair **by name**.

**A colour from an unvalidated environment is VOID, not provisional**: it is
discarded, re-run from scratch, and its log is **kept** as `*.INVALID.log`.

---

## 5. What this lane will NOT claim

* **"Graded tree identical at both ends" does not apply** (board #3215): this
  lane lands `#[cfg(test)]` code under `crates/`, so `gate.sh`'s content hash of
  `crates fixtures scripts` moves by construction. The discriminating check is
  the **gate-count identity diff**, line for line.
* **No release-binary sha256 comparison across worktrees** (board #3224):
  `CARGO_MANIFEST_DIR` is compiled in, so that comparison is void.
* GREEN/RED here are scoped to `cargo test --workspace --release`, 43 targets.
  The 381 × 18 fixture gate is **not** re-run under each mutant.
* A guard that fires is not a correctness proof. These cells grade the census's
  **key assignment**, not whether the key is the right thing to refuse on.
