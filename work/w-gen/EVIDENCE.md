# w-gen — the numbers behind `docs/rungs/2026-08-08-w-gen.md`

Lane `w-gen`, branch `wt-w-gen`, off master **`03560fde`**. Read-only with
respect to `crates/` — `git diff 03560fde..HEAD --stat -- crates/` is **empty**.

## 1. The fragment alone

```text
  C2RS_SWEEP_ONLY=88-store-run-call C2RS_SWEEP_JOBS=16 scripts/expr_sweep.sh
  C2RS_SWEEP_ONLY='88-store-run-call': 1 of 61 fragments — THE TOTAL BELOW IS PARTIAL
    fragment 88-store-run-call           1576 cases
  sweeping 1576 of 1576 generated cases
  checked=1576 mismatches=0 graded=1576 ungraded=0 unknown=0
```

Profile: the sweep's own **`/Ox /GS- /c`**, which is **not** the workload's
`/GR /O1 /Oi /EHsc` (board #1112). Nothing here is a statement about the
workload profile.

`verdict_tally.txt` — the port's own answers, tallied separately because
`mismatches=0` does not distinguish *right* from *silent*:

```text
    44  Port=Match
  1532  Port=NotImplemented
  1576  ReferenceReplay=ByteExact   <- the oracle ruled on every case
```

## 2. The whole corpus, in the merge gate

```text
  61 fragments, 18286 cases total (18286 .cpp on disk)
  sweeping 18286 of 18286 generated cases   (jobs 8, 129s)
  checked=18286 mismatches=0 graded=18190 ungraded=96 unknown=0
```

Against master's **16,710 reached / 16,614 graded / 96 ungraded / 0 mismatch**:
**+1,576 reached, +1,576 graded, +0 ungraded, +0 mismatch.**

## 3. The mode cross (board #1143)

```text
  assigned 18286 cases over 18 lanes = 110273 cells (full cross would be 329148)
    3 fragment(s) have NO ROW in scripts/mode_classes.txt and are graded at ALL 18 lanes:
        12-alloc-depth
        77-reinterpret-2c
        88-store-run-call
```

Master reads `assigned 16710 cases over 18 lanes = 81905 cells`. The fail-open
default costs 1,576 × 18 = **28,368** cells against a corpus average of 4.9
lanes per case.

The cross graded **109,885 of 110,273**, against master's **81,517 of 81,905**.
Both leave **388** ungraded — the new cells add **zero** ungraded residue.

## 3b. The whole gate

```text
lanes:  18 in the registry — 18 PASS, 0 FAIL, 0 SKIP, 0 NO-RESULT
graded: 4770 fixture-verdicts across all lanes
sweep:  PASS — 18286 of 18286 selected cases reached, 18190 GRADED by the
        oracle (96 ungraded: no reference obj), 0 mismatch (corpus 18286)
cross:  PASS — 109885 of 110273 selected cells graded, 0 mismatch (product 110273)
GATE: PASS
```

## 4. The marker table (boards #1139, #1140)

`markerdelta.out`, produced by `markerdelta.py` — the corpus with and without
the fragment, through `sweep_shapes.py`'s own `markers_of`. **0 rows opened,
0 closed**; twelve already-non-zero rows move.

`pure virtual` reads **1,240** and **14** of those TUs contain the word
`virtual`; on master it read **166** and the same test leaves 14. `bitwise`
reads **1,988**, of which **1,166** look like a real operator.

## 4b. Workspace tests

**36 targets, 1,119 passed, 0 failed, 1 ignored** — the baseline exactly.
The first full run was **21 / 541 / 1 failed** (`rung_registry`: a bare
`Fixtures: none` in this lane's own rung header). `cargo test` fail-fasts across
targets, so the 36 is as load-bearing as the 1,119.

## 5. Reproducing

```sh
python3 scripts/sweep_gen.py <outdir> scripts/sweep.d          # 18,286 cases
C2RS_SWEEP_ONLY=88-store-run-call scripts/expr_sweep.sh <dir>  # 1,576, graded
python3 work/w-gen/markerdelta.py                              # the marker delta
scripts/gate.sh --jobs 8 --require-graded                      # the whole gate
```
