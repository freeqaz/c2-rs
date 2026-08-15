# w-counted — PREREG

Frozen and committed **before the first change to `crates/` and before the first
`cl.exe` this lane runs.** (`scripts/setup_worktree.sh` compiles one fixture as
its own toolchain check; that is the setup script's, not this lane's probe.)

Lane `w-counted`, worktree branch `wt-w-counted`, off master **`1f85d14c`**.

---

## 0. The question, in one line

`w-slots` found-and-not-taken **#5**: `counted_accum_loop` is the only one of the
five loop shapes whose reader admits **two** modes, and *"whether its `/Ox`
acceptance is even correct appears UNGRADED"*. **"Appears" is what this lane
converts into a measurement.** If the `/Ox` arm is wrong or unwitnessed,
narrowing the reader to `/O1` retires `#746` fence B's exclusion for this class
without solving the charge. If it is right and witnessed, narrowing is a
regression and the decline is the deliverable — **priced two-sided**.

## 1. What is already on the record (read-only, before any probe)

Not evidence — the thing to be graded. Three published label numbers in two
weeks were wrong in the direction that made a fence look dearer to lift
(#3091, #3148, `LABEL_COUNTER.md` §4.2.1).

* `docs/rungs/2026-08-09-w-bdnz.md` §1 claims `fixtures/cpp/wbdnz_ctr.cpp` is a
  whole-TU `match` at `/O1` **and** `/Ox` over eleven cells, and P8b claims all
  18 gate lanes (`/Ox` 141, `/Ox /Gy` 139). **So `/Ox` acceptance does not
  "appear ungraded" on the record; it appears graded.** This lane re-takes it
  against the oracle rather than quoting it.
* `crates/c2-il/src/func/body/shapes/counted_accum_loop.rs:233-235` admits
  `Some(O1) | Some(Ox)`; `crates/c2-il/src/func/mod.rs:4493` returns `None` from
  `label_slots` for the class.
* `work/w-bdnz/LABEL_LEAD.md` measures the charge **+7 at `/O1`, +8 at `/Ox`**
  — and its instrument is *"the control puts a `leaf-none` in the first slot,
  the test puts the cell there"*, i.e. **a difference across two TUs whose
  source text differs**. That is exactly the form board **#3148** refuted:
  a TU's `.gl` counter depends on its own source text, so a lead differenced
  across two TUs is `Δcharge + Δseed`. **The `+7`/`+8` is therefore suspect and
  is re-taken with `work/w-slots/lead.py`, which cancels the seed inside the
  TU.**
* Board **#2002**: `counted_accum_loop`'s own committed `Err` over the 878-TU
  workload puts **2,239 of 2,286 (97.9 %) at clause 1** and no workload body in
  the class. **The workload delta is +0 by construction, not by hope.**

## 2. The registered claims

| # | claim | P |
|---|---|---:|
| **W1** | **`/Ox` acceptance is WITNESSED** — at least one tracked fixture is `match` at an `/Ox`-family gate lane *through this class*, re-measured in this lane's own run against real `c2.dll` under wibo | **0.92** |
| **W2** | **`/Ox` acceptance is CORRECT** on a cross of the class's free axes that has **never been crossed**: 7 accumulate opcodes × 2 counter signednesses = 14 cells. All `match` at `/Ox`; `mismatch` 0 | **0.75** |
| **W3** | the **narrowing probe FIRES** — narrowing the reader's mode gate to `/O1` alone moves ≥ 1 fixture verdict on **each** of the six `/Ox`-family lanes (`Ox`, `Ox-EHsc`, `Ox-Gy`, `Ox-Gy-EHsc`, `Ox-GR`, `Ox-EHsc-GR`) | **0.90** |
| **N1** | **this lane does NOT narrow the reader** (contingent on W1 ∧ W2 ∧ W3 all holding) | **0.85** |
| **L1** | the **`+7`/`+8` mode-dependence is a cross-TU differencing artifact**, and the seed-cancelled leads are the **same integer** at `/O1` and `/Ox` | **0.35** |
| **L2** | the seed-cancelled lead is **linear in the number of loops** over a 1/2/3 series with residual 0 — i.e. the charge is per-function and there is no per-TU slot hiding in it (`w-slots`' `2n+1` trap) | **0.70** |
| **L3** | conditional on L1 ∧ L2: the exclusion is **retired without narrowing**, by adding `+k * counted_accum_loop.is_some()` to `label_lead` and deleting the `None` arm, and `fixtures/cpp/wbdnz_ctr_then_framed_neg.cpp` converts at **both** modes | **0.30** |
| **M1** | ≥ 3 mutants go red with a separating control green under **every** one | **0.85** |
| **M4** | the **shipped must-fail claim reproduces**: `Some(self.label_lead() + 1)` for this class turns `wbdnz_ctr_then_framed_neg.cpp` into a live `mismatch` while `wbdnz_ctr.cpp` stays `match`. (`w-slots` found a shipped must-fail claim citing a fixture that never existed; this one's fixture exists and nobody has re-run it) | **0.85** |

## 3. Falsifiers — named in advance, with what each forces

* **F1 — a `mismatch` anywhere in the `/Ox` cross.** W2 dies, **narrowing
  becomes the right move**, and the finding is reported as an **alarm** (a live
  wrong emit), not as a gap. This is the outcome that would make the lane
  `converted` by refusal.
* **F2 — the narrowing probe moves NOTHING at `/Ox`.** Then `/Ox` acceptance is
  *unwitnessed on the tracked corpus*, the decline would be priced at zero, and
  narrowing is free. This is the absence-read-as-success family and would be
  stated in those words.
* **F3 — peer collision.** `crates/c2-core/src/codegen/labels.rs` and
  `block_ir.rs` are `w-fencea`'s; `docs/LABEL_COUNTER.md` is `w-labeltable`'s;
  `crates/c2-core/src/coff/` is off-limits. Zero files under any of them are
  opened. No shared predicate other than this class's own reader/`label_slots`
  arm is narrowed, shadowed or redefined.
* **F4 — L3 installed and something reddens.** Revert, commit the revert with
  its reasoning, report the number that failed.
* **F5 — the lead series is not linear.** `w-slots` shipped-a-3-that-was-a-2 by
  reading one cell. **`L1` is registered NON-DECLINING**: if the series
  disagrees with any single cell's reading, the *series* is followed and the
  disagreement is the result, not something to adjust to.

## 4. The registered outcome numbers — CEILING, no discount factor

**Naming the population every time** (#3125: `match` has three meanings).

| quantity | **population** | registered ceiling |
|---|---|---|
| `mismatch` | every one of the 18 gate lanes, `expr_sweep`, `mode_cross`, and the 878-TU scan | **0**, absolutely |
| **fixture-gate `match`** | **381 fixtures × 18 mode lanes** | **+1 on each of the 12 `/O1`- and `/Ox`-family lanes (= +12 verdicts) IF L3 hits; otherwise +0 on all 18.** Must not FALL on any lane |
| `c2rs perf` `Match` | **381 fixtures at the `/Ox` DEFAULT** | **+1 if L3 hits, else +0** |
| **878-TU workload `match`** | **878 dc3 TUs** | **25 → 25, +0** — board #2002: no workload body is in this class |
| `fnbyte-exact` | 878-TU scan | **35734 → 35734, +0**; must not fall |
| `codegen-gap` / `vocab-gap` / `capture-fail` | 878-TU scan | **0 / 845 / 8**, unchanged |
| census | `c2rs census` | **+0** — no new function class is admitted either way |
| workspace tests | `cargo test --workspace --release --no-fail-fast` | **≤ 1616 / 42 targets** (base **1610 / 42**) |

## 5. What this lane will NOT do

* Not use, extend or re-fit `w-fenceb`'s `R1′` or the disqualified 23-of-23
  rule. **No general loop-kind rule is claimed** (#3127).
* Not file "two classes charge 2, both `for`" as a third witness for a
  kind-keyed rule (`w-slots` #4; one-witness discriminators refuted twice).
* Not edit `docs/LABEL_COUNTER.md`. If a number in it is wrong, it is
  **reported** for `w-labeltable` to settle against the oracle.
* Not mint board numbers. Rows land **unnumbered**; next free is #3151 and the
  coordinator serializes.
