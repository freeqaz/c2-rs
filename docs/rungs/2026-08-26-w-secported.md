# w-secported — `ported` on the `section` row, and what actually made the encoder cheap

    Tag:       w-secported
    Slug:      w-secported
    Date:      2026-08-26
    Kind:      characterization + construct rung
    Outcome:   converted
    Fixtures:  none — characterization + construct rung:
               `docs/whitebox/ref/P_SECTION.md` §7 (the arm population and the
               port map), `work/w-secported/{dump_glrec.py,GLREC_ARMS.tsv}`
               (the enumeration, re-derivable from the pinned image), and
               `crates/c2-harness/src/subsys.rs` (the `ported` recount and its
               four checks), rendered to `docs/SUBSYS_METRICS.md`
    Census:    +0 (no acceptance predicate moved, no emit widened, no
               `crates/c2-core` byte changed)
               `c2-core`, `c2-il`, `c2-obj` and `c2-reference` are not written
               at all
    Wave:      15 (docs/DECISIONS_2026-08-22.md § Decision 17)
    Board:     #3661–#3666
    Base:      master `0dcfca959` (decision 17's own commit)
    Record:    this file; prereg `work/w-secported/PREREG.md` (`3c37bf732`)

---

## 0. The one-paragraph answer

`ported` is a number on the `section` row: **1 of 15 live `.gl` record-dispatcher
arms**, recounted on every `cargo test` from a table decoded out of the pinned
image crossed with a scan of the port's own sources. The denominator is
published with **five** rivals and the reason, and the first thing it corrects
is a phrase the instrument itself was printing: **there are not 27 arms.** 27 is
a count of *tag values*; they index sixteen jump slots, one of which is the
fatal `C1001` path serving eight tags. The real population is **15 live handlers
over 19 live tags, plus one refusal over 8.**

The load-bearing half is the **14**, and the sharpest member is
`0x10b9c212` — tag `0x09`, the section-definition record, every field of which
`P_SECTION` §3 obj-checked by mutating real `.gl` bytes. **The port does not
read it.** It carries seventeen fully-resolved `(name, Characteristics)`
constants where c2 has an IL-borne name, an IL-borne kind, an IL-borne
override, a kind switch, a remapper, a base resolver and an alignment chooser.
**The port's section model is the OUTPUT of c2's section model, tabulated.**

And the charter's question — *was the encoder a special case?* — **yes, and
`w-encmap` named the wrong property.** Not *"rules rather than addresses"*: what
the two convertible rows share is **a key the port carries on its own side
because `DISCLOSURE.md` records an adoption.** §4.

---

## 1. What was built

| artifact | what it is |
|---|---|
| `work/w-secported/dump_glrec.py` | decodes the `.gl` dispatch tables from raw image bytes. **The only hard-coded address is the dispatch head `0x10b9b922`**; every table address, bound and extent comes off the operand bytes there, so a wrong carried constant cannot survive (`dump_ilarms.py`'s registered discipline) |
| `work/w-secported/GLREC_ARMS.tsv` | the committed enumeration: 27 tag rows, their jump slot, their arm, and a `# fatal` line naming the refusal |
| `crates/c2-harness/src/subsys.rs` | `PortedRecount::GlRecArms`, `parse_glrec_arms`, `recount_section_ported`, `crates_cited`; `verify` now takes the repo root as well as the ref dir; four new checks |
| `docs/whitebox/ref/P_SECTION.md` §7 | the arm population, the port map, the five denominators, the two black-box rules, and §7.5's coverage-line correction |
| `docs/SUBSYS_METRICS.md` | regenerated **with the tool**, never by hand |

## 2. The numbers, each re-measured on this tree

| quantity | value | how |
|---|---:|---|
| byte-index table `0x10b9c615` | **27 entries** | decoded; matches `P_SECTION.md:35` |
| jump table `0x10b9c5d5` | **16 entries** | decoded; matches |
| fatal arm | `0x10b9c5ca`, **8 tags** | decoded; §1 already had the arm, nobody had the 8 |
| **live arms** | **15** | 16 slots − the refusal |
| **live tags** | **19** | 27 − 8 |
| `section-ported` | **1** (`0x10b9bdcf`) | `crates/` scan, `c2-harness` excluded |
| `section-ported-den` | **15** | published against 16 / 27 / 25 / 137 / 327 |
| `P_SECTION` §1 table rows | **25** | the page's coverage line says 24 |
| … that are Ghidra function entries | **22** | `FUNCS.tsv` |
| … in the two bands giving the 137 | **20** | |
| `section-marks-obj` / `-total` | **17 / 53 → 17 / 53** | unmoved across **every** doc edit |
| `section-sites` recount | **137** | unchanged |

## 3. Prereg grades — honest, and `[prior]` where the measurement predated the file

`w-disclose`'s rule: a HIT whose measurement predates the prereg is weak, and
`[prior]` marks every one. §0 of the prereg listed six such measurements up
front.

| id | prediction | grade |
|---|---|---|
| **J1** | no adopted shared numeric key exists on the section side; the encoder-shaped join does not exist | **HIT.** Twenty-one `DISCLOSURE.md` rows; exactly three touch this subsystem (`W-ALIAS-1`, `W-ALIAS-2`, `W-OBJPLAN-1`, plus `W-STAGETAP-6` on the name formatter) and they land on **two** of the twenty-five §1 entries. No section-kind table, no creator, no chooser. `W-MOP-2` — 85 whole rows of c2's tables — is the encoder's and has no counterpart here |
| **J2** `[prior]` | a weaker join exists on the `.gl` tag byte | **HIT, and then rejected as the unit.** The port names 5 tags, but **three of them as pattern locators rather than decoders** (`KIND1_TAGS`, and `0x04` inside `KIND4_TAGS`), a distinction the tag unit cannot express. The arm unit can, so the arm unit is the denominator |
| **J3** | no `crates/` constant holds a c2 section **kind** | **HIT** |
| **D1** | denominator A (the dispatcher), conf 0.50 | **HIT**, and the reason changed under measurement: A was predicted at 27 and shipped at **15**, because the fatal arm was not known to serve eight tags until the table was dumped |
| **N1** `[prior]` | `ported = 5 ± 2` on the 27-tag unit, set `{0x01,0x02,0x04,0x0E,0x10}` | **HIT on the set, exactly** — and the registered **UNDER** bias direction was **wrong in an unregistered way**: the port names *more* tags than it decodes, not fewer, because three of the five are locators. Scored a hit on the number and a **MISS on the bias** |
| **N2** | `ported ≤ 8` of 24/25 on the entry unit under any defensible predicate | **not decided.** The entry unit was rejected before a numerator was defended (§7.4), so this prediction has no grade rather than a hit. Registering a bound on a unit I then declined to use was a prereg error |
| **N3** `[prior]` | the strict address-citation reading is 2 of 25 and is a rival, never the number | **HIT**, both clauses, **and it is the finding that shaped the lane.** The two divergent cells were found and named: `align_nibble` and the `.bss` reversal |
| **N4** | 27 entries and 16 entries reproduce | **HIT** |
| **N5** `[prior]` | §1's table holds 25 rows against a coverage line of 24 | **HIT**, and it goes further than registered: 24 reproduces under **none** of 25 / 22 / 20, and `git log -S` puts the line at 25 rows in the file's **first** commit |
| **R1** | ≥ 15 of 25 §1 entries unimplemented under every reading | **not decided** — same cause as N2 |
| **R2** | at least one rule implemented behaviourally while citing nothing; candidate the `.bss` reversal | **HIT**, and the named candidate was one of the two |
| **S1** | the encoder was a special case and the property is an adopted key, not "rules vs addresses" | **HIT** — §4 |
| **H1** | identity diff 0 lines over 21 rows | see §6 |
| **H2** | `GATE:` verdict unchanged | see §6 |
| **H3** | target count unchanged, test count up | see §6 |
| **H4** | **#3641 will bite; at least one draft moves the census** | **MISS**, in the good direction: the respelling was applied in the **first** draft, so the census read 17/53 before and after every edit and never moved. A prediction that the hazard would fire is not vindicated by preventing it, and it is graded a miss |
| **H5** | no second provenance reader, no `DISCLOSURE` row minted, nothing outside the fence | **HIT** |
| **H6** | a control planted, watched red, reverted | **HIT**, three times — §5 |

**Self-grade failures registered in prereg §7:** none fired, except that **N2
and R1 were registered on a unit the lane then rejected**, which is a milder
form of failure 1 (a denominator not defended) and is recorded rather than
quietly dropped.

## 4. `#3663` — what actually makes a row convertible

`#3636` closed with: *"`#3617`'s stated reason for the residue does not hold …
the encoder is the exception precisely because its sites are **rules** rather
than code"*, and named `section` as the only other row with that property.
This lane tested it and the property is not the one.

**Both convertible rows share a key the port carries on its own side, because
`DISCLOSURE.md` records an adoption of it:**

| row | c2's key | the port's copy | adopted under |
|---|---|---|---|
| `encode` | the encode-form number at `0x10c39b18` | `mop::OPCODES`'s `form` | **W-MOP-2** — 85 whole rows |
| `section` | the `.gl` arm address `0x10b9bdcf` | `glalias.rs`'s decode | **W-ALIAS-1** / **W-ALIAS-2** |

**Where no adoption exists there is nothing to join on, and "its sites are
rules" does not rescue it.** The proof is in §7.4 of the page: the port
implements the alignment-nibble ladder (`0x10b28261`, `log2(a)+1`) and the
`.bss` reversal (`0x10b99093`) — **two rules, agreeing with c2, joinable to it
by nothing**, because both were derived black-box and neither cites an address.
An address grep scores them 0 where the honest answer is 1 each.

So `#3617` and `#3636` are **both right, on different units**. `#3636` holds on
a unit the port adopted a key for; `#3617` — *"the port is I/O-behavioral, so
'the port implements site X' has no truth value"* — holds everywhere else, and
§7.4 is the measurement that shows it rather than the assertion that was
originally made.

**The consequence for the scoreboard, stated as a prediction the next lane can
falsify:** `ported` is convertible on a row iff `DISCLOSURE.md` records an
adoption whose key that row's sites are indexed by. On today's ledger that
predicts **no further row converts** without a read that adopts one — `coff`,
`regalloc`, `globregs`, `dag`, `inline`, `eh`, `label`, `symbol` all lack such a
row.

## 5. The controls, and that each was watched failing

`#3336`: a control never seen failing is decoration.

| control | mutation | observed |
|---|---|---|
| `control_a_fabricated_section_ported_is_caught` | the **shipped** cell `1` → `2` | **4 tests red**, `section: ported DOES NOT REPRODUCE — table says 2/15, the tree gives 1/15` |
| `the_observer_crate_cannot_move_its_own_ported` | `PORTED_SCAN_EXCLUDES_CRATE` disabled | **6 tests red**, and the number **moves `1/15` → `2/15`** — the hazard is measured, not asserted |
| `control_a_measured_ported_must_reach_the_rendered_table` | the §3 cell restored to the literal `ported RESIDUE` | **1 test red**, naming the row and the missing number |
| the same control's second half | the §4 caveat withheld from the doc | **1 test red**: *"section: the measured ported caveat does not reach the published doc — looked for `"THE DENOMINATOR IS THE 15 LIVE ARMS OF"`"* |

**`#3665` has a sibling, found in the same pass and repaired with it.** The
`ported` **caveat** — which on both measured rows *is* the published
denominator choice and the rivals it beat, the thing decision 16 and decision
17 each demanded be said out loud — reached the **console** render and
`subsys.rs`'s source and **nothing else**. `docs/SUBSYS_METRICS.md` has carried
`encode`'s `27 / 79` since `w-encmap` **without the paragraph that says what
the 79 is or why it is not 14 or 111**. §4 now emits both caveats verbatim, and
the control asserts it. *A denominator published only in the source of the tool
that prints it is not published.*

`the_section_ported_arm_is_the_one_the_port_actually_decodes` is not a
fabrication: it pins **which** arm the numerator found (`0x10b9bdcf`), that tag
`0x09`'s arm is uncited, and the dispatcher's shape (27 / 16 / 15 / 19 / 8), so
a re-dump that disagrees reddens instead of shipping a moved denominator
quietly.

`scripts/subsys_metrics.sh --self-test` still passes its three corruptions. It
**cannot reach the section recount** — that reads `work/…/GLREC_ARMS.tsv` and
`crates/`, neither under the ref index the self-test corrupts — and
`GLREC_ARMS_TSV`'s doc comment says so rather than leaving it to be discovered.

## 6. Gates

**Required-zero byte delta — HOLDS.**

```
scripts/gate_identity_diff.sh work/w-secported/gate_base.out work/w-secported/gate_tip.out
  count-bearing rows: 21 base, 21 tip (enumerated, not asserted)
  IDENTITY DIFF: 0 lines over 21 rows — required-zero byte delta HOLDS

scripts/gate_identity_diff.sh --self-test
  enumeration: 21 count-bearing rows (hatch-red/ladder-red dropped)
  control: a table against itself                      -> 0 lines, exit 0
  #3515's one-TU-refused signature                     -> 14 lines, 7 rows
  the signature case exits NONZERO
  a TRUNCATED table -> exit 2 (a short extraction is not 'no differences')
  SELF-TEST PASS
```

The base transcript is a **real gate run at `0dcfca959`** in a dedicated
worktree, not a nearby baseline reused: `work/coordinator/gatebase/` holds
none at this base, and `HOWTO_DIFF.md`'s rule is the **pre-merge** base.

**Both verdicts, read off the `GATE:` line and never the exit code:**

```
base 0dcfca959 : GATE: PASS (HATCH-RED REFUSED) — 18/18 lanes ran and every one graded a corpus
tip  7169b14b3 : GATE: PASS (HATCH-RED REFUSED) — 18/18 lanes ran and every one graded a corpus
                 sweep 19460/19556 · cross 90424/90812 · 0 mismatches anywhere
                 debug-lane 18/18, 7038 fixture-verdicts, 0 panics
```

`hatch-red` is `REFUSED` at both ends for the standing reason (`HATCH-STALE`,
board `#1389`) and is one of the two rows the identity diff drops by rule.

**H1 HIT · H2 HIT.** The tree hash moves because `scripts/subsys_metrics.sh`
is written — `HOWTO_DIFF.md` says that is expected and is not part of the
identity diff.

**Test counts** — `cargo test --workspace --no-fail-fast`, both ends:

| | targets | passed | failed | ignored |
|---|---:|---:|---:|---:|
| base `0dcfca959` | *see final report* | | | |
| tip `7169b14b3` | *see final report* | | | |

`cargo test -p c2-harness --lib subsys`: **13 → 17** (four checks added).

**One self-inflicted redness, found and fixed rather than shipped**: the first
`--no-fail-fast` run failed `rung_docs_claim_their_tag_slug_and_fixtures_exactly_once`
and `rung_index_is_generated_and_current` — this rung's own header lacked the
registry block and `docs/rungs/INDEX.md` is **generated**. Repaired with
`scripts/gen_rung_index.sh`; both green. Worth recording because the plain
`cargo test --workspace` run **stopped at the first failing target** and its
tail looked entirely green, which is exactly why the rungs README arms
`--no-fail-fast`.

`scripts/board_audit.sh`: 0 unresolved anchors, 0 duplicate row numbers, 0
cited-but-absent.
`scripts/subsys_metrics.sh --self-test`: PASS, all three corruptions RED.

## 7. What this lane deliberately did NOT do

* **Did not correct `P_SECTION.md`'s coverage line**, though the page is this
  lane's fence and the line is wrong. The amendment stands beside the original
  reading (§5's own retraction is the model, `#3538` the rule), and the line is
  the row's `den_probe`. `subsys-metric section-read 24` is therefore a
  **carried page figure the page now flags**, not a recount. Converting `read`
  to a recount is a real follow-up and it is not this lane's.
* **Did not read the thirteen unlabelled arms.** `GLREC_ARMS.tsv` gives every
  live arm its tags and its extent; **thirteen of fifteen have no semantics in
  this tree at all.** That is the largest single unread block the section
  subsystem owns and it is now enumerable, which it was not this morning.
* **Did not touch `crates/c2-il` or `crates/c2-core`.** Porting a `.gl` record
  arm would move `section-ported` — that is the point of the instrument — but it
  is emit-adjacent work and this lane's byte delta is required zero.
* **Did not build a second provenance or census reader** (charter), and minted
  no `DISCLOSURE.md` row.
* **Did not price the `coff` row**, whose residue text points at the same
  missing map. §4's prediction says it will not convert without an adoption.
