# WB_F0PRICE — PREREGISTRATION — lane `w-f0price`, 2026-08-27

**Frozen before the image was opened.** Written after reading only repository
prose (`CLAUDE.md`, `docs/REGALLOC_BRIEF_2026-08-27.md`,
`docs/DECISIONS_2026-08-22.md` decision 20, `docs/whitebox/READ_PLAN_2026-08-21.md`,
`docs/whitebox/ref/P_REGALLOC.md`, `docs/whitebox/ref/P_DAG.md`,
`docs/WHITEBOX_LEVERAGE_2026-08-21.md`, `docs/CEILING.md` §6.1,
`docs/STEP5_PRICING_2026-08-21.md` §3, `docs/whitebox/WB_ITEMF_FINDINGS.md` §4–§6,
`docs/rungs/2026-08-15-itemfprice.md`). **No byte of `c2.dll` and no line of the
Ghidra export has been read at the time of this commit**, and no `FUNCS.tsv` row
has been looked up for any address named below.

Base tree: `42f76b84921fbda6c7db06e63f2f81de9d06bbba`, branch `wt-w-f0price`.
Board numbers reserved for this lane: **`#3712`–`#3716`**.

---

## 1. The question, restated so the deliverable is checkable

Decision 20 funds this lane to *"price F0 **by reading**, settling the
8-vs-4-raw disagreement between `P_REGALLOC` §7 and `READ_PLAN` R7"*, and
specifies the deliverable as *"a price with its derivation, not a pick"*.

The two live numbers:

* `P_REGALLOC.md` §7 — *"F0 — priced at 8 — is what produces it"*;
* `READ_PLAN_2026-08-21.md` §3 row R7 — *"F0 re-priced 8 → 4 raw"*.

**The derivation must be re-derivable.** A price here is a claim that decays
(`#3505`: four for four, every lane dispatched off a constructed ranking found
the ranking was an artifact), so this lane owes the tree, the date, the
enumeration, and the ranking-check.

## 2. What the price is denominated in — declared before measuring

The unit both published numbers nominally use is **the lane** (one worktree,
one rung), per `STEP5_PRICING_2026-08-21.md` §3 (*"The unit is a **lane** (one
worktree, one rung)"*) and `WB_ITEMF_FINDINGS.md` §6.1 (*"lanes (ceiling)"*).

**The 8 is an enumeration, not an estimate.** `WB_ITEMF_FINDINGS.md` §6.1's F0
row lists **eight numbered sub-items** in its "what the ceiling counts" cell —
(1) tuple-level IR below item A; (2) region finder `0x10be5d4b` + DAG builder
`0x10b328da`; (3) the machine model (`0x10c1c1d4`, `0x10be5df6`,
`LAB_10c1bfe2`) with DISCLOSURE rows; (4) cycle loop + ready list + `node+0x44`
tie-break; (5) K1/K2 `0x10b3b167`/`0x10b3b41b`; (6) M4 `0x10b3baa8` →
`0x10b3a790`; (7) the lowering band `0x10b7dd2c`/`0x10b7ddff`/`0x10b7de4a`;
(8) the four-pass interleave with globregs `0x10b57633`. **8 sub-items, 1 lane
each.** That is the enumeration this lane re-derives against.

**The re-derivation therefore has a defined shape**: resolve every address in
those eight cells against `FUNCS.tsv` and the pinned image, give each sub-item
a verdict, and report the total *with its denominator* — `priced n of 8`.

## 3. Registered predictions

Scored HIT / MISS / PARTIAL / UNGRADED in the findings. Probabilities are the
lane's, written before any measurement.

| # | prediction | p |
|---|---|---:|
| **P1** | **The 8 and the 4 are the same nominal unit (raw lanes) and NOT the same quantity.** The 8 is published as *"ceiling, NO discount factor"*; the 4 is published as a raw figure to which `CEILING` §5's ~5:1 **is applied in the same sentence** (`STEP5_PRICING` §3: *"F0 8 → 4 lanes raw (×5 = 20)"*). On a common scale the "reduction" is an **increase**: 8 (ceiling, uncalibrated) vs 20 (calibrated). | 0.60 |
| **P2** | **`STEP5_PRICING` §3's *"the 4 that leave are search lanes, not construction lanes"* cannot be mapped onto the eight named sub-items.** At most **2** of the 8 are search lanes; the rest are construction. The re-price halved a number without re-enumerating what it counts. | 0.70 |
| **P3** | **Sub-item 7 (the lowering band) is under-priced at 1 lane.** The three functions `0x10b7dd2c` / `0x10b7ddff` / `0x10b7de4a` total **> 2,000 bytes** of body across the three entries, i.e. "large" in the sense the origin lane meant (*"If any of the three is large, F0 is larger"* — `rungs/2026-08-15-itemfprice.md` §2). | 0.55 |
| **P4** | **Sub-item 7's callee closure is wide** — the three entries reach **> 40 distinct direct callees** between them, so the band is a *phase* and not a *pass*, and cannot be read, let alone built, in one lane. | 0.50 |
| **P5** | **At least one of the eight sub-items names an address that is not what the cell says it is.** Base rate in this wave is four for four (R4: `0x10b55732` is the renamer not the mint; R6: `FUN_10c182b4` is the peephole not an expansion pass; R8: all three suggested addresses are dead leads; R9: `0x10b26268` holds format-string pointers not widths). | 0.60 |
| **P6** | **The honest price for F0 exceeds 8 sub-lanes** once the unread sub-items are resolved. | 0.50 |
| **P7** | **Some sub-item's *characterization* half is already discharged by a landed read** (R7 read the machine model to `SCHED_LATENCY.tsv` 10/10 and graded the region finder 1,461/1,461), so the correct answer is not a single scalar but a **split price**: characterization owed vs construction owed. | 0.65 |

## 4. What would make the price LARGER — registered in advance

* Any of the three lowering-band entries is large (P3) or deeply nested (P4).
* A sub-item decomposes into independent fail-closed steps on inspection —
  the same move that took item F from 6 steps to 7.
* An order-changing stage exists between `0x10b31c9a` and the obj that
  `WB_ITEMF_FINDINGS.md` §4.1's four does not name (the read of
  `FUN_10b7e6af` in §4.1 is elided with `...` after `FUN_10b7e032`).
* The `node+0x44` tie-break of sub-item 4 turns out to depend on state R7
  showed is per-cycle rather than static (`P_DAG.md` §3's R7 amendment: the
  ready list is *"fully re-sorted every cycle"*), making sub-item 4 a
  simulator rather than a comparator.

## 5. What would make the price SMALLER — registered in advance

* Two or more sub-items **collapse** into one because no distinguishing cell
  exists — precisely the anti-inflation check that fired twice in the origin
  lane (availability folded into F2; five refusal strings read one variable).
  A collapse must come **out of the total**, as it did there.
* A landed read (R4, R7, R8) already supplies a sub-item's whole deliverable,
  reducing it from "characterize + construct" to "construct".
* An address in the enumeration turns out to be a thin wrapper (`< 100 B`,
  one callee) rather than a pass.

## 6. How "F0 is cheap" is distinguished from "I did not look at all of F0"

**A denominator on every number.** The findings will carry a table with one
row per sub-item (8 rows, fixed by §2's enumeration and not by what turns out
to be convenient), each carrying:

1. the address(es) the cell names;
2. **whether that address resolves** in `FUNCS.tsv` / the pinned image, with
   entry and size — and `README.md` §5.4's trap applied (check entry *and*
   size; "Ghidra found N functions" is a statement about Ghidra);
3. a verdict from a closed set: **READ** (a landed read supplies the spec),
   **PARTIAL**, **UNREAD**, or **UNRESOLVED**;
4. the lane count this lane assigns, or the token **UNPRICED**.

**`UNPRICED` is not 0.** Any sub-item this lane cannot reach is reported
`UNPRICED` and the headline is written as `n priced + m UNPRICED of 8`, never
as a bare total. Absence is not evidence.

## 7. The decline criteria — registered so a decline is priced, not improvised

This lane **declines and says so in the rung header** if any of:

1. The pinned image's sha256 does not match `ref/README.md`'s digest, or the
   Ghidra export cannot be verified against it. (Then the addresses are
   unquotable and everything below is UNPRICED.)
2. More than **3 of the 8** sub-items resolve to `UNRESOLVED`.
3. The lane finds itself needing to **build** a scheduler or an allocator to
   answer the question. Decision 20 §2 forbids it; the correct output is a
   statement that the price is not obtainable without construction.

## 8. What this lane will NOT do

* **Not pick a number.** Decision 20's deliverable is *"a price with its
  derivation, not a pick"*. If the honest answer is that F0 is more expensive
  than both published figures, that is the result.
* **Not build a scheduler or an allocator** (decision 20 §2).
* **Not add a count-bearing row to `gate.sh`** (`#3691` — a 22nd row makes
  `gate_identity_diff.sh` exit 2 for every live lane holding a 21-row base).
* **Not touch `crates/`.** Predicted `crates/` diff: **empty**. Predicted
  census delta: **+0**. Predicted reach: **0**.
* **Not re-price item F as a whole**, and not re-price I1/I2 — `WHITEBOX_LEVERAGE`
  §3.1(1) forbids the second explicitly.
* **Not rank the sub-items by size** and dispatch off the ranking. If any
  ordering appears in the output it is presentation only, and `#3505`'s check
  is stated beside it.

## 9. The ranking check this lane owes (`#3505`)

The eight sub-items come from a **published enumeration** in
`WB_ITEMF_FINDINGS.md` §6.1, not from a ranking this lane constructs. The
check: **does any conclusion here depend on the sub-items' order?** Registered
answer — it must not; the price is a **sum over a set**, and the findings will
state the sum's terms unordered and say so. If this lane ends up needing an
order, that is itself reported as a defect in the price.
