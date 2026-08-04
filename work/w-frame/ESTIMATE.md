# Pre-registered estimate — lane w-frame

    Committed BEFORE the implementation. Nothing under `fixtures/` or `crates/`
    has been touched at the time of this commit. Frozen; scored verbatim in the
    rung doc.

Companion to [`RANKING.md`](RANKING.md) (deliverable 1) and to the lane prereg
`docs/rungs/_2026-08-04-w-frame-prereg.md`, which registered the bias and the
decline clauses before the first measurement.

---

## 1. Target selection, and why the ranking's own head is not the target

The ranking's top three are `xboxheap.cpp` (gap 0), `xboxmem.cpp` (gap 1) and
`Biquad.cpp` (gap 1). **None is the target**, and the reason is the pair of blind
spots §1.1 of `RANKING.md` registered before the numbers existed:

* `xboxheap.cpp` — **prereg decline clause 2**. w-pair measured it diverging at
  **instruction 0** on instruction *order*, with every instruction already in
  the port's vocabulary. It is the demonstration cell for "the key is blind to
  schedule", not a candidate.
* `xboxmem.cpp` — **clause 2**. w-cfgimpl's 10-cell probe grid priced its two
  remaining functions at **seven** independent facts, two of which are an
  allocation rule and a schedule slot with **one witness each**.
* `Biquad.cpp` — **clause 2**. Its `lis`/`lfs` pair **straddles** the compare
  (`lis` at slot 0, `cmplwi` at slot 1, `lfs` at slot 2): the same interleave.

### 1.1 The cheapest frontier TU, hand-counted

Reading the disassembly and counting **independent** refusals — the quantity the
project's estimate rule is written in, where *"if one quantity governs several
boundaries, that is one refusal"*:

**`src/system/negate_test.cpp`** — 2 functions, byte-identical, 80 B each.

| # | refusal | independent because |
|---|---|---|
| 1 | **A framed body containing basic blocks** | `cond_tail.rs` computes its displacement from a *fixed* 3-block shape and w-cfgimpl §2.4 deliberately built **no fixup list**. This body has 5 blocks, 4 branches and 3 targets, two of them shared. Forces `CFG_SHAPE.md` §6's block IR. |
| 2 | **`cmpwi` / `bt`** — signed-literal compare, true-sense branch | one quantity (the compare/branch encoder) governs both boundaries → **one** refusal |
| 3 | **The intra-section unconditional `b`** — true displacement, no relocation | board **#191**: the port's `b` today is a tail call with a section-start placeholder + `REL24`. Choosing the encoding is a decision distinct from computing the displacement. |
| 4 | **Two `bl` sites in one body** | board **#35**, still PARTIAL and explicitly blocked on ">1 call per body". Here the calls are on exclusive paths, which is the easy half — but `CallSeq` admits one. |
| 5 | **The register plan** — scrutinee parked in r10, result temp in r11, shared argument hoisted to r3, `mr r11,r3` / `mr r3,r11` across the join | `plan_cond_pair` parks at **r11 and only r11**; `CODEGEN_W6_COMPARE.md` §6 records the descent to r10 as *"demonstrably richer than a descending counter and not characterized"*. An allocation rule with **ONE witness** — the TU's two functions are byte-identical, so it is one shape. |

**Five independent refusals, ceiling taken neat, no discount.** Prereg decline
clause 1 (*"minimum ≥ 4 → write up the measurement instead of building"*) fires,
and clause 2 fires on refusal 5.

**No frontier TU is converted by this lane. Registered: TU match stays 8.**

## 2. What IS built, and why it is not a consolation prize

Refusal **2** is the one that is not a mechanism, and the ranking says it is the
**most wanted construct on the whole frontier**: `bt` is missing from **8 of 17**
TUs and `cmpwi` from **6**.

Reading `crates/c2-core/src/codegen/cond_tail.rs` turned that into something
sharper, and it is a **correction to this lane's own ranking**:

> **`bt` and `cmpwi` are already written.** `branch_sense` maps all six
> relations to `BO_TRUE`/`BO_FALSE`, and the emitter picks `cmpwi` for a signed
> operand and `cmplwi` for an unsigned one. **Neither has ever been graded by
> the real `c2`.**

Every W8 fixture — `w8_cond_tail.cpp`, `w8_cond_tail_value.cpp`,
`w8_cond_tail_neg.cpp` — tests `v1 == 0` on a **pointer**. That is `Rel::Eq`
(→ `BO_FALSE`, never `BO_TRUE`) on an **unsigned** operand (→ `cmplwi`, never
`cmpwi`) against the literal **0**. So:

* **five of `branch_sense`'s six rows** and
* **the entire signed-compare path**

are asserted only by a unit test that compares the port's table to *itself*, and
have never met the oracle. `STATUS.md` **trap 5** — *"absence reads as success
unless something forbids it"*, recorded twelve times — and board **#137**'s
shape, where WR1 landed ~1,500 lines with the test count unmoved.

`port_vocab` is measured from objs the port has **demonstrably emitted**, so
`insn:bt` and `insn:cmpwi` are honestly absent from it. **The ranking is right
that they are missing from the port's evidence and wrong to imply they are
missing from its code.** That correction travels with the ranking.

**The rung: fixtures that drive every unwitnessed cell through the real
differential.** Six relations × two signednesses, in the band-3 class the W8
gate already accepts.

## 3. Predictions

| # | prediction | rival |
|---|---|---|
| **E1** | **TU match = 8.** Zero TUs converted. | E1 fails at 9. Nothing in this rung can convert a TU; §1.1's five refusals are the reason. |
| **E2** | **Census delta = 0** on the 878-TU workload. The IL side already accepts every relation and both signednesses (`Rel::from_opcode`, `eat_cmp_operand_type`), so no workload function changes class. | R-E2: the delta is positive, meaning some workload body was in this class all along and the class boundary is not where either side thinks it is. |
| **E3** | **At least ONE of the new cells does NOT come out `Port=Match` on the first differential run.** Ceiling neat: `branch_sense`'s `Lt`/`Ge`/`Gt`/`Le` rows and the whole `cmpwi` path have **zero** oracle witnesses, and an unwitnessed table row is unwitnessed. | R-E3: all ten pass first time, and `CFG_SHAPE.md` §3.1's table transfers whole. **I want R-E3** — registering E3 is registering against my own bias. |
| **E4** | **mismatch 0** in the 878-TU scan and in every `gate.sh` lane. An alarm, not a metric. | — |
| **E5** | The `#[test]` count rises. A rung that adds fixtures and no portable assertion is the defect this project has already recorded. | — |
| **E6** | `capture-fail` stays **7** and the FRONTIER stays **17**. | — |

## 4. What a failing cell means, and what I will do about it

If E3 comes true, the honest response is **not** to fit the table to the new
byte. It is to work out which quantity the row got wrong, and — if that cannot
be settled from the witnesses in hand — to **narrow the accepted class so the
unwitnessed relation is refused**, exactly as `plan_cond_pair` already refuses a
schedule its rules cannot deliver. A shape the rules mis-handle must come out as
a **gap**, never as a plausible-looking wrong branch.
