#!/bin/sh
# THE MODE-LANE GATE — run every lane in `scripts/lanes.txt`, one result each.
#
# ---- why this exists -----------------------------------------------------------
#
# `mode_lane.sh` runs ONE lane and has always worked. Nothing enumerated the lanes,
# so the set that actually ran on any given day was the set somebody remembered to
# type — and the four recorded through `docs/` (`/Ox`, `/O1`, `/O2`, `/Ox /Gy`)
# contain no `/EH` at all, on a workload that compiles `/EHsc` on every TU. Two
# `/EHsc` lanes were added, went green, and caught a live wrong-bytes emit every
# other lane was blind to; nothing whatsoever made them run again.
#
# A lane that exists but is not enumerated is a lane that does not run. The list is
# now data (`scripts/lanes.txt`) and this is the one command that runs it.
#
# ---- what this gate promises ---------------------------------------------------
#
# The promise is deliberately stated as a POSITIVE: **every lane in the registry
# produced a result, and the gate says how many.** Not "no lane failed" — the
# expensive failure class on this project is not a lane going red, it is a lane
# going ABSENT and the absence reading as zero. Eight instruments here have now
# reported green from an absence, including one whose every check `sed`-ed a number
# out of a report and read the missing number as 0, passing a run that graded
# literally nothing.
#
# So, concretely:
#
#   * Each lane must print a `LANE-RESULT` line. The gate parses that line and
#     re-derives the verdict from its fields. **A zero exit status is not accepted
#     as evidence that a lane ran** — a lane that dies, is killed, or is skipped by
#     the loop prints no such line and is reported `NO-RESULT`, which fails.
#   * `PASS` additionally requires `graded > 0`: a lane that submitted 197 fixtures
#     and graded none of them is not a passing lane, whatever its exit status.
#     (That is the toolchain PRESENT and every capture failing — a relative outdir,
#     an exhausted tmpfs inode table, a bad flag — and it is a different thing from
#     the toolchain being absent.)
#   * The result table is rendered by walking the REGISTRY, not by walking whatever
#     result files happen to exist, and the number of rendered rows is compared
#     against the registry length before any verdict is printed. A lane cannot
#     vanish out of the table.
#   * `SKIP` is its own verdict and prints as `SKIP`, never as `PASS`. An all-SKIP
#     run prints `GATE: SKIPPED` and says in the headline that nothing was graded.
#     It exits 0 because CLAUDE.md requires the toolchain-absent path to degrade
#     cleanly — but it cannot be mistaken for green by anything reading the output.
#   * A PARTIAL skip fails. If the toolchain is present, every lane runs; some
#     lanes skipping and others not means a lane declined for a lane-specific
#     reason, which is a fault, not a degradation.
#
# `--selftest` proves all of the above against fabricated lane logs, needs no
# toolchain, and is the answer to "has anyone ever seen this gate fail?".
#
# ---- AND THE ONE THING ALL OF THAT STILL COULD NOT SAY (2026-08-07) ------------
#
# Everything above makes an empty run *legible*. It does not make it *fail*, and
# on 2026-08-05 and again on 2026-08-08 a lane read the exit code instead of the
# headline:
#
#   * `rungs/2026-08-05-w-subclass.md` §10.1 — "A bare `scripts/gate.sh --jobs 6`
#     from a worktree reported 18 SKIP, exit 0. … a lane that read the exit code
#     would have banked a green gate over nothing graded."
#   * lane `w-root`, merge `f57fe61e` — the identical run, three days later.
#     `compilers/` is gitignored so it does not follow a `git worktree add`, and
#     the `../wibo` fallback resolves relative to the WORKTREE, not the main repo.
#
# That is the **thirteenth** recorded instance of *absence read as success* here,
# and the first two payments both produced a fix that made the absence more
# legible. Legibility was not the binding constraint. **Nobody read the headline.**
#
# `SKIPPED` still exits 0 — CLAUDE.md requires the toolchain-absent path to
# degrade cleanly, the portable lane has no toolchain BY DESIGN, and turning that
# into a hard red would trade one silent failure for a noisy one in the only lane
# entitled to be empty. So the caller states its expectation instead:
#
#     scripts/gate.sh --require-graded          (or C2RS_GATE_REQUIRE_GRADED=1)
#
# **The demand is the CALLER's, not the gate's.** A lane in a worktree that
# intends to land work sets it and gets exit 1 over an unresolvable toolchain; the
# portable lane does not set it and gets the documented exit 0. Nothing about the
# default moves, and the exit-code contract below is unchanged when it is unset.
#
# Three properties, and each is the reason for the shape it has:
#
#   * **It is a POSITIVE check on a COUNT, never an enumeration of the ways a run
#     can be empty** — the standing mitigation, from the incident where a lane
#     `sed`-ed a number out of a report and read the missing number as `0`. The
#     quantity is `graded`, summed over the whole gate: fixture-verdicts from the
#     lanes' `LANE-RESULT graded=` fields, plus the sweep's `graded=`, plus the
#     cross's. A field that is absent or unparseable counts **0**, which is the
#     safe direction here — it can only make the demand FAIL, never pass.
#   * **It is checked at the LAST POINT WHERE EVERY REMAINING OUTCOME EXITS 0.**
#     One site, not four. Every branch before it already returns 1; every branch
#     after it — `SKIPPED`, `PASS (LANES FILTERED)`, `PASS (SAMPLED)`, `PASS`, and
#     any zero-exit outcome a future lane adds — is behind it by construction. An
#     enumeration of today's empty outcomes would be blind to tomorrow's.
#   * **It DUPLICATES NOTHING.** The gate already fails on a partial skip, on a
#     lane with `graded=0`, on a vacuous sweep, and on `NO-RESULT`; every one of
#     those returns 1 before the demand is consulted. The demand only reaches
#     outcomes that were going to exit 0.
#
# **SAMPLED and `--lane`-filtered runs SATISFY the demand, deliberately.** They
# graded something — a strided sample of 400 cases is 400 gradings — and the
# demand is `graded > 0`, not `graded == the whole corpus`. Both already refuse to
# print an unqualified PASS and say in the headline what they did not establish,
# which is the right instrument for "less than everything"; conflating it with
# "nothing at all" would make one flag mean two things and would give a lane no
# way to iterate under the demand. **A partial skip is likewise not covered**, for
# the stronger reason that it is already a FAIL: covering it again would be a
# second implementation of a rule that has one, which is the "one rule, two
# implementations" shape `docs/GAPS.md` §6 keeps recording.
#
# And the smaller half of the same finding: **when the gate skips, it now says
# WHICH path did not resolve** — see `toolchain_hint`. w-root read a clean skip
# and had to work out for itself that a worktree cannot see `compilers/`.
#
# ---- the GENERATED SWEEP is part of this gate (2026-08-04) ----------------------
#
# `scripts/expr_sweep.sh` enumerates ~14.5k small TUs and grades every one against
# the real `c2.dll`. Until today this file had **zero** references to it, and that
# blindness is not theoretical: board **#232** was a live `Port=Mismatch` — a
# refusal that had become a wrong emit, the one direction the correctness rule
# exists to forbid — and it survived **255 commits** (`d0d8a98..be86f9d`, two
# days) while every lane's gate run and every coordinator re-gate came back
# green. Count it yourself before requoting it: the `376` on #232's row is the
# BISECT RANGE from the last recorded green sweep, which starts before the defect
# existed, and it is a different quantity from how long the defect survived. The 12 mode lanes grade hand-built
# fixtures and `c2rs gap` grades workload TUs; **neither generates that shape.** A
# check that runs when somebody remembers it is a check that does not run — the
# same defect `lanes.txt` was written for, one level out.
#
# So the sweep is now a ROW in the table above, produced by the same
# walk-and-count discipline as a lane and subject to the same rules:
#
#   * It runs UNCONDITIONALLY. There is no `--no-sweep`; an omittable check is an
#     omitted check, and this project has thirteen recorded instances of an
#     absence reading as a success — the thirteenth being the one `--require-graded`
#     above was written for.
#   * Its POSITIVE check is re-derived here, not believed. The sweep prints
#     `sweeping R of T generated cases` (and `sweep_gen.py` has already reconciled
#     T against the `.cpp` on disk) and then `checked=C mismatches=M`. **This gate
#     fails unless `C == R`** — a run that graded fewer cases than it selected is a
#     dead worker, not a pass, and `M == 0` over `C == 0` is the vacuous green the
#     whole file exists to forbid.
#   * `--sweep-cases N` is the only way to grade less than the whole corpus, and a
#     sampled run **cannot print an unqualified PASS** — the verdict reads
#     `GATE: PASS (SWEEP SAMPLED)` and says what it did not establish, exactly as
#     `GATE: SKIPPED` does. The sample is a STRIDE across the sorted case list, not
#     a prefix. Measured: a 400-case budget reaches **1 of 47 fragments** as a
#     prefix and **46 of 47** as a stride, and #232's case is **line 9,538 of
#     14,484** — a prefix cheap enough to want was blind to it by construction.
#   * The sweep grades the SAME PINNED BINARY as the lanes (`C2RS_BIN`), so the
#     table is one run of one binary rather than two runs that might not be.
#   * `C2RS_SWEEP_ONLY` is unset for the gate's run. It filters fragments and makes
#     the total meaningless by design; a gate over a filtered corpus is not a gate.
#
# COST, measured on this box 2026-08-04 (32 cores, warm capture cache), and it is
# the whole basis of the "unconditional" decision:
#
#     12 lanes alone, --jobs 8                      7 s
#     sweep alone, serial (as it was written)   9 min 51 s
#     sweep alone, --jobs 8                     1 min 26 s
#     THIS GATE, --jobs 8                       1 min 34 s
#
# Both sweep runs printed `checked=14484 mismatches=0` — the parallel split is an
# equivalence, not an approximation.
#
# RE-MEASURED 2026-08-08 (lane w-throughput), 18 lanes and a 19,556-case corpus,
# `--require-graded`, box carrying two other lanes — the table above is a
# 12-lane/14,484-case measurement and is kept as the record of what was true then:
#
#     THIS GATE, --jobs 4 (the old default)    ~300 s  (sweep 262 s, cross 27 s)
#     THIS GATE, --jobs 16 (today's default)    105 s  (sweep  82 s, cross 21 s)
#
# Same verdict block at both: 18/18 lanes, 5,184 fixture-verdicts, sweep
# `checked=19556 mismatches=0 graded=19460 ungraded=96 unknown=0`, cross
# `checked=90812 mismatches=0 graded=90424 ungraded=388`. See the block above
# `jobs=` for why 16 and not 24, and for the killed-worker demonstration.
#
# Every case is an independent `c2rs diff` and
# the loop was serial for no reason. **Parallelising it is what makes
# "unconditional" affordable: the trade-off was resolved by removing the cost, not
# by making the check optional.**
#
# ---- AND THE INSTRUMENTS THEMSELVES ARE A ROW NOW (2026-08-08, board #1406) ----
#
# Everything above grades the PORT. `hatch-red` grades an INSTRUMENT: it runs
# `work/w-hatch/hatch_red.py`, which fires every one of `work/w-front3/hatch.py`'s
# refusals on purpose — 14 arms, 11 red, 3 green, 10 distinct leading words.
# `hatch.py` is the frontier ladder's lift, and its `revert` had already discarded
# a peer lane's unstaged fix (board #1380) before the refusals existed. The arms
# were run by hand and nothing executed them; this row is what executes them.
#
# It differs from every other row here in three ways, all deliberate:
#
#   * It needs NO TOOLCHAIN, so it is not part of the all-skip rule and it still
#     runs on the portable lane.
#   * It runs FIRST, before `pin_harness` — the arms write into `crates/` and
#     restore it, and the gate's one `cargo build` must not start alongside them.
#   * A DIRTY `crates/` (or a hatch whose needles have drifted out of the tree)
#     is `REFUSED`, not `FAIL`. Those are properties of the tree, not of
#     `hatch.py`, and a lane mid-wave is routinely in the first. `REFUSED` exits 0
#     and forfeits the unqualified headline — `GATE: PASS (HATCH-RED REFUSED)` —
#     which is the same treatment `SAMPLED` and `SKIPPED` get.
#
# Its arms are NOT counted toward `--require-graded`. That demand is about the
# port, and the arms of a `work/` script say nothing about the port.
#
# ---- AND THE SECOND INSTRUMENT ROW (2026-08-08, board #1406's second half) ----
#
# `ladder-red` is the same idea aimed at the OTHER half of the frontier ladder.
# `work/w-front3/ladder.py` is what climbs a TU's refusal ladder and prices it;
# `work/w-ladders/ladder_red.py` fires its `LADDER-NOWIDTHTABLE` guard three ways
# and checks the width table it derives from the tree two ways — 5 arms, 3 red,
# 2 green, and **5 of 5 fail against the pre-`w-ladders` `ladder.py`**, which is
# what makes it non-vacuous. It was run by hand and nothing executed it. #1406's
# first half was closed by the row above; this is the second.
#
# Same three deliberate differences, with ONE shape change that matters: the
# ladder arms never write into `crates/` — `EXPR_RS` is repointed at a temp file
# under `work/` — so this row's interlock is **narrow**. It refuses over the ONE
# tree file the arms read (`crates/c2-il/src/func/body/expr.rs`), not over all of
# `crates/`. Copying `hatch-red`'s wide interlock here would produce a row that
# `REFUSED`s for reasons that have nothing to do with it, which is how a row
# stops being read.
#
# ---- THE RUN TREE, REAPED, AND THE DISK RED TOLD APART FROM THE RED RED --------
#
# This gate writes a ~112 MB run tree per invocation and, until 2026-08-05, **never
# reaped one**. On a night with twenty lanes each re-gating a few times, `/tmp` —
# a 47 GB tmpfs on this box — fills, and then a gate goes red for a reason that has
# nothing to do with the port. It has already happened once and it is on record:
# `rungs/_2026-08-05-w-reach.md` §8.1 has a merge-base run whose `mode-cross` row
# came back `NO-RESULT` on
#
#     /tmp/c2rs-gate-484396/cross/lane-results/Ox: No space left on device
#
# with ~190 leftover trees on the tmpfs. The lane had to reason its way out of a
# **false red** by hand. That is this project's most-repeated defect wearing its
# other face: not *absence read as success*, but **resource exhaustion read as a
# correctness failure**. Sixteen instances of the family are on record, and the
# generalizing fix is always the same one — **a positive check with a printed
# count**. So:
#
#   * **Reaping is observable, never silent.** Every run prints what it reaped,
#     what it kept and why, and how much space the removal freed. A reaper that
#     ran quietly would be indistinguishable from a reaper that did not run — the
#     same shape as every other silence this file exists to forbid.
#   * **A reaper must never delete a tree a CONCURRENTLY RUNNING gate is using.**
#     Several lanes run this script at once on this box; a lane once spent a day
#     misdiagnosing a red that came from another lane changing shared state
#     mid-flight (board #294). A tree is kept if a live process owns it, where
#     "owns" means: its `gate.pid` (or, for pre-2026-08-05 trees, the pid in its
#     name) is alive **and** `/proc/<pid>/cmdline` says that pid is a `gate.sh`.
#     A pid that is alive but demonstrably something else is **pid reuse** — the
#     box mints hundreds of these a night — and that tree is reapable. Anything
#     that cannot be established (no `/proc`, unreadable cmdline) resolves to
#     **KEEP**: "unknown" must not mean "delete".
#   * **The run that just finished keeps its logs.** The gate's own output points
#     at `/tmp/c2rs-gate-<n>/<lane>.log`, so reaping the newest finished tree
#     would delete the thing the previous line just told somebody to read. The
#     `C2RS_GATE_KEEP` (default 3) most recent finished trees are kept.
#   * **Free space is checked UP FRONT and the check prints its counts** — free
#     bytes, free inodes, and the floors they are being compared against. Below a
#     floor, the gate stops **before grading anything** and prints
#     `GATE: FAIL (DISK)`, **exit 3**, which is not the exit code of a mismatch.
#   * **And when the red arrives mid-run anyway**, the verdict says which kind of
#     red it is. A FAIL whose logs carry `No space left on device` and whose
#     mismatch count is **0** prints a DISK banner: nothing about the port's bytes
#     was established. A FAIL carrying a **mismatch** never gets that banner —
#     bytes were compared and they differed, and a mismatch outranks every other
#     piece of work whatever else went wrong on the box.
#
# ---- AND A KEPT TREE IS KEPT FOR ITS LOGS, WHICH ARE 1.6 % OF IT ---------------
#
# The reaper above bounds the count. It does not bound the COST, and the cost is
# where the remaining inodes were. Measured on this box 2026-08-06, on four real
# finished trees left by a night of gating:
#
#     16,983 inodes per tree, of which 16,750 (98.6 %) are `sweep/`
#     16,710 of those are generated `*.cpp` cases
#     everything the gate's own output points at — 39 `*.log`, 18 `*.status`,
#     `results.tsv`, `cross/*.report`, `lanes/*/report.txt` — is 273 inodes
#
# So `C2RS_GATE_KEEP=3` was reserving ~51k inodes, 4.9 % of a 1,048,576-inode
# `/tmp`, to preserve 819 inodes' worth of logs, and holding it for as long as the
# box stays up — 2 weeks 2 days when this was measured, with nothing aging `/tmp`.
# That is not the accumulation defect (the reaper fixed that, and the count was
# correctly bounded at KEEP+1 the whole time) and it is not a leak. It is a
# **standing reservation nobody chose**, and on a `/tmp` shared with every other
# lane on the box it was the single largest occupant.
#
# The cases are not evidence. `scripts/expr_sweep.sh` regenerates them from
# `scripts/sweep.d` via `sweep_gen.py` on every run — it `rm -f`s the whole case
# set and rebuilds it before grading, precisely so a stale case cannot be graded.
# What IS evidence is the grading result, and that lives in `sweep/parts/` and
# `sweep/cases.txt`, which are 30-odd inodes and are kept.
#
#   * **Kept trees beyond `C2RS_GATE_KEEP_CASES` (default 1) lose their case
#     corpus and nothing else.** Every log, status, report, flags file, scan.jsonl
#     and graded part survives. 16,983 -> 273 inodes, a 98.4 % cut, and the steady
#     state goes ~68k -> ~17.8k.
#   * **Only a GREEN tree is stripped.** A tree whose `results.tsv` carries any
#     non-PASS row keeps its cases, because a mismatch is exactly when somebody
#     opens the `.cpp` the report names. A tree with no `results.tsv`, or an
#     unreadable one, is **not green** and is not stripped: unknown must not mean
#     delete, the same rule the pid check already follows.
#   * **A stripped tree says so, in the directory where the cases were.**
#     `sweep/CASES_STRIPPED` records the count removed, the run's verdict, and the
#     one command that rebuilds them. Cases that had merely vanished would be this
#     project's oldest defect wearing yet another face — absence read as a fact
#     about the corpus rather than as a fact about the reaper.
#   * **And it is counted out loud**, in the same `reap:` summary as everything
#     else. A strip nobody can see is a strip nobody can audit.
#
# `--selftest` drives all of it: the pid-liveness rule against a real live gate, a
# real live non-gate and a real dead pid; the reaper against a fabricated tree
# directory containing a stale tree, a reused-pid tree, a live-gate tree, a
# within-window tree and the current run dir; the stripper against a green tree, a
# mismatched tree and a verdict-less tree, asserting in each case both what went
# and what stayed; the disk floor from both sides; and the ENOSPC discrimination
# with and without a mismatch beside it.
#
# ---- usage ---------------------------------------------------------------------
#
#   scripts/gate.sh                       run every lane in the registry + the sweep
#   scripts/gate.sh --lane O1-Oi-EHsc     run named lanes only (repeatable)
#   scripts/gate.sh --jobs 24             lanes in parallel (default 16); also the
#                                         sweep's grading concurrency
#   scripts/gate.sh --sweep-cases 400     STRIDED subset — never an unqualified PASS
#   scripts/gate.sh --cross-cells 4000    ditto, for the mode-cross row
#   scripts/gate.sh --list                print the registry and exit
#   scripts/gate.sh --check               validate the registry, the corpus's shape
#                                         coverage AND the gate's own tree-integrity
#                                         arms (#2668/#2907); no toolchain, no compiler
#   scripts/gate.sh --selftest            prove the gate fails when it should
#   scripts/gate.sh --work DIR            run directory (default /tmp/c2rs-gate-$$)
#   scripts/gate.sh --reap-only           run the reaper and the disk preflight and
#                                         STOP. Grades nothing, so it is never a
#                                         verdict; exit 0 = clear, 3 = below a
#                                         floor. This is how you reclaim /tmp, and
#                                         how `--reap-dry-run` becomes affordable.
#   scripts/gate.sh --no-reap             keep every old run tree (see the block
#                                         above); the disk check still runs
#   scripts/gate.sh --reap-dry-run        classify every run tree and print what
#                                         WOULD be reaped, removing nothing. This
#                                         is how the concurrency rule is checked
#                                         against a live shared /tmp without
#                                         betting other lanes' logs on it.
#   scripts/gate.sh --require-graded      THE CALLER DEMANDS A GRADED RUN. Any
#                                         outcome that would exit 0 having graded
#                                         **0** units becomes exit 1 and says so.
#                                         Opt-in; unset, nothing below moves. Run
#                                         mode only — see the block above for why
#                                         SAMPLED and `--lane` runs satisfy it.
#   scripts/gate.sh --allow-dirty-crates  GRADE AN UNCOMMITTED `crates/` ANYWAY.
#                                         Without it, a `crates/` that differs from
#                                         HEAD is REFUSED (exit 4) and nothing runs
#                                         — boards #2668/#2907, where the gate
#                                         silently reverted a lane's work and then
#                                         graded the reverted tree. With it, the run
#                                         proceeds, NAMES itself on every headline,
#                                         and the one row that writes into `crates/`
#                                         is refused rather than run. It never
#                                         re-enables the restore.
#
# every run prints `graded tree: <hash> (N files)` twice — once before any row can
# write, once after everything has — over `crates/ fixtures/ scripts/`. That, and
# not `git rev-parse HEAD`, is the identity of what a run graded: HEAD is exactly
# what lied in #2907. Two hashes that differ mean the tree moved mid-run and the
# verdict is evidence about neither end.
#
# exit codes:  0 = PASS / SKIPPED / SAMPLED   1 = a real gate failure
#              2 = usage   **3 = out of disk, and nothing was graded**
#              **4 = refused: `crates/` is dirty, and nothing was graded or touched**
#
#              These are the codes WITHOUT `--require-graded`, and they do not
#              move when it is unset. **With it set**, exactly one thing changes:
#              an outcome that would have exited 0 while `graded == 0` — today
#              that is `GATE: SKIPPED`, tomorrow whatever else grades nothing —
#              exits **1** under `GATE: FAIL (NOTHING GRADED)` instead. PASS,
#              SAMPLED, `LANES FILTERED`, the usage code and the disk code are
#              untouched; a partial skip and a vacuous lane were already 1.
#              Combining it with `--reap-only`, which grades nothing by
#              construction, is a contradiction and is refused as usage (2).
#
# env:  C2RS_GATE_REQUIRE_GRADED  1 = the same demand as `--require-graded`, for
#                             callers that set an environment rather than a
#                             command line (a lane's `env.sh`, CI). Any other
#                             value, including unset, leaves the default.
#       C2RS_GATE_KEEP        finished run trees to keep (default 3)
#       C2RS_GATE_KEEP_CASES  of those kept trees, how many keep their REGENERABLE
#                             sweep corpus (default 1 = the newest only; see the
#                             98.4 % block below). 0 strips every kept tree.
#       C2RS_GATE_MIN_MB      free-space floor, MiB (default 2048)
#       C2RS_GATE_MIN_INODES  free-inode floor (default 150000 = 3x one run's
#                             MEASURED PEAK draw of 50,250; see below)
#
# ---- ACCUMULATION *AND* CONCURRENCY, and the first measurement got it wrong ----
#
# A FINISHED run tree is 112 MB and 16.6k inodes, so ~63 accumulated trees exhaust
# a 1,048,576-inode /tmp and ~430 exhaust its 47 GB. On that arithmetic alone,
# 6 concurrent gates are ~100k inodes — under 10 % — and accumulation is the whole
# story. **That arithmetic was wrong, and the gate's own new low-water instrument
# is what caught it**: a real green run measured here 2026-08-05 drew
# 853,187 -> 802,937 free inodes, a peak of **50,250**, because a run in flight
# holds the lanes' scratch and the sweep's and the cross's corpora all at once and
# only settles to 16.6k when it finishes. So one run costs **3.0x in flight what
# it leaves behind**, ~21 concurrent runs exhaust the inode table on their own, and
# on a twenty-lane night the transient term is the same order as the accumulated
# one. Both mechanisms are real; the reaper answers the accumulation and the
# preflight answers the transient, which is why this file has both.
#
# Lane run directories stay PER LANE, inherited from `mode_lane.sh`, which uses one
# per mode precisely because a shared directory had concurrent lanes overwriting
# each other's flags file and report and the mismatch count then came out of
# whichever report won.
set -eu

TAB=$(printf '\t')
repo_root="$(cd "$(dirname "$0")/.." && pwd)"
registry="${C2RS_LANES:-$repo_root/scripts/lanes.txt}"
# ---- THE DEFAULT CONCURRENCY, AND WHY IT IS 16 AND NOT 4 ----------------------
#
# `jobs=4` stood here from this file's creation commit (`25def085`, 2026-07-31)
# to 2026-08-08 and was never revisited: `git log -L347,347` has exactly one
# entry and no board row explains the number. It is an untuned constant, not a
# safety limit — this is a 32-core box, and it governs three legs (lane
# parallelism, `C2RS_SWEEP_JOBS`, and the `C2RS_JOBS` handed to `mode_cross.sh`),
# all three of which are embarrassingly parallel over per-case scratch dirs.
#
# MEASURED on this box 2026-08-08, same tree, same corpus, back to back on an
# IDLE box:
#
#     gate.sh --require-graded                ~300 s   (sweep 266 s, cross  27 s)
#     gate.sh --jobs 16 --require-graded       105 s   (sweep  82 s, cross  21 s)
#
# and **the two verdict blocks are identical, digit for digit** — 18/18 lanes,
# 5,184 fixture-verdicts, sweep `checked=19556 mismatches=0 graded=19460
# ungraded=96 unknown=0`, cross `checked=90812 mismatches=0 graded=90424
# ungraded=388`.
#
# Lane w-throughput re-took the sweep leg at both settings on a CONTENDED box
# (four `gate.sh` processes at once, load average 7-100 across the pair) and got
# the sweep leg 262 s at 4 and 309 s at 16 (the flag reads BACKWARDS when the box is at load 100) — the counts were identical again, which is the part that
# does not move with load. The parallel split is an EQUIVALENCE, not an
# approximation, and
# that is a statement this file has already made once for 4 -> 8 in the cost
# table below. Nothing about what is graded moves with this number: the corpus is
# generated before the split, `awk` assigns every case to exactly one worker by
# line number, and the worker counts are reconciled against the selected count
# afterwards.
#
# **Why 16 and not 24.** The knee is ~24 (68 s standalone; 32 gives 70 s, i.e.
# nothing) — so 24 buys 14 s more on an IDLE box. 16 is the physical core count
# (32 logical), and past it each extra worker is a whole `cl` + wibo process tree
# competing for a hyperthread sibling: the first twelve workers buy 180 s and the
# next eight buy 14 s. The box also routinely carries two or three lanes gating
# at once, where a per-gate 24 means 72 concurrent capture trees and the 68 s no
# longer holds. 16 is the value that is fast when the box is busy and within 20 %
# of the best when it is idle.
#
# **What breaks first, checked rather than assumed.** Not inodes: the in-flight
# draw is set by the CORPUS, not by the concurrency — measured 19,810 free-inode
# low-water draw at 4 workers and 19,885 at 16, against the 150,000 floor the
# preflight enforces (`/tmp` had 753,110 free on the run that produced the table
# above). Not wibo: board #201 measured 48 concurrent captures to one distinct
# `.gl` sha and censuses at 1/14/32 jobs byte-identical, and `census_gate` has
# been running 2x16 capture threads on every merge for weeks. The failure
# direction of over-concurrency is a DISK red, which `GATE: FAIL (DISK)` / exit 3
# already tells apart from a mismatch.
#
# **The false-green question, answered by making it happen.** A worker that dies
# at higher concurrency must not shorten the corpus quietly. It cannot: each
# worker writes its own `checked.N` only after finishing its chunk, and a missing
# file makes the sum fall short of the selected count, which is `FATAL: selected
# R cases and only C were graded` (exit 3) inside `expr_sweep.sh` and, redundantly,
# `SHORT — selected R cases, reached C` in `sweep_verdict` here. Lane w-throughput
# verified that by KILLING a worker mid-run rather than by reading the code;
# the transcript is in `docs/rungs/2026-08-08-w-throughput.md` §3.
#
# `--jobs` is unchanged as an override and the header line below still prints the
# value actually used, so a run's own output says what concurrency produced it.
jobs=16
work=""
want=""
mode=run
sweep_cases=0
cross_cells=0
reap=1
: "${C2RS_GATE_KEEP:=3}"
# 1, not KEEP. A kept tree is kept for its LOGS, and 98.4 % of it is not logs —
# see the block above. The newest finished run keeps its cases because that is the
# one somebody actually opens; the rest keep every log, report, flags file and
# graded part and lose only what `sweep_gen.py` will rebuild on demand.
: "${C2RS_GATE_KEEP_CASES:=1}"
: "${C2RS_GATE_MIN_MB:=2048}"
# 150k, not 50k. A FINISHED run tree is 16.6k inodes, but a run IN FLIGHT peaks
# much higher — the lanes' scratch, the sweep and the cross all exist at once.
# Measured on a real green run on this box 2026-08-05: free inodes on /tmp went
# 853,187 -> 802,937, a peak draw of **50,250**, 3.0x the residual. A floor of
# 50,000 would therefore have been *exactly one run's peak*: the preflight would
# pass with 50,000 free and the run would then exhaust the filesystem it had just
# certified. The floor is 3x the measured peak, and the measurement is the reason.
: "${C2RS_GATE_MIN_INODES:=150000}"

# THE CALLER'S DEMAND. Default 0 — the documented exit-code contract is what you
# get unless somebody asks for more. Only the exact string `1` enables it: a
# half-set variable (`C2RS_GATE_REQUIRE_GRADED=`, `=no`, `=0`) must not silently
# arm a check that turns exit 0 into exit 1, and must not silently DISARM one
# either, so the value is compared rather than tested for emptiness.
require_graded=0
if [ "${C2RS_GATE_REQUIRE_GRADED:-0}" = 1 ]; then require_graded=1; fi

# THE DIRTY-`crates/` ESCAPE HATCH — board #2907/#2668, lane `w-gatefix`.
# Default 0: a graded run over a `crates/` that differs from `HEAD` is REFUSED,
# because this gate has rows that write there and one of them silently reverted
# a lane's uncommitted work and then graded the reverted tree. Compared against
# the exact string `1` for the reason `require_graded` is: a half-set variable
# must neither arm nor disarm a check that changes what the gate will run.
allow_dirty=0
if [ "${C2RS_GATE_ALLOW_DIRTY_CRATES:-0}" = 1 ]; then allow_dirty=1; fi

while [ $# -gt 0 ]; do
    case "$1" in
        --allow-dirty-crates) allow_dirty=1 ;;
        --list)     mode=list ;;
        --check)    mode=check ;;
        --selftest) mode=selftest ;;
        --reap-only) mode=reap ;;
        --lane)     shift; want="$want $1" ;;
        --jobs)     shift; jobs="$1" ;;
        --sweep-cases) shift; sweep_cases="$1" ;;
        --cross-cells) shift; cross_cells="$1" ;;
        --work)     shift; work="$1" ;;
        --no-reap)  reap=0 ;;
        --reap-dry-run) reap=2 ;;
        --require-graded) require_graded=1 ;;
        --registry) shift; registry="$1" ;;
        -h|--help)  sed -n '2,/^set -eu$/p' "$0" | sed '$d'; exit 0 ;;
        *) echo "gate.sh: unknown argument '$1' (try --help)" >&2; exit 2 ;;
    esac
    shift
done
[ -n "$work" ] || work="/tmp/c2rs-gate-$$"

# --------------------------------------------------------------------------------
# THE DEMAND MEETS THE MODES THAT GRADE NOTHING BY CONSTRUCTION.
#
# `--require-graded` is a statement about a VERDICT-PRODUCING run, and `run` is the
# only mode that produces one. The other four grade nothing on purpose and each
# says so in its own output, so the demand cannot be silently satisfied by them —
# but it can be silently IGNORED by them, and a demand that is quietly ignored is
# the same shape as everything else in this file.
#
# `--reap-only` is refused outright. It is the one non-run mode a caller could
# plausibly run INSTEAD OF a gate and then read the exit code of — its documented
# codes are 0 = clear and 3 = below a floor, and 0 there means "the disk is fine",
# never "the port is fine". Asking for a graded run and for the mode that grades
# nothing is a contradiction in the command line, and a contradiction is usage (2).
#
# `--list`, `--check` and `--selftest` are inspection modes; nobody substitutes
# them for a gate. The demand does not apply, and they SAY it does not apply,
# because the alternative is a lane exporting `C2RS_GATE_REQUIRE_GRADED=1` in its
# env.sh and believing every command it then runs is under the demand.
# --------------------------------------------------------------------------------
if [ "$require_graded" -eq 1 ]; then
    case "$mode" in
    reap)
        echo "gate.sh: --require-graded and --reap-only contradict each other." >&2
        echo "  --reap-only grades NOTHING by construction; it is housekeeping and" >&2
        echo "  never a verdict (exit 0 there means the disk is clear, not that the" >&2
        echo "  port is right). Asking for a graded run and for the mode that cannot" >&2
        echo "  grade is a contradiction, not a run. Drop one of the two." >&2
        exit 2 ;;
    list|check|selftest)
        echo "gate.sh: note — --require-graded has no effect in --$mode; it binds the" >&2
        echo "  graded run only. This mode grades nothing and does not claim to." >&2
        require_graded=0 ;;
    esac
fi

# --------------------------------------------------------------------------------
# The registry, parsed once. `slug<TAB>flags`, one per line, comments stripped.
# --------------------------------------------------------------------------------
parse_registry() {
    _pr_src="$1"; _pr_dst="$2"
    if [ ! -f "$_pr_src" ]; then
        echo "FATAL: no lane registry at $_pr_src" >&2
        return 1
    fi
    # Every line with content must BECOME a lane. A row that carries a slug and no
    # flags used to be dropped by the `NF >= 2` filter, so `--list` reported one
    # lane fewer than the file contains and nothing said so — a registry silently
    # short a row is the same absence-reads-as-success bug one layer further out
    # than the one this gate was built for. Malformed rows are named and fatal.
    sed 's/#.*//' "$_pr_src" | awk 'NF > 0' > "$_pr_dst.rows"
    awk 'NF >= 2 { slug=$1; $1=""; sub(/^[ \t]+/,""); printf "%s\t%s\n", slug, $0 }' \
        "$_pr_dst.rows" > "$_pr_dst"
    _pr_rows=$(wc -l < "$_pr_dst.rows")
    _pr_n=$(wc -l < "$_pr_dst")
    if [ "$_pr_n" -ne "$_pr_rows" ]; then
        echo "FATAL: $_pr_src has $_pr_rows non-comment rows but only $_pr_n parse as lanes." >&2
        echo "  A row needs a slug AND at least one flag. Offending row(s):" >&2
        awk 'NF > 0 && NF < 2 { printf "    %s\n", $0 }' "$_pr_dst.rows" >&2
        return 1
    fi
    # An EMPTY registry is a gate that runs nothing and exits 0 — the exact shape
    # this whole file exists to make impossible. It is a hard error, never a pass.
    if [ "$_pr_n" -eq 0 ]; then
        echo "FATAL: lane registry $_pr_src defines NO lanes." >&2
        echo "  A gate with an empty lane list grades nothing and would exit 0." >&2
        return 1
    fi
    _pr_dup=$(cut -f1 "$_pr_dst" | sort | uniq -d)
    if [ -n "$_pr_dup" ]; then
        echo "FATAL: duplicate lane slug(s) in $_pr_src: $_pr_dup" >&2
        echo "  Two rows under one slug means one silently replaces the other's" >&2
        echo "  result while the table still shows the expected number of rows." >&2
        return 1
    fi
    return 0
}

# --------------------------------------------------------------------------------
# Verdict for ONE lane, derived from its log. Deliberately a pure function of the
# log text, so `--selftest` can drive it with fabricated logs and no toolchain.
#
# Emits: <verdict>|<graded>|<total>|<match>|<mismatch>|<detail>
# --------------------------------------------------------------------------------
lane_verdict() {
    _lv_log="$1"; _lv_status="${2:-}"

    if [ ! -f "$_lv_log" ]; then
        echo "NO-RESULT|0|0|0|0|the lane produced no log at all"
        return 0
    fi
    _lv_line=$(grep -m1 '^LANE-RESULT ' "$_lv_log" 2>/dev/null || true)
    if [ -z "$_lv_line" ]; then
        echo "NO-RESULT|0|0|0|0|log has no LANE-RESULT line (exit ${_lv_status:-?})"
        return 0
    fi

    _lv_v=$(printf '%s\n' "$_lv_line" | awk '{print $2}')
    _lv_g=$(printf '%s\n' "$_lv_line" | sed -n 's/.* graded=\([0-9][0-9]*\).*/\1/p')
    _lv_t=$(printf '%s\n' "$_lv_line" | sed -n 's/.* total=\([0-9][0-9]*\).*/\1/p')
    _lv_m=$(printf '%s\n' "$_lv_line" | sed -n 's/.* match=\([0-9][0-9]*\).*/\1/p')
    _lv_x=$(printf '%s\n' "$_lv_line" | sed -n 's/.* mismatch=\([0-9][0-9]*\).*/\1/p')

    # Every field must be PRESENT. An unparseable result line is a lane that did
    # not report — NOT a lane that reported zeros. That distinction is the entire
    # bug class this gate is built around.
    if [ -z "$_lv_g" ] || [ -z "$_lv_t" ] || [ -z "$_lv_m" ] || [ -z "$_lv_x" ]; then
        echo "NO-RESULT|0|0|0|0|malformed LANE-RESULT line"
        return 0
    fi

    case "$_lv_v" in
    SKIP)
        echo "SKIP|0|$_lv_t|0|0|toolchain absent"
        ;;
    PASS)
        # Re-derive rather than believe. A lane claiming PASS while having graded
        # nothing, or while carrying a mismatch, is a lane wrong about itself, and
        # the gate is the second opinion.
        if [ "$_lv_g" -eq 0 ]; then
            echo "FAIL|0|$_lv_t|$_lv_m|$_lv_x|claimed PASS having graded 0 of $_lv_t"
        elif [ "$_lv_x" -ne 0 ]; then
            echo "FAIL|$_lv_g|$_lv_t|$_lv_m|$_lv_x|claimed PASS with mismatch=$_lv_x"
        elif [ "${_lv_status:-0}" != "0" ]; then
            echo "FAIL|$_lv_g|$_lv_t|$_lv_m|$_lv_x|claimed PASS but exited $_lv_status"
        else
            echo "PASS|$_lv_g|$_lv_t|$_lv_m|$_lv_x|"
        fi
        ;;
    FAIL)
        if [ "$_lv_x" -ne 0 ]; then
            echo "FAIL|$_lv_g|$_lv_t|$_lv_m|$_lv_x|MISMATCH — the port emitted wrong bytes"
        elif [ "$_lv_g" -eq 0 ]; then
            echo "FAIL|$_lv_g|$_lv_t|$_lv_m|$_lv_x|vacuous — 0 of $_lv_t graded"
        else
            echo "FAIL|$_lv_g|$_lv_t|$_lv_m|$_lv_x|lane reported FAIL"
        fi
        ;;
    *)
        echo "NO-RESULT|0|0|0|0|unrecognized verdict '$_lv_v'"
        ;;
    esac
    return 0
}

# --------------------------------------------------------------------------------
# Verdict for the GENERATED SWEEP, derived from its log. Same contract as
# `lane_verdict`: a pure function of the log text, so `--selftest` drives it with
# fabricated logs and no toolchain.
#
# The sweep's own positive check is RE-DERIVED here rather than believed. It prints
#
#     sweeping 14484 of 14484 generated cases      <- selected R of generated T
#     checked=14484 mismatches=0                   <- graded C, found M
#
# and `sweep_gen.py` has already failed the run if T disagreed with the `.cpp` on
# disk. This function's job is the next link: **C must equal R.** A sweep whose
# workers died grades fewer cases than it selected and still prints
# `mismatches=0`, which is the exact shape of a green from an absence.
#
# Emits: <verdict>|<checked>|<selected>|<total>|<mismatch>|<detail>
# --------------------------------------------------------------------------------
sweep_verdict() {
    _sv_log="$1"; _sv_status="${2:-}"

    if [ ! -f "$_sv_log" ]; then
        echo "NO-RESULT|0|0|0|0|the instrument produced no log at all"
        return 0
    fi
    # SKIP is the toolchain-absent path and is the ONLY path allowed to have no
    # counts. It is checked first so a skipped run is never read as malformed.
    if grep -q '^SKIP: toolchain absent' "$_sv_log" 2>/dev/null; then
        echo "SKIP|0|0|0|0|toolchain absent"
        return 0
    fi

    # The unit word is deliberately NOT anchored: `expr_sweep.sh` says
    # "generated cases" and `mode_cross.sh` says "case-lane cells", and both are
    # ruled on by this one function. A second copy of these rules for the second
    # instrument is the "one rule, two implementations" shape `docs/GAPS.md` §6
    # keeps recording, and it is how the two would drift into disagreeing about
    # what a short count means.
    _sv_sel=$(sed -n 's/^sweeping \([0-9][0-9]*\) of \([0-9][0-9]*\) .*/\1/p' "$_sv_log" | head -1)
    _sv_tot=$(sed -n 's/^sweeping \([0-9][0-9]*\) of \([0-9][0-9]*\) .*/\2/p' "$_sv_log" | head -1)
    _sv_c=$(sed -n 's/^checked=\([0-9][0-9]*\) mismatches=\([0-9][0-9]*\).*/\1/p' "$_sv_log" | head -1)
    _sv_m=$(sed -n 's/^checked=\([0-9][0-9]*\) mismatches=\([0-9][0-9]*\).*/\2/p' "$_sv_log" | head -1)
    # `checked` is only "cases the loop reached". `graded` is "cases the ORACLE
    # ruled on", and until 2026-08-04 the two were reported as one number while
    # 96 of 14,635 cases had no reference obj at all — `c2rs diff` prints four
    # verdicts and the sweep's classifier recognized one of them. Both fields are
    # now required; a log carrying only the old line is NO-RESULT, not a log
    # reporting `ungraded=0`.
    _sv_g=$(sed -n 's/^checked=.* graded=\([0-9][0-9]*\).*/\1/p' "$_sv_log" | head -1)
    _sv_u=$(sed -n 's/^checked=.* ungraded=\([0-9][0-9]*\).*/\1/p' "$_sv_log" | head -1)
    _sv_k=$(sed -n 's/^checked=.* unknown=\([0-9][0-9]*\).*/\1/p' "$_sv_log" | head -1)

    # Both lines must be PRESENT. A missing `sweeping` line is a run whose corpus
    # never got enumerated; a missing `checked=` line is a run that died. Neither
    # is a run that found nothing.
    if [ -z "$_sv_sel" ] || [ -z "$_sv_tot" ]; then
        echo "NO-RESULT|0|0|0|0|log has no 'sweeping R of T' line (exit ${_sv_status:-?})"
        return 0
    fi
    if [ -z "$_sv_c" ] || [ -z "$_sv_m" ]; then
        echo "NO-RESULT|0|$_sv_sel|$_sv_tot|0|log has no 'checked=' line (exit ${_sv_status:-?})"
        return 0
    fi
    if [ -z "$_sv_g" ] || [ -z "$_sv_u" ] || [ -z "$_sv_k" ]; then
        echo "NO-RESULT|$_sv_c|$_sv_sel|$_sv_tot|$_sv_m|count line has no graded=/ungraded=/unknown= — a pre-2026-08-04 sweep, whose 'checked' includes cases the oracle never ruled on"
        return 0
    fi

    if [ "$_sv_c" -ne "$_sv_sel" ]; then
        echo "FAIL|$_sv_c|$_sv_sel|$_sv_tot|$_sv_m|SHORT — selected $_sv_sel cases, reached $_sv_c|$_sv_g|$_sv_u"
        return 0
    fi
    if [ "$_sv_c" -eq 0 ]; then
        echo "FAIL|0|$_sv_sel|$_sv_tot|$_sv_m|vacuous — 0 cases reached|0|0"
        return 0
    fi
    if [ "$_sv_k" -ne 0 ]; then
        echo "FAIL|$_sv_c|$_sv_sel|$_sv_tot|$_sv_m|UNRECOGNIZED verdict on $_sv_k case(s) — an unenumerated verdict is the next silence|$_sv_g|$_sv_u"
        return 0
    fi
    if [ "$_sv_g" -eq 0 ]; then
        echo "FAIL|$_sv_c|$_sv_sel|$_sv_tot|$_sv_m|vacuous — $_sv_c cases reached and NONE graded|0|$_sv_u"
        return 0
    fi
    if [ "$_sv_m" -ne 0 ]; then
        echo "FAIL|$_sv_c|$_sv_sel|$_sv_tot|$_sv_m|MISMATCH — the port emitted wrong bytes on $_sv_m case(s)|$_sv_g|$_sv_u"
        return 0
    fi
    if [ "$_sv_sel" -lt "$_sv_tot" ]; then
        echo "SAMPLED|$_sv_c|$_sv_sel|$_sv_tot|0|a STRIDED sample, not the corpus|$_sv_g|$_sv_u"
        return 0
    fi
    if [ "${_sv_status:-0}" != "0" ]; then
        echo "FAIL|$_sv_c|$_sv_sel|$_sv_tot|$_sv_m|graded every case cleanly but exited $_sv_status|$_sv_g|$_sv_u"
        return 0
    fi
    echo "PASS|$_sv_c|$_sv_sel|$_sv_tot|0||$_sv_g|$_sv_u"
    return 0
}

# ================================================================================
# WHICH TREE DID THIS RUN ACTUALLY GRADE?  (boards #2668 / #2907, lane w-gatefix)
# ================================================================================
#
# Until 2026-08-10 the only identity a gate run printed was `pin_harness`'s
# `tree <short HEAD>[-dirty]`, and **HEAD is exactly what lied**. Two runs of
# this gate, same invocation, same printed identity, different graded code:
#
#   | `crates/` when the gate started     | pinned binary  | printed identity |
#   |-------------------------------------|----------------|------------------|
#   | one uncommitted edit                | `0a252107b376` | `tree 0d0a74d2`  |
#   | the SAME edit, committed            | `6efc8913269f` | `tree b8801857`  |
#
# The first row is the defect: `hatch_red_run`'s arm-count probe
# (`hatch_red.py --list`) ran a `git checkout -- crates/` before the interlock
# that was supposed to prevent exactly that, the edit was gone, the run graded
# the pre-edit tree, and `-dirty` was ABSENT from the identity because the
# thing that computed it ran after the thing that cleaned the tree. Lane
# `w-seclayout` shipped a "byte-identical binary" claim off such a run and had
# to retract it (#2907); lane `w-xtea2` had to re-apply an edit the gate ate
# (#2668). Both fixes are below; this is the instrument that makes a third
# instance visible instead of silent.
#
# WHY A CONTENT HASH AND NOT A COMMIT
# -----------------------------------
# A commit id answers "what is checked in". A gate grades **what is on disk**,
# and the two diverge exactly when it matters — uncommitted work, a
# half-restored tree, a peer session writing into the worktree mid-run (which
# has happened here). So the identity is a hash of file CONTENT over the three
# directories a run's verdict depends on:
#
#   * `crates/`   — the port and the harness, i.e. the code under test;
#   * `fixtures/` — the corpus the fixture scan grades;
#   * `scripts/`  — the graders themselves, this file included.
#
# `docs/STATUS.md`'s generated block already pins a binary hash for the same
# reason; this is that precedent applied to the inputs rather than the output.
#
# The file COUNT travels with the hash. A digest computed over three files and a
# digest computed over seven hundred are both 12 hex characters, and a `find`
# that silently returned almost nothing would otherwise print a confident,
# stable, meaningless identity — this project's most-repeated defect (absence
# reads as success) aimed at the instrument built to stop it.
#
# Deterministic by construction: `LC_ALL=C sort -z` fixes the order, the digest
# is over `sha256sum`'s own `<hash>  <path>` lines so a RENAME moves it, and no
# mtime, inode or timestamp is read. `xargs -r` so an empty file list produces
# no `sha256sum` invocation at all — without it, `sha256sum` with no operands
# reads standard input and the gate hangs forever.
#
# Emits `<12hex>:<nfiles>`, or `:0` when it cannot be computed. Never fatal on
# its own: a missing `sha256sum` must degrade to "identity unavailable" and say
# so, not turn a green gate red.
#
# ---- AND THEN THE GATE VOIDED ITS OWN FIRST RUN (board #3048, lane w-keygen) --
#
# The identity above is correct and it fired on the gate itself. `expr_sweep.sh`
# imports `scripts/sweep_gen.py`, CPython writes the bytecode cache next to the
# source, and `scripts/` is one of the three graded directories. So the run
# CREATED its own 724th file, between its own two hashes, and the interlock —
# correctly, by its own rule — declared the run evidence about neither tree.
#
# It self-triggers exactly once per FRESH worktree, which is what every lane
# starts from (`setup_worktree.sh`), so **every lane's first gate run was void
# and the lanes that did not read both hashes reported it clean.** That is
# absence-read-as-success (trap 5) inside the merge gate itself, which is the
# one place this project cannot afford it. It also triggers in a WARM tree once
# per python minor version: #3048's own tree carried a `cpython-313` cache
# beside a `cpython-314` one, so "once per worktree" is a lower bound.
#
# THE RULE, AND WHY IT IS A RULE AND NOT A LIST
# --------------------------------------------
#   **The graded set is every file on disk under `GRADED_DIRS` that git is not
#   EXPLICITLY IGNORING.**
#
# Not "everything except `__pycache__`". An enumeration of the ways a byproduct
# can appear is the thing that fails here — this file's own history is a list of
# instruments that were exhaustive over the cases somebody had thought of. The
# repo already maintains exactly the declaration this needs: a byproduct has to
# be gitignored or `git status` becomes unusable, so `.gitignore` is a
# continuously-maintained, independently-motivated statement of "this is
# generated". `target/`, `*.obj`, `*.il`, `_CL_*`, `*.profraw`, `/work`,
# `**/*.tmp` and `__pycache__/` are all in it today, and the *next* byproduct
# will be in it too, for reasons that have nothing to do with this gate.
#
# FAIL-OPEN, DELIBERATELY, IN THE DIRECTION THAT OVER-GRADES
# ----------------------------------------------------------
# The filter SUBTRACTS from the filesystem walk; it does not replace it with a
# git listing. So:
#   * a file nobody gitignored is graded, whatever it is — a byproduct that was
#     never declared still voids the run, and the fix for that is one line of
#     `.gitignore`, not a change here;
#   * a TRACKED file is graded no matter what pattern it matches (`git ls-files
#     --others` cannot name it), so nothing that is checked in can fall out;
#   * an untracked file that is NOT ignored — a peer session's new source file,
#     a half-applied patch — is graded exactly as before. This is the property
#     `w-gatefix` established and it is not weakened: the identity is still
#     CONTENT, still not HEAD, and a rename still moves it.
#   * when there is no git to ask, the filter subtracts nothing and this is the
#     pre-#3048 function exactly.
#
# WHAT IT DOES NOT COVER, said here rather than discovered later:
#   * a writer that modifies a TRACKED file and restores it before the second
#     hash. Neither this nor any content hash can see that. `hatch_red.py` does
#     precisely this by design, which is why it has its own interlock;
#   * an undeclared byproduct, as above — that still voids, on purpose;
#   * a grading INPUT that someone gitignores under a graded directory would
#     silently leave the identity. Nothing does today (measured: the only
#     ignored files under these three directories on a warm tree are
#     `scripts/__pycache__/*.pyc`), and the count of what was excluded is
#     PRINTED beside the identity at both ends of every run so that the day it
#     stops being true is a number that moved rather than a silence.
#
# The rejected alternative was `PYTHONDONTWRITEBYTECODE=1` / `-B`, and it is
# rejected for three reasons, none of them performance: it changes what the gate
# RUNS in order to make a statement about what the gate MEASURES; it is specific
# to one writer, so the second writer is a second incident; and it would have
# MASKED this one — the interlock would have gone quiet without anybody
# learning that a gate row writes into the tree it grades.
# --------------------------------------------------------------------------------
GRADED_DIRS="crates fixtures scripts"

# The declared-byproduct set under `GRADED_DIRS`, one path per line, into
# <outfile>. Returns 0 when git ANSWERED (an empty answer is an answer) and 1
# when it could not be asked at all — the caller must be able to tell "this repo
# declares no byproducts here" from "there is no repo here to ask".
graded_ignored_list() {   # <root> <outfile> -> 0 asked / 1 unaskable
    _gi_root="$1"; _gi_out="$2"
    : > "$_gi_out" 2>/dev/null || return 1
    command -v git >/dev/null 2>&1 || return 1
    git -C "$_gi_root" rev-parse --git-dir >/dev/null 2>&1 || return 1
    ( cd "$_gi_root" && git ls-files -o -i --exclude-standard -- $GRADED_DIRS ) \
        > "$_gi_out" 2>/dev/null || return 1
    return 0
}

# How many files the identity below deliberately did NOT hash. Printed at both
# ends of every run: a subtraction nobody can see is a subtraction that grows.
# Emits `?` when git could not be asked, never `0` — an unavailable answer must
# not read as "nothing was excluded".
graded_ignored_count() {   # <root> -> <n> | '?'
    _gc_out="${TMPDIR:-/tmp}/c2rs-graded-ign-count.$$"
    if graded_ignored_list "$1" "$_gc_out"; then
        wc -l < "$_gc_out" | tr -d ' '
    else
        echo '?'
    fi
    rm -f "$_gc_out" 2>/dev/null || true
}

graded_tree_hash() {   # <root> [dirs] -> <12hex>:<nfiles>
    _gh_root="$1"; _gh_dirs="${2:-$GRADED_DIRS}"
    if [ ! -d "$_gh_root" ] || ! command -v sha256sum >/dev/null 2>&1; then
        echo ":0"; return 0
    fi
    # Capability probe, because a `grep` without `-z` would emit NOTHING and an
    # empty list reads as "identity unavailable" — loud, but for a reason that
    # has nothing to do with the tree. `-f /dev/null` is the empty pattern file:
    # it must match nothing, so `-v` must pass the input through.
    _gh_flt="${TMPDIR:-/tmp}/c2rs-graded-ign.$$"
    if printf 'a\0' | grep -zvxF -f /dev/null >/dev/null 2>&1 \
       && graded_ignored_list "$_gh_root" "$_gh_flt"; then
        _gh_filter=1
    else
        _gh_filter=0
        rm -f "$_gh_flt" 2>/dev/null || true
    fi
    if [ "$_gh_filter" = 1 ]; then
        _gh_n=$(cd "$_gh_root" && find $_gh_dirs -type f -print0 2>/dev/null \
            | LC_ALL=C sort -z | grep -zvxF -f "$_gh_flt" 2>/dev/null \
            | tr -dc '\000' | wc -c | tr -d ' ')
    else
        _gh_n=$(cd "$_gh_root" && find $_gh_dirs -type f 2>/dev/null | wc -l)
    fi
    [ -n "$_gh_n" ] || _gh_n=0
    if [ "$_gh_n" -le 0 ]; then rm -f "$_gh_flt" 2>/dev/null; echo ":0"; return 0; fi
    if [ "$_gh_filter" = 1 ]; then
        _gh_h=$(cd "$_gh_root" && find $_gh_dirs -type f -print0 2>/dev/null \
            | LC_ALL=C sort -z | grep -zvxF -f "$_gh_flt" 2>/dev/null \
            | xargs -0 -r sha256sum 2>/dev/null \
            | sha256sum | cut -c1-12)
    else
        _gh_h=$(cd "$_gh_root" && find $_gh_dirs -type f -print0 2>/dev/null \
            | LC_ALL=C sort -z | xargs -0 -r sha256sum 2>/dev/null \
            | sha256sum | cut -c1-12)
    fi
    rm -f "$_gh_flt" 2>/dev/null || true
    [ -n "$_gh_h" ] || { echo ":0"; return 0; }
    echo "$_gh_h:$_gh_n"
}

# --------------------------------------------------------------------------------
# THE TRACKED-MODIFICATION VIEW OF `crates/`, which is what `git checkout --`
# destroys and what the refusal below is about.
#
# `git diff --name-only HEAD` and NOT `git status --porcelain`, deliberately and
# with the trade-off named: `status` also reports UNTRACKED files, and a stray
# `crates/scratch.txt` would then refuse every gate on this box for a reason
# that has nothing to do with the hazard — a check that refuses for unrelated
# reasons is a check somebody deletes. Untracked files are not lost to a
# checkout; they are still covered, because the content hash above walks the
# filesystem and sees them.
# --------------------------------------------------------------------------------
crates_dirty() {    # <root> -> space-separated paths, empty when clean
    _cd_root="$1"
    git -C "$_cd_root" rev-parse --git-dir >/dev/null 2>&1 || { echo ''; return 0; }
    git -C "$_cd_root" diff --name-only HEAD -- crates/ 2>/dev/null | tr '\n' ' '
}

# --------------------------------------------------------------------------------
# THE HATCH-RED ROW (board #1406, lane w-cache, 2026-08-08).
#
# `work/w-front3/hatch.py` is the frontier ladder's lift mechanism and its
# `revert` was, until lane `w-hatch` repaired it, a bare `git checkout --` over
# six `crates/` files — which had already eaten a peer lane's unstaged fix
# (#1380). The repair added eight refusals, each leading with its own word, and
# `work/w-hatch/hatch_red.py` fires every one of them on purpose: 11 arms, 9 red
# and 2 green, and against the UNREPAIRED file it fails 10 of 11 with the key arm
# reading *"the foreign edit SURVIVED: NO — IT WAS EATEN"*.
#
# **It was run by hand and nothing executed it automatically.** That is trap 5 —
# absence reads as success unless something forbids it — sitting one level
# outside the thing that forbids it. This row is the forbidding.
#
# WHY IT RUNS FIRST, BEFORE `pin_harness`
# ---------------------------------------
# The arms WRITE INTO `crates/` and restore afterwards. `pin_harness` runs the
# gate's one and only `cargo build`, so the row has to be finished before that
# starts or a compile could see a half-hatched tree. It needs no toolchain and no
# binary, and it takes seconds, so running it first also means a broken hatch is
# on screen before twenty minutes of compiler.
#
# WHY A DIRTY TREE IS `REFUSED` AND NOT `FAIL`
# --------------------------------------------
# Two of the outcomes here are properties of the TREE, not of `hatch.py`:
#
#   * `crates/` differs from `HEAD` — the arms would `git checkout --` over it,
#     which is #1380's incident being re-enacted by the test written to prevent
#     it. Refuse, and name the files.
#   * the hatch's needles have drifted out of the tree, so `apply` cannot build a
#     hatched tree at all (#1389's shape, and it is live).
#
# A lane mid-wave is routinely in the first state and can be in the second, and
# reddening a peer's gate for a reason unrelated to their rung is not a thing
# this row is allowed to do. So `REFUSED` exits 0 — and it **cannot print an
# unqualified `GATE: PASS`**, exactly the treatment `SAMPLED` and `SKIPPED`
# already get, for exactly their reason. A skip is visible or it is a silent pass.
#
# WHAT IT DOES *NOT* COUNT TOWARD
# -------------------------------
# `--require-graded`'s unit sum. Deliberately. That demand is "this run
# established something about the PORT", and 11 arms of a `work/` script
# establish nothing about the port; folding them in would let an all-skip
# toolchain-less run satisfy a demand it must fail.
#
# Emits: <verdict>|<armspass>|<armstotal>|<red>|<green>|<detail>
# where verdict is PASS | FAIL | REFUSED | NO-RESULT and every non-PASS detail
# LEADS WITH ITS OWN WORD — a shared prefix is how a later refusal comes to
# satisfy an earlier case's expectation, which has silently passed two of six
# mutations on this project before.
#
# A pure function of the log text plus two numbers, so `--selftest` drives it
# with fabricated logs and never runs the real thing.
# --------------------------------------------------------------------------------
hatch_red_verdict() {   # <log> <exit-status> <expected-arm-count>
    _hv_log="$1"; _hv_st="${2:-0}"; _hv_exp="${3:-0}"
    if [ ! -s "$_hv_log" ]; then
        echo "NO-RESULT|0|$_hv_exp|0|0|NO-LOG hatch_red.py produced no output at all"
        return 0
    fi
    # ORDER IS LOAD-BEARING, and it is the mutation trap in person: a `SETUP
    # FAILED` arm ALSO prints a `FAILED:` line at the end, so the tree-property
    # arm must be read BEFORE the guard-property arm. Read the other way round, a
    # tree whose hatch has drifted is indistinguishable from a `hatch.py` whose
    # guards have stopped working — the second is a real defect and the first is
    # a Tuesday.
    if grep -q '^REFUSING to run on a dirty tree' "$_hv_log" 2>/dev/null; then
        echo "REFUSED|0|$_hv_exp|0|0|DIRTY-TREE hatch_red.py refused: crates/ was written since it looked"
        return 0
    fi
    if grep -q 'SETUP FAILED' "$_hv_log" 2>/dev/null; then
        echo "REFUSED|0|$_hv_exp|0|0|HATCH-STALE \`hatch.py apply\` cannot hatch this tree, so the arms have no tree to run on (board #1389)"
        return 0
    fi
    _hv_p=$(sed -n 's/^ALL \([0-9][0-9]*\) ARMS PASS.*[^0-9]\([0-9][0-9]*\) red, \([0-9][0-9]*\) green.*/\1 \2 \3/p' \
        "$_hv_log" 2>/dev/null | head -1)
    if [ -n "$_hv_p" ]; then
        _hv_n=$(echo "$_hv_p" | cut -d' ' -f1)
        _hv_r=$(echo "$_hv_p" | cut -d' ' -f2)
        _hv_g=$(echo "$_hv_p" | cut -d' ' -f3)
        # ANTI-VACUITY, and each arm has its own word. A green line is not
        # evidence on its own: the count has to be the count the file declares,
        # the red arms have to exist, and the controls have to exist. Every one
        # of these three has an equivalent that has read green on this project
        # while measuring nothing.
        if [ "$_hv_exp" -le 0 ] || [ "$_hv_n" -ne "$_hv_exp" ]; then
            echo "FAIL|$_hv_n|$_hv_exp|$_hv_r|$_hv_g|TRUNCATED $_hv_n of $_hv_exp declared arms ran — a short run is not a pass"
            return 0
        fi
        if [ "$_hv_r" -le 0 ] || [ "$_hv_g" -le 0 ] || [ $((_hv_r + _hv_g)) -ne "$_hv_n" ]; then
            echo "FAIL|$_hv_n|$_hv_exp|$_hv_r|$_hv_g|VACUOUS $_hv_r red and $_hv_g green do not account for $_hv_n arms — a run with no red arms proves nothing and a run with no controls proves less"
            return 0
        fi
        if [ "$_hv_st" != "0" ]; then
            echo "FAIL|$_hv_n|$_hv_exp|$_hv_r|$_hv_g|EXIT reported every arm passing and then exited $_hv_st"
            return 0
        fi
        echo "PASS|$_hv_n|$_hv_exp|$_hv_r|$_hv_g|"
        return 0
    fi
    if grep -q '^FAILED: ' "$_hv_log" 2>/dev/null; then
        _hv_d=$(grep -m1 '^FAILED: ' "$_hv_log" | tr '|' ' ')
        echo "FAIL|0|$_hv_exp|0|0|ARMS-FAILED $_hv_d"
        return 0
    fi
    echo "NO-RESULT|0|$_hv_exp|0|0|UNRECOGNIZED no ALL-ARMS-PASS line and no FAILED line — an unenumerated outcome is the next silence"
    return 0
}

# --------------------------------------------------------------------------------
# Run the hatch-red arms, with the interlock in front and the postcondition
# behind. Not selftest-driven: it touches `git` and the real tree, and everything
# it decides on its own has its own word, so the classifier above stays pure.
# --------------------------------------------------------------------------------
hatch_red_run() {   # <log-path> -> echoes the tuple
    _hr_log="$1"
    _hr_py="$repo_root/work/w-hatch/hatch_red.py"
    if [ ! -f "$_hr_py" ]; then
        echo "NO-RESULT|0|0|0|0|MISSING work/w-hatch/hatch_red.py is not in this tree"
        return 0
    fi
    if ! command -v python3 >/dev/null 2>&1; then
        echo "NO-RESULT|0|0|0|0|MISSING no python3, and the arms are a python script"
        return 0
    fi
    # ---- THE INTERLOCK RUNS BEFORE ANYTHING INVOKES THE SCRIPT (#2907) --------
    #
    # It used to run one line lower, AFTER the `--list` arm-count probe, and
    # that ordering is the whole defect: `hatch_red.py`'s module-level
    # `finally: restore()` fired on `--list` too, so the probe did the very
    # `git checkout -- crates/` this interlock exists to prevent, and the
    # interlock then read the tree the probe had just cleaned and let the row
    # run. A dirty tree came out as `REFUSED HATCH-STALE`, the lane's work was
    # gone, and the gate went on to build and grade `HEAD`.
    #
    # `hatch_red.py` is fixed too (its `restore()` is armed by `arm()`), and
    # this order is the second lock rather than the same lock twice: the file
    # could regain a destructive path, and the rule "nothing runs that script
    # until this has passed" survives that. **No invocation of `$_hr_py` — not
    # even `--list` — may appear above this block.**
    # --------------------------------------------------------------------------
    if ! git -C "$repo_root" rev-parse --git-dir >/dev/null 2>&1; then
        echo "REFUSED|0|0|0|0|NO-GIT this tree is not a checkout, so the arms cannot restore what they write"
        return 0
    fi
    # `HEAD`, not the worktree-vs-index diff `hatch_red.py` checks itself: a
    # STAGED `crates/` edit survives the arms' `git checkout --` but can still
    # move one of `hatch.py`'s needles, which would turn a peer's ordinary
    # work-in-progress into this row's red.
    _hr_dirty=$(crates_dirty "$repo_root")
    if [ -n "$_hr_dirty" ]; then
        echo "REFUSED|0|0|0|0|DIRTY-TREE crates/ differs from HEAD and the arms would overwrite it: $_hr_dirty"
        return 0
    fi
    # And the caller's explicit escape hatch is a refusal HERE as well, with its
    # own word. `--allow-dirty-crates` says "grade this tree as it is"; the one
    # row that would rewrite it must not run under that promise, and it must not
    # run merely because the tree happened to be clean at this instant either —
    # the flag is the statement of intent, not the diff.
    if [ "${allow_dirty:-0}" = 1 ]; then
        echo "REFUSED|0|0|0|0|DIRTY-ALLOWED --allow-dirty-crates was given: these arms write into crates/ and this run promised not to touch it"
        return 0
    fi
    # The DECLARED arm count, read out of the file rather than written here, so
    # adding an arm cannot leave this row silently grading the old number.
    #
    # **`cd "$repo_root"` and not the bare invocation it used to be.**
    # `hatch_red.py` resolves the tree it may write into from the INVOKING CWD
    # (board #1460, lane w-hatchroot) and refuses when that is a different
    # checkout from its own — so a gate launched from outside the repository
    # would have got an arm count of 0 here, and 0 is `LADDER`/`TRUNCATED`'s own
    # trigger. The real run below already cds; this line did not.
    # THE PROBE'S OWN POSTCONDITION, and it is a CONTENT hash rather than a `git
    # diff`. `--list` is documented to run nothing and for two lanes it silently
    # reverted `crates/` instead; "the probe is harmless" is exactly the belief
    # that cost #2907 a retracted claim, so it is checked. A git diff could not
    # be the check here — the interlock above has already established that the
    # tree matches `HEAD`, so a *revert* is invisible to `git` and visible only
    # to a hash of the bytes. This also covers untracked files, which the
    # interlock deliberately does not.
    _hr_h0=$(graded_tree_hash "$repo_root" "crates")
    _hr_exp=$(cd "$repo_root" && python3 "$_hr_py" --list 2>/dev/null | grep -c . || echo 0)
    _hr_h1=$(graded_tree_hash "$repo_root" "crates")
    if [ "$_hr_h0" != "$_hr_h1" ]; then
        echo "FAIL|0|$_hr_exp|0|0|PROBE-WROTE reading the arm count with --list changed crates/ ($_hr_h0 -> $_hr_h1), which it must never do"
        return 0
    fi
    _hr_st=0
    if command -v timeout >/dev/null 2>&1; then
        (cd "$repo_root" && timeout 600 python3 "$_hr_py") > "$_hr_log" 2>&1 || _hr_st=$?
    else
        (cd "$repo_root" && python3 "$_hr_py") > "$_hr_log" 2>&1 || _hr_st=$?
    fi
    # THE POSTCONDITION, and it outranks the classifier: the arms write into
    # `crates/` and restore in a `finally`, but a SIGKILL has no `finally`. The
    # gate's `cargo build` is next, so a tree left hatched here would be compiled
    # and graded. Say so and stop rather than build it.
    #
    # A DELTA against the state the arms inherited, for `ladder-red`'s reason and
    # one of its own: `git diff HEAD` cannot see a revert, and these arms' bug of
    # record IS a revert. Here the two happen to coincide — the interlock above
    # has already established the tree matched `HEAD` — but a postcondition that
    # is only correct because of an earlier check is one refactor from silent.
    _hr_res1=$(graded_tree_hash "$repo_root" "crates")
    if [ "$_hr_h1" != "$_hr_res1" ]; then
        echo "FAIL|0|$_hr_exp|0|0|RESIDUE the arms left crates/ changed ($_hr_h1 -> $_hr_res1) and the build must not proceed from it"
        return 0
    fi
    hatch_red_verdict "$_hr_log" "$_hr_st" "$_hr_exp"
    return 0
}

# --------------------------------------------------------------------------------
# THE LADDER-RED ROW (board #1406's second half, lane w-hatchroot, 2026-08-08).
#
# `work/w-ladders/ladder_red.py` — 5 arms, 3 red and 2 green — fires
# `work/w-front3/ladder.py`'s `LADDER-NOWIDTHTABLE` on a missing width-table
# file, an empty table and a table truncated to two arms, and checks the real
# table both ways round (`0xBD`/`0x4C`/`0x41` IN, `0x00`/`0x1C` OUT) plus the
# rename rule on seven grants. **5 of 5 fail against the pre-lane `ladder.py`**,
# which is the non-vacuity demonstration; it was run by hand and nothing
# executed it.
#
# WHY THE INTERLOCK IS NARROW HERE AND WIDE THERE
# -----------------------------------------------
# `hatch_red.py` WRITES into `crates/` and restores it, so anything in `crates/`
# that is not `HEAD` is at risk from it and the whole directory is the interlock.
# These arms write nothing there — `EXPR_RS` is repointed at a temp file under
# `work/` — and read exactly ONE tree file. So the interlock is that one file.
# A wide interlock here would refuse for reasons unrelated to the row, and a row
# that refuses for unrelated reasons is a row nobody reads.
#
# It is still `REFUSED` and not `FAIL`, for the row above's reason: a dirty
# `expr.rs` is a property of the TREE — a peer lane mid-wave editing the width
# table would move `G1 REAL-TABLE`'s counts — and reddening a peer's gate for
# their own work in progress is not something this row may do. `REFUSED` exits 0
# and forfeits the unqualified headline (`GATE: PASS (LADDER-RED REFUSED)`).
#
# Emits: <verdict>|<armspass>|<armstotal>|<red>|<green>|<detail>
# Eleven distinct leading words, none of them shared with `hatch-red`'s eight:
# LADDER-NO-LOG, LADDER-TRUNCATED, LADDER-VACUOUS, LADDER-EXIT,
# LADDER-ARMS-FAILED, LADDER-UNRECOGNIZED, LADDER-MISSING, LADDER-NOSUBJECT,
# LADDER-NOGIT, LADDER-DIRTY, LADDER-RESIDUE.
#
# A pure function of the log text plus two numbers, so `--selftest` drives it
# with fabricated logs and never runs the real thing.
# --------------------------------------------------------------------------------
ladder_red_verdict() {   # <log> <exit-status> <expected-arm-count>
    _lv_log="$1"; _lv_st="${2:-0}"; _lv_exp="${3:-0}"
    if [ ! -s "$_lv_log" ]; then
        echo "NO-RESULT|0|$_lv_exp|0|0|LADDER-NO-LOG ladder_red.py produced no output at all"
        return 0
    fi
    _lv_p=$(sed -n 's/^ALL \([0-9][0-9]*\) ARMS PASS.*[^0-9]\([0-9][0-9]*\) red, \([0-9][0-9]*\) green.*/\1 \2 \3/p' \
        "$_lv_log" 2>/dev/null | head -1)
    if [ -n "$_lv_p" ]; then
        _lv_n=$(echo "$_lv_p" | cut -d' ' -f1)
        _lv_r=$(echo "$_lv_p" | cut -d' ' -f2)
        _lv_g=$(echo "$_lv_p" | cut -d' ' -f3)
        # ANTI-VACUITY, the same three checks the row above carries and for the
        # same reason: the count has to be the count the file declares, the red
        # arms have to exist, and the controls have to exist. A run with no red
        # arms proves nothing and a run with no controls proves less.
        if [ "$_lv_exp" -le 0 ] || [ "$_lv_n" -ne "$_lv_exp" ]; then
            echo "FAIL|$_lv_n|$_lv_exp|$_lv_r|$_lv_g|LADDER-TRUNCATED $_lv_n of $_lv_exp declared arms ran — a short run is not a pass"
            return 0
        fi
        if [ "$_lv_r" -le 0 ] || [ "$_lv_g" -le 0 ] || [ $((_lv_r + _lv_g)) -ne "$_lv_n" ]; then
            echo "FAIL|$_lv_n|$_lv_exp|$_lv_r|$_lv_g|LADDER-VACUOUS $_lv_r red and $_lv_g green do not account for $_lv_n arms"
            return 0
        fi
        if [ "$_lv_st" != "0" ]; then
            echo "FAIL|$_lv_n|$_lv_exp|$_lv_r|$_lv_g|LADDER-EXIT reported every arm passing and then exited $_lv_st"
            return 0
        fi
        echo "PASS|$_lv_n|$_lv_exp|$_lv_r|$_lv_g|"
        return 0
    fi
    if grep -q '^FAILED: ' "$_lv_log" 2>/dev/null; then
        _lv_d=$(grep -m1 '^FAILED: ' "$_lv_log" | tr '|' ' ')
        echo "FAIL|0|$_lv_exp|0|0|LADDER-ARMS-FAILED $_lv_d"
        return 0
    fi
    echo "NO-RESULT|0|$_lv_exp|0|0|LADDER-UNRECOGNIZED no ALL-ARMS-PASS line and no FAILED line — an unenumerated outcome is the next silence"
    return 0
}

# --------------------------------------------------------------------------------
# Run the ladder-red arms, with the narrow interlock in front and the
# postcondition behind. Not selftest-driven: it touches `git` and the real tree.
# --------------------------------------------------------------------------------
LADDER_SUBJECT="crates/c2-il/src/func/body/expr.rs"
ladder_red_run() {   # <log-path> -> echoes the tuple
    _lr_log="$1"
    _lr_py="$repo_root/work/w-ladders/ladder_red.py"
    if [ ! -f "$_lr_py" ]; then
        echo "NO-RESULT|0|0|0|0|LADDER-MISSING work/w-ladders/ladder_red.py is not in this tree"
        return 0
    fi
    if ! command -v python3 >/dev/null 2>&1; then
        echo "NO-RESULT|0|0|0|0|LADDER-MISSING no python3, and the arms are a python script"
        return 0
    fi
    # THE SUBJECT, checked by name. Without this an absent `ladder.py` makes
    # every arm fail on an import error and the row reads LADDER-ARMS-FAILED —
    # "the guards stopped working" when the truth is "the instrument is gone".
    # Two different facts, two different words.
    if [ ! -f "$repo_root/work/w-front3/ladder.py" ]; then
        echo "NO-RESULT|0|0|0|0|LADDER-NOSUBJECT work/w-front3/ladder.py is not in this tree, so there is nothing for the arms to fire"
        return 0
    fi
    # The DECLARED arm count, read out of the file rather than written here.
    _lr_exp=$(cd "$repo_root" && python3 "$_lr_py" --list 2>/dev/null | grep -c . || echo 0)
    if ! git -C "$repo_root" rev-parse --git-dir >/dev/null 2>&1; then
        echo "REFUSED|0|$_lr_exp|0|0|LADDER-NOGIT this tree is not a checkout, so the interlock cannot be read"
        return 0
    fi
    # THE NARROW INTERLOCK. `HEAD`, and exactly the file the arms read: `G1`
    # asserts the tree's own width table parses to a known shape, so a peer's
    # in-flight edit to it is a statement about their work and not about
    # `ladder.py`'s guards.
    _lr_dirty=$(git -C "$repo_root" diff --name-only HEAD -- "$LADDER_SUBJECT" 2>/dev/null | tr '\n' ' ')
    if [ -n "$_lr_dirty" ]; then
        echo "REFUSED|0|$_lr_exp|0|0|LADDER-DIRTY the width table the arms read differs from HEAD: $_lr_dirty"
        return 0
    fi
    # The BEFORE half of the residue delta below — read after the interlocks and
    # immediately before the arms, so it is the state the arms inherited.
    _lr_res0=$(graded_tree_hash "$repo_root" "crates")
    _lr_st=0
    if command -v timeout >/dev/null 2>&1; then
        (cd "$repo_root" && timeout 600 python3 "$_lr_py") > "$_lr_log" 2>&1 || _lr_st=$?
    else
        (cd "$repo_root" && python3 "$_lr_py") > "$_lr_log" 2>&1 || _lr_st=$?
    fi
    # THE POSTCONDITION. These arms have NO code path that writes into `crates/`
    # — that is the claim, and this is the check of it. It is a postcondition
    # against a future edit and against a SIGKILL, not against a known path, and
    # the rung says so rather than counting it as a fired guard.
    #
    # **A DELTA, NOT A DIFF AGAINST `HEAD` (lane w-gatefix, 2026-08-10).** It
    # was `git diff HEAD -- crates/`, which reads "the arms modified crates/"
    # for any file that was already modified when the row started — so under
    # `--allow-dirty-crates` this row accused itself of a peer's edit and the
    # gate failed on `LADDER-RESIDUE`, naming a file no arm here has ever
    # touched. Observed, not anticipated. A postcondition must measure what the
    # thing under it DID, and the difference only shows up on the trees where
    # the answer matters. Content hash rather than `git`, so the delta also
    # covers untracked files and a revert-to-`HEAD` (invisible to a diff).
    _lr_res1=$(graded_tree_hash "$repo_root" "crates")
    if [ "$_lr_res0" != "$_lr_res1" ]; then
        echo "FAIL|0|$_lr_exp|0|0|LADDER-RESIDUE the arms changed crates/ ($_lr_res0 -> $_lr_res1), which they have no path to do"
        return 0
    fi
    ladder_red_verdict "$_lr_log" "$_lr_st" "$_lr_exp"
    return 0
}

# ================================================================================
# THE TREE-INTEGRITY ARMS — board #2969, lane w-gatefix, 2026-08-10.
#
# The regression test for #2668 / #2907. Everything else in this file grades the
# port or an instrument; these arms grade **this file's own ability to be wrong
# about which tree it graded**, which is the failure that makes every other
# number here unreadable.
#
# WHY THEY RUN AGAINST A SYNTHETIC CHECKOUT
# -----------------------------------------
# The behaviour under test is destructive by construction: the arm that matters
# asserts that a dirty `crates/` SURVIVES, and the pre-fix code destroys it. Run
# against the real tree, a red arm would be indistinguishable from the incident
# — the test would have to eat a lane's work in order to report that work gets
# eaten. So each arm builds a throwaway `git init` tree with a `crates/`, a
# `fixtures/`, a `scripts/` and a copy of `hatch_red.py`, points `repo_root` at
# it, and drives the REAL `hatch_red_run` / `graded_tree_hash` / `crates_dirty`
# — not a reimplementation, which would only prove the copy agrees with itself.
#
# THE COUNTERFACTUAL IS BUILT IN, AND THAT IS THE POINT
# -----------------------------------------------------
# `C1` and `C2` run the same two arms against the code AS IT WAS AT THE PRE-LANE
# COMMIT, read out of git rather than described here, and require them to FAIL.
# A red test nobody has watched fail on the unfixed code is a red test that
# might be asserting nothing — the same argument that makes `hatch_red.py`'s
# `--master` mode exist, one level up. Without C1/C2 these arms could be
# tautologies and read green forever.
#
# `TI_BASE` is the merge base this lane branched from and is a FIXED string on
# purpose: it names the last commit at which the defect was present. Resolving
# it to `HEAD~n` or to `master` would make the counterfactual quietly re-target
# itself once this lands, and it would then compare the fix against the fix.
# When git cannot produce that blob (a shallow clone, an exported tarball) the
# arm reports `SKIP` and says so — an unavailable counterfactual must not read
# as a passing one.
# ================================================================================
TI_BASE=0d0a74d2

# The same fixed-string discipline for board #3048's counterfactual (C4 below):
# the last commit at which the gate still hashed its own `__pycache__` byproduct
# and voided its own first run in every fresh worktree. Resolving it to `master`
# or `HEAD~n` would re-target it once this lands and compare the fix to itself.
TI3048_BASE=31a83377

# One synthetic checkout. `<dir>` is rebuilt from scratch; `<hatchred>` is the
# copy of `hatch_red.py` under test, which is how the counterfactual arms swap in
# the pre-lane file.
_ti_mktree() {   # <dir> <hatchred-path>
    rm -rf "$1" 2>/dev/null || true
    mkdir -p "$1/crates" "$1/fixtures" "$1/scripts" "$1/work/w-hatch" \
        "$1/work/w-front3" || return 1
    printf 'pub fn probe() -> u32 { 1 }\n' > "$1/crates/probe.rs"
    printf 'int f(void) { return 1; }\n'   > "$1/fixtures/f.cpp"
    printf '#!/bin/sh\nexit 0\n'           > "$1/scripts/s.sh"
    cp "$2" "$1/work/w-hatch/hatch_red.py" || return 1
    # `hatch.py` is the subject the arms load. It is copied so a full run has
    # something to import; every arm below refuses before that point, but a
    # missing subject would change WHICH refusal fires and the arms assert on
    # the word, so it is present.
    cp "$repo_root/work/w-front3/hatch.py" "$1/work/w-front3/hatch.py" 2>/dev/null || true
    git -C "$1" init -q 2>/dev/null || return 1
    # By name, never `add -A` over a directory: a lane swept captured IL and objs
    # into git that way last night. This tree is synthetic and small enough to
    # enumerate, so it is enumerated.
    git -C "$1" add crates/probe.rs fixtures/f.cpp scripts/s.sh \
        work/w-hatch/hatch_red.py 2>/dev/null || return 1
    [ -f "$1/work/w-front3/hatch.py" ] && git -C "$1" add work/w-front3/hatch.py 2>/dev/null
    git -C "$1" -c user.name=gate -c user.email=gate@localhost \
        commit -q -m "synthetic tree for the gate's tree-integrity arms" 2>/dev/null || return 1
    return 0
}

# The foreign edit every arm uses, and the question every arm asks about it.
_ti_dirty()   { printf '\n// A FOREIGN UNCOMMITTED EDIT. IT MUST SURVIVE.\n' >> "$1/crates/probe.rs"; }
_ti_survived() { grep -q 'IT MUST SURVIVE' "$1/crates/probe.rs" 2>/dev/null; }

# --------------------------------------------------------------------------------
# Run the arms. Emits one `PASS`/`FAIL`/`SKIP` line each; returns 1 if any FAILed.
# `<scratch>` is a directory this may destroy.
# --------------------------------------------------------------------------------
tree_integrity_arms() {   # <scratch-dir>
    _ti_dir="$1"; _ti_bad=0; _ti_n=0
    _ti_say() {   # <verdict> <name> <detail>
        _ti_n=$((_ti_n + 1))
        printf '  %-5s %-34s %s\n' "$1" "$2" "$3"
        [ "$1" = FAIL ] && _ti_bad=$((_ti_bad + 1))
        return 0
    }
    if ! command -v python3 >/dev/null 2>&1 || ! command -v git >/dev/null 2>&1; then
        _ti_say SKIP arms-need-python3-and-git "not both present; these arms grade nothing here"
        return 0
    fi
    _ti_save="$repo_root"
    _ti_hr="$repo_root/work/w-hatch/hatch_red.py"

    # ---- A1. THE HASH IS ABOUT CONTENT, AND HEAD CANNOT MOVE IT ---------------
    # The deliverable in one arm: at a FIXED commit, changing a byte under
    # `crates/` must change the printed identity. `git rev-parse HEAD` is the
    # control — it does not move, which is exactly why it could not be the
    # identity.
    if _ti_mktree "$_ti_dir/a1" "$_ti_hr"; then
        _ti_h0=$(graded_tree_hash "$_ti_dir/a1")
        _ti_head0=$(git -C "$_ti_dir/a1" rev-parse HEAD 2>/dev/null || echo '?')
        _ti_dirty "$_ti_dir/a1"
        _ti_h1=$(graded_tree_hash "$_ti_dir/a1")
        _ti_head1=$(git -C "$_ti_dir/a1" rev-parse HEAD 2>/dev/null || echo '!')
        if [ "$_ti_h0" != "$_ti_h1" ] && [ "$_ti_head0" = "$_ti_head1" ]; then
            _ti_say PASS hash-moves-when-content-moves \
                "${_ti_h0%%:*} -> ${_ti_h1%%:*} at one unchanged HEAD"
        else
            _ti_say FAIL hash-moves-when-content-moves \
                "content changed and the identity did not ($_ti_h0 / $_ti_h1)"
        fi
        # ...and it counts what it hashed. A digest over an empty file list is
        # still 12 stable hex characters, which is this project's own defect
        # (absence reading as success) wearing the instrument's face.
        if [ "${_ti_h1##*:}" -eq 3 ]; then
            _ti_say PASS hash-reports-its-file-count "3 files, one in each graded directory"
        else
            _ti_say FAIL hash-reports-its-file-count \
                "expected 3 files under $GRADED_DIRS, counted ${_ti_h1##*:}"
        fi
        # Each of the three directories is really in it. A hash that silently
        # covered `crates/` alone would pass every arm above and be blind to a
        # fixture or a grader changing under the run.
        # These three arms read `printf ... >> "$_ti_dir/a1/$_ti_d/"*` until
        # 2026-08-13 and were GREEN FOR THE WRONG REASON. **POSIX `sh` does not
        # glob a redirection target** (pathname expansion on the redirect word
        # is optional and only in interactive shells), so `>> crates/*` created
        # a file whose name is literally `*` instead of appending to the tracked
        # `crates/probe.rs`. The arm named "editing crates/ moves the identity"
        # was in fact testing "CREATING an untracked file moves the identity" —
        # a true statement about a different property, and one that would have
        # kept the arm green through a change that stopped hashing tracked
        # content altogether. Found by a #3048 mutant (graded set = tracked
        # files only), which reddened all three; board **#3050**. The file is
        # named per directory now, so the arm grades the sentence it prints.
        for _ti_d in crates fixtures scripts; do
            case "$_ti_d" in
                crates)   _ti_f=probe.rs ;;
                fixtures) _ti_f=f.cpp ;;
                *)        _ti_f=s.sh ;;
            esac
            _ti_hb=$(graded_tree_hash "$_ti_dir/a1")
            printf '\n// touched\n' >> "$_ti_dir/a1/$_ti_d/$_ti_f"
            _ti_ha=$(graded_tree_hash "$_ti_dir/a1")
            if [ "$_ti_hb" != "$_ti_ha" ]; then
                _ti_say PASS "hash-covers-$_ti_d" "editing the TRACKED $_ti_d/$_ti_f moves the identity"
            else
                _ti_say FAIL "hash-covers-$_ti_d" \
                    "$_ti_d/ changed and the identity did not — it is not in the hash"
            fi
        done
        # Determinism: same bytes, same answer. Without this the arms above pass
        # on an identity that is a clock.
        _ti_hx=$(graded_tree_hash "$_ti_dir/a1")
        _ti_hy=$(graded_tree_hash "$_ti_dir/a1")
        if [ "$_ti_hx" = "$_ti_hy" ]; then
            _ti_say PASS hash-is-deterministic "two reads of an unchanged tree agree"
        else
            _ti_say FAIL hash-is-deterministic "$_ti_hx then $_ti_hy on an unchanged tree"
        fi
    else
        _ti_say FAIL synthetic-tree-a1 "could not build the scratch checkout"
    fi

    # ---- A2. THE ARM-COUNT PROBE WRITES NOTHING ------------------------------
    # #2907's mechanism, named: `hatch_red.py --list` is documented to run
    # nothing and used to `git checkout -- crates/` on its way out.
    if _ti_mktree "$_ti_dir/a2" "$_ti_hr"; then
        _ti_dirty "$_ti_dir/a2"
        (cd "$_ti_dir/a2" && python3 work/w-hatch/hatch_red.py --list) >/dev/null 2>&1 || true
        if _ti_survived "$_ti_dir/a2"; then
            _ti_say PASS list-probe-writes-nothing "an uncommitted crates/ edit survives --list"
        else
            _ti_say FAIL list-probe-writes-nothing \
                "--list DESTROYED an uncommitted crates/ edit (board #2907)"
        fi
    else
        _ti_say FAIL synthetic-tree-a2 "could not build the scratch checkout"
    fi

    # ---- A3. THE ROW REFUSES A DIRTY TREE, AND THE EDIT IS STILL THERE --------
    # Two assertions in one arm ON PURPOSE, because either alone has read green
    # while the defect was live: the row DID refuse (`REFUSED HATCH-STALE`) on
    # the run that ate w-seclayout's work, and a row that preserved the tree by
    # never running would satisfy the other half without being the guard.
    if _ti_mktree "$_ti_dir/a3" "$_ti_hr"; then
        _ti_dirty "$_ti_dir/a3"
        repo_root="$_ti_dir/a3"
        _ti_t=$(hatch_red_run "$_ti_dir/a3.log" 2>/dev/null || echo 'NO-RESULT|0|0|0|0|the row itself errored')
        repo_root="$_ti_save"
        _ti_v=$(printf '%s\n' "$_ti_t" | cut -d'|' -f1)
        _ti_w=$(printf '%s\n' "$_ti_t" | cut -d'|' -f6 | cut -d' ' -f1)
        if [ "$_ti_v" = REFUSED ] && [ "$_ti_w" = DIRTY-TREE ] && _ti_survived "$_ti_dir/a3"; then
            _ti_say PASS dirty-tree-refused-and-preserved "REFUSED DIRTY-TREE, edit intact"
        else
            _ti_say FAIL dirty-tree-refused-and-preserved \
                "verdict=$_ti_v word=$_ti_w survived=$(_ti_survived "$_ti_dir/a3" && echo yes || echo NO)"
        fi
    else
        _ti_say FAIL synthetic-tree-a3 "could not build the scratch checkout"
    fi

    # ---- A4. THE INTERLOCK IS ABOVE EVERY INVOCATION, IN SOURCE ORDER --------
    # A structural arm, and it is not redundant with A2/A3: those grade today's
    # `hatch_red.py`, and this one grades the rule that survives it regaining a
    # destructive path. `_hr_py` must not be executed anywhere above the
    # `crates_dirty` interlock inside `hatch_red_run`.
    _ti_body=$(sed -n '/^hatch_red_run()/,/^}/p' "$0")
    _ti_lock=$(printf '%s\n' "$_ti_body" | grep -n 'crates_dirty "\$repo_root"' | head -1 | cut -d: -f1)
    _ti_first=$(printf '%s\n' "$_ti_body" | grep -n 'python3 "\$_hr_py"' | head -1 | cut -d: -f1)
    if [ -n "$_ti_lock" ] && [ -n "$_ti_first" ] && [ "$_ti_lock" -lt "$_ti_first" ]; then
        _ti_say PASS interlock-precedes-every-invocation \
            "interlock at body line $_ti_lock, first invocation at $_ti_first"
    else
        _ti_say FAIL interlock-precedes-every-invocation \
            "interlock=${_ti_lock:-ABSENT} first-invocation=${_ti_first:-ABSENT} — the probe runs first"
    fi

    # ---- C1/C2. THE COUNTERFACTUAL: BOTH MUST FAIL ON THE PRE-LANE FILE ------
    _ti_old="$_ti_dir/hatch_red.$TI_BASE.py"
    if git -C "$_ti_save" show "$TI_BASE:work/w-hatch/hatch_red.py" > "$_ti_old" 2>/dev/null \
       && [ -s "$_ti_old" ]; then
        if _ti_mktree "$_ti_dir/c1" "$_ti_old"; then
            _ti_dirty "$_ti_dir/c1"
            (cd "$_ti_dir/c1" && python3 work/w-hatch/hatch_red.py --list) >/dev/null 2>&1 || true
            if _ti_survived "$_ti_dir/c1"; then
                _ti_say FAIL counterfactual-list-probe-was-destructive \
                    "the edit SURVIVED $TI_BASE's --list, so arm A2 is asserting nothing"
            else
                _ti_say PASS counterfactual-list-probe-was-destructive \
                    "$TI_BASE's --list ate the edit; A2 is a real assertion"
            fi
        else
            _ti_say FAIL synthetic-tree-c1 "could not build the scratch checkout"
        fi
        if _ti_mktree "$_ti_dir/c2" "$_ti_old"; then
            _ti_dirty "$_ti_dir/c2"
            repo_root="$_ti_dir/c2"
            hatch_red_run "$_ti_dir/c2.log" >/dev/null 2>&1 || true
            repo_root="$_ti_save"
            if _ti_survived "$_ti_dir/c2"; then
                _ti_say PASS counterfactual-row-now-preserves \
                    "with $TI_BASE's file the row's own interlock now saves the edit"
            else
                _ti_say FAIL counterfactual-row-now-preserves \
                    "$TI_BASE's file still eats the edit through THIS gate — the reorder did not take"
            fi
        else
            _ti_say FAIL synthetic-tree-c2 "could not build the scratch checkout"
        fi
    else
        _ti_say SKIP counterfactual-unavailable \
            "git cannot produce $TI_BASE:work/w-hatch/hatch_red.py; NOT a pass"
    fi

    # ---- C3. AND THE PRE-LANE `gate.sh` HAD THE ORDER THE OTHER WAY ROUND ----
    _ti_oldg="$_ti_dir/gate.$TI_BASE.sh"
    if git -C "$_ti_save" show "$TI_BASE:scripts/gate.sh" > "$_ti_oldg" 2>/dev/null \
       && [ -s "$_ti_oldg" ]; then
        _ti_ob=$(sed -n '/^hatch_red_run()/,/^}/p' "$_ti_oldg")
        _ti_ol=$(printf '%s\n' "$_ti_ob" | grep -n 'diff --name-only HEAD -- crates/' | head -1 | cut -d: -f1)
        _ti_of=$(printf '%s\n' "$_ti_ob" | grep -n 'python3 "\$_hr_py"' | head -1 | cut -d: -f1)
        if [ -n "$_ti_ol" ] && [ -n "$_ti_of" ] && [ "$_ti_of" -lt "$_ti_ol" ]; then
            _ti_say PASS counterfactual-interlock-was-second \
                "$TI_BASE ran the probe at body line $_ti_of, interlock at $_ti_ol"
        else
            _ti_say FAIL counterfactual-interlock-was-second \
                "$TI_BASE's order reads probe=${_ti_of:-ABSENT} interlock=${_ti_ol:-ABSENT}; A4 may be vacuous"
        fi
    else
        _ti_say SKIP counterfactual-gate-unavailable \
            "git cannot produce $TI_BASE:scripts/gate.sh; NOT a pass"
    fi

    # ================================================================================
    # B1/B2/C4 — BOARD #3048: THE GATE'S OWN BYPRODUCT MUST NOT MOVE THE IDENTITY,
    # AND A SOURCE FILE MUST STILL MOVE IT.
    #
    # #3048 is the interlock above firing on the GATE rather than on a peer
    # session: `expr_sweep.sh` imports `scripts/sweep_gen.py`, CPython writes
    # `scripts/__pycache__/*.pyc`, and `scripts/` is a graded directory. Every
    # lane's FIRST run in a fresh worktree was therefore void, and the lanes that
    # did not read both hashes reported it clean.
    #
    # B1 is the fix. B2 is the reason B1 is not "exclude everything" — an
    # exclusion with no lower bound would pass B1 and every arm above it. C4 is
    # the counterfactual, built the way C1/C2/C3 are: the PRE-fix
    # `graded_tree_hash` is read out of git, renamed, and required to MOVE on the
    # same tree. A red test nobody has watched fail is a red test that might be
    # asserting nothing.
    # ================================================================================
    _ti_pymod() {   # <tree> -> makes a graded-dir byproduct the way the gate does
        # The REAL writer, not a fabricated filename: `import` makes THIS
        # interpreter write THIS interpreter's cache name. #3048's own tree
        # carried a `cpython-313` cache beside a `cpython-314` one, so the name
        # is not a constant and must not be hard-coded here. If this interpreter
        # writes none (`PYTHONDONTWRITEBYTECODE` in the environment, a read-only
        # tree), fall back to fabricating one — an arm that vanishes because of
        # an environment variable is an unavailable control, not a passing one.
        ( cd "$1/scripts" && python3 -c 'import timod' ) >/dev/null 2>&1
        if [ -z "$(find "$1/scripts/__pycache__" -maxdepth 1 -type f 2>/dev/null | head -1)" ]; then
            mkdir -p "$1/scripts/__pycache__" 2>/dev/null || return 1
            printf 'fabricated bytecode cache\n' > "$1/scripts/__pycache__/timod.fallback.pyc" || return 1
            echo fabricated; return 0
        fi
        echo real; return 0
    }
    _ti_mkign() {   # <tree> -> a synthetic tree that DECLARES __pycache__ a byproduct
        _ti_mktree "$1" "$_ti_hr" || return 1
        printf '__pycache__/\n' > "$1/.gitignore" || return 1
        printf 'VALUE = 1\n'    > "$1/scripts/timod.py" || return 1
        git -C "$1" add .gitignore scripts/timod.py >/dev/null 2>&1 || return 1
        git -C "$1" -c user.name=gate -c user.email=gate@localhost \
            commit -q -m "declare __pycache__ a byproduct (board #3048)" >/dev/null 2>&1 || return 1
        return 0
    }

    # ---- B1. A DECLARED BYPRODUCT CANNOT MOVE THE IDENTITY --------------------
    if _ti_mkign "$_ti_dir/b1"; then
        _ti_b0=$(graded_tree_hash "$_ti_dir/b1")
        _ti_how=$(_ti_pymod "$_ti_dir/b1")
        _ti_b1=$(graded_tree_hash "$_ti_dir/b1")
        if [ -z "$_ti_how" ]; then
            _ti_say FAIL byproduct-arm-could-not-write "no .pyc and no fallback; this arm graded nothing"
        elif [ "$_ti_b0" = "$_ti_b1" ]; then
            _ti_say PASS ignored-byproduct-cannot-move-id \
                "a $_ti_how .pyc appeared under scripts/; identity held at ${_ti_b0}"
        else
            _ti_say FAIL ignored-byproduct-cannot-move-id \
                "$_ti_b0 -> $_ti_b1 over one gitignored .pyc — board #3048 is LIVE"
        fi
        # ---- B2. ...AND AN UNDECLARED FILE STILL DOES. THE NOT-WEAKENED ARM ----
        printf '#!/bin/sh\nexit 0\n' > "$_ti_dir/b1/scripts/ti_new_source.sh"
        _ti_b2=$(graded_tree_hash "$_ti_dir/b1")
        if [ "$_ti_b2" != "$_ti_b1" ] && [ "${_ti_b2##*:}" -gt "${_ti_b1##*:}" ]; then
            _ti_say PASS untracked-source-still-moves-id \
                "an untracked, unignored scripts/ file: ${_ti_b1##*:} -> ${_ti_b2##*:} files"
        else
            _ti_say FAIL untracked-source-still-moves-id \
                "a new unignored scripts/ file did not move the identity ($_ti_b1 / $_ti_b2) — the exclusion is too wide"
        fi
    else
        _ti_say FAIL synthetic-tree-b1 "could not build the scratch checkout"
    fi

    # ---- C4. THE COUNTERFACTUAL: THE PRE-#3048 IDENTITY DID MOVE --------------
    _ti_g3048="$_ti_dir/gate.$TI3048_BASE.sh"
    if git -C "$_ti_save" show "$TI3048_BASE:scripts/gate.sh" > "$_ti_g3048" 2>/dev/null \
       && [ -s "$_ti_g3048" ] \
       && sed -n '/^graded_tree_hash() {/,/^}/p' "$_ti_g3048" | grep -q 'sha256sum'; then
        # Read the pre-lane function out of git and rename it. Reimplementing it
        # here would only prove the copy agrees with itself — C1/C2's argument.
        eval "$(sed -n '/^graded_tree_hash() {/,/^}/p' "$_ti_g3048" \
                | sed '1s/^graded_tree_hash()/graded_tree_hash_pre3048()/')"
        if _ti_mkign "$_ti_dir/c4"; then
            _ti_c0=$(graded_tree_hash_pre3048 "$_ti_dir/c4")
            _ti_how4=$(_ti_pymod "$_ti_dir/c4")
            _ti_c1=$(graded_tree_hash_pre3048 "$_ti_dir/c4")
            if [ "$_ti_c0" != "$_ti_c1" ]; then
                _ti_say PASS counterfactual-pre3048-id-moved \
                    "$TI3048_BASE's hash: $_ti_c0 -> $_ti_c1 on a $_ti_how4 .pyc; B1 is a real assertion"
            else
                _ti_say FAIL counterfactual-pre3048-id-moved \
                    "$TI3048_BASE's hash held over a .pyc ($_ti_c0) — B1 may be asserting nothing"
            fi
        else
            _ti_say FAIL synthetic-tree-c4 "could not build the scratch checkout"
        fi
    else
        _ti_say SKIP counterfactual-pre3048-unavailable \
            "git cannot produce $TI3048_BASE:scripts/gate.sh; NOT a pass"
    fi

    repo_root="$_ti_save"
    rm -rf "$_ti_dir" 2>/dev/null || true
    if [ "$_ti_bad" -gt 0 ]; then
        echo "  tree integrity: $_ti_bad of $_ti_n arms FAILED — this gate can be wrong about"
        echo "  which tree it graded, and every number it prints is unreadable until that is fixed."
        return 1
    fi
    echo "  tree integrity: $_ti_n arms, 0 failed (boards #2668 / #2907)"
    return 0
}

# --------------------------------------------------------------------------------
# THE TWO RESOURCES, MEASURED SEPARATELY.
#
# Space and inodes are different resources and this gate exhausts the SECOND one
# first. Measured on this box 2026-08-05 (`work/w-ledger/CAUSE.md`): one run tree
# is **112 MB and 16.6k inodes**, against a `/tmp` ceiling of 47 GB and 1,048,576
# inodes — so ~63 trees exhaust inodes and ~430 exhaust space. **Inodes bind ~7x
# earlier.** Lane w-alias's red had 19 GB free and 1048576/1048576 inodes used: a
# free-space check on its own would have passed it straight through.
#
# Both are read with `df`, both are printed, and either being unavailable is
# reported as UNKNOWN rather than being silently treated as fine — a filesystem
# that does not report inodes (btrfs, zfs) must not read as a filesystem with
# infinitely many.
# --------------------------------------------------------------------------------
fs_free_kb() {      # <dir> -> free 1K blocks, or '' if df cannot say
    df -kP "$1" 2>/dev/null | awk 'NR==2 && $4 ~ /^[0-9]+$/ {print $4}'
}
# -> free inodes, or '' when the filesystem HAS NO INODE TABLE TO REPORT.
#
# btrfs on this box prints `0 0 0 -` for total/used/free: it allocates inodes
# dynamically and has no ceiling to report. An earlier draft of this function
# returned that literal `0` and the preflight then refused to run at all with
# "out of INODES" on a filesystem with 430 GiB free — which would have broken
# `--work <dir on /home>`, i.e. the exact workaround lanes use to escape a full
# /tmp. Caught by the selftest before it shipped. **A zero TOTAL means the
# question does not apply; it never means the answer is zero.**
fs_free_inodes() {  # <dir>
    df -iP "$1" 2>/dev/null \
        | awk 'NR==2 && $2 ~ /^[0-9]+$/ && $2 > 0 && $4 ~ /^[0-9]+$/ {print $4}'
}
human_kb() {
    awk -v k="${1:-}" 'BEGIN{
        if (k == "") { print "unknown"; exit }
        if (k >= 1048576) printf "%.1f GiB", k/1048576
        else if (k >= 1024) printf "%.1f MiB", k/1024
        else printf "%d KiB", k
    }'
}
human_n() { awk -v n="${1:-}" 'BEGIN{ if (n=="") {print "unknown"; exit}
    s=""; while (length(n) > 3) { s = "," substr(n, length(n)-2) s; n = substr(n, 1, length(n)-3) }
    print n s }'; }

# The LOW-WATER MARK across the run, not just the value at the start. The failure
# this defends against is TRANSIENT — several lanes peak together, one gate goes
# red, the trees drain, and by the time anybody looks `df` says everything is
# fine. A red that does not reproduce is the most corrosive thing a project whose
# epistemics rest on its gate can have. So the gate samples as it goes and reports
# the minimum it saw, which is the number that explains the red.
RES_DIR=""; RES_KB0=""; RES_IN0=""; RES_KBMIN=""; RES_INMIN=""
res_init() {  # <dir>
    RES_DIR="$1"
    RES_KB0=$(fs_free_kb "$1"); RES_IN0=$(fs_free_inodes "$1")
    RES_KBMIN="$RES_KB0"; RES_INMIN="$RES_IN0"
}
res_sample() {
    [ -n "$RES_DIR" ] || return 0
    _rs_kb=$(fs_free_kb "$RES_DIR"); _rs_in=$(fs_free_inodes "$RES_DIR")
    if [ -n "$_rs_kb" ] && { [ -z "$RES_KBMIN" ] || [ "$_rs_kb" -lt "$RES_KBMIN" ]; }; then
        RES_KBMIN="$_rs_kb"
    fi
    if [ -n "$_rs_in" ] && { [ -z "$RES_INMIN" ] || [ "$_rs_in" -lt "$RES_INMIN" ]; }; then
        RES_INMIN="$_rs_in"
    fi
    return 0
}

# --------------------------------------------------------------------------------
# THE PREFLIGHT. A POSITIVE CHECK WITH PRINTED COUNTS, and its own exit code.
#
# Below a floor this stops the gate BEFORE it grades anything, because a gate that
# runs out of disk halfway prints `NO-RESULT` on whichever instruments were still
# to come — w-alias saw it on the sweep AND the cross at once, which reads exactly
# like one change breaking two instruments. Refusing up front costs two minutes and
# removes the whole misdiagnosis.
#
# Returns 0 = clear, 1 = below a floor (the caller exits 3, which is NOT the exit
# code of a mismatch).
# --------------------------------------------------------------------------------
preflight_disk() {  # <dir> <min-mb> <min-inodes>
    _pf_dir="$1"; _pf_mb="$2"; _pf_in="$3"
    _pf_fkb=$(fs_free_kb "$_pf_dir"); _pf_fin=$(fs_free_inodes "$_pf_dir")
    _pf_needkb=$((_pf_mb * 1024))

    printf 'disk:   %s — free %s / %s inodes (floors: %s / %s inodes)\n' \
        "$_pf_dir" "$(human_kb "$_pf_fkb")" "$(human_n "$_pf_fin")" \
        "$(human_kb "$_pf_needkb")" "$(human_n "$_pf_in")"

    _pf_bad=""
    if [ -z "$_pf_fkb" ]; then
        echo "        FREE SPACE UNKNOWN — df could not report it. Not treated as fine."
        _pf_bad="space (unreadable)"
    elif [ "$_pf_fkb" -lt "$_pf_needkb" ]; then
        _pf_bad="SPACE"
    fi
    if [ -z "$_pf_fin" ]; then
        echo "        FREE INODES UNKNOWN — this filesystem does not report them, so the"
        echo "        inode floor is NOT CHECKED on this run. Unreported is not unlimited."
    elif [ "$_pf_fin" -lt "$_pf_in" ]; then
        _pf_bad="${_pf_bad:+$_pf_bad and }INODES"
    fi
    [ -n "$_pf_bad" ] || return 0

    echo
    echo "GATE: FAIL (DISK) — out of $_pf_bad on $_pf_dir, before anything was graded."
    echo "  free: $(human_kb "$_pf_fkb") and $(human_n "$_pf_fin") inodes"
    echo "  need: $(human_kb "$_pf_needkb") and $(human_n "$_pf_in") inodes"
    echo
    echo "  *** THIS IS A RESOURCE FAULT AND NOT A MISMATCH. ***"
    echo "  NOTHING was graded, so this run establishes NOTHING about the port —"
    echo "  neither that it is right nor that it is wrong. It exits 3, which no"
    echo "  correctness failure ever exits. One run tree costs ~112 MB and ~16.6k"
    echo "  inodes, and INODES run out about 7x sooner than bytes do."
    echo
    echo "  Fixes, in order: re-run (this gate reaps stale run trees on entry);"
    echo "  \`--work DIR\` on a filesystem with room; raise C2RS_GATE_MIN_MB /"
    echo "  C2RS_GATE_MIN_INODES only once you know why they were set here."
    return 1
}

# --------------------------------------------------------------------------------
# IS THIS PID A LIVE GATE? Conservative by construction.
#
# Deleting a running gate's run tree is a far worse failure than keeping a stale
# one, so every branch that cannot ESTABLISH death resolves to "live".
#
#   not a number            -> not live (a tree we cannot attribute is reapable)
#   kill -0 fails           -> dead, reapable
#   alive, cmdline says gate.sh   -> LIVE, keep
#   alive, cmdline says otherwise -> PID REUSE. This box mints hundreds of pids a
#                                    night and the tree names ARE pids, so a
#                                    naive `kill -0` keeps stale trees forever
#                                    behind an unrelated process.
#   alive, cmdline unreadable     -> LIVE, keep. Unknown must not mean delete.
# --------------------------------------------------------------------------------
gate_pid_live() {  # <pid>
    _gp="${1:-}"
    case "$_gp" in ''|*[!0-9]*) return 1 ;; esac
    [ "$_gp" -gt 0 ] || return 1
    kill -0 "$_gp" 2>/dev/null || return 1
    if [ -r "/proc/$_gp/cmdline" ]; then
        if tr '\0' ' ' < "/proc/$_gp/cmdline" 2>/dev/null | grep -q 'gate\.sh'; then
            return 0
        fi
        return 1
    fi
    return 0
}

# --------------------------------------------------------------------------------
# Was this run GREEN? Reads the verdict the run itself wrote, never the exit code
# of anything running now. `results.tsv` is `slug<TAB>flags<TAB>VERDICT|...`.
#
# Green iff the file exists, is non-empty, and EVERY row's verdict is one of the
# three that mean "nothing differed": PASS, SKIPPED, SAMPLED. Anything else —
# missing file, empty file, unreadable file, one FAIL among eighteen PASSes —
# answers NO. This gates a deletion, so it is written to fail closed: the caller
# strips only on an affirmative, and "cannot tell" is not affirmative.
# --------------------------------------------------------------------------------
tree_is_green() {  # <tree>
    _tg_r="$1/results.tsv"
    [ -s "$_tg_r" ] || return 1
    # A row that is not PASS/SKIPPED/SAMPLED is disqualifying. Counting the bad
    # rows rather than short-circuiting keeps this a positive check: a parse that
    # matched nothing at all would otherwise report green.
    _tg_bad=$(awk -F'\t' '
        NF == 0 { next }
        { split($3, v, "|")
          if (v[1] != "PASS" && v[1] != "SKIPPED" && v[1] != "SAMPLED") bad++ }
        END { print bad + 0 }' "$_tg_r" 2>/dev/null || echo 1)
    _tg_rows=$(awk 'NF { n++ } END { print n + 0 }' "$_tg_r" 2>/dev/null || echo 0)
    [ "${_tg_rows:-0}" -gt 0 ] 2>/dev/null || return 1
    [ "${_tg_bad:-1}" -eq 0 ] 2>/dev/null || return 1
    return 0
}

# --------------------------------------------------------------------------------
# Strip the REGENERABLE case corpus from a finished, green run tree. Removes
# `sweep/*.cpp` and nothing else: every log, status, report, flags file and graded
# part stays where the gate's output said it would be.
#
# Echoes the number of cases removed (0 if it declined). Returns 0 always — this
# is housekeeping and must never be able to fail a gate.
# --------------------------------------------------------------------------------
strip_case_corpus() {  # <tree> [dry-run]
    _sc_t="$1"; _sc_dry="${2:-}"
    _sc_sw="$_sc_t/sweep"
    [ -d "$_sc_sw" ] || { echo 0; return 0; }
    [ -e "$_sc_sw/CASES_STRIPPED" ] && { echo 0; return 0; }
    _sc_n=$(find "$_sc_sw" -maxdepth 1 -name '*.cpp' -type f 2>/dev/null | wc -l)
    [ "${_sc_n:-0}" -gt 0 ] 2>/dev/null || { echo 0; return 0; }
    if [ -n "$_sc_dry" ]; then echo "$_sc_n"; return 0; fi
    # Scoped by -maxdepth and -name to the generated cases. `sweep/parts/`,
    # `cases.txt` and `cases.run` are deliberately out of range.
    find "$_sc_sw" -maxdepth 1 -name '*.cpp' -type f -delete 2>/dev/null || true
    # The note goes where the cases were, so anyone who follows a path out of
    # `cases.txt` and finds it missing lands on the reason instead of a mystery.
    {
        echo "The $_sc_n generated *.cpp cases that were here have been removed."
        echo
        echo "They are DERIVED INPUTS, not evidence: scripts/sweep_gen.py rebuilds"
        echo "them from scripts/sweep.d, and expr_sweep.sh regenerates the whole set"
        echo "before every grade. This run was GREEN (every results.tsv row PASS,"
        echo "SKIPPED or SAMPLED), so no case here was named by a mismatch."
        echo
        echo "Kept: cases.txt, cases.run, parts/ (the graded result), and every log,"
        echo "status, report and flags file in the tree above."
        echo
        echo "Removed by gate.sh's reaper because a kept tree is kept for its logs,"
        echo "which are 1.6% of it; C2RS_GATE_KEEP_CASES trees keep their corpus."
        echo
        echo "To rebuild:  scripts/expr_sweep.sh $_sc_sw"
    } > "$_sc_sw/CASES_STRIPPED" 2>/dev/null || true
    echo "$_sc_n"
    return 0
}

# --------------------------------------------------------------------------------
# THE REAPER. Removes run trees that no live gate owns, keeping the most recent
# few so the logs this gate just pointed at are still there.
#
# Prints one line per tree with the REASON, and a summary carrying a COUNT and the
# space and inodes freed — measured as a df delta over the parent filesystem, which
# is cheap and honest (a concurrent gate writing during the reap moves it, and that
# is stated rather than hidden by a per-tree `du` that would walk 16k inodes a
# tree). Silence here would be the same defect one level out.
#
# Emits to stdout. Returns 0 always: a reaper that fails must never fail a gate.
# --------------------------------------------------------------------------------
reap_run_trees() {  # <parent> <current-run-dir> <keep-recent> <scratch> [dry-run]
    _rr_parent="$1"; _rr_cur="$2"; _rr_keep="$3"; _rr_scr="$4"; _rr_dry="${5:-}"
    _rr_stale="$_rr_scr/reap.stale"; _rr_sorted="$_rr_scr/reap.sorted"
    # The reaper needs two scratch files, and the filesystem it is reaping may be
    # the one with no room to write them. Say so and stand down: the preflight
    # immediately after this is what turns that into a verdict. A reaper that took
    # the gate down with it would be worse than one that did not run.
    if ! { : > "$_rr_stale"; : > "$_rr_sorted"; } 2>/dev/null; then
        echo "reap:   STOOD DOWN — cannot write scratch under $_rr_scr."
        echo "        That is itself a symptom; the disk check below is the verdict."
        return 0
    fi
    _rr_live=0; _rr_cur_n=0; _rr_kept=0; _rr_gone=0; _rr_strip=0; _rr_cases=0
    # Not a positional: the signature's 5th slot is the dry-run flag and threading
    # a 6th through every caller would be a worse trade than reading the knob the
    # rest of the file already sets. The selftest overrides it per call.
    _rr_keepc="${C2RS_GATE_KEEP_CASES:-1}"

    _rr_kb0=$(fs_free_kb "$_rr_parent"); _rr_in0=$(fs_free_inodes "$_rr_parent")

    for _rr_d in "$_rr_parent"/c2rs-gate-*; do
        [ -d "$_rr_d" ] || continue
        if [ "$_rr_d" = "$_rr_cur" ]; then
            _rr_cur_n=1
            printf '  keep    %-28s this run\n' "$_rr_d"
            continue
        fi
        _rr_pid=""
        if [ -r "$_rr_d/gate.pid" ]; then
            _rr_pid=$(head -1 "$_rr_d/gate.pid" 2>/dev/null | tr -dc '0-9' || true)
        fi
        [ -n "$_rr_pid" ] || _rr_pid=$(printf '%s' "${_rr_d##*/}" | sed 's/^c2rs-gate-//')
        if gate_pid_live "$_rr_pid"; then
            _rr_live=$((_rr_live + 1))
            printf '  keep    %-28s LIVE — pid %s is a running gate (a concurrent lane)\n' \
                "$_rr_d" "$_rr_pid"
            continue
        fi
        printf '%s\n' "$_rr_d" >> "$_rr_stale"
    done

    # Newest first, so `--keep N` keeps the N runs whose logs somebody might still
    # be reading — starting with the one that just finished.
    if [ -s "$_rr_stale" ]; then
        tr '\n' '\0' < "$_rr_stale" | xargs -0 ls -dt > "$_rr_sorted" 2>/dev/null || \
            cp "$_rr_stale" "$_rr_sorted"
    fi

    while IFS= read -r _rr_d; do
        [ -n "$_rr_d" ] || continue
        if [ "$_rr_kept" -lt "$_rr_keep" ]; then
            _rr_kept=$((_rr_kept + 1))
            printf '  keep    %-28s recent — %s of %s finished runs kept for their logs\n' \
                "$_rr_d" "$_rr_kept" "$_rr_keep"
            # Kept for its LOGS. Beyond the newest KEEP_CASES, the regenerable
            # case corpus goes and everything the logs point at stays. Green only:
            # a mismatched run keeps the cases its report names.
            if [ "$_rr_kept" -gt "$_rr_keepc" ]; then
                if tree_is_green "$_rr_d"; then
                    _rr_n=$(strip_case_corpus "$_rr_d" "$_rr_dry")
                    if [ "${_rr_n:-0}" -gt 0 ] 2>/dev/null; then
                        _rr_strip=$((_rr_strip + 1))
                        _rr_cases=$((_rr_cases + _rr_n))
                        if [ -n "$_rr_dry" ]; then
                            printf '          STRIP*  %s regenerable cases (green run; dry run)\n' "$_rr_n"
                        else
                            printf '          stripped %s regenerable cases; logs, reports and parts/ kept\n' "$_rr_n"
                        fi
                    fi
                else
                    printf '          cases KEPT — results.tsv is absent or carries a non-PASS row\n'
                fi
            fi
            continue
        fi
        # The path shape is asserted again HERE, immediately before the rm, rather
        # than trusted from the glob 40 lines up. `rm -rf` on a variable is the one
        # place in this file where being wrong is unrecoverable.
        case "${_rr_d##*/}" in
            c2rs-gate-*) ;;
            *) printf '  REFUSE  %-28s does not look like a run tree — not removed\n' "$_rr_d"
               continue ;;
        esac
        case "$_rr_d" in
            "$_rr_parent"/*) ;;
            *) printf '  REFUSE  %-28s outside %s — not removed\n' "$_rr_d" "$_rr_parent"
               continue ;;
        esac
        if [ -n "$_rr_dry" ]; then
            printf '  REAP*   %-28s stale — no live gate owns it (dry run)\n' "$_rr_d"
        else
            rm -rf -- "$_rr_d" 2>/dev/null || true
            printf '  reaped  %-28s stale — no live gate owns it\n' "$_rr_d"
        fi
        _rr_gone=$((_rr_gone + 1))
    done < "$_rr_sorted"

    _rr_kb1=$(fs_free_kb "$_rr_parent"); _rr_in1=$(fs_free_inodes "$_rr_parent")
    _rr_dkb=""; _rr_din=""
    [ -n "$_rr_kb0" ] && [ -n "$_rr_kb1" ] && _rr_dkb=$((_rr_kb1 - _rr_kb0))
    [ -n "$_rr_in0" ] && [ -n "$_rr_in1" ] && _rr_din=$((_rr_in1 - _rr_in0))

    printf 'reap:   %s stale run tree(s) removed, %s kept (%s live, %s recent, %s this run)\n' \
        "$_rr_gone" "$((_rr_live + _rr_kept + _rr_cur_n))" "$_rr_live" "$_rr_kept" "$_rr_cur_n"
    # Printed unconditionally, including the 0 case. "no strip line" and "nothing
    # to strip" must not look alike — that ambiguity is the defect this file keeps
    # closing, and a counter that only appears when it is interesting is the same
    # silence as a reaper that only speaks when it reaps.
    printf 'strip:  %s kept tree(s) lost %s regenerable cases; %s newest kept theirs (C2RS_GATE_KEEP_CASES=%s)\n' \
        "$_rr_strip" "$_rr_cases" \
        "$( [ "$_rr_kept" -lt "$_rr_keepc" ] && echo "$_rr_kept" || echo "$_rr_keepc" )" \
        "$_rr_keepc"
    if [ "$_rr_gone" -gt 0 ] || [ "$_rr_strip" -gt 0 ]; then
        # The delta is measured over a SHARED filesystem, so a concurrent gate
        # writing during the reap can swamp it — measured live on 2026-08-05, two
        # other lanes gating, a real reap of two trees came back at a NET NEGATIVE
        # delta. An earlier draft printed `${d#-}`, stripping the sign, and so
        # reported a filesystem that had got FULLER as space freed. A number whose
        # sign is discarded is worse than no number: the COUNT above is the
        # reliable signal and the delta is reported for what it is.
        if [ -n "$_rr_dkb" ] && [ "$_rr_dkb" -gt 0 ]; then
            printf '        freed %s and %s inodes (df delta over a SHARED filesystem)\n' \
                "$(human_kb "$_rr_dkb")" "$(human_n "${_rr_din:-0}")"
        else
            printf '        df delta over the reap: %s KiB / %s inodes — NOT POSITIVE, because\n' \
                "${_rr_dkb:-unknown}" "${_rr_din:-unknown}"
            printf '        a concurrent gate was writing at the same time. The COUNT above is\n'
            printf '        the signal; a delta on a shared filesystem is not this reap alone.\n'
        fi
    fi
    return 0
}

# --------------------------------------------------------------------------------
# WHICH KIND OF RED IS THIS RED?
#
# Linux returns ENOSPC for BOTH a full filesystem and an exhausted inode table, so
# the log text alone cannot say which; the low-water samples can. Printed under any
# FAIL that carries **mismatch 0**. A FAIL carrying a mismatch never gets this
# banner: bytes were compared and they differed, and that outranks whatever else
# was wrong with the box.
# --------------------------------------------------------------------------------
resource_banner() {  # <run-dir>
    _rb_run="${1:-}"
    [ -n "$_rb_run" ] && [ -d "$_rb_run" ] || return 0
    _rb_hits=$(grep -l 'No space left on device' "$_rb_run"/*.log 2>/dev/null || true)
    [ -n "$_rb_hits" ] || return 0
    echo
    echo "  *** THIS RED IS A RESOURCE FAULT, NOT A MISMATCH. ***"
    echo "  These logs carry 'No space left on device':"
    printf '%s\n' "$_rb_hits" | sed 's/^/    /'
    if [ -n "$RES_DIR" ]; then
        echo "  $RES_DIR at this run's start: $(human_kb "$RES_KB0") / $(human_n "$RES_IN0") inodes"
        echo "  LOW-WATER during this run:   $(human_kb "$RES_KBMIN") / $(human_n "$RES_INMIN") inodes"
        if [ -n "$RES_INMIN" ] && [ "$RES_INMIN" -lt 1000 ]; then
            echo "  -> the exhausted resource is INODES. Space was never the binding one;"
            echo "     one run tree is ~112 MB and ~16.6k inodes, so inodes go ~7x sooner."
        elif [ -n "$RES_KBMIN" ] && [ "$RES_KBMIN" -lt 65536 ]; then
            echo "  -> the exhausted resource is SPACE."
        else
            echo "  -> both had headroom at every sample point, so the exhaustion was"
            echo "     TRANSIENT and peaked between samples. It is still a resource fault."
        fi
    fi
    echo "  An instrument that could not WRITE says NOTHING about whether the port"
    echo "  emits the right bytes. Re-run before reading this as a regression."
    return 0
}

# --------------------------------------------------------------------------------
# A COUNT, OR ZERO. Never a status, and never an empty string arithmetic would
# then have to guess at.
#
# `sweep_verdict` leaves the `graded` field empty on a SKIP tuple, and a
# hand-fabricated or future tuple could carry anything at all. `$(( "" + 1 ))` is a
# syntax error under `set -eu` and would take the gate down inside the very check
# that exists to keep it honest. So an absent or unparseable count reads **0**.
#
# That is the safe direction HERE and only because of which side of the comparison
# it lands on: the demand requires the sum to be > 0, so a field that cannot be
# read can only make the demand FAIL. It is the exact inverse of the defect this
# project keeps recording — there, a missing number read as 0 made a check PASS.
# --------------------------------------------------------------------------------
num() {
    case "${1:-}" in
        ''|*[!0-9]*) echo 0 ;;
        *) echo "$1" ;;
    esac
}

# --------------------------------------------------------------------------------
# WHY DID THE TOOLCHAIN NOT RESOLVE? Printed under every SKIP.
#
# The gate used to skip cleanly and say nothing about WHERE it looked, and twice —
# `rungs/2026-08-05-w-subclass.md` §10.1 and lane w-root on 2026-08-08 — a lane in
# a worktree had to work out for itself that `compilers/` is gitignored (so it does
# not follow a `git worktree add`) and that the `../wibo` fallback resolves
# relative to the WORKTREE, three directories below the main repo.
#
# **This block DECIDES NOTHING.** `Toolchain::locate()` in
# `crates/c2-reference/src/lib.rs` is the resolver and the sole authority; this is
# a signpost printed after the fact. It is written so it cannot silently go stale:
#
#   * the version directory is read OUT OF THE RUST SOURCE at run time rather than
#     copied here, and if it cannot be read the line says so instead of printing a
#     literal that used to be right;
#   * `--selftest` asserts every environment variable named below is still read by
#     `crates/c2-reference/src/lib.rs`, so renaming one there fails the gate's own
#     selftest rather than leaving a signpost pointing at nothing.
#
# `C2RS_DC3` is deliberately NOT here. It points at the dc3-decomp SOURCE tree for
# `scripts/status.sh`; no lane in this gate reads it, and naming it would send a
# lane to set a variable that cannot fix this.
# --------------------------------------------------------------------------------
toolchain_hint() {
    _th_ref="$repo_root/crates/c2-reference/src/lib.rs"
    _th_ver=$(sed -n 's/^const X360_TOOLCHAIN_REL: &str = "\([^"]*\)";.*/\1/p' \
        "$_th_ref" 2>/dev/null | head -1)
    # The COMPILERS ROOT the resolver would use, by the precedence `compilers_root`
    # documents (env verbatim > <repo>/compilers > the ../dc3-decomp compat path >
    # <repo>/compilers as the fallthrough). Transcribed here ONLY so the cl.exe and
    # c2.dll lines below name the file the resolver would actually open: an earlier
    # draft printed the default while `C2RS_COMPILERS` pointed elsewhere, and
    # reported `found` for two paths nothing had consulted — a signpost aimed at
    # the wrong road is worse than no signpost.
    if [ -n "${C2RS_COMPILERS-}" ]; then
        _th_croot="$C2RS_COMPILERS"
    elif [ -d "$repo_root/compilers/$_th_ver" ]; then
        _th_croot="$repo_root/compilers"
    elif [ -d "$repo_root/../dc3-decomp/build/compilers/$_th_ver" ]; then
        _th_croot="$repo_root/../dc3-decomp/build/compilers"
    else
        _th_croot="$repo_root/compilers"
    fi
    if [ -z "$_th_ver" ]; then
        _th_dir="<could not read X360_TOOLCHAIN_REL from $_th_ref>"
    else
        _th_dir="$_th_croot/$_th_ver"
    fi

    echo
    echo "  WHY: the toolchain did not resolve from $repo_root"
    if [ -f "$repo_root/.git" ]; then
        echo "       This tree is a git WORKTREE (.git is a file, not a directory), and"
        echo "       every default below is relative to IT, not to the main repo. That is"
        echo "       the cause both recorded times: rungs/2026-08-05-w-subclass.md §10.1"
        echo "       and lane w-root, 2026-08-08."
    fi
    _th_gone=0
    # Report the EFFECTIVE path, not the default. `Toolchain::locate` takes an
    # override VERBATIM and does not fall back, so a run whose `C2RS_COMPILERS`
    # points at a typo skips while the default sitting beside it is perfectly
    # fine — and a signpost that printed the default there would send the reader
    # to look at a path the resolver never consulted.
    _th_say() {  # <label> <default-path> <env-var>
        eval "_th_ov=\${$3-}"
        if [ -n "${_th_ov:-}" ]; then
            _th_p="$_th_ov"
            _th_src="override: $3 — SET in this environment, taken verbatim (no fallback)"
        else
            _th_p="$2"
            _th_src="override: $3 (unset — this is the default)"
        fi
        if [ -e "$_th_p" ]; then
            _th_mark="found  "
        else
            _th_mark="MISSING"
            _th_gone=$((_th_gone + 1))
        fi
        printf '       %-10s %s  %s\n' "$1" "$_th_mark" "$_th_p"
        printf '       %-10s %s\n' "" "$_th_src"
    }
    _th_say "compilers" "$_th_croot" "C2RS_COMPILERS"
    _th_say "cl.exe"    "$_th_dir/cl.exe"      "C2RS_CL_EXE"
    _th_say "c2.dll"    "$_th_dir/c2.dll"      "C2RS_C2_DLL"
    _th_say "wibo"      "$repo_root/../wibo/build/release/wibo" "C2RS_WIBO"
    echo "                  ...or \`wibo\` anywhere on PATH"
    # THE ARM THAT KEEPS THIS BLOCK FROM LYING. Every default can exist and the
    # run still skip — an override pointing somewhere else, a wibo too old, a
    # lane-local refusal. Printing four `found` lines under a heading that says
    # "did not resolve" would be a signpost contradicting the run it explains, so
    # it says so and hands over to the resolver's own voice instead of guessing.
    if [ "$_th_gone" -eq 0 ]; then
        echo
        echo "       NOTE: all four defaults above EXIST, so the skip has a cause this"
        echo "       block cannot see — an override pointing elsewhere, an unrunnable"
        echo "       loader, a stale wibo. Ask the resolver directly:"
        echo "           cargo run --release -p c2-harness --bin c2rs -- \\"
        echo "               census fixtures/cpp/w5_chain.cpp     # 4/4, or it SKIPs"
        echo "       and check C2RS_COMPILERS / C2RS_WIBO / C2RS_CL_EXE / C2RS_C2_DLL"
        echo "       in this environment — an override is taken VERBATIM and does not"
        echo "       fall back."
    fi
    echo
    echo "  FIX, from the main repo:  scripts/configure_existing_worktree.sh $repo_root"
    echo "      (symlinks compilers/, reflinks target/, and links ../wibo beside the"
    echo "       worktrees — then REBUILDS and refuses to finish if it still SKIPs)"
    echo "  If compilers/ is missing in the main repo too:  scripts/fetch_compilers.sh"
    echo "  \`Toolchain::locate()\` in crates/c2-reference/src/lib.rs is the authority;"
    echo "  the lines above report what was FOUND and resolve nothing themselves."
    return 0
}

# --------------------------------------------------------------------------------
# Walk the REGISTRY (never the directory listing) and produce one row per lane.
# --------------------------------------------------------------------------------
collect() {
    _c_reg="$1"; _c_run="$2"; _c_out="$3"
    : > "$_c_out"
    while IFS="$TAB" read -r _c_slug _c_flags; do
        [ -n "$_c_slug" ] || continue
        _c_st=""
        if [ -f "$_c_run/$_c_slug.status" ]; then _c_st=$(cat "$_c_run/$_c_slug.status"); fi
        _c_v=$(lane_verdict "$_c_run/$_c_slug.log" "$_c_st")
        printf '%s\t%s\t%s\n' "$_c_slug" "$_c_flags" "$_c_v" >> "$_c_out"
    done < "$_c_reg"
}

# --------------------------------------------------------------------------------
# THE GENERATED-INSTRUMENT ROWS. Two of them now, and they obey one set of rules.
#
#   expr-sweep   14,635 generated cases at ONE profile (`c2rs diff` hardcodes
#                `/Ox /GS- /c`).
#   mode-cross   the PRODUCT of those cases with `scripts/lanes.txt`, minus the
#                cells `scripts/mode_invariance.py` proved redundant.
#
# Both are ruled on by `sweep_verdict` and both are rendered and checked by the
# loops below, because a second copy of "what does a short count mean" is how the
# two would come to disagree about it.
# --------------------------------------------------------------------------------
gen_tuple() {   # <SWEEP|CROSS>
    case "$1" in
        SWEEP) printf '%s\n' "$_d_sw" ;;
        CROSS) printf '%s\n' "$_d_cx" ;;
    esac
}
gen_name() {
    case "$1" in
        SWEEP) echo "expr-sweep" ;;
        CROSS) echo "mode-cross" ;;
    esac
}
gen_unit() {
    case "$1" in
        SWEEP) echo "generated cases" ;;
        CROSS) echo "case-lane cells" ;;
    esac
}
gen_why() {
    case "$1" in
        SWEEP) echo "board #232: the 12 mode lanes grade hand-built fixtures and
  \`c2rs gap\` grades workload TUs; neither GENERATES the shapes it covers." ;;
        CROSS) echo "w-order Y-a: the sweep runs ONE profile, so a defect that needs
  \`/EHsc\` as well as its shape is invisible to every instrument without this row." ;;
    esac
}

# The explanation behind a `(HATCH-RED REFUSED)` headline. A function, not four
# copies: the suffix and the paragraph must never be able to disagree about
# whether the row ran.
hatch_refusal_note() {
    [ "${_d_hrv:-}" = REFUSED ] || return 0
    echo
    echo "  HATCH-RED REFUSED — $_d_hrd"
    echo "  The arms write into crates/ and restore it, so they will not run over a tree"
    echo "  that is not HEAD, and they cannot run at all on a tree the hatch will not"
    echo "  apply to. Neither is a statement about \`hatch.py\`'s eight refusals, so this"
    echo "  run does NOT establish what a full run establishes (board #1406). This is why"
    echo "  the headline above is qualified: commit or stash crates/ and re-run."
}

# The same, for the ladder-red row. A separate function and not a parameterised
# one: the two rows refuse for DIFFERENT reasons over DIFFERENT files, and a
# shared paragraph that said "the instrument row refused" would be the message
# collapse this file's own trap B is about.
ladder_refusal_note() {
    [ "${_d_lrv:-}" = REFUSED ] || return 0
    echo
    echo "  LADDER-RED REFUSED — $_d_lrd"
    echo "  The arms read the width table out of $LADDER_SUBJECT and assert its"
    echo "  shape, so an in-flight edit to that file is a statement about YOUR work and"
    echo "  not about \`ladder.py\`'s guards. This run does NOT establish what a full run"
    echo "  establishes (board #1406). That is why the headline above is qualified:"
    echo "  commit or stash that file and re-run."
}

decide() {
    _d_reg="$1"; _d_res="$2"; _d_run="${3:-}"; _d_sw="${4:-}"; _d_filt="${5:-}"
    _d_cx="${6:-}"
    # The hatch-red tuple (board #1406). Its own argument rather than a third
    # `gen_*` arm, because the `gen_*` rows are TOOLCHAIN-BOUND — the all-skip
    # path below requires every one of them to read SKIP — and this row needs no
    # toolchain and must keep working on the portable lane.
    _d_hr="${7:-}"
    # The ladder-red tuple (#1406's second half). Same argument, same reason.
    _d_lr="${8:-}"
    _d_n=$(wc -l < "$_d_reg")
    _d_rows=$(wc -l < "$_d_res")

    echo
    echo "LANE                 VERDICT     graded/total  match  mismatch  flags"
    echo "-------------------- ---------- ------------- ------ --------- --------------------"
    awk -F"$TAB" '{
        split($3, f, "|")
        printf "%-20s %-10s %6s/%-6s %6s %9s  %s%s\n", $1, f[1], f[2], f[3], f[4], f[5],
               $2, (f[6] == "" ? "" : "   <- " f[6])
    }' "$_d_res"

    # The generated instruments are ROWS of this table, not notes beside it. **An
    # ABSENT verdict is a failure**, checked before anything else about the lanes:
    # a gate that forgot to run one of them is the state this addition exists to
    # make impossible, and it must not be able to reach a PASS line below.
    for _g in SWEEP CROSS; do
        if [ -z "$(gen_tuple "$_g")" ]; then
            echo
            echo "GATE: FAIL — no $(gen_name "$_g") verdict was produced at all."
            echo "  \`scripts/$( [ "$_g" = SWEEP ] && echo expr_sweep.sh || echo mode_cross.sh )\` is part of this gate."
            echo "  $(gen_why "$_g")"
            return 1
        fi
        printf "%-20s %-10s %6s/%-6s %6s %9s  %s%s\n" \
            "$(gen_name "$_g")" \
            "$(gen_tuple "$_g" | cut -d'|' -f1)" \
            "$(gen_tuple "$_g" | cut -d'|' -f2)" \
            "$(gen_tuple "$_g" | cut -d'|' -f3)" \
            "$(gen_tuple "$_g" | cut -d'|' -f7)" \
            "$(gen_tuple "$_g" | cut -d'|' -f5)" \
            "$(gen_unit "$_g") (of $(gen_tuple "$_g" | cut -d'|' -f4))" \
            "$([ -z "$(gen_tuple "$_g" | cut -d'|' -f6)" ] && echo "" || echo "   <- $(gen_tuple "$_g" | cut -d'|' -f6)")"
    done

    # The hatch-red row. Absent is a failure too — but that is ruled on BELOW the
    # completeness check, not here: an early guard that fires first takes the
    # headline away from the later one, and `short-table` lost its own message to
    # exactly that while this row was being written.
    _d_hrv=$(printf '%s\n' "$_d_hr" | cut -d'|' -f1)
    _d_hrp=$(printf '%s\n' "$_d_hr" | cut -d'|' -f2)
    _d_hrt=$(printf '%s\n' "$_d_hr" | cut -d'|' -f3)
    _d_hrr=$(printf '%s\n' "$_d_hr" | cut -d'|' -f4)
    _d_hrg=$(printf '%s\n' "$_d_hr" | cut -d'|' -f5)
    _d_hrd=$(printf '%s\n' "$_d_hr" | cut -d'|' -f6)
    # `n/a` in the mismatch column, never `0`: this row grades instruments, not
    # objs, and it cannot produce a mismatch. A `0` there would read as a graded
    # zero, which is the strongest claim on this table and one it cannot make.
    if [ -n "$_d_hr" ]; then
        printf "%-20s %-10s %6s/%-6s %6s %9s  %s%s\n" \
            "hatch-red" "$_d_hrv" "$_d_hrp" "$_d_hrt" "$_d_hrr" "n/a" \
            "arms ($_d_hrg green controls)" \
            "$([ -z "$_d_hrd" ] && echo "" || echo "   <- $_d_hrd")"
    fi
    _d_lrv=$(printf '%s\n' "$_d_lr" | cut -d'|' -f1)
    _d_lrp=$(printf '%s\n' "$_d_lr" | cut -d'|' -f2)
    _d_lrt=$(printf '%s\n' "$_d_lr" | cut -d'|' -f3)
    _d_lrr=$(printf '%s\n' "$_d_lr" | cut -d'|' -f4)
    _d_lrg=$(printf '%s\n' "$_d_lr" | cut -d'|' -f5)
    _d_lrd=$(printf '%s\n' "$_d_lr" | cut -d'|' -f6)
    if [ -n "$_d_lr" ]; then
        printf "%-20s %-10s %6s/%-6s %6s %9s  %s%s\n" \
            "ladder-red" "$_d_lrv" "$_d_lrp" "$_d_lrt" "$_d_lrr" "n/a" \
            "arms ($_d_lrg green controls)" \
            "$([ -z "$_d_lrd" ] && echo "" || echo "   <- $_d_lrd")"
    fi
    echo

    _d_swv=$(printf '%s\n' "$_d_sw" | cut -d'|' -f1)
    _d_swc=$(printf '%s\n' "$_d_sw" | cut -d'|' -f2)
    _d_swsel=$(printf '%s\n' "$_d_sw" | cut -d'|' -f3)
    _d_swtot=$(printf '%s\n' "$_d_sw" | cut -d'|' -f4)
    _d_swm=$(printf '%s\n' "$_d_sw" | cut -d'|' -f5)
    _d_swg=$(printf '%s\n' "$_d_sw" | cut -d'|' -f7)
    _d_swu=$(printf '%s\n' "$_d_sw" | cut -d'|' -f8)
    _d_cxv=$(printf '%s\n' "$_d_cx" | cut -d'|' -f1)
    _d_cxg=$(printf '%s\n' "$_d_cx" | cut -d'|' -f7)
    _d_cxsel=$(printf '%s\n' "$_d_cx" | cut -d'|' -f3)
    _d_cxtot=$(printf '%s\n' "$_d_cx" | cut -d'|' -f4)

    # COMPLETENESS FIRST, and as its own statement. If the table has fewer rows
    # than the registry has lanes, nothing else printed here means anything — a
    # lane silently dropped from the walk is precisely how an absence becomes a
    # green. Checked before any verdict is computed.
    if [ "$_d_rows" -ne "$_d_n" ]; then
        echo "GATE: FAIL — the registry has $_d_n lanes and the table has $_d_rows rows."
        echo "  Rows are produced by walking the registry, so this means the walk itself"
        echo "  broke. No verdict below this line would be trustworthy."
        return 1
    fi

    _d_pass=$(awk -F"$TAB" '{split($3,f,"|"); if (f[1]=="PASS") c++} END{print c+0}' "$_d_res")
    _d_fail=$(awk -F"$TAB" '{split($3,f,"|"); if (f[1]=="FAIL") c++} END{print c+0}' "$_d_res")
    _d_skip=$(awk -F"$TAB" '{split($3,f,"|"); if (f[1]=="SKIP") c++} END{print c+0}' "$_d_res")
    _d_none=$(awk -F"$TAB" '{split($3,f,"|"); if (f[1]=="NO-RESULT") c++} END{print c+0}' "$_d_res")
    _d_graded=$(awk -F"$TAB" '{split($3,f,"|"); g+=f[2]} END{print g+0}' "$_d_res")
    # Lanes that graded a corpus, as its own count. `graded` above is a sum over
    # lanes and one busy lane can carry it; this says how many lanes contributed.
    _d_gradedlanes=$(awk -F"$TAB" '{split($3,f,"|"); if (f[2]+0 > 0) c++} END{print c+0}' "$_d_res")

    echo "lanes:  $_d_n in the registry — $_d_pass PASS, $_d_fail FAIL, $_d_skip SKIP, $_d_none NO-RESULT"
    echo "graded: $_d_graded fixture-verdicts across all lanes"
    echo "sweep:  $_d_swv — $_d_swc of $_d_swsel selected cases reached, ${_d_swg:-?} GRADED by the"
    echo "        oracle (${_d_swu:-?} ungraded: no reference obj), $_d_swm mismatch (corpus $_d_swtot)"
    echo "cross:  $_d_cxv — ${_d_cxg:-?} of $_d_cxsel selected cells graded, $(printf '%s\n' "$_d_cx" | cut -d'|' -f5) mismatch (product $_d_cxtot)"
    if [ -n "$_d_run" ] && [ -d "$_d_run" ]; then echo "logs:   $_d_run/<lane>.log, $_d_run/sweep.log, $_d_run/cross.log"; fi

    # The generated instruments' failures are ruled on FIRST when they carry a
    # mismatch, because a mismatch outranks every other piece of work (CLAUDE.md)
    # and burying it under a lane table is how it goes unread.
    for _g in SWEEP CROSS; do
        _gv=$(gen_tuple "$_g" | cut -d'|' -f1)
        _gm=$(gen_tuple "$_g" | cut -d'|' -f5)
        _gd=$(gen_tuple "$_g" | cut -d'|' -f6)
        _gt=$(gen_tuple "$_g" | cut -d'|' -f4)
        if [ "$_gv" = "FAIL" ]; then
            echo
            echo "GATE: FAIL — $(gen_name "$_g") failed: $_gd"
            if [ "${_gm:-0}" -gt 0 ]; then
                echo
                echo "  *** A MISMATCH IS AN ALARM AND OUTRANKS EVERY OTHER PIECE OF WORK. ***"
                echo "  The real c2.dll under wibo plus a byte-exact obj compare is the sole"
                echo "  judge; outside its class the port must REFUSE, not mis-emit."
                _gl="$_d_run/$( [ "$_g" = SWEEP ] && echo sweep.log || echo cross.log )"
                if [ -n "$_d_run" ] && [ -f "$_gl" ]; then
                    grep '^MISMATCH ' "$_gl" | sed 's/^/    /'
                fi
            else
                resource_banner "$_d_run"
            fi
            return 1
        fi
        if [ "$_gv" = "NO-RESULT" ]; then
            echo
            echo "GATE: FAIL — $(gen_name "$_g") produced NO RESULT: $_gd"
            echo "  An instrument that did not run is a failure, not a pass. Nothing in this"
            echo "  run establishes anything about the ~$_gt $(gen_unit "$_g") it covers."
            resource_banner "$_d_run"
            return 1
        fi
    done

    # ABSENT IS A FAILURE, same rule and same reason as the sweep and the cross:
    # a gate that forgot to run this row must not be able to reach a PASS.
    if [ -z "$_d_hr" ]; then
        echo
        echo "GATE: FAIL — no hatch-red verdict was produced at all."
        echo "  \`work/w-hatch/hatch_red.py\` is part of this gate (board #1406). It fires"
        echo "  every one of \`work/w-front3/hatch.py\`'s eight refusals on purpose; the"
        echo "  \`revert\` those refusals guard had already eaten a peer lane's unstaged"
        echo "  fix once (board #1380), and nothing executed the arms automatically."
        return 1
    fi
    # The hatch-red row's two hard outcomes, ruled on here — above the
    # `--require-graded` line, which is where every `return 1` in this function
    # lives. `REFUSED` is NOT here: it exits 0 and is qualified at the bottom.
    case "$_d_hrv" in
        FAIL)
            echo
            echo "GATE: FAIL — hatch-red: $_d_hrd"
            echo "  \`work/w-hatch/hatch_red.py\` fires \`work/w-front3/hatch.py\`'s refusals on"
            echo "  purpose — 9 red arms, 2 green controls, 8 distinct leading words. A guard"
            echo "  that no longer fires is a guard that is no longer there, and the one this"
            echo "  covers had already discarded a peer lane's unstaged fix (board #1380)."
            if [ -n "$_d_run" ] && [ -f "$_d_run/hatchred.log" ]; then
                echo "  log: $_d_run/hatchred.log"
                grep -E '^\s*\*\*\* ARM FAILED|^  [A-Z][0-9] .*ARM FAILED|^FAILED: ' \
                    "$_d_run/hatchred.log" | sed 's/^/    /' || true
            fi
            return 1 ;;
        NO-RESULT)
            echo
            echo "GATE: FAIL — hatch-red produced NO RESULT: $_d_hrd"
            echo "  An instrument that did not run is a failure, not a pass. Nothing in this"
            echo "  run establishes anything about \`hatch.py\`'s eight refusals."
            return 1 ;;
        PASS|REFUSED) : ;;
        *)
            echo
            echo "GATE: FAIL — hatch-red reported an unrecognized verdict '$_d_hrv'."
            echo "  An unenumerated verdict is the next silence; enumerate it or fix the"
            echo "  classifier, but do not let it fall through to a PASS."
            return 1 ;;
    esac

    # THE LADDER-RED ROW (#1406's second half). Ruled on AFTER hatch-red's, and
    # with its own words throughout — an absent-tuple check that shares a headline
    # with another case's is how a lane had one case steal another's message.
    if [ -z "$_d_lr" ]; then
        echo
        echo "GATE: FAIL — no ladder-red verdict was produced at all."
        echo "  \`work/w-ladders/ladder_red.py\` is part of this gate (board #1406's second"
        echo "  half). It fires \`work/w-front3/ladder.py\`'s \`LADDER-NOWIDTHTABLE\` three"
        echo "  ways and checks the derived width table both ways round; an empty table"
        echo "  would make every ladder rung read as a rename and every price wrong."
        return 1
    fi
    case "$_d_lrv" in
        FAIL)
            echo
            echo "GATE: FAIL — ladder-red: $_d_lrd"
            echo "  \`work/w-ladders/ladder_red.py\` fires \`work/w-front3/ladder.py\`'s width-table"
            echo "  refusal on purpose — 3 red arms, 2 green controls — and it fails 5 of 5"
            echo "  against the \`ladder.py\` this row's guards were added to. A guard that no"
            echo "  longer fires is a guard that is no longer there."
            if [ -n "$_d_run" ] && [ -f "$_d_run/ladderred.log" ]; then
                echo "  log: $_d_run/ladderred.log"
                grep -E '^\s*\*\*\* ARM FAILED|ARM FAILED|^FAILED: ' \
                    "$_d_run/ladderred.log" | sed 's/^/    /' || true
            fi
            return 1 ;;
        NO-RESULT)
            echo
            echo "GATE: FAIL — ladder-red produced NO RESULT: $_d_lrd"
            echo "  An instrument that did not run is a failure, not a pass. Nothing in this"
            echo "  run establishes anything about \`ladder.py\`'s width table."
            return 1 ;;
        PASS|REFUSED) : ;;
        *)
            echo
            echo "GATE: FAIL — ladder-red reported an unrecognized verdict '$_d_lrv'."
            echo "  An unenumerated verdict is the next silence; enumerate it or fix the"
            echo "  classifier, but do not let it fall through to a PASS."
            return 1 ;;
    esac

    if [ "$_d_none" -gt 0 ]; then
        echo
        echo "GATE: FAIL — $_d_none lane(s) produced NO RESULT:"
        awk -F"$TAB" '{split($3,f,"|"); if (f[1]=="NO-RESULT") printf "    %-20s %-24s (%s)\n", $1, $2, f[6]}' "$_d_res"
        echo "  A lane that did not run is a failure, not a pass. Nothing in this run"
        echo "  establishes anything about the configurations those lanes cover."
        resource_banner "$_d_run"
        return 1
    fi

    if [ "$_d_fail" -gt 0 ]; then
        echo
        echo "GATE: FAIL — $_d_fail lane(s) failed:"
        awk -F"$TAB" '{split($3,f,"|"); if (f[1]=="FAIL") printf "    %-20s %-24s (%s)\n", $1, $2, f[6]}' "$_d_res"
        if awk -F"$TAB" '{split($3,f,"|"); if (f[1]=="FAIL" && f[5]+0 > 0) found=1} END{exit !found}' "$_d_res"; then
            echo
            echo "  *** A MISMATCH IS AN ALARM AND OUTRANKS EVERY OTHER PIECE OF WORK. ***"
            echo "  The real c2.dll under wibo plus a byte-exact obj compare is the sole"
            echo "  judge; outside its class the port must REFUSE, not mis-emit."
        else
            resource_banner "$_d_run"
        fi
        return 1
    fi

    # --------------------------------------------------------------------------
    # THE CALLER'S DEMAND: `--require-graded` / `C2RS_GATE_REQUIRE_GRADED=1`.
    #
    # **THIS LINE IS THE LAST POINT AT WHICH EVERY REMAINING OUTCOME EXITS 0.**
    # Everything above returns 1 — completeness, the generated instruments' FAIL
    # and NO-RESULT, the lanes' NO-RESULT, the lanes' FAIL. Everything below
    # returns 0 in some form: SKIPPED, PASS (LANES FILTERED), PASS (SAMPLED),
    # PASS. So one check placed here covers every zero-exit outcome the gate has
    # AND every one a future lane adds, which an enumeration of today's empty
    # outcomes would not. If a `return 1` is ever added below this line, this
    # comment is the thing that has become wrong; move the block, do not copy it.
    #
    # It duplicates nothing. A partial skip, a vacuous lane, a short sweep and a
    # NO-RESULT are all already failures and are all already behind us; the demand
    # only ever converts a would-be zero exit.
    #
    # The quantity is a COUNT — the standing mitigation is *compare a count, never
    # a status* — and it is summed across the whole gate rather than read off any
    # one instrument, because "this run graded something" is a fact about the run.
    # --------------------------------------------------------------------------
    if [ "${require_graded:-0}" -eq 1 ]; then
        _d_units=$(( _d_graded + $(num "$_d_swg") + $(num "$_d_cxg") ))
        if [ "$_d_units" -eq 0 ]; then
            echo
            echo "GATE: FAIL (NOTHING GRADED) — you asked for a graded run and got 0."
            echo "  units graded, summed over this whole gate:  $_d_units"
            echo "    lanes that graded a corpus   $_d_gradedlanes of $_d_n"
            echo "    fixture-verdicts             $_d_graded"
            echo "    sweep cases graded           $(num "$_d_swg")   ($_d_swv)"
            echo "    cross cells graded           $(num "$_d_cxg")   ($_d_cxv)"
            if [ "$_d_skip" -eq "$_d_n" ]; then
                echo "  Every one of the $_d_n lanes SKIPPED: the toolchain did not resolve."
            else
                echo "  This run reached a zero-exit outcome having graded nothing, and it is"
                echo "  NOT the all-skip one — the counts above are the whole story about it."
            fi
            echo
            echo "  Without --require-graded this run exits 0 BY DESIGN, and that is not a"
            echo "  bug: CLAUDE.md requires the toolchain-absent path to degrade cleanly and"
            echo "  the portable lane has no toolchain. The demand is the CALLER's — you"
            echo "  said this run had to establish something, and it established nothing."
            echo "  Do not report this tree as gated. Nothing here is evidence about the"
            echo "  port: not that it is right, not that it is wrong."
            if [ "$_d_skip" -eq "$_d_n" ]; then
                toolchain_hint
            fi
            return 1
        fi
    fi

    if [ "$_d_skip" -eq "$_d_n" ]; then
        # Toolchain absence must skip the generated instruments too. If it did
        # not, one of them resolved a toolchain the others could not, which is a
        # fault in the resolution, not a degradation — the partial-skip rule
        # applied across the gate's halves instead of across the lanes.
        for _g in SWEEP CROSS; do
            _gv=$(gen_tuple "$_g" | cut -d'|' -f1)
            if [ "$_gv" != "SKIP" ]; then
                echo
                echo "GATE: FAIL — all $_d_n lanes skipped but $(gen_name "$_g") reported $_gv."
                echo "  Toolchain absence skips EVERYTHING. One part resolving a toolchain"
                echo "  another could not is a fault in the resolution."
                return 1
            fi
        done
        echo
        echo "GATE: SKIPPED — all $_d_n lanes, the sweep and the cross skipped, NOTHING WAS GRADED."
        echo "  (hatch-red is toolchain-free and reported $_d_hrv, ladder-red likewise and"
        echo "  reported $_d_lrv — they grade INSTRUMENTS, not objs, and deliberately count"
        echo "  toward nothing on this line.)"
        echo "  The toolchain is absent (see CLAUDE.md); this exits 0 by design and is"
        echo "  NOT a green gate. This run establishes nothing about the port."
        echo "  Run with --require-graded (or C2RS_GATE_REQUIRE_GRADED=1) to make this"
        echo "  exit 1 instead — that is the flag for a caller that meant to gate a tree."
        toolchain_hint
        return 0
    fi
    if [ "$_d_skip" -gt 0 ]; then
        echo
        echo "GATE: FAIL — $_d_skip of $_d_n lanes skipped while $_d_pass ran."
        echo "  Toolchain absence skips EVERY lane. A partial skip means a lane declined"
        echo "  for a reason of its own, which is a fault, not a degradation:"
        awk -F"$TAB" '{split($3,f,"|"); if (f[1]=="SKIP") printf "    %-20s %s\n", $1, $2}' "$_d_res"
        return 1
    fi
    for _g in SWEEP CROSS; do
        if [ "$(gen_tuple "$_g" | cut -d'|' -f1)" = "SKIP" ]; then
            echo
            echo "GATE: FAIL — $_d_pass lanes graded a corpus and $(gen_name "$_g") skipped."
            echo "  The lanes found a toolchain, so its absence is not a degradation."
            return 1
        fi
    done

    echo
    # A REFUSED hatch-red exits 0 and forfeits the unqualified headline, exactly
    # as SAMPLED and SKIPPED do. `_d_hrq` is appended to whichever zero-exit
    # headline is reached below, so a future headline that forgets it is a
    # missing suffix rather than a missing check.
    _d_hrq=""
    if [ "$_d_hrv" = REFUSED ]; then _d_hrq=" (HATCH-RED REFUSED)"; fi
    # And the second instrument row, APPENDED rather than merged. Both can refuse
    # at once (a dirty `expr.rs` trips the wide interlock and the narrow one), and
    # a merged suffix could not say which row it was.
    if [ "$_d_lrv" = REFUSED ]; then _d_hrq="$_d_hrq (LADDER-RED REFUSED)"; fi
    # `--lane` filters the registry, and every check above then treats the filtered
    # list AS the registry — which is right, and which also means a one-lane run
    # can print `12/12 lanes ran` shaped exactly like a full gate. Same hole as an
    # unqualified PASS over a sampled sweep, on the half that already existed.
    if [ -n "$_d_filt" ]; then
        echo "GATE: PASS (LANES FILTERED)$_d_hrq — $_d_pass/$_d_n SELECTED lanes ran, out of"
        echo "  $_d_filt in the registry. --lane is for iterating; this run says nothing"
        echo "  about the lanes it did not run. Re-run without --lane before reporting."
        if [ "$_d_swv" = "SAMPLED" ]; then
            echo "  The sweep was also SAMPLED — $_d_swc of $_d_swtot generated cases."
        fi
        if [ "$_d_cxv" = "SAMPLED" ]; then
            echo "  The cross was also SAMPLED — $_d_cxsel of $_d_cxtot case-lane cells."
        fi
        hatch_refusal_note
        ladder_refusal_note
        return 0
    fi
    if [ "$_d_swv" = "SAMPLED" ] || [ "$_d_cxv" = "SAMPLED" ]; then
        # A sample is a legitimate way to iterate and an illegitimate way to
        # report. It exits 0 and it does NOT get to print an unqualified PASS —
        # same treatment as GATE: SKIPPED, for the same reason.
        echo "GATE: PASS (SAMPLED)$_d_hrq — $_d_pass/$_d_n lanes ran and every one of them graded"
        echo "  a corpus, but a generated instrument graded only part of its corpus:"
        [ "$_d_swv" = "SAMPLED" ] && \
            echo "    expr-sweep  $_d_swc of $_d_swtot generated cases"
        [ "$_d_cxv" = "SAMPLED" ] && \
            echo "    mode-cross  $_d_cxsel of $_d_cxtot case-lane cells"
        echo "  A strided sample is unbiased across fragments and is still a sample: this"
        echo "  run does NOT establish what a full run establishes. Re-run without"
        echo "  --sweep-cases / --cross-cells before reporting or landing."
        hatch_refusal_note
        ladder_refusal_note
        return 0
    fi
    echo "GATE: PASS$_d_hrq — $_d_pass/$_d_n lanes ran and every one of them graded a corpus,"
    echo "  the sweep graded ${_d_swg:-?} of $_d_swtot generated cases and the cross graded"
    echo "  ${_d_cxg:-?} of $_d_cxtot case-lane cells, with 0 mismatches anywhere"
    echo "  (${_d_swu:-?} sweep cases carried ungraded — the reference rejects the source)."
    hatch_refusal_note
    ladder_refusal_note
    return 0
}

# --------------------------------------------------------------------------------
# Registry load + `--lane` filter, shared by every mode.
# --------------------------------------------------------------------------------
# The run dir itself costs an inode, so on a filesystem that is ALREADY at zero the
# gate dies here — before the preflight that exists to explain it. `set -e` would
# make that a bare non-zero exit with a `mkdir` error, which reads like a broken
# script rather than a full disk. Same verdict, same exit 3, one line earlier.
if ! mkdir -p "$work" 2>/dev/null; then
    echo "GATE: FAIL (DISK) — could not even create the run dir $work."
    df -kP "$(dirname "$work")" 2>/dev/null | sed 's/^/  /'
    df -iP "$(dirname "$work")" 2>/dev/null | sed 's/^/  /'
    echo "  *** THIS IS A RESOURCE FAULT AND NOT A MISMATCH. *** Exit 3; nothing ran."
    echo "  Note the INODE line: this filesystem can be full of inodes with bytes to spare."
    exit 3
fi
# --------------------------------------------------------------------------------
# REAP, THEN CHECK, THEN WRITE ANYTHING — in that order, and BEFORE the registry is
# parsed or the pid is stamped.
#
# Both of those write to the run tree, and on a filesystem with no bytes left the
# write fails, `set -e` fires, and the gate exits **1** with a bare
# `echo: write error: No space left on device`. Exit 1 is the exit code of a real
# gate failure. Measured on a purpose-built 2 MB tmpfs while writing this
# (`work/w-ledger/tinyfs_test.sh` case `space-gone`) — the first draft of this fix
# had the preflight *after* these two writes and never reached it.
# --------------------------------------------------------------------------------
work_parent=$(dirname "$work")
if [ "$mode" = run ] || [ "$mode" = reap ]; then
    echo "lane gate: preflight on $work_parent"
    if [ "$reap" -eq 1 ]; then
        reap_run_trees "$work_parent" "$work" "$C2RS_GATE_KEEP" "$work"
    elif [ "$reap" -eq 2 ]; then
        echo "reap:   DRY RUN (--reap-dry-run) — classification only, nothing removed."
        reap_run_trees "$work_parent" "$work" "$C2RS_GATE_KEEP" "$work" dry
    else
        echo "reap:   SKIPPED (--no-reap). Old run trees are being kept deliberately;"
        echo "        one is ~112 MB and ~16.6k inodes and ~63 of them exhaust a 1M-inode /tmp."
    fi
    res_init "$work_parent"
    preflight_disk "$work_parent" "$C2RS_GATE_MIN_MB" "$C2RS_GATE_MIN_INODES" || exit 3
fi

# --------------------------------------------------------------------------------
# `--reap-only` stops here. The reaper and the preflight are the two things on this
# box that answer "can the next gate run at all", and until now the only way to
# reach either was to grade 16,710 cases across 18 lanes first. That made
# `--reap-dry-run` — whose whole documented purpose is checking the concurrency
# rule against a LIVE SHARED /tmp — cost twenty minutes of compiler to exercise a
# classification that takes a second, so in practice nobody ran it, and the reaper
# was only ever observed through the runs it was a preamble to.
#
# It grades nothing, so it cannot PASS or FAIL a port and does not print a verdict
# that could be mistaken for one. Exit 0 = the reaper ran and the floors are clear;
# exit 3 = the floors are not clear, same code and same meaning as everywhere else.
# --------------------------------------------------------------------------------
if [ "$mode" = reap ]; then
    # Leave no litter. The reaper wrote its two scratch files into this run's own
    # dir, and a housekeeping command that adds a tree to the pile it just pruned
    # would be its own joke. `rmdir` (not `rm -rf`) so that anything unexpected in
    # there survives and the directory stays, visibly, rather than being swept.
    rm -f "$work/reap.stale" "$work/reap.sorted" 2>/dev/null || true
    if rmdir "$work" 2>/dev/null; then
        echo "reap-only: nothing was graded, and this run left no tree of its own."
    else
        echo "reap-only: nothing was graded. $work is not empty and was left alone."
    fi
    echo "           This is housekeeping, not a verdict — no port was checked."
    exit 0
fi

# Stamp the owner INTO the tree. The directory name has always carried the pid, but
# a name is not a claim: a tree named for a pid that has since been recycled is
# indistinguishable from a tree whose gate is still running, and this box mints
# hundreds of pids a night. `gate.pid` plus `/proc/<pid>/cmdline` is what lets the
# reaper tell a live concurrent lane from a corpse.
if ! echo "$$" > "$work/gate.pid" 2>/dev/null; then
    echo "GATE: FAIL (DISK) — cannot write $work/gate.pid; $work_parent is full."
    df -kP "$work_parent" 2>/dev/null | sed 's/^/  /'
    df -iP "$work_parent" 2>/dev/null | sed 's/^/  /'
    echo "  *** THIS IS A RESOURCE FAULT AND NOT A MISMATCH. *** Exit 3; nothing ran."
    exit 3
fi
reg="$work/registry.tsv"
parse_registry "$registry" "$reg"

# `--lane` filters the registry, and the filtered list then IS the registry for
# every check above — so `--lane` naming nothing is an empty registry and a hard
# error, never a run of zero lanes that exits 0.
filtered=""
if [ -n "$want" ]; then
    filtered=$(wc -l < "$reg")
    sel="$work/selected.tsv"; : > "$sel"
    for w in $want; do
        if ! awk -F"$TAB" -v w="$w" '$1==w{found=1} END{exit !found}' "$reg"; then
            echo "FATAL: --lane '$w' is not in $registry. Known lanes:" >&2
            cut -f1 "$reg" | sed 's/^/    /' >&2
            exit 2
        fi
        awk -F"$TAB" -v w="$w" '$1==w' "$reg" >> "$sel"
    done
    reg="$sel"
fi
nlanes=$(wc -l < "$reg")

case "$mode" in
list)
    echo "lane registry: $registry  ($nlanes lanes)"
    awk -F"$TAB" '{printf "  %-20s %s\n", $1, $2}' "$reg"
    exit 0 ;;
check)
    echo "lane registry: $registry"
    echo "  $nlanes lanes, slugs unique, every row parses."
    awk -F"$TAB" '{printf "  %-20s %s\n", $1, $2}' "$reg"
    # The corpus's own shape coverage, asserted. Toolchain-free, so it belongs in
    # the preflight rather than in the graded run: a fragment deleted or a
    # generator that stops emitting one of its axes puts a shape marker back at
    # zero, and every other instrument here keeps printing a clean count over a
    # corpus that can no longer say the thing. See `scripts/sweep_shapes.py
    # --check`; `--selftest` proves it fails when it should.
    echo
    if command -v python3 >/dev/null 2>&1; then
        python3 "$repo_root/scripts/sweep_shapes.py" --check || exit 1
    else
        echo "FATAL: no python3 — the corpus shape check and the sweep both need it." >&2
        exit 2
    fi
    # And the gate's own ability to be wrong about which tree it graded (#2907).
    # It belongs in the preflight for the corpus check's reason — toolchain-free,
    # seconds — and for one of its own: the answer is a property of this file and
    # of `work/w-hatch/hatch_red.py`, so it is knowable before twenty minutes of
    # compiler rather than after.
    echo
    echo "tree integrity (boards #2668 / #2907): the gate cannot report a tree it did not grade"
    tree_integrity_arms "$work/integrity" || exit 1
    exit 0 ;;
esac

if [ "$mode" = selftest ]; then
    # ----------------------------------------------------------------------------
    # Prove the gate fails when it should, with no toolchain and no compiler.
    # Every case fabricates lane logs and drives the REAL collect+decide path —
    # not a reimplementation of it, which would only prove the copy agrees.
    #
    # No command substitution anywhere in the loop: a `fails` counter incremented
    # inside `$(...)` lives in a subshell and is discarded, which would make this
    # selftest itself an instrument that reports green from an absence.
    # ----------------------------------------------------------------------------
    st="$work/selftest"; rm -rf "$st"; mkdir -p "$st"
    printf 'A\t/O1\nB\t/O1 /EHsc\n' > "$st/reg.tsv"
    fails=0
    cases=0
    CASE_DIR=""
    # The generated-instrument verdicts every lane case is driven with, so those
    # cases keep testing exactly what they tested before. Instrument-specific
    # cases override them. Eight fields: the last two are `graded` and `ungraded`,
    # added 2026-08-04 when the sweep was found to be counting 96 cases the oracle
    # never ruled on inside `checked`.
    SWEEP_OK='PASS|14635|14635|14635|0||14539|96'
    SWEEP_FOR_CASE="$SWEEP_OK"
    CROSS_OK='PASS|61539|61539|61539|0||61151|388'
    CROSS_FOR_CASE="$CROSS_OK"
    # The hatch-red row (board #1406). Every pre-existing case is driven with a
    # clean tuple so its verdict is what it always was; the row's own cases
    # override it. An EMPTY tuple stays a hard failure in `decide`, which is only
    # safe because every caller here supplies one.
    HR_OK='PASS|11|11|9|2|'
    HR_FOR_CASE="$HR_OK"
    # The ladder-red tuple, same shape (#1406's second half). Fabricated, like
    # HR_OK: these cases drive the CLASSIFIER and the RULING, never the arms.
    LR_OK='PASS|5|5|3|2|'
    LR_FOR_CASE="$LR_OK"

    check_that() {  # <label> <ok?0/1>
        if [ "$2" -eq 0 ]; then
            printf '        %s\n' "also: $1"
        else
            printf '  FAIL  %s\n' "also: $1"
            fails=$((fails + 1))
        fi
    }

    run_case() {  # <name> <PASS|FAIL> <slug=body>...
        _rc_name="$1"; _rc_want="$2"; shift 2
        CASE_DIR="$st/$_rc_name"
        rm -rf "$CASE_DIR"; mkdir -p "$CASE_DIR"
        for _rc_spec in "$@"; do
            _rc_slug=${_rc_spec%%=*}
            _rc_body=${_rc_spec#*=}
            case "$_rc_body" in
                MISSING) : ;;
                NOLINE)
                    echo "grading 197 fixtures at /O1" > "$CASE_DIR/$_rc_slug.log"
                    echo 0 > "$CASE_DIR/$_rc_slug.status" ;;
                *)
                    printf '%s\n' "$_rc_body" > "$CASE_DIR/$_rc_slug.log"
                    echo 0 > "$CASE_DIR/$_rc_slug.status" ;;
            esac
        done
        collect "$st/reg.tsv" "$CASE_DIR" "$CASE_DIR/results.tsv"
        _rc_got=PASS
        if ! decide "$st/reg.tsv" "$CASE_DIR/results.tsv" "" "$SWEEP_FOR_CASE" "" \
                "$CROSS_FOR_CASE" "$HR_FOR_CASE" "$LR_FOR_CASE" > "$CASE_DIR/out.txt" 2>&1; then
            _rc_got=FAIL
        fi
        SWEEP_FOR_CASE="$SWEEP_OK"
        CROSS_FOR_CASE="$CROSS_OK"
        HR_FOR_CASE="$HR_OK"
        LR_FOR_CASE="$LR_OK"
        _rc_hdl=$(grep -m1 '^GATE: ' "$CASE_DIR/out.txt" || echo 'GATE: <none printed>')
        cases=$((cases + 1))
        if [ "$_rc_got" = "$_rc_want" ]; then
            printf '  ok    %-32s %s\n' "$_rc_name" "$_rc_hdl"
        else
            printf '  FAIL  %-32s wanted %s, got %s — %s\n' "$_rc_name" "$_rc_want" "$_rc_got" "$_rc_hdl"
            fails=$((fails + 1))
        fi
    }
    saw()    { if grep -q "$1" "$CASE_DIR/out.txt"; then check_that "$2" 0; else check_that "$2" 1; fi; }
    saw_no() { if grep -q "$1" "$CASE_DIR/out.txt"; then check_that "$2" 1; else check_that "$2" 0; fi; }

    P='LANE-RESULT PASS flags=[/O1 /GS- /c] graded=197 total=197 match=91 mismatch=0'
    M='LANE-RESULT FAIL flags=[/O1 /EHsc /GS- /c] graded=197 total=197 match=90 mismatch=1'
    V='LANE-RESULT FAIL flags=[/O1 /EHsc /GS- /c] graded=0 total=197 match=0 mismatch=0'
    S='LANE-RESULT SKIP flags=[/O1 /EHsc /GS- /c] graded=0 total=197 match=0 mismatch=0'
    L1='LANE-RESULT PASS flags=[/O1 /EHsc /GS- /c] graded=0 total=197 match=0 mismatch=0'
    L2='LANE-RESULT PASS flags=[/O1 /EHsc /GS- /c] graded=197 total=197 match=90 mismatch=3'

    echo "gate.sh --selftest: driving the real collect+decide with fabricated lane logs"
    echo

    run_case both-pass PASS "A=$P" "B=$P"
    saw 'GATE: PASS' 'a wholly green run does say PASS'

    run_case lane-B-mismatch FAIL "A=$P" "B=$M"
    saw '^    B ' 'the failing lane is NAMED'
    saw 'ALARM'   'a mismatch raises the alarm banner'

    run_case lane-B-vacuous              FAIL "A=$P" "B=$V"
    run_case lane-B-no-log-at-all        FAIL "A=$P" "B=MISSING"
    saw 'NO RESULT' 'a lane that never ran is NO RESULT, not a pass'

    run_case lane-B-exit-0-no-result     FAIL "A=$P" "B=NOLINE"
    saw 'NO RESULT' 'exit 0 alone is not evidence a lane ran'

    run_case lane-B-lies-graded-0        FAIL "A=$P" "B=$L1"
    run_case lane-B-lies-with-mismatch   FAIL "A=$P" "B=$L2"
    run_case both-absent-is-not-a-skip   FAIL "A=MISSING" "B=MISSING"

    SWEEP_FOR_CASE='SKIP|0|0|0|0|toolchain absent'
    CROSS_FOR_CASE='SKIP|0|0|0|0|toolchain absent'
    run_case all-skip PASS "A=$S" "B=$S"
    saw    'GATE: SKIPPED' 'all-skip says SKIPPED and that nothing was graded'
    saw_no 'GATE: PASS'    'all-skip never says PASS'

    run_case partial-skip FAIL "A=$P" "B=$S"

    # ---- the SWEEP row (board #232) --------------------------------------------
    # Every one of these drives the real `sweep_verdict` + `decide` path. The gate
    # had ZERO references to the sweep until 2026-08-04, and a check nobody has
    # seen fail is not evidence of anything.

    # Drives ONE generated-instrument row from a fabricated log through the real
    # `sweep_verdict` + `decide`. `which` selects the row, so the sweep and the
    # cross are proved against the SAME rules rather than against two copies.
    gen_case() {  # <which:sweep|cross> <name> <PASS|FAIL> <log-body-or-MISSING> [exit]
        _sc_w="$1"; _sc_name="$2"; _sc_want="$3"; _sc_body="$4"; _sc_st="${5:-0}"
        CASE_DIR="$st/$_sc_name"; rm -rf "$CASE_DIR"; mkdir -p "$CASE_DIR"
        printf '%s\n' "$P" > "$CASE_DIR/A.log"; echo 0 > "$CASE_DIR/A.status"
        printf '%s\n' "$P" > "$CASE_DIR/B.log"; echo 0 > "$CASE_DIR/B.status"
        _sc_log="$CASE_DIR/$_sc_w.log"
        if [ "$_sc_body" = MISSING ]; then
            rm -f "$_sc_log"
        else
            printf '%s\n' "$_sc_body" > "$_sc_log"
        fi
        _sc_v=$(sweep_verdict "$_sc_log" "$_sc_st")
        if [ "$_sc_w" = sweep ]; then
            _sc_sw="$_sc_v"; _sc_cx="$CROSS_OK"
        else
            _sc_sw="$SWEEP_OK"; _sc_cx="$_sc_v"
        fi
        collect "$st/reg.tsv" "$CASE_DIR" "$CASE_DIR/results.tsv"
        _sc_got=PASS
        if ! decide "$st/reg.tsv" "$CASE_DIR/results.tsv" "$CASE_DIR" "$_sc_sw" "" \
                "$_sc_cx" "$HR_OK" "$LR_OK" > "$CASE_DIR/out.txt" 2>&1; then
            _sc_got=FAIL
        fi
        _sc_hdl=$(grep -m1 '^GATE: ' "$CASE_DIR/out.txt" || echo 'GATE: <none printed>')
        cases=$((cases + 1))
        if [ "$_sc_got" = "$_sc_want" ]; then
            printf '  ok    %-32s %s\n' "$_sc_name" "$_sc_hdl"
        else
            printf '  FAIL  %-32s wanted %s, got %s — %s\n' \
                "$_sc_name" "$_sc_want" "$_sc_got" "$_sc_hdl"
            fails=$((fails + 1))
        fi
    }
    sweep_case() { gen_case sweep "$@"; }
    cross_case() { gen_case cross "$@"; }

    SW_FULL='sweeping 14635 of 14635 generated cases
checked=14635 mismatches=0 graded=14539 ungraded=96 unknown=0'
    SW_MISM='sweeping 14635 of 14635 generated cases
MISMATCH  /t/62-ctor-base-delegation-0032.cpp  |  struct Bd { Bd(); ~Bd(); int b0; };
checked=14635 mismatches=1 graded=14539 ungraded=95 unknown=0'
    SW_SHORT='sweeping 14635 of 14635 generated cases
checked=9107 mismatches=0 graded=9011 ungraded=96 unknown=0'
    SW_VAC='sweeping 0 of 14635 generated cases
checked=0 mismatches=0 graded=0 ungraded=0 unknown=0'
    SW_SAMP='sweeping 400 of 14635 generated cases (STRIDE 37 — a SAMPLE, not the corpus)
checked=400 mismatches=0 graded=397 ungraded=3 unknown=0'
    SW_NOCOUNT='sweeping 14635 of 14635 generated cases'
    SW_NOSWEEP='checked=14635 mismatches=0 graded=14539 ungraded=96 unknown=0'
    SW_SKIP='SKIP: toolchain absent — the sweep would be vacuous'
    # The pre-2026-08-04 count line. `checked` was reported as though it were
    # `graded`, and 96 of 14,635 cases had no reference obj at all — `c2rs diff`
    # prints four verdicts and the sweep's `case` recognized one. A log in the old
    # shape is NO-RESULT, not a log saying `ungraded=0`.
    SW_OLDLINE='sweeping 14635 of 14635 generated cases
checked=14635 mismatches=0'
    # Every case reached, NONE ruled on by the oracle. `mismatches=0` over
    # `graded=0` is the vacuous green this whole file exists to forbid, and it was
    # unreachable before `graded` existed as a separate number.
    SW_ALLUNGRADED='sweeping 14635 of 14635 generated cases
checked=14635 mismatches=0 graded=0 ungraded=14635 unknown=0'
    # A verdict string the classifier does not enumerate. That is how the 96 hid:
    # they fell out of a `case` with no default arm and were counted as clean.
    SW_UNKNOWN='sweeping 14635 of 14635 generated cases
checked=14635 mismatches=0 graded=14634 ungraded=0 unknown=1'

    CX_FULL='sweeping 61539 of 61539 case-lane cells
checked=61539 mismatches=0 graded=61151 ungraded=388 unknown=0'
    CX_MISM='sweeping 61539 of 61539 case-lane cells
MISMATCH  [/O1 /EHsc /GS- /c]  63-emit-order-0117.cpp
checked=61539 mismatches=1 graded=61151 ungraded=388 unknown=0'
    CX_SHORT='sweeping 61539 of 61539 case-lane cells
checked=47000 mismatches=0 graded=46700 ungraded=300 unknown=0'
    CX_SAMP='sweeping 4000 of 61539 case-lane cells (STRIDE 16 — a SAMPLE, not the product)
checked=4000 mismatches=0 graded=3975 ungraded=25 unknown=0'
    CX_SKIP='SKIP: toolchain absent — the cross would be vacuous'
    CX_NOCOUNT='sweeping 61539 of 61539 case-lane cells'

    sweep_case sweep-clean            PASS "$SW_FULL"
    saw 'GATE: PASS' 'a full clean sweep does say PASS'
    saw '14539 of 14635' 'the PASS line carries the sweep GRADED count over the corpus'

    sweep_case sweep-mismatch         FAIL "$SW_MISM" 1
    saw 'ALARM' 'a sweep mismatch raises the alarm banner'
    saw '62-ctor-base-delegation-0032' 'the failing CASE is named, by file'

    sweep_case sweep-short-count      FAIL "$SW_SHORT"
    saw 'SHORT' 'grading fewer cases than were selected is a FAIL, not a pass'

    sweep_case sweep-vacuous          FAIL "$SW_VAC"
    sweep_case sweep-no-checked-line  FAIL "$SW_NOCOUNT" 137
    saw 'NO RESULT' 'a sweep that died mid-run is NO RESULT, not 0 mismatches'

    sweep_case sweep-no-sweeping-line FAIL "$SW_NOSWEEP"
    saw 'NO RESULT' 'a checked= line with no corpus count is not a result'

    sweep_case sweep-log-missing      FAIL MISSING
    sweep_case sweep-clean-but-exit-1 FAIL "$SW_FULL" 1

    sweep_case sweep-sampled          PASS "$SW_SAMP"
    saw    'GATE: PASS (SAMPLED)' 'a sampled sweep says so in the headline'
    saw_no '^GATE: PASS —' 'a sampled sweep never prints an unqualified PASS'

    # Lanes green, sweep skipped: the partial-skip rule across the gate's halves.
    sweep_case sweep-skip-while-lanes-ran FAIL "$SW_SKIP"

    # ---- the three holes the sweep's own classifier had (w-modes) -------------
    sweep_case sweep-pre-graded-count  FAIL "$SW_OLDLINE"
    saw 'NO RESULT' 'a count line with no graded= is NO RESULT, not ungraded=0'
    sweep_case sweep-none-graded       FAIL "$SW_ALLUNGRADED"
    saw 'NONE graded' 'every case reached and none GRADED is vacuous, not clean'
    sweep_case sweep-unknown-verdict   FAIL "$SW_UNKNOWN"
    saw 'UNRECOGNIZED' 'an unenumerated verdict fails instead of counting as clean'

    # ---- the MODE-CROSS row (w-order Y-a) -------------------------------------
    # Same rules, same `sweep_verdict`, driven through the second row. The cross
    # is the only instrument that can see a defect needing a shape the fixtures
    # do not have AND a flag the sweep does not run.
    cross_case cross-clean             PASS "$CX_FULL"
    saw 'GATE: PASS' 'a full clean cross does say PASS'
    saw '61151 of 61539' 'the PASS line carries the cross GRADED count over the product'

    cross_case cross-mismatch          FAIL "$CX_MISM" 1
    saw 'ALARM' 'a cross mismatch raises the alarm banner'
    saw '63-emit-order-0117' 'the failing CASE is named, by file'
    saw 'O1 /EHsc' 'and the LANE it mismatched at is named too'

    cross_case cross-short-count       FAIL "$CX_SHORT"
    saw 'SHORT' 'grading fewer cells than were selected is a FAIL, not a pass'

    cross_case cross-log-missing       FAIL MISSING
    cross_case cross-no-checked-line   FAIL "$CX_NOCOUNT" 137
    cross_case cross-clean-but-exit-1  FAIL "$CX_FULL" 1

    cross_case cross-sampled           PASS "$CX_SAMP"
    saw    'GATE: PASS (SAMPLED)' 'a sampled cross says so in the headline'
    saw_no '^GATE: PASS —'        'a sampled cross never prints an unqualified PASS'

    cross_case cross-skip-while-lanes-ran FAIL "$CX_SKIP"

    # And the one the cross row exists for: no cross verdict AT ALL must fail,
    # even with every lane green and a clean sweep. This is the pre-w-modes gate,
    # which was green over w-order's Y-a for its whole life.
    CASE_DIR="$st/cross-absent"; rm -rf "$CASE_DIR"; mkdir -p "$CASE_DIR"
    printf '%s\n' "$P" > "$CASE_DIR/A.log"; echo 0 > "$CASE_DIR/A.status"
    printf '%s\n' "$P" > "$CASE_DIR/B.log"; echo 0 > "$CASE_DIR/B.status"
    collect "$st/reg.tsv" "$CASE_DIR" "$CASE_DIR/results.tsv"
    cases=$((cases + 1))
    if decide "$st/reg.tsv" "$CASE_DIR/results.tsv" "$CASE_DIR" "$SWEEP_OK" "" "" \
            "$HR_OK" "$LR_OK" > "$CASE_DIR/out.txt" 2>&1; then
        printf '  FAIL  %-32s a run with NO cross verdict PASSED\n' cross-absent
        fails=$((fails + 1))
    else
        printf '  ok    %-32s %s\n' cross-absent "$(grep -m1 '^GATE: ' "$CASE_DIR/out.txt")"
    fi

    # All lanes skipped but the cross reported a result: the partial-skip rule
    # across the gate's THIRD part, which did not exist before this row.
    CASE_DIR="$st/allskip-cross-ran"; rm -rf "$CASE_DIR"; mkdir -p "$CASE_DIR"
    printf '%s\n' "$S" > "$CASE_DIR/A.log"; echo 0 > "$CASE_DIR/A.status"
    printf '%s\n' "$S" > "$CASE_DIR/B.log"; echo 0 > "$CASE_DIR/B.status"
    collect "$st/reg.tsv" "$CASE_DIR" "$CASE_DIR/results.tsv"
    cases=$((cases + 1))
    if decide "$st/reg.tsv" "$CASE_DIR/results.tsv" "$CASE_DIR" \
            'SKIP|0|0|0|0|toolchain absent' "" "$CROSS_OK" "$HR_OK" "$LR_OK" \
            > "$CASE_DIR/out.txt" 2>&1; then
        printf '  FAIL  %-32s all lanes skipped, cross ran, and it PASSED\n' allskip-cross-ran
        fails=$((fails + 1))
    else
        printf '  ok    %-32s %s\n' allskip-cross-ran "$(grep -m1 '^GATE: ' "$CASE_DIR/out.txt")"
    fi

    # And the one this whole addition exists for: no sweep verdict AT ALL must
    # fail, even with every lane green. This is the pre-2026-08-04 gate.
    CASE_DIR="$st/sweep-absent"; rm -rf "$CASE_DIR"; mkdir -p "$CASE_DIR"
    printf '%s\n' "$P" > "$CASE_DIR/A.log"; echo 0 > "$CASE_DIR/A.status"
    printf '%s\n' "$P" > "$CASE_DIR/B.log"; echo 0 > "$CASE_DIR/B.status"
    collect "$st/reg.tsv" "$CASE_DIR" "$CASE_DIR/results.tsv"
    cases=$((cases + 1))
    if decide "$st/reg.tsv" "$CASE_DIR/results.tsv" "$CASE_DIR" "" "" "$CROSS_OK" \
            "$HR_OK" "$LR_OK" > "$CASE_DIR/out.txt" 2>&1; then
        printf '  FAIL  %-32s a run with NO sweep verdict PASSED\n' sweep-absent
        fails=$((fails + 1))
    else
        printf '  ok    %-32s %s\n' sweep-absent "$(grep -m1 '^GATE: ' "$CASE_DIR/out.txt")"
    fi

    # A --lane-filtered run is a legitimate way to iterate and an illegitimate way
    # to report, exactly like a sampled sweep.
    CASE_DIR="$st/lanes-filtered"; rm -rf "$CASE_DIR"; mkdir -p "$CASE_DIR"
    printf '%s\n' "$P" > "$CASE_DIR/A.log"; echo 0 > "$CASE_DIR/A.status"
    printf '%s\n' "$P" > "$CASE_DIR/B.log"; echo 0 > "$CASE_DIR/B.status"
    collect "$st/reg.tsv" "$CASE_DIR" "$CASE_DIR/results.tsv"
    cases=$((cases + 1))
    if decide "$st/reg.tsv" "$CASE_DIR/results.tsv" "$CASE_DIR" "$SWEEP_OK" 12 \
            "$CROSS_OK" "$HR_OK" "$LR_OK" > "$CASE_DIR/out.txt" 2>&1; then
        if grep -q 'LANES FILTERED' "$CASE_DIR/out.txt" \
           && ! grep -q '^GATE: PASS —' "$CASE_DIR/out.txt"; then
            printf '  ok    %-32s %s\n' lanes-filtered "$(grep -m1 '^GATE: ' "$CASE_DIR/out.txt")"
        else
            printf '  FAIL  %-32s a --lane run printed an unqualified PASS\n' lanes-filtered
            fails=$((fails + 1))
        fi
    else
        printf '  FAIL  %-32s a --lane run over green lanes did not exit 0\n' lanes-filtered
        fails=$((fails + 1))
    fi

    # The completeness assertion itself: a table short a row must fail even when
    # every row it does contain is a PASS.
    CASE_DIR="$st/short-table"; mkdir -p "$CASE_DIR"
    printf 'A\t/O1\tPASS|197|197|91|0|\n' > "$CASE_DIR/results.tsv"
    cases=$((cases + 1))
    if decide "$st/reg.tsv" "$CASE_DIR/results.tsv" "" "$SWEEP_OK" "" "$CROSS_OK" \
            "$HR_OK" "$LR_OK" > "$CASE_DIR/out.txt" 2>&1; then
        printf '  FAIL  %-32s a 1-row table for a 2-lane registry PASSED\n' short-table
        fails=$((fails + 1))
    else
        printf '  ok    %-32s %s\n' short-table "$(grep -m1 '^GATE: ' "$CASE_DIR/out.txt")"
    fi

    # An empty registry must be a hard error, not a run of zero lanes.
    : > "$st/empty.txt"
    cases=$((cases + 1))
    if parse_registry "$st/empty.txt" "$st/empty.tsv" >/dev/null 2>&1; then
        printf '  FAIL  %-32s an empty registry parsed clean\n' empty-registry
        fails=$((fails + 1))
    else
        printf '  ok    %-32s refused (a gate over 0 lanes cannot exist)\n' empty-registry
    fi

    # As must a duplicated slug.
    printf 'A /O1\nA /Ox\n' > "$st/dup.txt"
    cases=$((cases + 1))
    if parse_registry "$st/dup.txt" "$st/dup.tsv" >/dev/null 2>&1; then
        printf '  FAIL  %-32s a duplicated slug parsed clean\n' duplicate-slug
        fails=$((fails + 1))
    else
        printf '  ok    %-32s refused\n' duplicate-slug
    fi

    # As must a row that carries a slug and no flags. It used to be dropped, so
    # the registry silently held one lane fewer than the file listed.
    printf 'A /O1\nBroken\nC /Ox\n' > "$st/short.txt"
    cases=$((cases + 1))
    if parse_registry "$st/short.txt" "$st/short.tsv" >/dev/null 2>&1; then
        printf '  FAIL  %-32s a slug-only row was silently dropped (%s lanes from 3 rows)\n' \
            malformed-row "$(wc -l < "$st/short.tsv")"
        fails=$((fails + 1))
    else
        printf '  ok    %-32s refused (a row that does not parse is not a row that vanishes)\n' malformed-row
    fi

    # And the registry actually shipped must parse, and must carry an /EH lane —
    # the specific hole this whole registry was built to close.
    #
    # This is a deliberately WEAKER SUBSET of `crates/c2-harness/tests/
    # lane_registry.rs`, which is the binding assertion and is what `cargo test`
    # runs: that test additionally requires the /EHsc axis to be crossed over
    # EVERY base configuration, requires a lane that actually varies `/Oi`, and
    # requires the full lane count. This case is kept only so `--selftest` remains
    # self-contained on a machine with no cargo. It is a smoke check, not a second
    # definition of the rule — it cannot pass anything the test rejects, so the
    # two cannot drift in the direction that matters.
    cases=$((cases + 1))
    _n_real=$(wc -l < "$work/registry.tsv")
    _n_eh=$(cut -f2 "$work/registry.tsv" | grep -c -- '/EH' || true)
    if [ "$_n_real" -lt 2 ] || [ "$_n_eh" -lt 1 ]; then
        printf '  FAIL  %-32s %s lanes, %s of them /EH\n' shipped-registry "$_n_real" "$_n_eh"
        fails=$((fails + 1))
    else
        printf '  ok    %-32s %s lanes, %s of them compile /EH\n' shipped-registry "$_n_real" "$_n_eh"
    fi

    # ---- THE CORPUS'S SHAPE COVERAGE (lane w-shapes) --------------------------
    #
    # Every instrument above grades the corpus. Nothing asserted that the corpus
    # can still SAY anything. `scripts/sweep_shapes.py` had reported the zero rows
    # since w-modes wrote it, and a report nobody fails on is a shape that goes
    # quietly back to zero: delete a fragment, or let a generator stop emitting
    # one of its axes, and `expr_sweep.sh` keeps printing `checked=N
    # mismatches=0` over a corpus that has lost a shape. That is trap 5 exactly,
    # and every wrong-emit family found on 2026-08-04 lived in a shape no
    # instrument could represent.
    #
    # `--check` needs no toolchain and no compiler, so it belongs here. Each case
    # drives the REAL check; the two red ones fabricate a fragment directory, so
    # this proves the check FAILS when it should rather than that it exists.
    _sh_py=$(command -v python3 || true)
    _sh_run() {  # <name> <want 0|1> <frag-dir>
        cases=$((cases + 1))
        if [ -z "$_sh_py" ]; then
            printf '  FAIL  %-32s no python3 — the sweep cannot run either\n' "$1"
            fails=$((fails + 1))
            return
        fi
        _sh_rc=0
        C2RS_MAX_ZERO_MARKERS=0 "$_sh_py" "$repo_root/scripts/sweep_shapes.py" \
            --check --frag-dir "$3" > "$st/$1.out" 2>&1 || _sh_rc=$?
        [ "$_sh_rc" -ne 0 ] && _sh_rc=1
        if [ "$_sh_rc" -eq "$2" ]; then
            printf '  ok    %-32s %s\n' "$1" \
                "$(grep -m1 -E '^(check|DEGENERATE|FRAGMENT)' "$st/$1.out" || echo '(no summary line)')"
        else
            printf '  FAIL  %-32s wanted rc=%s, got rc=%s\n' "$1" "$2" "$_sh_rc"
            fails=$((fails + 1))
        fi
    }

    # The shipped corpus: no marker may be at zero. This is the assertion lane
    # w-shapes' twelve fragments exist to keep true.
    _sh_run corpus-shape-check 0 "$repo_root/scripts/sweep.d"

    # A corpus that expresses almost nothing must FAIL, not report a clean table.
    _shd="$st/frag-thin"; rm -rf "$_shd"; mkdir -p "$_shd"
    printf 'def cases(emit):\n    emit("int f(int a){return a+1;}\\n")\n' > "$_shd/10-thin.py"
    _sh_run corpus-zero-row-fails 1 "$_shd"

    # A fragment that emits NOTHING must fail even if every marker is covered by
    # the others — the observable symptom of the counter bug.
    _shd2="$st/frag-empty"; rm -rf "$_shd2"; mkdir -p "$_shd2"
    cp "$repo_root/scripts/sweep.d/"*.py "$_shd2/"
    printf 'def cases(emit):\n    pass\n' > "$_shd2/99-emits-nothing.py"
    _sh_run corpus-empty-fragment-fails 1 "$_shd2"

    # And a fragment directory with no fragments at all: "0 markers have zero
    # cases" is also what a corpus of nothing prints.
    _shd3="$st/frag-none"; rm -rf "$_shd3"; mkdir -p "$_shd3"
    _sh_run corpus-degenerate-fails 1 "$_shd3"

    # ---- THE GROUND-TRUTH OBJ READER (lanes w-llvm / w-gr) --------------------
    #
    # `scripts/gt_dump.py` is the reader every hand-measurement on this project
    # goes through, and it carried two silent defects for its whole existence:
    # `/NNN` long section names were returned literally (`crates/c2-obj`,
    # `tools/coffdump.py` and `llvm-readobj` all resolve them), and three
    # entries of its relocation table were the **i386** table's — `0x000A` named
    # SECTION where PPC means ADDR32NB, `0x0013` named SECREL where PPC means
    # SECRELLO, and `0x000C` absent altogether.
    #
    # Neither was reachable by any sweep. `/NNN` appears in **0 of 65,401** real
    # sections, and `0x000C` needs `/Z7`, which the workload's flag string does
    # not carry. So they were found by an outside reader (w-llvm) and pinned
    # here, because a defect no corpus can reach is a defect only an assertion
    # can hold closed.
    #
    # No toolchain, no LLVM: the obj is synthesised in-process.
    cases=$((cases + 1))
    if [ -z "$_sh_py" ]; then
        printf '  FAIL  %-32s no python3\n' gt-dump-selftest
        fails=$((fails + 1))
    elif "$_sh_py" "$repo_root/scripts/gt_dump.py" --selftest > "$st/gt_dump.out" 2>&1; then
        printf '  ok    %-32s %s\n' gt-dump-selftest "$(cat "$st/gt_dump.out")"
    else
        printf '  FAIL  %-32s %s\n' gt-dump-selftest "$(tail -1 "$st/gt_dump.out")"
        fails=$((fails + 1))
    fi

    # ---- THE RUN-TREE REAPER AND THE DISK RED (lane w-ledger) -----------------
    #
    # Every case below is POSITIVE: it constructs the situation the mechanism
    # exists for and shows the mechanism doing the right thing in it. "Nothing
    # broke" is not evidence, and it is the exact reasoning shape this project
    # has sixteen recorded instances of.
    #
    # The pid rules are driven against REAL processes — a real live gate (this
    # one), a real live non-gate, and a real dead pid — because the whole point
    # of the rule is what the operating system says, and a mocked `kill` would
    # only prove the mock agrees with itself.

    t_case() {  # <name> <ok?0/1> [detail]
        cases=$((cases + 1))
        if [ "$2" -eq 0 ]; then printf '  ok    %-32s %s\n' "$1" "${3:-}"
        else printf '  FAIL  %-32s %s\n' "$1" "${3:-}"; fails=$((fails + 1)); fi
    }

    # A live gate: this very shell. Its /proc cmdline says gate.sh.
    gate_pid_live "$$" && _r=0 || _r=1
    t_case pid-live-gate-is-kept "$_r" "pid $$ (this gate) reads as LIVE"

    # A live NON-gate: pid reuse, which a naive `kill -0` would keep forever.
    sleep 30 &
    _sleeper=$!
    gate_pid_live "$_sleeper" && _r=1 || _r=0
    t_case pid-reuse-is-reapable "$_r" "pid $_sleeper (a live \`sleep\`) reads as NOT a gate"

    # A real dead pid: launched, reaped, then asked about.
    (exit 0) & _dead=$!
    wait "$_dead" 2>/dev/null || true
    gate_pid_live "$_dead" && _r=1 || _r=0
    t_case pid-dead-is-reapable "$_r" "pid $_dead (exited) reads as dead"

    gate_pid_live "" && _r=1 || _r=0
    t_case pid-unattributable-is-reapable "$_r" "an empty pid is not a live gate"

    # ---- the reaper, against a fabricated tree directory -----------------------
    # Five trees, one of each kind the reaper must tell apart, plus a sixth that
    # is not a run tree at all and must survive whatever else happens.
    _rd="$st/reapdir"; rm -rf "$_rd"; mkdir -p "$_rd"
    _cur="$_rd/c2rs-gate-$$"                       # this run
    mkdir -p "$_cur"; echo "$$" > "$_cur/gate.pid"
    mkdir -p "$_rd/c2rs-gate-999000001"            # stale, oldest
    mkdir -p "$_rd/c2rs-gate-999000002"            # stale, second oldest
    mkdir -p "$_rd/c2rs-gate-999000003"            # stale, newest -> inside keep=1
    mkdir -p "$_rd/c2rs-gate-live"                 # LIVE by gate.pid, and note the
    echo "$$" > "$_rd/c2rs-gate-live/gate.pid"     #   name carries no pid at all
    mkdir -p "$_rd/c2rs-gate-$_sleeper"            # pid reuse: alive, not a gate
    mkdir -p "$_rd/not-a-gate-tree"                # must never be touched
    # `ls -dt` resolution is 1s; stamp the ages explicitly so the ordering is a
    # fact of the fixture and not of how fast the mkdirs ran.
    touch -d '2020-01-01 00:00' "$_rd/c2rs-gate-999000001"
    touch -d '2020-01-02 00:00' "$_rd/c2rs-gate-999000002"
    touch -d '2020-01-03 00:00' "$_rd/c2rs-gate-999000003"
    touch -d '2020-01-01 00:00' "$_rd/c2rs-gate-$_sleeper"

    reap_run_trees "$_rd" "$_cur" 1 "$st" > "$st/reap.out" 2>&1

    [ ! -d "$_rd/c2rs-gate-999000001" ] && [ ! -d "$_rd/c2rs-gate-999000002" ] && _r=0 || _r=1
    t_case reaper-removes-stale "$_r" "the two oldest unowned trees are gone"

    [ -d "$_rd/c2rs-gate-999000003" ] && _r=0 || _r=1
    t_case reaper-keeps-recent "$_r" "the newest finished run keeps its logs (keep=1)"

    [ -d "$_rd/c2rs-gate-live" ] && _r=0 || _r=1
    t_case reaper-keeps-live-gate "$_r" "a tree whose gate.pid is a running gate survives"

    [ -d "$_cur" ] && _r=0 || _r=1
    t_case reaper-keeps-current "$_r" "this run's own tree survives"

    [ ! -d "$_rd/c2rs-gate-$_sleeper" ] && _r=0 || _r=1
    t_case reaper-reaps-reused-pid "$_r" "alive-but-not-a-gate is reaped, not kept forever"

    [ -d "$_rd/not-a-gate-tree" ] && _r=0 || _r=1
    t_case reaper-touches-nothing-else "$_r" "a sibling that is not a run tree is untouched"

    # Three removed: the two oldest, PLUS the reused-pid tree — which is the row
    # that makes this an assertion rather than a restatement. A first draft of this
    # case expected `2 live`, counting the reused pid as an owner; the reaper was
    # right and the expectation was wrong, and the count is what caught it.
    grep -q '^reap:   3 stale run tree(s) removed, 3 kept (1 live, 1 recent, 1 this run)' "$st/reap.out" \
        && _r=0 || _r=1
    t_case reaper-reports-a-count "$_r" "$(grep -m1 '^reap:' "$st/reap.out" || echo '(no reap: line)')"

    # And with keep=0 — the aggressive setting — the live trees STILL survive.
    # This is the property the concurrency hazard turns on, so it is asserted
    # separately rather than inferred from the keep=1 case.
    reap_run_trees "$_rd" "$_cur" 0 "$st" > "$st/reap0.out" 2>&1
    [ -d "$_rd/c2rs-gate-live" ] && [ -d "$_cur" ] && [ ! -d "$_rd/c2rs-gate-999000003" ] \
        && _r=0 || _r=1
    t_case reaper-keep-0-still-spares-live "$_r" "keep=0 reaps the last finished run and no live one"

    # ---- the GREEN predicate, on its own, before anything deletes on it --------
    # It gates a deletion, so each answer is asserted separately rather than
    # inferred from the stripper's behaviour downstream.
    _gd="$st/greendir"; rm -rf "$_gd"; mkdir -p "$_gd/all-pass" "$_gd/one-fail" \
        "$_gd/no-tsv" "$_gd/empty-tsv" "$_gd/sampled"
    printf 'O1\t/O1\tPASS|265|265|129|0|\nOx\t/Ox\tPASS|265|265|125|0|\n' > "$_gd/all-pass/results.tsv"
    printf 'O1\t/O1\tPASS|265|265|129|0|\nOx\t/Ox\tFAIL|265|264|125|1|\n'  > "$_gd/one-fail/results.tsv"
    : > "$_gd/empty-tsv/results.tsv"
    printf 'O1\t/O1\tSAMPLED|400|400|129|0|\n' > "$_gd/sampled/results.tsv"
    tree_is_green "$_gd/all-pass"  && _r=0 || _r=1
    t_case green-all-pass-is-green "$_r" "every row PASS reads as green"
    tree_is_green "$_gd/sampled"   && _r=0 || _r=1
    t_case green-sampled-is-green "$_r" "SAMPLED is a no-difference verdict too"
    tree_is_green "$_gd/one-fail"  && _r=1 || _r=0
    t_case green-one-fail-is-not "$_r" "ONE FAIL among PASSes disqualifies the tree"
    tree_is_green "$_gd/no-tsv"    && _r=1 || _r=0
    t_case green-absent-tsv-is-not "$_r" "no results.tsv is NOT green — unknown never means delete"
    tree_is_green "$_gd/empty-tsv" && _r=1 || _r=0
    t_case green-empty-tsv-is-not "$_r" "an empty results.tsv is not a run that passed"

    # ---- the stripper, against green / mismatched / newest ---------------------
    # keep=3 so NOTHING is removed here: this fixture isolates the strip tier from
    # the reap tier, so a failure names one of them rather than both.
    C2RS_GATE_KEEP_CASES=1
    _sd="$st/stripdir"; rm -rf "$_sd"; mkdir -p "$_sd"
    _scur="$_sd/c2rs-gate-$$"; mkdir -p "$_scur"; echo "$$" > "$_scur/gate.pid"
    for _t in 998000001 998000002 998000003; do
        mkdir -p "$_sd/c2rs-gate-$_t/sweep/parts"
        for _c in 1 2 3; do echo 'int f(void){return 0;}' > "$_sd/c2rs-gate-$_t/sweep/case-$_c.cpp"; done
        echo 'case-1.cpp' > "$_sd/c2rs-gate-$_t/sweep/cases.txt"
        echo 0 > "$_sd/c2rs-gate-$_t/sweep/parts/mism.0"
        echo 'lane output somebody may still be reading' > "$_sd/c2rs-gate-$_t/O1.log"
    done
    printf 'O1\t/O1\tPASS|265|265|129|0|\n' > "$_sd/c2rs-gate-998000003/results.tsv"   # newest, green
    printf 'O1\t/O1\tPASS|265|265|129|0|\n' > "$_sd/c2rs-gate-998000002/results.tsv"   # green
    printf 'O1\t/O1\tFAIL|265|264|129|1|\n' > "$_sd/c2rs-gate-998000001/results.tsv"   # MISMATCH
    touch -d '2020-01-01 00:00' "$_sd/c2rs-gate-998000001"
    touch -d '2020-01-02 00:00' "$_sd/c2rs-gate-998000002"
    touch -d '2020-01-03 00:00' "$_sd/c2rs-gate-998000003"

    # A dry run must classify and remove nothing — this is the rehearsal the
    # `--reap-dry-run` flag promises, and it is worthless if it is not asserted.
    reap_run_trees "$_sd" "$_scur" 3 "$st" dry > "$st/strip-dry.out" 2>&1
    [ -f "$_sd/c2rs-gate-998000002/sweep/case-1.cpp" ] && _r=0 || _r=1
    t_case strip-dry-run-removes-nothing "$_r" "a dry run leaves every case on disk"
    grep -q 'STRIP\*  3 regenerable cases' "$st/strip-dry.out" && _r=0 || _r=1
    t_case strip-dry-run-still-counts "$_r" "and still prints the count it would have taken"

    reap_run_trees "$_sd" "$_scur" 3 "$st" > "$st/strip.out" 2>&1

    [ ! -f "$_sd/c2rs-gate-998000002/sweep/case-1.cpp" ] && _r=0 || _r=1
    t_case strip-removes-green-cases "$_r" "a green tree past KEEP_CASES loses its .cpp corpus"

    # The whole point is WHAT SURVIVES. Four kinds of evidence, asserted as one
    # case because any one of them going missing is the same defect.
    [ -f "$_sd/c2rs-gate-998000002/O1.log" ] \
        && [ -s "$_sd/c2rs-gate-998000002/results.tsv" ] \
        && [ -f "$_sd/c2rs-gate-998000002/sweep/parts/mism.0" ] \
        && [ -f "$_sd/c2rs-gate-998000002/sweep/cases.txt" ] && _r=0 || _r=1
    t_case strip-keeps-every-log "$_r" "logs, results.tsv, parts/ and cases.txt all survive the strip"

    [ -f "$_sd/c2rs-gate-998000003/sweep/case-1.cpp" ] && _r=0 || _r=1
    t_case strip-spares-newest "$_r" "the newest finished run keeps its cases (KEEP_CASES=1)"

    [ -f "$_sd/c2rs-gate-998000001/sweep/case-1.cpp" ] && _r=0 || _r=1
    t_case strip-spares-mismatch "$_r" "a tree carrying a FAIL keeps the cases its report names"

    grep -q 'cases KEPT' "$st/strip.out" && _r=0 || _r=1
    t_case strip-says-why-it-declined "$_r" "and says so, rather than silently doing nothing"

    [ -s "$_sd/c2rs-gate-998000002/sweep/CASES_STRIPPED" ] \
        && grep -q '^The 3 generated' "$_sd/c2rs-gate-998000002/sweep/CASES_STRIPPED" && _r=0 || _r=1
    t_case strip-leaves-a-note "$_r" "the note lands where the cases were and carries the count"

    grep -q '^strip:  1 kept tree(s) lost 3 regenerable cases' "$st/strip.out" && _r=0 || _r=1
    t_case strip-reports-a-count "$_r" "$(grep -m1 '^strip:' "$st/strip.out" || echo '(no strip: line)')"

    # Idempotence. A second pass must find nothing left to do and SAY zero — the
    # CASES_STRIPPED marker is what stops it re-reporting work it did yesterday.
    reap_run_trees "$_sd" "$_scur" 3 "$st" > "$st/strip2.out" 2>&1
    grep -q '^strip:  0 kept tree(s) lost 0 regenerable cases' "$st/strip2.out" && _r=0 || _r=1
    t_case strip-is-idempotent "$_r" "$(grep -m1 '^strip:' "$st/strip2.out" || echo '(no strip: line)')"

    # KEEP_CASES=0 strips every kept tree that is green — including the newest —
    # and STILL spares the mismatched one. Asserted separately because "green" and
    # "recent" are two different reasons to be spared and the knob only moves one.
    C2RS_GATE_KEEP_CASES=0
    reap_run_trees "$_sd" "$_scur" 3 "$st" > "$st/strip0.out" 2>&1
    [ ! -f "$_sd/c2rs-gate-998000003/sweep/case-1.cpp" ] \
        && [ -f "$_sd/c2rs-gate-998000001/sweep/case-1.cpp" ] && _r=0 || _r=1
    t_case strip-keep-cases-0-still-spares-mismatch "$_r" \
        "KEEP_CASES=0 takes the newest green tree and never the failed one"
    C2RS_GATE_KEEP_CASES=1

    kill "$_sleeper" 2>/dev/null || true
    wait "$_sleeper" 2>/dev/null || true

    # ---- the preflight, from BOTH sides ---------------------------------------
    # An impossible floor must refuse; a satisfiable one must pass AND print its
    # counts. A check that only ever passes is the thirteenth silent instrument.
    preflight_disk "$st" 999999999 1 > "$st/pf-space.out" 2>&1 && _r=1 || _r=0
    t_case preflight-refuses-on-space "$_r" \
        "$(grep -m1 'GATE: FAIL (DISK)' "$st/pf-space.out" || echo '(no DISK verdict)')"
    grep -q 'RESOURCE FAULT AND NOT A MISMATCH' "$st/pf-space.out" && _r=0 || _r=1
    t_case preflight-says-not-a-mismatch "$_r" "the refusal names itself as a resource fault"

    # The inode arm needs a filesystem that HAS an inode table. `--work` may sit on
    # btrfs, which reports none, so the probe dir is chosen by asking rather than
    # assumed — and if no filesystem on this box can answer, the inode rule went
    # UNTESTED and that is reported as a FAIL. An untested rule reported quietly is
    # the shape this file exists to forbid.
    _ifs=""
    for _c in "$st" /dev/shm /tmp; do
        [ -d "$_c" ] || continue
        if [ -n "$(fs_free_inodes "$_c")" ]; then _ifs="$_c"; break; fi
    done
    if [ -z "$_ifs" ]; then
        t_case preflight-refuses-on-inodes 1 "NO inode-reporting filesystem found — the inode rule is UNTESTED here"
        t_case preflight-names-the-resource 1 "untested for the same reason"
    else
        preflight_disk "$_ifs" 1 999999999 > "$st/pf-inode.out" 2>&1 && _r=1 || _r=0
        t_case preflight-refuses-on-inodes "$_r" \
            "$(grep -m1 'GATE: FAIL (DISK)' "$st/pf-inode.out" || echo '(no DISK verdict)')"
        grep -q 'out of INODES' "$st/pf-inode.out" && _r=0 || _r=1
        t_case preflight-names-the-resource "$_r" \
            "on $_ifs — space and inodes are separate; INODES bind ~7x sooner, and w-alias's red had 19 GB free"
    fi

    # btrfs reports `0 0 0` inodes. That must read as UNKNOWN, never as exhausted:
    # treating it as exhausted made an earlier draft refuse to run on /home, which
    # is where a lane escaping a full /tmp puts its --work dir.
    _btrfs=""
    for _c in "$repo_root" "$st" /home; do
        [ -d "$_c" ] || continue
        if [ -z "$(fs_free_inodes "$_c")" ]; then _btrfs="$_c"; break; fi
    done
    if [ -n "$_btrfs" ]; then
        preflight_disk "$_btrfs" 1 999999999 > "$st/pf-noino.out" 2>&1 && _r=0 || _r=1
        t_case preflight-inodeless-fs-is-not-full "$_r" \
            "$_btrfs reports no inode table; the floor is skipped, not failed"
        grep -q 'inode floor is NOT CHECKED' "$st/pf-noino.out" && _r=0 || _r=1
        t_case preflight-says-the-floor-was-skipped "$_r" "and it SAYS so — unreported is not unlimited"
    else
        t_case preflight-inodeless-fs-is-not-full 0 "(no inodeless filesystem on this box to drive it)"
        t_case preflight-says-the-floor-was-skipped 0 "(likewise)"
    fi

    preflight_disk "$st" 1 1 > "$st/pf-ok.out" 2>&1 && _r=0 || _r=1
    t_case preflight-passes-when-clear "$_r" "$(head -1 "$st/pf-ok.out")"
    grep -q '^disk:   .* free .* inodes (floors: ' "$st/pf-ok.out" && _r=0 || _r=1
    t_case preflight-prints-both-counts "$_r" "the clear path prints both numbers, not a status"

    # ---- ENOSPC told apart from a mismatch, through the real `decide` ----------
    disk_case() {  # <name> <B's LANE-RESULT or NOLINE> <want-banner 0/1>
        CASE_DIR="$st/$1"; rm -rf "$CASE_DIR"; mkdir -p "$CASE_DIR"
        printf '%s\n' "$P" > "$CASE_DIR/A.log"; echo 0 > "$CASE_DIR/A.status"
        if [ "$2" = NOLINE ]; then
            printf 'grading 197 fixtures at /O1 /EHsc\n/tmp/c2rs-gate-484396/cross/lane-results/Ox: No space left on device\n' \
                > "$CASE_DIR/B.log"
        else
            printf '%s\n/tmp/x: No space left on device\n' "$2" > "$CASE_DIR/B.log"
        fi
        echo 1 > "$CASE_DIR/B.status"
        collect "$st/reg.tsv" "$CASE_DIR" "$CASE_DIR/results.tsv"
        decide "$st/reg.tsv" "$CASE_DIR/results.tsv" "$CASE_DIR" "$SWEEP_OK" "" \
            "$CROSS_OK" "$HR_OK" "$LR_OK" > "$CASE_DIR/out.txt" 2>&1 || true
        if grep -q 'RESOURCE FAULT, NOT A MISMATCH' "$CASE_DIR/out.txt"; then _r=0; else _r=1; fi
        [ "$3" -eq 1 ] && { [ "$_r" -eq 0 ] && _r=1 || _r=0; }
        t_case "$1" "$_r" "$(grep -m1 '^GATE: ' "$CASE_DIR/out.txt" || echo '(no headline)')"
    }
    RES_DIR="$st"; RES_KB0=$(fs_free_kb "$st"); RES_IN0=$(fs_free_inodes "$st")
    RES_KBMIN="$RES_KB0"; RES_INMIN="$RES_IN0"

    disk_case enospc-no-result-says-disk NOLINE 0
    saw 'NO RESULT' 'it is still a FAIL — a resource fault does not become a pass'

    # The one that matters most: a mismatch beside an ENOSPC is STILL a mismatch.
    # Softening a wrong-emit alarm because the box was also unhappy would be a far
    # worse defect than the one this whole block fixes.
    disk_case enospc-never-softens-a-mismatch "$M" 1
    saw 'ALARM' 'a mismatch keeps its alarm even with ENOSPC in the same log'
    saw_no 'RESOURCE FAULT, NOT A MISMATCH' 'and never gets the resource banner'

    # ---- THE CALLER'S DEMAND (lane w-gate) ------------------------------------
    #
    # `--require-graded` is checked in BOTH directions and the pair is the point:
    # a test that goes red everywhere identifies nothing. Every `require-graded-*`
    # case below is shadowed by a `demand-off-*` control built from the IDENTICAL
    # fabricated logs, differing only in the flag — so a mutation that reddens the
    # demand and the control together has broken something else, and the two names
    # say which.
    #
    # `require_graded` is set INLINE before each case rather than around the group.
    # A group-scoped setting is a mode, and a case that silently inherited the
    # wrong mode is how a green from an absence gets into the instrument that
    # exists to catch greens from absences.
    #
    # Every assertion carries a DISTINCT message. A shared one cannot tell you
    # which arm fired, and this project has a recorded instance of a count floor
    # tripping first so that the assertions behind it never executed at all.

    # 1. The rung's own case: all-skip + the demand is RED, and says the count.
    SWEEP_FOR_CASE='SKIP|0|0|0|0|toolchain absent'
    CROSS_FOR_CASE='SKIP|0|0|0|0|toolchain absent'
    require_graded=1
    run_case require-graded-all-skip-fails FAIL "A=$S" "B=$S"
    require_graded=0
    saw    'GATE: FAIL (NOTHING GRADED)' 'the demand turns an all-skip run RED'
    saw    'units graded, summed over this whole gate:  0' \
           'and it fails on a COUNT of graded units, never on a status string'
    saw    'lanes that graded a corpus   0 of 2' \
           'the lanes-that-graded count is printed beside the sum'
    saw_no 'GATE: SKIPPED' 'a demanded run that graded nothing never also says SKIPPED'
    saw_no 'GATE: PASS'    'nor PASS — nothing in that output can be read as green'
    saw    'C2RS_COMPILERS' 'the resolution hint rides along, so the lane is told the fix'

    # 2. THE CONTROL. Byte-identical inputs, demand OFF: the documented behaviour
    #    is unchanged, exit 0, `GATE: SKIPPED`. This case must stay GREEN under
    #    every mutation aimed at case 1 — if it goes red too, the mutation broke
    #    the default path and the demand is not what was tested.
    SWEEP_FOR_CASE='SKIP|0|0|0|0|toolchain absent'
    CROSS_FOR_CASE='SKIP|0|0|0|0|toolchain absent'
    require_graded=0
    run_case demand-off-all-skip-still-exits-0 PASS "A=$S" "B=$S"
    saw    'GATE: SKIPPED' 'without the demand an all-skip run is UNCHANGED: SKIPPED, exit 0'
    saw_no 'NOTHING GRADED' 'and the demand banner is not printed when nobody asked for it'

    # 3. A genuinely graded run is unaffected BY the demand...
    require_graded=1
    run_case require-graded-green-run-passes PASS "A=$P" "B=$P"
    require_graded=0
    saw    '^GATE: PASS —' 'a run that graded 4 corpora passes WITH the demand set'
    saw_no 'NOTHING GRADED' 'the demand is silent when the count it wants is positive'

    # 4. ...and unaffected WITHOUT it. The second half of "either way".
    require_graded=0
    run_case demand-off-green-run-passes PASS "A=$P" "B=$P"
    saw '^GATE: PASS —' 'and the same run passes without the demand — either way, unaffected'

    # 5. SAMPLED SATISFIES THE DEMAND, deliberately. A strided 400-case sample
    #    graded 400 things; the demand is `graded > 0`, not `graded == corpus`.
    #    The instrument for "less than everything" is the qualified headline,
    #    which is still printed. Asserted because it is a DECISION, not a default.
    SWEEP_FOR_CASE='SAMPLED|400|400|14635|0|a STRIDED sample, not the corpus|397|3'
    require_graded=1
    run_case require-graded-sampled-satisfies PASS "A=$P" "B=$P"
    require_graded=0
    saw    'GATE: PASS (SAMPLED)' 'a SAMPLED run graded something, so the demand is met'
    saw_no 'NOTHING GRADED' 'the demand does not moonlight as a completeness check'

    # 6. NO DUPLICATION, part one. A partial skip is ALREADY a failure; under the
    #    demand it must still fail with the PARTIAL-SKIP message, not a second
    #    implementation of the same rule wearing the demand's banner.
    require_graded=1
    run_case require-graded-partial-skip-keeps-its-own-message FAIL "A=$P" "B=$S"
    require_graded=0
    saw    'lanes skipped while' 'a partial skip still fails as a PARTIAL SKIP'
    saw_no 'NOTHING GRADED' 'the demand did not duplicate a check that already existed'

    # 7. NO DUPLICATION, part two, and the one that matters most: a mismatch keeps
    #    its alarm and never gets relabelled. A wrong emit outranks every other
    #    piece of work, including the caller's demand.
    require_graded=1
    run_case require-graded-never-shadows-a-mismatch FAIL "A=$P" "B=$M"
    require_graded=0
    saw    'ALARM' 'a mismatch under the demand still raises the mismatch alarm'
    saw_no 'NOTHING GRADED' 'and is never relabelled as a nothing-graded run'

    # ---- the RESOLUTION HINT, and its anti-drift assertion --------------------
    #
    # w-root read a clean SKIP out of a worktree and had to derive the cause. The
    # hint is diagnostic text, so what can be asserted about it is that it NAMES
    # the right things and that those names still exist where the resolver reads
    # them.
    toolchain_hint > "$st/hint.out" 2>&1
    # `override: <NAME>`, not merely `<NAME>` anywhere in the block. The first
    # draft grepped the whole output and stayed GREEN under a mutation that
    # deleted the override from the path it belongs to, because the name was
    # still mentioned in a sentence further down. A name in prose is not a name
    # attached to the path that failed.
    _th_missing=""
    for _v in C2RS_COMPILERS C2RS_WIBO C2RS_CL_EXE C2RS_C2_DLL; do
        grep -q "override: $_v" "$st/hint.out" || _th_missing="$_th_missing $_v"
    done
    if [ -z "$_th_missing" ]; then
        _r=0; _th_detail="C2RS_COMPILERS / C2RS_WIBO / C2RS_CL_EXE / C2RS_C2_DLL"
    else
        _r=1; _th_detail="NOT NAMED BY THE HINT:$_th_missing"
    fi
    t_case hint-names-every-override "$_r" "$_th_detail"

    grep -q 'configure_existing_worktree.sh' "$st/hint.out" && _r=0 || _r=1
    t_case hint-names-the-one-command-that-fixes-it "$_r" \
        "a lane in a worktree is told the command, not just the symptom"

    grep -q 'C2RS_DC3' "$st/hint.out" && _r=1 || _r=0
    t_case hint-does-not-name-c2rs-dc3 "$_r" \
        "C2RS_DC3 is status.sh's dc3 SOURCE tree; no lane in this gate reads it"

    # THE ANTI-DRIFT ARM. The hint is a second place where these names are
    # written, and a signpost pointing at a variable nobody reads is worse than no
    # signpost. The names are read OUT OF THE HINT'S OWN OUTPUT rather than from a
    # list repeated here — a fixed list would go on agreeing with itself while the
    # hint printed something else, which is the drift, one level up.
    _th_ref="$repo_root/crates/c2-reference/src/lib.rs"
    _th_stale=""
    for _v in $(grep -o 'C2RS_[A-Z0-9_]*' "$st/hint.out" | sort -u); do
        grep -q "\"$_v\"" "$_th_ref" 2>/dev/null || _th_stale="$_th_stale $_v"
    done
    if [ -z "$_th_stale" ]; then
        _r=0; _th_detail="every C2RS_* the hint prints is still read by Toolchain::locate"
    else
        _r=1
        _th_detail="NAMED BY THE HINT, NOT READ BY crates/c2-reference/src/lib.rs:$_th_stale"
    fi
    t_case hint-overrides-still-read-by-the-resolver "$_r" "$_th_detail"

    # And the version directory must come OUT OF the Rust source, not out of a
    # literal here — the same drift, one level down.
    _th_ver=$(sed -n 's/^const X360_TOOLCHAIN_REL: &str = "\([^"]*\)";.*/\1/p' \
        "$_th_ref" 2>/dev/null | head -1)
    if [ -n "$_th_ver" ] && grep -q "$_th_ver" "$st/hint.out"; then _r=0; else _r=1; fi
    t_case hint-reads-the-version-dir-from-source "$_r" \
        "the toolchain dir printed is the one crates/c2-reference declares (${_th_ver:-UNREADABLE})"

    # BOTH SIDES OF THE `found` / `MISSING` COLUMN, driven for real. A column that
    # only ever prints one value is not a measurement, and this one has to be
    # right in the direction that matters: a path that is NOT there must say so.
    _th_saveroot="$repo_root"
    repo_root="$st/no-such-tree"
    toolchain_hint > "$st/hint-absent.out" 2>&1
    repo_root="$_th_saveroot"
    [ "$(grep -c 'MISSING' "$st/hint-absent.out")" -eq 4 ] && _r=0 || _r=1
    t_case hint-marks-absent-paths-missing "$_r" \
        "against a tree with no toolchain, all four defaults read MISSING ($(grep -c 'MISSING' "$st/hint-absent.out") of 4)"
    grep -q 'all four defaults above EXIST' "$st/hint-absent.out" && _r=1 || _r=0
    t_case hint-does-not-claim-they-exist "$_r" \
        "and it does not then print the everything-is-present note"

    # ...and the mirror: on THIS tree (whose toolchain the gate just used) every
    # default is present, so the hint must decline to blame a path and say the
    # cause is elsewhere. Without this arm the block would print four `found`
    # lines under a heading saying the toolchain did not resolve.
    if grep -q 'MISSING' "$st/hint.out"; then
        t_case hint-declines-to-blame-a-present-path 0 \
            "(this tree has a MISSING default of its own, so the mirror arm is moot here)"
    else
        grep -q 'all four defaults above EXIST' "$st/hint.out" && _r=0 || _r=1
        t_case hint-declines-to-blame-a-present-path "$_r" \
            "every default present -> the hint says the cause is elsewhere, not 'found, found, found'"
    fi


    # ---- the HATCH-RED row (board #1406) ---------------------------------------
    # Two halves, checked separately because they fail separately: the CLASSIFIER
    # (`hatch_red_verdict`, a pure function of a log) and the RULING (`decide`).
    # None of this runs the real arms — a selftest that needed a clean `crates/`
    # would refuse on exactly the trees a lane runs it from.
    HR_PASS_LOG="$st/hr-pass.log"
    printf '%s\n' \
        'hatch.py RED-TEST — board #1380, lane w-hatch' \
        '  R1 DIRTY-NOHATCH       HATCH-DIRTY              RED as expected' \
        'ALL 11 ARMS PASS — 9 red, 2 green' > "$HR_PASS_LOG"
    # A SETUP FAILED log ALSO carries a `FAILED:` line — that is the real shape,
    # and it is the trap: read in the wrong order, a tree whose hatch has drifted
    # is indistinguishable from a `hatch.py` whose guards have stopped working.
    HR_SETUP_LOG="$st/hr-setup.log"
    printf '%s\n' \
        'SETUP FAILED (2) — arm cannot run' \
        '  *** ARM FAILED ***' \
        'FAILED: R2 DIRTY+HATCH, C1 HATCH-ONLY' > "$HR_SETUP_LOG"
    HR_ARMS_LOG="$st/hr-arms.log"
    printf '%s\n' \
        '  [postcondition] the foreign edit SURVIVED: NO — IT WAS EATEN' \
        'FAILED: R1 DIRTY-NOHATCH' > "$HR_ARMS_LOG"
    HR_DIRTY_LOG="$st/hr-dirty.log"
    printf '%s\n' \
        'tree state before any arm: crates/c2-il/src/func/body/expr.rs' \
        'REFUSING to run on a dirty tree — every arm writes to crates/' > "$HR_DIRTY_LOG"
    HR_SHORT_LOG="$st/hr-short.log"
    printf '%s\n' 'ALL 9 ARMS PASS — 7 red, 2 green' > "$HR_SHORT_LOG"
    HR_VAC_LOG="$st/hr-vac.log"
    printf '%s\n' 'ALL 11 ARMS PASS — 0 red, 11 green' > "$HR_VAC_LOG"
    HR_JUNK_LOG="$st/hr-junk.log"
    printf '%s\n' 'some output that says nothing either way' > "$HR_JUNK_LOG"
    HR_EMPTY_LOG="$st/hr-empty.log"
    : > "$HR_EMPTY_LOG"

    hr_case() {  # <name> <log> <status> <expect-verdict> <expect-leading-word|-->
        _hc_got=$(hatch_red_verdict "$2" "$3" 11)
        _hc_v=$(printf '%s\n' "$_hc_got" | cut -d'|' -f1)
        _hc_d=$(printf '%s\n' "$_hc_got" | cut -d'|' -f6)
        _hc_w=$(printf '%s\n' "$_hc_d" | cut -d' ' -f1)
        if [ "$_hc_v" = "$4" ] && { [ "$5" = "--" ] || [ "$_hc_w" = "$5" ]; }; then
            t_case "$1" 0 "$_hc_v  ${_hc_d:-(no detail, as required for a clean PASS)}"
        else
            t_case "$1" 1 "wanted $4/$5, got $_hc_v/${_hc_w:-none} — $_hc_d"
        fi
    }
    hr_case hatchred-clean-log-passes        "$HR_PASS_LOG"  0 PASS      --
    hr_case hatchred-dirty-tree-refuses      "$HR_DIRTY_LOG" 1 REFUSED   DIRTY-TREE
    # THE ORDERING ARM. Its log contains `FAILED:` as well, so a classifier that
    # reads the guard-property arm first calls a drifted hatch a broken one.
    hr_case hatchred-stale-hatch-is-not-a-broken-guard "$HR_SETUP_LOG" 1 REFUSED HATCH-STALE
    hr_case hatchred-failed-arm-is-a-fail    "$HR_ARMS_LOG"  1 FAIL      ARMS-FAILED
    hr_case hatchred-short-run-is-not-a-pass "$HR_SHORT_LOG" 0 FAIL      TRUNCATED
    hr_case hatchred-no-red-arms-is-vacuous  "$HR_VAC_LOG"   0 FAIL      VACUOUS
    hr_case hatchred-green-then-nonzero-exit "$HR_PASS_LOG"  3 FAIL      EXIT
    hr_case hatchred-empty-log-is-no-result  "$HR_EMPTY_LOG" 0 NO-RESULT NO-LOG
    hr_case hatchred-junk-log-is-no-result   "$HR_JUNK_LOG"  0 NO-RESULT UNRECOGNIZED

    # TRAP B, asserted rather than hoped for: every non-PASS outcome leads with a
    # word no other outcome uses. A collapse back onto a shared prefix would let a
    # later refusal satisfy an earlier case's expectation, which has silently
    # passed two of six mutations on this project.
    _hr_words=""
    for _hr_l in "$HR_DIRTY_LOG" "$HR_SETUP_LOG" "$HR_ARMS_LOG" "$HR_SHORT_LOG" \
                 "$HR_VAC_LOG" "$HR_EMPTY_LOG" "$HR_JUNK_LOG"; do
        _hr_words="$_hr_words
$(hatch_red_verdict "$_hr_l" 0 11 | cut -d'|' -f6 | cut -d' ' -f1)"
    done
    _hr_n=$(printf '%s\n' "$_hr_words" | grep -c .)
    _hr_u=$(printf '%s\n' "$_hr_words" | grep . | sort -u | grep -c .)
    [ "$_hr_n" -eq "$_hr_u" ] && _r=0 || _r=1
    t_case hatchred-every-refusal-leads-with-its-own-word "$_r" \
        "$_hr_u distinct leading words across $_hr_n refusal shapes"

    # ---- and the RULING half, through the real `decide` ------------------------
    # `<ladder-tuple>` defaults to the green one so a hatch-red case is a
    # statement about the hatch-red row only. The mirror cases below pass it
    # explicitly and hand HR_OK in as the hatch tuple, for the same reason: two
    # absent-tuple checks in one function is exactly how a lane had one case
    # steal another's headline, and the only defence is that each case holds the
    # OTHER row fixed and green.
    hr_decide() {  # <name> <PASS|FAIL> <hatch-tuple> <grep-or--> [<ladder-tuple>]
        CASE_DIR="$st/$1"; rm -rf "$CASE_DIR"; mkdir -p "$CASE_DIR"
        printf '%s\n' "$P" > "$CASE_DIR/A.log"; echo 0 > "$CASE_DIR/A.status"
        printf '%s\n' "$P" > "$CASE_DIR/B.log"; echo 0 > "$CASE_DIR/B.status"
        collect "$st/reg.tsv" "$CASE_DIR" "$CASE_DIR/results.tsv"
        _hd_lr="$LR_OK"
        [ "$#" -ge 5 ] && _hd_lr="$5"
        _hd_got=PASS
        if ! decide "$st/reg.tsv" "$CASE_DIR/results.tsv" "$CASE_DIR" "$SWEEP_OK" "" \
                "$CROSS_OK" "$3" "$_hd_lr" > "$CASE_DIR/out.txt" 2>&1; then
            _hd_got=FAIL
        fi
        _hd_ok=1
        if [ "$_hd_got" = "$2" ]; then
            _hd_ok=0
            if [ "$4" != "--" ] && ! grep -q "$4" "$CASE_DIR/out.txt"; then _hd_ok=1; fi
        fi
        t_case "$1" "$_hd_ok" "$(grep -m1 '^GATE: ' "$CASE_DIR/out.txt" || echo '(no headline)')"
    }
    hr_decide hatchred-pass-is-an-unqualified-pass PASS "$HR_OK" '^GATE: PASS —'
    hr_decide hatchred-fail-fails-the-gate FAIL \
        'FAIL|0|11|0|0|ARMS-FAILED FAILED: R1 DIRTY-NOHATCH' 'hatch-red: ARMS-FAILED'
    hr_decide hatchred-no-result-fails-the-gate FAIL \
        'NO-RESULT|0|11|0|0|MISSING no python3, and the arms are a python script' \
        'hatch-red produced NO RESULT'
    hr_decide hatchred-residue-fails-the-gate FAIL \
        'FAIL|0|11|0|0|RESIDUE the arms left crates/ modified' 'hatch-red: RESIDUE'
    hr_decide hatchred-absent-tuple-fails-the-gate FAIL '' 'no hatch-red verdict'
    hr_decide hatchred-unknown-verdict-fails-the-gate FAIL \
        'WOBBLY|0|11|0|0|something new' 'unrecognized verdict'
    # THE ONE THAT MUST NOT REDDEN A PEER. A dirty tree exits 0 — and forfeits
    # the unqualified headline, exactly as SAMPLED and SKIPPED do.
    hr_decide hatchred-refused-exits-zero PASS \
        'REFUSED|0|11|0|0|DIRTY-TREE crates/ differs from HEAD and the arms would overwrite it: crates/c2-il/src/func/body/expr.rs' \
        'GATE: PASS (HATCH-RED REFUSED)'
    CASE_DIR="$st/hatchred-refused-exits-zero"
    saw_no '^GATE: PASS —' 'a REFUSED hatch row never prints an unqualified PASS'
    saw 'crates/c2-il/src/func/body/expr.rs' 'and it NAMES the files it refused over'

    # The 11 arms must NOT satisfy `--require-graded`. They grade instruments, not
    # objs; folding them into the unit sum would let an all-skip run on a box with
    # no toolchain answer a demand it has to fail.
    CASE_DIR="$st/hatchred-does-not-satisfy-the-demand"; rm -rf "$CASE_DIR"; mkdir -p "$CASE_DIR"
    printf '%s\n' "$S" > "$CASE_DIR/A.log"; echo 0 > "$CASE_DIR/A.status"
    printf '%s\n' "$S" > "$CASE_DIR/B.log"; echo 0 > "$CASE_DIR/B.status"
    collect "$st/reg.tsv" "$CASE_DIR" "$CASE_DIR/results.tsv"
    require_graded=1
    if decide "$st/reg.tsv" "$CASE_DIR/results.tsv" "$CASE_DIR" \
            'SKIP|0|0|0|0|toolchain absent' "" 'SKIP|0|0|0|0|toolchain absent' \
            "$HR_OK" "$LR_OK" > "$CASE_DIR/out.txt" 2>&1; then _r=1; else _r=0; fi
    require_graded=0
    t_case hatchred-does-not-satisfy-require-graded "$_r" \
        "$(grep -m1 '^GATE: ' "$CASE_DIR/out.txt" || echo '(no headline)')"
    saw 'NOTHING GRADED' 'an all-skip run with 11 green arms still graded nothing'

    # ---- the LADDER-RED row (board #1406's second half) ------------------------
    # Same two halves, driven the same way, with its own logs and its own words.
    # Nothing here runs the real arms either.
    LR_PASS_LOG="$st/lr-pass.log"
    printf '%s\n' \
        'ladder.py RED-TEST — lane w-ladders' \
        '  W1 NO-TABLE-FILE       LADDER-NOWIDTHTABLE      OK' \
        'ALL 5 ARMS PASS — 3 red, 2 green' > "$LR_PASS_LOG"
    LR_ARMS_LOG="$st/lr-arms.log"
    printf '%s\n' \
        '  => *** ARM FAILED ***' \
        'FAILED: W2 EMPTY-TABLE' > "$LR_ARMS_LOG"
    LR_SHORT_LOG="$st/lr-short.log"
    printf '%s\n' 'ALL 3 ARMS PASS — 1 red, 2 green' > "$LR_SHORT_LOG"
    LR_VAC_LOG="$st/lr-vac.log"
    printf '%s\n' 'ALL 5 ARMS PASS — 0 red, 5 green' > "$LR_VAC_LOG"
    LR_JUNK_LOG="$st/lr-junk.log"
    printf '%s\n' 'Traceback (most recent call last):' > "$LR_JUNK_LOG"
    LR_EMPTY_LOG="$st/lr-empty.log"
    : > "$LR_EMPTY_LOG"

    lr_case() {  # <name> <log> <status> <expect-verdict> <expect-leading-word|-->
        _lc_got=$(ladder_red_verdict "$2" "$3" 5)
        _lc_v=$(printf '%s\n' "$_lc_got" | cut -d'|' -f1)
        _lc_d=$(printf '%s\n' "$_lc_got" | cut -d'|' -f6)
        _lc_w=$(printf '%s\n' "$_lc_d" | cut -d' ' -f1)
        if [ "$_lc_v" = "$4" ] && { [ "$5" = "--" ] || [ "$_lc_w" = "$5" ]; }; then
            t_case "$1" 0 "$_lc_v  ${_lc_d:-(no detail, as required for a clean PASS)}"
        else
            t_case "$1" 1 "wanted $4/$5, got $_lc_v/${_lc_w:-none} — $_lc_d"
        fi
    }
    lr_case ladderred-clean-log-passes        "$LR_PASS_LOG"  0 PASS      --
    lr_case ladderred-failed-arm-is-a-fail    "$LR_ARMS_LOG"  1 FAIL      LADDER-ARMS-FAILED
    lr_case ladderred-short-run-is-not-a-pass "$LR_SHORT_LOG" 0 FAIL      LADDER-TRUNCATED
    lr_case ladderred-no-red-arms-is-vacuous  "$LR_VAC_LOG"   0 FAIL      LADDER-VACUOUS
    lr_case ladderred-green-then-nonzero-exit "$LR_PASS_LOG"  3 FAIL      LADDER-EXIT
    lr_case ladderred-empty-log-is-no-result  "$LR_EMPTY_LOG" 0 NO-RESULT LADDER-NO-LOG
    lr_case ladderred-junk-log-is-no-result   "$LR_JUNK_LOG"  0 NO-RESULT LADDER-UNRECOGNIZED

    # TRAP B again, and ACROSS the two rows this time. Both instrument rows can
    # print on the same table, so a word shared between them would let a
    # ladder-red refusal satisfy a hatch-red expectation. Asserted, not hoped.
    _lr_words=""
    for _lr_l in "$LR_ARMS_LOG" "$LR_SHORT_LOG" "$LR_VAC_LOG" "$LR_EMPTY_LOG" \
                 "$LR_JUNK_LOG"; do
        _lr_words="$_lr_words
$(ladder_red_verdict "$_lr_l" 0 5 | cut -d'|' -f6 | cut -d' ' -f1)"
    done
    for _hr_l in "$HR_DIRTY_LOG" "$HR_SETUP_LOG" "$HR_ARMS_LOG" "$HR_SHORT_LOG" \
                 "$HR_VAC_LOG" "$HR_EMPTY_LOG" "$HR_JUNK_LOG"; do
        _lr_words="$_lr_words
$(hatch_red_verdict "$_hr_l" 0 11 | cut -d'|' -f6 | cut -d' ' -f1)"
    done
    _lr_n=$(printf '%s\n' "$_lr_words" | grep -c .)
    _lr_u=$(printf '%s\n' "$_lr_words" | grep . | sort -u | grep -c .)
    [ "$_lr_n" -eq "$_lr_u" ] && _r=0 || _r=1
    t_case ladderred-shares-no-word-with-hatchred "$_r" \
        "$_lr_u distinct leading words across $_lr_n refusal shapes on BOTH rows"

    # ---- and the RULING half. Every case holds the hatch row GREEN, so the
    # headline it produces can only have come from the ladder row.
    hr_decide ladderred-pass-is-an-unqualified-pass PASS "$HR_OK" '^GATE: PASS —' "$LR_OK"
    hr_decide ladderred-fail-fails-the-gate FAIL "$HR_OK" 'ladder-red: LADDER-ARMS-FAILED' \
        'FAIL|0|5|0|0|LADDER-ARMS-FAILED FAILED: W2 EMPTY-TABLE'
    hr_decide ladderred-no-result-fails-the-gate FAIL "$HR_OK" \
        'ladder-red produced NO RESULT' \
        'NO-RESULT|0|5|0|0|LADDER-MISSING no python3, and the arms are a python script'
    hr_decide ladderred-nosubject-fails-the-gate FAIL "$HR_OK" \
        'LADDER-NOSUBJECT' \
        'NO-RESULT|0|0|0|0|LADDER-NOSUBJECT work/w-front3/ladder.py is not in this tree, so there is nothing for the arms to fire'
    hr_decide ladderred-residue-fails-the-gate FAIL "$HR_OK" 'ladder-red: LADDER-RESIDUE' \
        'FAIL|0|5|0|0|LADDER-RESIDUE the arms modified crates/, which they have no path to do: crates/c2-il/src/func/body/expr.rs'
    # THE ABSENT-TUPLE MIRROR, and the trap it is written against: the hatch row
    # is held GREEN here, so the headline cannot be the hatch row's absent check
    # standing in for this one.
    hr_decide ladderred-absent-tuple-fails-the-gate FAIL "$HR_OK" \
        'no ladder-red verdict' ''
    CASE_DIR="$st/ladderred-absent-tuple-fails-the-gate"
    saw_no 'no hatch-red verdict' 'and it is NOT the hatch row absent check wearing this case name'
    hr_decide ladderred-unknown-verdict-fails-the-gate FAIL "$HR_OK" \
        "ladder-red reported an unrecognized verdict" 'WOBBLY|0|5|0|0|something new'
    # THE ONE THAT MUST NOT REDDEN A PEER, on this row too.
    hr_decide ladderred-refused-exits-zero PASS "$HR_OK" 'GATE: PASS (LADDER-RED REFUSED)' \
        'REFUSED|0|5|0|0|LADDER-DIRTY the width table the arms read differs from HEAD: crates/c2-il/src/func/body/expr.rs'
    CASE_DIR="$st/ladderred-refused-exits-zero"
    saw_no '^GATE: PASS —' 'a REFUSED ladder row never prints an unqualified PASS'
    saw 'crates/c2-il/src/func/body/expr.rs' 'and it NAMES the file it refused over'
    # BOTH rows refusing must print BOTH suffixes. A merged one could not say
    # which row refused, and a dirty expr.rs trips the wide interlock and the
    # narrow one at the same time — so this is the ordinary case, not a corner.
    hr_decide bothrows-refused-print-both-suffixes PASS \
        'REFUSED|0|11|0|0|DIRTY-TREE crates/ differs from HEAD and the arms would overwrite it: crates/c2-il/src/func/body/expr.rs' \
        'GATE: PASS (HATCH-RED REFUSED) (LADDER-RED REFUSED)' \
        'REFUSED|0|5|0|0|LADDER-DIRTY the width table the arms read differs from HEAD: crates/c2-il/src/func/body/expr.rs'

    # And the ladder arms must not satisfy `--require-graded` either.
    CASE_DIR="$st/ladderred-does-not-satisfy-the-demand"; rm -rf "$CASE_DIR"; mkdir -p "$CASE_DIR"
    printf '%s\n' "$S" > "$CASE_DIR/A.log"; echo 0 > "$CASE_DIR/A.status"
    printf '%s\n' "$S" > "$CASE_DIR/B.log"; echo 0 > "$CASE_DIR/B.status"
    collect "$st/reg.tsv" "$CASE_DIR" "$CASE_DIR/results.tsv"
    require_graded=1
    if decide "$st/reg.tsv" "$CASE_DIR/results.tsv" "$CASE_DIR" \
            'SKIP|0|0|0|0|toolchain absent' "" 'SKIP|0|0|0|0|toolchain absent' \
            "$HR_OK" "$LR_OK" > "$CASE_DIR/out.txt" 2>&1; then _r=1; else _r=0; fi
    require_graded=0
    t_case ladderred-does-not-satisfy-require-graded "$_r" \
        "$(grep -m1 '^GATE: ' "$CASE_DIR/out.txt" || echo '(no headline)')"
    saw 'NOTHING GRADED' 'an all-skip run with 5 green ladder arms still graded nothing'

    # ---- the demand meets the modes that grade nothing -------------------------
    # Driven as REAL subprocesses: these are argument-parse decisions and the only
    # honest way to check an exit code is to produce one.
    _rgs=0
    sh "$0" --require-graded --reap-only --work "$st/never-created" \
        > "$st/rg-reap.out" 2>&1 || _rgs=$?
    [ "$_rgs" -eq 2 ] && _r=0 || _r=1
    t_case require-graded-refuses-reap-only "$_r" \
        "--require-graded --reap-only exits $_rgs (want 2 = usage, a contradiction)"
    [ ! -d "$st/never-created" ] && _r=0 || _r=1
    t_case require-graded-refuses-before-reaping "$_r" \
        "and refuses BEFORE creating a run tree or reaping anything"

    _rgs=0
    C2RS_GATE_REQUIRE_GRADED=1 sh "$0" --list --work "$st/rg-list-work" \
        > "$st/rg-list.out" 2>"$st/rg-list.err" || _rgs=$?
    [ "$_rgs" -eq 0 ] && _r=0 || _r=1
    t_case require-graded-env-leaves-list-alone "$_r" \
        "an exported demand does not break --list (exit $_rgs)"
    grep -q 'has no effect in --list' "$st/rg-list.err" && _r=0 || _r=1
    t_case require-graded-says-when-it-does-not-apply "$_r" \
        "and SAYS so on stderr — a demand quietly ignored is this file's own bug class"

    echo
    # The floor was 15 when the gate covered lanes only; the sweep row took it to
    # 27, w-modes added 3 sweep-classifier cases plus 10 mode-cross cases, and
    # w-shapes adds 4 corpus-shape cases, and w-gr adds the gt_dump reader case,
    # and w-ledger adds 18 for the reaper, the two-resource preflight and the
    # ENOSPC/mismatch discrimination, and w-gate adds 19 for `--require-graded`
    # (7 demand cases, each paired with a control, plus 8 for the resolution hint
    # and 4 for the modes that grade nothing) — 65 -> 84.
    # It is a floor on the COUNT, per the standing mitigation — compare a count,
    # never a status — and it must be raised whenever cases are added.
    #
    # **THE FLOOR IS CHECKED AFTER EVERY CASE HAS RUN, AND THAT IS DELIBERATE.**
    # `crates/c2-harness/tests/lane_registry.rs` has a count floor that trips
    # FIRST, and `docs/GAPS.md` records the consequence: every mutation failed on
    # the count and the specific assertions behind it never executed, so they
    # proved nothing. Here `fails` is accumulated by every case before the count
    # is looked at, so a mutation reddens the assertion it actually broke and the
    # per-assertion message says which one.
    # ---- THE TREE-INTEGRITY ARMS, RUN HERE TOO AND COUNTED HERE TOO ----------
    # They are in `--check` because that is the toolchain-free preflight, and
    # they are here because this is the mode whose one job is proving the gate
    # fails when it should. Running them in only one of the two would leave the
    # other's floor able to read green over the defect that makes every number
    # in this file unattributable. Each arm becomes a case, so the floor below
    # governs them as well.
    echo
    echo "tree integrity (boards #2668 / #2907):"
    if tree_integrity_arms "$st/integrity" > "$st/integrity.out" 2>&1; then _r=0; else _r=1; fi
    sed 's/^/  /' "$st/integrity.out"
    _ti_ran=$(grep -c '^  \(PASS\|FAIL\|SKIP\) ' "$st/integrity.out" 2>/dev/null || echo 0)
    t_case tree-integrity-arms-all-pass "$_r" \
        "the gate cannot report a tree it did not grade ($_ti_ran arms)"
    # ANTI-VACUITY, and it is the arm that matters: an `integrity` function that
    # printed nothing and returned 0 would satisfy the case above. The count is
    # a floor on the arms, inside the floor on the cases.
    # The floor moves with the file: 12 at w-gatefix, **15** after board #3048
    # added B1 (a declared byproduct cannot move the identity), B2 (an undeclared
    # file still must) and C4 (the pre-#3048 hash, out of git, does move).
    [ "${_ti_ran:-0}" -ge 15 ] && _r=0 || _r=1
    t_case tree-integrity-arms-not-truncated "$_r" \
        "$_ti_ran arms ran (floor 15) — a short run of them is not a pass"
    # And the counterfactual arms must have RUN, not SKIPped: they are the proof
    # that the arms above are asserting something, and a SKIP there is an
    # unavailable counterfactual, never a passing one. Four since #3048's C4.
    [ "$(grep -c '^  PASS  counterfactual-' "$st/integrity.out" 2>/dev/null || echo 0)" -ge 4 ] \
        && _r=0 || _r=1
    t_case tree-integrity-counterfactual-ran "$_r" \
        "all four pre-lane counterfactuals resolved and failed as required"

    # The floor moves with the file: 102 before board #1406's hatch-red row, 120
    # after it, **138** after `w-hatchroot` added the ladder-red row (7
    # classifier cases, 9 rulings, the cross-row word-distinctness assertion, the
    # absent-tuple mirror with its own anti-theft check, both-suffixes, and the
    # `--require-graded` pair), and **141** after `w-gatefix` added the three
    # tree-integrity cases above. A truncated selftest is the failure it exists
    # to catch, so this is a COUNT and not a "some cases ran".
    if [ "$cases" -lt 141 ]; then
        echo "gate.sh --selftest: FAIL — only $cases cases ran; the selftest itself was"
        echo "  truncated, and a truncated selftest is the failure it exists to catch."
        exit 1
    fi
    if [ "$fails" -eq 0 ]; then
        echo "gate.sh --selftest: PASS — $cases cases, the gate fails on every one that should."
        exit 0
    fi
    echo "gate.sh --selftest: FAIL — $fails of $cases checks did not behave as required."
    exit 1
fi

# --------------------------------------------------------------------------------
# run
# --------------------------------------------------------------------------------
echo
echo "lane gate: $nlanes lanes from $registry"
echo "  run dir: $work   (per-lane run dirs under $work/lanes/)"

# ================================================================================
# THE TREE PREFLIGHT — FIRST, BEFORE ANY ROW CAN WRITE INTO IT (#2668 / #2907).
#
# This runs above `hatch-red`, which is itself deliberately first, because
# `hatch-red` is the row that writes into `crates/`. The identity read here is
# therefore the identity of the tree AS THE CALLER SUPPLIED IT, not as some
# earlier row left it — which is the whole difference between a run that says
# what it graded and a run that says what it ended up with. `pin_harness`'s
# `-dirty` marker is computed after `hatch-red` and has already read `clean` for
# a run that started dirty; this line cannot.
#
# TWO THINGS HAPPEN HERE AND THEY ARE INDEPENDENT:
#
#   1. **The identity is printed.** Always, whatever the verdict, and again in
#      the epilogue after everything has run. Two hashes that agree say "the
#      tree did not move under this run"; two that differ say the run is not
#      evidence about either tree, which is a thing a gate has never been able
#      to say here before.
#
#   2. **A dirty `crates/` REFUSES THE RUN.** Not a warning, not a restore.
#      Board #2668 and #2907 are the same incident twice: the gate silently
#      reverted a lane's uncommitted work and then graded the reverted tree,
#      once costing a re-applied edit and once a retracted claim. A gate that
#      quietly reverts the thing under test is worse than one that stops, and
#      the standing rule — commit before running any gate row or mutation
#      script — is now enforced by the gate rather than remembered by the lane.
#
# `--allow-dirty-crates` is the opt-in, and it is NAMED IN THE OUTPUT every time
# it is used, on the banner and again in the epilogue. It does not re-enable the
# restore; it disables the row that would perform it (`DIRTY-ALLOWED`), so the
# promise "this run grades the tree you gave it" holds under the flag too.
#
# Exit 4, its own code: 2 is usage, 3 is the disk preflight, 1 is a real gate
# failure. A refusal to start is none of those, and a lane that reads exit
# codes must be able to tell "your tree was not gradeable" from "the port is
# wrong" — reusing 1 here would put a tooling refusal in the same bucket as a
# mismatch, which is the reading error this gate exists to prevent.
# ================================================================================
GRADED_TREE_0=$(graded_tree_hash "$repo_root")
GRADED_IGNORED_0=$(graded_ignored_count "$repo_root")
echo "  graded tree: ${GRADED_TREE_0%%:*}  (${GRADED_TREE_0##*:} files under $GRADED_DIRS, content-hashed)"
echo "               $GRADED_IGNORED_0 gitignored byproduct file(s) under those directories were NOT hashed (board #3048)"
if [ "${GRADED_TREE_0##*:}" = 0 ]; then
    echo "  *** IDENTITY UNAVAILABLE — no sha256sum, or those directories are empty."
    echo "      This run cannot say which tree it graded. That is not a mismatch."
fi
gate_dirty=$(crates_dirty "$repo_root")
if [ -n "$gate_dirty" ] && [ "$allow_dirty" -eq 0 ]; then
    echo
    echo "GATE: REFUSED (DIRTY crates/) — nothing was graded and nothing was touched."
    echo "  These files differ from HEAD:"
    for _f in $gate_dirty; do echo "    $_f"; done
    echo
    echo "  This gate has a row that writes into crates/ and restores it with"
    echo "  \`git checkout --\`. Until 2026-08-10 it ran on trees like this one and"
    echo "  the restore ate the edit: board #2668 (a lane re-applied work the gate"
    echo "  discarded) and board #2907 (a lane shipped, then retracted, a claim made"
    echo "  from a run that graded the pre-edit tree while printing the post-edit"
    echo "  commit). The row is fixed; this refusal is the part that does not depend"
    echo "  on any row staying fixed."
    echo
    echo "  Commit or stash, then re-run — a gate run is evidence about the tree it"
    echo "  graded, and uncommitted work has no name to put on that evidence."
    echo "  If you truly mean to grade an uncommitted tree, pass"
    echo "  --allow-dirty-crates: the run proceeds, says so on every headline, and"
    echo "  the row that would rewrite crates/ is refused instead of run."
    exit 4
fi
if [ -n "$gate_dirty" ]; then
    echo "  *** --allow-dirty-crates: crates/ differs from HEAD and is being graded AS IS."
    for _f in $gate_dirty; do echo "        $_f"; done
    echo "      The commit id below names a tree this run did NOT grade. The content"
    echo "      hash above is this run's only true identity."
fi

# --------------------------------------------------------------------------------
# THE HATCH-RED ROW RUNS FIRST (board #1406). Before `pin_harness`, because that
# is where the gate's one `cargo build` happens and these arms WRITE INTO
# `crates/` before restoring it — a build that started alongside them could
# compile a half-hatched tree. It needs no toolchain and no binary and takes
# seconds, so a broken hatch is also on screen before twenty minutes of compiler.
# The interlock and the postcondition are both inside `hatch_red_run`.
# --------------------------------------------------------------------------------
echo
echo "hatch-red:       work/w-hatch/hatch_red.py  (no toolchain; fires hatch.py's refusals)"
hr_res=$(hatch_red_run "$work/hatchred.log")
printf '  %s  %s\n' "$(printf '%s\n' "$hr_res" | cut -d'|' -f1)" \
    "$(printf '%s\n' "$hr_res" | cut -d'|' -f6)"
echo "ladder-red:      work/w-ladders/ladder_red.py  (no toolchain; fires ladder.py's width-table refusal)"
lr_res=$(ladder_red_run "$work/ladderred.log")
printf '  %s  %s\n' "$(printf '%s\n' "$lr_res" | cut -d'|' -f1)" \
    "$(printf '%s\n' "$lr_res" | cut -d'|' -f6)"

# Pin ONE binary for the whole gate and hand it to every lane. Stronger than each
# lane pinning its own copy: all $nlanes lanes are then provably grading the same
# code, and the sha below is the answer to "which binary produced this table".
. "$repo_root/scripts/harness_bin.sh"
pin_harness "$repo_root" "$work"
export C2RS_BIN="$C2RS_PINNED"
export C2RS_MODE_LANE_WORK="$work/lanes"
# **This one is deliberately NOT raised, and the reason is that it MULTIPLIES.**
# `C2RS_JOBS` is read only by `mode_lane.sh` (`c2rs gap --jobs`), so it is the
# per-lane thread count *inside* each of the `$jobs` lanes running at once:
# raising `jobs` 4 -> 16 already took the lane leg from 4x8 to 16x8 concurrent
# capture threads. `mode_cross.sh` never sees this value — the cross is invoked
# with `C2RS_JOBS="$jobs"` explicitly, below.
#
# And the leg it governs is not where the time is: MEASURED 2026-08-08, the lane
# leg is 16 s at `--jobs 4` and 2 s at `--jobs 16`, out of gates of
# ~300 s and ~105 s. There is nothing left in it to win, and a second knob
# raised at the same time as the first would make the next timing unattributable.
# It stays 8, and it stays overridable from the environment.
: "${C2RS_JOBS:=8}"
export C2RS_JOBS

# Clear every lane's log and status FIRST, as its own pass, so that a result file
# which EXISTS is necessarily from this run. The `>` redirection below already
# truncates the log of any lane that is actually launched, so this is not covering
# a live hole; it closes the residual one — a lane the loop never launches at all
# (an interrupted run, a future `continue`, a re-run into a `--work` directory a
# previous run used) being graded from a previous run's log. That would be a stale
# PASS indistinguishable from a real one, which is the class `harness_bin.sh` was
# written to close one level down, and it is a two-line pass to make impossible.
while IFS="$TAB" read -r slug flags; do
    [ -n "$slug" ] || continue
    rm -f "$work/$slug.log" "$work/$slug.status"
done < "$reg"

started=$(date +%s)
running=0
while IFS="$TAB" read -r slug flags; do
    [ -n "$slug" ] || continue
    (
        st=0
        # shellcheck disable=SC2086
        sh "$repo_root/scripts/mode_lane.sh" $flags > "$work/$slug.log" 2>&1 || st=$?
        echo "$st" > "$work/$slug.status"
    ) &
    running=$((running + 1))
    if [ "$running" -ge "$jobs" ]; then wait; running=0; fi
done < "$reg"
wait
res_sample                     # the lanes' peak, before their scratch is released
elapsed=$(( $(date +%s) - started ))

collect "$reg" "$work" "$work/results.tsv"
echo
echo "wall clock: ${elapsed}s for $nlanes lanes at --jobs $jobs (C2RS_JOBS=$C2RS_JOBS)"

# --------------------------------------------------------------------------------
# The generated sweep — the other half of this gate (board #232).
#
# Run AFTER the lanes so a fast lane failure is on screen first, and always: there
# is no path through this file that reaches `decide` without a sweep verdict.
# `C2RS_BIN` is already exported, so `pin_harness` inside the sweep takes the
# gate's pinned copy and the two halves provably grade one binary.
# --------------------------------------------------------------------------------
unset C2RS_SWEEP_ONLY          # a filtered corpus is not a gate; see the header
export C2RS_SWEEP_JOBS="$jobs"
sweep_out="$work/sweep"
echo
echo "generated sweep: scripts/expr_sweep.sh $sweep_out ${sweep_cases}  (jobs $jobs)"
sw_started=$(date +%s)
sw_status=0
sh "$repo_root/scripts/expr_sweep.sh" "$sweep_out" "$sweep_cases" \
    > "$work/sweep.log" 2>&1 || sw_status=$?
res_sample
tail -n 4 "$work/sweep.log" | sed 's/^/  /'
echo "  ($(( $(date +%s) - sw_started ))s)"
sweep_res=$(sweep_verdict "$work/sweep.log" "$sw_status")

# --------------------------------------------------------------------------------
# The MODE CROSS — the sweep's corpus x the lane registry (w-modes, 2026-08-04).
#
# The sweep grades 14,635 shapes at ONE profile, because `c2rs diff` hardcodes
# `/Ox /GS- /c`. The lanes grade 12 profiles over 228 hand-built shapes. w-order's
# **Y-a** was a live wrong emit that needed BOTH — an empty-bodied locally-defined
# unwind target *and* `/EHsc` — and it sat at `/O1 /EHsc`, the dc3 workload's own
# profile, where neither instrument could see it.
#
# The naive product is 175,620 gradings. `scripts/mode_invariance.py` measured how
# much of it is not redundant — 61,539 cells, 2.85x smaller — by proving, per
# fragment, which lanes hand the port a byte-identical IL bundle AND get a
# byte-identical obj back from c2 AND agree on `gy`. Those three together are the
# port's whole input and the oracle's whole output, so a merged lane is the same
# computation twice; `scripts/mode_classes.txt` is that table, every row keyed to a
# digest of its own fragment's cases so a generator edit un-applies the row rather
# than silently keeping a stale exclusion.
#
# It runs UNCONDITIONALLY, like the sweep, and for the same reason: there are
# twelve recorded instances on this project of an absence reading as a success, and
# an omittable check is an omitted check. `--cross-cells N` strides it for
# iteration and, like `--sweep-cases`, forfeits the unqualified PASS.
# --------------------------------------------------------------------------------
cross_out="$work/cross"
echo
echo "mode cross:      scripts/mode_cross.sh $cross_out ${cross_cells}  (jobs $jobs)"
cx_started=$(date +%s)
cx_status=0
C2RS_JOBS="$jobs" sh "$repo_root/scripts/mode_cross.sh" "$cross_out" "$cross_cells" \
    > "$work/cross.log" 2>&1 || cx_status=$?
grep -E '^(assigned|sweeping|checked=|SKIP:|FATAL|VACUOUS)' "$work/cross.log" \
    | sed 's/^/  /' || true
echo "  ($(( $(date +%s) - cx_started ))s)"
res_sample
cross_res=$(sweep_verdict "$work/cross.log" "$cx_status")

echo
printf 'disk:   %s low-water this run — %s and %s inodes free (start: %s / %s)\n' \
    "$work_parent" "$(human_kb "$RES_KBMIN")" "$(human_n "$RES_INMIN")" \
    "$(human_kb "$RES_KB0")" "$(human_n "$RES_IN0")"

# --------------------------------------------------------------------------------
# THE VERDICT, AND THEN THE IDENTITY OF WHAT IT IS A VERDICT ABOUT (#2668/#2907).
#
# `decide`'s status is captured rather than inherited, because the epilogue below
# has to print on EVERY path — a headline nobody can attribute to a tree is the
# defect this lane exists to close, and `GATE: FAIL` needs the attribution just
# as much as `GATE: PASS` does. `|| st=$?` and not a bare call: `set -e` is on
# and `decide` returns 1 on a real failure.
#
# The second hash is not ceremony. A peer session has modified a lane's worktree
# mid-run on this box, `hatch-red` writes into `crates/` by design, and a killed
# run leaves whatever it left. If the tree moved, the two headline numbers were
# produced by two different trees and the run is evidence about neither — which
# is a thing that can now be SAID rather than discovered later from a binary
# hash that did not match.
# --------------------------------------------------------------------------------
gate_status=0
decide "$reg" "$work/results.tsv" "$work" "$sweep_res" "$filtered" "$cross_res" "$hr_res" "$lr_res" \
    || gate_status=$?

GRADED_TREE_1=$(graded_tree_hash "$repo_root")
echo
GRADED_IGNORED_1=$(graded_ignored_count "$repo_root")
echo "graded tree: ${GRADED_TREE_1%%:*}  (${GRADED_TREE_1##*:} files: $GRADED_DIRS, content-hashed)"
echo "  $GRADED_IGNORED_1 gitignored byproduct file(s) under those directories were NOT hashed"
echo "  (board #3048; it was $GRADED_IGNORED_0 at the start of this run). The subtraction is"
echo "  printed at BOTH ends on purpose: a denominator nobody prints on both sides of a"
echo "  change is a denominator that grows unwatched (#1002). A rise here is not a failure —"
echo "  it is the gate saying which of its own byproducts it declined to grade itself on."
if [ "$GRADED_TREE_0" != "$GRADED_TREE_1" ]; then
    echo "  *** THE TREE MOVED UNDER THIS RUN — it began at ${GRADED_TREE_0%%:*}"
    echo "      (${GRADED_TREE_0##*:} files) and ended at ${GRADED_TREE_1%%:*} (${GRADED_TREE_1##*:} files)."
    echo "      The verdict above was produced partly from each, so it is evidence"
    echo "      about NEITHER tree. Find what wrote to $GRADED_DIRS during the run"
    echo "      (a gate row, a peer session, a killed process) and re-run."
    gate_status=1
fi
if [ -n "$gate_dirty" ]; then
    echo "  *** graded with --allow-dirty-crates: this is NOT the tree at"
    echo "      $(git -C "$repo_root" rev-parse --short HEAD 2>/dev/null || echo '?'), whatever any commit id printed above suggests."
fi
echo "  This is the identity of what was graded. It is a hash of file CONTENT, not"
echo "  a commit — HEAD is what lied in #2907, and a commit id is silent about"
echo "  uncommitted work, a half-restored tree and a concurrent writer alike."
exit "$gate_status"
