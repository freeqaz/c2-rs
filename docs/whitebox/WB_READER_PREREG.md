# WB-A `wb-reader` — PREREG

> **PROVENANCE — DISASSEMBLY-DERIVED.** See
> [`C2_MAP_METHOD.md`](C2_MAP_METHOD.md) §0 for the exact bytes and
> [`DISCLOSURE.md`](DISCLOSURE.md) for what adoption costs. Nothing here is
> adopted into `crates/`.

Registered **before** the frontier listing was grouped and before any probe
existed, per board #770's standing rule (streak ~10 optimistic / 2 pessimistic /
1 hit). Scored in [`WB_READER_FINDINGS.md`](WB_READER_FINDINGS.md) §7.

Lane: `wb-reader` / `wt-wb-reader`, branched at master `c34c388c`
(match 11 · mismatch 0 · codegen-gap 0 · vocab-gap 860 · capture-fail 7 ·
FRONTIER 16 = 59 emitted functions = 10 exact + 1 wrong + 0 cg-refused +
**48 reader-refused**).

## P1 — family sizes of the 48

| # | prediction | direction if wrong |
|---|---|---|
| P1.1 | the largest single first-blocker key holds **8–16** of the 48 | — |
| P1.2 | the top 3 keys together hold **≥ 24** of the 48 (half) | — |
| P1.3 | the 48 fall into **≥ 12** distinct keys (a long tail, not 3 families) | — |
| P1.4 | `src/keygen_xbox.cpp` alone contributes **18** of the 48 and its keys are **not** all one family | — |

## P2 — decode difficulty

| # | prediction |
|---|---|
| P2.1 | c2's `.ex` reader dispatches operand *width* off a **per-opcode table**, not a per-opcode function, so one table read gives the width grammar for **all** 256 opcodes at once |
| P2.2 | at least one of the top-3 families' opcodes turns out to take **zero** operand bytes — i.e. the census key is a *semantic* refusal wearing a reader's name |
| P2.3 | **≥ 1** place where the port's published width disagrees with c2's table (a live latent desync) |
| P2.4 | the reading for the largest family is obtainable in this lane (`high` in the C2_MAP sense) |

## P3 — the thing this lane is actually for

| # | prediction |
|---|---|
| P3.1 | **HEADLINE, registered as the pessimistic call**: the 48 are **not** grammar-bound. The port's own width scanner (`control_flow::scan_full`) already walks **≥ 40 of the 48** bodies end-to-end, so decoding c2's reader recovers **0** functions by itself. |
| P3.2 | recovered-vs-renamed on the 48, *if* every top-3 family's construct were given a reader model and nothing else changed: **recovered 0, renamed ≥ 44**. |
| P3.3 | frontier TUs converted by anything this lane can ship: **0** (this is a docs lane). |

## P4 — black-box checks

Coded outcomes: `IDENT` (obj byte-identical to the replayed baseline with
`TimeDateStamp` zeroed) · `DIFF` (obj produced, bytes differ) · `NOOBJ`
(c2 produced no obj — an ICE or a refusal).

Frozen cell predictions, written before the first `cl.exe` of the grid:

| cell | edit | prediction |
|---|---|---|
| **A0** | rewrite `1F` over itself | `IDENT` |
| **A1** | `1F` → `20` (table says same operand class) | `DIFF` |
| **A2** | `1F` → `27` (table says a different class — one TYPE) | `NOOBJ` |
| **A3** | `1F` → `26` (table says a different class — one symbol token) | `NOOBJ` |
| **B0** | rewrite a `3A` jump token over itself | `IDENT` |
| **B1** | retarget the `3A` token to another label token of the same body | `DIFF` |
| **B2** | byte-swap the `3A` token (`lo hi` → `hi lo`, `lo ≥ 0x80`) | `NOOBJ` — the token is a **little-endian `varU` with a bit-15 continuation**, so a swapped pair sets the continuation and eats two more bytes. A big-endian rival predicts `IDENT`; a plain-2-byte-LE rival predicts `DIFF`. |
| **C0** | rewrite a `27` TYPE over itself | `IDENT` |
| **C1** | `27`'s TYPE **class nibble** `…43` → `…41` (same width, ptr → signed) | `IDENT` — the reading says the TYPE reader **skips the whole classification tail when the opcode is `0x27`**, so the nibble is never consumed. The rival ("`27`'s type is classified like every other operand's") predicts `DIFF`. |
| **C2** | `27`'s TYPE tag `A6` → `C6` (set bit `0x40`) | `NOOBJ` — the wide bit makes the type word 3 bytes instead of 2, desynchronising by one. |

**Decline clauses.** Declared in advance so a silent drop cannot be read as a
result:

1. If the capture/replay harness cannot reproduce the pipeline obj byte-exactly
   on the **baseline** for a TU, that TU is dropped and **said so**; no cell of
   it is scored.
2. If a cell's site cannot be located unambiguously in the `.ex` (the census hex
   window is not unique), the cell is `SKIPPED`, not guessed.
3. If ≥ 2 of the 11 cells miss, the *whole* class-table reading drops to
   `medium` in `C2_MAP.md` rather than being patched cell by cell — the `.bss`
   rule (method doc §7): a reading an obj refutes is retracted, not hedged.
4. This lane adopts nothing. Pre-drafted DISCLOSURE rows are drafts; if a code
   lane does not carry one, no row is added.
