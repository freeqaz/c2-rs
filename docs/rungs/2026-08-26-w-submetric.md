# w-submetric — the per-subsystem scoreboard is BUILT, and the strength decision 15 named FIRST is the one no instrument in this tree can reach

    Tag:       w-submetric
    Slug:      w-submetric
    Date:      2026-08-26
    Kind:      instrument rung
    Outcome:   instrument
    Fixtures:  none — instrument rung: `crates/c2-harness/src/subsys.rs`,
               `crates/c2-harness/src/cli/subsys.rs`, `scripts/subsys_metrics.sh`,
               rendered to `docs/SUBSYS_METRICS.md`
    Census:    +0 — no acceptance predicate moved, no emit widened, the admitted
               set is untouched. `crates/c2-core`, `c2-il`, `c2-obj` and
               `c2-reference` are not written at all.
    Base:      master `6c753ead0` (decision 15's own commit)
    Prereg:    `work/w-submetric/PREREG.md`, the FIRST commit on
               `wt-w-submetric` (`005fce256`), before any deliverable measurement
    Board:     #3617–#3622
    Spec:      `docs/DECISIONS_2026-08-22.md` § Decision 15 (owner) · `#3616`
    Workload:  878-TU workload; the section census this lane reads is the
               committed `work/w-bss/census/sections.jsonl`, **871 records**,
               `.prov` stamp `2026-08-04T10:06:18Z`, corpus `../dc3-decomp`
               `940d07dcb0960964ad61aa5f025658f993eb46b2`, `dirty=false`,
               `data_sha256 fb2f5865…`. Recomputed here: 871 TUs, 393,236
               sections, 14 distinct names, `nsec-disagree 0` (known answer 0).

---

## 1. RESULT

Decision 15 restructured the working goal onto per-subsystem scoreboards and funded
this lane to build the instrument they are tracked with — *"without it the new goal
is prose"*. The instrument exists, verifies itself against the tree on every
`cargo test`, and its four controls have been watched going red.

**The finding is in the first strength.** Decision 15's strength 1 asks *"of the
subsystem's enumerable sites in the image, how many the port implements"*. **No
port↔image site map exists in this tree for any of the ten subsystems, and for most
of them the question is not well formed** — the port is I/O-behavioral by
construction (`CLAUDE.md`'s one correctness rule: it may use AVX and restructured
CFGs so long as its *output obj* matches), so *"the port implements `0x10b2e7f8`"* has
no truth value for most of these addresses. Strength 1 therefore ships as a
**containment** — `sites ⊇ read ⊇ ported` — with `ported` a **named residue on all
ten rows**, each carrying its own reason rather than a blank.

That is not a hedge, it is the deliverable working: the alternative was a numerator
invented to fill a column, which is the failure mode this repo has priced repeatedly.

### The tuple table

Committed, generated, at [`docs/SUBSYS_METRICS.md`](../SUBSYS_METRICS.md). Summary
(read every number with its denominator; §3 of that file carries them all):

| subsystem | 1 read `sites ⊇ read ⊇ ported` | 2nd denominator (TU-level) | 2 agreement (page marks `[O]`/total) | 3 exercised | 4 byte-owned |
|---|---|---:|---|---|---|
| `coff` | 120 ⊇ 21 ⊇ RESIDUE | 129 (1.1×) | 16 / 57 = 28.1 % | proxy 871/871 TUs | CITED `#3534` |
| `section` | 137 ⊇ 24 ⊇ RESIDUE | 327 (2.4×) | 17 / 53 = 32.1 % | proxy 14/14 names | CITED `#3534` |
| `regalloc` | 70 ⊇ 33 ⊇ RESIDUE | 230 (3.3×) | 7 / 49 = 14.3 % | RESIDUE | CITED `#3534` |
| `globregs` | 19 ⊇ **26** ⊇ RESIDUE | — | 2 / 48 = 4.2 % | RESIDUE | CITED `#3534` |
| `dag` | 61 ⊇ 32 ⊇ RESIDUE | 83 (1.4×) | 6 / 47 = 12.8 % | RESIDUE | CITED `#3534` |
| `inline` | 93 ⊇ 16 ⊇ RESIDUE | 350 (**3.8×**) | 10 / 31 = 32.3 % · **PENDING `w-inlmetric`** | RESIDUE | CITED `#3534` |
| `encode` | 14 ⊇ **79 arms** ⊇ RESIDUE | — | 9 / 28 = 32.1 % · **630,548 / 634,457 words = 99.38 % `[O]`** | proxy 863/871 TUs | CITED `#3534` |
| `eh` | 47 ⊇ 19 ⊇ RESIDUE | 127 (2.7×) | 14 / 41 = 34.1 % | proxy 849/871 TUs | CITED `#3534` |
| `label` | 163 ⊇ 163 ⊇ RESIDUE | — | 11 / 73 = 15.1 % | RESIDUE | CITED `#3534` |
| `symbol` | **5** ⊇ **27** ⊇ RESIDUE | 5 (1.0×) | 4 / 52 = 7.7 % | proxy 675/871 TUs (**cited**) | CITED `#3534` |

`globregs` and `symbol`'s numerators exceed their denominators. That is not a bug in
this table — it is §3 below.

---

## 2. WHAT WAS RE-MEASURED, AND WHAT THE RE-MEASUREMENT FOUND

The coordinator's brief said explicitly it had verified none of its figures. Every
one was re-taken on this tree.

### 2.1 All seven band denominators reproduce — and the ten pages do **not** share an endpoint convention (`#3618`)

Recounted from `docs/whitebox/ref/FUNCS.tsv` (4,917 function rows), not carried:

| subsystem | band(s) | page says | recount | convention that reproduces it |
|---|---|---:|---:|---|
| `coff` | `0x10b281af`–`0x10b2b0dd` | 120 | **120** | **inclusive** (119 half-open) |
| `section` | `0x10b97dfb`–`0x10b9b8e9` + `0x10be71c9`–`0x10be7e81` | 137 | **102 + 35 = 137** | **inclusive** (101 + 34 half-open) |
| `regalloc` | `0x10b2c21d`–`0x10b3219f` | 70 | **70** | **half-open** (71 inclusive) |
| `dag` | `0x10b3219f`–`0x10b3433f` + `0x10be5cce`–`0x10be663f` | 61 | **48 + 13 = 61** | either |
| `inline` | `0x10b5b86d`–`0x10b62b00` | 93 | **93** | either |
| `encode` | `0x10bf96d0`–`0x10bfae2a` | 14 | **14** | either |
| `eh` | `0x10be04e7`–`0x10be3800` | 47 | **47** | either |

**Seven of seven reproduce, and two of them only under one convention each — and it
is a different convention for each.** `P_REGALLOC`'s 70 needs the high end excluded
(`0x10b3219f` is `dag.c`'s own anchor, so the exclusion is *correct* and the page is
right); `P_COFF`'s 120 needs it included. A future lane recounting these without
recording the convention would report two "discrepancies" that are not there. The
convention is now carried per row as data (`Band::end`), and
`the_endpoint_convention_is_load_bearing` asserts both directions so flipping either
one reddens.

`globregs` (19 = the R4 target plus its 18 callees) and `label` (163 = 31 direct
allocator call sites + 132 constructor sites) have **no band at all**; they are
verified by requiring their page still carries the sentence the number came from.

### 2.2 Every subsystem has TWO defensible site denominators and they differ by up to 3.8× (`#3620`)

`FUNCS.tsv` carries a `subsys` column, and it is **not** the band. `build_ref.py`'s
`TU_PAGE` assigns it per **translation unit**, so `inline` there is all of
`inline.c` + `ptinl.c` = **350** functions where `P_INLINE`'s band is **93**.

| subsystem | band | TU-level | ratio |
|---|---:|---:|---:|
| `inline` | 93 | 350 | **3.8×** |
| `regalloc` | 70 | 230 | 3.3× |
| `eh` | 47 | 127 | 2.7× |
| `section` | 137 | 327 | 2.4× |
| `dag` | 61 | 83 | 1.4× |
| `coff` | 120 | 129 | 1.1× |
| `symbol` | 5 | 5 | 1.0× |

Both are defensible and neither is wrong. **"The inliner is 17 % read" and "the
inliner is 4.6 % read" are the same measurement under two denominators**, and a
scoreboard that published one of them silently would be `decode-reach`'s frame-vs-model
lesson repeated one layer up — *"quote them together or neither"*. The table
publishes both. Three subsystems (`globregs`, `encode`, `label`) have **no** TU-level
value at all, because `build_ref.py`'s `PAGE_SUBSYS` has no entry for their pages.

### 2.3 Five of the ten `SUBSYS.md` §1 cells are not fractions, and one number reproduces nowhere (`#3619`)

Reported, **not corrected** — `SUBSYS.md` and the `P_*.md` pages are not this lane's
to edit, and a disagreement recorded beside a page beats a silent rewrite of it
(`#3538`'s own rule, applied to the directory `#3538` is about).

* **`symbol` — `27 / 5`, a ratio greater than 1.** The numerator is *addresses*, the
  denominator is *functions*. Recounted here, the page's own address band
  `0x10b28a9b`–`0x10b28d6f` holds exactly **one** Ghidra function entry, so there is
  no band reading under which `5` is a function count of that span: the 5 is
  `FUN_10b28a9b` plus four callees that live elsewhere in `coff.c`'s gap.
* **`encode` — `14 / 14` is the BAND, and the page's coverage line is `79 of 79
  arms` covering `660 of 660` opcodes.** Both correct, different units. A reader
  taking `14 / 14` as the coverage statement is off by 5.6× in the numerator.
* **`globregs` — the read (26) is LARGER than its own denominator (19)**, and the
  page says why in its own words: the read went outside the registered denominator
  deliberately, because the three functions that decide the order are not callees of
  the target at all. `SUBSYS.md`'s cell prints no denominator whatsoever.
* **`regalloc` — `33 / 70` mixes units**: the 33 is 18 code entries + **15 data
  entries**, and data entries are tables, not functions.
* **`label` — `163 sites / 86+25 callers`. The `25` reproduces NOWHERE on
  `P_LABEL.md`.** The 86 does (`All 132 are direct E8 calls from 86 distinct
  functions`, :445/:471). The nearest live figure is **85** — the *placement*
  population calling `FUN_10bd415e` (:505) — and the only literal `25` on the page is
  `fitted from 25 TUs` at :222, in an unrelated sentence about `OBJ_GY_SHAPES`.

**The generalizable half**: `SUBSYS.md` §1's column is headed `entries / band`, and on
half its rows the two halves are not commensurable. It is a *navigation* column and it
reads like a coverage fraction. That is why this instrument recomputes rather than
carries.

### 2.4 Per-site exercise is unmeasurable for all ten, and the proxy that exists is an OUTPUT proxy (`#3621`)

**Nothing in this tree traces `c2.dll`'s own addresses over the workload.** No row can
say which of `P_INLINE`'s 93 functions the workload entered. Strength 3 therefore
ships as a **labelled workload-output proxy** where one exists and a named residue
otherwise, with `OUTPUT PROXY, NOT A SITE COUNT` printed in the cell itself — five
rows measured, five residue.

Two of the five measured proxies are **100 % by construction** (`coff` 871/871 —
every obj went through the writer; `section` 14/14 — the denominator is the observed
set) and say so in their own caveat. That is deliberate: a proxy whose ceiling is
structural is worth printing *with* the reason it is pinned, and worth nothing
without it.

**The one that earned its place**: `eh`'s census counts `.xdata$x` in exactly **67 of
871** TUs, which **independently reproduces `P_EH.md`'s own *"67 workload objs, all
STLport"*** from a different instrument on a different route. That is a corroboration
the page did not have, and it is asserted in
`the_section_census_control_holds_or_the_file_is_absent` so it cannot silently drift.

The census parser carries its own known-answer control: every record states `nsec`,
and the parsed `order` list must have exactly that many entries. **871 of 871, zero
disagreements** — a parser that dropped names would fail the control rather than
report a plausible small number.

### 2.5 `agreement` — what could be measured, and the caveat that rides with it

For nine of ten subsystems **no differential against the read spec exists**, and that
prints as a named residue in those words. What every row *can* carry is the page's own
**evidence-mark census**: `[O]` (obj-confirmed) against `[R]`+`[O]`+`[I]`, under the
mark definitions in `whitebox/ref/README.md` §2. It is uniform across all ten pages
and mechanically recomputable, and it spreads from **4.2 %** (`globregs`, 2 of 48) to
**34.1 %** (`eh`, 14 of 41), so it is not degenerate.

**It is not a differential and the doc says so in the cell.** A mark is a page
annotation, not a site: a page may mark one sentence `[O]` and cover twenty addresses
with it. It is published because the alternative was ten rows of silence, and §0's
rule is that a strength with no data prints a *name*, never a zero.

One row carries a real differential and it is quoted with **its own** denominator, not
the row's: `encode`'s **630,548 of 634,457** executable `.text` words explained by the
page's arm masks (**99.3839 %**, `P_ENCODE` §8.2), over **500** `dc3-decomp` objs —
*not* the 878-TU workload. The caveat carries the page's own warning verbatim: its
second, generous pass reads 99.8060 % and **must not be quoted as stronger**, because
sixteen VMX128 forms are masked at `0x03FFFFFF` and a generous mask cannot fail.
`inline`'s cell prints `PENDING — w-inlmetric`, cited and not waited on; that lane's
worktree was not read.

### 2.6 byte-owned is CITED and was not re-measured

Decision 15's own instruction, and `#3534` measured it on 2026-08-25 at port tree
`a8593651b`: the port's wrong bodies are 1,968 bodies / 7,912 substituted words,
**99.87 % opcode**, **0 pure reorderings**, **92.78 % wrong at word 0**
(`docs/DIFF_STRUCTURE.md`, `docs/PERMUTER_POPULATION.md` §3). Re-taking it is what
this repo calls *"check the board before dispatching"*.

**And the honest addition this lane owes the column**: `#3534` measured the *shape* of
the port's wrong bytes **workload-wide**. It did not attribute a single byte to
`coff.c` rather than `color.c`, and **no instrument in this tree does**. So byte-owned
is one cited figure repeated across ten rows, with a residue naming what an attributed
column would need — the port's emit path carrying a subsystem tag from
`codegen::select_function` to the COFF writer. Printing a per-subsystem byte-ownership
number today would have meant inventing one.

---

## 3. THE CONTROLS, WATCHED FAILING (`#3622`)

`#3336`: **a control never seen failing is decoration.** Two layers, both watched.

**Layer 1 — `cargo test -p c2-harness --lib subsys`, four fabrications**, each
asserting the verifier *refuses* and each pinned to the check that must own the
refusal, so a case cannot pass by being caught for the wrong reason:

| control | fabrication | must be caught by |
|---|---|---|
| `control_a_fabricated_denominator_is_caught` | inliner `93` → `94` | the `FUNCS.tsv` recount |
| `control_a_dropped_subsystem_is_caught` | the `eh` row deleted from the table | the `SUBSYS.md` §1 enumeration |
| `control_an_empty_residue_is_caught` | `dag`'s `ported` residue set to `"   "` | the no-silence check |
| `control_a_moved_coverage_line_is_caught` | `P_COFF`'s probe pointed at a line not on the page | the verbatim probe |

**And the real thing, watched**: `sites: 93` edited to `sites: 94` in the shipped
table reddened **4 of 10 tests** —

```
per-subsystem metric table no longer reproduces:
  inline: band denominator DOES NOT REPRODUCE — table says 94, FUNCS.tsv gives 93 over 1 band(s)
```

captured at `work/w-submetric/control_red_fabricated_denominator.txt` (exit 101), then
reverted and re-run green (10 passed).

**Layer 2 — `scripts/subsys_metrics.sh --self-test`** drives the **binary** against
three deliberately corrupted copies of the reference index and requires each to exit
non-zero through the right check: a function moved out of the inliner band
(→ `DOES NOT REPRODUCE`), `P_EH.md`'s coverage line edited (→ `den_probe not found`), a
subsystem deleted from `SUBSYS.md` §1 (→ `SUBSYS.md §1 has no row`). Captured at
`work/w-submetric/selftest_shell.txt`.

### 3.1 The self-test's own guard fired on me, first run

The first fabrication used address `10b5b901`, which **is not a row in `FUNCS.tsv`**.
The `sed` matched nothing, the "corrupted" copy was byte-identical to the real index,
and the case would have passed by testing the control twice. The mutation-applied
guard — `#3516`'s failure, borrowed verbatim from
`scripts/gate_identity_diff.sh --self-test` — printed

```
  FABRICATION DID NOT APPLY [a function moved out of the inliner band] — the case would test the control twice
```

and exited 3. **This is the guard earning its place on its first run, in a lane that
copied it in only because the precedent said to.** Fixed by using `10b5b9de`, a real
row inside the band.

---

## 4. PLACEMENT — `FUNCTION_BYTE_MATCH.md` §0 against `#1406`

§0 forbids a gradient from entering `gate.sh`; `#1406` binds any instrument quoted as
evidence to run under `cargo test` **or** `gate.sh`. The resolution is
`decode-reach`'s, taken deliberately and stated here as the brief required:

* **The logic and all four controls live in `crates/c2-harness/src/subsys.rs` and run
  under `cargo test --workspace`**, which is a `gate.sh` row. The verdict they
  contribute to is `cargo test`'s — *that every denominator still reproduces from the
  tree* — never the differential's.
* **Nothing prints inside a `c2rs gap` scan.** This is stronger than `decode-reach`'s
  placement and it was chosen for the required-zero constraint: `c2rs subsys` is a
  separate offline subcommand, so it cannot add a `gap-metric` key, cannot move the
  gate's 21-row count table, and cannot reach an accept/refuse path even by accident.
* **Keys are `subsys-metric`**, a namespace of their own, deliberately **not**
  `gap-metric`, so a lane's `gap-metric` key diff is unaffected.
* **Sorted by key NAME, never by mass** (`#3505`; *"ranking instruments measure
  themselves"*, four for four).
* **The instrument licenses no emit**, printed in the disclaimer on every render and
  asserted by `no_strength_prints_a_bare_zero`.

It needs **no toolchain**: it reads `docs/whitebox/ref/` and a committed census and
prints. An absent census makes every output proxy read `NO-DATA`, never `0`.

### 4.1 The one thing the precedent does that this lane did NOT do, and why

`decode-reach-*` and `symbind-*` each added a **dated amend-beside box to
`docs/FUNCTION_BYTE_MATCH.md` §0** announcing themselves as the fourth and fifth
gradients. `subsys-metric` is the sixth and **no box was added**, because
`FUNCTION_BYTE_MATCH.md` is not in this lane's write fence — decision 15 enumerates
`crates/c2-harness/**`, new `scripts/subsys_*`, `docs/SUBSYS_METRICS.md`,
`work/w-submetric/**`, this rung and board rows `#3617`–`#3622`, and that is all.
No peer owns the file either, so this is not a collision; it is a fence I was not
given and did not take.

**Reported rather than done, per the standing instruction** (*a lane that needs a
surface it does not own STOPS and reports*). The box, if the coordinator wants it,
is one paragraph: sixth gradient, lane `w-submetric`, boards `#3617`–`#3622`,
`crates/c2-harness/src/subsys.rs`, §0's five properties adopted verbatim, nothing on
that page edited or re-scored — and the one thing to carry off it is that **this
gradient's `read` strength has two denominators that differ by up to 3.8×**, the same
shape as `decode-reach`'s frame-vs-model split, *quote them together or neither*.
`docs/SUBSYS_METRICS.md` §0 already states the adoption on its own page in the
meantime, so the rule is published; only the cross-link is missing.

---

## 5. REQUIRED-ZERO

*(filled in at §7 with the measured identity diff.)*

---

## 6. PREREG GRADE

Graded against `work/w-submetric/PREREG.md`, committed at `005fce256` before any
deliverable measurement. Predictions were not edited.

| # | prediction | outcome |
|---|---|---|
| **P1** | `ported` a named residue for ≥ 8 of 10 | **HIT** — 10 of 10, at the top of my stated bias direction ("more residue than decision 15's prose implies") |
| **P2** | `agreement` measured for ≤ 3 of 10; inliner prints `pending` | **HIT** — 1 of 10 carries a real differential (`encode`); `inline` prints `PENDING — w-inlmetric`. The mark census then gave all ten a *second, weaker* number I had not predicted at all, and it is labelled as the weaker thing it is |
| **P3** | per-SITE exercise measurable for **0** of 10 | **HIT** — 0 of 10, and it is structural rather than a gap: no address trace of `c2.dll` over the workload exists |
| **P4** | an output proxy for ≥ 3 subsystems | **HIT** — 5 (`coff`, `section`, `encode`, `eh`, `symbol`), though two of the five are 100 % by construction and say so |
| **P5** | ≥ 2 §1 cells in different units, incl. **at least one I had not already seen** | **HIT** — 5, and three (`regalloc`, `globregs`, `label`) were not among the two disclosed in the prereg. `label`'s `25` reproducing nowhere is the one I would not have predicted |
| **P6** | all band denominators reproduce, 10 of 10 | **HIT with a correction to my own framing** — 7 of 7 *band* rows reproduce; the other three (`globregs`, `label`, `symbol`) have no band, exactly the escape my stated bias flagged. I wrote "10 of 10" in a prediction about a quantity only 7 rows have |
| **P7** | band vs TU-level differ > 2× on ≥ 3 | **HIT** — 4 (`inline` 3.8×, `regalloc` 3.3×, `eh` 2.7×, `section` 2.4×) |
| **P8** | required-zero holds, `0 lines over 21 rows` | *(§7)* |
| **P9** | controls watched red on both fabrications | **HIT** — and a third red I had not predicted: the self-test's mutation-applied guard fired on my own bad `sed` (§3.1) |

**Decline floor: not reached.** All ten rows carry a measured strength-1 denominator
recomputed or probe-verified on this tree; the controls were seen failing; and no
strength-2/3 row is silent. The floor's clause about *"a table of ten rows of 'no
differential exists' is prose with a border"* was the live risk and the mark census
plus the five output proxies is what kept it off — reported here because that clause
was written before I knew whether it would bind.

---

## 7. GATE

*(filled in below.)*

---

## 8. WHAT I DID NOT DO, DELIBERATELY

* **Did not re-measure byte-ownership** (`#3534`, decision 15's own instruction).
* **Did not build a provenance-marker counter.** `w-provenance` owns the
  derived-vs-fitted census this wave, and decision 15's fence says owned surfaces
  include *predicates, keys and facts, not just files*. It is also the quantity that
  would most directly convert `ported` from a residue to a number, which is why the
  residues name it.
* **Did not read `w-inlmetric`'s worktree** and did not wait on it.
* **Did not edit any `P_*.md` page, `SUBSYS.md`, or `DISCLOSURE.md`** — §2.3's five
  disagreements are recorded beside the pages, not applied to them.
* **Did not widen emission**, move the admitted set, or add anything to `gate.sh`.
* **Did not build an arm → port-function map for the encoder.** It is the cheapest
  path from `ported: RESIDUE` to a real number on one row — `P_ENCODE`'s 79 arms
  against the port's 89 `encode_*` functions, with §8.1's 82-of-89 base-word
  comparison already done — and it is a lane, not an add-on: §8.1's own text says the
  port's encoders were derived black-box and never from these arms, so the map has to
  be *built*, not extracted. Named, not taken.

## 9. FOLLOW-UPS FOR WHOEVER COMES NEXT

1. **`SUBSYS.md` §1's `entries / band` column needs a unit per cell**, or a reader
   keeps taking it for a coverage fraction on the five rows where it is not one
   (§2.3). Not fixed here because the file is not this lane's.
2. **`P_LABEL.md`/`SUBSYS.md`'s `25 callers` needs an owner.** It reproduces nowhere
   on the page and 85 is the number that looks intended (§2.3).
3. **A subsystem tag through the emit path** is the only thing that would make
   strength 4 a real per-subsystem column rather than one cited figure repeated ten
   times (§2.6).
4. **`build_ref.py` has no `PAGE_SUBSYS` entry for `P_GLOBREGS.md`, `P_ENCODE.md` or
   `P_LABEL.md`**, so three of the ten subsystems have no TU-level denominator at
   all. Adding them is a `docs/whitebox/scripts/` change and would give those rows
   the second denominator the other seven have.
5. **`FUNCTION_BYTE_MATCH.md` §0's sixth-gradient box is unwritten** (§4.1) — a
   one-paragraph cross-link, outside this lane's fence, wanted so the six gradients
   are enumerable from one page rather than five.
