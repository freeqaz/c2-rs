# `w-stmt5` — gate evidence at the tip

Branch `wt-w-stmt5`. Built off master `5a25656a`, **rebased onto `bf4f9a09`**
(the `w-json2` merge, itself on `w-itemf-price` at `c0f129e6`) and **re-gated
there**. Every number below is the rebased one.

Every run below is a **single writer in the foreground of one job**, with no
`crates/` edit in flight (**#3075**, **#3117**, **#3128**) — **after the first
re-gate attempt violated exactly that and was killed and discarded.** See
"The discarded run" at the foot of this file.

## `scripts/gate.sh --jobs 4 --require-graded`

```
GATE: PASS (HATCH-RED REFUSED)
lanes:  18 in the registry — 18 PASS, 0 FAIL, 0 SKIP, 0 NO-RESULT
graded: 6858 fixture-verdicts across all lanes
sweep:  PASS — 19556 of 19556 selected cases reached, 19460 GRADED, 0 mismatch
cross:  PASS — 90424 of 90812 selected cells graded, 0 mismatch (product 90812)
ladder-red  PASS  5/5 arms — 3 red, 2 green controls
hatch-red   REFUSED — HATCH-STALE (board #1389)
graded tree: 75864f22df31  (731 files) — at the HEADER and at the SUMMARY, identical
```

Script exit **0**. Per-lane: **381/381 graded on every one of the 18**,
`mismatch 0` throughout.

### `hatch-red` REFUSED is pre-existing and is NOT this lane's

`REFUSED HATCH-STALE` is the same verdict `w-readphase`, `w-deaccept`,
`w-fenceb` and `w-read2` all recorded this week. Its cause is read out of
`hatchred.log` rather than assumed:

```
HATCH-DRIFT  id=call-arg-lit-permuted   crates/c2-il/src/func/body/shapes/calls.rs
             needle matched 0 times, want 1
```

**`calls.rs` is not touched by this branch.** `git diff --name-only
5a25656a..HEAD -- crates/` lists eleven files and that is not one of them:

```
crates/c2-harness/src/gap/{classify,factors,mod,render,report,scan,tests}.rs
crates/c2-il/src/func/body/shapes/{control_flow,mod,step5}.rs
crates/c2-il/src/func/census.rs
```

### `graded tree`

**`75864f22df31` (731 files)** — printed at the **header** and at the
**summary**, identical. Master gates at `ea1afd2965f8` (730 files); this lane
adds exactly one file to the hashed set
(`crates/c2-il/src/func/body/shapes/step5.rs`), so **731 is the expected count
and the hash MUST differ** — four of the eleven changed files are inside
`crates`, which is a hashed directory.

**Byproduct, board #3048:** the run reported `0` gitignored byproducts at the
start and `1` at the summary — `scripts/__pycache__/` (a `python3` analysis
import). It is gitignored, never hashed, and **removed at this tip**;
`git status --porcelain --ignored -- crates fixtures scripts` prints **0** `!!`
lines here.

## `scripts/debug_lane.sh`

```
DEBUG-LANE-TOTAL lanes=18 ran=18 failed=0
```

`graded=381 total=381 mismatch=0 panics=0 rc=0` on every one of the eighteen.

## `cargo test --workspace --release --no-fail-fast`

**1,638 passed · 0 failed · 42 targets**, exit 0. The merged base is
**1,619 / 0 / 42** (`docs/STATUS.md`'s generated block), so the delta is
**+19** — 15 in `c2-il`'s `step5` module, 4 in `c2-harness`'s `gap::tests` —
and that is the number the PREREG registered as "a stated count, not 'a few'".

## `scripts/board_audit.sh`

Exit **0**, all-zero: cited-but-not-on-board **0** · unresolved section anchors
**0** · raw line-number anchors **0** · rows-behind-the-prose **0** · duplicate
row numbers **0**.

**The audit cannot see this lane's six rows**, because they carry letters and
not numbers (`docs/BOARD.md`'s tail block says so in those words). That is the
`#3161`–`#3164` mechanism and it is named rather than hoped away.

## `rung_registry`

**2 passed / 0 failed.** `INDEX.md` regenerated with
`scripts/gen_rung_index.sh`, never hand-edited.

## The 878-TU workload scan, at BOTH ends

Population: the **878-TU workload scan** (`c2rs gap --list
work/dc3-workload/files.txt --flags-file …/flags.txt --cwd ../dc3-decomp
--jobs 16`), denominator `fnbyte-denominator` **162,049**. Not the 381×18
fixture gate and not `c2rs perf`'s `/Ox` gate (**#3125**).

| key | base (`5a25656a`) | tip | Δ |
|---|---:|---:|---:|
| **`fnbyte-refused-parse`** | **113,612** | **113,612** | **0** |
| `fnbyte-exact` | 35,734 | 35,734 | 0 |
| **`match`** (878-TU workload) | **25** | **25** | **0** |
| `mismatch` | **0** | **0** | **0** |
| `fnbyte-refused-codegen` | 949 | 949 | 0 |
| `fnbyte-denominator` | 162,049 | 162,049 | 0 |
| `codegen-gap` · `vocab-gap` · `frontier` · `capture-fail` · `port-error` | 0 · 845 · 2 · 8 · 0 | identical | 0 |

**Identity diff: 372 → 396 `gap-metric` keys — 24 NEW, 0 GONE, 0 MOVED.**
**879 of 879 verdict lines identical** on a `src`-keyed comparison of class,
reason, `fn_in_class` and `fn_total`.

**One key MOVED on the first attempt and it was a real defect**:
`cflow-residue-inclass-offclass` 517,425 → 1,222,684, §5.3 of the rung doc.
Fixed; mutant **S5** reproduces it on demand.

## Mutation controls

**15 run, 12 RED, 3 GREEN, every colour registered in `PREREG.md` §5 before any
of them ran.** Five are graded on real IL at corpus scale. Full table in the
rung doc §6; runners are `work/w-stmt5/{mutate,scan_mutants}.sh` and both refuse
to start against a dirty `crates/` tree.


---

# The rebase

**No `crates/` conflict.** `w-json` touched
`c2-core/src/codegen/{if_call_join,json_utf8_copy,labels,reach}.rs`; this lane
touches eleven files in `c2-harness/src/gap/` and `c2-il/src/func/`. Intersection
**empty**. `docs/BOARD.md` conflicted — resolved by keeping master's blocks whole
and appending this lane's at the **bottom**. `docs/rungs/INDEX.md` was
**regenerated**, never hand-merged.

## The merged base, measured from its own `crates/`

A throwaway `git worktree` at `bf4f9a09` with its own `CARGO_TARGET_DIR`:

```
match 25 · mismatch 0 · codegen-gap 0 · vocab-gap 845 · frontier 2
capture-fail 8 · fnbyte-exact 35,734 · fnbyte-denominator 162,049
fnbyte-refused-parse 113,612 · fnbyte-refused-codegen 949 · 370 gap-metric keys
```

**Every digit of the pre-rebase base**, so neither peer merge moved the workload
numbers. Workspace tests at that base: **1,624 / 0 / 42**, measured rather than
inferred.

## This lane against the tree it will land on

| key | merged base `bf4f9a09` | tip | Δ |
|---|---:|---:|---:|
| **`fnbyte-refused-parse`** | **113,612** | **113,612** | **0** |
| `fnbyte-exact` | 35,734 | 35,734 | 0 |
| **`match`** (878-TU workload) | **25** | **25** | **0** |
| `mismatch` | **0** | **0** | **0** |
| `fnbyte-refused-codegen` | 949 | 949 | 0 |
| workspace tests | 1,624 / 0 / 42 | **1,643 / 0 / 42** | **+19** |

**Identity diff: 370 → 394 keys — 24 NEW, 0 GONE, 0 MOVED. 879 of 879 verdict
lines identical.**

**The +19 holds against two different bases** — 1,619 → 1,638 pre-rebase,
1,624 → 1,643 post-rebase. (`STATUS.md`'s generated block reads 1,619 and
`w-json`'s merge note quotes 1,629 → 1,634 for a `+5`: the documented **#3076**
offset. 1,619 + 5 = **1,624**, which is what was measured.)

**All 15 mutants re-run on the merged tree and all 15 reproduce**, including
S5's 517,425 → 1,222,684.

---

# The discarded run

**The first re-gate attempt was contaminated by this lane and was killed, not
reported.** `scripts/gate.sh` was in flight when `work/w-stmt5/mutate.sh` was
started, and that script patches `crates/`, rebuilds and reverts, ten times.
This lane's own PREREG §6 registers the single-writer rule and this lane broke
it.

**The precise exposure is the tree identity, not the grading.** `gate.sh` pins a
run-private binary at startup (**#3128**'s fix), so the rebuilds could not have
swapped the binary under a lane. What they could corrupt is `graded tree`, which
is content-hashed over `crates fixtures scripts` **at both ends** — a mutant live
at either end makes the two disagree, or agree by luck on a tree that was HEAD at
no point during grading. **A gate whose tree identity is unreliable is not
evidence, whatever its lane counts say.**

Terminated with `SIGTERM` (exit 144); log discarded; tree confirmed clean. The
`crates/`-patching mutants were then run **first, to completion**, and the gate
started only once `git status --porcelain -- crates fixtures scripts` was empty.
**The discarded run's header hash was `75864f22df31` — and so is the clean
run's, at both ends.** That is positive evidence that all ten mutants reverted
exactly, a property `mutate.sh` depends on and nothing else here checks. It does
**not** rehabilitate the discarded run: its *summary* hash was never taken, and
that is the end that would have caught a live mutant. **Nothing from it is
quoted** — not a lane count, not the sweep, not the hash.
