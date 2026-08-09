# w-biquad — `wbiquad_fp_store_diamond_neg.cpp`, read per cell

Produced by the probe patch in `decline_probe.md`, applied to **both**
recognizers' decline paths, run at `/O1 /GS- /c` over the fixture alone, and
reverted. The probe prints the segment length beside the key so a cell can be
identified without a one-function-per-file split.

```sh
W_BIQUAD_PROBE=1 ./target/release/c2rs gap \
    --list work/w-biquad/neglist.txt --flags-file work/w-biquad/o1flags.txt \
    --no-cache 2>&1 | grep W-BIQUAD-PROBE | sort | uniq -c
```

## The result — ten cells, ten distinct clauses, and one that is not a reader cell

| cell | one step out along | key reached |
|---|---|---|
| `A` | one pooled constant, not two | `fpdiamond-one-pool` |
| `B` | three constants in the then-arm | `fpdiamond-then-not-one-constant` |
| `C` | two different constants in the JOIN | `fpdiamond-join-not-one-constant` |
| `D` | a run of ONE division | `fpdiamond-fewer-than-2-divisions` |
| `E` | five different divisors | `fpdiamond-divisors-differ` |
| `F` | one then-store | `fpdiamond-then-fewer-than-2-stores` |
| `G` | no join stores | `fpdiamond-empty-join` |
| `H` | `!=` instead of `==` | `fpdiamond-guard-rel-not-eq` |
| `I` | `double` members | `fpdiamond-then-store` (the `float`-only literal/store gate) |
| `J` | a third formal | `fpdiamond-formals-not-this-plus-1` |
| `K` | the ctor's callee is an undefined external | **no reader key** — accepted, and declined in `c2_core::comdat` |

`K` prints no line at all from either probe, and that is the positive statement
about it: `try_parse_ctor_forward_call` **accepts** its shape. Its refusal is one
layer down, where the callee's lowering exists and M-RULE's park register can be
decided — `fnbyte-decline-gy-shape`, which this lane moved from 0 to 10 across
the whole workload.

Eleven lines are printed for ten cells: cell `A`'s appears **twice**, because
`IlBundle::functions()` short-circuits on the FIRST refusing function while
`function_census` walks all of them. Recorded rather than filed as a defect.

## The confound this probe caught, which is the reason it was run

The first draft read **seven of the eleven cells as `fpdiamond-then-close-4`** —
a refusal on a `4F` line marker, i.e. on SOURCE FORMATTING and not on the axis
any cell was written for. `Biquad.cpp` puts each closing brace on its own line
and carries a marker before each `54`; these cells write their arms on one line
and carry one marker for both closes. The recognizer had skipped a marker once
before the pair, which fits the workload TU exactly and refuses a semantically
identical body written differently.

**That is the `_neg`-cell confound six of the last nine lanes have paid for**,
caught here by *running* the probe rather than by reading the cells. The fix
moved the marker skip inside `eat_close`, `eat_label` and `eat_transfer` — where
`eat_return_head` has always had it — and the ten distinct clauses above are the
post-fix reading. The class is strictly wider for it, and no accepted body moved
(`Biquad.cpp` still `match`).
