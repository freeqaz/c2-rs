# <TAG> — <one-line what the rung admitted>

    Tag:       <TAG or the slug, if the number is not assigned yet>
    Slug:      <slug — must equal this file's name minus the date and .md>
    Date:      <YYYY-MM-DD>
    Kind:      <fixture-claim | construct | characterization — README "Lane kinds">
    Outcome:   <converted | declined | instrument | built | FAILED — exactly one>
    Fixtures:  <wNN_slug.cpp> <wNN_slug_neg.cpp — or `none — <reason>` for kinds 2/3>
    Census:    <before> → <after> (<%> → <%>), <+delta>
    Record:    <the authoritative write-up, if the detail lives elsewhere>

## What it admits, and what it refuses

<The rule, named. The refusals by name — a class is defined by its boundary.>

## Estimate vs outcome

<The number predicted BEFORE building, the number realized, and the direction
and size of the bias. Estimates taken from a counterfactual build, not from
blocker-row sizes — `GAPS.md` §6's unstable-attribution rule.>

> **Before pricing this as codegen, run `CEILING.md` §11.4.** Three consecutive
> conversion lanes (`w-bdnz`, `w-blockir`, `w-main`) found their LAST blocker
> was a type list, a whole-obj symbol, or a clause that named the wrong layer —
> each discovered at the end of a lane that had already paid for codegen.
> `w-blockir`'s cost **one line** and no per-function byte instrument could see
> it.

## Gate evidence

| lane | result |
|---|---|
| `C2RS_REQUIRE_TOOLCHAIN=1 cargo test --workspace --release --no-fail-fast` | |
| `c2rs bench` | |
| `scripts/gate.sh --jobs 16 --require-graded` | <N>/<N> PASS, 0 FAIL, 0 SKIP, 0 NO-RESULT, <n> fixture-verdicts |
| `scripts/expr_sweep.sh` | |
| 878-TU workload scan | match / mismatch / census / disagreement |
| fixtures, `c2rs census` | positive N/N, negative 0/N |

> **Two things in the two rows above changed on 2026-08-18 (lane `w-gateperf`)
> and both are one-word edits with a measurement behind them.**
>
> * **`C2RS_REQUIRE_TOOLCHAIN=1` on the suite row.** Without it the suite is
>   *identical in every printed count* whether or not a toolchain is present —
>   `w-calleeguard` measured a provisioned and an unprovisioned run both reading
>   **1,665 / 0 / 45**, differing only in `census_gate` (79.25 s vs 0.00 s) and
>   wall clock (222 s vs 7 s). A fresh `git worktree add` has no `compilers/`,
>   so the unprovisioned run is the *easy* one to produce by accident, and it
>   looks identical and 30× faster. The variable turns that into a hard failure
>   and changes nothing when a toolchain is there. `scripts/gate.sh` exports it
>   for its own children under `--require-graded`; the suite row is hand-typed
>   and has to carry it itself.
> * **`--jobs 16 --require-graded` on the gate row.** 16 is `gate.sh`'s own
>   measured default and has been since 2026-08-08; `--jobs 4` is the untuned
>   constant that default replaced, and it takes ~1.7× as long for a
>   digit-for-digit identical verdict block (246 s vs 142 s, measured warm at
>   `w-gateperf`'s tip). `--require-graded` is what makes a run that graded
>   nothing exit 1 instead of 0.

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
