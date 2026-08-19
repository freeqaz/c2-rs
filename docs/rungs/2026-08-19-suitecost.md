# w-suitecost — the runner reaches the floor in every configuration, and the floor is one binary running six serial passes over the 386-fixture corpus

    Tag:       w-suitecost
    Slug:      suitecost
    Date:      2026-08-19
    Kind:      construct — parallel test execution
    Outcome:   built
    Fixtures:  none — construct rung: parallel test execution
    Census:    +0
    Record:    this file · prereg `docs/rungs/_2026-08-19-suitecost-prereg.md`
               · logs `work/w-suitecost/logs/` · runners `scripts/partest.sh`,
               `scripts/mutpool.sh`

## Headline

`scripts/partest.sh` runs the workspace suite with the **test binaries in
parallel**, and over nine serial/parallel pairs the executed-test set is
**identical BY NAME** — 1,682 results, same names, same verdicts, in two pairs
including a test that was **RED in both arms**. Six serial runs against four at
`--jobs 16`: median wall **211.0 s → 117.0 s = 1.80×**, or **2.04×** on the
load-robust statistic (effective parallelism, CPU-seconds per wall-second,
**6.54 → 13.34** of 32 cores). On the one adjacent pair taken with the box quiet:
207.6 s → 122.9 s.

And it is **at the floor already**. In all eight parallel configurations
measured, the run's wall equals the wall of its single largest test binary:

| config | wall | `cli_flags` alone | wall − `cli_flags` |
|---|---|---|---|
| `--jobs 8`, no LPT hint | 253.7 s | 252.8 s | 0.9 s |
| `--jobs 4` | 166.0 s | 157.9 s | 8.1 s |
| `--jobs 2` | 123.6 s | 108.5 s | 15.2 s |
| `--jobs 8` | 134.2 s | 133.2 s | 1.0 s |
| `--jobs 8 --test-threads 4` | 202.8 s | 201.4 s | 1.4 s |
| `--jobs 16` | 104.4 s | 103.6 s | 0.8 s |
| `--jobs 16` | 156.3 s | 155.3 s | 1.0 s |
| `--jobs 16` | 111.2 s | 110.4 s | 0.8 s |
| `--jobs 16` (quiet box) | 122.9 s | 113.2 s | 9.7 s |

There is nothing left in binary-level scheduling. **The remaining cost is inside
`cli_flags.rs`, and it is not flag parsing** — §3.

`crates/` is untouched. The whole lane is two files in `scripts/`, this rung, its
prereg, and lane scratch.

## 0. Box load, declared

Every wall-clock number here was taken on a box other lanes were using. The
coordinator declared a heavy window (a full suite plus a full gate in the main
repo) covering roughly 06:05–06:20, and `w-cfgclass` / `w-c2map3` were live
throughout; 1-minute load averages recorded beside the runs span **1.6 to 48**.
That is why this lane took **thirteen** suite runs rather than two, records
CPU-seconds beside every wall, and leads with ratios taken *inside* one run
(Σ-vs-wall, wall-vs-largest-binary) which a neighbour cannot move.

## 1. The dispatched findings, re-measured

**Finding 1 — cargo runs test binaries strictly sequentially. CONFIRMED, seven
times.** Σ of the 48 targets' own `finished in` values against the run's wall:

| run | Σ finished-in | wall | ratio | 1-min load at start |
|---|---|---|---|---|
| `serial-A` | 199.2 s | 200 s | 0.996 | 5.16 |
| `S1` | 282.6 s | 283.5 s | 0.997 | 2.44 |
| `S2` | 213.8 s | 214.4 s | 0.997 | 43.22 |
| `S3` | 196.1 s | 196.6 s | 0.997 | 14.75 |
| `S4` | 203.3 s | 203.8 s | 0.998 | 9.44 |
| `S5` | 234.1 s | 234.6 s | 0.998 | 24.27 |
| `SQ` | 207.0 s | 207.6 s | 0.998 | 15.17 |

The prereg registered `P = 0.95` on `ratio ≥ 0.90`. Within a binary libtest
threads; across binaries nothing overlaps at all.

The corollary the brief did not state, and it is the one that bounds everything
below: **a serial suite keeps 6.5 of this box's 32 cores busy.** CPU-seconds per
wall-second, over the six serial runs with CPU accounting: 5.34, 6.74, 6.50,
6.57, 6.31, 6.64.

**Finding 2 — "the QUIET run was 1.7× SLOWER; likely capture-cache warmth from
the concurrent gate; co-scheduling may be a lever." REFUTED, twice** — §4.

**Dispatch note on the brief's absolute seconds.** None of the brief's run-B
figures reproduce at this commit. Its quiet-box suite was 402 s; seven serial
runs here spanning load 2.44–43.22 read 200 / 283.5 / 214.4 / 196.6 / 203.8 /
234.6 / 207.6 s, and **the slowest is the one taken at the lowest load**. Per
binary, run B's `fixture_profiles` 94.5 s against a measured range here of
29.7–43.0 s; its `cli_flags` 148.5 s against 82.8 s. I cannot say what run B
was. The useful form of the observation is §4's: a two-sample wall comparison on
this box supports no directional claim, because wall alone has a ±20 % spread
that the 1-minute load average does not predict (`S1` is the extreme case — the
quietest start and the longest run).

## 2. What `partest.sh` is, and the proof that it does not narrow anything

**Enumeration is cargo's own predicate, not a hand list.** Targets come from
`cargo test --workspace --release --no-run --message-format=json` filtered on
`profile.test == true`. That has to be read out of the **profile object**: the
artifact JSON also carries `target.test`, which is `true` for the plain `c2rs`
binary cargo builds so `CARGO_BIN_EXE_c2rs` resolves and then never runs.
Matching the line as a whole yields **44** "targets" where cargo runs **43** —
an off-by-one in the flattering direction, since it would make the runner look
like it ran *more*. Doc-tests are not in that JSON and cargo does run them
(5 targets, one `ignored` doctest), so they are a separate pool job.
43 + 5 = **48 targets**, which is the baseline's count. The binary set was also
diffed by name against the `Running` lines of a real serial run: equal.

**The identity proof is by name.** Every run emits
`<target> :: <test name> :: <verdict>`, sorted. Nine pairs, each a serial cargo
run and a parallel run of the same tree:

| pair | config | results | identical? |
|---|---|---|---|
| `serial-A` / first `j8` | j8 | 1,682 | yes |
| `S1` / `P8a` | j8, no LPT hint | 1,682 | yes |
| `S2` / `P4` | j4 | 1,682 | yes, **including 1 FAILED** |
| `S2` / `P16` | j16 | 1,682 | yes, **including 1 FAILED** |
| `S3` / `P8b` | j8 | 1,682 | yes |
| `S3` / `P8t4` | j8, `--test-threads 4` | 1,682 | yes |
| `S3` / `P2` | j2 | 1,682 | yes |
| `S4` / `P16b` | j16 | 1,682 | yes |
| `S5` / `P16c` | j16 | 1,682 | yes |
| `SQ` / `PQ` | j16, quiet box | 1,682 | yes |

Zero names added, zero removed, zero verdicts changed, in every pair.

**The RED pair is the strongest row and it was an accident.** Writing this
rung's placeholder without regenerating `INDEX.md` turned
`rung_registry::rung_index_is_generated_and_current` red for four consecutive
runs of the campaign — two serial, two parallel. All four name lists agree on
**the failure as well as on the 1,681 passes**. Ten green pairs would not have
shown the runner can carry a red at all, which is precisely the #3219/#3231
shape: a clean suite with the right target count and the right exit code is
compatible with the catcher never having executed.

**Controls, pinned by name.** Every run records the wall of `census_gate`, the
binary holding `the_census_and_the_port_agree_about_what_is_in_class`. It read
64.8–102.3 s in every suite run and 71.3–375.3 s in every mutant run; a run in
which it is ~0 s had no live toolchain and its colour would be void, not
provisional. Every suite invocation carried `C2RS_REQUIRE_TOOLCHAIN=1`; the
worktree's `compilers/` is the symlink `configure_existing_worktree.sh` installs.

**Two scheduling facts fell out.** `--jobs 8` read 253.7 s on the campaign's
first parallel run and 134.2 s later: the first ran with no `durations.tsv`, so
the pool took cargo's alphabetical order and started the 40 %-of-the-suite
binary late. The runner now writes its own per-target durations each run and
sorts longest-first next time; the ordering is advisory and correctness never
depends on it. Second: **`--test-threads 4` is actively harmful** (202.8 s, the
worst of any parallel configuration) — it throttles `census_gate`'s wide lane,
which is the one test in the suite that is genuinely well parallelised.

## 3. What the cost IS — six serial passes over one 386-fixture corpus

`fixtures/cpp` holds **386** `.cpp` files. Per-test durations (libtest
`--report-time` under `RUSTC_BOOTSTRAP=1`, one binary at a time):

| binary | test | s |
|---|---|---|
| `cli_flags` (18 tests) | `…_accepted_selftest` | 77.7 |
| | `…_accepted_bench` | 77.7 |
| | `…_accepted_perf` | 41.1 |
| | **the other fifteen, together** | **0.72** |
| `census_gate` (2 tests) | `…_over_the_generated_corpus` | 67.4 |
| | `…_about_what_is_in_class` | 20.3 |
| `fixture_profiles` (3 tests) | `…_compiles_at_the_profile_selftest_will_use` | 43.0 |
| | the other two, together | 0.009 |

Those three binaries are **89.0 %** of a serial suite (177.3 s of 199.2 s,
`serial-A`). Read as sources rather than as a clock, they are the same shape six
times over — a **serial `for` loop over `all_fixtures()` that spawns the
toolchain at least once per fixture**:

1. `cli_flags::…_accepted_selftest` → `c2rs selftest` → `cli/reference.rs:288`,
   `for cpp in &targets { oracle_selftest(…) }`.
2. `cli_flags::…_accepted_bench` → `c2rs bench` → `cli/reference.rs:638`,
   **the same loop over the same corpus**. `cmd_bench` and a no-positional
   `cmd_selftest` differ in their report format and in nothing else, so one
   suite run does this pass **twice**.
3. `cli_flags::…_accepted_perf` → `c2rs perf` → `cli/perf.rs:28`,
   `for cpp in &targets { perf::bench_fixture(…) }` at `PerfConfig::default()`,
   whose `ref_iters` is **5** — about **1,930 standalone-`c2` replays**.
4–5. `census_gate::…_about_what_is_in_class` → `cross_check_par(…, 1)`, i.e.
   **`jobs = 1`**, run once packed and once `/Gy`: two serial passes.
6. `fixture_profiles::…_compiles_at_the_profile_selftest_will_use` →
   `for cpp in &fixtures { tc.compile_obj_flags(…) }`. Its `cpu/wall` is **0.97**
   in every repetition, which is what a single-threaded loop looks like from
   outside.

**The first three exist only to assert that a command line parses.** Their whole
assertion is `assert_ne!(out.status.code(), Some(2))` — see the roster comment
at `cli_flags.rs:928-1046`, which records that this was one `#[test]` costing
116 s of a 119 s target until 2026-08-08, split into four so libtest's own pool
would overlap them. **That split works**: the three run concurrently, so the
binary's wall is `max` (77.7 s) and not `sum` (196.5 s) — the prereg gave
`P = 0.35` to the split *not* delivering and that was correctly rejected. It is
also why the binary cannot go below ~78 s: the pool's floor is one whole-corpus
`oracle_selftest`, and there are two of those plus a whole-corpus latency
benchmark inside one test binary.

Two things this diagnosis kills that looked like levers before it:

* **`census_gate`'s `jobs = 1`.** It is real — the fixtures lane is genuinely
  serial where `cross_check_par` takes a `jobs` argument three lines away and
  the wide lane passes `min(nproc, 16)` — and fixing it is worth **zero**. The
  two tests run concurrently inside the binary and the *wide* lane is 67.4 s
  against the fixtures lane's 20.3 s, so the serial one is not on the critical
  path. Only measuring the tests separately showed this; the source alone said
  the opposite.
* **"`cli_flags` is 37 % of the suite for argument parsing."** True as
  arithmetic, wrong as a diagnosis. The fifteen tests that actually parse
  arguments — including the sweep that derives its table from the dispatch
  `match` and spawns 23+ processes — cost **0.72 s together**, and that sweep
  itself is **0.017 s**.

## 4. Deliverable 4 — co-scheduling is a confound, not a lever

The brief's mechanism was "run A rode capture-cache warmth the concurrent gate
was creating". Two independent refutations.

**A static bound, from the source.** The suite's *entire* exposure to the shared
`<main-repo>/work/capture-cache` is **one invocation**:
`c2rs gap --list <a file naming one fixture> --flags-file … --limit 1 --jobs 1`,
row 10 of `scripts_invocation_roster` in `cli_flags.rs`. It runs inside
`…_accepted_rest`, and that test — all **eight** of its invocations together —
takes **0.461 s**. Nothing else in the suite constructs a `CaptureCache` at the
shared root: `capture_cache.rs`'s tests each build one in their own temp dir,
and `census_gate` / `fixture_profiles` call `Toolchain::capture_il` and
`compile_obj_flags` directly, neither of which is cached. A mechanism worth at
most ~0.5 s cannot explain a 164 s difference.

**A measurement**, on `fixture_profiles` — the binary where the brief's anomaly
was largest (29.8 → 94.5 s, 3.2×, while `census_gate` moved 1.04× in the same
pair) and the cleanest possible probe because it is single-threaded:

| arm | walls (s) | median | CPU-s |
|---|---|---|---|
| alone, four in a row | 37.2, 32.7, 32.0, 31.2 | 32.4 | 30.7–36.1 |
| `C2RS_GAP_CACHE` = a **fresh empty dir** | 30.6, 31.7 | 31.2 | 30.3, 31.5 |
| beside a real `gate.sh --jobs 16 --require-graded` | 55.2, 33.7, 42.7 | 42.7 | 33.1–42.4 |

An emptied shared capture cache is **indistinguishable** from the warm one, as
the static bound predicts. A live neighbour makes the binary **1.32× slower**,
and costs CPU-seconds as well as wall (42.4 against 30.7 at the extremes) —
which is what contention looks like and what warmth does not.

The honest statement is the inverse of the brief's: **"measure it cleanly in
isolation" is the right instinct, and co-scheduling is a cost, not a lever.**
What the brief got right is that one A/B wall pair on this box is not evidence.

## 5. Deliverable 2 — concurrent mutant execution, and it bought nothing

`scripts/mutpool.sh`. A campaign is `M` × (rebuild + full suite), serial only
because the mutation is a real edit to `crates/` in one tree. Slots are
`setup_worktree.sh` worktrees — `compilers/` symlinked, `target/` **reflinked**
so a slot starts warm — and each pulls from one queue with a sliding worker, not
a batch barrier. The shared capture cache is deliberately *not* isolated:
`provenance::main_repo_root()` resolves every linked worktree to the main repo's
and concurrent same-key captures there are guarded by an `O_EXCL` lockfile, so
slots help each other. It schedules and records; it **classifies nothing**.

Five items — one unmutated baseline plus four mutants re-used verbatim from
`w-calleeguard`'s registered campaign, each anchor re-checked to resolve to
exactly one site. Their colours are on record and are **not** re-derived here;
what is measured is wall clock.

| campaign | slots | inner suite | campaign wall | per-mutant build | per-mutant suite |
|---|---|---|---|---|---|
| status quo | 1 | `partest --jobs 16` | **598.7 s** | 7.1–10.4 s | 102.8–121.5 s |
| N-at-a-time | 4 | `partest --jobs 16` | **626.6 s** (0.96×) | 5.8–24.6 s | 117.1–**493.7 s** |
| N-at-a-time | 4 | `partest --jobs 1` | **753.1 s** (0.80×) | 0.0–23.3 s | 231.6–**564.0 s** |

**H4 predicted 5× with `P(≥3×) = 0.85`. Realized 0.96× and 0.80×. Refuted.**

The mechanism is visible in the same table and does not depend on the box being
quiet. A mutant is **88–92 % suite and 8–12 % rebuild** — the rebuild after a
one-line `crates/c2-il` edit is 9–25 s against a 103–122 s suite, so there is no
independent "build leg" to overlap. And the suite, run by this lane's own
runner, already claims **13.3 of 32 cores**. Four slots therefore ask for ~53
cores on a 32-core box that peer lanes were holding 10–20 of, and the per-mutant
suite leg duly inflated **4.5×** (102.8 → 493.7 s) while the campaign wall
stayed flat. That is the arithmetic signature of a saturated resource, not of a
scheduling win.

The rule, stated so the next lane does not re-buy it: **choose the slot count
and the inner `--jobs` together, so that `N × eff-par(inner) ≲ free cores`.**
They are one budget. With `partest --jobs 16` (13.3 cores) `N = 1` already
saturates this box; with the serial cargo suite (6.5 cores) `N = 2` would be the
ceiling. The third campaign tests exactly that composition and lost anyway,
because it ran in the window where the 1-minute load average was 46.

**What is NOT load-confounded, and is the reason to keep the runner:** the
by-name result set is **identical between the N = 1 and N = 4 campaigns for all
five items, including which tests are red** — `M1` 4 catchers, `M2` 6, `M3` 2,
`M4` 3, baseline 0, 1,682 results each. Slot concurrency changes what a campaign
*costs* and provably not what it *measures*. `mutpool.sh` records that set per
run precisely so a campaign can be graded by name rather than by exit code,
which is the #3231 repair applied one level up.

## 6. Estimate vs outcome

| registered | predicted | realized | verdict |
|---|---|---|---|
| **H1** Σ/wall ≥ 0.90 | `P = 0.95` | 0.996–0.998, seven runs | confirmed |
| **H2** speedup, point | 2.5×, 80 % `[1.8, 3.4]` | **1.80×** wall, **2.04×** eff-par | at / just under the lower bound; the point estimate was optimistic |
| **H2** `P(≥ 2.0×) = 0.75` | — | 1.80× wall, 2.04× eff-par | straddles it |
| **H2** critical path is `cli_flags` or `census_gate`, `P = 0.90` | — | `cli_flags`, **9 of 9** configs | confirmed |
| **H3** by-name identity | `P = 0.60` first try, 0.90 eventually | identical on the **first** run and in all 10 pairs | confirmed; I was far too pessimistic |
| **H4** mutant concurrency ≥ 3×, `P = 0.85` | 5× point | **0.96×** and **0.80×** | **refuted** |
| **H5** the three subprocess tests ≥ 70 % of `cli_flags` | `P = 0.90` | **99.1 %** | confirmed |
| **H5** corollary: the 4-way split is NOT delivering | `P = 0.35` | it **is** — wall ≈ max(77.7) not sum(196.5) | correctly rejected |
| **H6** `fixture_profiles` is one serial loop, ≥ 90 % | `P = 0.90` | **99.98 %**, `cpu/wall = 0.97` | confirmed |
| **H7A** the shared capture cache is the mechanism | `P = 0.35` | ≤ 0.5 s of exposure; emptying it changes nothing | rejected |
| **H7B** warmth alone reproduces the 1.7× | `P = 0.55` | the neighbour is 1.32× **slower** | rejected |

Two misses worth carrying forward. **H2 landed at the bottom of its interval
while its structural half was exactly right**: the runner does reach
`max(binary)` in every configuration, and `max(binary)` is a larger share of the
serial total than the brief's arithmetic implied, because `cli_flags` grows
under co-scheduling (82.8 s alone → 103–133 s beside the rest) for the reason
§4 measures. **H4 was refuted outright**, and its error was the same one in a
different unit: the prereg's mutant model assumed the suite leaves the box idle,
which is true of the *serial* suite (6.5 cores) and false of the runner this
same lane built (13.3 cores). Predicting a second parallelism without first
measuring the first one's core draw is what produced a `P(≥3×) = 0.85` on an
outcome of 0.96×.

## 7. Gate evidence

| lane | result |
|---|---|
| `C2RS_REQUIRE_TOOLCHAIN=1 cargo test --workspace --release --no-fail-fast` | **1,681 passed, 0 failed, 1 ignored, 48 targets** — at the tip (`SFINAL`, 225.3 s, load 4.1), and its 1,682-name list is byte-identical to `SQ`'s |
| `scripts/partest.sh --jobs 16` | **1,681 passed, 0 failed, 1 ignored, 48 targets, 1,682 named results** (`PQ`, 122.9 s, load 3.1) |
| by-name identity, serial vs parallel | **10 pairs, 1,682 results, 0 added, 0 removed, 0 verdict changes** |
| `scripts/gate.sh --jobs 16 --require-graded` | **PASS** at the clean tip — 18/18 lanes, sweep **19,460 / 19,556**, cross **90,424 / 90,812**, **0 mismatch**, debug 18/18 at **6,948** fixture-verdicts, 0 panics; graded tree `ba1c880c30e4`; 147 s at load 15.6 (89.5 s at load 3.9 on the run before) |
| `status.sh --tests-log <parallel suite.log>` | `1681 passed, 0 failed, 48 targets` — the same row a serial run produces |

Identical to the dispatched baseline on every count.

**`crates/` is untouched** — `git diff e82c9ede6 HEAD -- crates/` is empty — so
the required-zero byte delta is satisfied by construction rather than by
measurement. The graded tree moves only by `scripts/` (#3215: graded-tree
identity binds revert-everything lanes; this one lands two scripts).

**`hatch-red` reads `REFUSED HATCH-STALE` in all three gate runs, including one
on a fully clean tip.** It is not this lane, and that is checkable rather than
asserted: `work/w-front3/hatch.py`'s `EDITS`/`RETIRED` tables name **only files
under `crates/c2-il/`** (`lib.rs`, `func/sy.rs`, `func/body/shapes/{assign,
calls,leaf_store}.rs`, `func/body/expr.rs`), and `git diff e82c9ede6 HEAD --
crates/` is empty, so `hatch.py apply` sees exactly the tree it sees on master.
Whatever verdict it produces here it produces there. Worth someone's five
minutes; it is not mine to claim.

## 8. Found and not taken

Ranked by what they would remove from **the floor**, which after this lane is
`cli_flags` at ~78 s alone and 103–133 s co-scheduled. Every one changes what a
test *costs*, never what it *checks*; none is test-impact narrowing; none is
taken here.

1. **Make `oracle_selftest`'s corpus loop concurrent inside `cmd_selftest` /
   `cmd_bench`** (`cli/reference.rs:288`, `:638`). Two of the three heavy
   `cli_flags` tests are that loop, 386 fixtures each, single-threaded, and
   `c2rs bench` is also a gate row and a `STATUS.md` row, so the win is
   collected several times per merge. Construct-rung-shaped: collect
   `SelfTestReport`s into a `Vec` indexed by fixture and print in the same order
   afterwards, so **stdout stays byte-identical** and the required-zero delta is
   checkable rather than argued. Estimated `cli_flags` floor 77.7 → ~41 s
   (bounded below by item 3), and the suite floor → `census_gate`'s 67.4 s.
   **Do not** do this to `c2rs perf`: it is a latency benchmark and threading it
   corrupts the number it exists to produce.
2. **`fixture_profiles`' compile loop** — 43.0 s single-threaded, 386
   independent `compile_obj_flags` calls, no shared state. Worth ~40 s of
   *CPU-schedulable* work and **0 s of floor**, because it is not the critical
   path. Take it only after item 1 and only if it is critical then. Sort the
   failure list before the assert or the panic text stops being deterministic.
3. **`c2rs perf` inside `cli_flags`** is 41.1 s and ~1,930 standalone-`c2`
   replays to establish that eleven characters of argv parse. The obvious repair
   — pass `--fixtures add3.cpp` — **narrows what the roster tests** (its
   contract is *every invocation `scripts/` makes*, and `scripts/` makes the
   bare one), so it is unavailable. What is available is an `ref_iters`
   environment override the roster test sets and the standing benchmark does
   not, which needs pricing against the trap of a benchmark configured
   differently in the suite than on the command line.
4. **`census_gate`'s `cross_check_par(…, 1)`** — serial where the machinery for
   `jobs > 1` is three lines away. Zero floor value today (§3); worth doing only
   as a consequence of item 1 changing which binary is critical.
5. **Wire `status.sh`'s no-log path to `partest.sh`.** `collect_tests`
   (`scripts/status.sh:363`) runs the serial suite for `STATUS.md`'s `tests`
   row. `partest.sh` now emits `suite.log` in cargo's shape and `status.sh
   --tests-log` was verified to read it, so the wiring is one line — left out
   because it moves a gate-adjacent default and deserves its own required-zero
   diff.
6. **The batch barrier in `gate.sh` and `expr_sweep.sh`.** Both fan out with
   `if running >= jobs; then wait; running=0`, which stalls every worker until
   the slowest member of each batch finishes. It is the only pool a `#!/bin/sh`
   file can express, which is why `partest.sh` and `mutpool.sh` are bash.
   `gate.sh`'s lane leg is 2 s at `--jobs 16` and has nothing in it (measured
   2026-08-08); the row that could matter is `expr_sweep.sh` over 19,556 cases.
   **Unmeasured here** — the barrier's cost is a function of the variance of
   per-case durations, which this lane did not measure, and a ranking taken
   without it would be the artifact `ranking-instruments-measure-themselves`
   describes.
7. **A quiet-box mutant campaign.** §5's null is real in its mechanism and
   load-confounded in its wall clock. The measurement that would settle the size
   of the prize is `N = 1` and `N = 4` with the *serial* inner suite, back to
   back, on a box with no peer lanes — about 40 minutes, and worth it only if
   somebody is about to run a 21-mutant campaign.
