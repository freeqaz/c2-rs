# WB-J `wb-label` — PREREG (round 1), frozen before the first grep of the flat export

Lane `wb-label` (WB-J), 2026-08-09. Board rows **#2430–#2459**. Rung
`docs/rungs/2026-08-09-wb-label.md`. Scratch `work/wb-label/`.

> **FREEZE DISCIPLINE.** This file is the lane's **first commit**. Nothing has
> touched `~/ghidra-projects/export/c2/` beyond an `ls -la` (names and sizes),
> and **no `cl.exe` has run**. The two pre-freeze actions, disclosed because
> they are the only two:
>
> 1. `sha256sum ~/ghidra-projects/bin/c2dll` →
>    `c80981c015166effecc71ad8112d5577a065b2300891dfdb02b9c13787a66258`,
>    which **matches** `docs/whitebox/C2_MAP_METHOD.md`'s table. Standing rule 1,
>    and it must precede everything.
> 2. `ls -la ~/ghidra-projects/export/c2/` — file names and sizes only.
>
> Everything this lane knows at freeze time comes from **in-repo documents**:
> `docs/LABEL_COUNTER.md`, `docs/CFG_SHAPE.md` §7 and §8.2, `docs/BOARD.md`
> (#135, #285–#287, #741–#746, #1761, #1867, #1983), `work/w-ifn/LABEL_LEAD.md`,
> `scripts/gt_label_cod.py`'s header, and the five prior lanes' rungs.
>
> **A second freeze (`WB_LABEL_PREREG_R2.md`) carries the obj-check predictions
> and is committed before the first `cl.exe`,** on the `WB_READER_PREREG_R2.md`
> precedent. Round 1 (this file) is the *mechanism* registration and is written
> from documents only; round 2 is written from documents **plus the binary** and
> before any compile. Anything measured with `cl.exe` is graded against R2, never
> against a rule fitted after the fact.

---

## 0. The commission, restated as a falsifiable position

Five lanes have measured this channel and produced five mutually inconsistent
numbers for what looks like one quantity:

| lane | rows | measured | against the published table |
|---|---|---|---|
| `w-json` | #1800–#1812 | lead **4** | table predicts **2** |
| `w-osfinfo` | #1760–#1771 | lead = count of intra-section `b` words | **refuted one lane later** by `w-xlr` |
| `w-bdnz` | #1980–#1988 | **+7** `/O1`, **+8** `/Ox` | `for` row predicts **+1**; and it is **mode-dependent** |
| `w-blockir` | #2300–#2311 | **+10/+13**, and **+11/+15** for a second register plan of the same class | mode- **and sub-shape**-dependent |
| `w-ifn` | #2350–#2362 | stride **5/4** four ways, all agreeing — and the conclusion still wrong | a **once-per-TU** slot no stride can see |

**The lane's position, registered as the thing to be falsified:** these are not
five errors. They are five correct readings of a quantity that is **not a
function of the source construct**, and the published table is a table of
construct → charge for a quantity that has no such function.

---

## 1. P1 — the counter's mechanism (deliverable 1)

| # | prediction |
|---|---|
| **P1.1** | There is **exactly one** running counter behind `$M`, `$T`, `$S`, `$E`, `__catch$N` and `__unwind$N`, held in a single writable global in `c2.dll`'s data section (a `DAT_10c…`), and `wb-eh`'s `sym[+0x28]` is the field that *caches* a number already taken from it. Registered as near-certain; a MISS is two counters. |
| **P1.2** | The counter is **reset once per TU**, not per function and not per section, and the reset site is reachable from c2's per-TU entry. |
| **P1.3** | The number in `sym[+0x28]` is assigned at **label-object creation**, and the formatter `0x10b99dfe` / decimal writer `0x10b9a08e` **read** it and do not increment. So the formatter is not an increment site and counting formatter callers under-counts the consumers. |
| **P1.4** | There is a **small allocator** — `n = (*ctr)++` — and the interesting number is its **caller count**. Predicted **≥ 8 distinct call sites**, and predicted that they are *not* all symbol-minting: **at least one increment site allocates a number for an object that never reaches the COFF symbol table.** This is the whole mechanism claim and the one to shoot at. |
| **P1.5** | Rival to P1.4, registered so P1.4 is falsifiable: the allocator is **inlined** at every site (a raw `inc dword ptr [DAT]` pattern with no shared callee), in which case the increment sites are found by xref-on-the-global and not by callers-of-a-function. Either P1.4 or P1.5 must hold; if neither does the model is wrong. |
| **P1.6** | **The order is allocation order, not emission order** — already established for one EH body (ROADMAP §9.3, board #135, #1867: the funclet is allocated first and emitted last). Predicted to hold generally, and predicted that the binary shows *why*: allocation happens in a pass that runs before layout. |
| **P1.7** | **Once-per-TU contributors: predicted ≥ 3 distinct kinds exist**, of which the documents already name two (`_fltused`, `w-ifn`'s first-`memcpy`) and neither is in §1.1's table as a general rule. Predicted that they share **one** mechanism — *the first use of a compiler-minted external mints its symbol, and minting a symbol takes a number* — rather than being three special cases. |

## 2. P2 — the unifying model, registered as a numbered claim

Registered **before** any measurement, because it is derivable from
`LABEL_COUNTER.md` §1.1 + §2.1 alone and was never stated there:

> **P2.0 — THE TWO-POPULATION MODEL.** The counter is consumed by **every label
> object c2 creates**, and label objects fall into two populations:
>
> * **minting** — becomes a COFF symbol record (`$M`, `$T`, `$S`, `$E`, a `/Gy`
>   COMDAT section symbol, `_fltused`, `__savegprlr_N`, a pooled `__real@…` and
>   its `.rdata` section symbol, `memcpy`); and
> * **internal** — a branch target the assembler resolves, which mints nothing.
>
> §1.1's surcharge table is a table of the **minting** population. §4's
> control-flow rows are the **internal** population. They were fitted as one
> table and they are two different things.

| # | prediction |
|---|---|
| **P2.1** | Every row of §1.1's surcharge table equals **the number of COFF symbol records that surcharge causes c2 to mint**. Specifically: GPR helper pair = 2 symbols = **+2**; FPR pair = 2 = **+2**; `_fltused` = 1 = **+1**; a newly pooled FP constant = the `__real@` **plus** its `.rdata` COMDAT section symbol = 2 = **+2**; `memcpy` = 1 = **+1**. §2.1 refuted *"one slot per TU-level external"* — predicted the rule it should have reached is *"one slot per compiler-**minted symbol record**"*, and that `w-ifn`'s once-per-TU `memcpy` slot is a **derived consequence** of it, not a new fact. |
| **P2.2** | The framed base (5 `/Gy`, 4 packed) and the leaf base (1, both modes) are likewise symbol counts, and the `/Gy`−packed difference of exactly **1** on framed bodies and **0** on leaf bodies is the **per-function `.pdata` COMDAT section symbol**. Registered as the leading reading; the rival is the `.text` COMDAT section symbol, which is refuted if the leaf base does not also move by 1. **Note the tension in advance**: the leaf base is 1 in *both* modes, so at most one of the two section symbols can be the explanation, and the leaf's own 1 then needs a separate account. Registered as the weakest link in P2. |
| **P2.3** | The `+2` for a **materialised signed relational** (§4, and `w-cmp`'s comparison row, "the first surcharge that mints no symbol at all") is therefore in the **internal** population, and its `+2` is a coincidence of magnitude with the minting rows, not the same term. |
| **P2.4** | **The internal population is allocated from the optimizer's CFG, before the folds that produce the final bytes.** This is the prediction that explains §4.2.2's byte-identical triple (`do/while` +1, `for(;;)+break` +3, `goto` +1, all 24 identical bytes) and the branch-free `mulli` body charging +2: the labels were allocated, then the blocks were folded away. Registered as the mechanism behind *every* one of the five lanes' disagreements. |
| **P2.5** | Consequently **there is no closed-form function from source construct to charge**, and the published §4 rows are not a wrong model of the right quantity but a **right transcription of one probe corpus** promoted to a law. Registered as the answer to deliverable 2 *before* the measurement, so it can be scored. |

## 3. P3 — the `/FAsc` listing seam (deliverable 5's instrument)

`CFG_SHAPE.md` §7 says `$LN<k>` numbering is listing-local and *"nothing should
be derived from the numbers"*. That is a claim about the **numbers**. This lane
registers a claim about the **count**.

| # | prediction |
|---|---|
| **P3.1** | The `$LN<k>@fn` indices are **function-local and dense** (1..n), not global-counter values. (Confirms `CFG_SHAPE.md` §7; registered so a surprise is legible.) |
| **P3.2** | **THE KILLER CELL.** For §4.2.2's byte-identical triple, the listing prints a **different number of `$LN` labels** for `for(;;)+break` (+3) than for `do/while` (+1) and `goto` (+1), even though all three emit the same 24 bytes. If it does, the listing carries the internal population and is the ground-truth instrument the project has been missing. If all three print the same label count, **P3.2 is a MISS and the listing is not the instrument** — say so and do not rescue it. |
| **P3.3** | Weaker fallback, registered separately so P3.2 can fail alone: the listing's label count is **≥** the surcharge on every cell (the listing loses folded labels but never invents them), i.e. the listing bounds the charge from one side. |
| **P3.4** | The `for (i=0;i<10;i++) r=r+a;` body that emits `mulli`+`blr` with **no branch** charges +2 and prints **0** `$LN` labels. Registered as the cell that decides P3.2 vs P3.3 in the *other* direction: it is the one where the listing must under-report if it under-reports anywhere. |
| **P3.5** | The listing is **non-perturbing** here as everywhere else (ROADMAP §9.1 result 1, board #132): the obj beside a `.cod` is byte-identical to the obj without one, on every cell this lane compiles. Any perturbation is a lane-stopping finding and is reported as one. |

## 4. P4 — reconciling the five lanes (deliverable 3)

| # | prediction |
|---|---|
| **P4.1** | **`w-ifn` measured right and concluded wrong** — its own banner already says so, and this lane predicts the once-per-TU slot it found is P2.1's `memcpy` symbol, i.e. a **minting** slot, not a control-flow one. |
| **P4.2** | **`w-osfinfo`'s `b`-word rule was a fit to the minting/internal sum** and its refutation by `w-xlr` is the expected outcome of fitting one number to two populations. Predicted: the `b`-word count correlates with the internal population on simple bodies and decouples the moment a fold removes a `b`. |
| **P4.3** | **`w-bdnz`'s +7/+8 and `w-blockir`'s +10/+13/+11/+15 are all internal-population readings**, and their mode- and sub-shape-dependence is P2.4's optimizer dependence. Predicted: **none of the four numbers is wrong**, and no rule reproduces all four from the source text. |
| **P4.4** | **`w-json`'s 4-against-2** decomposes as a minting term the table has (the helper pair, or a pooled constant) **plus** an internal term, i.e. the §4-quoting error the *"read the `minted` column"* box already warns about. Registered as the most likely single explanation; the rival is a once-per-TU slot as in `w-ifn`. |
| **P4.5** | At least **one** of the five will **not** be fully explained by this lane and will be named as unexplained rather than assimilated. Registered pessimistic on purpose: a reading that explains all five perfectly on the first pass should be distrusted. |

## 5. P5 — `label_slots`' parameters (deliverable 5)

| # | prediction |
|---|---|
| **P5.1** | **`w-bdnz`'s argument survives**: `None` is the only currently-correct value for any class whose charge has not been measured *in situ*, and the reason is stronger than "it is mode-dependent" — the parameter `label_slots` would need is *"how many label objects did c2's optimizer allocate"*, which is **not a function of anything the port reads**. |
| **P5.2** | So the answer to *"mode parameter, sub-shape parameter, both, or neither"* is predicted **neither** — not because the charge is mode-independent (it is not) but because a parameterised `label_slots` would be a **fit**, and #1761 is the standing evidence that a fit here is refuted by the next obj. |
| **P5.3** | The deliverable a conversion lane needs is therefore predicted to be a **procedure, not a rule**, and the procedure is predicted to be: *put the subject in the MIDDLE of a three-function TU, read the stride off the obj, and cross-check the absolute numbers against the TU's first function.* One compile if the listing seam works (P3.2), two if it does not. |
| **P5.4** | Registered as the outcome that would make this lane's headline *wrong*: if the `/FAsc` listing turns out to expose the internal population exactly (P3.2 **and** P3.4 both land), then a **capture-time** channel exists and the right answer changes from *"refuse"* to *"a new IL channel, sourced from the listing"* — §4.1's *"any rung that wants a charging shape owes a new IL channel first"* would be **satisfiable** rather than merely stated. Registered so the lane cannot claim credit either way. |

## 6. P6 — the frozen obj-check (deliverable 4)

The **cells** are frozen here; the **numbers** are frozen in
`WB_LABEL_PREREG_R2.md`, before the first `cl.exe`, after the binary is read.

Shapes the port does **not** emit today, ≥3 required, **six** registered:

| cell | shape | why it is a real held-out cell |
|---|---|---|
| **X1** | a **jump-tabled `switch`** (sparse/wide arms, forcing a real table) | `LABEL_COUNTER.md` §4 records the 8-dense-arm switch at +0 with *"no jump table was emitted, so a jump-tabled switch is still unknown"* — an explicitly unmeasured row |
| **X2** | a **`for` loop containing an `if`** (the `w-bdnz`/`w-blockir` family's next rung) | both lanes measured its neighbours and neither predicted forward |
| **X3** | **`while` with an early `return`** out of the body | a second exit edge, unmeasured in §4 and §4.2.1 |
| **X4** | a **`try`/`catch` with two catch handlers** | `LABEL_COUNTER.md` has **no EH row**; `w-main` added one to a measured-not-modelled table |
| **X5** | a **`switch` inside a `for`** | the compound cell — the one where a per-construct additive rule must break if P2.4 is right |
| **X6** | a body whose **only** loop is fully unrolled at `/Ox` and not at `/O1` | the mode cell, registered to make the mode-dependence a prediction rather than an observation |

| # | prediction |
|---|---|
| **P6.1** | **This lane will MISS at least two of the six.** Registered pessimistic and registered *first*, because P2.5 says there is no closed-form rule and a lane that then goes 6/6 on frozen predictions has fitted something it should not have. **A 6/6 is a reason to distrust the model, not to celebrate it.** |
| **P6.2** | The **minting** component of every one of the six is predicted **exactly right** (P2.1), and every miss is predicted to be in the **internal** component. This is the split that makes the obj-check informative rather than a coin flip. |
| **P6.3** | X5 (`switch` in a `for`) is predicted **not** to equal `charge(switch) + charge(for)`. Registered as the sharpest single cell. |

## 7. What this lane will NOT do

* **No `crates/` changes at all.** Not one line. The findings land in `docs/`.
* **No Ghidra project** is opened — the flat export at `~/ghidra-projects/export/c2/`
  only, and `grep`/`sed` only.
* **No rule is written into `label_lead`** by this lane, whatever it finds.
  #1761 is the standing lesson: a fit written into the port is refuted by the
  next obj, and the next obj is a concurrent lane's.
* **No number in `LABEL_COUNTER.md`'s frozen sections is rewritten.** Corrections
  go in as a banner plus an appended section, the way `LABEL_LEAD.md` and
  `CEILING.md` were.
