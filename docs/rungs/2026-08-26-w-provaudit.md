# Rung — `w-provaudit` (2026-08-26)

    Tag:       w-provaudit
    Slug:      w-provaudit
    Date:      2026-08-26
    Kind:      characterization (instrument + two queued repairs)
    Outcome:   instrument
    Fixtures:  none — an instrument lane; it names no fixture and moves no census
    Census:    +0 (unchanged, and proved by `provenance_census.py --since 0dcfca959`)
    Record:    this file
    Charter:   decision 17, docs/DECISIONS_2026-08-22.md
    Board:     #3667–#3672
    Base:      0dcfca959   Tip: see §9
    Predicted reach: 0

---

## 0. The gap, in one sentence

The provenance census can say **whether** a constant is tagged. It cannot say
whether the tag is **TRUE** — and **every other control this repo owns catches
a fabricated NUMBER, while this defect class fabricates none.**

`#3643`: a `PROV[R]` marker in `crates/c2-core/src/codegen/mop.rs` said the
port emits *"71 distinct opcodes … the other 589 … 24 of c2's 109 forms"*.
The truth is **85 / 575 / 34 of 104**. The marker was well-formed, cited, and
counted as a tag. It was wrong since the file's first commit. The same file
had **575** right one comment away. The wrong figure was copied forward into a
board row and into `DISCLOSURE.md`. **No value moved and none could** — every
one of those numbers is a doc comment, so the identity diff was 0 over 21 rows
throughout.

`#3641` is the same family from the other side: writing prose *about* mark
letters moved a subsystem's own agreement census **9/28 → 13/34**, because the
counter cannot tell an evidence mark from a mention of one.

## 1. Deliverable 1 — `scripts/prose_audit.py`

Six checks that can go RED, one inventory that cannot.

| id | shape | what it can say |
|---|---|---|
| **C1** | **ROW-REF** | a `W-<NAME>-<N>` token **attributed to** `DISCLOSURE.md` that is not a row in it |
| **C2** | **ABSENCE** | prose asserting *no ledger row exists* for an address the ledger cites |
| **C3** | **SELF-COUNT** | a document miscounting a table it itself contains |
| **C4** | **BOUND-COUNT** | a prose number bound by `COUNT[<recipe>]` disagreeing with the recount — plus **C4b**, a binding arithmetically right and attached to no prose |
| **C5** | **MENTION-RISK** | a counted mark token in discussion context (`#3641`) |
| **C6** | **ADOPTED-PATH** | a ledger row's `Adopted into` path that no longer resolves |
| **I7** | **UNBOUND inventory** | *cannot go red* — the numeric claims on provenance surfaces that none of C1–C6 reach, **as a number, on every run** |

Recipes C4 understands: `ledger-rows`, `md-rows:<file>:<regex>`,
`grep:<file>:<regex>`, `rs-consts:<file>`, `rs-marks:<file>:<letter>`,
`rs-array:<file>:<IDENT>`. A recipe that cannot resolve is a **finding**, never
a zero that reads as agreement.

### 1.1 What it explicitly CANNOT see — and this is printed every run

**An audit whose coverage is unstated will be read as total.** That is trap 5
pointed at an auditor instead of at a gate, so the residue is a NUMBER:

    crates/** PROV marker lines (649)         345
    docs/whitebox/DISCLOSURE.md               464
    docs/whitebox/ref/README.md               195
    TOTAL UNBOUND                            1004
    checkable (C1+C2+C3+C4+C6)                529

Plus four things it can never see, named by kind rather than by count:

1. **Any claim of fact about c2's behaviour.** *"c2 reads the field through the
   VI32 reader"* is graded by the image and by the byte judge.
2. **Whether a cited ADDRESS holds the instruction claimed.** C1/C6 grade
   citation *targets*; `addrcheck.py` asks the other question and is
   deliberately not folded in.
3. **Whether a bound count is bound to the RIGHT population.** C4 checks the
   arithmetic of a binding a human wrote. A recipe aimed at the wrong array is
   green and wrong.
4. **`file.md:NNN` staleness** — `doc_cite_audit.sh`'s own stated LIMIT.

> **AND A FIFTH, WHICH IS THIS LANE'S OWN ERROR AND IS NOW PRINTED BESIDE THE
> RATIO.** `unbound : checkable` is **1.9 : 1**, and **those two numbers do not
> share a denominator**: `checkable` is dominated by C1's row-id tokens counted
> over the whole tree, `unbound` is counted over three surfaces. The prereg's
> **P6** predicted ≥ 10× and compared them as if they did. **MISS**, and the
> ratio now carries a paragraph saying it is not a coverage figure.

### 1.2 Three design facts learned by watching it fail

* **A SUPPRESSION CLASS WIDE ENOUGH TO SWALLOW THE CONTROL IS WIDE ENOUGH TO
  SWALLOW THE DEFECT.** The first phrase list carried `"does not exist"`, and
  self-test section [2] went green: the planted fixture's own sentence
  *"W-FAKE-9 — a row that does not exist"* suppressed the finding it existed to
  provoke. It happened a **second** time in section [13], with *"the token is a
  real pre-draft"* one line under the citation it was meant to leave alone.
  Every suppression class now has a control **and** a counter-control (§[14])
  that removes the suppressing device and demands twice the findings.
* **ATTRIBUTION OUTRANKS SUPPRESSION**, and `#3645` is the proof.
  `middle_interfaces.rs:634` reads
  `(WB_READER_FINDINGS.md §3.2 / DISCLOSURE W-EXT-1)` — it names the home
  document *and* falsely attributes the token to the ledger **on one line**.
  Ordered home-first, the one dead citation this lane was dispatched to find
  **disappeared**. Section [15] is that line, reproduced.
* **THE ROW-ID GRAMMAR IS NOT A RESERVED NAMESPACE.** `W-UNW-1` is used **27
  times** as a fixture-family label — `sweep.d/70-framed.py`,
  `differential.rs`, `GAPS.md` beside `W13b`/`W14` — and is a `DISCLOSURE` row
  nowhere. A checker treating every `W-*-N` as a ledger citation reports 27
  false positives and is thrown away on first use. Reported as its own
  **UNATTRIBUTED** class instead, and reported *at all*, because it is `#3641`'s
  shape one level up: two namespaces wearing one spelling.

  > **CORROBORATED BY ACCIDENT, AND THE ACCIDENT IS BETTER EVIDENCE THAN THE
  > INFERENCE.** This lane's `cargo test` at tip went red on
  > `rung_index_is_generated_and_current` because this very rung doc was added
  > mid-run, and the assertion printed `docs/rungs/INDEX.md`'s contents — which
  > contain the line `| 2026-07-30 | W-UNW-1 | [unwind-pdata](…) | 5 |`. So
  > **`W-UNW-1` is a RUNG TAG**, allocated from the same sequence as `W22`,
  > `W25`, `W26`, `W30` — and `crates/c2-harness/tests/rung_registry.rs`
  > *enforces* uniqueness on it. The collision is therefore not sloppiness in
  > one file: it is two namespaces, each with its own registry, each of which
  > believes it owns the spelling `W-<NAME>-<N>`.

### 1.3 Watched failing in BOTH directions

`--self-test`, 16 sections, planted fixtures, **exit read from `$?` and never
through a pipe** (a wave-13 lane read `EXIT=0` off `tee`).

| section | direction |
|---|---|
| [1] | **GREEN** — every planted claim true, whole audit exits **0** |
| [2] | RED — C1, a citation to a row that does not exist |
| [3] | RED — C2, an absence claim the ledger falsifies |
| [4] | RED — C4, a binding whose recount disagrees |
| [5] | the whole audit exits **1** on that tree |
| [6] | RED — C3, a document miscounting its own table |
| [7] | RED — **C4b**, a binding arithmetically RIGHT and attached to no prose |
| [8] | RED — C6, a row adopting into a vanished path |
| [9] | the escaped-pipe control: `\|` inside a ledger cell shears the `Adopted into` column on a naive split, and a naive split is shown to really get it wrong |
| [10] | **nothing to check is exit 3, not exit 0** |
| [11] | a missing ledger is exit 2, never a crash and never a green |
| [12] | the marker grammar is the census's, proved by BEHAVIOUR on a planted string, not by comparing two regex sources |
| [13] | every suppression class fires on its neighbour |
| [14] | removing the suppressing devices doubles the findings |
| [15] | attribution outranks suppression — `#3645`'s line, reproduced |
| [16] | a binding inside backticks is a MENTION — found by the tool going red on its own rung doc, and the counter-control un-backticks it and demands the red back |

## 2. Deliverable 2 — what it found, split by fence

Run at base `0dcfca959`: **3 findings** over 520 checked claims. Run at tip:
**0** over 529.

### 2.1 Inside this lane's fence — REPAIRED

| check | site | claim | truth |
|---|---|---|---|
| **C1** | `crates/c2-reference/tests/middle_interfaces.rs:634` | attributes `W-EXT-1` to `DISCLOSURE.md` | not a row there (`#3645`) |
| **C2** | `crates/c2-reference/tests/middle_interfaces.rs:624` | *"NO DISCLOSURE ROW EXISTS FOR THIS ADDRESS"* | `W-EXCLASS-1` cites `0x10b25e48` (`#3645`) |
| **C3** | `docs/whitebox/DISCLOSURE.md:301` | *"seventeen rows, exhaustively"* | the table had **21** (`#3668`) |

The C3 one is **`#3643`'s defect class turned on the ledger itself**: a false
count inside a provenance document, well-formed and uncounted, in the file
whose whole purpose is to be the register. It is now **22** and pinned by
`COUNT[ledger-rows] = 22`, so the next drift is a red run and not a discovery.

### 2.2 Outside this lane's fence — REPORTED, NOT EDITED

**Nothing was found in `crates/c2-core/src/codegen/**`** that C1–C6 can grade,
which is a statement about the *checks*, not about the code: `#3643`'s numbers
live in prose with no binding, so they sit in **I7's 345 unbound marker-line
claims**, not in a check. **`w-mopfold` owns the repair and this lane owns the
recipes it should bind them with** — routed in §7.

| class | count | where | routed to |
|---|---|---|---|
| **UNATTRIBUTED `W-*-N`** | 30 citations, 4 tokens (`W-UNW-1` ×27, `W-EX-1`, `W-MID-5`, `W-BOGUS-9`) | 16 files incl. `codegen/select.rs`, `comdat.rs`, `lib.rs`, `differential.rs`, `sweep.d/70-framed.py` | coordinator |
| **dated-record citations** | 133 | `BOARD.md`, `ROADMAP.md`, `rungs/`, `WB_*_FINDINGS.md`, the ledger's own `>` boxes | nobody — **correct as written** |
| **`P_*.md` mark ambiguity** | 488 marks, **481 backticked**, 318 in tables / 170 in prose | ten `ref/P_*.md` pages | `w-secported` / coordinator, §4 |

## 3. Deliverable 3a — `#3645`, and both halves landed together

`w-disclose` declined `W-EXT-1` on sight citing `#3626` — `W-INLINE-1`'s
pre-draft held **two wrong addresses in bold for eight days**. That precedent
is honoured rather than waved at: **the promotion condition was registered in
the prereg before the check ran**, and the check is a program against the
pinned image, not a re-reading of the pre-draft.

    CONDITION 1  0x10c1fe40 is a function entry          PASS  93 B, ioin.c
    CONDITION 2  the four 0x10b3d5* share ONE owner      PASS  FUN_10b3d546, getattr.c
    CONDITION 3  bytes at 0x10b3d5c1 are shr ebx,9 /
                 and ebx,7                               PASS  c1 eb 09 83 e3 07,
                                                               byte-identical
    image sha256 c80981c0…a66258 verified at run time    MATCH
    VERDICT: PROMOTE

`work/w-provaudit/verify_wext1.py`, evidence `wext1_verify.txt`.

**Corroboration the condition did not ask for.** `FUN_10c1fe40`'s own first
bytes are `… 8a 11 41 53 89 08 84 d2 79 47` — load `b1`, `test dl,dl`, `jns` —
which is the `0x80` discriminator the port transcribes, read straight off the
entry point.

The row states what is adopted (**the 1/2/3-byte width rule, and only it**) and
separately names three things read and **not** adopted: the value
decomposition, the size index `(v >> 9) & 7`, and the global gate on the
trailing LEB skip. `crates/c2-il`'s `.ex` widths remain a separate black-box
derivation.

Both repairs landed in **one commit** (`b0620b77e`), as `#3645` required.

## 4. Deliverable 3b — `#3641`, and the convention is adoptable on ONE of the two surfaces

**Adopted, on the `crates/` surface.** `provenance_census.py`'s `MARK_RE` and
`BLOCK_RE` now say **a marker inside backticks is a MENTION and is not
counted**.

**The 777 are proved identical, not asserted** — with `--since`, which is what
the charter asked for:

    provenance_census.py --since 0dcfca959
    [R] 100→100  [O] 207→207  [F] 41→41  [S] 88→88  [N] 341→341  untag 220→220
    →tag 0 · →untag 0 · reclass 0 · +new 0 · -gone 0   on every module
    DECOMPOSITION: HOLDS for all 6 classes

Level table base-vs-tip: **0 diff lines** apart from the DIRTY stamp. The cost
was measured before adoption: **0 of the 649 `PROV`/`PROV-BLOCK` tokens in
`crates/` are backticked.** Self-test section [11] carries the control both
ways: a backticked marker beside a bare one counts 1 of 2, and un-backticking
it moves the count to 2 of 2.

### 4.1 REFUTED on the other surface, and the refutation is the finding

`subsys.rs::count_marks` counts literal `[R]`/`[O]`/`[I]` on the ten
`ref/P_*.md` pages — the census `#3641` actually moved. **Measured here:**

| page | marks | backticked | in a table row | in prose |
|---|---:|---:|---:|---:|
| P_COFF | 57 | 57 | 42 | 15 |
| P_SECTION | 53 | 53 | 31 | 22 |
| P_REGALLOC | 49 | 49 | 41 | 8 |
| P_GLOBREGS | 48 | 48 | 30 | 18 |
| P_DAG | 47 | 47 | 38 | 9 |
| P_INLINE | 40 | 38 | 19 | 21 |
| P_ENCODE | 28 | 28 | **0** | **28** |
| P_EH | 41 | 36 | 24 | 17 |
| P_LABEL | 73 | 73 | 50 | 23 |
| P_SYMBOL | 52 | 52 | 43 | 9 |
| **TOTAL** | **488** | **481** | **318** | **170** |

**Neither candidate rule separates a mark from a mention there.** Backticks
cannot: those pages write **481 of 488** evidence marks in backticks, so the
rule would **zero** the census rather than clean it. Position cannot either,
and the counter-example is the page `#3641` was found on — `P_ENCODE.md`
carries **0** marks in table rows and **28** in prose.

**So the `#3641` hazard is still live on the surface it was found on**, any
lane amending a `P_*.md` page still moves that page's agreement cell as a side
effect, and the only convention that would work is a **distinct token** rather
than a delimiter — a migration of 488 marks across ten pages, which is a lane
with a price and is a peer's fence. **Reported, not taken.**

## 5. Deliverable 3c — `#3644`, one sentence

`README.md`:72–77 said the opcode/encoding tables are *"instrument-only …
touch no emit path"* **full stop**. True of the file it named; false one crate
over from the day `W-MOP-1..3` were filed. The paragraph now names `W-MOP-*`
as **on the emit path** and points at the ledger for the discrimination rather
than restating it — which is what its own closing parenthetical already
learned once about enumerations in prose. That parenthetical now records the
second occurrence, one layer up.

## 6. `#1406` — DECLINED, with the price, and the price is now sharp

**Not taken.** `P14` HIT, but a weak one: the fence made it close to
determined and I should have registered the *reason* instead of the outcome.

**Owning the script was not the blocker.** The charter offered `#1406` on the
grounds that this lane owns `provenance_census.py` and is *"the first lane that
could"*. It could not: the test has to live in `crates/`, and all three lanes
have been fenced to **comment-only** edits there. A
`crates/*/tests/provenance.rs` is a new file, not a comment; a `gate.sh` row
was in no lane's fence either.

**So the price is now known and it is not "a lane that owns a `crates/` test
file" in general — it is roughly one hour behind a fence nobody has granted:**
one new `crates/c2-harness/tests/provenance.rs` shelling out to both scripts'
`--self-test`, asserting exit 0, skipping cleanly when `python3` is absent (the
`SKIP: toolchain absent` idiom, so it cannot become a portability failure);
plus the `gate.sh` row if a verdict rather than a test is wanted. **It will
keep not happening until a lane is dispatched with exactly that fence.** Board
`#3672`.

## 7. Routed to peers and the coordinator

1. **`w-mopfold` — `#3643`'s numbers are still unbound.** The repair landed
   (85/575/34/104), but nothing checks it. The recipes exist now and this lane
   cannot write the file. Suggested, one line each, beside the prose:
   `COUNT[rs-array:crates/c2-core/src/codegen/mop.rs:OPCODES] = 85` and
   `COUNT[rs-consts:crates/c2-core/src/codegen/mop.rs] = 91`. Without a
   binding the count sits in I7's residue exactly as it did on 2026-08-22.
2. **Coordinator — `W-UNW-1` and the namespace collision.** 27 citations
   across 16 files, including three `crates/c2-core` files, use the ledger's
   row-id spelling for a fixture family. Either reserve the grammar or rename;
   until then, a grep for `W-*-N` cannot tell a provenance citation from a
   fixture label, and this lane's C1 needs an attribution heuristic to avoid
   27 false positives.
3. **`w-secported` / coordinator — §4.1's table.** The `#3641` hazard is
   unrepaired on the surface it was found on, and the per-page numbers are the
   input to pricing the token migration.
4. **`doc_cite_audit.sh` reports 40 findings at this tip, 0 of them in any file
   this lane touched** (all are `docs/DISCLOSURE.md` vs
   `docs/whitebox/DISCLOSURE.md` in dated rungs). Pre-existing; the base
   comparison is not directly quotable because a run from the main checkout
   walks `.claude/worktrees/` and inflates the ambiguity class 1218 vs 407.

## 8. Prereg grading — 8 HIT, 3 MISS, 1 unscorable, and §0 caps it

`work/w-provaudit/PREREG.md`, committed `f2362025b`, before the first run.

| # | prediction | outcome |
|---|---|---|
| **P1** | ≥ 75 % of C3+C4 disagreements understate, on ≥ 4 disagreements | **UNSCORABLE** — only **1** such disagreement exists tree-wide (`seventeen` vs 21). It *is* an understatement, and so is `#3643`'s 71 vs 85, but n = 1 is not 4 and the prereg said so in advance |
| **P2** | C1 finds `W-EXT-1` and at most two others (range 1–3) | **HIT** — exactly 1 live |
| **P3** | C2 finds the `EX_CLASS_TABLE` one and at most one other (1–2) | **HIT** — exactly 1 live (2 more in dated records) |
| **P4** | C3 finds ≥ 1 FURTHER false self-count beyond the observed one | **MISS** — 0 further. The `seventeen`/21 was the only one, and C3's registered population is one table |
| **P5** | C6 finds zero dead paths | **HIT** — 0 of 27 base / 28 tip |
| **P6** | I7's residue ≥ 10× the checkable population | **MISS** — 1.9:1, **and the comparison was ill-formed**; see §1.1 |
| **P7** | > half the findings sit outside this lane's fence | **MISS** — **all 3** findings sat *inside* it. The out-of-fence material is real (§2.2) but lands in the reported classes, not in findings |
| **P8** | C5 finds ≥ 1 mention-risk among the crates markers (0–4) | **HIT at 0**, registered as a legitimate outcome: the crates surface is clean, which is *why* the convention is adoptable there |
| **P9** | the convention costs **zero** of the 777 | **HIT** — 0 of 649 tokens backticked, `--since` +0 on all six classes |
| **P10** | the `W-EXT-1` condition holds and the row is promoted | **HIT** — 3 of 3, byte-verified |
| **P11** | both `#3645` repairs in one commit | **HIT** — `b0620b77e` |
| **P12** | identity diff 0 lines over 21 rows, self-test green beside it | **HIT** — §9 |
| **P13** | census 777/997 unmoved, `--since` all-zero | **HIT** |
| **P14** | `#1406` declined with a price | **HIT**, weak — see §6 |

**The three MISSes are the useful part.** P7 in particular: I predicted the
audit would mostly indict other people's files and it indicted the two files
this lane was given. P6 was wrong because I compared two numbers with
different scopes — inside the instrument I was building to catch exactly that
kind of error.

**§0 of the prereg caps the score up front**, `w-disclose`'s standard applied:
`#3645`'s two halves and `#3643` were handed to this lane, and the
`seventeen`/21 count was made while reading the charter. None is scored. What
is scored is what I did not know.

## 9. Gates and required-zero

    base 0dcfca959, detached, clean:
      GATE: PASS (HATCH-RED REFUSED) — 18/18 lanes ran and every one of them
      graded a corpus, the sweep graded 19460 of 19556 generated cases and the
      cross graded 90424 of 90812 case-lane cells, with 0 mismatches anywhere

    tip:
      GATE: PASS (HATCH-RED REFUSED) — 18/18 lanes ran and every one of them
      graded a corpus, the sweep graded 19460 of 19556 generated cases and the
      cross graded 90424 of 90812 case-lane cells, with 0 mismatches anywhere

    scripts/gate_identity_diff.sh work/w-provaudit/gate_base.out \
                                  work/w-provaudit/gate_tip.out
      count-bearing rows: 21 base, 21 tip (enumerated, not asserted)
      IDENTITY DIFF: 0 lines over 21 rows — required-zero byte delta HOLDS
      exit 0

    scripts/gate_identity_diff.sh --self-test        exit 0

**The verdict line is quoted, never the exit code** — `gate.sh` prints
`REFUSED` and exits 0. `HATCH-RED REFUSED` is present at **both** arms and is
`#1389`'s standing stale-hatch refusal, not a property of this lane.

The only `crates/` bytes this lane changed are **comment-only, in one file**,
`crates/c2-reference/tests/middle_interfaces.rs`.

### 9.1 `cargo test --workspace`

    base 0dcfca959   59 targets   1928 passed   0 failed   1 ignored
    tip              59 targets   1928 passed   0 failed   1 ignored

Both arms run in this worktree, back to back, with the toolchain present. The
base figure independently reproduces `docs/STATUS.md`'s wave-14 close
(`tests 1928/0/59`).

> **THE FIRST TIP RUN WENT RED AND IT WAS THIS LANE'S OWN DOING — recorded
> because a red that is explained away without being named is how a real one
> gets missed.** `cargo test` aborts at the first failing target, so it stopped
> at **37 targets / 1096 passed / 2 failed**. Both failures were
> `rung_registry.rs` and both were caused by *this rung doc*, created while the
> run was in flight: no `Tag:` in its header block, and `docs/rungs/INDEX.md`
> stale because it is GENERATED. Fixed by giving the header the template's
> fields and running `scripts/gen_rung_index.sh`; `rung_registry` is 2/2 green,
> and the figure above is a clean re-run, not a repaired transcript.
>
> **That red also produced this lane's best piece of evidence** — the
> assertion printed `INDEX.md`, which is where `W-UNW-1` turned out to be a
> rung tag (§1.2, `#3671`).

### 9.2 The other instruments, at tip, `$?` read directly

    scripts/prose_audit.py                     exit 0   CLEAN over 529 claims
    scripts/prose_audit.py --self-test         exit 0   16 sections
    scripts/provenance_census.py --self-test   exit 0   11 sections
    scripts/gate_identity_diff.sh --self-test  exit 0
    scripts/board_audit.sh                     exit 0

`scripts/doc_cite_audit.sh` reports **40** findings at this tip and **0 of them
name any file this lane touched** — all are `docs/DISCLOSURE.md` (no
`whitebox/`) in dated rungs. Pre-existing. The base figure is not directly
quotable beside it: a run from the main checkout walks `.claude/worktrees/`
and inflates the `ambiguous basename` class 1218 against 407, so the two runs
have different scopes. Named rather than quoted, which is the same discipline
§1.1 applies to this lane's own ratio.

## 10. Follow-ups deliberately NOT taken

* **Binding `#3643`'s counts in `mop.rs`** — peer's fence, routed (§7.1).
* **The `P_*.md` token migration** — 488 marks, ten pages, peer's fence (§4.1).
* **Reserving the `W-*-N` namespace** — needs a decision, not a lane (§7.2).
* **`#1406`** — priced, not taken (§6).
* **Extending C3 beyond one registered population.** C3 knows exactly one
  self-counting table today. Generalising it means guessing which table a
  `"N rows"` refers to, and a guess that is wrong is a false finding in the one
  instrument whose whole value is that its findings are true.
* **C4 recipes for the `subsys` metric pages** — `w-secported` is editing
  them this wave; binding a peer's live numbers would redden for reasons that
  mean nothing (`#3641`'s own rule about pinning live tree counts).
