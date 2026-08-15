# `w-stmt5` — gate evidence at the tip

Branch `wt-w-stmt5`, off master **`5a25656a`**. Every run below is a **single
writer in the foreground of one job**, with no `crates/` edit in flight
(**#3075**, **#3117**, **#3128**).

## `scripts/gate.sh --jobs 4 --require-graded`

```
GATE: PASS (HATCH-RED REFUSED)
lanes:  18 in the registry — 18 PASS, 0 FAIL, 0 SKIP, 0 NO-RESULT
graded: 6858 fixture-verdicts across all lanes
sweep:  PASS — 19556 of 19556 selected cases reached, 19460 GRADED, 0 mismatch
cross:  PASS — 90424 of 90812 selected cells graded, 0 mismatch (product 90812)
ladder-red  PASS  5/5 arms — 3 red, 2 green controls
hatch-red   REFUSED — HATCH-STALE (board #1389)
graded tree: 465ba5481dd9  (731 files: crates fixtures scripts, content-hashed)
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

**`465ba5481dd9` (731 files)** at the summary. Master's is `e6d4bfb38066`
(730 files); this lane adds exactly one file to the hashed set
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
