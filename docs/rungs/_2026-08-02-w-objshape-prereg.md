# Pre-registration — lane `w-objshape`, board #158 obj-shape half (2026-08-02)

Written and committed **before the first capture**. Scored verbatim in
`docs/OBJ_DYNINIT_SHAPE.md`; wrong predictions stay on the page.

## What I am measuring

The obj the real `c2` emits for a namespace-scope object with a non-trivial
constructor — the `??__E` dynamic-initializer thunk. Standalone reproduction is
`fixtures/cpp/il_dyninit_static.cpp`:

```cpp
struct L { L(const char* s, int r); };
static L sL("abc", 0);
```

Reference obj only. No code under `crates/` is touched by this lane.

## My bias, in writing

**I want this shape to come out regular and derivable** — a small set of
sections in a fixed order, one relocation per operand, a section/symbol layout
that extends the existing four-section shell by concatenation. That result makes
the emit half of #158 look cheap and makes this lane look productive. The
failure mode it points at is **fitting a rule to the two-line fixture and
calling it the shape**, when the workload TUs (`TomCryptLicense`, `ZlibLicense`)
or the destructor/two-object variants move fields the fixture holds constant.

Guard against it: every "CONST" claim must be CONST **across the varied grid**,
not across one capture; anything constant on only one cell is reported as
"unvaried", not as constant. And the honest outcome "this is not derivable from
the IL the port has" is registered here in advance as a **good** result, not a
failure of the lane.

## Registered predictions

Point predictions on `il_dyninit_static.cpp` at the fixture flags (`/Ox /GS- /c`).

| # | prediction | my confidence |
|---|---|---|
| P1 | **8 sections**, in order `.drectve`, `.debug$S`, `.XBLD$W`(C2), `.XBLD$W`(C1), `.text`, `.rdata`, `.bss`, `.CRT$XCU` | medium — the count more than the order |
| P2 | `.text`, `.rdata`, `.bss` are **COMDAT** (`LNK_COMDAT 0x1000` set); `.CRT$XCU` is **not** COMDAT | low-medium on `.CRT$XCU` |
| P3 | Every COMDAT here has selection kind **1 (NODUPLICATES)**, as every `/Gy` `.text` COMDAT does (`OBJ_GY_SHAPES.md`) | medium |
| P4 | **24 symbol records** total (11 shell + 4 × (section sym + aux) + 4 owned symbols + 1 undefined `??0L@@QAA@PBDH@Z`) | low — this is a compound count and any one term being off breaks it |
| P5 | **5 relocations on `.text`**: `REFHI`(string), `REFHI`(`sL`), `REFLO`(string), `REFLO`(`sL`), `REL24`(`??0L@@QAA@PBDH@Z`) | medium. **Named alternative:** classic PPC COFF requires a `PAIR` record after each `REFHI`, which would make it **7** |
| P6 | The `REFHI`/`REFLO` halves are **NOT adjacent** — both `lis` are hoisted above both `addi`, so the reloc offsets interleave `0,4,8,12` = HI,HI,LO,LO | medium-high (the listing shows the hoist; but the listing is not the obj) |
| P7 | **1 relocation on `.CRT$XCU`**, `ADDR32`, targeting `??__EsL@@YAXXZ`, at offset 0 | high |
| P8 | `.bss` `SizeOfRawData = 0`, and the **section header** carries the 1-byte size in `VirtualSize` (or the aux `Length`) | low-medium |
| P9 | `sL$initializer$` is a **STATIC** symbol (storage class 3), not EXTERNAL | low — could equally be EXTERNAL to survive the linker |
| P10 | `FIKCJHKP` is a **32-bit checksum of the string bytes**, encoded 4 bits per character over the alphabet `A`..`P` (`A`=0 … `P`=15), i.e. `0x58A297AF` | low. Registered so it can be scored **wrong** |

## Varied-grid predictions (structural axes, crossed; values varied inside cells)

The axes, chosen before capture. Argument **count** is structural; argument
**value** is not.

* A1 object size — `char`-sized vs a multi-word struct (`.bss` size)
* A2 constructor arity — 0, 1, 2, 3 arguments
* A3 argument type — `const char*` vs `int` vs `float` (float is expected to
  drag in `.rdata` FP pooling and `_fltused`, per `OBJ_GY_SHAPES.md` §1)
* A4 destructor present — adds a `??__F` atexit thunk and an `atexit` call
* A5 object count in the TU — one vs two static objects

| # | prediction |
|---|---|
| Q1 | The **four-section shell is invariant** across the whole grid (same order, same characteristics, same 11 leading symbol records) |
| Q2 | `.bss` `SizeOfRawData`/`VirtualSize` **tracks `sizeof(T)`**; nothing else moves with A1 |
| Q3 | A2 (arity) moves **only** `.text` size and the `.text` relocation count; the section set is unchanged for pointer/int arguments |
| Q4 | A4 (destructor) adds **`atexit` as an undefined external** and a `??__F` `.text` COMDAT — i.e. **+1 section, +3 symbol records at least** |
| Q5 | A5 (two objects) yields **two `.CRT$XCU` sections**, not one section with two entries |
| Q6 | Section **order** follows first-use, not a fixed table — i.e. reordering the source reorders `.rdata`/`.bss` |

## Decline floor, set in advance

If, after the grid, any of the following holds, this lane **declines** the emit
half rather than reporting a fitted rule:

* the `.CRT$XCU` / `.bss` / `.rdata` **section order or count** is not a
  function of anything visible in the IL the port has, or
* the string COMDAT's mangled name (`??_C@_03FIKCJHKP@abc?$AA@`) cannot be
  **computed** from the string bytes — because that name is a symbol the port
  must emit and cannot copy from `.gl` if `.gl` does not carry it, or
* ≥3 of Q1–Q6 come out false, which would mean the shape is not the
  concatenation-of-shell it looks like.

An honest decline with byte evidence is the registered good outcome.

## Out-of-sample protocol

Any ordering or counting rule derived from the first grid is committed as a
**prediction file** (git object) **before** the held-out cells are compiled.
The held-out set is chosen after the rule is written, from axes the rule did not
see.
