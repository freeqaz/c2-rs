# PREREG — `w-perfstep`, resolve `#3583`: why the published port speedup stepped 664× → 553×

    Tag:       w-perfstep
    Date:      2026-08-26
    Kind:      instrument lane
    Base:      f202268f6  (master, clean; `decisions: decision 14 — the five follow-ons wave 11 named are funded`)
    Board:     #3609–#3614 (reserved by the coordinator for this lane)
    Fixtures:  none — instrument lane: it builds a measurement of an already-published
               metric and does not touch the emitter
    Census:    +0

**Frozen before any measurement of this lane's own arms.** Predictions below are
never edited afterwards. Navigation pointers may be repaired by *amending
beside* (`#3495`'s convention); a number or a probability may not.

---

## 0. The question, as `#3583` states it

`docs/STATUS.md`'s generated block read **664×** at tree `b814d1db2`
(2026-08-24) and **553×** at tree `c13cebbca` (2026-08-25) — a **−16.7 %**
step. Three stamps moved between the two collections (tree, binary, workload),
and nothing the byte judge grades moved at all. Three candidates are named and
none is chosen between:

* **(a)** the external `../dc3-decomp` workload advanced under both arms;
* **(b)** `w-hygiene`'s `repo_root()` now resolves at **runtime** where it was
  `env!("CARGO_MANIFEST_DIR")` — a per-call cost on a possibly-hot path;
* **(c)** box state across a day.

`#3525`'s `.rodata`-layout mechanism is out of scope by `#3551`: experiment F
bounded that family at **0.93 %** and cannot cover a 16.7 % step.

## 1. What the metric IS, restated from the source, because the three candidates
##    were named without it

`crates/c2-harness/src/perf.rs`:

    speedup(fixture) = ref_median / port_median          (FixturePerf::speedup)
    geomean          = exp(mean(ln speedup))             over MATCHED fixtures only

* `ref_median` is the median of **`ref_iters = 5`** samples of `Toolchain::replay`
  — a **`wibo` process spawn** running real `c2.dll`, tens of milliseconds each.
* `port_median` is the median of **`port_iters = 2000`** in-process
  `PortC2::compile_to` calls, microseconds each.

So the published ratio has a **process-spawn numerator estimated from five
samples** and an in-process denominator estimated from two thousand. The two
sides are not equally noisy and a move in the ratio is not attributable to the
port without splitting them. **No prior reading of this row has split them**,
and `#3583` names three candidates all of which would have to act through one
side or the other.

The fixture population did **not** change across the two collections — both
blocks read `157 port Match, 0 mismatch, 234 not-implemented (of 391)` and
`391 PASS`. So `GAPS.md` §1's population trap (*"two geomeans are a change of
population"*) is **excluded by construction** and is not a fourth candidate.

## 2. Predictions, registered before any arm is built

Each is falsifiable and is graded in the rung with the number that decided it.

* **P1 — (b) is REFUTED BY READ.** `repo_root()` has exactly **one** call site
  in `crates/c2-reference` (`Toolchain::locate`, `lib.rs:298`), and inside the
  `c2rs` binary `Toolchain::locate` is reachable **only** through
  `argv::Args::toolchain{,_quiet}` — a fence `tests/cli_flags.rs`
  (`locate_is_reachable_only_through_the_arg_seam`) enforces. It is therefore
  called **at most twice per process**, never per fixture and never per timed
  iteration; neither `PortC2::compile_to` nor `Toolchain::replay` reaches it.
  Predicted contribution to a `c2rs perf` run: **< 0.1 %**.
  **Confidence 0.9.** *Graded by the read plus P2.*
* **P2 — the two published TREES will not reproduce the step.** Built as arms
  and run **interleaved in one session** on one box, `b814d1db2` and
  `c13cebbca` will differ in geomean speedup by **less than 5 %**, with
  overlapping round-to-round ranges. **Confidence 0.75.**
  *If they differ by ≥ 15 % and reproducibly in the published direction, P1/P2
  are a MISS and (b) or some other tree change is the answer.*
* **P3 — the metric's own cross-run spread is far above the ≈3 % `#3583`
  quotes.** Over ≥ 6 full `c2rs perf` runs of ONE binary in one session, the
  per-run geomean will span **> 5 %** (max/min − 1). **Confidence 0.6.**
  The ≈3 % figure came from a *same-session* re-run, which is the case where
  box state is held most nearly constant; `scripts/status.sh`'s own comment
  block already records **674× and 481× (−28.6 %)** across two collections of
  unchanged code.
* **P4 — the moving side is the REFERENCE, not the port.** The relative
  round-to-round spread of `geomean(ref_median)` will exceed that of
  `geomean(port_median)`. **Confidence 0.7.**
* **P5 — (a) is REFUTED.** `c2rs perf` reads `fixtures/cpp/**` and the fixture
  profiles; it never reads `../dc3-decomp`. Predicted: **zero** filesystem
  accesses under the workload root during a `perf` run, checked with `strace`
  rather than asserted from a grep. **Confidence 0.85.**

**Registered answer, in one sentence:** the step is **(c)**, and the deeper
finding is that this row has never had a noise floor stated in the units it is
published in — so the honest publication is a floor/range or a stamp-pinned
protocol, not a point estimate.

**A registered MISS is reported plainly.** If P2 misses — if the trees do
reproduce the step — that is the more interesting result and the rung says so
in its headline.

## 3. Method, registered

1. **Arms.** `pre` = `b814d1db2`, `post` = `c13cebbca`, `postdup` = a
   byte-identical **copy** of `post`'s binary (the NULL — `cost_arms.py`'s
   standing requirement; a null that is rebuilt is not a null). All three built
   from `git archive` scratch trees under `work/w-perfstep/`, and **tagged**
   (`#3552`: a pinned commit no ref names is one `gc` from gone).
2. **Toolchain pinned** with `C2RS_COMPILERS` / `C2RS_WIBO` exported before any
   arm runs — `pre` predates `w-hygiene`'s fix and will otherwise print
   `SKIP: toolchain absent` and **exit 0** (`#3470`, biting backwards).
   **Every arm preflighted with a denominator** before anything is timed.
3. **Rotation.** `cost_arms.py`'s carryover-balanced cycle (`#3521`), **reused
   by import, not retyped** — `#3451` is four rewrites of one protocol. Rounds a
   multiple of `2n = 6`.
4. **Reported per arm, per round:** geomean speedup, geomean `ref_median`,
   geomean `port_median`, over the fixtures **every arm matched in every
   round** (the stated denominator). Plus each arm's md5 / size / build dir
   (`#3525`), and load at both ends.
5. **Every control watched failing before its green is quoted** (`#3336`,
   `#1236`).

## 4. What this lane will NOT do

* **It will not revert anything.** If (b) turned out to have a cost, `#3470` is
  a correctness fix that bites backwards and a measured throughput price is a
  *finding*, not a regression to undo.
* It will not touch `crates/c2-il`, `crates/c2-harness/src/gap`,
  `docs/whitebox/**` or the top-level pricing docs — peers own those.
* It will not re-run `status.sh --write`. Regenerating the block is the
  coordinator's, and doing it here would move a fourth stamp.
