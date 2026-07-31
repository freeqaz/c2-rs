# <TAG> — <one-line what the rung admitted>

    Tag:       <TAG or the slug, if the number is not assigned yet>
    Slug:      <slug — must equal this file's name minus the date and .md>
    Date:      <YYYY-MM-DD>
    Fixtures:  <wNN_slug.cpp> <wNN_slug_neg.cpp>
    Census:    <before> → <after> (<%> → <%>), <+delta>
    Record:    <the authoritative write-up, if the detail lives elsewhere>

## What it admits, and what it refuses

<The rule, named. The refusals by name — a class is defined by its boundary.>

## Estimate vs outcome

<The number predicted BEFORE building, the number realized, and the direction
and size of the bias. Estimates taken from a counterfactual build, not from
blocker-row sizes — `GAPS.md` §6's unstable-attribution rule.>

## Gate evidence

| lane | result |
|---|---|
| `cargo test --workspace --release` | |
| `c2rs bench` | |
| `scripts/gate.sh --jobs 4` | <N>/<N> PASS, 0 FAIL, 0 SKIP, 0 NO-RESULT, <n> fixture-verdicts |
| `scripts/expr_sweep.sh` | |
| 878-TU workload scan | match / mismatch / census / disagreement |
| fixtures, `c2rs census` | positive N/N, negative 0/N |

> **Record the gate as `scripts/gate.sh`, never as a hand-typed list of modes.**
> This row used to read ``scripts/mode_lane.sh`` `/Ox` / `/O1` / `/O2` /
> `/Ox /Gy`, and every rung doc copied from this template recorded exactly those
> four — which is how a mode list that compiles **no `/EH` at any invocation**
> became the standing gate on a workload whose every TU is built `/EHsc`. The
> lane list is data now (`scripts/lanes.txt`), and quoting the gate's own summary
> line keeps a rung's evidence correct when a lane is added. Quote the counts
> the gate prints: a rung that records `12/12 PASS` and `0` graded has recorded
> nothing.

## Found and not taken

<Ranked, sized, with the frame axis applied. This is the section the next rung
reads first.>
