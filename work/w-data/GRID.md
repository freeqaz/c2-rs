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

---

# GRID C — the MULTI-OBJECT obj, frozen before the cell was compiled

`emit_comdat_obj` shipped refusing more than one defined object, on the honest
ground that every rule in it was read off **one** obj. Then the positive fixture
turned out to be exactly that refusal: three cells in one file is three defined
objects, and `c2rs gap` read
`codegen-gap … a /Gy obj whose defined COMDAT data is outside the measured
class`.

So the fence is doing its job and the question is whether it can be **graded**
away rather than kept. One `cl.exe` on the three-function file settles it, and
the two readings are frozen here first.

## The rivals

| | prediction |
|---|---|
| **R1 — GROUPED** (what the writer already emits) | every `.text` COMDAT first, in emission order, then every `.data` COMDAT, in the same order. Sections: `.drectve .debug$S .XBLD$W .XBLD$W .text .text .text .data .data .data` |
| **R2 — INTERLEAVED** | each function's `.data` immediately after its own `.text`: `… .text .data .text .data .text .data` |

**R2 is the live rival and not a straw man.** `emit_comdat_obj` already
interleaves for `.pdata` — a framed function's `.pdata` COMDAT is emitted
*immediately after* its own `.text` COMDAT and tied to it by
`SELECT_ASSOCIATIVE` — and `emit_obj`'s own comment records that the **packed**
layout interleaves `.rdata` and `.pdata` in `.text` order, six distinct orders
over 240 objs. An emitter that groups where c2 interleaves is wrong about the
section table, the section indices, the symbol indices and every relocation's
`SymbolTableIndex` at once.

A COMDAT `.data` is **not** associative — `Primes.cpp`'s reads `Selection = 2`
(ANY), not 5 — so nothing ties it to a `.text` the way `.pdata` is tied. That is
the reason to expect R1, and it is an argument rather than a measurement, which
is why the cell is being cut.

## Predicted symbol table under R1

```
  0 @comp.id  1/2 .drectve  3/4 .debug$S  5/6 XBLD$W(C2)  7 __C2_11886
  8/9 XBLD$W(C1)  10 __C1_11886
  11/12 .text(p0)  13 ?p0    14/15 .text(p1)  16 ?p1    17/18 .text(p2)  19 ?p2
  20/21 .data(p0)  22 <p0's array>
  23/24 .data(p1)  25 <p1's array>
  26/27 .data(p2)  28 <p2's array>
```

29 symbols. Under R2 the same 29 records appear in a different order and every
relocation's index moves, so the two are separated by the obj and not only by
the section table.

## Predicted alignment nibbles

`p0` 32 B → ALIGN_4 (`0xC0301040`); `p1` 256 B → ALIGN_8 (`0xC0401040`);
`p2` 32 B → ALIGN_4. If `p1` reads ALIGN_4 the size promotion is not the rule
and `Primes.cpp`'s `0xC0401040` came from somewhere else.

## The decline, restated with its size

If the cell says **R2**, the writer is wrong and the fence stays at one object:
the fixture is then split into three one-function files and GRID C is reported
as a refuted prediction, not as a widening. **Size of what that declines: two
of the three positive cells as a single obj, and the `undname`/`osfinfo` row
entirely** — those need two objects in one *body*, which is a further step again.
