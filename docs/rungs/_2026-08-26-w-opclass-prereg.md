# PREREG — `w-opclass`, wave 12

    Lane:      w-opclass
    Date:      2026-08-26
    Kind:      characterization lane (`README.md` § "Lane kinds")
    Base:      f202268f6
    Funded by: docs/DECISIONS_2026-08-22.md decision 14
    Fence:     docs/whitebox/** only. Zero crates/ bytes, zero scripts/ bytes.

**Committed BEFORE the first byte of `c2.dll` was decoded for this lane and
before any port-reader width was tabulated.** Everything in §0 was known when
this file was written and is disclosed here rather than presented later as a
discovery; everything in §2–§6 is a prediction whose grade is owed in the
findings page.

---

## 0. ORIENTATION — what I already knew before writing this, disclosed in full

**The brief's framing is that the 29 operand-class arms at `0x10b3d954` are an
unread target and that "one read closes all 65 at once".** Orientation found,
before this prereg was written, that **the 29 arms were read on 2026-08-08** by
lane `wb-reader` and published with all 29 arm VAs and their operand grammars
at [`../whitebox/WB_READER_FINDINGS.md`](../whitebox/WB_READER_FINDINGS.md) §3,
board **#1591**/**#1592**. This is disclosed here because it contaminates every
prediction below about *what the arms say*: I have read a prior lane's answer.

Four consequences, all stated now rather than discovered later:

1. **My re-derivation is a REPLICATION, not a first read.** It is still worth
   doing — `#3547` found a prior page's cell wrong in both clauses, and this
   tree's rule is derive-don't-inherit — but it must be graded as a
   replication. §2 registers the replication's own falsifiers.
2. **`WB_ILARMS_MAP.md` cites the page it declared unread.** Its §6.2 quotes
   `WB_READER_FINDINGS.md` twice, for `0x2c` (class `05`) and `0x54`
   (class `0d`), as *"prior art, NOT adopted as a premise"* — while §6 and §7
   publish reading the class arms as *"the single cheapest follow-up this map
   exposes"*. Both statements are in the same document.
3. **The tree's own read-index says the arms are READ, 26 lines above the row
   `w-ilarms` amended.** `../whitebox/READ_PLAN_2026-08-21.md:73` reads
   *"29-entry jump table `0x10b3d954`, **all 29 arms read**"*, in the
   **already-read** section. `w-ilarms` amended `:99` and `:174` of that same
   file (its findings §5) and did not read `:73`.
4. **Class `0c`'s sub-record reader is read too.** `ref/P_SUB4F.md`
   (`w-read-r9`, board #3442) reads `FUN_10b9761e` and all 64 field-type codes,
   so the one class whose grammar `WB_READER_FINDINGS.md` §3 leaves as a
   forward reference has its own page.

**What is therefore genuinely unbuilt, and is this lane's deliverable:** nobody
has crossed the class grammar against the port's per-opcode readers over the
whole handled set. `WB_READER_FINDINGS.md` §3.4 did **nine positions** and says
so; the 68 `MATCHED*` rows are the cross that has never been taken. Limb 2 is
closed by *that cross*, not by the read.

### 0.1 The entry points, enumerated BEFORE anything is bracketed

`#3505`'s counter-move, and `w-ilarms`'s V1 missed by 23 for want of it. Taken
from `../whitebox/labels/ilarms_portmap.txt`, which is `[src]`:

| ungated cursor-moving reader family | opcodes of the 68 it reaches |
|---|--:|
| `control_flow::{step,operand}` — the **general width reader** | **67** |
| `codec::{try_ex_token,try_prefix_token}` — fixed-shape recognizers | 19 |
| `bundle::bare_lo_after_prefix` | 2 |
| opcodes with **no** `control_flow` ungated moving site | **1** (`0x46`) |

So limb 2 is decidable against **one** reader for 67 of 68 rows, and the
population that matters is not "68 opcodes" but **the primitives that reader is
built out of**. Counted by hand off `control_flow.rs` before bracketing:

| port primitive | opcodes of the 68 routed through it |
|---|--:|
| `Scan::ty` → `readers::read_type` | **28** |
| fixed `p += 1` (payload-free) | 23 |
| `Scan::tok` → `readers::read_token_var` | 14 |
| `Scan::vint` → `readers::read_varint` | 9 |
| one-off arms (`0x28`, `0x43`, `0x54`, `0x4c`, `0x4f`, `0x66`, `0x33`, `0x46`) | 8 |

**This is what the bracket in §3 is anchored on.** A verdict change is not a
per-opcode event: one divergent primitive moves a whole block, and `ty` alone
is 28 of 68. Any bracket built from "how many opcodes look odd" would be
measuring the artifact again.

---

## 1. The verdict vocabulary, FIXED HERE

`w-ilarms` fixed four verdicts (`ABSENT`, `NARROW(gate)`, `MATCHED*`,
`UNRESOLVED`) and then needed a fifth, `NARROW(fields)`, for `0x28`. Inventing
a category after the numbers are in is how a count stops being gradeable, so
the vocabulary for limb 2 is fixed **now**, with four values and no others:

* **`MATCHED`** — the port's ungated reader advances by exactly c2's width, and
  consumes the same field sequence, **on every input the class grammar
  accepts**.
* **`NARROW(fields)`** — there is an input the class accepts on which the port
  advances by **less**, or refuses outright. Sub-noted `refuses` (fail-closed,
  the correct direction) or `under-reads` (advances less and continues — a
  desync).
* **`WIDE(fields)`** — there is an input the class accepts on which the port
  advances by **more**. Always a desync; always the unsafe direction.
* **`UNRESOLVED`** — the class grammar does not decide it, reported with the
  read that would.

A row that is both narrow and wide reports **`WIDE(fields)`** and says so in
prose. `ABSENT` and `NARROW(gate)` are inherited unchanged and this lane does
not re-derive them.

**The width-function reading, not the field-count reading, is primary — and
that is `w-ilarms`'s own precedent rather than my choice.** Its §6.1 calls
`0x28` `NARROW(fields)` because the port reads a *fixed 2-byte literal* where
the class reads a *variable-width token*: same field **count**, different width
**function**. The strict field-count reading is published as a secondary count
in the findings so a reader can have both denominators.

---

## 2. The replication — does my raw read agree with `wb-reader`'s §3?

Instrument rule, inherited from `w-ilarms` and binding here: the script
hard-codes **one** address and derives every table VA, bound, stride and target
from the operand bytes of the instructions it decodes there.

| # | p | prediction |
|---|--:|---|
| **R1** | 0.90 | From the single hard-coded operand-decoder head, the instrument derives class table `0x10b25e48`, bound `0x1c` (**29** classes) and jump table `0x10b3d954` — the three values `WB_READER_FINDINGS.md` §2 records |
| **R2** | 0.80 | The 29 targets are **not** 29 distinct arms: exactly **27** distinct (`0x10b3d922` shared by classes `0D`/`11`, `0x10b3d941` by `10`/`16`). This is the check `w-ilarms` registered for the *other* table and nobody ran on this one |
| **R3** | 0.60 | My independent decode of all 29 arms agrees with `WB_READER_FINDINGS.md` §3 on **all 29 rows**. Set below 0.9 deliberately: `#3547` found a prior page wrong in both clauses of one cell, and §3 there is `high` = *"read correctly"*, not obj-confirmed |
| **R4** | 0.85 | The class byte my instrument reads out of `0x10b25e48` reproduces `w-ilarms`'s class column on **all 95** handled opcodes (a control on both instruments) |
| **R5** | 0.55 | At least **one** class arm carries a **conditional** the two prior pages' one-line grammar does not surface (a global test, an opcode test, a type-dependent branch) |

**Decline floor.** If the arms cannot be decoded from the image at all, this
lane reports **FAILED** in those words, not a compound headline.

---

## 3. The limb-2 closure — the bracket, anchored on §0.1

**B1 — how many of the 65 change verdict away from `MATCHED`.**
Registered: **[22, 34]**, p = 0.55.

The reasoning is registered with it, because a bracket without its anchor is
`#3505` again. 28 of the 68 route through `Scan::ty`; if that one primitive
diverges from c2's TYPE word the answer is dominated by it, and if it does not
the answer is small. So the distribution is **bimodal** and the bracket is the
upper mode. Registered explicitly:

* **B1a**, p = 0.80: `Scan::ty` diverges from c2's TYPE grammar on some input
  the class accepts → the answer lands in `[22, 34]`.
* **B1b**, p = 0.20: it does not → the answer lands in `[1, 6]`.
* **B1c**: if the measured answer is below 10, the cause will be that I judged
  the shared-primitive divergence out of scope for limb 2 after seeing the
  count. Registered as a self-grade failure, not as an outcome.

| # | p | prediction |
|---|--:|---|
| **B2** | 0.90 | `NARROW(fields)` outnumbers `WIDE(fields)` among the 65 — the port fails closed more often than it over-reads |
| **B3** | 0.70 | At least one row other than `0x43` comes out **`WIDE(fields)`** |
| **B4** | 0.55 | At least one of the 65 is still **`UNRESOLVED`** after the class arms are read, and its residual read is named with an address |
| **B5** | 0.80 | The verdict changes have **≤ 6 distinct root causes** — they concentrate in shared port primitives rather than being 20+ independent per-opcode facts |
| **B6** | 0.50 | The two rows `w-ilarms` named `UNRESOLVED` (`0x2c`, `0x54`) are **both** resolved by the class arms, and **at least one of them is `WIDE(fields)`** |

---

## 4. `0x28` — confirm or refute `NARROW(fields)`

| # | p | prediction |
|---|--:|---|
| **S1** | 0.90 | **CONFIRMED.** Class `02`'s grammar is a variable-width token; the port's `28 00 00` is a fixed 3-byte literal that refuses everything else. `NARROW(fields)/refuses` |
| **S2** | 0.50 | `w-ilarms` §6.1's supporting sentence — *"all six of its class-`02` siblings take a variable-width token"* — needs an amendment, because the class-`02` arm is **not** unconditionally a token read for every member of the class |

---

## 5. Arms 17 and 26 — the `0x43` escape, and whether the hazard is reachable

| # | p | prediction |
|---|--:|---|
| **H1** | 0.95 | `0x43` is class `00`, payload-free. There is no escape and no sub-opcode table; the port's `+4`/`+2` reproduces two witnessed cases by width coincidence |
| **H2** | 0.60 | The port's fixed `+4` is wrong in a **second** way nobody has named — a way that makes `43 42` **narrower** than 4, not wider |
| **H3** | 0.40 | **The wide-token hazard is REACHABLE in the 878-TU workload**: at least one `43 42` site carries a token whose second byte has bit 7 set |
| **H4** | 0.75 | The hazard is **constructible** under the container model even if H3 misses — i.e. nothing in the encoding or in `IlBundle`'s framing bounds the token below `0x8000` |
| **H5** | 0.65 | `43 42` occurs at ≥ 100 sites in the workload (so this is a live production, not a curiosity) |

**Method for H3/H5, fixed now.** The workload's `.ex` streams are read out of
the capture cache (`c2rs cache index` for the entry→source map — the supported
reader — and a minimal `cachefmt` payload extractor for the `.ex` bytes, whose
own control is that its reported `.ex` length equals `c2rs cache show`'s on a
sample). Sites are counted **in body-token position**, by walking the body with
c2's own class widths, never by a raw byte grep; a raw-grep count is reported
beside it as an upper bound, labelled as such.

**If H3 hits, the fix is NOT made here.** An emit change is outside a
characterization lane's fence and outside this lane's file fence. The finding is
published with its evidence and left to a future lane.

---

## 6. The record — what the tree says about whether these arms were read

| # | p | prediction |
|---|--:|---|
| **X1** | 0.70 | ≥ 2 live surfaces in `docs/` say the class arms are unread/unbudgeted, while ≥ 1 (`READ_PLAN_2026-08-21.md:73`) says they are read — a contradiction inside one tree |
| **X2** | 0.60 | The same shape holds for a **second** target this lane touches (a page pricing a read that another page records as already taken) |

---

## 7. Self-grade — five ways this lane fails even if every number is right

1. **A fifth verdict invented after the numbers are in.** §1 fixes four.
2. **Inheriting `wb-reader`'s table instead of re-deriving it** — or
   "re-deriving" it with a script that hard-codes the arm VAs.
3. **A count without its denominator**, anywhere.
4. **Rewriting `w-ilarms`'s text.** The closure lands as a clearly marked
   second-round section; the original stays, per `DOC_CONVENTIONS.md` §2.
5. **Any table ordered by mass** — `#3505`. Tables are ordered by opcode or by
   class number and by nothing else.
6. **Reporting the `0x43` hazard as a defect, or fixing it.** It is a hazard
   until a cell grades it, and the fix is an emit change outside the fence.

## 8. Predicted reach

**0.** Census `+0`. Zero `crates/` bytes, zero `scripts/` bytes, checked with
`git diff --numstat f202268f6..HEAD -- crates/ scripts/ fixtures/` and quoted
in the rung, not asserted.

## 9. Disclosure

This lane adopts nothing into `crates/`, so it owes **0** `DISCLOSURE.md` rows.
The count is to be **checked** by the numstat above, not asserted. What a future
adopter would owe is stated with a number in the findings page.
