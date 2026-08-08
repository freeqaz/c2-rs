# Merge-verification cycle: measured findings and prioritized proposals

Investigation of the per-merge re-gate cycle, 2026-08-08, master `cc9a56fb`.
Every number below was **measured on this box**; each timing says whether the
box was idle. No file under `scripts/` or `crates/` was modified. The
measurement runs used private outdirs (`/tmp/c2rs-sweep-fable*`, since
removed) and ordinary `gate.sh` invocations whose run trees the reaper
manages like any other.

---

## 0. Where the cycle's time actually goes (measured)

The serial cycle on the merge that was in flight when this investigation
started (`work/regate-w-lineage2.txt`, tree `cc9a56fb`, box carrying light
lane load):

| leg | wall | of which |
|---|---:|---|
| `cargo test --workspace --release` | ~300s (sum of 36 targets: 299.23s; cargo runs test binaries **serially**, wall ≈ sum) | `census_gate` 155.66s + `cli_flags` 125.71s = **94 %**; next largest 6.62s |
| `gate.sh --require-graded` (no `--jobs`!) | ~300s | lanes 2s · **sweep 266s** · cross 27s · pin/preflight/reap remainder |
| `status.sh --write` (when run) | ~380s+ | it re-runs `cargo test --workspace --release` in `collect_tests` (~200-300s), plus selftest 44s + perf 29s + fixture gate; the 878-TU scan itself is **2.1s warm** on the capture cache |
| `rung_registry` test + `board_audit.sh` | seconds | — |

Same legs re-measured on an **idle** box: `cargo test` 203s (sum 202.15s;
`cli_flags` 117.90s, `census_gate` 66.87s, everything else ≤ 6.43s).
So the "20-40 min per merge" is ≈ 10 min of serial compute inflated by
cross-lane load, plus post-merge rebuild, plus `status.sh` when run.

### Where the sweep leg's time goes

Each sweep case is one `c2rs diff` (`crates/c2-harness/src/cli/reference.rs`
→ `differential()` in `crates/c2-harness/src/lib.rs`): a full `cl /Bd`
pipeline capture **under strace** + a standalone-c2 replay under wibo + the
port compare. Two wibo process trees per case, **no capture-cache use on this
path at all** (deliberate — board #282: only `diff` asserts the reference
replay is byte-exact). It is CPU/process-spawn bound at ~0.05-0.07 s·core per
case, embarrassingly parallel (per-case scratch dirs, nothing shared), and
was measured to scale almost linearly to 24 workers:

| `C2RS_SWEEP_JOBS` | full-corpus sweep wall (idle box, incl. ~12s generation+pin) |
|---:|---:|
| 4 (the default the gate exports) | 266s (in-gate, light load) |
| 16 | 82s (in-gate, idle) |
| 24 | **68s** (standalone, idle) |
| 32 | 70s (standalone, idle) — **the knee is ~24** (32 logical cores, likely 16 physical) |

All four runs printed identical counts: `checked=19556 mismatches=0
graded=19460 ungraded=96 unknown=0`. The parallel split is an equivalence,
not an approximation — same discipline the gate header already records for
4→8.

---

## 1. Pass `--jobs 16` to every gate invocation — **the single biggest lever, zero code**

**Finding:** `jobs=4` (gate.sh line 347) and `C2RS_JOBS:=8` (line 2612) both
date to the gate's **creation commit** `25def085` (2026-07-31) and have never
been touched: they are untuned constants, not safety limits. No board row
explains them. STATUS.md's own reproduction table says `gate.sh --jobs 8`,
but the observed merge cycle ran `scripts/gate.sh --require-graded` with no
`--jobs`, so the sweep and the cross both ran at **4 workers on a 32-core
box**.

**Measured:** full gate `--jobs 16 --require-graded`, idle box: **107s
total** (lanes 1s, sweep 82s, cross 23s) vs ~300s at defaults — and the
verdict block is numerically identical (5,184 fixture-verdicts, sweep
19,460 graded, cross 90,424 graded, 0 mismatch).

**Expected saving:** ~200s per merge (~300s → ~107s on this leg).

**What breaks first at higher concurrency — checked, not assumed:**
- **Inodes:** in-flight low-water draw was 19,810 inodes at jobs 4 and
  19,885 at jobs 16 — *concurrency does not scale the draw*; the generated
  corpus (19.6k cases, written at either setting) does. /tmp had 733k free
  against a 150k floor, and the preflight + reaper already guard this
  fail-red (`GATE: FAIL (DISK)`, exit 3, distinct from a mismatch).
- **wibo under concurrency:** board **#201** measured 48 *concurrent*
  captures → one distinct `.gl` sha, and censuses at jobs 1/14/32
  byte-identical. `census_gate` already runs 2×16 capture threads on every
  merge. 16-24 concurrent wibo trees is proven territory.
- **Capture cache:** per-key `O_EXCL` lockfiles since board #181,
  fail-open; no global serialization. (The sweep leg does not touch it
  anyway.)
- **Memory:** 24 concurrent cl trees drew no visible pressure on a 93 GB
  box (57 GB available).

**False-green analysis:** none new. The sweep's reconciliation is positive
(`checked == run`, per-worker count files; a worker killed at higher
concurrency contributes a *short* count and the run FATALs red, never
green). The failure direction of over-concurrency is false **red** (disk),
already discriminated by the DISK banner.

**Effort:** zero for "type `--jobs 16`". One line if you want the default
changed in `gate.sh` (with the header's cost table updated — it still
records the 14,484-case/8-jobs numbers). Recommend 16 as the new documented
invocation; 24 buys ~14s more on an idle box and nothing at 32.

---

## 2. Split `cli_flags`' one 116-second test into three — ~70s per merge

**Finding:** `cli_flags` (117.90s idle) is 36 targets' second-largest cost,
and ~116s of it is **one test**,
`every_invocation_the_scripts_make_is_still_accepted`, which runs bare
`c2rs selftest` (44s), `c2rs bench` (43s) and `c2rs perf` (29s) **serially
in one #[test]** — three whole-corpus commands, executed to completion, to
assert only that their argv **parses** (`exit code != 2`).

**Proposal:** split that one test into three (or four) `#[test]`s holding
the identical command lines and the identical assertion. The default
parallel test harness then runs them concurrently: target wall ≈ max(44s) +
the small tests ≈ ~50s. Nothing is removed from what runs; every invocation
and every assertion is preserved verbatim.

**Expected saving:** ~70s per merge (118s → ~50s), derived from the
measured per-command times above.

**Risk / false green:** none in the grading sense — the assertions are
unchanged and each still names its own command. The three commands use
pid+time-keyed scratch dirs, so concurrent execution does not share state.
The only behavioural difference is load during the test, which no assertion
in the file measures.

**Effort:** small, one test file. (A cheaper-still variant — asserting parse
acceptance without waiting for the corpus run to finish — would change what
the test exercises, so I am not proposing it.)

---

## 3. Run the 36 test binaries concurrently — test leg 203s → ~120s (~70s with #2)

`cargo test --workspace` builds in parallel but runs test binaries
**serially**: measured wall 203s ≈ sum of targets 202.15s. The binaries are
independent processes with pid-keyed scratch; `census_gate` and `cli_flags`
between them use ≤ ~35 threads, so two-at-a-time on 32 cores loses little
per-target.

**Proposal:** a `scripts/` wrapper (std tools only): `cargo test --workspace
--release --no-run`, enumerate the test executables from cargo's JSON
messages, run them concurrently at a small bound (4-6), and **re-derive the
verdict positively**: require exactly N `test result:` lines for N
enumerated binaries, 0 `failed`, and a summed pass-count floor — the same
compare-a-count discipline as everything else here. Keep `status.sh`'s own
serial `collect_tests` untouched (it parses its own log).

**Expected saving:** wall → max(target) + build ≈ 120s today, ≈ 70-80s once
#2 lands. Derived: the runtime sum is 202s but the max target is 118s (50s
after #2).

**Risk / false green:** the real hazard is instance #17 of *absence read as
success* — a binary the wrapper never launched. That is exactly what the
positive count closes: the expected N comes from cargo's own artifact list
for *this* build, not from a constant, and a missing or extra `test result:`
line fails. A second hazard — two targets interfering — is bounded by what
already happens (`census_gate` runs 32 capture threads inside one target
today); any interference failure is a red, not a green.

**Effort:** medium (a ~100-line shell/python script plus adopting it in the
merge ritual). Do #1 and #2 first; #3 is worth it only if the ~2 min test
leg still bites after that.

---

## 4. Docs-only merges: a *sound* skip exists, with a printed provenance line

Two of today's merges changed no file outside `docs/` and still paid the
full cycle. The question is whether "a merged tree is a new configuration"
forces a re-gate anyway. **It does not, when the diff is confined to paths
the graded computation provably never reads** — the gate is a function of
(binary ← `crates/`+`Cargo.*`, `scripts/`, `fixtures/`, toolchain, /tmp),
and the toolchain is outside the tree.

**The input closure, derived from the tree** (this is the part to re-check,
not assume — I grepped, and it is wider than "crates/"):

- workspace tests read `crates/`, `fixtures/`, `scripts/lanes.txt`
  (lane_registry), `scripts/sweep.d` + `sweep_gen.py` (sweep_registry),
  **`docs/rungs/` (rung_registry)**, and — the surprise —
  **`work/w-inl0/cells/*.cpp` via `include_str!`** in
  `dead_temp_elision.rs` (and siblings). `work/` is *in the closure*.
- `gate.sh` reads `scripts/` + `fixtures/` + the pinned binary.
- `board_audit.sh` and `rung_registry` read `docs/`.
- No non-test crate `include_str!`s anything outside `crates/`; there are
  no `build.rs` files.

**Proposal — the rule:** if every path in `git diff --name-only
<gated-base>..<merged>` is under `docs/`, skip `cargo test --workspace` and
`gate.sh`, and run only the legs whose inputs changed: `cargo test -p
c2-harness --test rung_registry`, `scripts/board_audit.sh`, and (if
reporting) the status regen. Anything touching `crates/`, `scripts/`,
`fixtures/`, `Cargo.*`, or `work/` ⇒ full cycle, no exceptions. Print the
decision loudly: `SKIP-REGATE: diff vs <base> (green gate sha <s>) confined
to docs/ — N files: …` — a skip nobody can see is this project's oldest
defect.

**Why it is sound:** the skipped legs would re-execute a byte-identical
computation on byte-identical inputs — the configuration *is* covered, by
the base's green run. Capture determinism is not assumed: board #201
measured it.

**False green, and the check that closes it:** a *future* test that starts
reading a docs/ path would silently widen the closure. Close it at use
time, not by memory: the skip script re-derives the closure each run —
`grep -rl 'docs/' crates/*/tests/` must return exactly the allowlisted
consumers (today: `rung_registry.rs`), and any new hit refuses the skip.
Second hole: a merge that changes the *base* you diff against to one that
was never gated — so the rule must name the base's green gate log (its
pinned-binary sha) and verify that log exists and says PASS, not trust the
ref name.

**Expected saving:** the full ~10 min on each docs-only merge (two today).

**Effort:** small script + merge-ritual discipline.

---

## 5. A 7-second fail-fast pre-gate already exists in the gate's own flags

**Measured, idle box:**

    scripts/gate.sh --jobs 16 --sweep-cases 400 --cross-cells 2000 --require-graded
    → 7 seconds: 18/18 lanes FULL (5,184 verdicts), 400-case strided sweep,
      2,000-cell strided cross, verdict `GATE: PASS (SAMPLED)`

The stride is by design representative (46 of 47 fragments at 400 cases; a
prefix would be blind — the machinery from board #232's postmortem), and a
sampled run **cannot print an unqualified PASS**, so it cannot be banked as
a full gate by anything reading the output. Use it immediately after the
merge commit to fail fast on gross breakage (the fixture lanes are complete,
not sampled), then run the full `--jobs 16` gate exactly as now before
pushing.

**Risk / false green:** none, structurally — it is additive, and the
SAMPLED verdict is the mitigation. The one discipline it needs: never let a
SAMPLED line be quoted as the merge's gate. Effort: zero.

---

## 6. Things measured and **declined**, so they are not re-proposed blindly

- **Caching the sweep's reference captures** (wiring `differential()` to
  `CaptureCache`): would cut the sweep to ~30-40s warm, but the cache key
  (`capture_cache.rs::new`) covers toolchain digests, wibo version, tree
  token and cache root — **not the c2-reference capture *code***. A merge
  that changes how `cl` is invoked would then be graded green against
  captures made by the old invocation: a real false-green hole, in the one
  leg whose distinct job is asserting the capture/replay mechanism itself
  (board #282). After #1 the sweep is 68-82s; not worth that hole.
  (`--validate-cache` would catch capture-code drift only at its sampling
  rate, and its in-place re-capture blind spot is already on record.)
- **Overlapping `cargo test` with `gate.sh`:** structurally safe — the gate
  runs a run-private pinned binary (`harness_bin.sh`), run dirs are
  pid-keyed, cargo target-lock contention just queues, the capture cache is
  per-key locked. But after #1-#3 the two legs are ~107s + ~120s, both
  partially saturating the box; overlap saves at most ~1 min, muddies any
  timing you might want to quote, and makes a red harder to attribute
  (board #294's shape). Serial is the right default; overlap only when the
  box is otherwise idle and nothing in the run will be quoted as a timing.
- **`status.sh --write` on every merge:** it re-runs the entire workspace
  test suite inside `collect_tests` and takes its perf geomean under load
  (a documented misreading trap). Keep it for *reporting*, not for gating —
  the scan it uniquely contributes is 2.1s warm. Do not teach it to consume
  a prior test log; a stale-log false green is worse than the duplication.
- **Trimming corpora / narrowing lanes:** not examined further; ruled out
  by constraint 1.

---

## 7. The cycle after #1 + #2 + #5 (no code beyond one test-file split)

| step | wall (idle) |
|---|---:|
| pre-gate (sampled, fail-fast) | 7s |
| `cargo test --workspace --release` | ~135s (203 − ~70 from #2) |
| `gate.sh --jobs 16 --require-graded` | ~107s |
| `rung_registry` + `board_audit.sh` | ~10s |
| **total per merge** (+ post-merge rebuild, which varies with the diff) | **~4-5 min** |

vs ~10 min serial compute today (and 20-40 min observed under cross-lane
load). With #3: ~3.5-4 min. Docs-only merges: under a minute via #4.

The honest closing note: after #1 the binding constraint is no longer any
single leg but the *sum* of ~100-second legs, each individually justified
and none redundant with another — `census_gate`'s captures are not the
sweep's diffs are not the cross's cached gap cells (board #282 names the
one real overlap and why it is not one). Further gains past #3 mean either
overlap (declined above, revisit if lane count grows) or caching with the
provenance hole named in §6 — there is no free leg left to delete.
