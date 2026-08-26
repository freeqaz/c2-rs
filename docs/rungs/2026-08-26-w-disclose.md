# w-disclose — the constants nearest the judge now have a ledger: four rows, 89 constants, and the emit path's first entry in the provenance register

    Tag:       w-disclose
    Slug:      w-disclose
    Date:      2026-08-26
    Kind:      characterization (decision 16's second lane)
    Outcome:   instrument
    Fixtures:  none — characterization lane: it files provenance rows for
               constants already in the port and changes not one non-comment
               byte in `crates/`
    Census:    +0
    Record:    `docs/whitebox/DISCLOSURE.md` (the four rows and the two
               amend-beside boxes); `docs/whitebox/ref/README.md` §7.1 (what
               adoption does to that directory's own status); board
               `#3642`–`#3646`, with `#3647` unspent
    Fail axis: this lane can fail with every byte identical, and the failure
               would be a row nobody can re-derive or a row registering a
               provenance the value does not have — so the values were graded
               against the pinned image BEFORE the rows were written, not after

**Base:** `e548f01fd` · **Branch:** `wt-w-disclose` · **Reach:** 0, as
predicted. **Prereg:** `work/w-disclose/PREREG.md`, committed at `e0150d1c1`
before the first ledger row was written.

---

## 0. What this lane was for, in one paragraph

Lane `w-provenance` built the derived-vs-fitted census and its first real run
found a hole in the provenance register: `crates/c2-core/src/codegen/mop.rs`
holds **88** constants read out of `c2.dll`, on the **emit path**, and
`docs/whitebox/DISCLOSURE.md` had no row whose `Adopted into` was any
`crates/c2-core/src/codegen/` file at all — while `mop.rs`'s own module doc
asserted *"`docs/whitebox/DISCLOSURE.md` carries the provenance rows."* That
lane stated the hole and deliberately did not repair it, because **filing a row
is an adoption decision**. This lane takes the decision. Four rows, 89
constants, and the file's false claim repaired comment-only.

**This is engineering provenance and it is not a legal question.** `CLAUDE.md`
§ "Whitebox analysis is AUTHORIZED", project owner, 2026-08-17: disassembling
`c2.dll` to interoperate is settled (*Sega v. Accolade*; *Sony v. Connectix*;
17 U.S.C. §1201(f); EU Software Directive Art. 6). Nothing below hedges,
because there is nothing to hedge about. The register exists so a future reader
can tell a **measured** fact from a **read** one and re-derive either.

## 1. What was filed

Ledger **17 rows → 21**.

| row | kind | what it registers | constants |
|---|---|---|---|
| **`W-MOP-1`** | adoption | the **85 opcode NUMBERS** — c2's own 0-based indices into the mnemonic table `0x10b1b260`, the same index `0x10c3a578` and `0x10c39b18` are read with — and the extent `MAX_C2_OPCODE = 0x294` (`_last` `0x295` minus one) | **86** |
| **`W-MOP-2`** | adoption | **85 whole ROWS**: base word from `0x10c3a578`, form number from `0x10c39b18`, mnemonic from `0x10b1b260`, as source literals in `OPCODES` | **1** |
| **`W-MOP-3`** | adoption | the **field placements** — `ref/P_ENCODE.md` §5's arms as 27 field plans over 35 form numbers, plus form 68's doubly-split rotate and form 55's operand-free `\| 0x02800000` | **1** |
| **`W-EXCLASS-1`** | adoption | **`0x10b25e48`**, the `.ex` operand-class table's *address* and indexing rule; no entry transcribed, the byte is read out of the image at run time and every untraced opcode refuses | **1** |

**Coverage: 88 of 88 in `mop.rs`, 1 of 1 for `EX_CLASS_TABLE`.**

### 1.1 The grouping rule, registered in the prereg before a row was written

> **One row per c2 artifact READ — never one row per constant.**

This is the ledger's **existing** convention rather than a new one, and that is
why it was chosen: `W-MID-1` is *one* row for a table address plus a stride plus
an index origin plus a sentinel; `W-STAGETAP-1` is *one* row for **seven**
call-site addresses. Eighty-five transcriptions of one table are one read.

`mod op`'s 85 constants are reachable from `W-MOP-1` through the single
`PROV-BLOCK[R]` at that module's head — the block form the marker convention
defines as covering every population member lexically inside it, and which the
census counts **per constant** regardless, so the block saves lines and never
saves a number.

### 1.2 `W-MID-1` and `W-MID-2` are EXTENDED, not duplicated — and not one word of either is rewritten

The brief asked whether the existing `W-MID-*` rows already cover part of this.
They cover a **neighbouring** fact, and the difference is the reason the new
rows exist:

* both adopt a **table address, a stride and an index origin** into
  `crates/c2-reference/tests/middle_interfaces.rs`;
* both state in their own text that **"no table entry is copied"**, because
  that test reads the strings and words out of the pinned image at run time;
* that is **true where they say it** and **false one crate over**, where 85
  entries are Rust literals, because `mop.rs` must emit with no image present.

So an **amend-beside** box was added under them (`ref/README.md` §2.1's rule: a
document that silently absorbs its own corrections is one nobody can grade),
and neither row's text was touched.

**`W-MID-3` was deliberately left entirely alone.** Its closing sentence —
*"2 of the 111 arms are read, and the relocation/label half of the emit seam is
read NOT AT ALL"* — is an accurate statement about **its own lane**. `W-MOP-3`
states its own count (**27 of the 79 distinct arms**) separately rather than
editing somebody else's row, and it repeats the second clause, which still
holds: **no relocation and no label placement is adopted anywhere in `mop.rs`.**

### 1.3 Why the new rows are `W-MOP-*` and not `W-MID-5/6/7`

Every `W-MID-*` and `W-STAGETAP-*` row is **instrument-only** by its own text —
a test file, or `c2host/stagetap.c`. **`W-MOP-1`/`-2`/`-3` are the ledger's
first rows whose `Adopted into` is on the emit path**: `mop::base_word` is *the
port's only source of a primary opcode*. Folding them into a family whose whole
character is "touches no emit" is the one thing a provenance register must not
do, so they got their own name and the distinction is stated in each row.

## 2. Verification — against the image, not against the transcription

`work/w-disclose/verify_rows.py`, committed, six checks, two directions:

| | check | result at the tip |
|---|---|---|
| **A** | ledger → `crates/`: every cited path exists, every cited symbol present | **20 rows name a code path, 20 live, 0 dead** — 17 of them name a `crates/` path (**#3631**'s 13, plus this lane's 4) |
| **B** | `crates/` → ledger: every `DISCLOSURE <row>` citation names a real row | **1 dead**, and it is not this lane's — see §4 |
| **C** | coverage: every `[R]` constant in `codegen/` reaches a row **that names its file** | **88 of 88**, from **0 of 88** at base |
| **D** | values: `OPCODES` against a **live dump of the pinned image** | **85 of 85** agree on mnemonic **and** base word **and** form; `MAX_C2_OPCODE` = the dumped extent `0x294` |
| **E** | `ref/ENCODE_OPCODES.txt` reproduces from the image | **byte-identical**, 660 rows |
| **F** | every c2 address `mop.rs` cites is a table, an arm, or a composer | **36 of 36 accounted for** (27 arms, 4 composers, 4 tables, form 2's arm) |

**Check C's bar is deliberately not *"the cited row exists"*.** All 88 constants
passed that weaker test at `e548f01fd` — they cite `W-MID-1`/`W-MID-2`, which
are real rows that name a different file. **That gap is exactly `#3632`**, and a
checker that could not see it would have reported the hole as clean.

**Check D is the one that would have caught a wrong row.** A row registering a
provenance the value does not have is worse than no row at all, so the values
were graded against the binary before the rows were written, not after.

### 2.1 The controls were watched failing, and one caught a defect in itself on run one

Five plants, each required to produce a **new** failure line naming **its own**
check:

```
plant A  DISCLOSURE.md, a row's path -> nonexistent   -> "path missing: …"
plant B  mop.rs, W-MID-1 -> W-BOGUS-9                 -> "cites a row that is NOT in the ledger"
plant D  mop.rs, add's base 7c000214 -> 7c000215      -> "ADD: base 7c000215, c2 says 7c000214"
plant E  ENCODE_OPCODES.txt, same one-nibble edit     -> "DIFFERS from a live dump"
plant F  mop.rs, arm 10bfa456 -> 10bfa999             -> "not a c2 table, arm or composer"
```

`self-test PASSED`, evidence `work/w-disclose/selftest_base.txt`.

**Plant A failed on its first run, and the failure was real.** It targeted
`mop.rs`'s path inside `DISCLOSURE.md` — a string that **is** in that file at
base, but only in the *"Adoptions this ledger does not carry"* **prose**, which
check A does not read. The plant matched, changed the file, and changed nothing
gradeable: **#3516**'s shape exactly, and the reason that board row says every
corruption-based self-test must confirm the mutation reached the thing under
test. The plant was moved to a path that is inside an actual table row at both
ends of the lane, and the reason is written into the code beside it.

**The baseline is deliberately not required to be green.** At `e548f01fd` it is
red — 6 failures — and that redness is this lane's subject. The control is
therefore stated as a **delta**: a plant must add failure lines the clean run
did not have.

## 3. What was repaired in `crates/`, comment-only

**Not one non-comment byte.** Verified mechanically, not by eye — every added
and removed line in `crates/` stripped of its leading `+`/`-` and whitespace
begins with `//`, `/*` or `*`, so a `code; // comment` line could not hide in
it.

1. **The false claim itself.** The module doc said *"`DISCLOSURE.md` carries the
   provenance rows"* while it did not. It now names the three rows, says the
   sentence was untrue for four days, and states why `W-MID-1`/`W-MID-2` are
   the neighbouring facts and not these.
2. **`mod op`'s block marker** still carried *"A DISCLOSURE ROW IS OWED FOR THIS
   FILE AND DOES NOT EXIST"*. It now cites `W-MOP-1`.
3. **`OPCODES`, `MAX_C2_OPCODE`, `EncodeParams::C2`** now cite `W-MOP-2`,
   `W-MOP-1`, `W-MOP-3`. **No marker letter changed and no marker was minted**:
   the census reports the same `100 [R] · 49 [O] · 4 [F] · 18 [S] · 18 [N] ·
   6 rule marks` at base and at tip.
4. **`plan`'s composer note** (§4's `#3626` class): the four memory groups cite
   the composer, not the jump-table arm, and the mapping is now written down.

### 3.1 A count inside a provenance marker was wrong by 14 rows — **#3643**

`mop.rs` said *"the port emits **71** distinct opcodes and the other **589** are
not transcribed"*, *"**71** of c2's 660 rows"* (inside a `PROV[R]`), and *"the
port's 71 opcodes reach **24** of c2's **109** forms"*.

Measured: **85** rows, therefore **575** untranscribed, over **34** distinct
forms, out of the **104** distinct form values c2's table contains
(`ref/P_ENCODE.md` §3). Wrong since the file's **first** commit `227b90dd7` —
the table has had 85 rows throughout — and **the same file already had 575
right**, three hundred lines down in `EncodeParams::row`'s own comment.

**The census counts *whether* a constant is tagged, never whether the tag's own
text is true**, so the 71 rode inside a `PROV[R]` for four days and was quoted
forward into `#3632` and into `DISCLOSURE.md`'s prose. Corrected comment-only in
both places, amended beside in the ledger. **No value moved and none could.**

## 4. Found, reported, and deliberately NOT repaired

Three items. Each is outside this lane's fence, and a lane that needs a peer's
surface **stops and reports**.

* **`README.md` (repo root) — #3644.** Its per-finding paragraph says *"the
  opcode/encoding tables read by `crates/c2-reference/tests/middle_interfaces.rs`
  … touch no emit path and no refusal predicate."* Literally true of the file it
  names; misleading about the tables, which are now also in `mop.rs`. This is
  `DISCLOSURE.md`'s **own checklist step 4** — *"tell the coordinator:
  `README.md`'s wording must change … it must not lag the code"* — and telling
  the coordinator is the prescribed action, not editing it.
* **`middle_interfaces.rs:634` cites `DISCLOSURE W-EXT-1`, which is not a row —
  #3645.** It is a **pre-draft** in `WB_READER_FINDINGS.md` §5.3 (the `.ex` TYPE
  word's 1/2/3-byte form, `0x10c1fe40`, **#1594**). **`#3631` checked the
  ledger → `crates/` direction and found 13 of 13 live; the reverse direction
  had never been checked**, and it has one dead member. Not filled, because
  **#3626** is the standing precedent against carrying a pre-draft on sight —
  `W-INLINE-1`'s pre-draft carried two wrong addresses **in bold** for eight
  days — and because the citing file is outside the fence. The same file's
  `EX_CLASS_TABLE` marker also still reads *"NO DISCLOSURE ROW EXISTS FOR THIS
  ADDRESS"*, which `W-EXCLASS-1` makes false; both repairs land together.
* **A rule mark is owed on `mop::plan` — #3646.** `plan` is a *rule* in the
  census's sense. Adding a `PROV[R]` token there would have moved
  `provenance_census.py`'s rule-mark numerator from 6 to 7 — a peer's number,
  mid-wave, with the marker surface fenced to `w-provext` by name. The doc
  comment cites `W-MOP-3` in prose and says in its own text that it carries no
  marker token and why. **The fence held in the one place it was cheapest to
  break**, which is the whole reason `#3635` fences three facts by name rather
  than by file.

**A fifth item, repaired rather than reported, because it is a typo and not a
claim.** Every row in the ledger was cell-counted while filing: **`W-OBJPLAN-1`
renders as an eleven-column row in a seven-column table**, because its
twice-amended Notes cell writes `|names| / |emitted|` with the pipes unescaped —
backticks do not escape a pipe in a markdown table. A reader loses the last
third of that row's notes into four phantom columns. Three backslashes, no claim
touched, and the other 20 rows are now positively verified at 7 cells rather
than merely unchallenged. **This is `#3626`'s shape at its smallest**: the
ledger's most-amended row had a rendering defect nobody saw because nothing
counted.

**A fourth thing was checked and is NOT a defect.** `plan`'s header says every
arm cites *"the address of the c2 arm it was read from"*, and four memory groups
cite `0x10bf9e55` / `0x10bf9eb5` / `0x10bf9788` / `0x10bf97c8` instead of arms
`0x10bfa667` / `0x10bfa676` / `0x10bfa17f` / `0x10bfa1a1`. `P_ENCODE.md` §5.5
shows those arms do nothing but `call <composer>; or ebx,eax` — so the citation
is one level **deeper**, and it is the level the field placement actually lives
at. The imprecision was in the sentence, not in the addresses; the mapping is
now written down and check F holds all 36 addresses.

## 5. REQUIRED-ZERO and gate evidence

**`IDENTITY DIFF: 0 lines over 21 rows — required-zero byte delta HOLDS`**

Both ends are **real gate runs on this worktree** (`#3075`/`#3117`/`#3128`: the
base end is measured, never inherited):

* **base** `e548f01fd`, from a **detached clean checkout** of the base commit in
  this worktree — `work/w-disclose/gate_base.txt`
  * `GATE: PASS (HATCH-RED REFUSED) — 18/18 lanes ran and every one of them graded a corpus,`
    `the sweep graded 19460 of 19556 generated cases and the cross graded`
    `90424 of 90812 case-lane cells, with 0 mismatches anywhere`
* **tip** `4c6822549` — `work/w-disclose/gate_tip.out`
  * **byte-identical verdict block**, same four lines, same counts
* `work/w-disclose/identity_diff.txt`: `count-bearing rows: 21 base, 21 tip
  (enumerated, not asserted)`

**The verdict line was read, never the exit code** — `gate.sh` prints
`GATE: REFUSED (DIRTY crates/)` and still exits 0, so a status is not evidence.
The diff instrument's own control was run beside it:
`scripts/gate_identity_diff.sh --self-test` → `SELF-TEST PASS: enumeration 21,
control silent, #3515's signature found exactly (14 lines / 7 rows) and
nonzero, truncation refused.`

`cargo test --workspace`, both ends, **59 targets · 1925 passed · 0 failed ·
1 ignored — identical**. `work/w-disclose/test_base.txt`,
`work/w-disclose/test_tip.txt`.

**Which check this lane owes, per `#3215`.** `#3215` asks a test-landing lane
for the BINARY hash and a revert-everything lane for the TREE hash. This lane is
**neither and is closer to the second**: it lands **zero** non-comment bytes in
`crates/`, so the identity diff is the discriminating check and it is 0 lines.
The tree hash necessarily moves — comments are content — and claiming it did not
would be false.

**The first tip run was RED and is recorded rather than quietly re-run.**
`cargo test --workspace` failed `rung_registry` twice over: *"no `Tag:` in the
header block"* (this rung's header used bold prose instead of the template's
indented `Tag:`/`Slug:`/`Kind:` block) and *"`docs/rungs/INDEX.md` is stale — it
is GENERATED"*. Both were **found by the suite, not by review**, and both are
exactly the class a docs-only lane assumes it cannot hit. Fixed at `4c6822549`,
re-run green.

## 6. Prereg grade

`work/w-disclose/PREREG.md`, committed at `e0150d1c1` before the first row was
written and never edited. **Eleven predictions: 11 HIT, 0 MISS.**

| # | predicted | measured | |
|---|---|---|---|
| **P1** | ledger ends at **21** rows | 21 | **HIT** |
| **P2** | **4** new rows | `W-MOP-1`, `W-MOP-2`, `W-MOP-3`, `W-EXCLASS-1` | **HIT** |
| **P3** | **89** constants covered | 88 (`mop.rs`) + 1 (`EX_CLASS_TABLE`) | **HIT** |
| **P4** | `mod op`'s 85 covered by **one** row via the block marker | one row, `W-MOP-1` | **HIT** |
| **P5** | a live dump reproduces `ENCODE_OPCODES.txt`, and `OPCODES` agrees with the **image** 85/85 | byte-identical, 660 rows; 85/85 on mnemonic + base + form | **HIT** |
| **P6** | **0** further provenance mismatches | 0 | **HIT** |
| **P7** | **0** dead ledger→`crates/` citations over 17 rows | 17 of 17 live (20 of 20 counting `c2host/`) | **HIT** |
| **P8** | identity diff **0 lines over 21 rows** | 0 over 21 | **HIT** |
| **P9** | test and target counts identical base vs tip | 59 / 1925 / 0 both ends | **HIT** |
| **P10** | gate GREEN, 0 mismatch, graded > 0 | `GATE: PASS`, 18/18 lanes, 0 mismatches, 19,460 + 90,424 graded | **HIT** |
| **P11** | **5** board rows, `#3647` unspent | `#3642`–`#3646` filed, `#3647` unspent | **HIT** |

**Eleven for eleven is a weak result and the prereg says why in its own §0.**
Most of the measurement was already taken when the prereg was written — the
census re-run, the 85-row comparison against `ENCODE_OPCODES.txt`, the dead
`W-EXT-1` citation, the 71/589 count error — and §0 lists all ten of those as
**M1–M10, explicitly not predictions**. The predictions that carried real risk
were **P5** (a stale committed transcription would have shown), **P6** and
**P7**, and they were stated with their bias direction (**optimistic**) before
the checks existed. **P3 is the one that would have been a MISS under a weaker
definition of covered** and was not, because check C's bar was fixed in the
prereg's grouping rule rather than after the fact.

**The decline floor was not reached.** Its first clause discharged at M1 (the
census's `[R]` count for `mop.rs` is 88 on this tree); its second — *"`OPCODES`
disagrees with a live dump, in which case the finding is a board row and not a
row in the ledger"* — was the one worth having, and the dump agreed 85/85.

## 7. Deliberately not taken

* **Carrying `W-EXT-1`** — §4, `#3645`. It needs its addresses re-verified and
  its adoption re-priced by a lane that owns `crates/c2-reference/`.
* **Editing the root `README.md`** — §4, `#3644`. Outside the fence, and the
  ledger's own checklist says the action is to tell the coordinator.
* **`ref/P_ENCODE.md`** — `w-encmap`'s this wave. This lane cites §5 and §5.5
  and writes nothing there. **`w-encmap` is adjudicating whether `encode.rs` and
  `mop.rs` are two readers of one fact** (`#3635`); nothing here pre-empts that
  and nothing here depends on the answer — a row records where a value came
  from, which is true whichever way the duplicate-reader question falls.
* **Transcribing the other 575 base-word rows.** `W-MOP-2` records that the
  subset is deliberate and `mop.rs` says why: 575 unexercised claims behind the
  same green test as the exercised ones is `STATUS.md` trap 0.
* **Reconciling `WB_READER_FINDINGS.md` §3.1's nine disagreements** between c2's
  class table and the port's transcribed `.ex` widths, three of them latent
  desyncs. `W-EXCLASS-1` registers the address and says in its own text that it
  licenses none of that. It is a lane.
