# WB_EXPAND — FINDINGS and GRADE for read R6 (the final-expansion switches)

> **PROVENANCE — DISASSEMBLY-DERIVED.** See [`DISCLOSURE.md`](DISCLOSURE.md).

**Lane** `w-read-r6` · characterization · **Fixtures:** none · **Census:** +0 ·
**reach:** 0, as registered · **zero `crates/` bytes**.
Prereg: [`WB_EXPAND_PREREG.md`](WB_EXPAND_PREREG.md), frozen as this branch's
first commit `193ea5055` before a byte of any target body was read.
Spec page: [`ref/P_EXPAND.md`](ref/P_EXPAND.md). Board **#3429**–**#3432**.

**Read the spec page for what c2 does. This page is only the grade** — what was
predicted, what came back, and which of it a control could have caught.

---

## 0. The one-paragraph result

The read plan expected a table of pseudo-ops fanning out into many words. **The
switch does not work that way**: 24 of its 29 arm bodies emit 0 or 1 words (5 are
unbounded), and
the prologue/epilogue arms emit **zero directly** — they delegate. The
count-changing work lives in four helpers, and the prologue's own word count
turns out to be **written into the object by c2 itself**, so a port never has to
predict it. Along the way this lane arbitrated a two-document contradiction
against *both* sides, found an unrecorded per-opcode table, closed the one gap
in the published peephole table, and **caught a table that looks exactly like
the answer and is not** — before it shipped.

**Of 22 gradeable predictions: 12 HIT, 4 MISS, 3 PARTIAL, 3 UNGRADED.** The
four misses and the three ungraded are the informative half and are not
smoothed — one control (§4) was built to test a claim and turned out to be
structurally incapable of testing it, which is reported in those words.

---

## 1. The scorecard

| # | prediction | grade | what actually happened |
|---|---|---|---|
| **P1.1** | binary decision tree, no jump table | **HIT** | `cmp eax,0x270 / ja / je` binary search on `eax` loaded at `0x10c0d58f`. Reproduces `WB_SELECT_FINDINGS.md:668`'s PARTIAL from the bytes |
| **P1.2** | ≤ 40 distinct opcodes get a non-default arm | **MISS** | **69** over 29 bodies. My bound was too tight by 72 % |
| **P1.3** | **headline** — every count-increasing arm has opcode > `0x294`; machine band is 1→1 or 1→0 | **MISS** | `bc` (`0x21`) is `0..1`; the four divides (`0x43`/`0x47`/`0x4b`/`0x4f`) are `0..1` and one body is `1..1`. Machine-band opcodes **do** expand |
| **P1.4** | if P1.3 misses, `bc` at `0x21` is where | **HIT** | Named in advance and it is exactly where — the long-branch expansion (`CFG_SHAPE.md:477`). The divides were **not** anticipated |
| **P2.1** | every arm is one of {1→0, 1→1, 1→k, 1→n}; none many→1 | **PARTIAL** | No many→1 found (correct). But the classification is **incomplete**: there is a fifth class I did not register — **delegate**, where the arm emits 0 and a helper emits many. That is the class the prologue is in |
| **P2.2** | data-dependent class ≤ 4 members, **prologue among them** | **MISS** | Unbounded arms are `retaddr`, `nopalign` (×3 bodies) and `0x2e5` — 3 distinct opcodes, so "≤ 4" holds. **But the prologue is not among them**: its variability lives in the delegate, not the arm. The letter misses; see P2.1 |
| **P2.3** | fixed fan-out `k ≤ 4` for every arm | **HIT** | Maximum direct fan-out is **1** |
| **P2.4** | ≥ 1 arm's fan-out is conditional on an operand value | **HIT** | `bc` `0..1` (branch displacement) and the divides `0..1` |
| **P3.1** | `0x2f0`/`0x2f4` are two pseudo-ops sharing one driver, **likely prologue/epilogue** | **PARTIAL** | Two pseudo-ops sharing one driver: **correct**, and the 2-callers coherence check closes the population. **Prologue/epilogue: wrong** — both are prologue; the restore side is `0x2f6`. The registered alternative ("framed vs frameless") is nearer but also wrong: they differ by whether a **second entry point** is supplied |
| **P3.2** | the count is **not** a constant | **HIT `[O]`** | Seven distinct values over 12,610 framed functions |
| **P3.3a** | common case ≤ 8 words | **HIT `[O]`** | **12,610 of 12,610 = 100 %**; max observed 7 |
| **P3.3b** | **not** linear in saved registers (helper-call reasoning) | **MISS `[O]`** | The saves are **inline stores** and the count **is** linear in the save count. Only **30 of 282** prologues contain a `bl`. The flat 3-word common case is "few registers saved", not "a helper absorbs them" |
| **P3.4** | epilogue within ±2 of prologue | **UNGRADED** | `.pdata` records **no** epilogue word count, and I built no second oracle. Not measured, so not claimed |
| **P3.5** | count computable from upstream quantities | **HIT** | Frame size `fn[0x1a]` and the saved mask are both decided before the pass. And **stronger than predicted**: it is recorded in `.pdata`, so it need not be computed at all |
| **P4.1** | W-TABLES right, the brief wrong: `FUN_10c182b4` is a peephole | **HIT** | One caller `0x10b7dd2c`, gated on `DAT_10c2e2fc`, list walked twice — all three reproduce |
| **P4.2** | no peephole arm increases the instruction count | **PARTIAL** | All 18 arm thunks are `no-mint`. But the check reads the **24-byte thunk**, not the handler it tail-calls. Bounded, **not settled** |
| **P4.3** | byte index `0x10c184a8` → jump table `0x10c18460`, opposite dispatch shape | **HIT** | Plus a correction prior art did not have: the index is `0x293` entries, and reading one more invents a bogus arm 51 |
| **P5.1** | ≥ 95 % of prologues parse under the read's grammar | **HIT, on a stated sub-population** | **282 of 282 = 100 %** parse (first word ∈ {`mflr`, a save}; last ∈ {`stwu`, a save, a helper `bl`}). Denominator is the 282 shape-attributable records, **not** the 12,610 |
| **P5.2** | non-parsing residual concentrated in a named class | **UNGRADED — vacuous** | The residual is **zero**, so there was nothing to concentrate. A control that cannot fail did not pass; it abstained, and saying so is the honest version |
| **P5.3** | ≥ 200 functions from ≥ 20 TUs | **HIT** | 12,610 framed functions from 6,000 objs — 63× the floor |
| **P6.1** | grid: Δwords matches the table | **HIT**, narrowly | `prolog_words` held at 5 while the body grew 17→33 words, so the field tracks the prologue and is not a mis-decoded length. §4 |
| **P6.2** | the cell that can embarrass P3.3 | **UNGRADED** | The cell **failed to vary its own independent variable**: c2 saved exactly two GPRs at every *N*. Not a result about c2 — a defect in the cell, and one board **#3052** already warned about. §4 |

---

## 2. What the read established that no prediction covered

Registered here as *unpredicted*, because a finding that was not forecast should
not be quietly folded into a hit.

1. **A two-document contradiction is arbitrated, and both sides lose.**
   `WB_SELECT_FINDINGS.md:177` (`0x2f0` = prologue) and
   `WB_SELECT_FINDINGS_R2.md:217` (the reverse) disagree;
   `WB_SELECT_RECONCILED.md` settled only which *function* owns the arms.
   **Both `0x2f0` and `0x2f4` reach the prologue driver**; the restore side is
   `0x2f6` → `FUN_10bffb72` → `FUN_10bffaa3`. The family is **five** arms
   (`0x2f0`,`0x2f4`,`0x2f6`,`0x2f7`,`0x2f8`), with `0x2f1`/`0x2f6` as region
   terminators, and **`0x2f5` is in no arm at all** despite `P_ILRECORD.md:254`
   recording IL arm 48 minting it beside `0x2f4`.
2. **An unrecorded table.** The dispatch tail `0x10c0e30b` re-dispatches on a
   per-opcode attribute byte table at **`0x10c3afd8`** (low 3 bits = class).
   767 opcodes reach it.
3. **The peephole's missing row**: arm **6** is `fmr` at `0x10c1838b` — absent
   from the only published arm table.
4. **`ADDR.tsv:1124`'s "41 arms" is not an arm count** — it is the function's
   callee count. Nobody had ever counted this function's arms.
5. **Independent corroboration of R2.** The 16 instruction constructors were
   found by inverting the call graph on the list-insert wrapper `0x10bd5732`;
   every one ORs bit 0 into `node+9`. That is `P_ENCODE.md`'s *"real instruction
   iff `tuple+0x9` bit 0"* reached from the **constructor** end, having been
   established from the **encoder** end. Two derivations, one bit.

### 2.1 The near-miss worth more than several hits

At `0x10b1d180` there is a stride-16 table `{name, machine_opcode, BO, BI}` that
decodes **perfectly** — `beq → (bc, BO=12, BI=2)`. It is a real pseudo-op
expansion table and it is **not the one governing codegen**: its only two
references are string-compare loops in the assembler name-lookup path.

The tell was a contradiction, not a doubt: under the obvious index hypothesis
`0x2f0` decodes to the trap mnemonic `twlti`, while `0x2f0`'s arm demonstrably
calls the prologue driver. Both cannot be true. **The index mapping is therefore
unresolved and deliberately unpublished** (`P_EXPAND.md` §6).

This is the `.bss`-bump failure mode (`C2_MAP_METHOD.md` §7) — a small, clean,
correctly-read table that is simply not on the path the inputs take — caught
before adoption rather than after. Had this lane published it, it would have
been a citable, addressed, wrong table.

---

## 3. The corpus control, and why it could fail

`probe_prolog_words.py`, 12,610 framed functions from 6,000 cached objs.

**The oracle needs no tap**: c2 writes `prolog_words` into the low 8 bits of
every `.pdata` unwind word (`WB_EH_FINDINGS.md` §5 row W-EH-1). The expansion's
output size is a directly observable field of the object.

**It was built so the two hypotheses give numerically different answers** — one
store per saved register climbs with the save count; a `__savegprlr_N` helper
stays flat. **It came back on the side I did not predict** (P3.3b). A control
that can only confirm is not a control; this one falsified.

**One defect found and fixed before any number was quoted.** Shape decoding
first took *"the first `.text` long enough"*, which misattributes words in a
multi-COMDAT obj. Shapes are now restricted to single-`.text` objs whose length
matches the record — **282 of 12,610, 2.2 %** — and that fraction is printed
beside the claim. The histogram is unaffected because `prolog_words` is read
from `.pdata` directly. **The 100 % in P5.1 is over 282, not 12,610**, and
quoting it against the larger denominator would be wrong.

---

## 4. The grid control — it ran, and it FAILED TO DISCRIMINATE

Seven minimal-pair cells (`work/w-read-r6/grid/cell{0,1,2,3,4,6,9}.cpp`, real
`c2` under wibo at `/O1 /GS- /c`), each holding *N* values live across an opaque
call, `N ∈ {0,1,2,3,4,6,9}` — designed so that "one store per saved register"
and "a `__savegprlr_N` helper" would give **numerically different** answers.

```
cell   prolog_words   len_words   prologue words
cell1        5            17      mflr r12 | stw r12,-8 | std r30,-24 | std r31,-16 | stwu r1,-112
cell2        5            19      (identical)
cell3        5            21      (identical)
cell4        5            23      (identical)
cell6        5            27      (identical)
cell9        5            33      (identical)
```

**`prolog_words` is 5 in every cell.** The grid did not vary what it was built to
vary: c2 saved exactly **two** GPRs (`r30`, `r31`) at every *N* and spilled the
rest to the frame, so the save count — the independent variable — was **held
constant by the register allocator** while I believed I was sweeping it.

**This is a defect in the cell design, not a result about c2, and it is
reported as such.** P6.2 asked whether the cell could embarrass P3.3; it could
not, because it never moved the input. A control that cannot fail did not pass —
it abstained. **P6.2: UNGRADED.**

Worse, the repo already knew: board **#3052** (`wb-live`) says *"framing is a
consequence of allocation, not a cause"*. A source-shape knob is the wrong
instrument for a saved-register count, and this lane walked into a finding that
was already on the board — the fifth time by that row's own count.

**What the grid did establish, and it is not nothing.** `prolog_words` stayed at
5 while the body grew from 17 to 33 words. Had the field been mis-decoded — had
the low 8 bits actually been a length or an offset — it would have moved with
`len_words`. It did not. **P6.1: HIT**, on the narrow claim that `prolog_words`
tracks the prologue and not the function. And the 5-word shape
`mflr | stw | std | std | stwu` reproduces the corpus's `mfspr|stw|op62|op62|stwu`
stratum exactly, which is a cross-check between the two controls.

**So the two-sided requirement is met asymmetrically and the asymmetry is the
honest headline:** the **corpus** control was capable of failing and did falsify
P3.3b; the **grid** control ran, cross-checked one shape, and was structurally
incapable of testing the claim it was built for.

---

## 5. What these controls are structurally incapable of catching

The prereg's §7, re-stated after the fact with what each one actually cost:

1. **An obj is post-everything.** Every counted word has been through selection,
   expansion, the peephole and the encoder. If the peephole deleted a word the
   expansion emitted, §3 is right about the obj and wrong about the expansion.
   **This is live, not theoretical**: P4.2 came back PARTIAL precisely because
   the peephole's transitive effect on the count is unsettled.
2. **Word count is a scalar projection.** Every arity in `P_EXPAND.md` §3 could
   be right while the *words* are wrong. Nothing here tests instruction identity.
3. **A grid reaches only what source I can write triggers**, and "unreachable"
   is indistinguishable from "not reached".
4. **Dead arms are undetectable.** Of the 29 bodies, this lane can say which
   opcodes *route* to them, never that any is live.
5. **The corpus cannot see refusals** or shapes absent from it — and §3's
   `bl`-helper fraction (30/282) is exactly a corpus-composition fact, not a
   fact about c2.
6. **A hidden upstream input reads as a constant.** `DAT_10c2e980` moves the
   saved-register range (`0x0f` → `0x0c`) and `DAT_10c2e2fc` gates the whole
   peephole; neither was varied. Both controls held them fixed.
7. **The pre-expansion instruction list is never observed.** Every `1→k` in
   `P_EXPAND.md` §3 is read-derived and confirmed only through its effect on a
   final count — the same limit R3 reported.

---

## 6. Findings owed to other lanes — reported, not acted on

**This lane changed zero `crates/` bytes**, per its fence. Three items are
handed over rather than fixed:

1. **For `w-s1bc` / the S1 bijection.** `ROADMAP_SLICING` §5's AMENDED block
   demotes the bijection to a per-function ratio because the prologue pseudo-op
   is one tuple that becomes many words. **That correction term is observable**:
   `.pdata`'s `prolog_words`. An equality of the form
   `words == real_instruction_tuples + (prolog_words − 1) + …` may be
   recoverable. **Not validated by this lane** — three caveats: leaf functions
   emit no `.pdata` record at all; the epilogue term is **UNGRADED** (P3.4);
   and `WB_INLINE_FINDINGS.md:286` names a **third** breaker the AMENDED block
   does not list (*"a straight-line body's tuple count tracks its word count; a
   loop's does not"*). `nopalign`'s unbounded arm is a fourth.
2. **For whoever maintains `ref/ADDR.tsv`.** Row 1124 presents `FUN_10c0d57e`'s
   **callee** count as "41 arms". It is a defect in a navigational file, and
   this lane is docs-fenced to `docs/whitebox/` pages it owns.
3. **For a follow-up read.** `0x10c3afd8` (767 opcodes), the `0x10b1d180` index
   mapping, and `0x2f5`'s consumer — ranked in `P_EXPAND.md` §8.

**No `DISCLOSURE.md` row is owed.** That file is the ledger of findings
**adopted into `crates/`**, and this lane adopts nothing — R1's precedent.

---

## 7. Coverage, with denominators

| claim | numerator / denominator | tier |
|---|---|---|
| arm bodies enumerated | 29 bodies / 69 discriminated opcodes | `[R]` |
| opcodes NOT covered by that map | **767** reach the tail + 10 shared bodies | — |
| word counts computed | 29 / 29 bodies | `[R]` |
| peephole arms | **18 / 18**, 659 / 659 opcodes | `[R]` |
| peephole arms proven non-minting | 18 / 18 **thunks**, 0 / 18 handlers | `[R]` partial |
| prologue word count | **12,610** framed functions | **`[O]`** |
| prologue shape | 282 / 12,610 = **2.2 %** | **`[O]`** |
| prologue/epilogue arm identity | 5 / 5 arms of the family | `[R]` |

**What this subset is structurally incapable of showing:** the 69/29 map covers
the opcodes the tree *discriminates*. The 767 that reach `0x10c3afd8` are **not
shown to be unchanged** — they are shown to be dispatched by a table nobody has
read. Any statement of the form *"opcode X is not expanded"* is out of this
lane's reach unless X is one of the 69.
