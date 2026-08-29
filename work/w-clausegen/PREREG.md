# PREREG — lane `w-clausegen`

**Committed BEFORE the screen is changed and before `P_INLINE.md` §6.1 is
touched.** Charter: `docs/WAVE20_BRIEF_2026-08-29.md` §2 "L1". Lane kind:
**instrument**. Branch `worktree-agent-a82c28c43fa4df59c`, dispatched at master
`c5bfe89d9`. Board `#3817`–`#3823`.

> Nothing below may be edited after this file is committed. Corrections go in
> the rung as scored outcomes, never as edits here (`docs/rungs/README.md`).

---

## 0. DISCLOSED: what was measured BEFORE this prereg, and why

Three things were run before this file was written, and it would be dishonest to
present their results as predictions:

1. **`python3 work/w-inlmetric/check_table.py`** on the dispatch tree — the
   before-measurement §3 requires.
2. **`work/w-clausegen/landscape.py`** — for each of the 24 rows, which files
   under `crates/` cite the row's `addr`. This was run first *because the whole
   instrument in §2 is unbuildable if `crates/` cites no addresses at all*, and
   a prereg that registers an unbuildable instrument is worse than one that
   admits it looked. Its output is §3's right-hand column.
3. **`git grep -c -F -- '0x10b60a1c' 72caf2586 -- crates/`** and the same for
   `0x10b625b6` — the two counts that decide whether §4's retrospective control
   is available at all.

Everything in §5 is registered *unobserved*. Anything that was already observed
is labelled **OBSERVED**, not predicted, and is not scored as a hit.

## 1. Deliverable 1 — §6.1 is GENERATED, and drift is RED

`docs/whitebox/ref/P_INLINE.md` §6.1 and `work/w-inlmetric/CLAUSES.tsv` are the
same instrument published twice. §6.1 has been hand-re-synced three times in
three days (`w-paramfill`, `w-inlclause`+coordinator, the wave-19 merge) and
`check_table.py` printed GREEN through every one of them, because it grades the
machine table and cannot see the prose copy. Board `#3814`.

**What will be built**

* `work/w-inlmetric/gen_table.py`, which renders §6.1's six-column markdown
  table **and** its two count lines from `CLAUSES.tsv`, between two HTML-comment
  markers placed in `P_INLINE.md`. `--write` regenerates in place; the default
  mode is `--check`, which diffs and exits non-zero on any divergence.
* The page path is a positional argument, so a **mutated copy** can be graded —
  that is how the control in §4 is run without touching the tracked file.
* Two **additive** `CLAUSES.tsv` columns, `wgloss` and `egloss`, carrying the
  parentheticals the page prints beside `witness` and `exercised` and the TSV
  does not currently hold (`(0x40)`, `(W-GLATTRS-1)`, `yes (F8, 6 cells)`,
  `no — /O1 pins the bit`, …). **No existing cell is edited.** Generating
  without them would silently drop published information, which is the
  `#3748` degenerate re-bless in a new costume.
* An `absent`/`unexercisable` row renders witness as `—`. This is not
  cosmetic: it is what keeps the generator from ever spelling a screened token
  into a tracked file, which is how the coordinator reddened this row twice.

**What will NOT be built**: no new `scripts/gate.sh` row (`#3691` — a 22nd
count-bearing row makes `gate_identity_diff.sh` exit 2 and refuse to diff for
all four lanes of this wave). Enforcement is a `cargo test` target in
`crates/c2-harness/tests/clause_table.rs`.

## 2. Deliverable 2 — the absence screen's FALSE-NEGATIVE half

**The hole.** `token_in_crates` asks *"is this one spelling absent from
`crates/`?"* A counterpart adopted under a **different name** answers `yes` and
the row stays `absent` forever, silently. C14 and C18 sat in that state for a
full wave. C3 and C19 converted in wave 18 only because the adopting lane
happened to choose colliding tokens. Nothing counts a false negative, which is
why the column looked stuck.

**Which instrument is being built, stated as the brief requires.** Not a
narrowing of the token screen to `crates/*/src/` — that redefines what `absent`
MEANS and was declined at the wave-19 merge. **An ADDRESS-RESOLUTION screen,
recorded as an explicit `CLAUSES.tsv` column.** Both of the brief's two named
options, because they are the same option: the column is the frozen record and
the address is what populates it.

**Why an address is a name-independent handle on adoption.** `CLAUSES.md`-side
naming is a lane's free choice; the **address is not**. `CLAUDE.md` §"Whitebox"
requires *"a row naming the address in the same commit that adopts a
disassembly-derived constant into `crates/`"*, and `PROV[R]` citation is the
convention every adopting lane in this subsystem has followed. So an adoption
under any spelling still leaves an address fingerprint.

**The check (check 6, `CITES`).** For every row, the set of files under
`crates/` matching `git grep -l --untracked --exclude-standard -F -- 0x<addr>`
is measured and compared against the row's frozen `cites` cell. **Any
difference is RED.** It is a **frozen-set differ, not a judge** — it does not
decide whether a citation is a counterpart or a mention, it decides whether the
citation footprint has *changed since a human last read it*. That is the
property the token screen lacks and the one that catches an adoption.

**Its blindness, declared in advance rather than discovered (`#3684`).**

* It inherits `#3641` exactly, transposed: it **cannot tell a counterpart from a
  mention**. `clause_table.rs`'s own doc comment cites `0x10b60a2f` and
  `0x10b5c06b`; `subsys.rs` cites `0x10b5b86d` as a band boundary. Those are
  mentions and the screen will report them.
* It is blind to an adoption that cites **no address** — `C24`'s counterpart is
  the standing example. Its sensitivity is therefore bounded by how well the
  DISCLOSURE convention is followed, and §5 registers a measurement of that
  bound rather than a claim about it.
* **Self-reference hazard**: editing `clause_table.rs`'s doc comment changes the
  measured set for C11/C12/C13/C15. This is not a defect to suppress. It is the
  instrument doing its job on the one file most likely to talk about these
  clauses, and the remedy is a reviewed cell edit in the same commit — the same
  remedy the token screen already prescribes.

**No clause is converted this lane.** Not one `state` cell moves. A row flagged
by check 6 is published as an **adjudication finding** for the wave that owns
adoption; `#3505` is six for six on lanes that moved a number by constructing
one.

## 3. BEFORE-measurement — all 24 rows, on `c5bfe89d9`

`python3 work/w-inlmetric/check_table.py` →
`CONFORMANCE-CHECK: GREEN (0 failure(s) over 24 rows)`,
`state: {'absent': 12, 'R-derived': 7, 'fitted': 2, 'unexercisable': 3}`,
`ALIGN: 424,232 instruction starts, 24 of 24 rows graded`,
`DECODE: 24 of 24`.

`TOKEN` is check 5's verdict today (`✓` = screen satisfied). `ADDR-CITED` is
what `landscape.py` measured and what check 6 will freeze.

| # | state | token screened | TOKEN verdict | files under `crates/` citing the row's `addr` |
|---|---|---|---|---|
| C1 | absent | yes | ✓ absent | — |
| C2 | absent | yes | ✓ absent | — |
| C3 | R-derived | — | ✓ witness present | `c2-core/src/splice.rs` |
| C4 | absent | yes | ✓ absent | **`c2-core/src/splice.rs`** |
| C5 | absent | yes | ✓ absent | — |
| C6 | absent | yes | ✓ absent | — |
| C7 | absent | yes | ✓ absent | — |
| C8 | fitted | — | ✓ witness present | — |
| C9 | absent | yes | ✓ absent | — |
| C10 | absent | yes | ✓ absent | **`c2-core/src/splice.rs`** |
| C11 | absent | yes | ✓ absent | **`comdat.rs`, `clause_table.rs`, `noinline_boundary.rs`, `gl.rs`** |
| C12 | absent | yes | ✓ absent | **`comdat.rs`, `clause_table.rs`, `noinline_boundary.rs`, `gl.rs`** |
| C13 | R-derived | — | ✓ witness present | `comdat.rs`, `clause_table.rs`, `noinline_boundary.rs`, `gl.rs` |
| C14 | R-derived | — | ✓ witness present | `c2-core/src/splice.rs` |
| C15 | R-derived | — | ✓ witness present | `c2-core/src/splice.rs`, `clause_table.rs` |
| C16 | absent | yes | ✓ absent | — |
| C17 | absent | yes | ✓ absent | — |
| C18 | R-derived | — | ✓ witness present | `c2-core/src/splice.rs` |
| C19 | R-derived | — | ✓ witness present | `c2-core/src/splice.rs` |
| C20 | fitted | — | ✓ witness present | — |
| C21 | unexercisable | yes | ✓ absent | — |
| C22 | unexercisable | yes | ✓ absent | — |
| C23 | unexercisable | yes | ✓ absent | **`c2-harness/src/subsys.rs`** |
| C24 | R-derived | — | ✓ witness present | **—** ← counterpart exists, cites no address |

**Fifteen rows carry a `none:` token and all fifteen pass. Zero of 24 rows are
flagged by anything today.** Five rows (C4, C10, C11, C12, C23) hold an
`absent`/`unexercisable` verdict while `crates/` cites the very address that
pins the clause, and **no instrument in this repo can currently see that.**

## 4. Controls — watched RED before any green is quoted (`#3336`, `#3787`)

Registered in advance, all four to be run and their RED output pasted into the
rung:

* **C-1 (retrospective, and it is the real one).** Run the new check 6 against
  `git show 72caf2586:work/w-inlmetric/CLAUSES.tsv` with `crates/` as it stood
  at that commit. At that tree C14 read `absent`/`none:INLINE_MAX_DEPTH` and C18
  read `absent`/`none:FORTY_INSTRS`, **both tokens genuinely absent so check 5
  was GREEN**, while `splice.rs` cited `0x10b60a1c` twice and `0x10b625b6`
  twice. This is a **real historical false negative, not a plant**. If the new
  screen does not go RED on it, deliverable 2 is FAILED and the lane says so.
* **C-2.** `gen_table.py --check` against a **mutated copy** of `P_INLINE.md`
  with one table cell altered → must go RED and name the row.
* **C-3.** `check_table.py --set C1.cites=<a path>` → check 6 RED on C1.
* **C-4.** The existing `--plant C16=10b5c06b` control must stay RED.

**A checker that has not been watched failing is decoration**, and this lane is
funded because that rule was broken three times in three days (`#3689`,
`#3787`, `#3814`).

## 5. Predictions, registered UNOBSERVED

**P1 — the retrospective control C-1 goes RED on exactly C14 and C18, and on no
other row.** p = 0.7. A RED on additional rows is not a failure but *is* a
finding about the `72caf2586` tree and will be reported.

**P2 — the generated §6.1 will NOT reproduce the current prose byte-for-byte,
and the surviving differences will be information `CLAUSES.tsv` does not carry.
Predicted 4–8 cells** (accept 2–12). A generator that reproduces the page
exactly on the first run would mean the page carries nothing the table does not,
and I do not believe that.

**P3 — the address screen's SENSITIVITY against the nine rows that already have
a counterpart (7 `R-derived` + 2 `fitted`) is 5–8 of 9.** This is the number
that says whether an address screen is worth having, and I register the range
before computing it. Below 5 of 9 and the screen is too weak to be the answer to
the false-negative hole, and the rung must say that in those words.

**P4 — `SPLIT` and `ROWS` in `clause_table.rs` DO NOT MOVE.** `absent 12 ·
R-derived 7 · fitted 2 · unexercisable 3`, 24 rows. This lane converts nothing.
If the split moves, this lane has converted a clause and violated its charter.

**P5 — byte delta zero.** `scripts/gate.sh --jobs 16 --require-graded` prints an
unqualified `GATE: PASS` and no counted quantity moves. This lane writes no
`crates/` code outside `tests/`.

**P6 — the number of rows check 6 flags on the CURRENT tree is 5 (OBSERVED, not
predicted — §0).** What is *not* observed and is registered here: **I predict
none of the five is a clean, complete, unreported adoption.** Specifically I
predict C11/C12 are address-sharing artifacts of C13, C23 is a band mention, and
that C4 and C10 will turn out to be **partial** — the port carries a
counterpart to part of the clause and the `absent` cell is over-stated rather
than wrong. p = 0.6. **Registered bias direction: OPTIMISTIC about the table.**
The repo's base rate (`#770`, ~11 optimistic / 2 pessimistic) says I am more
likely to be under-calling how stale the table is than over-calling it, so a
result where C4 or C10 is a *full* uncredited adoption is the direction I expect
to be wrong in.

## 6. Decline floor — what makes this lane FAILED

* **FAILED** if `gen_table.py --check` is not wired to a `cargo test` target
  that has been watched RED, or if check 6 has not been watched RED.
* **FAILED** if check 6 flags **zero** rows on the current tree *and* fails C-1.
  A screen that cannot fire is `#3470`, and shipping one is worse than shipping
  nothing.
* **FAILED** if any `state` cell moves, or if `SPLIT` moves — that is a
  conversion, which this lane is forbidden.
* A shortfall is reported as a shortfall in the rung's `Outcome:` line, never
  folded into a compound headline.

## 7. Stamps

* `c2-rs` dispatch tip `c5bfe89d9`; base `master`.
* `c2.dll` sha256 `c80981c0…a66258` (`C2_MAP_METHOD.md` §0).
* objdump listing present on this box → ALIGN/DECODE grade 24 of 24, not SKIP.
* No dc3 read, no compile-flag profile: this lane runs no grid.
