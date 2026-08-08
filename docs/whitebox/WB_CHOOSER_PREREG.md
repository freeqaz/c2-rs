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
