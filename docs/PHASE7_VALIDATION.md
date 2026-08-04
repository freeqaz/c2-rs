# PHASE7_VALIDATION — the out-of-sample gate for the fitted emit predicate (#161)

    Lane:      w-emitpred, 2026-08-02 (relaunched 2026-08-04)
    Prereg:    rungs/_2026-08-02-w-emitpred-prereg.md — committed at `a5f355c`,
               before any cell was compiled or any held-out truth read. Floors,
               incumbents, populations and verdict rules are all there and are
               not restated here in any way that could drift from them.
    Status:    IN PROGRESS. Every results section below is either filled with
               measured numbers or marked PENDING; a PENDING section is not a
               result (§9.18.8: absence reads as success unless forbidden —
               this line forbids it).
    Provenance: `../dc3-decomp` @ **51fb5b73** (moved again since the plan
               session's 13b583df; nothing cached from an older rev is
               byte-comparable). Toolchain `compilers/X360` under wibo.

**What this doc decides:** whether #161 — the emit predicate fitted black-box
on 172 designed cells with zero violations (`PHASE7_PLAN.md` §2) — survives
contact with anything it was not fitted on, and whether it may ever ship into
R3. In-sample 360/360 has already burned this project once; the number that
decides is the one that cannot be revised.

---

## 1. The gate, in one table

| part | population | discipline |
|---|---|---|
| Held-out PROC-set prediction (D1(b)) | 20 real workload TUs, seed-161 draw, frozen in the prereg | predictions committed as a git object **before** any truth artifact is read; one shot; no re-fitting |
| Structural axes (the ones 172 cells could not vary) | 9 axes, ≥4 designed cells each | per-cell predictions hand-derived from §2's text and committed **before** the axis's first compile; violations count only after an independent re-derivation |
| Warning-channel cross-check (D1(a)) | DEV TUs + probe cells | reported, not gated; attributes Part-1 misses to a half |

Incumbents registered as controls (never a bare threshold): **never-emit**
(~93 % per-body accuracy on this workload) and **emit-everything** (the port's
current behaviour). The predicate must beat both on the same universe by
≥ 2.0 pp, or it is refuted as a model regardless of anything else.

## 2. Held-out gate — result

**PENDING.** Predictions commit: _not yet made_. Truth read: _not yet
permitted._

| metric | registered floor | measured |
|---|---|---|
| micro-F1 | ship ≥ 0.80; refuted < 0.50 | PENDING |
| micro-accuracy vs both incumbents (same universe) | ≥ +2.0 pp over each | PENDING |
| micro-precision | ship ≥ 0.95 | PENDING |
| per-TU exact sets (of 20) | reported only | PENDING |
| F1 excl. synthesized-name families | attribution only | PENDING |

## 3. Structural axes — results

**PENDING.** Predictions committed per axis before its first compile; see
`rungs/_2026-08-02-w-emitpred-prereg.md` Part 2 for the axis list A1–A9.

| axis | cells | verdict |
|---|---|---|
| A1 header inclusion depth | PENDING | PENDING |
| A2 template instantiation | PENDING | PENDING |
| A3 virtual/multiple inheritance, thunks | PENDING | PENDING |
| A4 anonymous namespaces | PENDING | PENDING |
| A5 static/inline/extern "C" crossings | PENDING | PENDING |
| A6 multi-TU shared header | PENDING | PENDING |
| A7 pragma-created roots | PENDING | PENDING |
| A8 PCH | PENDING | PENDING |
| A9 vtable kept without kept ctor (D6) | PENDING | PENDING |

## 4. Warning-channel on real headers — result

**PENDING.**

## 5. Verdict

**PENDING.** The verdict rule is fixed in the prereg (SHIP-CANDIDATE /
SURVIVES-NOT-SHIPPABLE / REFUTED-ON-REAL / DECLINE / INSTRUMENT-FAIL) and the
verdict will be one of those five words, with the numbers that forced it.

## 6. Clean-room ledger

All channels used are black-box under ROADMAP §9.8's existing blessing:
compile-and-observe probe cells, `/Wall` C4505/C4514 stderr, `/FAsc` PROC
name sets (names only), obj byte analysis of our own captures, `.gl` reads
via the separator-aware extractors. Disassembly-derived constants adopted:
**none** so far; if that changes it will be disclosed here per-finding.
