# w-throughput — the merge-verification cycle itself: the gate's default concurrency was an untuned constant, `cli_flags` was one 116-second `#[test]`, and the docs-only skip fires on ZERO of the last 40 merges

    Tag:       w-throughput
    Slug:      w-throughput
    Date:      2026-08-08
    Fixtures:  none — this lane changes the verification cycle, not an accepted class
    Census:    711,486 / 2,463,443 unchanged (28.88 %), **+0** — this lane ships
               no `crates/` behaviour; the only `crates/` file it touches is a
               test. TU match **11 → 11**, mismatch **0 → 0**.
    Record:    this file; the specification is `work/fable-perf/PROPOSAL.md`,
               committed by this lane because it was written under a gitignored
               path and would otherwise have been lost.
    Lane:      w-throughput, worktree branch `wt-w-throughput` off master
               **`f49fe5e1`**.
    Ships:     `scripts/gate.sh` default `--jobs` **4 → 16**; the `cli_flags`
               mega-test split into **four** `#[test]`s over one roster, plus a
               partition control; `scripts/status.sh --tests-log`, gated on four
               positive checks. Board rows **#1323**–**#1330**; **#1331** and
               **#1332** are deliberately UNMINTED.

---

## 0. The rung, and the one thing it may not do

The rung is the **verification cycle**, not the port. Every change here is an
argument that a specific cost is paid for nothing. The constraint that bounds
all of it: **nothing may reduce what is graded.** Real `c2.dll` plus a byte-exact
obj compare remains the sole judge, and the three items that shipped are checked
against that by showing the verdict counts identical before and after — a
concurrency change that quietly dropped cases would be instance seventeen of
*absence read as success*, which is the defect this project has paid for more
than any other.

**The box was contended for the whole session.** Two other lanes were gating
concurrently; `uptime`'s 1-minute load average ranged **4.5 to 99** on 32 logical
cores. Every timing below says which. The one number where it mattered — the
gate A/B — is therefore taken as a **same-tree, same-cache pair** rather than as
base-vs-tip, for the reason in §2.3.

---

## 1. Results

| | base `f49fe5e1` | tip |
|---|---|---|
| `cargo test --workspace --release` | **206 s** · 1,159 passed / 0 failed / **36 targets** | TIP_TEST |
| — of which `cli_flags` | **119.08 s** | TIP_CLIFLAGS |
| — of which `census_gate` | 68.46 s | TIP_CENSUSGATE |
| `git grep -c '#\[test\]' -- crates` | **1,160** (1,159 anchored) | **1,168** (1,163 anchored; the loose count includes six prose mentions of `#[test]` in the new comment block) |
| `scripts/gate.sh --require-graded` | GATE_BEFORE | GATE_AFTER |
| 878-TU scan | **match 11, mismatch 0, codegen-gap 0, vocab-gap 860, capture-fail 7**, 139 `gap-metric` lines | **every digit unchanged** — see §9 |

---

## 2. Item 1 — the gate's default `--jobs`, 4 → 16

### 2.1 It was never a decision

`jobs=4` (line 347) and `: "${C2RS_JOBS:=8}"` (line 2612) both date to
`scripts/gate.sh`'s **creation commit** `25def085` (2026-07-31). Verified rather
than assumed:

    $ git log -L347,347:scripts/gate.sh --oneline | grep -E '^[0-9a-f]{7,} '
    25def085 harness: enumerate the mode lanes, because an un-enumerated lane does not run
    $ git blame -L347,347 -- scripts/gate.sh
    25def0854 (freeqaz 2026-07-31 18:58:37 +0000 347) jobs=4

One entry each in their line histories, and `grep -n 'jobs=4\|C2RS_JOBS' docs/BOARD.md`
returns **nothing** — no board row explains either number. They are untuned
constants on a 32-core box, not safety limits.

`jobs` governs three legs: lane parallelism, `C2RS_SWEEP_JOBS` for the generated
sweep, and the `C2RS_JOBS` handed explicitly to `mode_cross.sh`. All three are
embarrassingly parallel over per-case scratch directories.

### 2.2 The verdict is an equivalence, not an approximation

EQUIV_BLOCK

### 2.3 Why the A/B is same-tree and not base-vs-tip

`mode_cross.sh` keys the capture cache on the **source path**, so it keeps a
stable case directory at `<repo_root>/work/mode-cross/cases`. In a worktree that
directory starts empty — `scripts/setup_worktree.sh` copies only
`work/dc3-workload/` — so the **first** cross in a new worktree is cold. The
script's own comment prices that at **5 min 45 s cold vs 13.8 s warm** (25×) at
8 jobs over 61,539 cells; this gate's cross is **90,812** cells.

So a base-vs-tip gate comparison out of a fresh worktree measures the cache, not
the concurrency. The lane's baseline run confirmed it the expensive way: at
`--jobs 4` with a cold cross on a box that reached load 99, it was **killed at
1,324 s with 8 of 18 cross lanes done**, having already produced the sweep leg.
The pair that is actually an A/B is `--jobs 4` vs `--jobs 16` **on one tree with
the cache warm**, and that is the pair quoted above. The sweep leg is comparable
at both ends regardless — it drives `c2rs diff`, which does not consult the
cache at all (board #282).

### 2.4 Why 16 and not the knee at 24

The proposal measured the knee at ~24 (68 s standalone) with 32 giving 70 s.
16 is chosen anyway, and the reasons are ranked:

1. **The marginal is tiny and the risk is not.** 4 → 16 buys ~180 s on the sweep
   leg. 16 → 24 buys ~14 s more, on an idle box. 32 buys nothing at all.
2. **16 is the physical core count** (32 logical). Past it each worker is a whole
   `cl` + wibo process tree contending for a hyperthread sibling.
3. **This box does not run one gate at a time.** Two other lanes gated
   throughout this session, and three concurrent gates at 24 is 72 capture trees
   on 32 cores, a regime in which the 68 s does not hold. 16 is the value that
   is fast when the box is busy and within 20 % of the best when it is idle.
4. **A fixed number is quotable; an adaptive one is not.** `nproc`-derived
   defaults were considered and rejected: the gate's cost is quoted across boxes
   and across months in this repo, and a default that silently differs per host
   makes two runs incomparable without saying so.

`--jobs` remains overridable and the header line still prints the value actually
used (`wall clock: Ns for 18 lanes at --jobs J (C2RS_JOBS=K)`), so a run's own
output says what concurrency produced it.

### 2.5 `C2RS_JOBS` is deliberately NOT raised, and that is a measurement

`C2RS_JOBS` is read by `mode_lane.sh` only (`c2rs gap --jobs`) — `mode_cross.sh`
receives `C2RS_JOBS="$jobs"` explicitly and never sees the default. So it is the
**per-lane** thread count *inside* each of the `$jobs` lanes running at once, and
it **multiplies**: raising `jobs` 4 → 16 already took the lane leg from 4×8 to
16×8 concurrent capture threads.

And the leg it governs is not where the time is. Measured on this box today:
the lane leg is **16 s** of a ~1,300 s gate at `--jobs 4`, and LANE_LEG_16 at
`--jobs 16`. There is nothing left in it to win, and raising a second
concurrency knob in the same commit would make the next timing unattributable.
It stays 8 and stays overridable.

### 2.6 Inode headroom at the new default

The concurrency does **not** scale the in-flight inode draw; the corpus does.
The proposal measured 19,810 at 4 workers and 19,885 at 16. Confirmed at the
chosen value on this box: INODE_BLOCK

The floor is 150,000 (`C2RS_GATE_MIN_INODES`), which is 3× the measured peak
draw of a run in flight, and the preflight fails **red and distinctly** below it
(`GATE: FAIL (DISK)`, exit 3, not the exit code of a mismatch).

---

## 3. How this could produce a false green, and the check that closes it — VERIFIED BY KILLING A WORKER

The hazard raising concurrency introduces is a worker that dies and takes its
cases with it, leaving a **short corpus** reported as a pass. The proposal
asserts this fails red by short-count reconciliation. Asserting it is not
enough here — the project has sixteen instruments that were green over a
population they could not reach — so it was **made to happen**.

`work/w-throughput/kill_a_worker.sh` runs `expr_sweep.sh` at 16 workers, waits
for the fork loop to have produced `parts/chunk.15`, and SIGKILLs one worker
subshell **and its `c2rs diff` child**. The victim is found by walking children
of a PID the script launched (`pgrep -P <numeric pid>`), never by a `pgrep -f`
pattern that could match the script's own argv.

    == part A: expr_sweep.sh at 16 workers, one killed mid-chunk ==
       sweep pid 3884241 has 16 worker subshells
       killing worker 3886602 (and its own children) with SIGKILL
       expr_sweep.sh exit status: 3
    -- the last lines of its output --
    checked=734 mismatches=0 graded=732 ungraded=2 unknown=0
    FATAL: selected 783 cases and only 734 were graded
      15 of 16 workers reported a count. A short count is a worker
      that died; the cases it held were never graded and this run establishes
      nothing about them.

**`mismatches=0` over a short corpus is exactly the shape that would have been
a false green, and it exits 3.** Two independent mechanisms produce the red and
neither is an exit status: `expr_sweep.sh` compares its summed `checked` against
the `run` it selected, and `gate.sh`'s own `sweep_verdict` re-derives the same
comparison from the log (`FAIL|…|SHORT — selected R cases, reached C`), so a
sweep that died without exiting non-zero is still red.

PART_B_BLOCK

---

## 4. Item 2 — `cli_flags` was one 116-second `#[test]`

### 4.1 What it was

`cargo test` runs its 36 test binaries **serially**, so the workspace wall is
the *sum* of the targets. `cli_flags` was 119.08 s of a 206 s leg, and ~116 s of
that was one test — `every_invocation_the_scripts_make_is_still_accepted` — which
ran `c2rs selftest` (~44 s), `c2rs bench` (~43 s) and `c2rs perf` (~29 s) to
completion, serially, inside one `#[test]`, in order to assert that their argv
**parses** (exit code ≠ 2).

### 4.2 What shipped, and what did not change

Four `#[test]`s — `..._selftest`, `..._bench`, `..._perf`, `..._rest` — with the
**same eleven command lines** and the **same assertion**, verbatim. Nothing is
removed; the three whole-corpus commands still run to completion. The default
parallel harness overlaps them, so the target's wall becomes ≈ max(44 s) rather
than sum(116 s).

**The split is a partition of ONE roster.** Four hand-copied lists would be four
places to lose an invocation in, and a lost invocation is a check that silently
stops running. So there is one `scripts_invocation_roster()`, each row tagged
with its group; `the_split_is_a_partition_of_the_roster` pins the roster at 11
rows and pins each group's share; and `accepted_group` asserts `ran > 0` so a
filter that matches nothing is a failure rather than a vacuous pass.

### 4.3 Must-fail evidence — three mutations, three distinct messages

Each of the three expensive invocations was mutated **separately** to an argv the
parser refuses, because `docs/GAPS.md` records a case where an early guard made
later assertions unreachable and the demonstration "passed while demonstrating
nothing". Full transcripts in `work/w-throughput/cli_flags_mutations.txt`; the
distinguishing lines:

    test ..._accepted_selftest ... FAILED
      assertion `left != right` failed: `c2rs selftest --no-such-flag` is an
      invocation `scripts/` makes and the parser REFUSED it. …
      stderr: selftest: unknown option: --no-such-flag

    test ..._accepted_bench ... FAILED
      assertion `left != right` failed: `c2rs bench --jobs 4` is an invocation
      `scripts/` makes and the parser REFUSED it. …
      stderr: bench: unknown option: --jobs

    test ..._accepted_perf ... FAILED
      assertion `left != right` failed: `c2rs perf --no-such-flag` is an
      invocation `scripts/` makes and the parser REFUSED it. …
      stderr: perf: unknown option: --no-such-flag

**Independence, which is the part the split could have broken.** With only the
first mutation applied and all four group tests run together:

    test ..._accepted_selftest ... FAILED
    test ..._accepted_rest ... ok
    test ..._accepted_bench ... ok
    test ..._accepted_perf ... ok
    test result: FAILED. 3 passed; 1 failed; … finished in 190.64s

Exactly one red. Note what that says about the *old* shape: when these were one
`#[test]`, the `for` loop aborted on the first failure and every invocation after
it went unchecked. The split yields **more** evidence per red, not less.
(190.64 s was measured while this lane's own baseline gate was saturating the
box; it is not a timing.)

**The roster control, mutated too** — deleting one invocation rather than
breaking it:

    test the_split_is_a_partition_of_the_roster ... FAILED
      assertion `left == right` failed: the roster held 11 invocations when it
      was one test; it holds 10. An invocation removed from here stops being
      checked at all, and nothing else in this file would notice.

---

## 5. Item 3 — `status.sh` re-ran the whole suite, and now need not

`collect_tests` runs `cargo test --workspace --release` **inside** the status
regen — 206 s idle, ~300 s under load — for a report whose other unique
contribution, the 878-TU scan, is **2.1 s warm**. A merge ritual that runs the
suite and then `status.sh --write` pays for it twice.

`scripts/status.sh --tests-log FILE` reads the row out of a log the caller
already produced. The obvious version of that is a false green with this
project's own name on it, so the reuse path is gated on **four positive checks**,
each rendering `NO-RESULT` **with the reason inside the value**:

| check | refusal |
|---|---|
| the file exists | `NO-RESULT (--tests-log MISSING: …)` |
| it is non-empty | `NO-RESULT (--tests-log EMPTY: …)` |
| **nothing cargo would read is newer than it** | `NO-RESULT (--tests-log STALE: <path> is newer than the log, so the log did not test this tree)` |
| **every launched target reported a result** | `NO-RESULT (--tests-log SHORT: cargo launched R targets and only T reported a result)` |
| the log ends on a `test result:` line | `NO-RESULT (--tests-log INTERRUPTED: …)` |

Two of those are load-bearing and worth naming.

**The freshness closure is DERIVED, not remembered.** `_tests_inputs` re-derives
it from the tree on every run: `crates/`, `fixtures/`, `Cargo.toml`/`Cargo.lock`,
the data files the registry tests read (`scripts/lanes.txt`, `scripts/sweep.d`,
`scripts/sweep_gen.py`, `docs/rungs/`) **and every path an `include_str!` in
`crates/` actually names** — which is how `work/w-inl0/cells/*.cpp` and its
siblings are in the list. Nobody would have typed those; the proposal found them
by grepping and this script re-finds them by grepping.

**The short-count check is the same reconciliation the sweep uses.** `Running` /
`Doc-tests` lines and `test result:` lines are counted and must be equal and
non-zero. STATUS.md trap 5's newest instance was a runner reporting `ok` for
every target with **169 tests silently not run**; this is the compare-a-count
answer to it, applied to a log the caller hands over.

It is **not a cache**. Without the flag the suite runs, exactly as before. There
is no sentinel, no skip-if-unchanged, and no state.

### 5.1 Six mutations, six reds — `status.sh --check` proves every gate

`--check` needs no toolchain and now runs `collect_tests_from_log` for real over
seven synthesized logs. Each gate was disabled in turn and `--check` reddened on
exactly the case it protects:

| mutation | `--check` says |
|---|---|
| existence gate → `if false` | `--tests-log absent rendered 'NO-RESULT (--tests-log EMPTY: …)', expected 'NO-RESULT (--tests-log MISSING…'` |
| empty gate → `if false` | `--tests-log empty rendered 'NO-RESULT (--tests-log NO-RUN: … 0 launched, 0 reported)', expected 'NO-RESULT (--tests-log EMPTY…'` |
| STALE gate → `if false` | `--tests-log stale rendered '7 passed, 0 failed, 2 targets', expected 'NO-RESULT (--tests-log STALE…'` |
| SHORT gate → `if false` | `--tests-log short rendered '7 passed, 0 failed, 2 targets', expected 'NO-RESULT (--tests-log SHORT…'` |
| end-of-log gate → `if false` | `--tests-log interrupted rendered '7 passed, 0 failed, 2 targets', expected 'NO-RESULT (--tests-log INTERRUPTED…'` |
| the `include_str!` grep deleted | `_tests_inputs names no work/ path — the include_str! closure is not being re-derived` |

The two that render a **number** where a refusal belongs — STALE and SHORT — are
the two that would have been the false green.

**A methodological note worth carrying.** The first version of these checks
expected the generic prefix `NO-RESULT (--tests-log: ` for several cases, and two
of the six mutations then **passed**: with a gate removed, the *next* gate's
message matched the same prefix, so the check could not tell which refusal had
fired. Each refusal now leads with its own word (`MISSING`, `EMPTY`, `STALE`,
`NO-RUN`, `SHORT`, `INTERRUPTED`) and every mutation is discriminated. A
must-fail suite whose expectations are not distinct is a must-fail suite that
does not fail.

---

## 6. Item 4 — the docs-only skip, DECLINED with a measurement

`PROPOSAL.md` §4 proposes skipping the cycle when a merge's diff is confined to
`docs/`. Its closure derivation is careful and found a genuine surprise: `work/`
is in the graded closure, via `include_str!` in
`crates/c2-harness/tests/dead_temp_elision.rs` and siblings. What it never
measured is **how often the rule would fire**.

Over the **last 40 merge commits** on master (diff of `M^1..M`, the same
expression the rule uses):

| rule | fires on |
|---|---:|
| strict — every changed path under `docs/` | **0 of 40** |
| narrow — `docs/`, or `work/` paths no `include_str!` names | 7 of 40 |

Not one of the last 40 merges changed only `docs/`. The median merge changes ~50
files of which ~90 % are under `work/`, because **every lane commits its evidence
there** — that is the convention the project runs on, and it is what makes the
rule dead. The two merges that motivated the proposal are among the misses:
`59b6e3d2` (2 files, 1 under `work/`) and `fea2daea` (24 files, 21 under
`work/`).

**Both versions are declined, and the narrow one is declined for a reason other
than its fire rate.** Its closure is a *derived allowlist* whose licence to skip
is a **negative** grep result — "no `include_str!` names this path" — and an
absence read as permission is the shape of every defect on STATUS.md's trap-5
list. It is also narrower than the truth: `scripts/regen_census.sh` reads
`work/w-bss/census/sections.jsonl` and `work/w-bss2/*.py`, none of which the rule
can see. At 7 fires over 40 merges and a ~5-minute cycle after items 1 and 2, the
whole prize is ~35 minutes spread over 40 merges, against a permanent
false-green surface in the merge ritual.

The measurement is the deliverable. `work/w-throughput/docs_only_skip.md` has the
method and the per-merge table.

---

## 7. What this lane did NOT do

* **Proposal §3 — run the 36 test binaries concurrently.** Not attempted. After
  §4 the test leg is TEST_LEG_AFTER, and §3's own estimate (~120 s → ~70-80 s)
  is a ~50 s prize for a new ~100-line runner whose failure mode is *"a binary
  the wrapper never launched"* — instance seventeen, invited deliberately. It is
  worth doing only if the test leg still bites, and it does not yet.
* **Proposal §6's declined items** — caching the sweep's reference captures,
  overlapping `cargo test` with `gate.sh` — are left declined, on the reasoning
  there. The capture-cache one is the sharper of the two: the cache key does not
  cover the c2-reference **capture code**, and the sweep's distinct job is
  asserting that the capture/replay mechanism is byte-exact (board #282).
* **Board #1331 and #1332 are UNMINTED.** The range assigned was #1323–#1332 and
  the lane found eight things worth a permanent number. A row minted to fill a
  range is a row nobody can retire.

---

## 8. Gate evidence

GATE_EVIDENCE
