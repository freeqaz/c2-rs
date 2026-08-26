# PREREG — `w-atend`: can the admission layer own a refusal REASON at all?

    Lane:      w-atend
    Date:      2026-08-26
    Kind:      construct rung — required-zero byte delta, Fixtures: none, Census: +0
    Base:      f202268f6   (master; `crates/` identical to c13cebbca — the two
                            commits between touch only docs/, checked)
    Base table: work/coordinator/gatebase/base_c13cebbca.txt
    Board:     #3591–#3596 reserved
    Funded by: docs/DECISIONS_2026-08-22.md decision 14 (2026-08-26)

**Frozen before any measurement of this lane's own changes.** Nothing below is
edited after the first `cargo test` of a tree carrying this lane's `crates/`
diff. Everything in §0 is a READ of the tree as it stands at `f202268f6` and of
work two peers landed on 2026-08-25 — it is not a measurement of anything this
lane does, and §0 is written first on purpose, because my brief's step 1 is
*"establish the fact before designing anything"*.

---

## §0 — THE FACT, READ BEFORE ANYTHING WAS DESIGNED

### §0.1 The claim under test

Board **#3556** (`w-unfuse`, 2026-08-25), restated in
`crates/c2-il/src/func/body/decode.rs`'s own module doc:

> **THE ADMISSION LAYER CANNOT OWN A REFUSAL *REASON*** — a `Block` says where
> the **READ** stopped, and a policy that refuses a body the decode read whole
> has no such point. Minting a `Block` for it publishes a `:eof` census key
> **no scan can ever reach**.
>
> …and its stated consequence: *"an admission verdict is a yes/no, and **every
> refusal KEY in this port belongs to the DECODE**."*

### §0.2 What the tree says — two independent readings, both cited

**(a) The key space already expresses it, and has for a long time.**
`Block::at_end` (`crates/c2-il/src/func/body/mod.rs:1601`) is documented as
*"a refusal raised **after the parse reached the end of the segment** — the
post-parse gates, which run only on a body that already parsed end to end"*.
There are **7** production `Block::at_end(` sites in `crates/c2-il/src`, all in
`crates/c2-il/src/func/census.rs` (lines 1446, 1450, 1500, 1516, 1528, 1594,
1684). At least two of them are admission predicates in the plainest sense, by
their own constants' documentation:

* `OPT_MODE` (`func/body/mod.rs:1145`) — *"a function whose body parses in class
  but whose optimization-settings word is not one this port emits under"*;
* `CALLEE_DEFINED_IN_TU` (`func/body/mod.rs:1199`) — *"the body parses in class,
  its callee resolves, and the TU DEFINES that callee, so c2 may inline it and
  **the port may not emit**"*.

Neither is a statement about where a read stopped. Both are refusal REASONS
owned by a layer downstream of the decode, and both are census keys.

**(b) A scan reaches them, and a peer measured how wide.**
`w-decodereach` (`docs/rungs/2026-08-25-w-decodereach.md` §13.1, board **#3582**)
ran the I1 divergence detector over the 878-TU workload and reports
`decode-reach-grammar` **711,729** against `decode-reach-admitted` **707,728** —
**`grammar-not-admitted` = 4,001** bodies the decode read whole and admission
refused, decomposed by the very key each one publishes:

| census key | bodies |
|---|---:|
| `callee-unresolved-tail-call:eof` | 2,282 |
| `data-sym-unresolved:eof` | 1,665 |
| `data-sym-not-extern:eof` | 52 |
| `callee-defined-in-tu:eof` | 1 |
| `data-sym-strlit-fenced:eof` | 1 |

*"Every one is `:eof` — the parse ran to the end of the segment and the refusal
came afterwards."*

### §0.3 So the claim is FALSE as stated, and TRUE only of one thing

Read together, (a) and (b) say the general claim is refuted by the tree that
carries it: **4,001 workload bodies are refused by admission after a whole read,
under five distinct `:eof` keys, every one of them reachable by a scan and
ranked by it.** *"Every refusal key in this port belongs to the decode"* is
false of five keys and 4,001 bodies as of 2026-08-25.

What is true is narrower and it is about the **caller**, not the layer:
`AdmissionPolicy::Nothing`'s key would be unreachable **because no production
call site passes a non-default policy** — not because admission cannot name a
reason. That is a property of who selects the policy. The tree already carries
the counter-precedent for that too: `Relax` (`func/census.rs:94`) is a named,
settable parameter on the census whose **non-default** level is selected by
production code (`crates/c2-harness/src/gap/scan.rs:982`,
`crates/c2-harness/src/gap/factors.rs:2430`) and whose numbers are published
(`fnbyte-blind-level|<name>`, `gap/blind.rs:446`).

**Therefore this lane takes my brief's branch 3** — *"if it is false or only
locally true: build the second variant, take `EXPECTED_AT_END_SITES` 7 → 8, and
grade it at required-zero"* — rather than branch 2 (`declined`).

### §0.4 What will be built

1. **`AdmissionPolicy::Nothing`** — the exact variant #3556 says cannot be
   built. `is_admitted_under` → `false`. `into_admit` splits on where the reason
   lives, which is the design statement this lane is making:

   * decode did **not** reach a grammar → return the **decode's** own `Block`
     (unchanged; the reason was never admission's);
   * decode read the body **whole** → return
     `Block::at_end(seg, ADMISSION_DECLINED)` — admission's own reason, and the
     8th `at_end` site.

2. **`ADMISSION_DECLINED`** as a fence-key constant in `func/body/mod.rs`
   beside the others, so `fence_site_census.rs`' per-key table enumerates it
   rather than it living unenumerated in `decode.rs`.

3. **`AdmissionPolicy::ALL` / `::name()` / `::index()`** — the settable-parameter
   surface, shaped after `Relax`, so an instrument can sweep the policy and name
   what it swept. `index()` is an exhaustive match, so a future variant does not
   compile until it is visited.

4. **`fence_site_census.rs`**: `EXPECTED_AT_END_SITES` 7 → 8, the new `EXPECTED`
   row, the headline `(20, 23)` → `(21, 24)` — **and a PARTITION** rather than a
   bare bump: the 8 sites split **7 in `func/census.rs` (a production scan
   reaches their keys) + 1 in `func/body/decode.rs` (instrument-only)**,
   asserted per file, with the guard's own rationale amended to say so. A bare
   `8` would hide exactly the hazard #3556 correctly identified.

### §0.5 THE HARD PROHIBITION, restated as a design property

This lane may not widen emission. `Nothing` is a refusal-direction instrument
state and **no production call site passes it**: `AdmissionPolicy::DEFAULT`
stays `RecognizedShape` and stays pinned by a test. The `RecognizedShape` arm of
both matches is textually unchanged. The admitted set is therefore identical by
construction, and it is checked per SYMBOL anyway (§1 P6).

### §0.6 The cost clause (#3336) — the axis this rung can fail on with 0 bytes moved

Named before starting, three of them:

* **A published key nothing measures** — the exact hazard #3556 names. Observed
  as: the 8th `at_end` site's key. Measured by: whether a test can fail on it,
  and by the file partition in §0.4.4 which refuses to let an instrument-only
  site be counted as a production one.
* **A denominator** — `fence_site_census.rs`' own headline `(keys, raises)`.
  Observed as: the pair moving by something other than exactly `(+1, +1)`.
* **The census-key population** — if `Nothing` were reachable by accident, a
  workload scan's key histogram would gain a row. Observed as: the per-symbol
  row dump and the `gap-metric` key set, both arms.

Throughput is **not** named as this rung's axis: `into_admit` gains a `match`
arm on a `Copy` enum with a constant discriminant, and `#3551` measured this
repo's build-to-build floor at `F = 0.93 %` with `w-unfuse`'s within-arm scatter
at 6–7 %. An instrument that cannot resolve what the change could plausibly cost
would abstain, which is what #3336 forbids calling a pass.

---

## §1 — PREDICTIONS, with confidence, frozen

**P0 — THE ANSWER TO THE LANE'S QUESTION, registered up front so the outcome
cannot be back-fitted.** The answer is **YES, the admission layer can own a
refusal reason**, and #3556 is **refuted as a general claim** and survives only
as a statement about which call sites select a policy. Registered at **0.90**.
(§0.2's two readings were taken before any code was written; the residual
uncertainty is that building it turns up a mechanism neither reading predicts.)

| # | prediction | P |
|---|---|---|
| P1 | identity diff of the 21 count-bearing gate rows, tip vs `base_c13cebbca.txt` = **0 lines** | 0.85 |
| P2 | `scripts/gate.sh` prints `GATE: PASS`, `mismatch 0` in every lane, the sweep and the cross; graded tree hash **equal at both ends of the run** and no movement line | 0.85 |
| P3 | **CONTROL A — the procedure can fail.** `scripts/gate_identity_diff.sh --self-test` reproduces `#3515`'s one-TU-refused signature at **exactly 14 lines / 7 rows** and exits nonzero on it | 0.92 |
| P4 | **CONTROL B — the seam is on a path the gate exercises.** With `AdmissionPolicy::DEFAULT` flipped to `Nothing`, **≥ 18 of the 21 rows move**, every mode lane's `match` goes to **0**, and `debug-lane`'s delta equals the sum of the 18 mode-lane deltas | 0.80 |
| P5 | `gap-metric` keys: the same set at base and tip, no key added, removed or moved | 0.85 |
| P6 | per-SYMBOL census row dump over all 878 TUs, both arms: **byte-identical after sorting**, `comm -3` empty in BOTH directions, and the row count is **> 2,000,000** (a denominator, because two empty dumps also compare equal) | 0.88 |
| P7 | `census/gate disagreement` = 0 at base and at tip; `match` = **25** at both arms (an INCREASE falsifies too — this lane may not widen) | 0.90 |
| P8 | no test green at base is red at tip; `cargo test -p c2-il` passing count moves only by the tests this lane adds, measured at base and at tip rather than inferred | 0.85 |
| P9 | `fence_site_census.rs` at tip: `at_end` = **8**, partitioned **7 census.rs + 1 decode.rs**; per-key headline `(21, 24)` | 0.90 |
| P10 | outcome word is **`built`** | 0.78 |

## §2 — FALSIFIERS

Any one of these and the lane says so in the rung, in the row that carries it:

1. the 21-row identity diff is non-empty;
2. `mismatch` > 0 anywhere — gate lanes, sweep, cross, or the 878-TU scan;
3. `census/gate disagreement` ≠ 0 at the tip;
4. any `gap-metric` key added, removed or moved;
5. a test green at base is red at tip;
6. `match` moves in **either** direction;
7. any per-symbol movement in the admitted set;
8. control B moves **fewer than 10** rows — that would mean `into_admit` is not
   on a path the gate exercises and the zero in P1 is vacuous (`#3346`);
9. the 8th `at_end` site turns out to have **no test that can fail on it** —
   that is `#3556`'s hazard arriving after all, and it is reported as such.

## §3 — THE DECLINE FLOOR

If any of the following holds, this lane reverts the `crates/` half and reports
**`declined`** with a two-sided price rather than shipping:

* the identity diff is non-zero and the cause is the new variant rather than an
  unrelated tree move;
* `Nothing` turns out to be reachable from a production call site by any route
  (that is a widening hazard in the refusal direction, and it is still a
  widening of the *decision surface* into production);
* the fence's partition cannot be stated in a way that fails — i.e. if the only
  way to take 7 → 8 is a bare constant bump that hides the instrument-only site.

And, per `w-unfuse`'s clause that actually fired: **if the second variant makes
some widening look free or obvious, STOP and report it as a finding with its
two-sided price** rather than taking it.

## §4 — PROCESS RULES THIS LANE BINDS ITSELF TO

* **Do not edit `crates/` while a gate is running.** `w-unfuse` did, and the
  gate's tree-movement guard voided the run (`GATE: PASS` at exit **1**). Read
  the verdict line AND the exit code AND the two tree hashes.
* Every git command uses `git -C <absolute worktree path>`.
* Board numbers stay inside `#3591`–`#3596`; if more are needed, STOP and report.
* Never push, never merge to master, never touch a peer's worktree.
* Files owned: `crates/c2-il/**` and
  `crates/c2-harness/tests/fence_site_census.rs`, plus `docs/`. Nothing else in
  `crates/`. A need for a peer's file is a STOP-and-report.
