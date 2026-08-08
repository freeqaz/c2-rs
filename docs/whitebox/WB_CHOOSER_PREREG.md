# WB-G `wb-chooser` — PREREG

> **PROVENANCE — DISASSEMBLY-DERIVED.** See
> [`C2_MAP_METHOD.md`](C2_MAP_METHOD.md) §0 for the exact bytes and
> [`DISCLOSURE.md`](DISCLOSURE.md) for what adoption costs. Nothing here is
> adopted into `crates/`.

Lane: `wb-chooser` / branch `worktree-agent-a20932071db1f3a88`, branched at
master `9ed20248` ("docs: WB campaign 2 — four lanes on the GENERATORS").

Image pin verified before writing this document:
`sha256sum ~/ghidra-projects/bin/c2dll` = `c80981c015166effecc71ad8112d5577a065b2300891dfdb02b9c13787a66258`.

**Sequencing, stated so it cannot be re-narrated later.** This document is
frozen in **two commits**, and each half is committed before the work it
predicts:

* **§P0** — committed *before* the base re-derivation of the two choice points
  (deliverable 1). At the moment §P0 was written, the only things read were the
  committed decline records (board **#1767**, **#1770**, **#1786**, **#1792**,
  `rungs/2026-08-08-w-osfinfo.md` §6, `rungs/2026-08-08-w-xlr.md` §10) and the
  two TUs' **C++ sources** in the dc3 tree. **No obj had been built, no
  disassembly of either TU read, and no byte of
  `~/ghidra-projects/export/c2/` grepped.**
* **§P1–§P4** — committed after the base re-derivation and *before* the first
  grep of the flat export and the first `cl.exe` of the grid.

The split exists because deliverable 1 says the inherited descriptions have
been wrong twice this week (#1782: "one mechanism" was thirteen; #1760: single
number survey prices) — so the *re-derivation itself* is a scored prediction,
not setup.

---

## §P0 — what the two choice points ARE (frozen before re-derivation)

Board **#1770** and **#1792** both say, in one clause and no more: *"`mmio`'s
three clauses and Biquad's FP two-plan both need a chooser with one witness of
each side"*. That clause is the entire inherited description. What follows is
what I expect it to unpack to, written cold.

| # | prediction | direction if wrong |
|---|---|---|
| **P0.1** | The **`mmio` choice point is `memcpy` expansion**: inline word-copy vs an out-of-line `bl memcpy`. Both `mmioGetInfo` and `mmioSetInfo` copy **0x48 = 72** bytes, and c2 emits a **call**, not an expansion. | If wrong, the mmio chooser is something other than memcpy — most likely the guard-chain branch shape. |
| **P0.2** | **"three clauses" names the three BLOCKED FUNCTIONS, not three choosers.** `mmio.cpp` has 3 blocked of 11 (`mmioGetInfo`, `mmioSetInfo`, `mmioClose`); the phrase is a function count that got compressed into the word "clauses". | If wrong, there really are three distinct decision points in `mmio.cpp` and I have under-counted. |
| **P0.3** | The **`Biquad` choice point is the common-divisor division plan**: `SetCoefficients`'s else-arm divides **five** values by the *same* divisor `flts[3]`, so c2 chooses between **five `fdivs`** and **one reciprocal + five `fmuls`**. Registered call: at `/O1` with no fast-math, **c2 emits five `fdivs`** (reciprocal substitution is not value-safe). | If wrong, either c2 does form a reciprocal, or the "two-plan" is about something other than the divisions. |
| **P0.4** | `Biquad`'s **"two `.rdata` float pools"** are the constants **`0.0f` and `1.0f`**, and they are the *only* two float constants in the TU. | If wrong, the pools hold something I have not predicted (e.g. a merged 8-byte pool, or a doubled `0.0` for a `stfd`). |
| **P0.5** | **HEADLINE, registered as the pessimistic call (board #770's streak is ~10 optimistic / 2 pessimistic / 1 hit).** At least **ONE of the two inherited choice-point descriptions is materially wrong at base** — the same failure mode as #1782 and #1760. | If wrong, both inherited descriptions survive re-derivation intact, and the "re-derive at base" instruction cost this lane time for nothing. |
| **P0.6** | The **"one witness of each side" count is itself wrong for at least one choice point**, and wrong in the *optimistic* direction for the corpus: `mmio.cpp` alone supplies **two** witnesses of the memcpy **call** side (both at size 72), so that side has ≥ 2 before any manufacture. | If wrong, the two memcpy sites do not both lower to a call, or one of them is not in a blocked function. |
| **P0.7** | Neither choice point is the one **#1767 actually names**. #1767's own chooser is `slwi`-by-4 vs `mulli`-by-72 — the **power-of-two multiply** decision in `osfinfo.cpp` — and #1770 borrowed its *rule*, not its *instance*, to decline `mmio` and `Biquad`. So there are **three** choosers in play, not two, and the third one already has a shipped class. | If wrong, #1767's multiply chooser is the same mechanism as one of the two declined ones. |

### P0 decline clauses

1. If a TU's reference obj cannot be produced at the workload's own flags, that
   choice point is **dropped and said so**; no cell of it is scored.
2. Base re-derivation reads the **reference obj that the corpus already grades
   against**. That is not an experiment — it is the same already-paid-for
   ground truth every conversion lane reads. The experiments are §P3's
   manufactured cells, and they are frozen in the second commit.

---

## §P0 RESULT SUMMARY (filed here so §P1 is read in the right light)

Scored in full in [`WB_CHOOSER_FINDINGS.md`](WB_CHOOSER_FINDINGS.md) §1. The
headline, needed to read §P1: **the inherited description is a mis-copy.**
`rungs/2026-08-08-w-cfg2.md` §2's table — the source both #1770 and #1792 quote
— uses "**two plans**" and "3" to mean **blocked-function counts** ("outside the
brief's *ONE block plan* scope"). Neither phrase ever named a chooser. Board
#1770's own §10 says of itself *"no row here was compiled or disassembled by
this lane"*. So **"a chooser with one witness of each side" was never measured
for either TU** — it was inferred from a word.

§P1 onward therefore predicts the choosers that are **actually in the two
reference objs**, re-derived at base at `9ed20248`, and is frozen before the
first grep of `~/ghidra-projects/export/c2/` and the first `cl.exe` of the grid.

---

## §P1 — choice point **M** (`mmio.cpp`): the park register

**What the base obj shows.** Four values are parked out of an argument register
at function entry, and c2 picks a **volatile** register for two of them and a
**callee-saved** one for the other two — the callee-saved pick costing a
`std r31,-16(1)` / `ld r31,-16(1)` pair:

| site | park | reg | calls the parked value is live ACROSS |
|---|---|---|---|
| `mmioGetInfo` +0x0c | `mr 11,3` | **r11** volatile | none |
| `mmioSetInfo` +0x10 | `mr 31,3` | **r31** saved | `bl memcpy` (external) |
| `mmioClose` +0x10 | `mr 31,3` | **r31** saved | `bl mmioFlush`, then `bctrl` (indirect) |
| `mmioClose` +0x14 | `mr 5,4` | **r5** volatile | `bl mmioFlush` **only** |

**M-HYP (registered).** The pick is **liveness across calls, weighted by the
callee's KNOWN clobber set**. A value live across no call, or across only calls
whose clobber set c2 already knows to spare the register, stays in a volatile.
A value live across a call with an unknown or hostile clobber set goes to a
callee-saved register and pays the prologue pair. `mmioClose`'s `r5` is the
separating witness *already present at base*: it is live across `bl mmioFlush`,
and `mmioFlush` is a same-TU `li 3,0 ; blr` leaf **emitted earlier in the same
obj** (section 10 vs `mmioClose`'s 14).

| # | prediction | direction if wrong |
|---|---|---|
| **P1.1** | M-HYP survives its grid. | If wrong, the rival that survives is named. |
| **P1.2** | **The interprocedural clause is real**: c2 tracks the clobber set of same-TU functions it has ALREADY emitted, and a value live across a call to such a clean leaf stays volatile. | If wrong, `mmioClose`'s `r5` has some other cause and R-M-A wins. |
| **P1.3** | **The tracking is EMISSION-ORDER-SENSITIVE**: the same clean leaf defined *after* the caller in the source forces the callee-saved pick, because c2 has not emitted it yet. Registered as the discriminating cell (**M4**). | If wrong, c2 has whole-TU knowledge and order does not matter. |
| **P1.4** | An **indirect** call (`bctrl`) always forces callee-saved, whatever else is in the TU. | — |
| **P1.5** | The allocation order for the callee-saved side is **r31 downward** (r31, then r30, …), matching `undname`'s `std r30/r31`. | — |

**Rivals, each with a per-cell prediction in §P3:**

* **R-M-A** — "any call at all": a value live across *any* call goes callee-saved.
  Predicts M3 and M8 callee-saved. **Separated by M3.**
* **R-M-C** — "whole-TU knowledge": c2 knows every same-TU callee's clobber set
  regardless of emission order. Predicts M4 volatile. **Separated by M4.**
* **R-M-D** — "not liveness at all": the register is a function of the formal's
  argument slot / IL temp index, and the correlation with calls is a confound.
  Predicts M1 and M2 take the *same* register. **Separated by M2.**

## §P2 — choice point **B** (`Biquad.cpp`): the pooled-constant `lis` placement

**What the base obj shows.** `?SetCoefficients` uses two `.rdata` float pools,
each its own COMDAT and its own symbol, so their `lis` cannot be shared (#1786's
high-half rule does not apply — the halves are relocations, not constants). The
two `lis` land in *different* places:

| pool | `lis` at | `lfs` at | uses |
|---|---|---|---|
| `__real@00000000` | **+0x00**, the function's first word, **above** the `cmplwi` at +0x04 | +0x08 | 4 in the then-arm, 2 after the join |
| `__real@3f800000` | **+0x10**, first word of the then-block | +0x24, 5 words later, immediately before its one use | 1, in the then-arm |

**B-HYP (registered).** The `lis` of a pooled constant is emitted at the **top of
the earliest basic block that dominates every use of that pool symbol**; the
`lfs` is emitted **at the use**, not with the `lis`. One `lis` per pool symbol per
function.

| # | prediction | direction if wrong |
|---|---|---|
| **P2.1** | B-HYP survives its grid: single-arm use ⇒ `lis` at top of that arm; use in both arms (or arm + join) ⇒ `lis` at top of the entry block, **above the compare**. | — |
| **P2.2** | The `lfs` stays at the use even when the `lis` is hoisted many words away. | — |
| **P2.3** | Two distinct pools dominated by the same block get **two** `lis` at the top of that block, in **first-use order**. | If wrong, source-declaration order or symbol-name order. |
| **P2.4** | A pool used only inside a loop gets its `lis` **outside** the loop (loop pre-header dominates). Registered as the optimistic call. | — |

**Rivals:** **R-B-A** "no hoist — `lis` immediately precedes its `lfs`"
(refuted at base by `__real@00000000`, re-tested for ≥3 witnesses);
**R-B-B** "every pool `lis` goes to function entry" (refuted at base by
`__real@3f800000`, likewise); **R-B-C** "the placement is the *first* use's
statement, i.e. no block-level hoist, and the entry-block case is a coincidence
of the 0.0f being needed by the first statement of the function".

### §P2b — choice point **B′**: the divisor load-order flip

Also in the base obj, and NOT part of any inherited description: the else-arm
issues five `fdivs` by the same divisor `flts[3]`, reloading it every time (no
CSE). Four of them load **divisor then dividend**; the fifth and last loads
**dividend then divisor**.

| # | prediction | direction if wrong |
|---|---|---|
| **P2.5** | The flip is on the **LAST** division of the run — a last-use rule — and reproduces at run lengths 2, 3, 4 and 6. | If wrong, the flip is not run-position-driven. |
| **P2.6** | Registered as the **pessimistic** call (board #770): I expect P2.5 to **MISS**, and the flip to be a scheduling artifact with no readable predicate — i.e. B′ is the one that gets the honest *"not mechanism-driven"* finding the success floor allows. | — |

## §P3 — the grid, frozen cell by cell

Coded outcome per cell: the **register class of the parked value** (`VOL` =
r3–r12, `SAV` = r14–r31 with a matching `std`/`ld` pair) for M-cells; the
**word index of the `lis` relative to its block and to the compare** for B-cells.
Sources live in `docs/whitebox/grids/wb-chooser/`; objs stay in
`work/wb-chooser/` and are never committed.

### Grid M — the park register (10 cells)

Every cell is one `.cpp` with one function under test, compiled at the
workload's own flags via `work/w-frame/refobj.sh`'s profile.

| cell | the function under test | **M-HYP** | R-M-A | R-M-C | R-M-D |
|---|---|---|---|---|---|
| **M1** | park a formal, use it later, **no call** | `VOL` | `VOL` | `VOL` | `VOL` |
| **M2** | park a formal live across `bl` to an **extern** fn | `SAV` | `SAV` | `SAV` | `VOL` (same reg as M1) |
| **M3** | live across `bl` to a same-TU `li 3,0;blr` leaf **defined EARLIER** | **`VOL`** | **`SAV`** | `VOL` | `VOL` |
| **M4** | live across `bl` to a same-TU clean leaf **defined LATER** | **`SAV`** | `SAV` | **`VOL`** | `VOL` |
| **M5** | live across a same-TU leaf (earlier) that itself calls an extern | `SAV` | `SAV` | `SAV` | `VOL` |
| **M6** | live across an **indirect** call through a loaded fn pointer | `SAV` | `SAV` | `SAV` | `VOL` |
| **M7** | **two** formals live across an extern call | `SAV`×2, **r31 then r30** | `SAV`×2 | `SAV`×2 | `VOL`×2 |
| **M8** | live across a same-TU earlier clean leaf, **two** such calls | `VOL` | `SAV` | `VOL` | `VOL` |
| **M9** | `mmioClose`'s own shape reduced: one value across a clean earlier leaf **and** one across an indirect call | one `VOL`, one `SAV` | `SAV`×2 | one `VOL` one `SAV` | `VOL`×2 |
| **M10** | live across an earlier same-TU leaf that is `__declspec(noinline)` and **non-leaf** (calls extern) | `SAV` | `SAV` | `SAV` | `VOL` |

**Asserted minimum of discriminating cells: 3.** M3 separates M-HYP from R-M-A;
M4 separates M-HYP from R-M-C; M2-vs-M1 separates every liveness hypothesis from
R-M-D. If fewer than 3 of these actually discriminate (e.g. two rivals collapse),
the grid is declared **inconclusive** and no chooser rule is handed on —
w-clear's confound clause, checked in advance.

**Witness accounting, registered before the run.** M-HYP earns its floor only if
the finished grid holds **≥ 3 cells landing `VOL` and ≥ 3 landing `SAV`**, each
cell's outcome predicted before it was compiled. Base contributes 2 and 2.

### Grid B — the pooled `lis` placement (7 cells)

| cell | shape | **B-HYP** | R-B-A | R-B-B |
|---|---|---|---|---|
| **B1** | one float constant, used only in the then-arm | `lis` = first word of then-block, `lfs` at use | `lis` adjacent to `lfs` | `lis` at fn entry |
| **B2** | one constant, used in **both** arms | `lis` at fn entry, **above** the compare | adjacent ×2 | at entry |
| **B3** | one constant, used in the then-arm **and after the join** | `lis` at fn entry, above the compare | adjacent ×2 | at entry |
| **B4** | one constant, used **only after the join** | `lis` at top of the join block | adjacent | at entry |
| **B5** | **two** distinct constants, both used only in the then-arm | two `lis` at top of then-block, **first-use order** | adjacent ×2 | both at entry |
| **B6** | one constant used **twice** in the then-arm | **one** `lis`, **two** `lfs` | one `lis` per `lfs` | one at entry |
| **B7** | one constant used only inside a **loop** body | `lis` in the pre-header, outside the loop | adjacent, inside | at entry |

**Asserted minimum of discriminating cells: 2** (B1 separates B-HYP from R-B-B;
B2 or B4 separates it from R-B-A). Same inconclusive clause as Grid M.
Witness floor: **≥ 3 cells with a hoisted-above-the-compare `lis` and ≥ 3 with a
block-local `lis`**, base contributing 1 and 1.

### Grid B′ — the divisor order flip (4 cells)

**B′1**–**B′4**: runs of **2, 3, 4 and 6** divisions by one common divisor.
**P2.5** predicts exactly one flip per cell and it is the last division;
**P2.6** (the registered pessimistic call) predicts this pattern does **not**
hold across all four.

## §P4 — decline clauses

1. If a grid cell's obj cannot be produced at the workload's flags, the cell is
   `NOOBJ`, **said so**, and not scored as a confirmation.
2. If a cell's function is emitted with a shape that makes the question
   vacuous (e.g. the value is rematerialized instead of parked, so no `mr`
   exists), the cell is `VACUOUS`, not silently counted.
3. **A reading that a cell refutes is RETRACTED, not hedged** (method doc §7,
   the `.bss` rule). If M-HYP's interprocedural clause (P1.2) is refuted, the
   whole M chooser drops to "liveness only" and P1.2 is a retraction in the
   findings, not a footnote.
4. If **neither** choice point clears its witness floor, this lane's deliverable
   is the written *"not mechanism-driven"* finding the success floor allows,
   plus a statement of what the grid DID establish — and #1770/#1792's two
   frontier rows are retired on that basis, not on a guess.
5. This lane adopts **nothing** into `crates/`. DISCLOSURE rows here are
   **drafts**; if the follow-on code lane does not carry one, no row is added.
6. No prediction in §P1–§P3 was written after reading a byte of
   `~/ghidra-projects/export/c2/`. The mechanism reading in the findings doc is
   scored *against* these, not merged into them.
