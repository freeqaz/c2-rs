# PREREG — lane `w-wire` (2026-08-27)

Charter: `docs/DECISIONS_2026-08-22.md` § Decision 19. Board `#3680`.
Base: `bce2bfc68`. Branch `wt-w-wire`.

Written before the first edit to the tree. **Never edited after.** Graded in
`docs/rungs/2026-08-27-w-wire.md`; a wrong prediction is called a **MISS** in
that word.

## What was already measured before this file was written

Honesty note, because a prereg whose predictions are post-hoc is not a prereg.
Two of this lane's numbers were **recounted before** this file existed, by
reading the tree — the charter told me to recount them and the recount is a
read, not an edit. They are recorded here as **MEASURED**, not predicted, and
they are not graded as predictions:

- **MEASURED** `rs-array:crates/c2-core/src/codegen/mop.rs:OPCODES` — the
  recipe **does not resolve**. `OPCODES` at `mop.rs:327` is
  `pub static OPCODES: &[OpRow] = OPCODE_ROWS;` — an alias, not an array
  literal. `array_entries` returns `None`, and `run_recipe` turns that into a
  C4 **FINDING**. The array literal is `OPCODE_ROWS` (`mop.rs:341`) and it has
  **85** entries.
- **MEASURED** `rs-consts:crates/c2-core/src/codegen/mop.rs` = **92**, not the
  91 that `w-provaudit` supplied. `provenance_census.scan_file` returns 92
  items; the extra one over the quoted 91 is `OPCODE_ROWS` itself, added by
  `w-mopfold` after the number was quoted.

Both of the bindings handed to this lane are therefore **wrong as supplied**,
one in its recipe and one in its value. That is the case the charter warned
about and it is why it said to recount.

## Predictions — genuinely forward, graded at the end

**P1 — the `gate.sh` row: NO, and the condition is stated now.**
I will add **no** `gate.sh` row. The condition under which I would reverse:
only if the `cargo test` wiring proved impossible, which it has not.

The reason is mechanical rather than a judgement call, and I predict the
mechanism will hold when I check it end-to-end: `gate_identity_diff.sh`'s
`rows()` selects **by shape** —
`awk '/^[A-Za-z][A-Za-z0-9-]* +(PASS|FAIL|REFUSED|SKIP|NO-RESULT) /'` — and
excludes **by hard-coded name**, `grep -Ev '^(hatch-red|ladder-red) '`. So a
new row named anything else is counted **even if its mismatch column is
`n/a`**, `WANT_ROWS=21` is violated, and `checked_rows` exits **2** with
*"yielded 22 count-bearing rows, expected 21"*. That is not a changed
denominator that lanes could absorb — it is a **refusal to diff at all**, for
every live lane holding a 21-row base table, on a tree they did not touch.
Repairing it means editing `gate_identity_diff.sh` (not in my fence) and
re-basing every live lane's base table.

`#1406` is discharged by the `cargo test` half on its own words —
*"must run under `cargo test` **or** `scripts/gate.sh`"*. The charter says
*"if in doubt, do NOT add the row"*, and I am not even in doubt.

- **P1a (forward)**: I predict a synthetic 22-row table will make
  `gate_identity_diff.sh` print `expected 21` and exit 2. I will demonstrate
  this rather than assert it.

**P2 — scope of the wiring: `--self-test` only, not the tree audits.**
I will wire `scripts/provenance_census.py --self-test` and
`scripts/prose_audit.py --self-test`, exactly as the charter specifies, and
**not** the tree runs. Stated in advance so it is not a retreat later.

Reasoning, both sides counted: the tree runs are cheap (measured 0.66 s and
1.22 s) and both are **green at base** (census exit 0 over 999 items; prose
`VERDICT: CLEAN over 574 checked claims`), and `tracked_artifact_audit.rs` is
a same-repo precedent that gates **both** the tree run and the self-test. But
a tree audit under `cargo test` makes **every lane's** `cargo test` depend on
**every doc in the tree** being count-clean, and there are 20 live worktrees.
The charter is explicit about scope and about not disrupting peers.

- **P2a (forward, and it is the uncomfortable one)**: I predict this leaves
  deliverable 2's bindings **automatically ungraded** — nothing in `cargo
  test` or `gate.sh` will run the *tree* audit that checks them. I will say so
  in the report as a named follow-up rather than let "so they are graded" read
  as achieved. If at the end I judge that this hollows out deliverable 2, the
  honest outcome word is still `built`, with the gap named.

**P3 — `575` and `34 of 104`: I predict NOT bindable.**
`run_recipe`'s whole vocabulary is `ledger-rows`, `md-rows`, `grep`,
`rs-consts`, `rs-marks`, `rs-array`. Every one is a **count of matching lines
or literal entries**. `575` is a *difference* (660 − 85) and `34` / `104` are
*distinct-value* counts requiring deduplication over a column. No recipe does
subtraction and none does `uniq`.

- **P3a (forward)**: I predict **660** *is* bindable, via
  `grep:docs/whitebox/ref/ENCODE_OPCODES.txt:^0x` — the file is 661 lines with
  one `#` header. I have not run this yet. If it returns 660 I will say so; I
  will **not** bind it, because `mop.rs`'s prose says "c2's 660 rows" inside a
  sentence whose subject is 85, and binding a number I was not asked to bind
  is scope I was told not to take.

**P4 — the two bindings I will actually write.** Unbackticked, in `mop.rs`
comments only:
- `rs-array:…:OPCODE_ROWS = 85` — recipe **corrected** from `OPCODES`.
- `rs-consts:…mop.rs = 92` — value **corrected** from 91.
- **P4a (forward)**: I predict C4b (the DETACHED check) will force me to write
  a *new* prose sentence stating 92, because no prose in `mop.rs` says 92
  today. I will write one true sentence and flag in the report that this
  binding's prose was **manufactured to carry it**, whereas the 85 binding
  grades a claim that was already evidentiary (it sits inside `W-MOP-2`'s
  `PROV[R]`).
- **P4b (forward)**: I predict `prose_audit.py` on the tree goes from
  `574 checked claims / 0 findings` to `576 / 0` after both bindings land
  (checked +2). Bias if wrong: I expect the *checked* count to rise by exactly
  2 and `VERDICT: CLEAN` to hold.

**P5 — gate and identity diff.**
- No base gate table in `work/coordinator/gatebase/` matches this base: all
  eight differ from `bce2bfc68` over `crates fixtures scripts` (146–165 files).
  So I run my **own** base end, in this worktree at `bce2bfc68` with a clean
  tree, before the first edit.
- **P5a (forward)**: `GATE: PASS` at both ends, and
  `IDENTITY DIFF: 0 lines over 21 rows`. My `crates/` changes are one new
  `tests/` file and comment-only lines in `mop.rs`, so a nonzero diff would
  mean a comment changed an emit.

**P6 — test and target counts.**
- **P6a (forward)**: `cargo test --workspace` gains exactly **one target**
  (`crates/c2-harness/tests/provenance.rs` → target `provenance`) and exactly
  **two tests** (one per script). Bias if wrong: I expect no other count to
  move.

**P7 — watching it fail.**
- **P7a (forward)**: breaking each self-test in turn will produce **two
  distinct** failure messages, each naming its own script, because each
  assertion embeds that script's own report. If both reddened with the same
  text the test would identify nothing and I would rewrite it.

## Decline floor

I report **FAILED**, in that word, and do not paper over, if any of:

- the identity diff is anything other than 0 lines over 21 rows;
- either gate end is not `GATE: PASS` (read from the `GATE:` line, never the
  exit code);
- the two self-test breaks do not produce two distinct, script-naming messages;
- a binding I write does not recount, i.e. `prose_audit.py` goes red at my tip;
- the `python3`-absent path panics or fails instead of printing SKIP and
  passing.

## Out of scope, deliberately

- Qualifying the 30 unattributed `W-*-N` citations across 4 tokens. Reported as
  a list only; the charter says converting them is a separate job.
- Renaming anything. Decision 19 forbids it.
- Any edit to `scripts/provenance_census.py` or `scripts/prose_audit.py`. If
  one needs a change I stop and report — including the case where
  `run_recipe` has no vocabulary for `575`, which is P3 and which I will
  **report, not fix**.
