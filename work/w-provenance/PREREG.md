# PREREG — lane `w-provenance` (decision 15, board `#3629`–`#3634`)

    Lane:      w-provenance
    Kind:      instrument
    Base tree: 6c753ead0947c47b146db42d3921362b8fc99d63 (master tip at dispatch)
    Branch:    wt-w-provenance
    Date:      2026-08-26
    Charter:   docs/DECISIONS_2026-08-22.md § "Decision 15" — the derived-vs-fitted census

**Committed before the first census count.** Everything under §1 is a
denominator I re-measured by reading, and I say so; everything under §2 is a
blind prediction made before the counting instrument existed. §4 is the decline
floor, frozen here.

---

## 0. What already existed, checked before building (the "twice briefed an
existing doc" rule)

Checked at base `6c753ead0`:

* `scripts/provenance_*` — **does not exist.** `ls scripts/` has no file with
  that prefix.
* A per-module provenance counter — **does not exist.** No script in
  `scripts/` counts markers in `crates/`.
* Prior art with the same NAME but a different subject — **`#207` /`#202`,
  lane `w-prov` (2026-08-04): "Census provenance, enforced"**, which writes a
  `.prov` sidecar beside every *census* recording which dc3 corpus it was
  graded against, with `prov.py selfcheck` (21 PASS / 0 FAIL). That is
  **corpus provenance of a measurement**. This lane is **derivation provenance
  of a constant** — where a number in `crates/` came from. Disjoint subjects,
  colliding word; the doc must disambiguate them explicitly.
* The `[R]`/`[O]`/`[I]` vocabulary — **exists**, defined in
  `docs/whitebox/ref/README.md` §2 ("the provenance legend"), applied to
  *claims on whitebox pages*, never to code. This lane extends it into
  `crates/`, it does not invent it.

## 1. Denominators re-measured on MY tree (read, not predicted)

| figure | brief says | measured at `6c753ead0` | note |
|---|---|---:|---|
| `DISCLOSURE.md` length | 247 lines | **247** | `grep -c "" docs/whitebox/DISCLOSURE.md` |
| in-`crates/` provenance markers | "roughly 6 (≈3 `[R]`, 2 `[O]`, 1 `[src]`)" | **5 real, 1 false positive** | `[R]`×3 (`codegen/schedule.rs:23`, `codegen/order.rs:19`, `codegen/alloc.rs:51`), `[O]`×2 (the same `schedule.rs:23` and `order.rs:19` lines — **both `[O]`s are on lines that also carry an `[R]`**), `[src]`×1 = `params[src]` in `c2-il/src/func/mod.rs:2013`, **not a marker at all** |

**Two corrections to the brief, recorded before any count:**

1. **The "6" is 5, and it is 3 lines, not 6 sites.** `[O]` never appears
   without `[R]` beside it; the whole in-crates marker population is three
   prose sentences in three module doc-comments in `c2-core/src/codegen/`,
   none of them attached to a constant.
2. **`[src]` is not a marker.** It is an array index in a comment. Any census
   built on a bare-bracket grep inherits this false positive, which is the
   first argument for a **prefixed** marker token (§3).

## 2. Predictions — registered blind, before the counter exists

| # | prediction | conf | direction if wrong |
|---|---|---:|---|
| **P1** | `DISCLOSURE.md`'s adopted-findings table has **17** rows (`W-OBJPLAN-1`, `W-ALIAS-1/2`, `W-MEMCPY-1`, `W-GLATTRS-1`, `W-STAGETAP-1..6`, `W-MID-1..4`, `W-SEEDGAP-1`, `W-SUB4F-2`) | 0.95 | this one is from reading the file, so it is a transcription check, not a blind call |
| **P2** | of those 17, **11** name a `crates/` path in `Adopted into` | 0.55 | — |
| **P3** | rows whose cited `crates/` site **no longer exists** (dead citation): **2** | 0.40 | **OPTIMISTIC bias registered** — I expect this repo to be tidy and I expect to be wrong in the "more dead than I guessed" direction |
| **P4** | `const`/`static` items in non-test code under `crates/c2-core/src/codegen/**`: **120–250** | 0.60 | — |
| **P5** | ditto under `crates/c2-core/src/coff/**`: **60–120** | 0.60 | — |
| **P6** | ditto in the `c2-il` shapes/admission vocabulary files: **40–100** | 0.50 | — |
| **P7** | after the seed pass, `PROV[R]` markers in `crates/`: **8–15** | 0.50 | the DISCLOSURE rows adopt few *named constants*; several adopt layouts that live in code shape, not in a `const` |
| **P8** | fraction of the scoped constant population tagged at this lane's tip: **15–30 %** | 0.45 | 100 % is explicitly not required |
| **P9** | the largest tag class at tip is **`[F]`**, not `[R]` | 0.65 | if `[R]` wins, the port is better-read than the brief's premise assumes and the census's headline flips |
| **P10** | at least one DISCLOSURE row's `Adopted into` names a symbol whose **name still exists but whose file moved** — i.e. a partially-dead citation the row count cannot express | 0.50 | forces a three-way live/moved/dead split rather than two-way |

## 3. The convention, frozen before it is applied

* Marker token is **`PROV[X]`**, X ∈ `R` `O` `F` `S` `N`, in a Rust comment.
  **Prefixed deliberately**, because §1's `[src]` false positive proves a bare
  bracket grep cannot tell a marker from an array index, and because three
  existing prose `[R]`s would otherwise be counted as tags they are not.
* `R` read from disassembly · `O` confirmed against a real obj/listing ·
  `F` fitted to observations · `S` specified by a published external standard
  (PE/COFF, PowerPC ISA — neither read from c2 nor fitted) · `N` not
  load-bearing, with a reason.
* Every marker carries a citation after ` — `: a DISCLOSURE row id, a
  `doc.md §n`, or a rung/board id. A marker with an empty citation is a
  **defect the counter reports**, not a tag.

## 4. Decline floor — frozen

* If **fewer than 5** DISCLOSURE rows resolve to a live `crates/` constant,
  deliverable 2 (the seed pass) is reported **FAILED as a seed** and the lane
  ships the convention + counter only, saying so in those words.
* If the counter cannot produce a **denominator** for any scoped module, the
  lane does not publish a ratio for that module — it publishes the numerator
  and the words "no denominator", per `#3470`/`#1002`.
* If the positive control cannot be **watched failing**, the control is not
  shipped and the lane reports `FAILED` on deliverable 4. A control never seen
  red is decoration (`#3336`).

## 5. What the numbers will NOT license, registered here so it cannot be added later

* **A high `[R]` count licenses no emit.** `docs/FUNCTION_BYTE_MATCH.md` §0's
  separation rule: this census is published beside FBM, never in `gate.sh`.
* **This census must not become a ranking instrument.** `#3505`'s family —
  four of four lanes dispatched off a size ranking found the ranking measured
  itself. The tracked signal is the **CHANGE per module**, never the distance
  from 100 % `[R]`, and no module list in the output is sorted by count.
* **No constant's VALUE changes in this lane.** Comment-only edits, graded
  required-zero by `scripts/gate_identity_diff.sh` over its 21 rows.
