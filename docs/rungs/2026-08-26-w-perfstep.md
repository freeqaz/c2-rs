# w-perfstep — `#3583` answered: the step is smaller than a byte-identical arm's own spread

    Tag:       w-perfstep
    Slug:      w-perfstep
    Date:      2026-08-26
    Kind:      characterization
    Outcome:   instrument
    Fixtures:  none — instrument lane: it builds a measurement of an already-published metric and touches no emitter
    Census:    707728/2417794 → 707728/2417794 (29.27% → 29.27%), +0
    Record:    this file; prereg `_2026-08-26-w-perfstep-prereg.md`, frozen before any arm was built

## 1. The result

`docs/STATUS.md`'s generated block read **664×** at tree `b814d1db2` and **553×**
at `c13cebbca` — a **−16.7 %** step that `#3583` filed as unexplained, with three
candidates named and none chosen between.

**The answer is (c), box state — and the sharp form of it is that the step is
smaller than the noise floor of the metric measured on an arm whose true effect
is exactly zero.**

| what was measured | value |
|---|---|
| **NULL arm `predup`** (md5-identical to `pre`), 12 rounds, geomean speedup | **553.9× … 677.7× — spread 22.3 %** |
| **NULL arm `wdup`** (second run, same box, same day), 6 rounds | **539.6× … 633.0× — spread 17.3 %** |
| the step `#3583` was filed over | **16.7 %** |
| all 36 runs of the main experiment, one session | **549.8× … 889.2×** |
| the two published values | **664× and 553× — both inside that one range** |

A null arm cannot differ from its baseline. Its spread is pure measurement noise,
and it is **larger than the step**, twice, in two independent runs on one box on
one day. Nothing needs to have changed for this digit to move 16.7 %.

**`#3583`'s ≈3 % premise is where the row went wrong, and it is an instructive
error rather than a careless one.** That figure came from a **same-session
re-run** — the one condition in which box state is held nearly constant. It is a
single draw from the distribution above, not its spread. Measuring the spread
requires repetition across the axis you suspect, and the axis was the session.

## 2. The restatement that made the candidates testable at all

None of `#3583`'s three candidates could be tested as named, because nobody had
read what the number is (`crates/c2-harness/src/perf.rs:101`):

    speedup(fixture) = ref_median / port_median
    geomean          = exp(mean(ln speedup))   over MATCHED fixtures only

* `ref_median` — median of **`ref_iters = 5`** samples of `Toolchain::replay`, a
  **`wibo` process spawn** running real `c2.dll`, tens of milliseconds each.
* `port_median` — median of **`port_iters = 2000`** in-process
  `PortC2::compile_to` calls, microseconds each.

**The numerator is the reference.** A move in the port and a move in the
reference are the same published digit with opposite causes, and every candidate
would have had to act through one side or the other. `scripts/cost_arms.py`
deliberately discards the reference column — correct for a construct rung's cost
clause, exactly wrong for this row. `scripts/perf_arms.py` (**#3609**) is that
protocol for the published metric: it **imports** `cost_arms` rather than
restating it (**#3451** is four retypings of one protocol), keeping the balanced
rotation (**#3521**), the toolchain pin and per-arm preflight (**#3575**), the
arm-identity block (**#3525**) and the `cmp`'d null, and adds three columns —
speedup / `ref_ns` / `port_ns` — **per round** as well as pooled.

**And it is the reference that moves.** Round-to-round spread, ref vs port:

| arm | ref spread | port spread |
|---|---:|---:|
| `pre` | **84.5 %** | 37.9 % |
| `post` | **106.7 %** | 35.0 % |
| `predup` (NULL) | **56.4 %** | 39.3 % |

Three arms for three. Five samples of a process spawn against two thousand
samples of an in-process call is not a symmetric estimator, and the asymmetry is
where this row's instability lives (**#3613**).

## 3. Each candidate, and how it died

### (a) the workload — REFUTED by read, then by trace

`scripts/status.sh::collect_perf` is four lines and runs `"$c2rs" perf` with **no
`$workload`, no `$_list`, no `$_flags`**. Those belong to `collect_gap`. The
stamp is printed once in the block's shared header, so it decorates every row
including the ones it cannot reach.

Empirically: an `strace` of a full `perf` fixture cycle records **212 file
syscalls and 0 accesses to any `dc3` path**. Scope stated honestly — the trace
covers the `c2rs` process itself, not its `wibo`/`cl.exe` children, which are
driven with explicit toolchain paths. The two arms' fixture trees are **391 = 391
with an empty `diff`**. (**#3612**)

### (b) `repo_root()` at runtime — REFUTED three ways

1. **Read.** One call site in `crates/c2-reference` (`Toolchain::locate`,
   `lib.rs:298`); inside `c2rs`, `Toolchain::locate` is reachable **only**
   through `argv::Args::toolchain{,_quiet}`, a fence
   `tests/cli_flags.rs::locate_is_reachable_only_through_the_arg_seam` enforces.
   At most **twice per process**.
2. **Trace.** The whole runtime walk is **19 `statx` calls, contiguous, ONCE**,
   at `p5_trace.txt:12–31`, and never again in the run. It completes inside
   `locate()` — **before `bench_fixture` is called** — so it lands outside every
   `Instant::now()` bracket in `perf.rs`, which brackets `tc.replay` and
   `port.compile_to` and nothing else. Its contribution to `ref_median` and
   `port_median` is **zero by construction, not merely small.**
3. **Single-variable A/B.** One binary, 18 runs, three wrapper arms differing
   only in whether `C2RS_REPO_ROOT` is exported (it short-circuits `repo_root()`
   at its first line): **port +0.00 % [−0.12, +0.12], split 53 %**; ref −0.35 %
   [−0.79, +0.09], CI containing zero.

**The lane was briefed that a measured price here would be a finding, not a
regression to revert. The price is +0.00 % ± 0.12 %. Nothing was reverted and
nothing needed to be** (**#3611**).

### (c) box state — CONFIRMED, and §1 is the measurement

**The tree already said so, in the script that collects the number.**
`scripts/status.sh:439` has carried, since long before `#3583` was filed:

> *"on 2026-08-05 two collections of the same unchanged code read **674×** and
> **481×**. Nothing in `crates/` had moved — the second ran while three gates
> were saturating the machine."*

That is **−28.6 %** on unchanged code, in the collector's own comment block, and
`STATUS.md`'s what-each-number-is-for row says **load-sensitive** and quotes
623/653/689 on three consecutive runs of one binary. This lane's contribution is
not the discovery that the row is noisy; it is the **null-arm floor**, which is
the form that can be compared against a filed step (**#3610**).

### And the trees DID differ — by +2.48 %, the other way

Min-over-rounds (`cost_arms.py`'s estimator), `post` vs `pre`, n = 157:

| | speedup | ref_ns | port_ns |
|---|---:|---:|---:|
| `post` vs `pre` | **+2.48 %** | +2.06 % | **−0.41 %** |
| `predup` (NULL) vs `pre` | +0.32 % | +0.10 % | −0.21 % |

Paired per fixture: port **−0.41 % [−0.59, −0.23]** against a null of −0.21 %
[−0.34, −0.08] — **under `#3551`'s 0.93 % build floor, so not an effect**, and
reported as a bound rather than a sign. Real port code moved between these trees
(`c2-il` body decode, `c2-core` calls, 1,046 insertions) and it moved the port
**faster**, by an amount inside the floor. **The published step is −16.7 %; the
measured tree difference is +2.5 %: wrong size and wrong sign** (**#3613**).

## 4. Estimate vs outcome — all five registered predictions, graded

Prereg `_2026-08-26-w-perfstep-prereg.md` §2, frozen at `a490f3c3f` before any
arm was built.

| # | registered | conf | outcome |
|---|---|---:|---|
| P1 | (b) refuted by read; contribution < 0.1 % | 0.90 | **HIT** — and stronger than registered: zero *by construction*, not small |
| P2 | the two trees differ by < 5 % with overlapping ranges | 0.75 | **HIT** — +2.48 %, and opposite in sign to the published step |
| P3 | one arm's cross-run spread > 5 % | 0.60 | **HIT** — 22.3 % on the null alone, 56.8 % on `post` |
| P4 | the moving side is the reference | 0.70 | **HIT** — on all three arms (84.5/37.9, 106.7/35.0, 56.4/39.3) |
| P5 | (a) refuted; zero workload accesses | 0.85 | **HIT** — 0 of 212 file syscalls |

**Five for five is a worse sign than it looks, and it is worth saying so rather
than banking it.** P1 and P5 were read-derived before the prereg was written —
the `repo_root()` call-site fence and `collect_perf`'s argument list were both
already on screen — so they were registered at high confidence because they were
nearly known, not because the lane forecast well. The two that carried real risk
were **P2** (0.75) and **P3** (0.60), and only P3's *direction* was ever in
doubt. A prereg whose predictions are all HITs is a prereg that mostly recorded
reads. The registered bias is stated here rather than in the summary line.

## 5. What this lane did NOT do

* **Reverted nothing.** `#3470` is a correctness fix that bites backwards; its
  price is measured at zero and it stays.
* **Did not re-run `status.sh --write`.** Regenerating the block is the
  coordinator's, and doing it here would move a fourth stamp.
* **Did not touch a peer's file** — `crates/c2-il` (`w-atend`),
  `crates/c2-harness/src/gap` (`w-symbind`), `docs/whitebox/**` (`w-opclass`),
  the top-level pricing docs (`w-price4a`). `crates/c2-harness/src/perf.rs` is in
  this lane's fence and was **not modified**: every number here comes from
  parsing the harness's existing output, so no arm's binary differs from the tree
  it was built at.

## 6. The coordinator's mid-flight item — DECLINED, and the diagnosis changed it

Relayed mid-lane off `w-symbind`'s false red: `scripts/expr_sweep.sh` carries
`max_ungraded = 96` against a corpus reading exactly 96, and the prescribed shape
was *"quote the reference-capture failures as their own count, separate from the
ungraded total"*.

**Measured before implementing, and the measurement says that shape cannot be
built.** Watched going RED on demand (`C2RS_SWEEP_MAX_UNGRADED=0` →
`UNGRADED 96 exceeds the carried baseline 0`, exit 1) with the clean run beside
it (`checked=19556 mismatches=0 graded=19460 ungraded=96 unknown=0`, exit 0). Of
the 40 verdicts the guard prints:

* **37** read `ReferenceError: capture_reference failed: … produced no obj`
* **3** read `ReferenceError: replay failed: …`
* **`error C####` / `warning C####` appears ZERO times in any verdict**

So a source `cl.exe` rejects and a capture that dropped under load emit the
**identical string**. There is nothing to key a discriminator on: **94 content +
2 transient reads exactly 96 and passes.** Zero headroom is the lesser half of
this row; **silent reallocation of the window is the greater one.**

Two more facts the red path showed: the guard's list is **`head -40` of 96**, so
the failing set cannot be pinned by name from the gate's own output; and **3 of
40 are replay failures**, contradicting the script's own comment block
(`:472–486`, *"cl.exe rejects them"*, two causes, both compile-side).

**The fix that works is the repo's own standing rule, and nobody had connected it
to this row**: `docs/rungs/README.md` (2026-08-17, from **#3219**/**#3231**) —
*carry a control whose failing set is pinned by NAME, because a control pinned by
COUNT passes the moment the count matches.* Shipping that means harvesting all 96
names, proving the set reproduces, and changing the pass/fail logic of the one
instrument the whole funnel reads while five lanes gate against it. **That is a
lane, not a cheap add-on. Declined and named as #3614, not buried.**

## 7. What `STATUS.md`'s speedup row should carry

Recommended, not made — the generated block is the coordinator's surface and a
lane editing it would move a stamp:

1. **Not a point estimate.** Two nulls measured 22.3 % and 17.3 % on one box in
   one day. A single digit implies a precision the instrument does not have.
2. **A floor beside it, in its own units.** `#3551`'s 0.93 % is **not** this
   row's floor — that is a floor for the *port* under build-directory variation.
   This row's floor is a null arm's spread of the *ratio*, and it is ~20 %.
3. **`ref_iters` is the lever, and it is the counter-intuitive one.** The row is
   unstable because its numerator medians five process spawns. Raising
   `ref_iters` would narrow it; nothing done to the port will.
4. **Or drop the row for `perf-scale`**, which measures throughput against a time
   budget instead of a five-sample median, and which the README's ~1200–5000×/obj
   already rests on.

## Gate evidence

| lane | result |
|---|---|
| `scripts/gate.sh --jobs 16 --require-graded` | **`GATE: PASS (HATCH-RED REFUSED)`** — the verdict line, never the exit code (§8.1) |
| `scripts/gate_identity_diff.sh` vs `base_c13cebbca.txt` | **`IDENTITY DIFF: 0 lines over 21 rows`**, `21 base, 21 tip` (§8.2) |
| `scripts/expr_sweep.sh` | `checked=19556 mismatches=0 graded=19460 ungraded=96 unknown=0`, exit 0; and watched RED at `C2RS_SWEEP_MAX_UNGRADED=0` |
| `scripts/perf_arms.py --self-test` | `PASS`, every control watched failing first |
| `scripts/cost_arms.py --self-test` | unchanged by this lane; `perf_arms` imports it |
| `scripts/tracked_artifact_audit.sh` | forbidden artifact names **0**, absolute machine paths in code surfaces **0** |
| `scripts/board_audit.sh` | duplicate row numbers 0, unresolved anchors 0, rows-behind-prose 0 |
| 878-TU workload scan | not re-run — this lane changes no `crates/` byte; `Census: +0` |

## 8. Gate verdicts

### 8.1 `scripts/gate.sh --jobs 16 --require-graded` — the VERDICT LINE

    GATE: PASS (HATCH-RED REFUSED) — 18/18 lanes ran and every one of them graded a corpus,
      the sweep graded 19460 of 19556 generated cases and the cross graded
      90424 of 90812 case-lane cells, with 0 mismatches anywhere
      (96 sweep cases carried ungraded — the reference rejects the source),
      and 18/18 lanes ran again through a DEBUG-profile c2rs for
      7038 more fixture-verdicts at 0 panics

    graded tree: 880b6cf41a5f  (791 files: crates fixtures scripts, content-hashed)

**Quoted as the verdict line and not as an exit code, deliberately.** In wave 11
`gate.sh` printed `GATE: REFUSED (DIRTY crates/)` at exit **0** and `GATE: PASS`
at exit **1**. The line is the verdict; the status is not.

**`HATCH-RED REFUSED` is PRE-EXISTING and not this lane's.** The base run at
`c13cebbca` (`work/coordinator/gate_tip_c13cebbca.txt:100,108`) carries the
**identical** qualifier and the identical reason — `HATCH-STALE`, `hatch.py
apply` cannot hatch this tree, board **#1389**. The working tree was clean
(`git status --short` empty) when the gate ran, and nothing was edited while it
ran.

**One line of that verdict is now known to overclaim, by this lane's own §6:**
*"96 sweep cases carried ungraded — the reference rejects the source"*. The
instrument cannot establish that. Every one of the 96 prints `produced no obj`
with no diagnostic code, and 3 of the 40 shown are **replay** failures rather
than compile rejections. The count is right; the *reason* beside it is a
2026-08-04 hand investigation the run cannot re-derive (**#3614**).

### 8.2 `scripts/gate_identity_diff.sh` — with its row denominator

    count-bearing rows: 21 base, 21 tip (enumerated, not asserted)
    IDENTITY DIFF: 0 lines over 21 rows — required-zero byte delta HOLDS

Base `work/coordinator/gatebase/base_c13cebbca.txt`, tip
`work/w-perfstep/gate_tip.txt`. **The denominator is 21 at both ends** — a
diff over 0 rows and a diff over 21 rows both print "0 lines", which is
`w-s1c2` §3.2's lesson and why the count is quoted beside the verdict.

Zero is the expected and required result: **this lane changed no byte of
`crates/`.** Its whole diff is `scripts/perf_arms.py` (new), `docs/`, and
evidence under `work/w-perfstep/`.

### 8.3 The controls, each watched failing before its green was quoted

| control | watched RED | then GREEN |
|---|---|---|
| `perf_arms.py` geomean | `geomean([])` returns `None` and refuses | `geomean(1,100) = 10` |
| `perf_arms.py` row parse | `NotImplemented` / `Mismatch` / garbage rows all dropped | only `ok.cpp` kept; units reproduce the row's own `939x` |
| `preflight_arm` (inherited) | `cost_arms.py --self-test`'s three shapes — SKIP-at-exit-0, zero-Match, nonzero-exit — all REFUSED | `grades-one` passes with denominator 1 |
| the sweep's ungraded baseline | `C2RS_SWEEP_MAX_UNGRADED=0` → `UNGRADED 96 exceeds the carried baseline 0`, exit 1 | `ungraded=96`, exit 0 |
| per-arm denominator | — | all three arms preflight at **157 Match each**, printed before anything is timed |
| null-arm precondition | — | `predup` / `wdup` verified byte-identical by `cmp`, not assumed |

The two rows with no RED cell are marked so rather than left blank: a per-arm
denominator and a `cmp` are assertions about this run, not detectors with a
failing mode this lane exercised.
