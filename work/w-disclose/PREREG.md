# PREREG — lane `w-disclose` (wave 14, decision 16)

**Committed before the first ledger row is written. Never edited afterwards;
graded in the rung, and a miss is said in the word MISS.**

Base: `e548f01fd` (master tip at dispatch). Branch `wt-w-disclose`.
Charter: `docs/DECISIONS_2026-08-22.md` § Decision 16, the `w-disclose` row.
Board rows reserved: **#3642**–**#3647**.

Lane kind: **characterization**. Predicted reach **0** — nothing here licenses
an emit, `FUNCTION_BYTE_MATCH.md` §0's separation stands, and the only `crates/`
edits are comments.

---

## 0. What was already measured before this prereg, and what is therefore NOT a prediction

Stated separately so the grade below cannot be read as predicting things that
were already known. All at tree **`e548f01fdda3f75d14354db3c3f894dca33d5476`**,
reported clean by `scripts/provenance_census.py`.

| # | measured | value |
|---|---|---|
| M1 | `scripts/provenance_census.py --by-file`, `codegen/mop.rs` row | **pop 91 · `[R]` 88 · `[N]` 3 · untagged 0** — `#3632`'s 88 **CONFIRMED on this tree** |
| M2 | the 88, enumerated by re-running the census's own `scan_file` | **85** `mod op` opcode constants (under the `PROV-BLOCK[R]` at `mop.rs:100`) · `OPCODES` · `MAX_C2_OPCODE` · `EncodeParams::C2` |
| M3 | the 3 `[N]` | `OPCODE_INDEX`, `MAX_FIELDS`, `NONE_FIELD` |
| M4 | `OPCODES` against `docs/whitebox/ref/ENCODE_OPCODES.txt` (660 rows) | **85 of 85 agree on mnemonic, base word AND form — 0 mismatches** |
| M5 | `MAX_C2_OPCODE` = `0x294` against the dump's top opcode | **agrees** |
| M6 | c2 addresses cited anywhere in `mop.rs` | **30 distinct**: `0x10b1b260`, `0x10c3a578`, `0x10c39b18`, `0x10bfae2d`, 24 encoder addresses, `0x10bfa26c` |
| M7 | the crates→ledger citation direction (`grep -rn DISCLOSURE crates/`) | one citation names a row that **is not in the ledger**: `crates/c2-reference/tests/middle_interfaces.rs:634` cites `DISCLOSURE W-EXT-1`, which exists only as a **pre-draft** in `WB_READER_FINDINGS.md` §5.3's table |
| M8 | `mop.rs`'s own prose counts | *"the port emits **71** distinct opcodes"*, *"the other **589** are not transcribed"*, *"**71** of c2's 660 rows"*, *"the port's 71 opcodes reach **24** of c2's **109** forms"* — against **85** rows, **575** absent (which the same file already says correctly in `EncodeParams::row`'s comment), **34** distinct forms in `OPCODES`, and **104** distinct form values in c2's table (`P_ENCODE.md` §3). Present since the file's first commit `227b90dd7` |
| M9 | `plan()`'s claim *"Every arm below cites the address of the c2 arm it was read from"* | **four groups cite the composer, not the arm**: forms 26/50 cite `0x10bf9788` (arm `0x10bfa17f`), 28/61 cite `0x10bf97c8` (arm `0x10bfa1a1`), 21/45/46 cite `0x10bf9e55` (arm `0x10bfa667`), 27/58/71 cite `0x10bf9eb5` (arm `0x10bfa676`). `P_ENCODE.md` §5.5 names those four as exactly the composers those arms call, so the citation is one level **deeper**, not wrong |
| M10 | the pinned image | present at `compilers/X360/16.00.11886.00/c2.dll`, sha256 `c80981c0…a66258` — **verified** |

---

## 1. Predictions

Every one of these is unmeasured at the moment of this commit.

| # | prediction | value | bias if wrong |
|---|---|---|---|
| **P1** | **final row count in DISCLOSURE's adopted-findings table** | **21** (17 existing + 4 new) | — |
| **P2** | **new rows minted** | **4** | — |
| **P3** | **constants covered by a row after this lane** | **89** — `mop.rs`'s 88, plus `EX_CLASS_TABLE` | — |
| **P4** | `mod op`'s 85 constants are covered by **one** row via the block marker, not 85 rows | true | — |
| **P5** | a **live dump of the pinned image** reproduces `ENCODE_OPCODES.txt` byte-identically, and `OPCODES` agrees with the **image** 85/85 | **0 mismatches** | optimistic — a stale committed transcription would show here |
| **P6** | **further provenance mismatches** found (a `crates/` value disagreeing with the address/page it cites) beyond M7/M8/M9 | **0** | **optimistic.** M4 already cleared the 85 rows, so the residue is `EX_CLASS_TABLE` and the four §5.5 composers |
| **P7** | **dead citations** in the ledger→`crates/` direction at my tip, over 13 existing + 4 new = **17** crates-naming rows | **0 dead, 17 live** | optimistic |
| **P8** | `scripts/gate_identity_diff.sh` base→tip | **0 lines over 21 rows** | — |
| **P9** | `cargo test --workspace` **test count and target count identical** base vs tip | identical | — |
| **P10** | `scripts/gate.sh` at my final tip | **GREEN, 0 mismatch**, graded > 0 | — |
| **P11** | board rows filed | **5** (`#3642`–`#3646`), leaving `#3647` unspent | — |

## 2. The grouping rule, registered in advance

**One row per c2 artifact read — never one row per constant.** This is the
ledger's existing convention, not a new one: `W-MID-1` is *one* row for a table
address + stride + index origin + sentinel; `W-STAGETAP-1` is *one* row for
**seven** call-site addresses. A row is a claim about a **read**, and 85
transcriptions of one table are one read.

Applied here, that gives exactly four:

* **`W-MOP-1`** — the 85 **opcode NUMBERS** (c2's own indices into `0x10b1b260`
  / `0x10c3a578`) and the table's extent. Covers `mod op`'s 85 + `MAX_C2_OPCODE`
  = **86** constants.
* **`W-MOP-2`** — the 85 transcribed **ROWS**: mnemonic (`0x10b1b260`), base
  word (`0x10c3a578`), form number (`0x10c39b18`). Covers `OPCODES` = **1**.
* **`W-MOP-3`** — the **field placements**, `P_ENCODE.md` §5's arms. Covers
  `EncodeParams::C2` = **1**.
* **`W-EXCLASS-1`** — `EX_CLASS_TABLE = 0x10b25e48`. Covers **1**.

86 + 1 + 1 + 1 = **89**, which is P3.

**Why new names and not `W-MID-5/6/7`.** The `W-MID-*` family is
`w-ildecode`'s, and every one of its rows — like every `W-STAGETAP-*` row — is
**instrument-only**. `W-MOP-*` are the ledger's **first rows whose `Adopted
into` is on the emit path**, and collapsing that distinction into an existing
family is the one thing a provenance register must not do.

**`W-MID-1` and `W-MID-2` are EXTENDED, not duplicated.** Both already carry
the exact facts `mop.rs` cites in its markers (table address, stride, index
origin, `_last`), and both say in their own text *"no table entry is copied"* —
which is **true of `middle_interfaces.rs` and false of `mop.rs`**. That gets an
**amend-beside** box per `ref/README.md` §2.1, never a rewrite.

## 3. Decline floor

The lane **declines and files no row** if any of:

* the census's `[R]` count for `mop.rs` is not 88 on my own tree
  *(discharged at M1 — it is 88)*;
* `OPCODES` disagrees with a live dump of the pinned image, in which case the
  constants' provenance is not what the markers claim and the finding is a
  **board row, not a row in the ledger and not a fix**;
* the marker grammar or `MARK_RE` would have to move — that is `w-provext`'s and
  is a STOP-and-report;
* repairing `mop.rs`'s module doc would need a non-comment byte.

**A wrong row is worse than a missing one.** The hole `#3632` found survived
four lanes; one more day is cheaper than a ledger row nobody can re-derive.

## 4. Out of fence, to be reported and not touched

* the repo-root **`README.md`** — its per-finding paragraph says the
  opcode/encoding tables are *"instrument-only … touch no emit path"*, which
  these rows make **false**. `DISCLOSURE.md`'s own checklist step 4 says the
  action is *"tell the coordinator"*, and that is what this lane does.
* **`crates/c2-reference/tests/middle_interfaces.rs`** — M7's dead `W-EXT-1`
  citation. `#3626` is the precedent for **not** blind-copying a pre-drafted
  row: `W-INLINE-1`'s pre-draft carried two wrong addresses in bold for eight
  days. Reported.
* **`docs/whitebox/ref/P_ENCODE.md`** — `w-encmap`'s this wave.
* **`scripts/provenance_census.py`** and the census snapshot — `w-provext`'s.
