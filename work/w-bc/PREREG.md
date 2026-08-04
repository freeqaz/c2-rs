# w-bc — pre-registration

Tree: `c303ad0` (worktree `wt-w-bc`, branch off master at 2026-08-04).
Instrument: `c2rs gap` over `work/dc3-workload/files.txt` + `flags.txt`,
`--cwd ../dc3-decomp` (absolute at run time, never committed).

## What I am measuring

1. `B∧C` per TU at the current writer vocabulary (`C = 169`).
2. The projection `B∧C − A∧B∧C` — "what a perfect emit predicate is worth".
3. The greedy section ladder, in full.

## Registered prediction (before running on this tree)

**Disclosure, so this is not read as a blind prediction:** before writing this I
found an *uncommitted* scratch file from another lane, `work/gap-33cbdbe.txt`,
whose line 906 reads `B and C jointly … : 151` at tree `33cbdbe`. My tree is
`c303ad0`, two merges later. So this is a **confirmatory** prediction, not a
blind one, and it is registered as such. The reasoning below was derived
independently and is what I would have predicted without that file.

* **Analytic bound (checkable, not a guess).** `C = {TU : sections(TU) ⊆
  PORT_WRITER_SECTIONS}`. The writer's vocabulary only ever *grew* between the
  `C = 114` measurement and now (w-r1c's three names, then w-sect's
  `.data`/`.bss`), so `C_new ⊇ C_old` and therefore `B∧C_new ⊇ B∧C_old`.
  **`B∧C ≥ 107` is forced**, and `B∧C ≤ min(B, C) = min(338, 169) = 169`.
  So the answer lies in **[107, 169]** and the projection in **[80, 142]**.
* **Point prediction: `B∧C = 151`**, projection `151 − 27 = +124`.
  Rationale independent of the scratch file: at `C = 114`, `B∧C/C` was
  `107/114 = 0.939`; the names added since (`.data`, `.bss`, `.rdata` variants)
  admit data-heavy TUs, which are *less* likely than average to bind every
  emitted symbol, so the ratio should fall somewhat — ~0.89 × 169 ≈ 150.
* **Direction prediction: the projection went UP, not down.** Both ends moved
  (`107 → ?`, `25 → 27`), but the numerator moved much further.
* **Ladder prediction: 3 steps, reproducing `.rdata$r` → `.text$yd` →
  `.xdata$x`** with realized `C` of `590 / 804 / 871`.

## What would surprise me (i.e. refute something)

* `B∧C < 107` — that would falsify the monotonicity argument above and mean
  either `C` did not grow by vocabulary alone, or `factors()` changed meaning.
  It would make the whole "`107` cannot be extrapolated" framing *worse* than
  stated, not better.
* `B∧C = 169` — `C` entirely inside `B`, which would say the section shape is a
  strictly stronger condition than binding on this workload.
* A ladder of length ≠ 3, or a different head. The head is the one row anyone
  acts on.
* The projection going **down**. It cannot, given `B∧C ≥ 107` and
  `A∧B∧C = 27`: the worst case is `107 − 27 = +80`, which is a 2-TU drop.
  So "down" is bounded at −2 and "up" is unbounded to +142. Registering this
  because the brief asked which direction it moved, and the answer is
  *structurally* almost certain to be up — the interesting quantity is by how
  much.

## Anti-absence control

A `gap` run against a bad `--cwd` reports `capture-fail 878` / `match 0`. Every
figure below is quoted **only** from a run whose summary line is reproduced
verbatim in the rung, including its `capture-fail` count. `graded` is printed
by the factor block itself (`over N graded TUs`) and is quoted with every count.
