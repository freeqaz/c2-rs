# Item 4 — the docs-only re-gate skip, PRICED AND DECLINED

`work/fable-perf/PROPOSAL.md` §4 proposes skipping the merge cycle when the
merge's diff is confined to `docs/`. The derivation there is careful and its
surprise is real — `work/` **is** in the graded closure, via `include_str!` in
`crates/c2-harness/tests/dead_temp_elision.rs` and its siblings. What was never
measured is **how often the rule would fire**.

## Method

For each of the last 40 merge commits `M` on `master` (as of `f49fe5e1`), the
diff the merge brings in is `git diff --name-only M^1 M` — the same expression
§4's rule uses (`<gated-base>..<merged>`). Two rules were evaluated per merge:

* **strict** (the proposal's): every changed path is under `docs/`.
* **narrow closure**: every changed path is under `docs/`, **or** under `work/`
  and *not* one of the paths an `include_str!` in `crates/` actually names.
  The 31 named paths were re-derived from the tree, not from memory:

      grep -rho 'include_str!("\.\./\.\./\.\./work/[^"]*")' crates/

  → `work/w-inl0/cells/m01..m08.cpp`, `work/w-memset/cells/l01..l12.cpp`,
  `work/w-seed/cells/n01..n11.cpp`. Every other `work/` mention inside
  `crates/**/*.rs` is **prose in a doc comment** (checked: 20 hits, all `///`
  or `//!`), and no non-test crate reads `work/` at runtime.

## Result

| rule | fires on |
|---|---:|
| strict `docs/`-only | **0 of 40** |
| narrow (`docs/` + non-`include_str!` `work/`) | 7 of 40 |

Not one of the last 40 merges changed only `docs/`. The closest are
`52ab8c3e` (1 file, and it is not under `docs/`) and `59b6e3d2`
(2 files: 1 `docs/`, 1 `work/`). The median merge changes 50 files of which
~90 % are under `work/` — every lane commits its evidence there, which is the
convention the project runs on.

The 7 the narrow rule would reach:

    fea2daea  merge w-front3
    59b6e3d2  merge w-mrslot (eleventh commit)
    9afc0599  merge w-heap (tail)
    25bd166d  merge w-front2
    fe114e0e  merge w-rdata3
    f57fe61e  merge w-root
    ddab417c  merge w-quar

## Decision: DO NOT BUILD, either version

* The **strict** rule is dead on arrival at 0/40. A skip mechanism that never
  fires is pure risk surface: it is code nobody exercises, guarding a decision
  nobody takes, in the one place where a wrong answer is a *silent* green.
* The **narrow** rule fires 7/40 (17.5 %) and is where the temptation is. It is
  declined anyway, and the reason is not its fire rate:
  * Its closure is a **derived allowlist**, and the proposal's own false-green
    analysis says it must be re-derived at use time rather than remembered.
    Re-deriving it means a `grep` over `crates/` whose *negative* result is what
    licenses the skip — "no `include_str!` names this path" is an absence, and
    absence read as success is this project's characteristic defect (STATUS.md
    trap 5, twelve recorded instances, and the sixteenth instrument found the
    same shape in 2026-08-08's `#1148`).
  * The closure is **wider than `include_str!` and the rule cannot see the
    difference**. `scripts/regen_census.sh` reads `work/w-bss/census/sections.jsonl`
    and `work/w-bss2/*.py`; `scripts/gt_store_sched.sh` writes under `work/`.
    None of those is in the graded path *today*, which is exactly the sentence
    a future lane's change makes false without touching the skip script.
  * The saving is bounded by 7 merges × one cycle. At the new gate default
    (item 1) plus the `cli_flags` split (item 2) the cycle is ~5 min, so the
    narrow rule is worth ~35 min **spread over 40 merges** — against a
    permanent false-green surface in the merge ritual itself.

The measurement, not the argument, is the deliverable: **0 of 40**.
