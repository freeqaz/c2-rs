# wb-chooser — the two "one witness per side" blockers were a mis-copy, and the three real choosers all clear the bar

    Tag:       WB-G
    Slug:      wb-chooser
    Date:      2026-08-08
    Fixtures:  none — grid sources live in docs/whitebox/grids/wb-chooser/, deliberately NOT in fixtures/cpp/
    Census:    +0 — WHITEBOX/navigation lane, adopts nothing into crates/
    Record:    docs/whitebox/WB_CHOOSER_FINDINGS.md

---

## PREREG — frozen in two commits, each before the work it predicts

`docs/whitebox/WB_CHOOSER_PREREG.md`.

* **§P0** at `07f0e9ca`, before the base re-derivation. Written from the
  committed decline records and the two TUs' C++ sources only — no obj built,
  no disassembly of either TU read, no byte of the flat export grepped. It
  exists because deliverable 1 says inherited prices have been wrong twice this
  week, so the re-derivation is a *scored prediction*, not setup.
* **§P1–§P4** at `4091d837`, before the first grep of
  `~/ghidra-projects/export/c2/` and the first `cl.exe` of the grid. 21 cells,
  every rival's per-cell prediction, an asserted minimum of discriminating
  cells per grid, and six decline clauses.

Image pin verified before any VA was quoted:
`sha256sum ~/ghidra-projects/bin/c2dll` = `c80981…6258`.

Scored in findings §7: **18 items, 12 HIT, 4 MISS (2 of them retractions),
1 partial, 1 vacuous.**

---

## 1. The headline — the decline record does not say what it has been quoted as saying

> ### Board **#1770**/**#1792**'s clause *"`mmio`'s three clauses and Biquad's FP two-plan both need a chooser with one witness of each side"* traces to `rungs/2026-08-08-w-cfg2.md` §2, where **"two plans" is `Biquad.cpp`'s blocked-function COUNT and "3" is `mmio.cpp`'s.** The row that names `mmio` reads *"several plans **each** — outside the brief's ONE block plan scope"*, in the same sentence that defines a plan as a function. **Neither phrase named a chooser, and neither TU was ever measured for one** — w-osfinfo §10, where the clause was minted, closes with *"no row here was compiled or disassembled by this lane"*.

Two frontier TUs have been unpriceable since 2026-08-08 on the strength of a
paraphrase. This is the **third** time this week: **#1760** (a survey price
wrong in one row each way), **#1782** ("one mechanism" was thirteen), this. All
three are paraphrases of an accurate rung. The mitigation findings §8 proposes
is one line: *a survey paragraph that re-states another rung's price must quote
the rung's own words.*

**And stopping there would have been the cheap answer.** The substance behind
the worry — *does the port have to choose between two lowerings on evidence too
thin to fit?* — is real, and the two reference objs answer it **yes, three times
over**, at choice points nobody had named.

---

## 2. The three choosers, each manufactured past #1767's bar

27 cells under `docs/whitebox/grids/wb-chooser/`, every prediction frozen before
its cell was compiled, every cell graded by the real `c2.dll` under wibo at the
dc3 workload's own flags.

> ### **M-RULE — the park register.** Volatile vs callee-saved is **liveness across calls weighted by the callee's exact register footprint**, and the footprint is a **whole-TU** property. **7 volatile witnesses, 9 callee-saved**, all predicted in advance.

The cell that makes it a mechanism rather than a two-class rule is **M13**: its
callee writes r3, r8, r9, r10, r11, the call passes four arguments in r3–r6, and
c2 parks the live value in **r7** — the one register that is neither an argument
nor in the callee's footprint. c2 does not ask *"is this callee clean?"*; it
allocates around the footprint register by register.

Two sub-rules the port needs to be byte-exact rather than merely correct:
**coalescing beats allocation** (`mmioClose` parks in r5 because the *next* call
wants r5 there; M9-b and M14-b are never moved at all), and **r11 when the value
does not cross a call, r10 when it does** — which is **board #1762**'s open
r11-vs-r10 question, separated on 14 objs.

> ### **B-RULE — the pooled-constant `lis`** goes at the top of the earliest basic block that **dominates every use** of that pool symbol; the `lfs` stays at the use. **3 entry-block witnesses, 6 block-local.**

> ### **B-RULE-2 — compare/branch separation.** Exactly **one** instruction sits between a compare and the branch reading its CR field, when one is available to fill the slot. **6 filled, 5 empty.**

B-RULE-2 is the correction the base obj could not have given and is the reason
the lane was worth running: `Biquad` hoists *two* words into the entry block, so
one takes the separation slot and the other is pushed above the compare, making
the `lis` look like "the first word of the function". **B2 hoists one word and
it lands *below* the compare.** A port that transcribed the rule from `Biquad`
alone is wrong on B2 — the exact generalisation error this campaign exists to
catch.

> ### **B′-RULE — CSE reload order.** A value reloaded across a run of statements is loaded **first** in every statement except the last; in the last — its final use — the operands go in **source order**. **4 for 4 out of sample** at run lengths 2, 3, 4 and 6, matching `Biquad`'s 5.

B′ was registered as this lane's **pessimistic** call (**P2.6**: *"I expect P2.5
to MISS and B′ to be the not-mechanism-driven finding the success floor
allows"*). It hit instead. Board **#770**'s streak gains a pessimistic miss.

---

## 3. The cross-lane result — `10b2778e` is measured, and `WB_FRAME`'s item 2 is closed

> ### **`W-EMIT.tsv`'s `10b2778e` — *"topological sort (callee before caller)"* — is CONFIRMED BY OBJ.** `WB_FRAME_FINDINGS.md`'s "Found and not taken" item 2 names the interprocedural register footprint, proposes `10b2778e` as the mechanism, records it **unmeasured**, and reports that its probes 2 and 3 both failed **because c2 inlined the callees**. The trick is `__declspec(noinline)` — the same attribute `mmio.cpp` puts on `mmioFlush`, which is why `mmio` witnesses this in the corpus at all. **13 of this lane's cells reproduce it**, and M4/M16 show a callee defined *after* its caller emitted *before* it.

That is also why M-RULE's whole-TU clause is implementable rather than
mysterious: the sort makes every callee's footprint available at every caller
**by construction**.

---

## 4. Two retractions, both mine, both registered in advance

| # | registered | outcome |
|---|---|---|
| **P1.3** | the clobber tracking is **emission-order-sensitive**: the same clean leaf defined *later* forces the callee-saved pick | **RETRACTED.** M4 and M16 both park in a volatile. Rival **R-M-C** (whole-TU, order-independent) wins the cell — and §3 explains it: c2 removes the hazard by sorting, so source order never reaches the decision |
| **P1.5** | callee-saved allocated **r31 downward** | **RETRACTED.** M7 emits `mr 30,3` **before** `mr 31,4`. The *set* is the top N of r14…r31 (matching `undname`'s `std r30/r31`), but the *assignment within the set is ascending by first park*, and the prologue saves ascending. An emitter with this backwards gets the right instructions in the wrong registers |

Both are retractions of a *detail* of a rule whose *class* survived, which is
the failure mode a grid is supposed to produce.

---

## 5. The binary — one correction to a standing label, and one honest `unknown`

> ### **`10bfebf7` is a SCAN, not a decision.** `W-FRAME.tsv`'s `saved-gpr-mask` row is right about the instructions and one word off about their job: the function walks the block chain and ORs in `1 << (n-1)` for every **already-assigned** register number `n` in `0x0f…0x20` read at operand `+0x1c`. **The prologue saves whatever the allocator assigned; the volatile-vs-callee-saved choice is upstream in `color.c` and is not readable there.**

Register numbers in this image are `r+1` — the bound `0x0f…0x20` is r14…r31, and
the `DAT_10c2e980` narrowing to `0x12…0x20` is r17…r31, at eight sites. New
label file `docs/whitebox/labels/W-COLOR.tsv` opens the seam `C2_MAP.md`'s
`NOTKNOWN` block declared unmapped — **only** the assigned-register field, the
boundary constant, `color.c`'s copy coalescer at `10b2ceb7` with its
`{0x270, 0x272, 0x293, 0x7b}` opcode set, and the six-primitive bitset library
the whole back end allocates over. The *algorithm* stays unmapped, on purpose.

**The interprocedural clobber consult is UNLOCATED and is filed as `unknown`.**
Searched and eliminated: the prologue chain, `globregs.c`'s live-set builder
(`10b55eae`), `color.c`'s coalescer. `FUN_10b26eda` has 206 call sites and was
not enumerable at this budget; that enumeration, filtered to `regasg.c`'s range
around `FUN_10bc58d5`, is the next probe. **Nothing in §2 depends on it** — the
rules were frozen and graded before the first grep, which is the whole point of
the ordering.

---

## 6. Success floor

The contract: *at least one of the two choice points has ≥3 witnesses per side
with a mechanism reading consistent with all of them — or a written finding that
the choice is not mechanism-driven.*

**Three choice points cleared it, not one.** M at 7/9, B at 3/6, B′ at 5/15.
All three mechanism readings are consistent with every witness including the
base objs, which were never fitted to.

And the second half of the floor is also delivered, for the rows themselves:
**`mmio.cpp` and `Biquad.cpp` are not blocked by an evidence shortage.** They are
blocked by three lowering rules the port does not implement. Their prices are
engineering prices, and #1767 refuses none of them.

---

## 7. Board rows

Minted from the lane's range **#1880–#1899**: **#1880–#1888**.
**#1889–#1899 are left explicitly unminted.**

---

## 8. Found and not taken

Ranked by what the next lane would get:

1. **The float-constant materialisation chooser** (findings §4.2). B7 asked
   where the `lis` goes for a loop-carried constant; c2 answered with **no pool
   at all** — `lis 10,0x3fc0` (the bits of `1.5f`) built in a GPR and stored
   with an integer `stwu`. The same `1.5f` is pooled in B1–B6, so "low half is
   zero" is not the whole predicate. **One obj, four obvious axes, ungridded.**
2. **The interprocedural clobber consult in the binary** (§5). The obj side is
   settled on 16 cells; the code that records and unions the footprint is not
   located. Next probe named.
3. **The call CYCLE.** A topological sort has no answer for mutual recursion, so
   that is the one shape where M-RULE's whole-TU clause must degrade to "assume
   the full volatile set". **One cell**, not run.
4. **The r11/r10 reservation in the disassembly.** The correlation is 14-for-14
   and answers #1762; the reservation itself is unlocated, so the sub-rule is
   `medium` rather than `high`.
5. **The survey-paraphrase mitigation** (findings §8). Three occurrences this
   week, all the same shape, no instrument watches for it.
