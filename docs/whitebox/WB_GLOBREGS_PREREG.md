# WB_GLOBREGS — PREREG for read R4 (the globregs mint/merge)

> **PROVENANCE — DISASSEMBLY-DERIVED.** Every address below is an absolute VA
> in `compilers/X360/16.00.11886.00/c2.dll`. See
> [`DISCLOSURE.md`](DISCLOSURE.md); nothing here may enter `crates/` without a
> row there naming the address it came from. Whitebox analysis is authorized
> and encouraged (`CLAUDE.md`, project owner, 2026-08-17).

**Lane:** `w-read-r4` · **kind:** characterization lane
(`docs/rungs/README.md` § "Lane kinds" 3) · **Fixtures:** none ·
**Census:** +0 · **predicted reach:** 0, registered · **`crates/` bytes: 0**,
registered as a fence, not an expectation.

**Subject.** Read **R4** of the funded read-plan
(`docs/whitebox/READ_PLAN_2026-08-21.md` §3 row R4 and §5.2; funded by the
owner 2026-08-23 — `docs/DECISIONS_2026-08-22.md` decision 6; board
**#3410**): **`FUN_10b55732`**, `globregs.c`'s mint/merge, item **F1**. The
deliverable in the read plan's own words is *"the candidate mint order + merge
rule as an ordered algorithm — the missing input to the already-read
comparator."*

**Image.** sha256
`c80981c015166effecc71ad8112d5577a065b2300891dfdb02b9c13787a66258`,
**verified by this lane against the repo copy before this file was written**
(`C2_MAP_METHOD.md` §0). The flat export at `~/ghidra-projects/export/c2/` is
dated 2026-08-04, nineteen days before this lane; its input digest matching
the pinned image is what licenses quoting its addresses (`READ_PLAN` §5.4).

---

## 0. WHAT WAS LOOKED AT BEFORE THIS FILE WAS WRITTEN

Stated exactly, because the prereg tier is worth nothing if the boundary is
vague. Before writing this file the lane read:

* `functions.tsv`'s **row** for `0x10b55732` (entry, size, param/caller/callee
  counts, frame size) — the address verification the brief demands up front;
* the existing prose record: `ref/P_REGALLOC.md`, `WB_CANDID_FINDINGS.md`,
  `WB_LIVE_FINDINGS.md` §4/§10, `WB_ITEMF_FINDINGS.md` F1/§9, `READ_PLAN` §3/§5,
  `BOARD.md` #3056;
* `scripts/candid_c1.py` as the probe-driver model.

**No instruction of `FUN_10b55732` or of any function in its subtree has been
read, and `calls.tsv`'s callee list for it has not been opened.** Every
prediction below is made from the call-graph *counts* and the prose record
only.

### 0.1 Dispatch defect check — the brief's address, scored up front

The brief says every coordinator-supplied address list this project has issued
has needed correction, six lanes running, and that R4's was **not** verified by
the coordinator. Scored:

> **`FUN_10b55732` VERIFIES.** `functions.tsv:936` — `10b55732`, size
> **1676**, 2 params, **1** caller, **18** callees, frame 48. Entry, size and
> the "1,676 B" in the read plan all reproduce exactly. **This is the first
> coordinator-supplied address in this wave that needed no correction**, and
> it is recorded here rather than in the findings so that it cannot be
> confused with a result the lane produced after looking.

That the *entry* is right does not mean it is the *right function*. **P1.1
below is the prediction that scores that**, and it is registered as the one
most likely to fail, because `WB_CANDID_FINDINGS.md` §2 already establishes —
from `calls.tsv`, not from a read — that `FUN_10b55732` holds **no direct call
to the mint `0x10b54d32`**, while its *sibling* `FUN_10b55dbe` does
(`0x10b55e66`). A function that mints nothing directly may not be the mint
order.

---

## 1. Prior art this lane must NOT re-derive

Grepped across `docs/`, `scripts/`, `crates/` for `10b55732` and separately
through `docs/BOARD.md` by topic (`globregs`, `promotion`) before this file was
written — the brief's rule, and the reason four lanes have walked into an open
row.

| already held | where |
|---|---|
| `0x10b55732` is `globregs.c`'s renamer; it "walks the blocks, renames each definition, and **inserts merge candidates at join points**" (`FUN_10b54c07` on the join path) | `WB_LIVE_FINDINGS.md:278`, board **#3056** |
| its promotion policy — *which* symbols become candidates — is **not characterized**; a port that gets it wrong has the wrong value set | `WB_LIVE_FINDINGS.md:682-686`, `WB_ITEMF_FINDINGS.md` F1 |
| the candidate record is `0x48` bytes; `+0x1c` is the id, `+0x0c` the priority, `+0x44` the tie-break, `+0x28`/`+0x2c` first/last block, `+0x38` the preference list | `P_REGALLOC.md` §4.1 |
| the mint `FUN_10b54d32` stamps `+0x1c = DAT_10c400d4++` **only on a fresh `alloc`**; the free-list path at `0x10b54d48` skips it | `WB_CANDID_FINDINGS.md` §2.1 |
| `DAT_10c400d4` is **per-function, dense from 1**, reset at `0x10b57676` in `FUN_10b57633`, which is `FUN_10b55732`'s caller's caller-side phase | `WB_CANDID_FINDINGS.md` §1–§2 |
| the worklist `0x10b316b1` walks buckets `0…1023` of `DAT_10c43b80` and accumulates through `0x10b2b82d`; the finished list is the **reverse** of accumulation order | `P_REGALLOC.md` §2, §4 |
| the comparator `0x10b2b82d`: `+0x0c` DESC signed, tie `+0x44` DESC unsigned, `<=` so an exact tie in both puts the newest first | `P_REGALLOC.md` §4, `[O]` 20 cells |
| the selector `0x10b2e7f8` takes min cost over allowed, strict `<`, so register ties go to the earliest of `r11…r3, r31…r14` | `P_REGALLOC.md` §3, `[O]` 6/6 |
| `/O1` and `/Ox` **disagree on 6 of 20 cells and the relation is exact reversal**; the workload is `/O1`, the fixture corpus `/Ox` | `P_REGALLOC.md` §5, board #3241 |
| the globregs phase does not run at all when `DAT_10c2e2fc` is clear — cleared **per function** at `0x10b7e776`, `0x10b7e867`, and on a **size bail-out** `DAT_10c40f18 >= 40000` at `0x10b7e89b` | `WB_CANDID_FINDINGS.md` §2.3, board **#3375** |

**This lane's numerator is therefore the *algorithm*:** the traversal order,
the promotion predicate with its fields and constants, the merge rule, and
what the composition of those with the already-read comparator predicts about
the tie tier.

---

## 2. The grading rule

Tier **PREREG** by [`ref/PREREG.md`](ref/PREREG.md)'s ladder: committed to git
as the **first commit on `wt-w-read-r4`**, before a byte of the target was
read. Each prediction is scored HIT / MISS / UNGRADED in
`WB_GLOBREGS_FINDINGS.md`. **Misses are reported as misses and not smoothed**;
a prediction too vague to be falsified earns nothing and is marked UNGRADED
rather than counted as a hit.

**Denominator for the read: `subtree-functions-read / 19`** — the target plus
its 18 callees. A partial with named guards beats a claimed total reached by
skimming (R3's rule, adopted).

---

## 3. THE TRAP THIS LANE IS DEFINED AGAINST

Not one trap but two, and they pull in opposite directions.

**Trap A — the observable is a quotient of the claim.** The only thing an obj
can show about mint order is the **register assignment permutation**. Mint
order → (comparator) → worklist order → (selector) → registers. Two different
mint orders that induce the same colouring are **indistinguishable to every
probe this project can build.** So a green does not confirm the order; it
confirms the order's image under a many-to-one map. Registered here so that
§6's green cannot be read as more than it is.

**Trap B — #3363's ambiguity, which R1 hit and fenced.** Byte-identical
outputs are equally consistent with *"the rule is as read"* and with *"no tie
in these bodies decided anything"*. R1's C1 came back green and the lane
correctly refused to publish it as proof, publishing the containment argument
as the evidence and C1 as the check that could have overturned it. **This lane
inherits that discipline**: the probe in §6 must contain a cell whose
assignment **changes** when the claimed order changes, or the whole probe is
declared dead and its greens discarded.

**And the confound that makes Trap B sharp here**, which is this subject's own
and is stated as a design requirement rather than a caveat: `P_REGALLOC.md`
§5's *"the **source** use count and the **machine** use count are different
numbers"*. `a+b+b+b` folds to `3*b + a`. **A grid whose source order and
lowered first-definition order agree cannot separate the two hypotheses this
read exists to separate.** §6.2's cells are required to disagree.

---

## P1 — IS THIS THE RIGHT FUNCTION, AND WHAT SHAPE IS IT?

| # | prediction | grade if |
|---|---|---|
| **P1.1** | `FUN_10b55732` holds **no direct call** to `0x10b54d32`, and the candidates whose ids the comparator sorts are minted in its **callees** (`FUN_10b55dbe` and/or `FUN_10b5673e`'s subtree) or in its **caller's** later phases. **Consequence if HIT: the read plan's entry point is one hop above the mint**, and the findings must name the true minting sites and say so as a dispatch correction. | HIT if `calls.tsv` + the instruction stream agree on zero direct calls; MISS if a direct call exists and R1's `calls.tsv` reading was wrong |
| **P1.2** | The body's **outer loop iterates a block list** (a CFG walk), not a symbol table and not a hash. | HIT if the outer induction variable is a block-record `next` pointer; **MISS if the outer walk is over `DAT_10c43b80`, over a symbol table, or over an id range** — and that MISS is the more consequential result, because it would make mint index acausal with respect to program order and would *restore* an explanation of the R1-shaped kind |
| **P1.3** | The **inner** loop iterates tuples within a block, in **forward** program order. | HIT / MISS; a **backward** inner walk is a MISS and is separately reportable, because it inverts every order prediction downstream |
| **P1.4** | The traversal visits blocks in **layout / linear order**, not in RPO, DFS or any dominator-derived order. | HIT if the walk follows a single `next`-style chain; MISS if a worklist, a stack, or a numbering array is involved. **This is the prediction whose MISS costs a port the most**, since a port has the layout order for free and would have to build the other |
| **P1.5** | The function takes 2 params; the first is a **function/procedure record** and the second a **mode or class selector** (an integer, small range), consistent with the phase being run once per register class or once per direction. | HIT / MISS / UNGRADED if the params are not resolvable to a kind |

## P2 — THE PROMOTION POLICY (item **F1**, the thing 3 documents call uncharacterized)

| # | prediction | grade if |
|---|---|---|
| **P2.1** | There is a **locatable predicate** — a single test, or a short conjunction of tests, at one or two named addresses — that decides candidacy, and it is expressible as a boolean over **named record fields with their offsets**. | HIT if stated as such an expression with every field offset named; MISS if the policy turns out to be diffuse across more than 5 functions, in which case §7's decline applies and the partial is landed with its denominator |
| **P2.2** | The predicate **excludes** at least these three: symbols already of kind 1 (a physical register), symbols of kind 3 (memory), and symbols carrying an address-taken / aliased flag. | HIT per clause, scored 0–3; each clause needs the field and the bit named |
| **P2.3** | The predicate contains **at least one numeric threshold** — a size, a use count, or a live-range-length limit — i.e. the policy is not purely categorical. | HIT if a constant is found and its comparison read; MISS if none exists. **A MISS here is a clean result**: it says a port's promotion rule needs no fitted constant |
| **P2.4** | The policy consults **at least one compilation-mode global** (an `/Og`-family or `-Q`-family flag), so that promotion is mode-dependent and a characterization taken at one profile does not transfer. | HIT / MISS; the address of any such global is named |
| **P2.5** | **Formals (incoming arguments) are promoted, and they are promoted first** — before any body-local definition. | HIT / MISS. Registered because every obj cell this project has that shows candidate order (`wbl_x2`, `cnd_a2`, `cnd_a4`, `cnd_h2`) is a formals cell, so this is the clause the existing evidence already constrains |

## P3 — `cand+0x44`, THE FIELD THAT DECIDES EVERY TIE

The single most decision-relevant question in the lane. `P_REGALLOC.md` §4.1
records `+0x44` as the comparator's tie-break and as saved/restored by the
spiller, and **nowhere in the record does any document say what writes it or
what it means.**

| # | prediction | grade if |
|---|---|---|
| **P3.1** | The **complete set of writers** of `cand+0x44` is enumerable image-wide, two independent ways (Ghidra `xrefs.tsv` and a raw scan of the objdump), and is **small (≤ 6 sites)**. | HIT / MISS with the true count. **An enumeration that does not agree between the two methods voids the claim rather than averaging it** |
| **P3.2** | `FUN_10b55732` or its subtree is **among** those writers. | HIT / MISS |
| **P3.3** | **`+0x44` carries a program-order quantity** — a tuple index, a block number, or a definition position — and not a cost, a weight or a flag word. **If HIT, the tie tier has a source-derivable meaning and R4 answers §5.2 affirmatively.** | HIT / MISS |
| **P3.4** | ⚠ **The rival, registered so it cannot be adopted retroactively:** `+0x44` is **never written outside the spiller**, so on a non-spilling function it holds whatever `alloc(0x48)` left — in which case the tie tier degenerates to **`0x10b316b1`'s bucket walk**, i.e. to **descending mint index**, exactly as `P_REGALLOC.md` §4 consequence 3 describes mechanically. | HIT for P3.4 = MISS for P3.2/P3.3 and conversely; **both are publishable and the findings state which**, in those words |

## P4 — THE MERGE RULE

| # | prediction | grade if |
|---|---|---|
| **P4.1** | A **merge candidate is minted at a join point** for a value whose definitions reach that join from ≥ 2 predecessors, and the merge is a **union of live ranges**, not a fresh unrelated value. | HIT / MISS |
| **P4.2** | `FUN_10b54c07` (222 B) is on that path, as `WB_LIVE_FINDINGS.md:278` states. | HIT / MISS — a re-derivation of a held fact, so it scores as a **check on the predecessor**, not as this lane's numerator |
| **P4.3** | The merge is keyed on the **original symbol identity**, so two definitions of the same source variable merge and two definitions of different variables never do. | HIT / MISS |
| **P4.4** | A merged candidate takes the **id of one of its inputs** (or a fresh id minted at the join) — and **which** it is decides whether merge candidates sort before or after the definitions they subsume. **The findings must answer this specifically; "a merge happens" is not the deliverable.** | HIT if answered with an address; UNGRADED if the read cannot reach it |

## P5 — THE HEADLINE: DO THE TEN REFUTED KEYS NOW HAVE AN EXPLANATION?

`crates/c2-core/src/codegen/alloc.rs:103-539` catalogues ten fitted-then-refuted
allocation keys, "wrong on 5 to 42 each"; `alloc.rs:29-36`'s 52,416-configuration
preregistered search returns 179/236 with the residual **exactly the tie tier**.
R1 removed their only standing explanation and sent the question here.

| # | prediction | grade if |
|---|---|---|
| **P5.1** | **The affirmative branch.** Mint order is a deterministic function of the **lowered, post-`dag.c` program order** — so the tie key *is* source-derivable in principle, and the ten keys were refuted because **they fit source order while c2 keys on lowered first-definition order**, which `P_REGALLOC.md` §5 already shows differ (`a+b+b+b` folds; `/O1` vs `/Ox` reverse on 6 of 20). | HIT if the read yields an ordered algorithm whose input is the lowered tuple stream **and** §6's probe cannot refute it |
| **P5.2** | **The negative branch.** Mint order depends on something a port cannot compute from the IL without simulating an earlier phase (a hash order, an allocation-address order, a `globopt` numbering, or state carried from a pass not on the port's path) — in which case **R4 establishes that this mechanism does not explain them either.** | HIT if the read yields that; **this is a real result and the findings say so in those words, not as a shortfall** |
| **P5.3** | Exactly one of P5.1 / P5.2 is scored HIT. **A findings document that scores neither, or hedges between them, has not delivered R4** and the rung says `FAILED` in that word. | registered as the lane's own pass/fail |

---

## 6. THE CONFIRMATION PROBE — designed before it was run

`READ_PLAN` §5.3: `[R]` means *"the instructions were read correctly"*, not
*"this is what c2 does"* — the `.bss` bump rule was read correctly out of a
clean function and was wrong about c2. The brief's rule: **grids and corpora
fail in opposite directions — run both.**

Driver: **`scripts/globregs_c2.py`**, committed under `scripts/` and re-run
from there, not left in gitignored `work/` (#1406 binds anything whose output
is quoted as evidence). It must degrade to `SKIP: toolchain absent` (exit 2),
verified against `C2RS_COMPILERS=/nonexistent`. Oracle: real `cl.exe`
16.00.11886.00 under wibo through the reference seam. **Profile `/O1 /GS- /c`,
the workload's own** — `P_REGALLOC.md` §5's trap is that the fixture corpus is
`/Ox` and the two reverse on the cells that carry signal.

### 6.1 The failure modes the probe is built against, named in advance

R2's probe was specified against four named failure modes and still could not
see a fifth; only a 500-obj population made the control capable of failing.
The four this lane names:

* **FM1 — no tie is exercised.** Every cell's candidates differ on `+0x0c`, so
  the tie tier never runs and identical bytes mean nothing (Trap B).
  *Defence:* the **positive control** below, which must come back DIFFERENT.
* **FM2 — the source/lowered confound.** Cells where source order and lowered
  first-def order agree cannot separate P5.1 from a plain source-order rule.
  *Defence:* §6.2's **discriminator cells**, built to disagree.
* **FM3 — the profile trap.** A finding taken at `/Ox` publishes the reversed
  rule. *Defence:* every cell run at **both** `/O1` and `/Ox`, and the
  reversal relation itself reported as a datum.
* **FM4 — too few live values.** With ≤ 4 simultaneously-live values every
  candidate fits in the volatile run and no callee-saved ordering is
  observable. *Defence:* cells carry ≥ 6 values live across a call.

**The positive question, asked in the form the brief requires:** *would this
probe have gone red if the claim were false in the most likely way?* The most
likely way for the claim to be false is that **mint order is not program
order** — and §6.2's discriminator cells are exactly the cells whose predicted
permutation differs between the two hypotheses. If those cells come back
matching the program-order prediction and the positive control is live, the
probe **was** capable of going red and did not. If the discriminator cells
produce the *same* permutation under both hypotheses, the probe is **declared
blind on that axis** and the claim stays `[R]`.

### 6.2 The cells

| cell | shape | what it separates |
|---|---|---|
| **G-pos** | the positive control: two bodies differing by one operand | instrument liveness — must be DIFFERENT or every green is discarded |
| **G-src** | N values defined in source order, each used once, all live across a call | baseline permutation; N ∈ {3, 6, 8} |
| **G-fold** | same values, but the arithmetic **folds** so the lowered first-def order differs from source order | **FM2 — the discriminator**; P5.1 predicts the lowered order, a source-order rule predicts the source order |
| **G-perm** | G-src with the *declaration* order permuted and the *use* order held fixed, and the converse | which of declaration or first-use drives the mint |
| **G-join** | a value defined on both arms of an `if` and used after — a merge candidate by construction | P4.1/P4.4; the merge candidate's position in the permutation |
| **G-n3** | the `n=3` shape `P_REGALLOC.md` §4 fenced itself with (`b a c` where descending id predicts `c b a`) | **the one live piece of evidence pointing at R4.** Either the read explains this permutation or the findings say it remains unexplained |

### 6.3 The corpus half

A hand-built grid is ~20 cells and cannot see rules that appear only at scale
or on constructs nobody thought to write. The corpus half runs the predicted
order against the **existing tracked fixture corpus** at both modes and
reports the fraction of bodies on which the prediction is *checkable at all*
(many are single-candidate and carry no order), and of those, the fraction it
gets right. **A prediction checkable on 3 bodies is reported as 3, not
rounded up to a rate.**

---

## 7. WHAT THE CONTROLS WILL BE STRUCTURALLY INCAPABLE OF CATCHING

The section that has paid off every time. Registered before the work.

1. **The many-to-one map (Trap A).** No probe here distinguishes two mint
   orders that induce the same colouring. Every `[O]` this lane earns is an
   `[O]` **on the permutation**, never on the order itself.
2. **Any rule that needs > 1024 candidates in one function.** R1's ladder went
   to 400 *functions*; getting 1024 candidates inside a *single* body is a
   different and much larger fixture, and this lane will not build one. **The
   hash-wrap regime of `0x10b316b1` is untested here** and the mint-index ≡
   bucket-index identity is fenced to bodies under 1024 candidates.
3. **Floating point.** `P_REGALLOC.md` §7: the FPR order at `0x10c37f20` is
   read and never obj-checked, no cell in any grid uses floating point. If the
   promotion policy has an FPR-specific clause, **this lane is blind to it**
   and will say so rather than generalize from GPR cells.
4. **The spill/split mint sites** `0x10b2dfe2` and `0x10b2e4ae`. Candidates
   minted *during* allocation take ids after globregs'. If no cell spills, the
   interaction is invisible; if one does, its tie structure is different from
   every other cell. Either way **the ordered algorithm this lane publishes is
   fenced to the pre-colouring mint**, and that fence is registered now.
5. **`/Od` entirely.** The phase does not run when `DAT_10c2e2fc` is clear
   (#3375), and it is cleared per function including on a 40,000 size
   bail-out. **A `/Od` probe is blind by construction**, and the 21 `/Od`
   matches in `scripts/lanes.txt` cannot corroborate anything here.
6. **A field that is constant across every cell.** If `+0x44` holds the same
   value in all cells, no obj can distinguish "this field decides" from "this
   field is inert" — the identical trap `P_REGALLOC.md` §3 records for the
   cost array, which is *uniformly zero over its allowed set on all 25 cells
   this project has compiled*. **P3.3 is therefore not obj-gradable in
   general**, only readable, and its grade will carry `[R]` unless a cell
   happens to vary it.
7. **Whether the read function is on the path the inputs take.** The standing
   `C2_MAP_METHOD.md` §7 lesson. A guard this lane does not vary can route
   real compilations around everything read here.

---

## 8. WHAT WOULD MAKE THIS LANE DECLINE

Registered so a decline is a priced outcome and not a retreat.

* **D1 — the policy is diffuse.** If P2.1 misses and candidacy is decided
  across more than five functions with no locatable predicate, the lane lands
  the **traversal order and the merge rule** (P1, P4) and declines item **F1's
  promotion policy** with the denominator stated and the next address named.
* **D2 — the entry is one hop off and the true minter is large.** If P1.1 hits
  and the real mint order lives in a subtree materially larger than 1,676 B,
  the lane reports the corrected entry point as a **dispatch defect**, reads
  what fits the price, and states the residue.
* **D3 — the probe is dead.** If G-pos comes back IDENTICAL the instrument is
  dead; every green is discarded rather than published (R1's rule), and the
  read lands as `[R]` only, with the failure reported.
* **D4 — `crates/` pressure.** If the read implies a `crates/` change it is
  **reported as a finding for a follow-up lane and not made** — the fence is
  `Fixtures: none`, `Census: +0`, zero `crates/` bytes. R3 did exactly this
  with `LABEL_SEED_GAP` and was right to.

**FAILED**, in that word, if the lane produces neither an ordered algorithm
nor a stated negative on P5.

---

## 9. BOOKKEEPING REGISTERED IN ADVANCE

* Board rows **#3411–#3414** only, in `docs/BOARD.md`'s trailing live ledger,
  in the R2/R3 row format.
* Page: `docs/whitebox/ref/P_GLOBREGS.md`. `ref/P_REGALLOC.md` is **amended
  beside, never rewritten** (`ref/README.md` §2.1).
* Grade: `docs/whitebox/WB_GLOBREGS_FINDINGS.md`.
* Rung under `docs/rungs/`, `Outcome:` one word; `INDEX.md` regenerated with
  `scripts/gen_rung_index.sh`, never hand-edited.
* **`DISCLOSURE.md` gets a row only if a disassembly-derived constant is
  adopted into `crates/`.** This lane adopts nothing, so it is predicted to owe
  **none** — R1's precedent, and the read plan's own §5.2 correction.
* Peer lane `w-read-r5` is reading `FUN_10bc2d7a`. This lane does not touch
  `ref/P_ILRECORD.md` or anything R5 creates; any IL-record fact needed here is
  cited as an **open cross-reference** and reported to the coordinator.
