# w-clausefix — the conformance table was wrong about TEN of its 24 addresses, not eight, and its checker was run by nothing

    Tag:       w-clausefix
    Slug:      w-clausefix
    Date:      2026-08-28
    Kind:      characterization
    Outcome:   instrument
    Fixtures:  none — characterization lane: it repairs and wires an existing instrument, compiles nothing, and writes no port source
    Census:    707728/2417794 → 707728/2417794 (29.27% → 29.27%), +0
    Record:    this file; prereg `work/w-clausefix/PREREG.md`, committed at `36b616386` BEFORE `CLAUSES.tsv` or `check_table.py` was touched; derivations at `work/w-clausefix/REPAIRS.md`

Charter: `docs/ADOPTION_BRIEF_2026-08-28.md` §L5, discharging `w-inlfit`'s
`#3721`. Dispatched at master `4b79bf46a`. Board **#3780**–**#3785**.

> **Predicted reach 0, delivered 0.** No `crates/` *source* file was edited —
> `git diff master..HEAD -- crates/` is one **new test file** and nothing else.
> No `DISCLOSURE` row proposed, no constant adopted, no `gate.sh` row added
> (`#3691`), and **no clause's substantive claim changed**. The conformance
> split is `absent 17 · fitted 2 · R-derived 2 · unexercisable 3` over 24 rows,
> reachable denominator **21**, at the base and at the tip — verified cell by
> cell, not by re-running the summary.

---

## What it admits, and what it refuses

**Admits:** that ten `addr` cells named the wrong instruction, and moves them to
the instruction the row's own `clause` text describes. **Refuses** to touch
anything else: not a `state`, not a `witness`, not an `exercised`, not an
`owner`, not a `note`, not a word of a `clause`. Mechanically checked, before
and after — **10 of 24 `addr` cells changed, 0 non-`addr` cells changed**, one
additive column.

**Refuses** to re-grade. Three findings below would each justify a clause
re-reading (C19's asymmetric charge, C10's accept-before-every-later-test,
C15's inverted comparison). None was taken. `#3505` is five for five on lanes
that moved a denominator they were standing on.

---

## 1. The result, in one table

| | |
|---|---|
| rows repaired | **10 of 24** |
| …that `#3721` named | 8 (C2, C3, C4, C14, C16, C17, C18, C19) — all mid-instruction |
| …that `#3721` did not | **2** — C10 (filed by `w-inlfit`, not pursued) and **C15 (filed by nobody)** |
| rows that could not be located | **0** — `PREREG` §5's not-located path did not fire |
| rows verified correct and left alone | **14** — C1, C5, C6, C7, C8, C9, C11, C12, C13, C20, C21, C22, C23, C24 |
| repaired addresses leaving their `owner` function | **0** — `PREREG` §3.1/F3's flag-don't-move path did not fire |
| conformance split | **unchanged**, stated explicitly: `absent 17 · fitted 2 · R-derived 2 · unexercisable 3` |

Every repair is shown in disassembly context in `work/w-clausefix/REPAIRS.md`,
against the independent objdump listing — **424,232** instruction starts,
`objdump -d -M intel`, PE32 as `pei-i386` at true VAs, c2.dll sha256
`c80981c0..a66258`.

## 2. `#3721` re-derived, and where it stops (**#3780**)

**It reproduces exactly.** CTL-0, `work/w-clausefix/controls_ctl0.out`: the
pre-repair table graded by an independently written boundary set gives *eight*
ALIGN failures on *exactly* C2, C3, C4, C14, C16, C17, C18, C19. Prereg
falsifier **F1 did not fire**.

**And "eight of the 24" is not the defect count.** Alignment is a *necessary*
condition on a citation, never a sufficient one. Two rows are aligned, inside
the function their `owner` cell correctly names, and point at a different
instruction entirely:

| row | claims | `0x…` decodes to | what it should be |
|---|---|---|---|
| **C10** | `__forceinline: test [sym+0x4c], 0x2000` | `0x10b609d3` → `call 0x10b5e64d` | `0x10b60a28` `and eax,0x2000` |
| **C15** | `maxlevel != 0xff && maxlevel < level => decline` | `0x10b609bd` → `cmp ecx,ebx` | `0x10b60a2f` `cmp edx,0xff` |

`w-inlfit` filed C10 as *"a strictly harder class the new checker does not
reach"*. **It is not a harder class — it is the same class with a luckier
address.** Every one of the eight would have looked exactly like C10 had its
transcription shift happened to land on a boundary; C4's did not only because
the instruction it landed inside is ten bytes long. **C15 was filed by nobody**,
and `cmp ecx,ebx` at `0x10b609bd` is not a near miss: it is the test for whether
a POGO profile record exists, i.e. **C21's guard**, cited under C15's clause.

## 3. C10, settled (**#3781**)

`0x10b60a25`–`0x10b60a3e`, read whole:

```
10b60a25:  8b 47 4c          mov  eax,DWORD PTR [edi+0x4c]
10b60a28:  25 00 20 00 00    and  eax,0x2000          <- the clause's test
10b60a2d:  75 0d             jne  0x10b60a3c          <- skips C15's maxlevel test
10b60a2f:  81 fa ff 00 00 00 cmp  edx,0xff            <- C15
10b60a35:  74 05             je   0x10b60a3c
10b60a37:  39 55 08          cmp  DWORD PTR [ebp+0x8],edx
10b60a3a:  7f b7             jg   0x10b609f3          <- decline
10b60a3c:  3b c3             cmp  eax,ebx
10b60a3e:  75 19             jne  0x10b60a59          <- ACCEPT, return 1
```

The clause's word **BYPASSES** is exact and now has an address: the `jne` at
`0x10b60a3e` returns 1 **before** the POGO branch at `0x10b60a40`, the
caller-huge test C16 at `0x10b60a63`, and the budget test C17 at `0x10b60a73`
are reached. C10's `state` stays `absent` — the port has no accept path at all,
which is what the row already said, and nothing here changes it.

**What the old address actually was.** `call 0x10b5e64d` at `0x10b609d3` is on
the *diagnostic* path — the arm that reaches `0x10b609e4 push 0x10b025a0` and
the `InlBadCandidate` string `FUNCS.tsv` records against this function. It is a
real instruction in the right function on a plausible-looking path, which is
why ADDRESS and ALIGN are both green on it and only a decode can fail.

## 4. C18/C19 — the duplicate-function story is refuted, and one number replaces it (**#3782**)

`w-inlfit` §4 and `ADOPTION_BRIEF` §L5 both state that C18/C19 are `0x11b`
early *because* they landed in an **instruction-for-instruction duplicate of the
wrong function**, `0x10b62488`–`0x10b624be` copying
`0x10b5fb85`–`0x10b5fbbb`. Both ranges decoded here:

| the claim | verdict |
|---|---|
| the two ranges hold one idiom, instruction for instruction | **true** — nine operations, same order |
| …*instruction-for-instruction* identical | **FALSE** — the register differs throughout: `0x10b5fb85` uses `edi` as the zero (`39 3d`, `3b c7`), `0x10b62488` uses `ecx` (`39 0d`, `3b c1`) |
| …of the **wrong function** | **FALSE** — `0x10b62488` is inside `FUN_10b6242a`, which is the function C18/C19's `owner` cell **already names correctly** |
| there are two copies | **FALSE** — there are **three**; a third at `0x10b62519`–`0x10b6254f`, also inside `FUN_10b6242a` |

And it explains nothing even where it is true: the block at `0x10b62488`
contains **no `0x28` and no `+0x50` access**, so no search for C18's
`cmp WORD [callee+0x50], 0x28` could have landed there by matching content.

**One number replaces the whole story.** Both original addresses are exactly
`0x11b` below a real instruction boundary inside the block the clause describes:

```
C18   0x10b6249b + 0x11b = 0x10b625b6   cmp   eax,0x28                  <- the clause's test
C19   0x10b624a2 + 0x11b = 0x10b625bd   movzx eax,WORD PTR [esi+0x50]   <- feeds the clause's add
```

A **uniform transcription shift**. Two independent errors landing on real
instructions of the right block under one constant offset is not a coincidence
worth preferring over a shift.

> **A finding the clause does not carry, and this lane does not add.** At
> `0x10b625b9` a `jbe 0x10b625bd` guards the `sub`, so the **local** budget is
> charged only when `instrs > 40` while the **global** growth total
> `DAT_10c3f5cc` is charged **always**. C19's text says *"`*budget -=
> WORD[callee+0x50]`; `DAT_10c3f5cc += same`"* — `same` is exact (the value is
> re-loaded at `0x10b625bd` rather than reused from a register), and the
> asymmetry is real and unstated. Re-wording the clause is a re-grade and is
> **not taken**; it is filed in §8 for the lane that owns the page.

## 5. The checker: ALIGN folded in, DECODE added, SKIP kept loud (**#3783**)

`work/w-inlmetric/check_table.py` now runs **five** checks. ADDRESS, WITNESS and
ABSENCE are `w-inlmetric`'s and unchanged. **ALIGN** is `w-inlfit`'s
`addr_align.py`, folded in. **DECODE** is new: the table carries an additive
`asm` column holding the objdump text at `addr`, and the check asserts it
matches.

**Inside, not beside — and the reason is that the reason for `beside` expired.**
`addr_align.py`'s own docstring gives one justification for separateness:
*"that grader is another lane's frozen instrument and its green is quoted on the
table's own tree."* That is a **governance** constraint, not a design one, and
`work/w-clausefix/PREREG.md` §1 dissolves it by owning both files. What is left
is `#3679`: two programs is two chances to run only one, and ADDRESS and ALIGN
are two halves of one question — *is this the address of the thing the clause
names*. `work/w-inlfit/addr_align.py` survives as a **delegating shim**, so
`docs/rungs/2026-08-27-w-inlfit.md` §4's citation keeps resolving and there is
one implementation.

**SKIP stays loud and got louder.** No listing ⇒ ALIGN and DECODE print
`SKIP -- listing absent, so 0 of 24 rows were checked`, the verdict reads
`GREEN (ALIGN+DECODE SKIPPED)`, and **ADDRESS/WITNESS/ABSENCE still grade 24 of
24** — CTL-3b plants an ADDRESS defect with no listing on disk and it is still
caught. `#3470`: a clean report over zero rows is not clean.

> **A latent defect in the check being folded in, found by folding it
> (**#3784**).** `addr_align.py` matched `^[0-9a-f]{8}:\t` and counted
> **425,871** instruction starts. `check_table.py` requires a third
> tab-separated field — the mnemonic — and counts **424,232**. The 1,639-line
> difference is objdump's **byte-continuation lines** for instructions longer
> than 7 bytes, e.g. `10b6276b:\t63 c4 10`, the tail of the 10-byte
> `mov ds:0x10c46330,0x10c46334` at `0x10b62764`. **`addr_align.py` would have
> graded an address landing there GREEN.** It is mid-instruction.
>
> **Did it matter? No.** None of the 24 original addresses landed on a
> continuation line, so `#3721`'s eight are unaffected and stand. Reported
> because C4's own original address was `+6` into exactly that instruction —
> one byte further and the eight would have been seven, and nobody would have
> known.

## 6. Nothing invoked the checker (**#3785**)

A `grep` over every `.rs`, `.sh`, `.py` and `.toml` in this repo on 2026-08-28:
`check_table.py` is named in **seven `docs/` markdown files and three lane
`work/` notes, and nowhere else**. No `cargo test` target, no `scripts/gate.sh`
row, no script — the only two `.py` hits are `check_table.py` and
`addr_align.py` naming themselves and each other. Transcript:
`work/w-clausefix/wiring_evidence.out`, taken against master `4b79bf46a` so it
describes the tree the lane **found**, not the one it left. It was written
2026-08-26 and its `GREEN` was quoted by two later lanes.

That is `#3679`'s exact shape — *a `scripts/` entry no funnel invokes is not
enforcement* — and it is the mechanism by which the table carried ten wrong
addresses for two days while being cited as the inliner's conformance evidence.

**The cheapest honest wiring, and it is now done:**
`crates/c2-harness/tests/clause_table.rs`. **Deliberately not a `gate.sh` row**
(`#3691`): a 22nd count-bearing row makes `scripts/gate_identity_diff.sh` exit 2
and refuse to diff for every other live lane in this wave. A `cargo test` target
costs peers nothing and runs in the merge funnel (`#3687`), which is where a
stale table would otherwise reach `master`.

It asserts the verdict **line**, the **row count** and the **split**
separately — because `GREEN` over zero rows reads identically to `GREEN` over 24
in the verdict alone.

> **Blast radius, declared and not discovered (`#3684`).** The ABSENCE check
> greps `crates/` for tokens that must stay **absent** — `INLINE_BUDGET` (C3),
> `budget_decline` (C17), `inline_charge` (C19), `maxlevel` (C15), sixteen more.
> `w-inlbudget` is adopting `P_INLINE` §6.6.2's budget model into `splice.rs`
> **this same wave**. If it uses one of those spellings, this target goes
> **RED on its tree**. That is a **true positive** — the table's `absent`
> verdict would be stale, which is the reading everything downstream depends on
> — and the failure message says so in those words and names the remedy: a
> one-cell `state` edit by `CLAUSES.tsv`'s owner, **not** a change to the
> adopting lane's code.

## 7. Controls, every one watched RED before a green was quoted (`#3336`)

`addr_align.py` was watched red in `w-inlfit`'s tree. **That does not carry** —
it was re-watched here in its folded form. Transcripts:
`work/w-clausefix/controls.out`, `controls_ctl0.out`, `controls_target.out`.

| control | what was planted | result |
|---|---|---|
| CTL-0 | the pre-repair table, unmodified | **RED**, 8 ALIGN failures over 24 rows — `#3721` re-derived |
| CTL-1 | `C16` shifted **one byte** to `0x10b60a64` | **RED**, ALIGN |
| CTL-2 | `C10` moved to a **different real boundary** in the same function | ALIGN **green**, DECODE **RED** — the class ALIGN cannot see |
| CTL-2b | `C15`'s **original** address replanted | ADDRESS green, ALIGN green, DECODE **RED** |
| CTL-3 | `C2RS_OBJDUMP_ASM` → nonexistent | **SKIP**, exit 0, row count and ungraded count both printed |
| CTL-3b | as CTL-3 **plus** a planted wrong-function address | **RED** — the SKIP does not hide the checks that can still run |
| CTL-4 | nothing | **GREEN**, 0 failures over 24 rows, all five checks |
| CTL-5b | the `cargo test` target against the pre-repair table | **FAILED**, naming all eight |
| CTL-5c | the table shrunk to **2 rows** | **FAILED** on the row-count and split assertions — *while the checker itself printed `GREEN`*. `#3748`'s degenerate re-bless, caught |
| CTL-5d | the target with no objdump listing | **2 passed**, `PARTIAL` printed, planted defect still caught |

## Estimate vs outcome

`PREREG` §3.2 registered **ten** rows and named the exact repair address for
each, derived from the listing before any edit — and said so, rather than
dressing a derivation as a prediction. Outcome: **10 of 10 located and
repaired, 0 not located**. The genuinely predictive content was §3.2's *closed
set* and §7's falsifiers, and it is graded here:

| falsifier | fired? |
|---|---|
| **F1** fewer than eight misaligned on this tree ⇒ `#3721` stale | **no** — exactly eight (CTL-0) |
| **F2** a predicted repair does not decode as stated | **no** — 10 of 10 |
| **F3** a repair leaves its `owner` function | **no** — 0 of 10 |
| **F4** C15 is correct as written ⇒ "ten not eight" falls | **no** — `cmp ecx,ebx` is C21's guard |
| **F5** the uniform `0x11b` does not hold for both C18 and C19 | **no** — it holds for both |
| **F6** the conformance split moves | **no** — identical, cell by cell |

**The bias to report is in the other direction.** The prereg predicted the
defect set would be *larger than eight* and it was, but it also predicted the
fourteen remaining rows were correct — and they were, on a check
(`asm`/DECODE) that did not exist when the prediction was written. That is the
one place this lane could have been embarrassed by its own new instrument and
was not.

> **AND SIX FOR SIX IS NOT A CLEAN SWEEP — THE PREREG WAS WRONG SOMEWHERE IT
> HAD NOT WRITTEN A FALSIFIER.** `PREREG` §4 predicted that the ABSENCE check's
> blast radius would fire when a **peer** adopted the budget model. It fired at
> the first full suite run, from **this lane's own test file**, by a mechanism
> the prereg did not consider — a *mention* in a doc comment, not a
> counterpart — and none of this lane's controls could see it, because
> `git grep` is blind to untracked files and the file was unstaged when they
> ran. §10.
>
> **This is the lane's most useful result and it is not on the falsifier
> table**, which is itself the finding: F1–F6 all covered *the addresses*,
> because that is what the lane thought it was doing. The half it was also
> doing — building something that runs in every peer's suite — got a paragraph
> of prose and no falsifier, and that is exactly the half that broke.

## Gate evidence

| lane | result |
|---|---|
| `C2RS_REQUIRE_TOOLCHAIN=1 cargo test --workspace --release --no-fail-fast` | §9 — transcript `work/w-clausefix/cargo_test_tip.out` |
| `c2rs bench` | run as part of the suite (`every_invocation_the_scripts_make_is_still_accepted_bench`); not quoted separately — this lane compiles nothing and emits nothing |
| `scripts/gate.sh --jobs 16 --require-graded` | **18/18 PASS, 0 FAIL, 0 SKIP, 0 NO-RESULT, 7,038 fixture-verdicts** — transcript `work/w-clausefix/gate_tip.out` |
| `scripts/expr_sweep.sh` | a `gate.sh` row: **19,460 of 19,556 graded, 0 mismatch** |
| 878-TU workload scan | not run — no acceptance predicate moved, no emit widened |
| fixtures, `c2rs census` | `Fixtures: none`, `Census: +0` |
| `work/w-inlmetric/check_table.py` | **GREEN, 0 failures over 24 rows**, all five checks graded 24 of 24 — `work/w-clausefix/check_table_tip.out` |

## 8. Found and not taken

Ranked. The first two are corrections **to a page this lane may not edit** —
`docs/whitebox/ref/P_INLINE.md` is `w-inlswitch`'s this wave — so they are
quotable patch blocks here rather than edits there.

1. **`P_INLINE.md` §6.1's C19 text understates the charge.** Quotable
   replacement for the clause's parenthetical, address-cited and ready to
   apply:

   > The charge is **asymmetric**. At `0x10b625b9` a `jbe 0x10b625bd` guards
   > the `sub` at `0x10b625bb`, so `*budget` is decremented **only when
   > `WORD[callee+0x50] > 0x28`**, while `DAT_10c3f5cc` is incremented at
   > `0x10b625c1` **unconditionally**. The two are the same *value* — it is
   > re-loaded at `0x10b625bd` rather than reused — but not the same *event*.
   > The whole block is skipped for a `__forceinline` callee by the
   > `test DWORD PTR [esi+0x4c],0x2000` at `0x10b625a6`, which is a **second**
   > site with that effect alongside `w-inlfit` §3's `0x10b6240f`.

2. **`docs/whitebox/ref/P_INLINE.md` §6.1's table now DIVERGES from
   `CLAUSES.tsv` on ten addresses, and this lane may not close it.** This is
   the highest-value item in the section and it is a *divergence*, not merely a
   staleness: the page and the machine table are the same instrument published
   twice, and they no longer agree.

   > **First, the corroboration, because it matters.** §6.6.3 — written by
   > `w-inlfit`, on master, before this lane started — already publishes
   > repairs for **C4 → `0x10b6276e`**, **C18 → `0x10b625b6`** and
   > **C19 → `0x10b625bb`/`0x10b625c1`**, derived by hand. This lane derived
   > all three **independently, mechanically, from the objdump listing**, and
   > got **the same three addresses**. Two derivations by two methods meeting
   > is worth more than either alone, and it is registered here rather than
   > left implicit.

   What `w-inlswitch` (or the next owner of the page) should apply, ready to
   paste:

   * **§6.1's table, ten `addr` cells:** C2 `0x10b626d8`→**`0x10b62703`**,
     C3 `0x10b626f4`→**`0x10b62708`**, C4 `0x10b6276a`→**`0x10b6276e`**,
     C10 `0x10b609d3`→**`0x10b60a28`**, C14 `0x10b609ae`→**`0x10b60a1c`**,
     C15 `0x10b609bd`→**`0x10b60a2f`**, C16 `0x10b609ee`→**`0x10b60a63`**,
     C17 `0x10b60a04`→**`0x10b60a73`**, C18 `0x10b6249b`→**`0x10b625b6`**,
     C19 `0x10b624a2`→**`0x10b625bb`**. Every `state`, `witness` and
     `exercised` cell stays exactly as it is.
   * **§6.6.3's heading and three of its sentences.** "eight of the
     twenty-four" is **ten**; "425,871 instruction starts" is **424,232**
     (`#3784` — the larger figure counts objdump's byte-continuation lines);
     the paragraph beginning *"C18/C19's citations are `0x11b` bytes early
     because they landed in a DUPLICATE of the wrong function"* is refuted
     (`#3782`) and its replacement is:

     > **C18/C19's citations are `0x11b` bytes early, both of them, under one
     > uniform transcription shift**: `0x10b6249b + 0x11b = 0x10b625b6` (the
     > clause's `cmp eax,0x28`) and `0x10b624a2 + 0x11b = 0x10b625bd` (the
     > re-load feeding the clause's `add`). The nearby block at
     > `0x10b62488`–`0x10b624be` is **not** the explanation: it is
     > structurally but not byte-identical to `0x10b5fb85` (the register
     > differs — `ecx` against `edi`), it is inside `FUN_10b6242a`, which is
     > the function C18/C19's `owner` cell already names, there is a **third**
     > copy at `0x10b62519`, and it contains no `0x28` and no `+0x50` access
     > for a content search to have matched.
   * **§6.6.3's closing "filed as follow-ups"** — C10 is **settled** (`#3781`)
     and its class is not out of reach; it is the class **DECODE** now covers.
     `docs/whitebox/WB_INLINE_FINDINGS.md` §2.2–§2.4 and
     `docs/rungs/2026-08-08-wb-inline.md` carry the same ten addresses and are
     **dated records that stay as written**.

3. **`w-inlfit` §4 and `ADOPTION_BRIEF` §L5 carry the refuted
   duplicate-function story** (§4 above). Both are dated records and stay as
   written; a reader arriving at either should be sent here.

4. **The `asm` column is a weak witness on four rows and says so.** C1, C5/C6,
   C20 and C21 cite function **entries**, so their `asm` cell is `push ebp`.
   DECODE-green on those four means the address is an entry, not that it is the
   *right* entry. Documented in the table header rather than left for a reader
   to discover. A stronger form — a *window* witness, "the clause's constant
   appears within N bytes of `addr`" — is the obvious next instrument and is
   **not built here**, because it needs a per-row window size and that is a
   parameter somebody would fit.

5. **`docs/ADOPTION_BRIEF_2026-08-28.md` §4 and §L5 give `check_table.py`'s path
   as `docs/whitebox/scripts/check_table.py`.** It has always lived at
   `work/w-inlmetric/check_table.py`. Not moved: the path is cited by two
   landed rungs and `P_INLINE.md`, and moving it to fix a brief's typo would
   break three correct citations to repair one wrong one.

6. **The seam table in `ADOPTION_BRIEF` §4 lists `crates/` under this lane's
   *must not touch*, and the dispatch assigns it a `crates/c2-harness/tests/`
   target.** Resolved in favour of the dispatch (`PREREG` §1) and recorded so
   the conflict is visible: the new file collides with nothing any peer owns,
   and no existing `crates/` file was edited.

## 9. Gate and suite at the tip

Tip on a clean tree, under a box carrying **six** peer lanes' workspace suites
concurrently (load average 66–145 for the duration; the first suite run took
53 minutes for work that is ~4 minutes warm).

Transcripts: `work/w-clausefix/gate_tip.out`, `work/w-clausefix/cargo_test_tip.out`.

```
GATE: PASS (HATCH-RED REFUSED) — 18/18 lanes ran and every one of them graded a corpus
  lanes:  18 in the registry — 18 PASS, 0 FAIL, 0 SKIP, 0 NO-RESULT
  graded: 7038 fixture-verdicts across all lanes
  sweep:  PASS — 19556 of 19556 reached, 19460 GRADED, 0 mismatch
  cross:  PASS — 90424 of 90812 cells graded, 0 mismatch
  debug:  PASS — 18 of 18 lanes, 7038 fixture-verdicts, 0 mismatch, 0 PANIC
  graded tree: 879c07904ade (805 files under crates fixtures scripts)
  count-bearing rows: 21 — this lane added none (#3691), and `scripts/` is
  untouched: `git diff --stat master..HEAD -- scripts/` is empty
```

**`HATCH-RED REFUSED` is INHERITED, not this lane's.** `hatch.py apply` cannot
hatch this tree, so the arms have no tree to run on (`#1389`) —
`work/w-hatch/hatch_red_master.txt`, a committed artifact taken on master,
already records `SETUP FAILED: hatch.py apply refused`. This lane's `crates/`
delta against its base is **one new test file**, and the condition predates it.
`w-inlfit` recorded the identical inheritance on 2026-08-27.

```
C2RS_REQUIRE_TOOLCHAIN=1 cargo test --workspace --release --no-fail-fast
  __SUITE__
```

> **The FIRST run of this suite is reported and not hidden**, because it is
> §10's whole subject: at `53deb51ff` it read **61 targets, 1,983 passed, 1
> failed**, and the failure was this lane's own `clause_table` target going RED
> on four ABSENCE rows off its own doc comment. The tree above is the fixed one.
> A lane that reports only its last run has thrown away the only measurement
> that found anything.

## 10. The blast radius fired, from this lane's own file, and the controls could not see it

**This section exists because the lane's own prediction was wrong in an
instructive way, and the suite — not the lane — found it.**

`PREREG` §4 and §6 above predicted that the ABSENCE check would go red when a
**peer** adopted the budget model into `crates/`. What actually tripped it, at
the first full workspace run, was **this lane's own test file**: an earlier draft
of `clause_table.rs`'s doc comment listed four of the forbidden tokens **as
examples of the tokens that must not appear**. Four rows went red because of
prose *about* them:

```
FAIL C3   ABSENCE state absent but token '…' IS PRESENT in crates/
FAIL C15  ABSENCE state absent but token '…' IS PRESENT in crates/
FAIL C17  ABSENCE state absent but token '…' IS PRESENT in crates/
FAIL C19  ABSENCE state absent but token '…' IS PRESENT in crates/
```

Two distinct defects, and only one of them is mine.

**(a) The check cannot tell a MENTION from a COUNTERPART.** `#3641`'s class
exactly — *a counter cannot tell a mark from a mention* — one level down, in the
instrument rather than the census. `token_in_crates` is a **name screen over the
whole subtree**, so a doc comment, a test fixture and a real port constant all
read identically. **Not fixed here, on purpose:** narrowing it to
`crates/*/src/` would redefine what `absent` *means*, and that definition is
`work/w-inlmetric/PREREG.md` §5's, not this lane's. It is named at the site, in
`token_in_crates`'s own docstring, and anything under `crates/` that must
discuss these clauses now names them **by clause id**, never by token.

**(b) `git grep` is blind to untracked files, so the verdict changed at
`git add` time.** This is unambiguously a bug with no semantic trade-off, and it
is fixed: `--untracked --exclude-standard`.

> **This is the part worth reading.** CTL-4 and CTL-5a were watched **GREEN**,
> and they were green *because `clause_table.rs` was untracked at that moment*.
> The lane wrote a defect, ran its controls, saw green, and the defect was
> real the whole time — it became visible when the file was staged, with no
> edit in between. **A control that runs before `git add` was grading a
> different tree from the one the suite grades**, and nothing in the repo says
> so. `#3336` says a control nobody has seen fail is decoration; this adds that
> **a control run at the wrong moment is decoration too**, and the moment is
> not obvious.

**CTL-6**, `work/w-clausefix/controls_absence.out`, watched after the fix: an
untracked probe file under `crates/c2-core/src/` is **invisible** to the old
form and **RED** under the new one, and green returns when it is removed.

Reported rather than repaired away, and folded into **#3785** — which had
already declared this radius, and got the mechanism and the culprit wrong.
