# `docs/whitebox/` — the record of what `c2.dll` actually does

> **PROVENANCE — DISASSEMBLY-DERIVED.** Everything in this directory was
> obtained by statically disassembling Microsoft's `c2.dll`. Nothing here may be
> copied into `crates/` without first adding a row to
> [`DISCLOSURE.md`](DISCLOSURE.md) naming the address it came from. The one
> correctness rule is unchanged: **the real `c2` under wibo plus a byte-exact
> obj compare is the sole judge of the port.** A whitebox reading is a
> hypothesis; only the oracle settles anything.

**This page routes; it does not restate.** It is an index over 53 documents at
this level plus 11 in `ref/`, four TSV exports and four data directories — a
set that had no index above `ref/`. Nothing was moved to build it: every file
is where it always was, and every citation elsewhere in the tree still
resolves.

Whitebox analysis is **authorized, encouraged, and not a legal risk** (owner,
2026-08-17 — `CLAUDE.md` § "Whitebox analysis is AUTHORIZED"). Writing Ghidra
output to disk here is *wanted*, not tolerated. Under the 2026-08-21 goal
ranking this directory is **product**, not overhead: goal (1) is a clear
understanding of MSVC's internals, and this is where that understanding is
kept.

---

## Start here

| If you are asking… | Go to |
|---|---|
| **what should be read next, and what is the read worth?** | [`READ_PLAN_2026-08-21.md`](READ_PLAN_2026-08-21.md) — the inventory **with denominators**, an index of every fitted constant in `crates/` against the read that would replace it, and **nine ranked reads R1–R9** priced in days against the black-box lanes they displace. R1→R3 are **funded** (`../DECISIONS_2026-08-22.md` decision 1). A probe-grid lane on any of the nine must first say why it is not the read |
| **where in the binary is the code that decides X?** | [`C2_MAP.md`](C2_MAP.md) — the navigational map: the image's own screen of facts, the 53-file translation-unit partition recovered from the C1001 path, and the subsystem-to-address routing. A navigation aid, never a source of values |
| **how do I reproduce any of this from a clean checkout?** | [`C2_MAP_METHOD.md`](C2_MAP_METHOD.md) — §0 pins the exact image and its sha256. **Verify it before trusting any address in this directory** |
| **I have an address or a subsystem, what is already known about it?** | [`ref/README.md`](ref/README.md) — the address-indexed reference (`P_COFF`, `P_DAG`, `P_EH`, `P_INLINE`, `P_REGALLOC`, `P_SECTION`, `P_SYMBOL`, `SUBSYS`), plus `ADDR.tsv` / `FUNCS.tsv`. Deliberately *not* a findings archive; it points at the findings rather than restating them |
| **has a disassembly-derived fact been adopted into the port, and where?** | [`DISCLOSURE.md`](DISCLOSURE.md) — the engineering-provenance ledger. A row naming the address goes in the **same commit** that adopts the constant into `crates/`. Also states the two provenance tiers: the 53 file names are plain `strings` output (tier 1, no debt); every address is tier 2 |
| **what did a lane predict before it looked?** | [`PREREG.md`](PREREG.md) — location predictions made before the first grep, and the `WB_*_PREREG*.md` files below (one per lane, sometimes three rounds) |

## The reading rule, and why the pairs exist

Every lane here is a **characterization lane** (`../rungs/README.md` § "Lane
kinds"): predicted reach 0, deliverable is address-cited findings under
pre-registration. The convention is a **pair**:

* `WB_<SUBJECT>_PREREG.md` — frozen and committed as the lane's **first
  commit**, before the first grep of the flat export. Several subjects have a
  `_PREREG_R2` (frozen after the disassembly read and before the first
  `cl.exe`) and `wb-label` has an `_R3`.
* `WB_<SUBJECT>_FINDINGS.md` — what was read, every claim carrying an absolute
  VA, and the prereg scored against the outcome.

Three subjects break the pair and the tables below say which: `FRAME` and
`MEMCPY` have no prereg of their own, and `WB_SELECT_FINDINGS_R2.md` is a
second reading against the *first* lane's prereg rather than one of its own.

**Read the findings, but score them against their prereg.** A findings document
read on its own loses the only thing that makes it evidence rather than a story.

## The eighteen findings documents

Freshness class for all of them: **dated record**. They stay as written; where
a later lane overturned one, the overturn is bannered in place and the original
text is left alone.

### The middle end — scheduling, allocation, selection

| Document | What it established |
|---|---|
| [`WB_DAGORDER_FINDINGS.md`](WB_DAGORDER_FINDINGS.md) | `dag.c`'s "tree-to-tuple walk" **is a dependence-DAG list scheduler** — the axis two black-box lanes failed to fit. Prereg: [R1](WB_DAGORDER_PREREG.md), [R2](WB_DAGORDER_PREREG_R2.md) |
| [`WB_DAGORDER2_FINDINGS.md`](WB_DAGORDER2_FINDINGS.md) | the register allocator's candidate order is a **priority list, and a consequence of the scheduler**. Mixed provenance, split per claim: the order is obj- and listing-confirmed on a 20-cell grid, the mechanism is address-cited. Prereg: [R1](WB_DAGORDER2_PREREG.md), [R2](WB_DAGORDER2_PREREG_R2.md) |
| [`WB_DAGCLIENTS_FINDINGS.md`](WB_DAGCLIENTS_FINDINGS.md) | tuple order has a **second author**: a dependence-DAG block merger. Prereg: [R1](WB_DAGCLIENTS_PREREG.md), [R2](WB_DAGCLIENTS_PREREG_R2.md) |
| [`WB_MERGER4_FINDINGS.md`](WB_MERGER4_FINDINGS.md) | the fourth block merger is `0x10b3baa8` → `0x10b3a790`, **and it is not a DAG client**. Prereg: [R1](WB_MERGER4_PREREG.md), [R2](WB_MERGER4_PREREG_R2.md) |
| [`WB_REGALLOC_FINDINGS.md`](WB_REGALLOC_FINDINGS.md) | the register-choice policy and the instruction-order policy, as two separate machines. Prereg: [`WB_REGALLOC_PREREG.md`](WB_REGALLOC_PREREG.md) |
| [`WB_LIVE_FINDINGS.md`](WB_LIVE_FINDINGS.md) | the liveness and interference construction that feeds the selector. Prereg: [R1](WB_LIVE_PREREG.md), [R2](WB_LIVE_PREREG_R2.md) |
| [`WB_SELECT_FINDINGS.md`](WB_SELECT_FINDINGS.md) | how c2 selects PPC instructions. Prereg: [`WB_SELECT_PREREG.md`](WB_SELECT_PREREG.md) |
| [`WB_SELECT_FINDINGS_R2.md`](WB_SELECT_FINDINGS_R2.md) | **an independent second reading of the same question** by a lane the coordinator re-dispatched without noticing the first had landed. Both worked from the same frozen prereg — an accidental replication, and worth more than the duplication cost |
| [`WB_SELECT_RECONCILED.md`](WB_SELECT_RECONCILED.md) | **read this before either `WB_SELECT_*` document**: the two readings settled against a re-read of the export |
| [`WB_TABLES_FINDINGS.md`](WB_TABLES_FINDINGS.md) | the two WB-I disagreements settled, and the `rlandi` pass read. Prereg: [`WB_TABLES_PREREG.md`](WB_TABLES_PREREG.md) |

### Decisions the port has to reproduce

| Document | What it established |
|---|---|
| [`WB_INLINE_FINDINGS.md`](WB_INLINE_FINDINGS.md) | the **inliner's decision function**, read out of the binary and graded by objs — the mechanism `../DIFF_STRUCTURE.md` says dominates the port's wrong-body population. Prereg: [`WB_INLINE_PREREG.md`](WB_INLINE_PREREG.md) |
| [`WB_MEMCPY_FINDINGS.md`](WB_MEMCPY_FINDINGS.md) | the intrinsic-expansion decision function. No prereg pair |
| [`WB_LABEL_FINDINGS.md`](WB_LABEL_FINDINGS.md) | the label counter **settled**: one global, one increment instruction, and an id space shared with the front end. This is the reading that showed `../LABEL_COUNTER.md`'s tables were right and four consecutive lanes had measured them wrong. Prereg: [R1](WB_LABEL_PREREG.md), [R2](WB_LABEL_PREREG_R2.md), [R3](WB_LABEL_PREREG_R3.md) |
| [`WB_FRAME_FINDINGS.md`](WB_FRAME_FINDINGS.md) | the frame-opening predicate and the frame-size arithmetic. No prereg pair |
| [`WB_LOOP_FINDINGS.md`](WB_LOOP_FINDINGS.md) | how c2 lowers a counted loop. Prereg: [`WB_LOOP_PREREG.md`](WB_LOOP_PREREG.md) |
| [`WB_EH_FINDINGS.md`](WB_EH_FINDINGS.md) | factor D's machinery, read off c2's own EH emitter and graded by objs. Prereg: [`WB_EH_PREREG.md`](WB_EH_PREREG.md) |

### The front edge, the back edge, and one price

| Document | What it established |
|---|---|
| [`WB_READER_FINDINGS.md`](WB_READER_FINDINGS.md) | the frontier's **48 reader refusals**, read off c2's own `.ex` reader. Prereg: [R1](WB_READER_PREREG.md), [R2](WB_READER_PREREG_R2.md) — and R2 is worth reading for its own sake: all four round-1 `NOOBJ` predictions came back `DIFF`, because **c2 does not ICE on a desynchronised operand stream**, it decodes whatever the shifted bytes say and emits an obj |
| [`WB_MIDDLE_INTERFACES.md`](WB_MIDDLE_INTERFACES.md) | the opaque middle's **two edges**, addressed, with `[R]` marking every claim read but not yet confirmed. Prereg: [`WB_MIDDLE_PREREG.md`](WB_MIDDLE_PREREG.md) |
| [`WB_CHOOSER_FINDINGS.md`](WB_CHOOSER_FINDINGS.md) | the one-witness-per-side blockers. **Mixed provenance and the split matters** — §5's mechanism readings are disassembly-derived, the rest is not. Prereg: [`WB_CHOOSER_PREREG.md`](WB_CHOOSER_PREREG.md) |
| [`WB_ITEMF_FINDINGS.md`](WB_ITEMF_FINDINGS.md) | item F **priced**: 7 steps, a ceiling of 17 lanes, and **a buy of zero on every population the goal is written in**. A read whose deliverable is a NO. Prereg: [`WB_ITEMF_PREREG.md`](WB_ITEMF_PREREG.md) |
| [`WB_GLOBREGS_FINDINGS.md`](WB_GLOBREGS_FINDINGS.md) | read **R4**: the allocator's tie key **`cand+0x44` is written** — a tuple-visit ordinal at `0x10b55fac` — so the tie tier is a sort on program position and the hash-bucket walk is the **third** tier. **The ten fitted-then-refuted allocation keys finally have a mechanism**, and the 52,416-config null was structurally guaranteed. Spec page: [`ref/P_GLOBREGS.md`](ref/P_GLOBREGS.md). Prereg: [`WB_GLOBREGS_PREREG.md`](WB_GLOBREGS_PREREG.md) |
| [`WB_EXPAND_FINDINGS.md`](WB_EXPAND_FINDINGS.md) | read **R6**: the final-expansion switch **almost never changes the word count** — 26 of 29 arm bodies emit 0 or 1 words and the prologue arms emit **zero directly**, delegating instead. Arbitrates the `0x2f0`/`0x2f4` prologue-vs-epilogue contradiction **against both published sides**, and catches a table that decodes perfectly and is **not** the answer. The prologue's expansion size is **a field of the object** (`.pdata`'s `prolog_words`, 12,610 framed functions). Spec page: [`ref/P_EXPAND.md`](ref/P_EXPAND.md). Prereg: [`WB_EXPAND_PREREG.md`](WB_EXPAND_PREREG.md) |

> ⚠ **THIS TABLE IS STILL STALE FOR THE 2026-08-22/23 FUNDED READ WAVE, AND THE
> GAP IS NOW FOUR.** `w-read-r4` added its own row and deliberately did not
> backfill the others; `w-read-r6` has now done the same, for the same reason —
> writing rows for other lanes' findings would put one lane's summary of another
> lane's work in the index, and flagging the gap is the honest version.
> **Still missing: `WB_CANDID_FINDINGS.md` (R1), `WB_ENCODE_FINDINGS.md` (R2),
> `WB_LABELCHARGE_FINDINGS.md` (R3) and `WB_ILRECORD_FINDINGS.md` (R5)**, plus
> their preregs. Whoever next maintains this file: those are the four.

## Campaigns, proposals, data

| File | What it is | Class |
|---|---|---|
| [`CAMPAIGN_2026-08-08.md`](CAMPAIGN_2026-08-08.md) | campaign 1 — take the stuck questions to the binary; the lane letters (WB-A…WB-J) the findings above are keyed by | dated record |
| [`CAMPAIGN_2026-08-08_GENERATORS.md`](CAMPAIGN_2026-08-08_GENERATORS.md) | campaign 2 — **the generators, not the outputs** | dated record |
| [`BOARD_PROPOSED.md`](BOARD_PROPOSED.md) | rows the `w-map` lane asked the coordinator to mint. Proposals; the board is `../BOARD.md` | superseded on landing |
| [`README_DELTA.md`](README_DELTA.md) | a proposed wording change for the root `README.md`, written by a lane that deliberately did not edit it | proposal |
| `c2_functions.tsv` · `c2_strings.tsv` · `c2_diagnostics.tsv` · `c2_tus.tsv` | the flat exports `C2_MAP.md` is built from: the function table (addr, size, symbol, cluster, confidence, caller/callee counts), the string table with xrefs, the diagnostic-id table, and the translation-unit anchors. **Quote a count from `C2_MAP.md`, not from a `wc -l` here** — every file carries `#` provenance headers and a column row, and `C2_MAP.md` §0 states the denominators | generated (`scripts/build_*.py`) |
| `scripts/` | `ExportFlat.java` plus the `build_*.py` that turn the export into the TSVs, `dump_opcode_tables.py` (reads `0x10B1B260` and `0x10B202B0`, **and no other VA** — #3369), `dagorder_sim.py` | tooling |
| `grids/` | 13 per-lane obj grids, one directory per `wb-*` lane, holding the cells a findings document was graded on | evidence |
| `labels/` | 16 `W-*.tsv` function-label sets by subsystem (COLOR, CRT, EH, EMIT, FLAG, FRAME, GLREC, HASH, IL, LOOP, …) | generated |
| `ref/` | the address-indexed reference — start at [`ref/README.md`](ref/README.md) | live |

## Where to go next

| Go to | When |
|---|---|
| [`../README.md`](../README.md) | you want the whole documentation set, not just the binary record |
| [`../WHITEBOX_LEVERAGE_2026-08-21.md`](../WHITEBOX_LEVERAGE_2026-08-21.md) | you are about to budget a probe grid, and the standing doctrine says price the read first |
| [`../DOC_CONVENTIONS.md`](../DOC_CONVENTIONS.md) | you are adding a lane here and need the prereg/findings pairing and the `WB_*` naming rule written down |
| [`../rungs/README.md`](../rungs/README.md) | you need the characterization-lane contract, the one-word `Outcome:`, and the two rules a probe must satisfy |
