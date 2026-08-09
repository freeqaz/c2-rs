# WB-J `wb-tables` — PREREG, frozen before the first grep of the flat export

Lane `wb-tables` (WB-J), campaign 2 (`CAMPAIGN_2026-08-08_GENERATORS.md`),
2026-08-09. Board rows **#2110–#2129**.

> **FREEZE DISCIPLINE.** This file is committed as the lane's **first commit**,
> before any `grep`/`sed`/`python3` touches `~/ghidra-projects/export/c2/` or
> the image bytes, and before the first `cl.exe` of any grid. Two things
> touched the outside world before the freeze and are disclosed here because
> they are the only two:
>
> 1. `sha256sum ~/ghidra-projects/bin/c2dll` → `c80981c015166effecc71ad8112d5577a065b2300891dfdb02b9c13787a66258`,
>    which is standing rule 1 and must precede everything;
> 2. `ls -la ~/ghidra-projects/export/c2/` — file names and sizes only, to
>    confirm the export exists (the same disclosure run 1 made).
>
> Everything else this lane knows at freeze time came from **in-repo documents**:
> `WB_SELECT_FINDINGS.md`, `WB_SELECT_FINDINGS_R2.md`, the two rungs, the two
> `frozen.tsv` headers, `WB_REGALLOC_FINDINGS.md`, and ROADMAP §10.27/§10.27.1.
> That is not a clean-room position and this lane does not claim one: it is a
> **settlement** lane, dispatched precisely because two prior readings
> disagree. The predictions below are therefore registered as *arbitration*
> predictions — which prior claim survives — not as cold discoveries.

---

## P0 — the floor

| # | prediction |
|---|---|
| **P0.1** | The table count is settled **by enumeration** — every table listed with its VA, its operator and its 26 decoded entries — not by preferring one document. |
| **P0.2** | The settlement comes out **"both runs partly right"** rather than one run simply wrong. Registered as the optimistic-for-both outcome; if one run is flatly wrong this is a MISS. |

## P1 — the table count (deliverable 1)

| # | prediction |
|---|---|
| **P1.1** | `FUN_10c04cb9` @ `0x10c04cb9` stores **13** distinct pointer variables (`DAT_10c6fdac`…`DAT_10c6fddc`, stride 4), i.e. run 2's count is the count **of installed slots**. |
| **P1.2** | The number of distinct table **bodies** referenced by that function is **17** = 13 + the four `-QVMX128` alternates, i.e. run 1's "sixteen" is 12 + 4 and its error is **an omission of the convert table `0x10b1fd08`**, not a miscount of the alternates. |
| **P1.3** | The four `-QVMX128` alternates are installed by the **same** function under a test on `DAT_10c2e978`, overwriting four of the 13 slots rather than adding new ones. So "how many tables are there" has two defensible answers and the *precondition* on W-SELECT-2 is satisfied only by naming both. |
| **P1.4** | Every table is exactly **26 `int`s = 0x68 bytes**, and the bodies in `0x10c38f30`…`0x10c39548` are contiguous at that stride. |
| **P1.5** | The two published decode tables (run 1 §2.2, run 2 §2.2) **disagree on at least one decoded entry**, over and above the count. Registered pessimistic. |
| **P1.6** | At least one of the 13 tables has **≥8 of its 26 slots empty/illegal**, so "26 entries" overstates the live content and a port needs the slot map, not the raw array. |

## P2 — the grid disagreement (deliverable 2)

| # | prediction |
|---|---|
| **P2.1** | The two cell lists are **not comparable**: they test different constructs and the published scores are not two measurements of one quantity. Registered as the `w-memfit` shape. |
| **P2.2** | Re-running both cell lists against one obj run reproduces **every** published emission listing (24/24). Any failure is a transcription error in the losing doc, not drift. Registered optimistic. |
| **P2.3** | Neither published score moves: run 1 stays 10/12 primary, run 2 stays 9/12 core. This lane re-grades and **does not re-score anything in a run's favour**. |
| **P2.4** | The `/Gy` difference between the two flag sets (run 1 added it, run 2 did not) changes **COMDAT sectioning only** — not one instruction word in any of the 24 cells. |
| **P2.5** | The overlap between the two cell lists is **≥2 and ≤5** semantically-equivalent cells, and on every overlapping pair the two runs' objs **agree**. |

## P3 — `FUN_10c1772b`, the `rlandi` expander (deliverable 3)

Both prior runs named this as the deciding pass and neither could predict it.
Run 2 bounded it black-box: `rlwinm` when `rlandi`'s src and dst coincide,
`li`+`and` otherwise (its §6.1/§6.3b).

| # | prediction |
|---|---|
| **P3.1** | The function contains an explicit **mask-contiguity** computation (a scan for the leading/trailing bit run producing an `MB`/`ME` pair), because `rlwinm` cannot be minted without one. Registered as near-certain; a MISS here means the `MB`/`ME` come from somewhere else entirely. |
| **P3.2** | Contiguity alone does **not** decide it — run 2's diagnostics already refute that (masks 2, 4, 8, 16 are contiguous and still came out `li`+`and`). So the function has **≥2 gates**, and the second one is the interesting one. |
| **P3.3** | The second gate is an **operand-identity test** (`dst == src`, or equivalently a "may I overwrite in place" query), matching run 2's black-box bound. Registered as the leading rival. |
| **P3.4** | Rival to P3.3: the second gate is a **`andi.`/record-form/CR-clobber** test — i.e. c2 refuses `rlwinm` when some consumer needs the source value alive. Registered as the alternative so that P3.3 is falsifiable. |
| **P3.5** | The function also mints **`andi.`/`andis.`** on some path (run 1 §2.4 claims six output forms; run 2's cells never produced `andi.`). Predicted: the `andi.` path exists in the code and is **unreachable at `/O1` for a value**, so the two documents are both right. |
| **P3.6** | A frozen grid on this pass takes **≥2 misses out of ≤12 cells**. Registered pessimistic — this is the pass that beat two lanes. |

## P4 — the consolidated adoption note (deliverable 4)

| # | prediction |
|---|---|
| **P4.1** | §10.27's claim — **W-SELECT-3 is the only row in either campaign where the black-box alternative is genuinely insufficient** — is **CONFIRMED for the tie-break**, and **CORRECTED as stated**: it is the only *adoption* row with that property, but the table **count** and the **slot numbering** are equally non-derivable black-box, and the count is a stated precondition on W-SELECT-2. |
| **P4.2** | After this lane, `rlandi`'s expansion is **black-box re-derivable** (a fixture ladder decides it) and therefore needs **no** DISCLOSURE row — i.e. W-SELECT-5 stays adoption-ready and its "held" R2 twin is released. Registered optimistic. |
| **P4.3** | The set of genuinely-DISCLOSURE-requiring facts for `lower_expr` ends up **≤3 items**, all inside W-SELECT-3 and the opcode-number space. |

## P5 — process

| # | prediction |
|---|---|
| **P5.1** | This lane changes **no** `crates/` file and mints **no** DISCLOSURE row. |
| **P5.2** | At least one **in-place dated correction** lands in a prior findings doc; no prior doc is silently rewritten. |
| **P5.3** | The calibration pass for the D3 grid changes **≥1** cell before the freeze (wb-inline's lesson; run 2's calibration changed three). |

---

**Scoring rule inherited from `wb-regalloc` §7.4 and both prior WB-I runs: a
cell whose premise fails is scored a MISS, not excluded. A prediction with a
false conjunct is a MISS. Retract, do not hedge.**
