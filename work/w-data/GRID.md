# w-data — GRID B, the fixture fence

**Frozen before the first `cl.exe` on any cell of it.** Every prediction below
is written down before the cell exists as an obj; the outcomes are filled in
afterwards and the misses are kept.

The discipline is w-cfgclass §6.2's: **each negative cell must fail for its OWN
clause**, not for a clause an earlier cell already holds. `c2rs census` reports
only the fall-through blocker, so a `_neg` file can look complete while two of
its cells trip the same gate — that lane was bitten by exactly this on its first
draft and had to print the recognizer's own decline context per cell to find it.
Here the recognizer's clause names are distinct strings, so the check is
mechanical: six cells must produce six different `scan-*` keys.

## The class, restated so the cells can be read against it

`c2_core::codegen::static_scan_loop` emits **sixteen words with ZERO free
immediate fields**. Everything that varies across the class is the *object*:
its symbol name, its size, its natural alignment and its bytes. So:

* two positive cells whose arrays differ only in **contents** must have
  **byte-identical `.text`** and differ only in `.data` and the string table;
* a cell whose array differs in **size across the 64-byte promotion boundary**
  must change the `.data` section's alignment nibble and nothing in `.text`.

## Positive cells — predictions (must be `match` at `/O1`)

| cell | source | predicted `.data` size | predicted `Characteristics` | predicted `.text` |
|---|---|---:|---|---|
| **p0** | `static int a[8] = {…,0}` in a one-`int` function | 32 | `0xC0301040` (ALIGN_4, nibble 3) | the 16 words |
| **p1** | the same with `int a[64]` | 256 | `0xC0401040` (ALIGN_8, nibble 4) | **byte-identical to p0** |
| **p2** | the same as p0, different function name, different array name, different values | 32 | `0xC0301040` | **byte-identical to p0** |

**p1 is the separating cell.** It is the only one that crosses
`placement_align`'s 64-byte promotion, and it separates *"the nibble comes from
the `.gl` TYPE tag"* (which would give ALIGN_4 for both) from *"the nibble is
the size-promoted alignment"* (which gives ALIGN_8 here). The workload's own
`Primes.cpp` is 248 bytes and therefore on the promoted side; without p1 the
whole class is graded on one side of the boundary only.

**p2 is the zero-free-fields check.** If p0 and p2's `.text` ever differ, the
class has a field the emitter is not modelling and the transcription is wrong.

## Negative cells — predictions (must be `NotImplemented`, NEVER `Mismatch`)

| cell | what varies | predicted refusal | where |
|---|---|---|---|
| **n0** | the array is **namespace-scope** `static`, not function-local | non-COMDAT | `resolve_data_def`'s `!o.comdat` |
| **n1** | the array is **uninitialized** | `.bss` COMDAT | `resolve_data_def`'s `!o.initialized` |
| **n2** | element type **`short`** | scale is 2, not 4 | `scan-test-subscript` (the `INT_SCALE` clause) |
| **n3** | the guard is **`>`**, not `>=` | relation byte is not `23` | `scan-guard-not-ge` |
| **n4** | **two** formals | arity | `scan-formals-not-1` |
| **n5** | the index starts at **1**, not 0 | the init literal | `scan-index-init-not-zero` |

Six cells, six distinct clauses. n0 and n1 refuse in the **object** resolver and
n2–n5 in the **body** recognizer, so both halves of the fence have cells.

### What the negatives do NOT claim

n0 and n1 are shapes c2 emits perfectly well and this port refuses — a fence
**narrower** than the class, in the safe direction, exactly as w-cfgclass's `n5`
records for bracing. Widening either needs its own graded cells: n0 needs the
before-`.text` slot (`emit_data_obj`'s slot `A`/`B` logic, which serves a
different writer) and n1 needs a `.bss` COMDAT the differential has never seen.

## Outcomes

Filled in after the run; see `work/w-data/GRID_RESULT.md`.
