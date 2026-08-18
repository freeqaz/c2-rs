# w-gateperf — PREREG

Frozen as this lane's FIRST commit, before any profiling run. Base: master
`071d2d47`. No discount factor is applied to any row: every probability below is
the number I actually believe, and a row I get wrong is scored as wrong.

## 0. What is KNOWN at freeze time, and is therefore not predicted

These were established by **reading code**, before any measurement. They are
recorded here so the prereg cannot later take credit for "discovering" them.

* **K1.** `scripts/gate.sh`'s own default is `jobs=16` (line 536), tuned and
  documented at length (the block at lines 477–535, lane `w-throughput`,
  2026-08-08). **The standing lane instruction `--jobs 4` OVERRIDES that
  default with the untuned constant the tuning replaced.** The file's own table
  reads `--jobs 4 ≈ 300 s` vs `--jobs 16 ≈ 105 s` on an idle box, with
  identical verdict blocks.
* **K2.** `cmd_diff` (`crates/c2-harness/src/cli/reference.rs`) calls
  `differential()` against a fresh `Scratch` and **never touches
  `capture_cache`**. `scripts/mode_cross.sh`'s header states this explicitly:
  *"`expr_sweep.sh` cannot take this path: it drives `c2rs diff`, which does not
  consult the cache at all."*
* **K3.** `scripts/mode_cross.sh` drives `c2rs gap --list`, which DOES consult
  the cache, and its header records cold 5 min 45 s vs warm 13.8 s over 61,539
  cells — a 25× cache effect.
* **K4.** `expr_sweep.sh` spawns one `c2rs diff` process **per case**, from a
  `sh` `while read` loop, through a `$(...)` command substitution. At 19,556
  cases that is ≥19,556 `c2rs` process spawns plus ≥19,556 subshell forks.
* **K5.** `differential()` does capture **and** replay: `capture_fixture_reference`
  (strace + wibo + `cl.exe` → `c1xx.dll` → `c2.dll`) and then
  `Toolchain::replay` (wibo + `c2host.exe` + `c2.dll`). So a sweep case is
  strictly more work than a cross cell, which only captures.
* **K6.** `C2RS_JOBS` is documented in `gate.sh` as read only by
  `mode_lane.sh`, and the cross is invoked with `C2RS_JOBS="$jobs"` explicitly,
  so for the cross leg the two knobs do **not** multiply.

## 1. Predictions

Probability form. Interval rows give a point estimate and an interval.

| # | prediction | p / interval |
|---|---|---|
| **P1** | The **generated sweep** is the single dominant leg of `gate.sh --jobs 4 --require-graded`, taking **> 55 %** of the run's total wall clock, decomposed inside one run | p = 0.90 |
| **P2** | Within the sweep, **≥ 80 %** of per-case wall clock is process spawn + PE load under wibo (capture + replay), not the port's own codegen or the obj compare | p = 0.85 |
| **P3** | `execve` count per sweep case (counted with `strace -f -c` or `-e trace=execve` on one case) | point **6**, interval **4–12** |
| **P4** | Total process spawns attributable to the sweep leg alone exceeds **100,000** | p = 0.75 |
| **P5** | Serial warm per-case cost of one `c2rs diff` on a generated case | point **60 ms**, interval **30–150 ms** |
| **P6** | The capture cache serves **0** hits for the sweep leg (K2 makes this near-certain; scored anyway because a hit would refute my reading) | p = 0.95 |
| **P7** | The mode cross leg **does** benefit: its second consecutive run in one session is ≥ 3× faster than its first with a cold case dir | p = 0.85 |
| **P8** | Re-running the identical gate at `--jobs 16` instead of `--jobs 4` on the same box within one session gives a **total** speedup ratio ≥ 2.2× | p = 0.60 |
| **P9** | `--jobs` and `C2RS_JOBS` **multiply only on the lane leg**, and the lane leg is < 5 % of the total, so the multiplication is not worth acting on | p = 0.80 |
| **P10** | The largest coverage-preserving win available is **teaching the sweep's grading path the capture cache** (or an equivalent batch path that reuses captures), worth ≥ 60 % of the sweep leg once warm | p = 0.60 |
| **P11** | Eliminating the per-case `c2rs` process spawn alone (batching cases into one process) — with captures still uncached — is worth **< 25 %** of the sweep leg, i.e. the wibo/`cl.exe` tree dominates the `c2rs` startup | p = 0.70 |
| **P12** | I will need to change `crates/` (not only `scripts/`) to land the primary speedup | p = 0.65 |
| **P13** | The port's own `PortC2::compile_to` is **< 2 %** of the sweep leg | p = 0.80 |
| **P14** | After my change, an injected fault still reddens the gate with a legible message | p = 0.95 |
| **P15** | I will land at least one coverage-preserving change with a **measured** ≥ 25 % reduction in gate total wall clock | p = 0.65 |
| **P16** | I will find at least one *additional* piece of redundant or non-executing work in the gate beyond the sweep's cache hole | p = 0.55 |
| **P17** | The multi-lane recommendation I end up giving is "**bound total concurrency across lanes**" (a shared lock or a global job budget) rather than "nothing" or "lower per-lane jobs" | p = 0.45 |
| **P18** | `hatch-red` will read `REFUSED HATCH-STALE` on every run I take (board #3219, pre-existing) | p = 0.90 |
| **P19** | `Outcome:` will be `built` | p = 0.70 |
| **P20** | The 878-TU scan identity holds at **394** keys with 0 changed (fnbyte family may drift ±2 per #3249) | p = 0.85 |
| **P21** | I will propose at least one **coverage-REDUCING** change and leave it unimplemented as a priced proposal | p = 0.60 |
| **P22** | `cargo test --workspace --release --no-fail-fast` reads **1,660 / 0 / 43** at both ends | p = 0.85 |
| **P23** | Measured on a quiet box, the gate total at `--jobs 4` on the base tree is in **380–600 s** | p = 0.55 |

## 2. Invalidation rules — the probe-soundness clauses (#3219 / #3231)

* This worktree was created by `scripts/setup_worktree.sh`, which symlinks
  `compilers/` and verified the toolchain resolves (its own output said
  `OK: fixtures/cpp/w5_chain.cpp -> 4/4 functions in class`). **Every timing
  below is void unless the run it came from graded a non-zero count.** A leg
  that reports SKIP or `graded 0` is not a fast leg, it is an absent one.
* **The control pinned by NAME**: every gate run quoted must report the sweep
  at `checked=19556 ... graded=19460 ungraded=96` and the cross at
  `checked=90812 ... graded=90424 ungraded=388`, and the 18 lanes at
  `386/386`. A run that is fast and does not print those counts is discarded.
* **Timings are decomposed INSIDE one run** (`w-gatewire`'s method). Two totals
  taken an hour apart on a box whose load moved are not comparable and are not
  quoted as a delta. Every wall-clock figure is quoted with the load average it
  was taken at.
* **Counts and syscall totals outrank wall clock.** Where a claim can be made
  in spawns, syscalls or bytes, it is made there.
* Results tables are **derived from logs**, never accumulated (rule 2).
