# PREREG — lane `w-suitecost` (construct rung)

    Lane:      w-suitecost
    Branch:    wt-w-suitecost, off master e82c9ede6
    Kind:      construct rung — parallel test execution
    Fixtures:  none — construct rung: parallel test execution
    Census:    +0
    Frozen:    2026-08-19, BEFORE any measurement on this branch.
    Board:     block #3307–#3311 (allocated by the coordinator).

Worktree provisioned with `scripts/configure_existing_worktree.sh`
(`compilers/` symlinked, `target/` reflinked warm, `work/dc3-workload/`
reflinked). Verified: `c2rs census fixtures/cpp/w5_chain.cpp` → 4/4.

## The question

The workspace suite is the dominant per-lane cost. Cut its wall clock **without
narrowing what runs**. Success is graded by a **name-level identity diff** of
the executed-test set, not by counts.

## What the brief handed me, and what I will do with it

The brief supplies two back-to-back full-suite runs at master:

| | run A (gate concurrent) | run B (quiet box) |
|---|---|---|
| wall | 238 s | 402 s |
| Σ per-binary "finished in" | 236.5 s | 400.5 s |
| `cli_flags.rs` (18 tests) | 87.4 s | 148.5 s |
| `census_gate.rs` (2 tests) | 96.7 s | 100.2 s |
| `fixture_profiles.rs` | 29.8 s | 94.5 s |

These are **inherited, not mine**. Every number in the rung will be re-measured
on this branch and derived from logs kept under `work/w-suitecost/logs/`. If a
brief number does not reproduce I report it as a **dispatch defect**, not a
preamble.

## Hypotheses, in probability form

**H1 — cargo runs test binaries strictly sequentially.**
Predicate: over a full-suite run, `Σ(per-binary "finished in") / wall ≥ 0.90`.
`P(confirm) = 0.95`. Falsified if the ratio is < 0.75, which would mean cargo
already overlaps binaries and the whole lever is gone.

**H2 — a parallel binary runner reaches the max-single-binary floor.**
Predicate: with the runner at `N = 8` concurrent binaries, on the same tree and
the same box state as its own paired serial control,
`speedup = wall_serial / wall_parallel`.
Point prediction **2.5×**. 80 % interval **[1.8×, 3.4×]**.
`P(speedup ≥ 2.0×) = 0.75`. `P(speedup ≥ 1.5×) = 0.92`.
`P(speedup ≥ 4.0×) = 0.12`.
The floor is the largest single binary; I predict that binary remains
`cli_flags.rs` or `census_gate.rs`. `P(the critical-path binary is one of those
two) = 0.90`.

**H3 — the executed-test set is identical BY NAME.**
Predicate: the sorted multiset of `<binary>::<test name> <verdict>` from the
parallel runner equals that from `cargo test --workspace --release
--no-fail-fast`, exactly, with 0 names added and 0 removed, and identical
per-name verdicts.
`P(identical on the first attempt) = 0.60` — the named risk is cross-binary
interference through shared state (`work/capture-cache` in the **main** repo is
shared by every worktree; several test helpers key scratch dirs by pid only).
`P(identical after at most one round of isolating a shared path) = 0.90`.
**If I cannot make this exact, the lane reports FAILED.** A speedup with an
unproven name set is precisely the #3219/#3231 defect and is worth negative.

**H4 — mutant concurrency.**
Predicate: running `M` independent mutants `N`-at-a-time, throughput
`mutants/hour` vs `N`. Prediction: near-linear to `N = 4`, knee at `N = 4–8`,
`P(knee ≤ 8) = 0.80`. Predicted end-to-end campaign speedup for 21 mutants at
the best `N`: point **5×**, 80 % interval **[3×, 9×]**.
`P(≥ 3×) = 0.85`.

**H5 — `cli_flags.rs`'s cost is not flag parsing.**
Source-read hypothesis, formed from `crates/c2-harness/tests/cli_flags.rs`
lines 944–1046 **before** any timing: three of the eighteen tests
(`…_accepted_selftest`, `…_accepted_bench`, `…_accepted_perf`) each run a
**whole-corpus `c2rs` subcommand to completion as a subprocess**, solely to
assert that its argv parses. The file's own comment records 116 s of a 119 s
target from that cause and a 2026-08-08 split into four `#[test]`s so the
default intra-binary thread pool overlaps them.
Predicate: those three tests account for **≥ 70 %** of `cli_flags.rs`'s wall.
`P = 0.90`. `P(≥ 50 %) = 0.97`.
Corollary predicate: because they are already split, the binary's wall should be
≈ `max` of the three, not their sum; if the observed wall is instead ≈ their
**sum**, the split is not delivering and that is a finding.
`P(the split is NOT delivering, i.e. wall ≈ sum) = 0.35`.

**H6 — `fixture_profiles.rs`'s cost is one serial compile loop.**
Source-read hypothesis (lines 152–197): a single `#[test]` compiles **every**
fixture at its resolved profile, one `tc.compile_obj_flags` at a time, in one
thread. Predicate: ≥ 90 % of the binary's wall is inside
`every_fixture_compiles_at_the_profile_selftest_will_use`, and the loop is
serial. `P = 0.90`.

**H7 — the co-scheduling question (the brief's finding 2).**
The brief observed the **quiet** run 1.7× *slower* and proposed capture-cache
warmth created by the concurrent gate as the mechanism.
Predicate A: the suite's toolchain-heavy binaries read a cache that a
**concurrently running gate in another worktree** warms.
`P(the mechanism is a shared warm cache under the main repo's
`work/capture-cache`) = 0.35`.
`P(the mechanism is instead OS page cache / wibo+cl image warmth, i.e. any
recent heavy toolchain use warms it, gate or not) = 0.40`.
`P(the 1.7× is mostly box-load / measurement noise and does not reproduce as a
*reversed* ordering) = 0.35`.
(These are not exclusive; the probabilities are per-claim.)
Predicate B, the decisive one: run the suite **cold-vs-warm on the same box
state** — i.e. A/B the cache, not the neighbour. If a warm run is faster than a
cold run by ≥ 1.4× with no neighbour at all, the "co-scheduling is a lever"
reading is **wrong** and the honest statement is "warmth is a lever, and a
neighbour is one way to get it". `P(warmth alone reproduces it) = 0.55`.

## Controls (pinned BY NAME, not by count)

1. **Toolchain-liveness control.** `crates/c2-harness/tests/require_toolchain.rs`
   plus, by name, `census_gate::the_census_and_the_port_agree_about_what_is_in_class`
   — the test measured at 96–100 s. Any run in which that test's *duration* is
   < 10 s is a run with no live toolchain and its colour is **void, not
   provisional** (rungs/README rule 1). Every suite run in this lane records
   that test's duration.
2. **Name-set control.** The full sorted `<binary>::<name> <verdict>` list at
   the branch base, committed as an artifact, and re-diffed at the tip.
3. Every suite invocation carries `C2RS_REQUIRE_TOOLCHAIN=1`.
4. Box load (`uptime` 1-min average and `nproc`) is recorded beside every
   timing. Structural claims (Σ vs wall, ratios of paired runs) are preferred
   over absolute seconds throughout.

## Deliverables, and what counts as each outcome

- **`built`** — a parallel test-binary runner that (a) runs every binary the
  workspace suite runs, proven by name-level set equality of *binaries*, and
  (b) produces a name-level identical executed-test set with identical
  verdicts, at a measured speedup with its control.
- **`instrument`** — the runner does not reach a useful speedup but the
  diagnosis of the three heavy binaries lands with measurements.
- **`FAILED`** — no runner, or a runner whose name set I cannot prove equal.

**Required-zero byte delta**: no file under `crates/` changes behaviour. If the
lane touches `crates/` at all it is only to make an existing test cheaper
*without changing what it asserts*, and any such change is priced by the
name-level diff plus the gate. Default expectation: **`crates/` untouched**.

## What would make this lane FAIL, restated so I cannot drift

- Making the suite faster by running less of it. No test-impact narrowing ships
  in this lane. If I find one worth doing, it is written up under **Found and
  not taken**, not landed.
- Reporting a speedup without the by-name proof.
- Quoting seconds without box load.
