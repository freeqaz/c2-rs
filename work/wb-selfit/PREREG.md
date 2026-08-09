# PREREG — lane `wb-selfit`, the wb-select / wb-select2 reconciliation

    Lane:   wb-selfit  (RECONCILIATION, docs + work/ only)
    Date:   2026-08-09
    Base:   master 05d743f7
    Rows:   #2200-#2219

**FROZEN BEFORE THE FIRST GREP OF `~/ghidra-projects/export/c2/` AND BEFORE THE
FIRST SCORE IS COMPUTED.** What had already been read at freeze time, and is
therefore *not* registerable here, is stated in §0 so that nothing below can be
mistaken for a blind prediction.

---

## 0. What was already read before this freeze — the honest boundary

Read in full, from the repo, before this file was written:

* `docs/rungs/2026-08-09-w-memfit.md` (the method template),
* `docs/whitebox/WB_SELECT_FINDINGS.md` and its rung,
* `docs/whitebox/WB_SELECT_FINDINGS_R2.md` and its rung,
* both `frozen.tsv`, both `select_grid.cpp`, both calib/diag sources,
* `docs/BOARD.md` rows #2040–#2047 and #2100–#2109 (titles),
* `docs/whitebox/DISCLOSURE.md` in full.

So **every fact that is printed in those files is already known to me** and is
not predicted here. In particular I already know each lane's *published* score
(10/12 primary + 6/10 secondary; 9/12 core), each lane's *published* emitted word
sequences, and that the two docs name different addresses for the expansion
switch and for the `rlandi` expander. Registering those would be theatre.

**Not touched at freeze time:** `~/ghidra-projects/export/c2/` (a bare `ls` of
the directory ran, to confirm the export exists — the same single disclosure
`wb-select` made, and nothing else), the image sha256, and every score this lane
will compute.

**No new obj cells.** The toolchain is absent in this worktree (`compilers/` does
not exist, `wibo` is not on `PATH`, the sibling `dc3-decomp` fallback does not
exist). This lane therefore compiles nothing and **every obj number in its output
is inherited from the two lanes' committed files**. There is consequently no
"frozen predictions before the first `cl.exe`" obligation to discharge, and no
cell in this lane is new evidence.

---

## 1. Direction

Registered **PESSIMISTIC**. Board #770's running tally is twelve-of-fifteen
optimistic; the correction for that is to register the reconciliation as *less*
tidy than it wants to be. Concretely: I register that **neither lane is wholly
right on the table count**, that **at least one published PREREG verdict is
wrong on each side**, and that **the mask question does not close**.

---

## 2. The disagreements — which lane I expect to be right, in probability form

`p` is my credence *before* the first export grep.

| # | registered claim | p |
|---|---|---:|
| **D1** | `FUN_10c04cb9` installs **13** distinct destination slots (`DAT_10c6fdac`…`DAT_10c6fddc`, 4 bytes apart), i.e. **R2's count is the count of slots** | 0.70 |
| **D1a** | `wb-select`'s **16** omits the **convert** table `0x10b1fd08` (`DAT_10c6fdac`) and counts the four `-QVMX128` bodies; so 16 and 13 are **not** the same fact under two conventions — one lane's enumeration is short by one table | 0.75 |
| **D1b** | the number of distinct table *bodies* the installer can write is **17** (13 + 4 VMX alternates), and 17 is the reconciled number | 0.55 |
| **D1c** | the four `-QVMX128` bodies overwrite four of the same 13 slots rather than occupying four extra slots | 0.70 |
| **D2** | the dispatch jump table at `0x10c0fb32` has **41** distinct arms (R2 right, `wb-select`'s 46 counts something else — most likely handler call sites or case labels) | 0.55 |
| **D2a** | `wb-select`'s 46 is **not** reproducible as a count of jump-table entries | 0.65 |
| **D3** | `FUN_10c0d57e` and `FUN_10c182b4` are **both real and different** — the two lanes named two different switches, and neither is wrong about the existence of the one it named | 0.70 |
| **D3a** | the switch that expands `rlandi` is **`FUN_10c182b4`** (R2's), not `FUN_10c0d57e` | 0.80 |
| **D4** | **`FUN_10c1772b`** is reached from `FUN_10c182b4` arm 13 and is the `rlandi` expander | 0.60 |
| **D4a** | `FUN_10c0a2e2` (wb-select's name for the same thing) is a **real function** in the export | 0.85 |
| **D4b** | `FUN_10c0a2e2` and `FUN_10c1772b` are **not** the same function and not aliases | 0.70 |
| **D5** | the record-form model is **one fact, not two**: a fusion pass (`FUN_10c0b300`) rewrites the defining opcode to **opcode + 1**, gated on attribute bit `0x10`. Both lanes state exactly this in prose; only their P4.4 *verdicts* differ, and board **#2044**'s headline ("record forms are NOT a fusion") is the one that needs correcting | 0.85 |
| **D6** | the tie direction is **not** in dispute: both lanes read `if (cost_cntlzw <= cost_carry) → cntlzw`, i.e. **ties to the `cntlzw` expander**, and the export confirms it | 0.90 |
| **D6a** | but `wb-select`'s *evidence* for the tie is confounded: `wbs_s4` (`x == 0 ? 5 : 6`) has a compare operand that is the constant `0`, so by `wb-select`'s **own** §3.1 it takes the zero-operand fast path `FUN_10c1a908` and **never reaches the race** | 0.70 |
| **D6b** | consequently the project has **zero** unconfounded black-box evidence that the tie rule exists, and `wb-select` §7.3's "the only black-box evidence in this project that the tie-break exists" must be withdrawn | 0.60 |
| **D7** | "the split lives in three tables" and "the split lives in the type index" are **the same fact at two granularities** and are compatible: the type index selects the *row*, and only `div` / `cmp-imm` / `cmp-reg` have rows that differ across the signed/unsigned boundary | 0.90 |
| **D8** | the two `x & K` findings are **compatible cell by cell** (no cell of either grid contradicts a cell of the other) **but no published predicate fits all of them** | 0.75 |
| **D8a** | `wb-select`'s `W-SELECT-5` / #2046 clause "**`&` with a contiguous mask is `rlwinm`, never `andi.`**", carried as **adoption-ready**, is **over-general** and must be corrected: R2's `S11` and its seven diagnostic cells show `li` + `and` for a contiguous mask | 0.80 |
| **D8b** | R2's own hypothesis in §6.1 — that the deciding fact is whether `rlandi`'s source and destination land in the same register — is **refuted by `wb-select`'s cells** (`wbs_b1`/`wbs_b2` have different source and destination and still get `rlwinm`) | 0.65 |
| **D9** | `wb-select`'s **P3.4 = HIT** is wrong and is contradicted by `wb-select`'s **own** calibration cell `wbk_2` (§6.1: the `if` spelling is byte-identical to the `?:` spelling) as well as by R2's `S12`. Its correct verdict is **MISS**, and R2's retraction stands | 0.80 |
| **D10** | after the corrections this lane files, **each** lane has at least one published PREREG verdict changed | 0.70 |

## 3. The cross-score — registered before the script is written

| # | registered claim | p |
|---|---|---:|
| **G0** | the re-derivation control does **not** reproduce both lanes' published scores on its first run (w-memfit's §3.0 lesson: a plausible number from a mis-parsed verdict is the failure mode) | 0.60 |
| **G1** | once it does, `wb-select`'s published **10/12 primary** and **6/10 secondary** both reproduce exactly from `frozen.tsv` + its own §7 measured block | 0.75 |
| **G2** | R2's published **9/12 core** reproduces exactly from its `frozen.tsv` + its own §6 emitted column | 0.70 |
| **G3** | scoring is **three-valued** — HIT / MISS / ABSTAIN — because neither reading makes a claim on every one of the other grid's cells; a two-valued cross-score would manufacture misses out of silence | 0.90 |
| **G4** | `wb-select`'s reading **abstains** on at least 2 of R2's 12 cells (it explicitly does not claim the non-power-of-two divide, and the `lha` fusion is nowhere in its document) | 0.80 |
| **G5** | R2's reading **abstains** on at least 1 of `wb-select`'s 12 cells | 0.55 |
| **G6** | on the cells where it *does* claim, `wb-select`'s reading scores **≥ 7 of 10** on R2's grid | 0.60 |
| **G7** | R2's reading scores **≥ 9 of 11** on `wb-select`'s grid where it claims | 0.50 |
| **G8** | **neither** reading scores 12/12 on the other lane's grid | 0.90 |
| **G9** | there is **at least one cell in the union where the two readings predict different mnemonics** (not merely one abstaining) | 0.75 |
| **G10** | the shared cell `srawi`+`addze` (`wbs_k1` = `S5`) has **byte-identical emitted words in both lanes' objs**, which is the cheapest available cross-lane reproducibility control | 0.95 |
| **G11** | at least one *further* cell pair across the two grids is a same-source-shape pair with identical emissions, giving a second reproducibility point | 0.70 |

## 4. The merged deliverable

| # | registered claim | p |
|---|---|---:|
| **M1** | the merged `lower_expr` build list is **strictly longer** than either lane's own list (each lane has at least one clause the other lacks) | 0.85 |
| **M2** | the reconciled DISCLOSURE scope carries **exactly one** row that genuinely needs an address, and it is the **cost model / tie rule** — both lanes flagged it and the joint reading confirms it | 0.65 |
| **M2a** | a **second** row turns out to need an address after reconciliation, because a *count* (41 arms / 13 tables) is load-bearing for the ≈60-rule price and no obj yields a count | 0.55 |
| **M3** | at least one row one lane published as **adoption-ready** is downgraded by this lane | 0.75 |

## 5. The budgeted unnamed refusal

One place is pre-armed for a surprise, per #1401 / w-memfit §8.3.

> **The grader's treatment of the class-predicate cells.** `wbs_s5` and `wbs_s6`
> are prose conjunctions ("contains X, contains no Y, 4 or 5 words"), not
> sequences. Encoding those conjuncts by hand is exactly where a mis-parse
> produces a *plausible* published score, and w-memfit found that failure twice.
> If a refusal fires anywhere, I register that it fires **there**, at p = 0.55.

A refusal that fires anywhere else is reported as a **miss of the budget**, not
absorbed.

## 6. Ships

Docs and `work/` only. **No `crates/` change of any kind**, not even a comment;
`cargo test --workspace --release` is therefore expected byte-identical to
master's and is run to confirm rather than to discover (p = 0.95 that it is
unchanged).
