# PREREG — lane `w-inlmetric`

**Committed BEFORE the first measurement.** Charter: `docs/DECISIONS_2026-08-22.md`
decision 15, the `w-inlmetric` row. Lane kind: **characterization + instrument**.
Branch `wt-w-inlmetric`, dispatched at master `6c753ead0`.

> **Nothing below may be edited after this file is committed.** Corrections go in
> the rung as scored outcomes, never as edits here (`docs/rungs/README.md`).

---

## 0. What this lane will produce, and what it may not

**Will produce**

1. A **clause-by-clause conformance table** for c2's inline decision function:
   one row per clause of `P_INLINE.md` §1–§2, each carrying
   **clause → port state → witness**, where port state ∈
   `[R]-derived` / `fitted` / `absent` / `unexercisable`.
2. A **content-hash re-freeze** of `INLINE-P`'s `SAMPLE-B` hold-out (`#3045`'s
   named fix) and a re-grade through the `work/w-inline/grade_pair.py` lineage,
   with the **denominator printed beside the rate** and the dc3 workload stamp.
3. The **inliner's 4-tuple** (read / agreement / exercised / byte-owned) per
   decision 15's metric shape.
4. A **grader** for (1), watched failing on a planted wrong verdict before any
   green of its is quoted.

**May not produce, by charter**

* **Zero `crates/` bytes.** No decision rule, threshold or constant ships.
* No emission widening; no gate row; nothing published here is a gate
  (`docs/FUNCTION_BYTE_MATCH.md` §0 separation).
* No rewrite of an existing doc — amend-beside only.
* No new row in `docs/whitebox/READ_PLAN_2026-08-21.md` (the `#3607` precedent):
  a read that belongs there is **reported in the rung** for the coordinator.

---

## 1. Predictions — clause count

**P1.1 — the clause table will carry 18 clauses, ±3** (accept 15–21).
Enumerated from `P_INLINE.md` §1–§2 and `WB_INLINE_FINDINGS.md` §1–§2.4 before
counting: pass-entry skip · caller-instrs seed · budget clamp · driver entry ·
site collector (call kind `0x0f`) · site collector EH-nesting + conditional bit ·
candidacy ceiling value (`16<<k` / 1000) · candidacy size compare · favour-speed
skip · `__forceinline` bypass at candidacy · legality `[sym+0x20]` mask ·
legality `[sym+0x4c]` refusal mask · legality `[sym+0x4c]` bit 6 requirement ·
depth cap 16 · `maxlevel` · `__forceinline` bypass at accept · caller-huge
decline (35000) · budget accept/decline · the 40-instruction test · the charge ·
the expansion's recursion · POGO gate · POGO per-site discount · parameter-table
selection. That list is 24 candidate rows; **P1.1 predicts the published table
merges or drops to 18** because several of the above are one clause read at two
addresses (the 40-instruction test's two copies; `__forceinline` at two sites).

**P1.2 — at least one clause in the brief's own enumeration will turn out to be
un-citable at the address the brief gives**, because `P_INLINE.md` §2.1 already
carries a CORRECTION block showing four §2.1 addresses were in the wrong
function. Registered at **p = 0.55**. If every address checks out, this is a
miss and is scored as one.

## 2. Predictions — the per-state split, with bias direction

Predicted split over 18 rows:

| port state | predicted count |
|---|---:|
| `[R]`-derived | **2** |
| `fitted` | **3** |
| `absent` | **10** |
| `unexercisable` | **3** |

**P2.1 — `absent` will be the plurality state, ≥ 9 rows.**

**P2.2 — REGISTERED BIAS DIRECTION: OPTIMISTIC.** I expect my own predicted
`[R]`-derived and `fitted` counts to be **too high** and the true `absent` count
to be **at or above 10**. The repo's own prereg tally (`#770`, ~11 optimistic /
2 pessimistic) and `WB_INLINE_FINDINGS.md` §9.1's diagnosis — *"a lane that had
just read a clean call chain assumed the numbers it was hunting would be in
it"* — are the base rate this registers against. A result with `absent` **below**
10 is the direction I am most likely to be wrong in and will be reported as such.

**P2.3 — the single `[R]`-derived row will be the legality bit.** `P_INLINE.md`
§2.1a states `[sym+0x4c]` is the field the port reads as `FN_FLAG_INLINABLE`,
and `crates/c2-core/src/splice.rs`'s `S7-callee-noinline` refusal consults it.
Bit 6 of `[sym+0x4c]` is `0x40`. Predicted: this is the one clause where the
port's counterpart is derived from the same field c2's `0x10b5c06b` tests, and
it is the only such row. **p = 0.6.**

**P2.4 — `crates/` will have NO counterpart for any of:** the growth budget, the
depth cap, the favour-speed / optimization-flag axis, the POGO path, the
parameter tables, the EH-region / conditional-site flag, the 40-instruction
test, the charge, or the expansion's recursion. **Nine `absent` rows, named in
advance.** Any one of these that turns out to have a counterpart is a scored
miss.

## 3. Predictions — the re-frozen `INLINE-P` accuracy, against the NAMED INCUMBENT

**The control is the lineage, not a threshold.** The re-grade is compared to:

| reading | published | `#3045` re-run (dc3 `2277bb73ef23`) |
|---|---:|---:|
| leaf term **dropped** (the frozen configuration) | **0.9716** / n = 9,993 | **0.9681** / n = 8,916 |
| source-leaf (`/Ob0`) | 0.9688 | 0.9650 |
| `/O1`-obj leaf | 0.9631 | 0.9586 |

**P3.1 — accuracy in the dropped-leaf configuration will land in
[0.960, 0.975].** Point estimate **0.968**.

**P3.2 — the three readings' ORDERING will replicate**: dropped > source-leaf >
`/O1`. This has replicated twice (`INLINE_PREDICATE.md` §5, `#3045`). **p = 0.9.**

**P3.3 — the graded population will differ from `#3045`'s 8,916 by more than
2 %**, because dc3 has moved from `2277bb73ef23` to this lane's stamp and the
list is frozen by TU *name*. Predicted range **[7,800, 10,200]**.

**P3.4 — the content-hash re-freeze will find that a non-zero number of the 100
`sample_b.txt` TUs no longer exist or no longer compile.** Predicted **1–8** of
100. If it is 0, `#3045`'s population drop was entirely *within-file* churn and
that is a sharper finding than the one registered.

**P3.5 — a regression must be distinguishable from an improvement.** Registered
in advance: an accuracy **below 0.960** at a population within ±2 % of 8,916 is
a REGRESSION of the rule and will be reported in those words. An accuracy inside
[0.960, 0.975] at a population that moved more than 2 % is **a stable rule on a
moved corpus** and is NOT an improvement, whichever way the third digit went —
`#3045`'s own lesson, applied to this lane's own number in advance.

## 4. Predictions — the 4-tuple

**P4.1 — `read`: the 16/93 coverage line will re-measure within ±3 on the
denominator and ±1 on the numerator** on this tree's
`docs/whitebox/ref/FUNCS.tsv`, over the band `0x10b5b86d`–`0x10b62b00`.

**P4.2 — `exercised`: at least 4 of the 18 clauses will be structurally
unexercisable by the 878-TU workload**, each with a stated reason. `#3066` names
one mechanism already (the port's largest lowered body is 152 B against c2's
`>308 B` static-inline floor — the windows do not overlap); the POGO path
(`P_INLINE.md` §5) and the depth-16 cap are two more, named in advance.
**An unexercisable cell is never read as covered.**

**P4.3 — `byte-owned`: CITED, NOT RE-MEASURED.** `#3534` measured it on
2026-08-25 and decision 15 forbids re-taking it. Registered in advance:
**nothing this lane writes may reverse `#3534`'s flip-OFF of the
inline-decision permuter.** `INLINE_PREDICATE.md`'s model and `splice.rs`'s S7
are right for the port's population (99.87 % opcode substitutions, 0
reorderings) and measurably wrong for the permuter's (2.14 % opcode, 52.50 %
register, 7.90 % pure reorderings) — **both by measurement**, and this lane
publishes both halves or neither.

## 5. Nothing is fitted to the grade

No term, threshold or clause boundary in the conformance table is chosen after
seeing a grading result. The clause list is fixed by §1's enumeration above.
Where a clause's port state is ambiguous, the tie is broken **toward `absent`** —
the direction that under-credits the port — and the ambiguity is stated in the
row.

## 6. The grader, and the failure it must be watched taking

`work/w-inlmetric/check_table.py` will verify the conformance table
mechanically:

* every row citing a `crates/` or `docs/` witness must have that token present
  at that path (`file:sym`), else **RED**;
* every row whose state is `absent` must have its named token **absent** from
  `crates/`, else **RED**;
* every row citing a c2 address must have that address inside a function whose
  entry+size in `FUNCS.tsv` contains it (`P_INLINE.md` §2.1's CORRECTION is
  exactly this check, run mechanically), else **RED**.

**Positive control, registered in advance:** before any green of this grader is
quoted, a wrong verdict is **planted** — one `absent` row is flipped to
`[R]-derived` with a witness token that does not exist, and one address is moved
outside its function — and the grader is watched printing **RED** on both. The
planted rows are then reverted. A grader whose failure has not been watched is
not evidence (`docs/GAPS.md`'s standing rule).

## 7. Decline floor — what makes this lane `FAILED` or `declined`

* **FAILED** if fewer than **12** clause rows reach a graded port state with a
  witness, or if the re-freeze + re-grade cannot be run at all (toolchain
  absent, dc3 unreadable) **and** no substitute measurement is published.
* **`declined`**, a legitimate outcome, if the optional §4 read (the
  93 − 16 = 77 unread inliner-band functions) prices above one lane. The brief
  forbids an open-ended read campaign; the default is to **price and file**, not
  to read.
* A shortfall is reported as a shortfall in the rung's `Outcome:` line, never
  folded into a compound headline.

## 8. Stamps to be recorded at every measurement

* `c2-rs` tip sha of this branch;
* `../dc3-decomp` commit **and dirty count** (dc3 is a LIVE repo — readings at
  different stamps are both right and not comparable);
* the compile flag profile (`/nologo /c /GR /O1 /Oi /EHsc`, and `/Ob0` for the
  site enumerator);
* the `c2.dll` sha256 the address readings are against
  (`c80981c0…a66258`, `C2_MAP_METHOD.md` §0).

---

**Board rows reserved for this lane: `#3623`–`#3628`.** No other row is touched.
