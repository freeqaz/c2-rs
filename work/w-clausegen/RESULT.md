# RESULT — lane `w-clausegen`

Against `work/w-clausegen/PREREG.md`. Tree `c5bfe89d9` + this lane's commits.
Board `#3817`–`#3823`. **Outcome: `instrument`.**

---

## 1. The verdict change over all 24 rows, before and after

The brief requires both published, and `#3470` is why: a screen "fixed" into
never firing again looks identical to a screen that has nothing to report.

`before` = the five checks on `c5bfe89d9`. `after` = the same five plus check 6.
`Δ` is the row's *flag* status, not its `state` — **no `state` cell moved.**

| # | state | before | after | Δ | adjudication |
|---|---|---|---|---|---|
| C1 | absent | ✓ | ✓ | — | |
| C2 | absent | ✓ | ✓ | — | |
| C3 | R-derived | ✓ | ✓ | — | address cited by its own adoption |
| C4 | absent | ✓ | **FLAG** | **NEW** | **partial counterpart, address-cited** — §3 |
| C5 | absent | ✓ | ✓ | — | address shared with C6; cannot discriminate |
| C6 | absent | ✓ | ✓ | — | address shared with C5; cannot discriminate |
| C7 | absent | ✓ | ✓ | — | |
| C8 | fitted | ✓ | ✓ | — | counterpart exists and cites NO address — §4 |
| C9 | absent | ✓ | ✓ | — | |
| C10 | absent | ✓ | **FLAG** | **NEW** | **partial counterpart, address-cited ×4** — §3 |
| C11 | absent | ✓ | **FLAG** | **NEW** | address-sharing artifact of C13 — §3 |
| C12 | absent | ✓ | **FLAG** | **NEW** | address-sharing artifact of C13 — §3 |
| C13 | R-derived | ✓ | ✓ | — | address cited by its own adoption |
| C14 | R-derived | ✓ | ✓ | — | **the row check 6 was built for**; §2 |
| C15 | R-derived | ✓ | ✓ | — | address cited by its own adoption |
| C16 | absent | ✓ | ✓ | — | |
| C17 | absent | ✓ | ✓ | — | |
| C18 | R-derived | ✓ | ✓ | — | **the row check 6 was built for**; §2 |
| C19 | R-derived | ✓ | ✓ | — | address cited by its own adoption |
| C20 | fitted | ✓ | ✓ | — | counterpart exists and cites NO address — §4 |
| C21 | unexercisable | ✓ | ✓ | — | |
| C22 | unexercisable | ✓ | ✓ | — | |
| C23 | unexercisable | ✓ | **FLAG** | **NEW** | band-boundary mention — §3 |
| C24 | R-derived | ✓ | ✓ | — | counterpart exists and cites NO address — §4 |

**Before: 0 of 24 rows flagged by anything. After: 5 of 24.** The screen fires,
so it is not `#3470`; and it fires on 5 rather than 24, so it is not noise.

**The split did not move**, by charter: `absent 12 · R-derived 7 · fitted 2 ·
unexercisable 3`, and `SPLIT` in `crates/c2-harness/tests/clause_table.rs` is
unchanged. Every flagged row is published as an adjudication for the wave that
owns adoption. `#3505` is six for six on lanes that moved a number by
constructing one, and this lane declines to be the seventh.

**These five rows are frozen, not resolved.** Their `cites` cells now record
the footprint as it stands, so today's five are silent and a **new** citation on
any of the 24 is RED. That is the property the token screen lacked.

## 2. The retrospective control — a false negative that really happened

`work/w-clausegen/control_c1.py`, output in `control_c1.out`. Freezes `cites` at
`8b4ca972c^` (before `w-inlbudget` adopted), grades the table as it stood at
`72caf2586`, greps `crates/` at `72caf2586`.

At that commit:

| | C14 | C18 |
|---|---|---|
| `state` | `absent` | `absent` |
| screened token | `INLINE_MAX_DEPTH` | `FORTY_INSTRS` |
| token found under `crates/`? | **no** | **no** |
| → **check 5 (token)** | **GREEN** | **GREEN** |
| `splice.rs` citations of the row's own `addr` | **2** | **2** |
| → **check 6 (address)** | **RED** | **RED** |

The counterparts were `INLINE_LEVEL_DEPTH_CAP` and `INLINE_CHARGE_EXEMPT_MAX`,
both `PROV[R]` at these rows' own addresses, and `w-inlbudget`'s rung named C14
and C18 in those words. **They sat unseen for a full wave.** This is not a
plant; it is the defect, replayed.

**PREREG P1 is a MISS and is scored as one.** It predicted RED on exactly C14
and C18 at p = 0.7. The control is RED on **eight** rows: C3, C4, C11, C12, C13,
C14, C18, C19 — every new citation in that window, which includes the two rows
`w-inlbudget` did get credit for (C3, C19), C13's address-sharers, and C4.

Being wrong in this direction is a **stronger** result than the prediction: run
at the wave-18 merge, check 6 would have surfaced **all four** staleness cases
(C3 and C19, which were caught by token collision; C14 and C18, which were not)
in one run, a full wave before they were found by hand. It is still a miss.

## 3. Adjudication of the five — and two of them say the table is over-stating `absent`

**C4 — `absent` is over-stated, and the row's own note is now false.**
`crates/c2-core/src/splice.rs`'s `Expansion::at_pass_entry()` is documented at
this clause's own address as *"the pass entry's state — `FUN_10b61ee1(fn,
level = 1, budget = B, 0, 1e8, 0)`, `0x10b6276e`, with `DAT_10c3f50c` zeroed at
`0x10b6274c`"*, and it returns `level: 1, level_base: 0, budget: Parent`. The
row's `note` reads *"no depth/budget parameters exist to pass"* — **there are;
`Expansion` is them.** What is genuinely absent is the *value* of `B`, which
needs the pre-codegen instruction count the row's `blocker` column already names
(`no-instr-count`, L2's read this wave). So C4 is one row carrying two facts:
the entry state is adopted and address-cited; the budget value is blocked.

**C10 — `absent` is over-stated in a different and sharper way.** Four citations
of `0x10b60a28` in `splice.rs`, and they do not describe an unimplemented
clause: they establish that the bypass **lands between c2's two depth arms**,
and the port carries a `forceinline` **parameter** through
`declines_at_maxlevel` and sweeps it over both values in the registered decision
surface. What is absent is the *reader* — the port cannot see `[sym+0x4c] &
0x2000`, so it passes `false`. Under the amended goal (`CLAUDE.md`: *"expose
decision points … as named, settable parameters"*), **C10 already is such a
parameter with no input wired to it**, which is not what the word `absent`
conveys. The next wave should decide whether the table needs a state for that;
this lane does not invent one.

**C11, C12 — address-sharing artifacts, and a real limit of the instrument.**
Both cite `0x10b5c06b`, which is also C13's, and C13 is `R-derived`. All four
citing files (`comdat.rs`, `gl.rs`, `noinline_boundary.rs`, `clause_table.rs`)
discuss C13's bit-6 legality test. **The 24 rows use 21 distinct addresses** —
C11/C12/C13 share one and C5/C6 share another — so for **5 of 24 rows the
address cannot discriminate between the clauses at it**. Published here rather
than left to be discovered.

**C23 — a band mention.** `crates/c2-harness/src/subsys.rs` uses `0x10b5b86d` as
the inliner band's low bound, not as this clause's parameter-table selection.
`unexercisable` is unchanged and correct.

**PREREG P6 is a HIT on all four of its parts** — C11/C12 address-sharing, C23 a
mention, C4 and C10 partial rather than full. The registered bias direction
(optimistic about the table) did not pay off in the direction expected: the two
partials are more substantive than "the `absent` cell is slightly over-stated",
because C4's `note` is affirmatively false and C10's is arguably a category
error.

## 4. Sensitivity, MEASURED — 6 of 9, and it must never be quoted as 9 of 9

The number that decides whether an address screen is worth having is how often
an adoption leaves an address fingerprint. Over the nine rows that already have
a counterpart (7 `R-derived` + 2 `fitted`):

| cites its own address | does not |
|---|---|
| C3, C13, C14, C15, C18, C19 | **C8, C20, C24** |

**6 of 9 = 67 %.** `PREREG` **P3 predicted 5–8 of 9 → HIT.**

The three misses are diagnostic, not random:

* **C8 and C20 are the two `fitted` rows.** A fit is by definition not derived
  from a read, so there is no `PROV[R]` address to cite. **Check 6 is
  structurally blind to a fitted counterpart** — and a fit is exactly what a
  lane produces when it cannot read the clause, so the blindness is aligned with
  the cases the table most wants flagged. This is the instrument's weakest
  seam and it is stated here, not buried.
* **C24 is an `[O]`-derived counterpart** located from the container side, so
  it cites `0x10c1f9a6` (the `DISCLOSURE` row's address) and not `0x10b9bf6c`.
  A real adoption with a real address trail, invisible to this screen because
  the trail points somewhere else.

So check 6 is **necessary and not sufficient**, in the same relation to check 5
that DECODE has to ALIGN. Neither replaces the other: check 5 catches an
adoption that reuses the table's spelling, check 6 catches one that cites the
table's address, and **an adoption that does neither is still invisible.**

## 5. Deliverable 1 — and PREREG P2 was a bad miss

`work/w-inlmetric/gen_table.py` renders §6.1 between two markers; 34 generated
lines over 24 rows; `crates/c2-harness/tests/clause_table.rs` goes RED on
divergence, on a missing marker pair, and on a hand-edit.

**P2 predicted 4–8 differing cells (accept 2–12). The first run was RED on 27
differing lines** — `work/w-clausegen/gen_red_on_handwritten_page.out`.
**Outside the accept band; scored a miss.** I predicted a page that had drifted
in a few cells. What the page had was a *separately hand-written rendering of
every clause*, which is a much more thorough divergence than "re-sync needed"
implies — and is the strongest argument for generating that this lane found.

**The information audit, because generating replaces hand-written prose.** All
27 lines were read individually:

* **Nothing factual was lost.** The differences are typography (backticks,
  `1e8` vs `100000000`, `⇒`/`≥`), synonym (*"skipped wholesale"* vs *"whole pass
  skipped"*), and intra-clause bold on five rows where the TSV carries the same
  emphasis in capitals.
* **Three rows GAINED detail the page had abbreviated away**: C17's `⇒ DECLINE`,
  C18's `cmp WORD [callee+0x50], 0x28`, C19's `DAT_10c3f5cc += same`.
* **One drift was ended rather than preserved**: the page carried *two*
  spellings of the same state, ``**`[R]`-derived**`` on C3/C13/C19/C24 and
  `**[R]-derived**` on C14/C15/C18. A generator has one.
* `wgloss`/`egloss` were added so the page-only parentheticals survive; without
  them this would have been a `#3748` silent narrowing dressed as a fix.

## 6. What this lane refused to do

* **Convert any clause.** Five rows are flagged, two of them (C4, C10)
  substantively. Not one `state` cell moved and `SPLIT` is unchanged.
* **Narrow the screen to `crates/*/src/`.** Declined on the wave-19 grounds: it
  redefines what `absent` *means*.

  **And a second reason for declining, which I wrote here first, is FALSE.** I
  claimed narrowing would have re-hidden C4's and C10's citations. It would
  not: `splice.rs` **is** under `crates/c2-core/src/`. Measured properly
  (`work/w-clausegen/src_narrowing.py`, output in `src_narrowing.out`), **all
  five flagged rows carry at least one `crates/*/src/` hit**, so narrowing
  check 6 would change the flagged set by **0 of 5**. The narrowing question is
  therefore *orthogonal* to §3's finding; what narrowing would actually remove
  is the self-reference hazard (`clause_table.rs` and `noinline_boundary.rs`
  mentions on C11/C12), at the price of redefining `absent`.

  Recorded rather than quietly fixed, because it is this lane's own subject
  happening to this lane: **a consequence asserted instead of measured.** The
  grep took eleven seconds.
* **Rubber-stamp the five.** Their `cites` cells record a *measured footprint*,
  not a verdict. Check 6 stays able to fire on all 24 rows tomorrow.
* **Add a `scripts/gate.sh` row** (`#3691`).

## 7. Scored predictions

| | prediction | result |
|---|---|---|
| P1 | C-1 RED on exactly C14 + C18 | **MISS** — RED on 8 rows; §2 |
| P2 | 4–8 differing cells (accept 2–12) | **MISS** — 27 differing lines; §5 |
| P3 | sensitivity 5–8 of 9 | **HIT** — 6 of 9; §4 |
| P4 | `SPLIT` and `ROWS` do not move | **HIT** |
| P5 | byte delta zero, `GATE: PASS` | see the rung's gate block |
| P6 | none of the five is a clean full adoption | **HIT** on all four parts; §3 |

Two misses, four hits, and **both misses are in the pessimistic direction** —
the screen caught more than predicted and the page had drifted further than
predicted. The registered bias direction in `PREREG` §5 was OPTIMISTIC about the
instrument's state, so the base rate (`#770`) held.
