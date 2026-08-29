# w-clausegen — the page is a rendering now, and the absence screen follows an address instead of a spelling

    Tag:       w-clausegen
    Slug:      w-clausegen
    Date:      2026-08-29
    Kind:      instrument
    Outcome:   instrument
    Fixtures:  none — instrument lane: it generates `P_INLINE.md` §6.1 from
               `CLAUSES.tsv`, adds check 6 (`CITES`) to close the absence
               screen's false-NEGATIVE half, and wires both to `cargo test`.
               NO clause was converted and no `crates/` behaviour changed.
    Census:    707728/2417794 → 707728/2417794 (29.27% → 29.27%), +0
    Fail axis: FIVE, and a byte delta is not one of them — this lane writes no
               emitting code, so a zero byte delta is the floor and not the
               grade. (1) THE GENERATOR MUST BE SEEN RED ON THE REAL PAGE, not
               only on a plant: its first run reported 27 differing lines
               against the page as hand-maintained. (2) THE MARKER CASE — if
               the markers vanish there is nothing to compare, and "nothing
               differs" is the same string as "everything matches"; a control
               asserts RED on a marker-stripped copy. (3) CHECK 6 MUST FIRE ON
               A REAL HISTORICAL FALSE NEGATIVE, not a synthetic one — control
               C-1 replays `72caf2586`, where check 5 was GREEN on C14/C18 and
               splice.rs cited both rows' own addresses. (4) THE SPLIT MUST NOT
               MOVE: an instrument lane that converts rows because it changed
               the screen has constructed its own number (`#3505`), so `SPLIT`
               is asserted unchanged. (5) THE `CITES` DENOMINATOR — check 6
               needs only the repo and can never legitimately SKIP, but it CAN
               grade zero rows if the column is dropped, and it would print
               GREEN (`#3470`); the test asserts `24 of 24 compared`.
    Record:    this file; prereg `work/w-clausegen/PREREG.md`, committed at
               `4e6bdb81c` BEFORE the screen was touched and before the page
               was opened; the result at `work/w-clausegen/RESULT.md`; the
               retrospective control at `work/w-clausegen/control_c1.py` /
               `.out`; the red-on-real-input evidence at
               `work/w-clausegen/gen_red_on_handwritten_page.out` and
               `work/w-clausegen/demo2_falsenegative_caught.out`

Charter: `docs/WAVE20_BRIEF_2026-08-29.md` §2 "L1". Dispatched at master
`c5bfe89d9`. Board **#3817**–**#3823**.

> **The brief funded two repairs and got a third finding out of them.** The
> repairs landed: §6.1 is generated and a `cargo test` target goes red on
> divergence; the absence screen now follows an **address** as well as a name.
> The finding is what the repaired screen sees — **five rows hold an
> `absent`/`unexercisable` verdict while `crates/` cites the very address that
> pins the clause**, and on two of them (**C4**, **C10**) the `absent` verdict
> is over-stated in ways the table's own `note` cells contradict. **Not one
> `state` cell was moved.**

---

## 1. What it admits, and what it refuses

**Admits nothing into `crates/`.** No constant, no threshold, no decision rule.
The only `crates/` file this lane touched is
`crates/c2-harness/tests/clause_table.rs`, which is a test.

**Deliverable 1 — `P_INLINE.md` §6.1 is a rendering.**
`work/w-inlmetric/gen_table.py` emits the six-column table and both count lines
between two markers in the page; `--check` diffs, `--write` regenerates, and the
page path is positional so a **mutated copy** can be graded. Three additive
`CLAUSES.tsv` columns (`wgloss`, `egloss`, `cites`) were appended by
`work/w-clausegen/add_columns.py`, which copies every existing cell through and
**asserts equality before writing**; `wgloss`/`egloss` exist so that generating
drops none of the page-only parentheticals, which would have been a `#3748`
narrowing dressed as a fix.

**Deliverable 2 — check 6 (`CITES`), and which instrument it is.** The brief
required naming this rather than implying it. It is an **address-resolution
screen recorded as an explicit `CLAUSES.tsv` column** — both of the two options
the brief named, because they are one option: the column is the frozen record
and the address is what populates it. It is **not** a narrowing of check 5 to
`crates/*/src/`, which redefines what `absent` MEANS and was declined at the
wave-19 merge.

> **A correction this lane owes on its own claim.** The first draft of this
> rung, and of board `#3818`, gave a *second* reason for declining the
> narrowing: that it would have re-hidden C4's and C10's citations. **That is
> false** — `splice.rs` is under `crates/c2-core/src/`. Measured
> (`work/w-clausegen/src_narrowing.py`, output in `src_narrowing.out`): **all
> five flagged rows carry at least one `crates/*/src/` hit, so narrowing check 6
> would change the flagged set by 0 of 5.** The narrowing question is
> *orthogonal* to §3's finding; what it would actually remove is the
> self-reference hazard, at the price of redefining `absent`. Recorded rather
> than quietly fixed, because it is this lane's own subject happening to this
> lane — a consequence asserted instead of measured, and the grep that settled
> it took eleven seconds.

**Why an address.** A name is a lane's free choice. An address is not:
`CLAUDE.md` §Whitebox requires a `DISCLOSURE` row naming the address in the same
commit that adopts a disassembly-derived constant, and `PROV[R]` citation is what
every adopting lane in this subsystem has in fact done. An adoption therefore
leaves an address fingerprint **whatever it calls itself**, which is exactly the
thing a token screen cannot follow.

**It is a frozen-set differ, not a judge.** It does not decide counterpart vs
mention. It decides whether the citation footprint has **changed since a human
last read it**. Three blindnesses are declared in `cites_in_crates`' docstring
rather than left to be discovered (`#3684`), including that it inherits `#3641`
transposed and that its sensitivity is **6 of 9, not 9 of 9**.

**Refuses to convert.** Five rows are flagged; two substantively. Not one
`state` cell moved, and `SPLIT` is unchanged and asserted unchanged.

## 2. The before/after verdict change over all 24 rows

Published in full in `work/w-clausegen/RESULT.md` §1, as the brief requires.

| | before (`c5bfe89d9`) | after |
|---|---|---|
| rows flagged by any check | **0 of 24** | **5 of 24** — C4, C10, C11, C12, C23 |
| per-state split | `absent 12 · R-derived 7 · fitted 2 · unexercisable 3` | **unchanged** |
| checks | 5 | 6 |
| §6.1 verified against the table | **by nobody** | `cargo test`, RED on divergence |

The screen fires, so it is not `#3470`; it fires on 5 of 24 rather than all of
them, so it is not noise. **The five are frozen, not resolved** — their `cites`
cells record today's footprint, so a *new* citation on any of the 24 goes RED.

## 3. The finding — `absent` is over-stated on two rows, and this lane did not fix it

**C4.** `Expansion::at_pass_entry()` in `splice.rs` is documented at C4's own
address as the pass entry's state and returns `level: 1, level_base: 0,
budget: Parent`. The row's `note` reads *"no depth/budget parameters exist to
pass"* — **there are, and `Expansion` is them.** What is genuinely absent is the
*value* of `B`, which needs the pre-codegen instruction count the row's own
`blocker` column already names (`no-instr-count`, which is `w-instrcount`'s read
this wave). One row, two facts.

**C10 — the sharper one.** Four citations of C10's address in `splice.rs`, none
of them describing an unimplemented clause: they establish that the bypass lands
**between** c2's two depth arms, and the port carries a `forceinline`
**parameter** through `declines_at_maxlevel`, swept over both values in the
registered decision surface. What is absent is the *reader* — the port cannot
see the flag, so it passes `false`. Under the amended goal (`CLAUDE.md`: expose
decision points *"as named, settable parameters"*) **C10 already is one, with no
input wired to it**, and `absent` does not convey that.

**C11, C12** are address-sharing artifacts of C13, and **C23** is a
band-boundary mention in `subsys.rs`.

**This is an instrument result about the previous three waves' published
splits**, in the brief's own words, and it is left for the wave that owns
adoption. `#3505` is six for six on lanes that moved a number by constructing
one; this lane declines to be the seventh.

## 4. The controls, all four watched RED

* **C-1, retrospective and the real one.** `control_c1.py` freezes `cites` at
  `8b4ca972c^`, grades the table as it stood at `72caf2586`, and greps `crates/`
  at `72caf2586`. Tokens `INLINE_MAX_DEPTH` and `FORTY_INSTRS` are absent →
  **check 5 GREEN on C14 and C18**; `splice.rs` cites `0x10b60a1c` twice and
  `0x10b625b6` twice → **check 6 RED on both**. Not a plant: the defect,
  replayed.
* **C-2.** The generator on a mutated copy of the page → RED, naming the row.
* **C-3.** `--set C1.cites=…` → check 6 RED on C1.
* **C-4.** The pre-existing `--plant C16=…` → RED, unchanged.

**And both green-asserting tests were watched RED on real broken input**, which
is the demonstration the brief required:

1. one character hand-edited inside the page's generated block (`35000` →
   `34999` in C16's clause) — the generator test failed and named the differing
   line;
2. a file under `crates/` citing C16's own address as
   `INLINE_HUGE_CALLER_INSTRS` — **check 5 stayed GREEN, check 6 went RED**
   (`work/w-clausegen/demo2_falsenegative_caught.out`). The false negative,
   reproduced on demand.

Both mutations were reverted with `git checkout` + `touch` and the rebuild
confirmed before the closing green was quoted — `cp`/`mv` preserves an older
mtime and cargo then runs the **mutated** binary, which cost two lanes a day in
wave 18.

**The first draft of plant (2) spelled C16's screened token in its own prose**
and made check 5 fire for the wrong reason. That is this repo's own rule —
name clauses by **id**, never by token — catching the person applying it, for
the third recorded time.

## 5. Estimate vs outcome

Six predictions registered in `work/w-clausegen/PREREG.md` §5 before the screen
was touched. **Two misses, four hits, and both misses are pessimistic.**

| | prediction | result |
|---|---|---|
| P1 | C-1 RED on exactly C14 + C18 (p = 0.7) | **MISS** — RED on 8 rows |
| P2 | 4–8 differing cells, accept 2–12 | **MISS** — 27 differing lines |
| P3 | sensitivity 5–8 of 9 | **HIT** — 6 of 9 |
| P4 | `SPLIT` and `ROWS` do not move | **HIT** |
| P5 | byte delta zero, `GATE: PASS` | **HIT** — §6 |
| P6 | none of the five is a clean full adoption | **HIT** on all four parts |

**P1's miss is the informative one.** Run at the wave-18 merge, check 6 would
have flagged C3, C4, C11, C12, C13, C14, C18 and C19 — surfacing **all four**
staleness cases (C3 and C19, caught then only by token collision; C14 and C18,
not caught at all) **a full wave before they were found by hand**. I predicted a
precise instrument and built a broader one.

**P2's miss says the page had drifted further than "re-sync needed" implies.** I
expected a few stale cells. What §6.1 held was a *separately hand-written
rendering of every clause*. The information audit (`RESULT.md` §5) reads all 27:
**nothing factual was lost**, three rows **gained** detail the page had
abbreviated away (C17, C18, C19), and one live drift was ended — the page
carried **two** spellings of the same state, on different rows.

`PREREG` §5 registered the bias direction as **OPTIMISTIC about the
instrument's state**, against `#770`'s base rate. Both misses fell that way.

## 6. Gate evidence

All runs on this lane's branch at `efc45a8ea` + the corrections in this commit.

| lane | result |
|---|---|
| `scripts/gate.sh --jobs 16 --require-graded` | **`GATE: PASS`**, unqualified — **18/18 lanes PASS, 0 FAIL, 0 SKIP, 0 NO-RESULT**, **7,056 fixture-verdicts**; sweep **19,542 of 19,638** graded, cross **91,900 of 92,288** cells graded, **0 mismatch anywhere**; debug-lane 18/18, 7,056 verdicts, **0 panic**; `ROW LIVENESS: every row executed`. Graded tree `f0e5a69dd329`, 808 files. `work/w-clausegen/gate.out` |
| `C2RS_REQUIRE_TOOLCHAIN=1 cargo test --workspace --release --no-fail-fast` | **62 targets · 2,003 passed · 1 failed · 2 ignored.** The single failure is **`rung_registry::rung_index_is_generated_and_current`** — `docs/rungs/INDEX.md` is stale **because this lane added a rung file**, and the brief seams `INDEX.md` away from every lane and regenerates it at merge. **The delta is exactly one line** (this rung's row), measured by running the generator, diffing, and restoring: `work/w-clausegen/cargo_test.out`. `clause_table` is **6/6**. |
| `python3 work/w-inlmetric/check_table.py` | `CONFORMANCE-CHECK: GREEN (0 failure(s) over 24 rows)`; `state: {'absent': 12, 'R-derived': 7, 'fitted': 2, 'unexercisable': 3}`; `CITES: 11 of 24 rows have a non-empty frozen crates/ citation footprint, 24 of 24 compared`; `ALIGN: 424,232 instruction starts, 24 of 24 rows graded`; `DECODE: 24 of 24` |
| `python3 work/w-inlmetric/gen_table.py --check` | `TABLE-GEN: GREEN (0 differing line(s) over 24 rows)`, 34 generated lines |
| red-on-broken-input | §4 above; `gen_red_on_handwritten_page.out` (27 lines RED), `demo2_falsenegative_caught.out` (check 5 GREEN / check 6 RED), `control_c1.out` (8 rows RED at `72caf2586`) |

## 7. Found and not taken

1. **`check_table.py` rebound `shown` inside its failure path** and thereby
   rewrote its own `listing :` provenance line whenever an ABSENCE check fired
   — so the one run where a reader most needs to know which objdump listing
   graded ALIGN and DECODE is the run that will not tell them. **Fixed here**
   (board `#3823`), found by reading rather than by a failure. It is the
   `#3470` family one level down: not a clean report over zero rows, but a
   correct report whose provenance lies, and only in the failure case.
2. **Check 6 is structurally blind to a `fitted` counterpart**, and that is the
   worst place for a blind spot. A fit is by definition not derived from a read
   and has no `PROV[R]` address to cite, so C8 and C20 — the two `fitted` rows
   — cite nothing; and a fit is exactly what a lane produces when it *cannot*
   read a clause. Closing this needs a third handle (a `fits` column naming the
   constant a fit is expressed as), and it is not this lane's.
3. **The 24 rows use only 21 distinct addresses.** C11/C12/C13 share one and
   C5/C6 share another, so for **5 of 24 rows the address cannot discriminate
   between the clauses at it**. Any future sharpening of check 6 has to carry a
   sub-address discriminator or accept that those five are adjudicated by hand.
4. **`C24`'s counterpart is real, address-trailed, and invisible to both
   screens** — its `[O]` derivation cites the container-side `DISCLOSURE`
   address rather than the clause's. The general shape is that **a clause and
   its counterpart can be pinned by different addresses**, which a one-address
   column cannot express.
5. **The five flagged rows are the next wave's adoption queue, ranked**: C4 and
   C10 first (both partial, both with a false or misleading `note`), then
   C11/C12/C23 which need only a note that they are adjudicated mentions.
6. **`w-instrcount`'s read this wave lands directly on C4.** C4's blocker is
   `no-instr-count` and its *other* half is already adopted — so C4 is closer to
   convertible than the `absent 12` partition suggests, and that lane should be
   told.
7. **`scripts/gen_rung_index.sh` WRITES `INDEX.md` IN PLACE and prints its
   path to stdout** — there is no `--check` and no dry run. Running it to *see*
   the expected output modifies a file the wave's seam table assigns to nobody;
   it happened here and was reverted with `git checkout` + `touch` inside the
   same tool call. A generator with no read-only mode is a generator that
   cannot be consulted, only obeyed. `gen_table.py` was given `--check` as its
   **default** for exactly this reason.
8. **`INDEX.md`'s `fixtures` column prints a WORD COUNT for every
   `Fixtures: none — …` rung**, not a fixture count: this rung reads **9**,
   `w-inlclause` **12**, `w-paramfill` **19**, `w-globarms` **8** — all four
   declare no fixtures. Pre-existing, affects every construct/characterization
   /instrument rung in the index, and is a printed number that means nothing,
   which is the family this wave is funded to shrink. Not taken: `INDEX.md` and
   its generator are outside this lane's seam.
