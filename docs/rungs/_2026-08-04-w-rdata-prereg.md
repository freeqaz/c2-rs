# PREREG — lane w-rdata, the `.rdata$r` rung

    Lane:   w-rdata (`wt-w-rdata`), branched at master `b6fa935`
    Seam:   `crates/c2-core/` only
    Target: the greedy section ladder's top step — `.rdata$r`, +421 TUs of
            factor-C reachability (C 169 -> 590)

Registered **before** the decision to implement or decline, and before any
`crates/` edit. This file is committed first so the outcome cannot be fitted to
it.

## §0 What was already measured when this was written

Registering a prediction after some measurement is worth less than registering
it before, so the honest thing is to say exactly which measurements had already
run. Four had:

1. **The census reproduces the ladder.** `work/w-bss/census/sections.jsonl`,
   871 TUs, `.XBLD$W:C1`/`:C2` normalized to `.XBLD$W` (the artefact w-gr §2.1
   labelled): factor C over the writer's ten names = **169**; over those ten
   plus `.rdata$r` = **590**, delta **+421**; `.rdata$r` appears in **676** TUs
   as **24,163** sections. Blocking names, by TUs blocked: `.rdata$r` 676,
   `.text$yd` 243, `.xdata$x` 67.
2. **Four probes captured** at `/GR /O1 /Oi /EHsc /GS- /c` and dumped
   (`work/w-rdata/probes/p1..p4`): a polymorphic class with the destructor
   defined here (13 sections), with only the constructor defined here (**11**),
   an abstract one (11), and a two-deep hierarchy (16).
3. **`c2rs census` on p1** — both function bodies out of class:
   `expr-op-0x27` (the vfptr store) and
   `expr-call-in-expr-recv-load-then-bit-and-and-branch-more` (the `??_G`
   scalar deleting destructor).
4. **`grep` of the `Section { name: … }` literals** in `crates/c2-core/src/coff/`
   — the set is exactly `PORT_WRITER_SECTIONS`'s ten today.

Nothing below is implied by those four, and P1/P3/P4 are the load-bearing ones.

## §1 Predictions

| | claim |
|---|---|
| **P1** | **Factor C will NOT reach 590 in this lane. It will read 169, unchanged.** A name in `PORT_WRITER_SECTIONS` that no writer emits inflates C by 421 with nothing behind it, and `gap.rs`'s own control (`factor_control_on_match_tus`) states in its doc comment that it **cannot** catch that direction |
| **P2** | The minimal `.rdata$r`-bearing TU has **one** `.text` COMDAT and **11** sections, and the trigger is a constructor rather than a destructor — a *virtual* destructor drags in the compiler-generated `??_G` scalar deleting destructor, a frame and a `.pdata` |
| **P3** | The blockers between here and a byte-exact `.rdata$r` obj are **exactly two facts, and both live in `crates/c2-il`** — (i) the vfptr-store leaf body class (`expr-op-0x27`), (ii) a reader for the `??_R*` record graph. Neither is in this lane's seam |
| **P4** | **`B∧C` re-measures at 151**, unchanged, *because* C does not move. If C moves, this prediction is void and the number must come from the scan |
| **P5** | Sweep never-executed lines in `crates/c2-core`: **0 before, 0 after** — `coff/` has none today and this lane adds no emission line |
| **P6** | A test that reconciles `PORT_WRITER_SECTIONS` against the `Section { name: … }` literals in the crate **passes as written today** and would go red on a name with no writer |
| **P7** | Gate, tests and gap are unchanged from the baseline: 778 passed / 26 targets; gate 18/18, 4,410 verdicts; sweep 16,394 / 16,298 / 96 / 0; cross 75,829 of 76,217 / 0; gap `match 8, mismatch 0, C 169, FRONTIER 19, capture-fail 7` |

## §2 The standing decline clause

**I decline to add `.rdata$r` to `PORT_WRITER_SECTIONS` unless some arm of
`PortC2::build` emits a `.rdata$r` obj that the differential grades byte-exact
against real `c2` on at least one case.** Every one of the ten names in that
list today is behind an emitter with a caller; `.text$yc`/`.bss`/`.CRT$XCU`
came in behind `emit_dyninit_obj`, `.data`/`.bss` behind `emit_data_obj`, and
both are reached from `build`.

**And I decline to land an uncalled writer**, on the project's own precedent:
`container::bss_deferred_layout` was exactly that — a `.bss` layout the
differential had never graded one byte of — and when a *called* path was finally
written it disagreed with reality on the walk **and** on the free list. Board
**#278** deleted it. A second uncalled writer is that mistake repeated with a
larger surface.

If both clauses fire, the deliverable is the **specification plus the price**:
the record graph measured from real objs, the section and symbol order, and a
named list of what `crates/c2-il` must grow — which is the same order
`OBJ_DATA_BSS_SHAPE.md` -> `coff/data.rs` was built in.

## §3 What would refute me

* **P1 is refuted** if a `.rdata$r` obj comes out byte-exact from this seam
  alone — which requires the two c2-il facts of P3 to turn out to be reachable
  from `IlBundle`'s existing public surface.
* **P3 is refuted** by a third blocker (or by fewer than two).
* **P2 is refuted** by a `.rdata$r` obj with no `.text` at all — which would
  make the whole rung cheap, because `emit_data_obj`'s TU class is exactly
  "defines no functions".
