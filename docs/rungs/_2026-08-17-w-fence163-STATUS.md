# w-fence163 — lane status at wrap-up (NOT a rung doc; the rung doc is still owed)

    Lane:     w-fence163, branch `wt-w-fence163`, base `3835469c`
    Tip:      the commit carrying this file's parent (`7ea345c2` + this)
    State:    code SHIPPED on the branch and scan-verified at both ends;
              the merge-funnel checklist is INCOMPLETE (list below). The lane
              was asked to wrap up mid-verification. Outcome if graded now:
              **built** (the fence measured, grid-derived, shipped, scan-green)
              — but the funnel MUST run the missing checks before merging.

## What shipped (one commit, `crates/c2-il` only)

See `7ea345c2`'s message for the full accounting. Headline, every figure
measured in this worktree:

| column | base `3835469c` | tip | Δ |
|---|---:|---:|---:|
| fnbyte-exact | 35,734 | **35,897** | **+163** |
| fnbyte-exact-relocated | 3,822 | 3,985 | +163 |
| fnbyte-differs | 1,958 | **1,958** | **0** (identical per symbol: 0 new, 0 gone) |
| fnbyte-refused-parse | 113,612 | 113,449 | −163 |
| match / mismatch / codegen-gap | 25 / 0 / 0 | 25 / 0 / 0 | 0 |
| `data-sym-strlit-fenced:eof` (the fence's standing price) | — | **8** | all in `ContentMgr_Xbox.cpp`; **0 of the 163 held** |

Scan logs: `work/w-fence163/scan/{base,l5p,f2,f3,f4}-gap.log` (gitignored, in
the worktree). f4 is the tip configuration.

## The fence, and how it was found (for the rung doc)

1. `?ContentPath@…` = `{ return MakeString("UPDATE:"); }`; real c2 **inlines**
   the locally-defined `inline MakeString(const char*)` (14 words: 0x1070-byte
   frame + FormatString ctor + Str()); the port's un-inlined 3-word tail call
   is the wrong lowering. Mechanism: the census inline fence (clause (c)) is
   fail-open where `defined_name_set`'s walk binds 0 records — measured: 0 on
   `ContentMgr_Xbox.cpp`, and near-universally on real TUs.
2. Workload-wide, EVERY strlit body's callee is defined in its own TU
   (1,047 × `?__stl_throw_length_error@…`, 8 × `?__stl_throw_out_of_range@…`,
   1 × `?replace@String@…`, 1 × `?MakeString@…`; external-callee count **0**),
   so "callee defined here" fences everything — measured (F3) and rejected.
   F1 (walk-usable) and F2 (emit-name completeness) also held all 164 —
   measured and rejected.
3. **The discriminator is the callee's EH state.** Four-cell obj grid vs real
   c2.dll at `/O1 /Oi /EHsc /GR` (`work/w-fence163/cells/g1..g4`):
   dtor-temp+throw → call KEPT; dtor-local NO throw → call KEPT (the
   discriminating cell); no-dtor no-throw → INLINED (ContentPath reproduced);
   bare throw, no unwindable → INLINED. So: **c2's inliner keeps a call to a
   `maxState >= 1` callee; the throw itself is irrelevant.** The shipped
   clause (c2) admits a defined-here callee only if modelled (E/splice) or its
   own segment decodes `eh-state1`; anything else refuses, fail-closed.
4. Defense in depth: `IlBundle::functions()` refuses WHOLE any TU whose
   admitted body carries a `??_C@_0` data sym — the §17.2 item 7 enforcing
   line's re-imposition (the writer has no `.rdata` string COMDAT emitter).

## Prereg scoring (docs/rungs/_2026-08-17-w-fence163-prereg.md)

| id | outcome |
|---|---|
| P1 | **HIT** — base reproduced exactly (394 gap-metric lines included) |
| P2 | **HIT** — two-site widening alone: +163 / −164 / +1, the +1 is ContentPath |
| P3 | **HIT** — 25 / 0 / 0 at every rung including tip |
| P4 | **HIT** — fence holds 0 of the 163 (registered ceiling +163, realized +163, no discount) |
| P5 | **HIT** — 0 currently-right bodies refused (differs/exact/reloc columns identical per symbol) |
| P6 | **HIT** — all five w-guards tests GREEN against the tip, cell B included (contra guards §8.1's advance call of RED: cell B's refused name is not `??_C@_0`-prefixed) |
| P7 | **NOT SCORED** — full workspace suite not completed before wrap-up (c2-il 643/0 and the five guards pass; see below) |
| P8 | **PARTIAL HIT** — 0 conversions confirmed (match 25, vocab-gap 845 flat); the 67-TU count not independently re-derived |
| P9 | **NOT RUN** — identity-control revert scan skipped at wrap-up |
| P10 | **HIT** — `data-sym-strlit-fenced` prints (8) on the tip scan |
| H-A..H-D | mechanism = **H-C** (callee-side: c2 declines/performs inlining), P was 0.20 — the registered favourite H-A (0.35) MISSED |

Mutants MF1–MF5: **registered, NONE run** (wrap-up). Colours stand as
registered in the prereg for whoever runs them.

## Owed before merge (the funnel must not skip these)

1. `cargo test --workspace --release --no-fail-fast` — full run (expect
   1,648 + 0 new tests; this lane added none).
2. `scripts/gate.sh --jobs 4 --require-graded` — **load-bearing here**: the
   widening's effect on the sweep/fixture corpora was never measured by
   w-section either, and the gate-side strlit refusal (`functions()`) is what
   should keep every small-TU case at NotImplemented. Any sweep mismatch ⇒
   revert `7ea345c2` and the lane's Outcome becomes **declined**.
3. Mutants MF1–MF5 per the prereg (registered colours: all RED — note MF1/MF2
   need the fence unit tests that were also not written; without them MF5 is
   still RED via existing 26-separator tests, MF3/MF4's colour rests on tests
   this lane did not land — treat the registered colours as UNMET obligations,
   not as results).
4. **w-guards §8.1's recorded response**: add string-literal cells beside cell
   B in `crates/c2-harness/src/gap/tests.rs` and re-derive. The five existing
   guards are GREEN against this change (run at tip); nothing was weakened or
   deleted; the new cells are still owed. Suggested cells: (a) a `??_C@_0…`
   name at linkage 01 admitted by `resolve_data` (the widened behaviour,
   pinned); (b) the same body fenced when the callee is defined-here and
   `eh-none` (`DATA_SYM_STRLIT_FENCED`); (c) `??_C@_1…` (wide) still refused —
   this is mutant MF3's red-maker.
5. Unit tests in `c2-il` for clause (c2) and the `functions()` strlit refusal
   (MF1/MF2's red-makers).
6. Rung doc `docs/rungs/2026-08-17-fence163.md` (Kind: fixture-claim or
   construct — it ships behaviour; Outcome: built), `scripts/gen_rung_index.sh`,
   board rows (next-free pointer was #3218 at dispatch — VERIFY row-by-row,
   two peer lanes in flight), `work/merge-w-fence163.txt`, and the remaining
   gate evidence (debug_lane.sh, board_audit.sh, rung_registry).

## Found and not taken

1. `alias-inref-unbound` 617 → 0 under the separator admission (whole residue
   was strlit tokens) — an instrument residue closing as a side effect;
   nobody predicted it, worth a board row.
2. The strlit callee population is THREE names beyond MakeString — any future
   corpus shift in STLport versions moves the fence's yield, and the fence's
   grid (g1–g4) is the re-derivation recipe.
3. g4's finding — c2 INLINES a throwing callee when nothing is unwindable —
   corrects the folk rule "MSVC never inlines a throw" and belongs in
   `WB_INLINE_FINDINGS.md` §4.2's uncovered-axes list as now partially covered
   (obj-grid evidence, not whitebox).
4. The 7 unemitted fenced siblings in ContentMgr_Xbox.cpp are the fence's
   only reachable false-negative risk if the emit set ever widens — they are
   all MakeString callers and all would be wrong un-inlined; the fence already
   holds them.
