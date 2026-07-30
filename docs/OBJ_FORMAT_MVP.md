# MVP obj format — COFF byte map (MSVC 16.00.11886.00, X360 PPC)

Byte-level spec of the object file c2 emits for the MVP function class
(single straight-line integer leaf function, `/Ox /GS- /c`, no relocations).
Recovered by cross-diffing fixtures: `mvp` (`int add3(int,int,int)`, 790 B),
`add3` (4 functions, 953 B), `sub2` (`int sub2(int,int)`, 785 B). Every field
is classified **CONST** (identical across fixtures — hardcode) or **DERIVED**
(computed from content — thread from real state).

All COFF struct fields are **little-endian**, even though the `.text` PPC
payload is big-endian.

## File-level layout (mvp, 790 B)

```
0x000  COFF header            20 B
0x014  Section headers        5 × 40 = 200 B
0x0DC  .drectve   raw         0x84 (132)
0x160  .debug$S   raw         0x64 (100)
0x1C4  .XBLD$W#1  raw (C2)    0x10 (16)
0x1D4  .XBLD$W#2  raw (C1)    0x10 (16)
0x1E4  .text      raw         0x0C (12)
0x1F0  symbol table           14 × 18 = 252 B
0x2EC  string table           42 B → EOF 0x316
```

Section raw data is **packed contiguously — no inter-section padding**.
`PtrToRawData[n] = PtrToRawData[n-1] + SizeOfRawData[n-1]`, starting at
`0x14 + 40*nSections`. `PointerToSymbolTable = last Ptr + last Size` (no
relocation blocks in the MVP). The file ends exactly at the end of the
string table — no trailing padding.

## COFF file header (20 B @ 0x00)

| Off | Field | Value | Class |
|----|-------|-------|----|
| 0x00 | Machine u16 | `0x01F2` IMAGE_FILE_MACHINE_POWERPCBE | CONST |
| 0x02 | NumberOfSections u16 | 5 | CONST (this shape) |
| 0x04 | TimeDateStamp u32 | wall clock | DERIVED — the only field `c2-obj::normalized()` zeroes |
| 0x08 | PointerToSymbolTable u32 | 0x1F0 | DERIVED (formula above) |
| 0x0C | NumberOfSymbols u32 | 14 | DERIVED (counts aux slots) |
| 0x10 | SizeOfOptionalHeader u16 | 0 | CONST |
| 0x12 | Characteristics u16 | `0x0180` = 32BIT_MACHINE \| BYTES_REVERSED_LO | CONST |

## Section headers (40 B each)

Fixed order: `.drectve`, `.debug$S`, `.XBLD$W`(C2), `.XBLD$W`(C1), `.text`.
For every MVP section: VirtualSize/VirtualAddress/PtrToRelocations/
PtrToLinenumbers/NumberOfRelocations/NumberOfLinenumbers all 0.

| # | Name (8 B NUL-padded) | SizeOfRawData | Characteristics | Decode |
|---|---|---|---|---|
| 0 | `.drectve` | 0x84 CONST | `0x00100A00` | LNK_INFO \| LNK_REMOVE \| ALIGN_1 |
| 1 | `.debug$S` | formula below | `0x42100040` | CNT_INIT_DATA \| ALIGN_1 \| MEM_DISCARDABLE \| MEM_READ |
| 2 | `.XBLD$W\0` | 0x10 CONST | `0xC0401040` | CNT_INIT_DATA \| LNK_COMDAT \| ALIGN_8 \| MEM_READ \| MEM_WRITE |
| 3 | `.XBLD$W\0` | 0x10 CONST | `0xC2301040` | CNT_INIT_DATA \| LNK_COMDAT \| ALIGN_4 \| MEM_DISCARDABLE \| MEM_READ \| MEM_WRITE |
| 4 | `.text\0\0\0` | codegen size | `0x60400020` | CNT_CODE \| ALIGN_8 \| MEM_EXECUTE \| MEM_READ |

All five Characteristics are CONST across fixtures. The two `.XBLD$W`
sections are NOT identical: C2 is ALIGN_8/not-discardable, C1 is
ALIGN_4/discardable.

## `.drectve` raw (132 B) — 100% CONST

Exact ASCII, no NUL terminator, 3 leading spaces, 1 trailing space:

```
   /include:__C1_11886 /DEFAULTLIB:"OLDNAMES" /DEFAULTLIB:"LIBCMT" /DEFAULTLIB:"XAPILIB" /DEFAULTLIB:"XBOXKRNL" /include:__C2_11886 
```

`11886` is the compiler build number (constant for this toolchain).

## `.debug$S` raw — CodeView; CONST except the `-Fo` path

Layout: u32 CV signature `0x00000004` (C13) · u32 subsection type `0xF1`
(DEBUG_S_SYMBOLS) · u32 subsection length (content bytes, excl. this header)
· records · tail-pad the WHOLE subsection to a 4-byte multiple with 0x00.

**Record 1 — S_OBJNAME (`0x1101`):** `u16 reclen` (= 2+4+strlen+1) ·
`u16 0x1101` · `u32 signature=0` · the `-Fo` output path, NUL-terminated.
**This is the single derived wiring point of the whole section**: the
emitter must receive the exact obj-path string the reference `-Fo` used
(e.g. `Z:\...\mvp.obj`). It comes from c2's argv, NOT from the IL bundle.

**Record 2 — S_COMPILE2 (`0x1116` — note: COMPILE2, not COMPILE3; no QFE
fields):** 100% CONST, 57 bytes verbatim:

```
37 00 16 11 01 02 00 00 42 00 10 00 00 00 6e 2e 10 00 00 00 6e 2e
4d 69 63 72 6f 73 6f 66 74 20 28 52 29 20 4f 70 74 69 6d 69 7a 69
6e 67 20 43 6f 6d 70 69 6c 65 72 00 00
```

(flags `0x0201` = C++, machine `0x0042` = CV_CFL_PPC604, FE/BE version
16.0.11886, `"Microsoft (R) Optimizing Compiler"` + double NUL.)

**Packing rule:** CV records are packed back-to-back with **no inter-record
alignment** (proof: in add3.obj record 2 starts at an odd offset). Only the
subsection end is padded.

**Size formula (compute, never hardcode 0x64):**

```
reclen1       = 2 + 4 + strlen(objpath) + 1
subsec_len    = (2 + reclen1) + 57          // = 66 + strlen
SizeOfRawData = align4(12 + subsec_len)     // = align4(78 + strlen)
```

A path length crossing a 4-boundary changes `SizeOfRawData` and shifts the
entire downstream `PtrToRawData` chain.

## `.XBLD$W` raw (16 B each) — CONST watermarks

```
C2: 43 32 00 00 00 00 00 00  00 10 00 00  2e 6e 44 00
C1: 43 31 00 00 00 00 00 00  00 10 00 00  2e 6e 44 00
```

Byte 1 (`'2'`/`'1'`) is the only difference; `2e 6e` = 11886 LE. Content is
source-independent — emit the literals.

## Symbol table (14 × 18 B @ PointerToSymbolTable)

Record layout: `Name[8] | Value u32 | SectionNumber i16 | Type u16 |
StorageClass u8 | NumberOfAuxSymbols u8`.

**Name encoding:** len ≤ 8 → inline, NUL-padded (len-8 names have no NUL).
len > 8 → first u32 = 0, second u32 = LE byte offset into the string table
(offset includes the table's 4-byte size word, so the first name is at 4).

**Type:** `0x0020` iff function symbol (DTYPE_FUNCTION<<4); else 0.

Exact emit order (mvp):

| Slot | Name | Value | Sec | Type | Class | nAux |
|---|---|---|---|---|---|---|
| 0 | `@comp.id` | **0x00AB2E6E** | -1 ABS | 0 | 3 STATIC | 0 |
| 1+2 | `.drectve` + aux | 0 | 1 | 0 | 3 | 1 |
| 3+4 | `.debug$S` + aux | 0 | 2 | 0 | 3 | 1 |
| 5+6 | `.XBLD$W` + aux | 0 | 3 | 0 | 3 | 1 |
| 7 | `__C2_11886` (strtab@4) | 0 | 3 | 0 | 2 EXTERNAL | 0 |
| 8+9 | `.XBLD$W` + aux | 0 | 4 | 0 | 3 | 1 |
| 10 | `__C1_11886` (strtab@15) | 0 | 4 | 0 | 2 | 0 |
| 11+12 | `.text` + aux | 0 | 5 | 0 | 3 | 1 |
| 13 | `?add3@@YAHHHH@Z` (strtab@26) | fn offset in .text (0) | 5 | **0x0020** | 2 | 0 |

- `@comp.id` Value `0x00AB2E6E` — CONST toolchain stamp, all fixtures.
- COMDAT grouping order matters: section symbol → its aux → the associated
  EXTERNAL watermark symbol in the same section. The `/include:` tokens in
  `.drectve` are what pull the watermarks in at link time.
- Function `Value` = byte offset within `.text` (verified: add3.obj's four
  functions at 0/0x10/0x20/0x30).
- Mangled names come **verbatim from the `.gl` bundle** (c1xx mangles; c2
  copies bytes). Do not re-mangle in Rust.

### Aux section-definition record (18 B)

`Length u32 | nReloc u16 | nLineno u16 | CheckSum u32 | Number u16 |
Selection u8 | Unused[3]`.

- Length = section SizeOfRawData (DERIVED). nReloc = real reloc count
  (0 in MVP). nLineno = 0 (`/Ox` without `/Zi`).
- **CheckSum: nonzero ONLY for COMDAT sections**; `.drectve`/`.debug$S`/
  `.text` store 0 regardless of content (verified across all fixtures).
- Selection: 0 non-COMDAT; **2 = SELECT_ANY** for the two `.XBLD$W`
  COMDATs; Number = 0 for SELECT_ANY.

### COMDAT CheckSum algorithm (identified, verified)

> **Reflected CRC-32, polynomial `0xEDB88320`, init `0x00000000`, no final
> inversion**, over the section raw data.

```rust
fn coff_checksum(data: &[u8]) -> u32 {
    let mut c: u32 = 0;
    for &b in data {
        c ^= b as u32;
        for _ in 0..8 { c = if c & 1 != 0 { (c >> 1) ^ 0xEDB8_8320 } else { c >> 1 }; }
    }
    c
}
```

Reproduces both stored values: XBLD-C2 → `0x92F87AA0`, XBLD-C1 →
`0x838510D9` (independently re-verified). Since the XBLD contents are fixed
watermarks, the MVP may hardcode the two values; implement the CRC when
emitting COMDATs with variable content.

## String table (@ end of symbol table → EOF)

`Size u32 (LE, includes itself)` then NUL-terminated long names
concatenated. Only names with len > 8 land here. **Order = first reference
while walking the symbol table top-to-bottom** (verified against add3.obj's
6-name table). mvp: size 42 → `__C2_11886` @4, `__C1_11886` @15,
`?add3@@YAHHHH@Z` @26.

## Multi-function TUs (generalization, verified)

Several straight-line functions in one TU share a single `.text` and one
symbol table:

- Each function starts at an **8-byte-aligned offset** within `.text` (the
  section is `ALIGN_8`): c2 zero-pads between functions to the next 8-byte
  boundary, but does **not** pad the tail of `.text`. Verified two ways:
  `add2`(8 B)+`add4`(16 B) need no padding (both already 8-aligned) → offsets
  0 / 8, size 0x18; `sub3`+`mul3`+`submix` (three 12-byte functions) get 4
  zero bytes each between → offsets 0x0 / 0x10 / 0x20, size 0x2C (last
  function unpadded).
- Each function gets one EXTERNAL FUNCTION symbol (type `0x20`, section
  `.text`) whose **`Value` = its byte offset within `.text`** (0, then
  cumulative), emitted in `.gl`/`.ex` order.
- **`NumberOfSymbols = 13 + N`** (13 fixed slots — `@comp.id`, 4 section
  symbols + their aux, 2 watermark externals — plus one per function). The
  single-function MVP is the `N = 1` case (14).
- Long names (`len > 8`) enter the string table in first-reference order,
  after the two watermark externals.

## Relocations + external symbols (W4a, calls)

The pure-MVP class has none; a function that **calls** another breaks the
"no relocations" assumption. W4a covers a single-function single-external
**tail call** (`void f(){g();}`), verified byte-exact:

- **`.text`** is a single relative branch `b` (op 18) with displacement
  `−(instruction offset)` (offset 0 → `48000000`), paired with a relocation.
- **Relocation records** sit between the last section's raw data and the
  symbol table (so `PointerToSymbolTable` gains `NumberOfRelocations × 10`).
  Each is 10 bytes, packed: `VirtualAddress u32 | SymbolTableIndex u32 |
  Type u16`. Type `0x0006` = `IMAGE_REL_PPC_REL24`. `VirtualAddress` is the
  branch's `.text` offset; `SymbolTableIndex` points at the callee's symbol.
- **Section header** for `.text`: `PointerToRelocations` = the reloc block
  offset, `NumberOfRelocations` = count. The `.text` **section-def aux** must
  carry the same `NumberOfRelocations` (both must agree).
- **Symbols**: the callee is an **undefined external** — `SectionNumber = 0`,
  `Type = 0x20` (FUNCTION), `StorageClass = 2` (EXTERNAL), `Value = 0`, name
  from `.gl`. Layout interleaves: each defined function symbol is immediately
  followed by its callee's undefined symbol, so `NumberOfSymbols` = 13 fixed
  + one per defined function + one per callee. (Multi-function/multi-callee
  ordering needs the `.ex` call-index decode — a later rung.)

## 6-section framed-call variant (W4b2, `.pdata` + labels)

A **framed non-leaf call** `int f(int a){ return g(a) + k; }` (the call result
is used, so `f` allocates a 96-byte frame) adds a sixth section and grows the
symbol table to 20. This section describes the **single-function** case, which
is the smallest instance; §7 has the general encoding and sectioning. Emitted by
`coff::emit_obj` (packed) / `coff::emit_comdat_obj` (`/Gy`) — the dedicated
`emit_framed_obj` is gone. Recovered by diffing reference objs for `g(a)+1`, `g(a)+2`
(byte-identical but the `addi` immediate), and `g(a)*5` (0x28 body).

**Sections (6):** the leaf five, then `.pdata` (`SizeOfRawData` 0x8,
Characteristics `0x40400040` = CNT_INIT_DATA | ALIGN_8 | MEM_READ). `.text`
(size 0x24) and `.pdata` each carry **one** relocation.

**`.pdata` raw (8 B, big-endian like `.text`):** `BeginAddress u32 = 0` (patched
by the reloc) + packed unwind word `0x40000000 | (text_len/4 << 8) | 3`
(prolog = 3 words). For the `+k` class → `00000000 40000903`. See
`CODEGEN_PPC_MVP.md` for the length-encoding derivation.

**File layout — raw+reloc are INTERLEAVED, not all-raw-then-all-reloc.** MSVC
writes each reloc'd section's raw immediately followed by its relocations, in
section order. So after the four leaf raw blocks: `.text` raw (0x24), `.text`
reloc (10 B), `.pdata` raw (8 B), `.pdata` reloc (10 B), then the symbol table.
(The 5-section path coincidentally matches "all raw then relocs" only because
`.text` is last there.)

**Relocations (2):**

| Section | VirtualAddress | SymbolTableIndex | Type |
|---|---|---|---|
| `.text` | 0x0C (the `bl`) | 15 (`?g`, the external callee) | `0x0006` REL24 |
| `.pdata` | 0x00 (BeginAddress) | 13 (`?f`, the defined function) | `0x0002` ADDR32 |

`0x0002` = `IMAGE_REL_PPC_ADDR32` (new vs the W4a REL24). Note the `bl` reloc
targets the **callee external**, not a `$M` label.

**Symbol table (20) — exact slot order (single-function TU):**

| Slot | Name | Value | Sec | Type | Class | nAux |
|---|---|---|---|---|---|---|
| 0 | `@comp.id` | 0x00AB2E6E | -1 | 0 | 3 | 0 |
| 1+2 | `.drectve` + aux | 0 | 1 | 0 | 3 | 1 |
| 3+4 | `.debug$S` + aux | 0 | 2 | 0 | 3 | 1 |
| 5+6 | `.XBLD$W` C2 + aux | 0 | 3 | 0 | 3 | 1 |
| 7 | `__C2_11886` | 0 | 3 | 0 | 2 | 0 |
| 8+9 | `.XBLD$W` C1 + aux | 0 | 4 | 0 | 3 | 1 |
| 10 | `__C1_11886` | 0 | 4 | 0 | 2 | 0 |
| 11+12 | `.text` + aux (**nReloc=1**) | 0 | 5 | 0 | 3 | 1 |
| 13 | `?f@@YAHH@Z` | 0 | 5 | 0x0020 | 2 | 0 |
| 14 | `$M2546` | 0x24 (.text end) | 5 | 0 | **6** | 0 |
| 15 | `?g@@YAHH@Z` (external) | 0 | **0** | 0x0020 | 2 | 0 |
| 16 | `$M2545` | 0x0C (the `bl`) | 5 | 0 | **6** | 0 |
| 17+18 | `.pdata` + aux (**nReloc=1, CheckSum=CRC**) | 0 | 6 | 0 | 3 | 1 |
| 19 | `$T2547` | 0 | 6 | 0 | **3** | 0 |

- The three label symbols are compiler-counter names. They are **not constant**
  and are no longer hardcoded — see §7 below and `OBJ_GY_SHAPES.md` §3.5.
  `$M2545`/`$M2546` are storage-class 6 (LABEL); `$T2547` is class 3 (STATIC).
- The `.pdata` aux section-def CheckSum is a **real reflected CRC-32** over the
  8 raw bytes (0xd3dfb2ce for `+k`) — a non-COMDAT section that still gets a
  checksum (contrast the leaf `.text`/`.drectve`/`.debug$S` aux, which store 0).
- String table order (first reference, top-down): `__C2_11886`, `__C1_11886`,
  `?f@@YAHH@Z`, `?g@@YAHH@Z`.

## 7. The `.pdata` unwind record — the encoding, from c2's own output

Xbox 360 PPC unwind data is **not** x64 unwind data: there is no `.xdata`, no
`UNWIND_INFO` header and no unwind-code array. The whole record is the 8 bytes
in `.pdata`, big-endian like `.text`:

```text
  u32 BeginAddress    always 0 in the obj; an ADDR32 relocation against the
                      function's own symbol supplies the address, so the raw
                      value is the addend (0 for every record c2 emitted here)
  u32 unwind          bits  7..0   PrologLen     prologue length, INSTRUCTIONS
                      bits 29..8   FuncLen       function length, INSTRUCTIONS
                      bit  30      ThirtyTwoBit  1 in every record observed
                      bit  31      ExceptionFlag 1 iff the function has EH data
```

Every field below was read out of a reference obj, not from documentation.
`$M(n)` and `$M(n+1)` are the same two lengths as symbols, which is the cheapest
available cross-check on any implementation of this: **`PrologLen = $M(n)/4` and
`FuncLen = $M(n+1)/4`, always.**

| source | `.text` | unwind | FuncLen | PrologLen | prologue |
|---|---|---|---|---|---|
| `return g(a)+1` | 0x24 | `40000903` | 9 | 3 | `mflr;stw;stwu` |
| `return g(a)+g(a+1)` | 0x48 | `40001205` | 18 | 5 | + `std r30`/`std r31` |
| 100 KB local array + 2 calls | 0x58 | `40001607` | 22 | 7 | + `lis;ori;bl _RtlCheckStack12` |
| 6 int args, 6 calls | 0x88 | `40002203` | 34 | 3 | `mflr;bl __savegprlr_25;stwu` |
| leaf with a 70 KB frame | 0x3c | `40000f06` | 15 | 6 | still framed → still a record |
| `double` temporaries, 1 call | 0x58 | `40001605` | 22 | 5 | + two `stfd` |
| a body holding a destructor, `/EHsc` | 0x4c | `c0001306` | 19 | 6 | **bit 31 set** |

Three consequences the port depends on:

1. **A leaf gets no record at all.** `int f(int a){ volatile char buf[400];
   buf[a&255]=(char)a; return buf[0]; }` addresses its array below `r1` in the
   red zone (`addi r10,r1,-400`) and c2 emits neither `.pdata` nor `$M` labels.
   Grow the array to 70,000 bytes so `r1` must move and both appear. The
   predicate is "does this function establish a frame", which is a fact the
   emitter has by construction.
2. **`PrologLen` is not a constant.** `build_pdata` hardcoded 3 for as long as
   the framed class was one shape; 3, 5, 6 and 7 all occur in ordinary code.
3. **EH is out of class, and visibly so.** Bit 31 is the tell, and EH also
   splits one function across **several** records: `try { return g(a); }
   catch (int e) { return e; }` produced two `.pdata` COMDATs, the catch
   funclet's first with a non-zero `BeginAddress` addend (0x48 against
   `?ehtry`), the body's second.

### 7.1 Sectioning: packed vs `/Gy`

| | packed (`/Ox`) | `/Gy` (so `/O1`, `/O2`) |
|---|---|---|
| `.pdata` sections | one for the TU | one per **framed** function |
| position | last, after `.text` | immediately after that function's `.text` COMDAT |
| characteristics | `0x40400040` | `0x40401040` (+ `LNK_COMDAT`) |
| aux `Selection` | 0 | **5** (`SELECT_ASSOCIATIVE`) |
| aux `Number` | 0 | the **section number of its `.text`** |
| aux `CheckSum` | real CRC over all records | real CRC over the one record |
| records | all framed functions, `.text` order | one |
| `$T` value | the record's offset (0, 8, …) | 0 |

The association is the mechanism that makes `/Gy` sound: the linker discards a
function's unwind record with the function. A leaf's `.text` COMDAT has no
`.pdata` beside it, so section numbers are **not** `4 + 2i` — they interleave
by framed-ness (`.text` 5, `.pdata` 6, `.text` 7 for a leaf, `.text` 8,
`.pdata` 9).

### 7.2 Symbol group per framed function

```text
  packed, FIRST framed function of the TU:
    [fn] [$M(n+1) @ function end] [callee, if new] [$M(n) @ prologue end]
    [.pdata section sym + aux] [$T(n+2) @ record offset]
  packed, every later framed function:
    [fn] [$M(n+1)] [callee, if new] [$M(n)] [$T(n+2)]
  /Gy, every framed function:
    [.text section sym + aux] [fn] [$M(n+1)] [callee, if new] [$M(n)]
    [.pdata section sym + aux] [$T(n+2) @ 0]
```

Note the order inside the group is not the obvious one: the **end** label comes
before the callee external and the **prologue** label after it.

## Emitter build order

1. Compute every `SizeOfRawData` (drectve const, debug$S by formula,
   XBLD const ×2, text = codegen size).
2. Chain `PtrToRawData` from 0xDC; `PointerToSymbolTable` = end of last raw.
3. Emit header, section headers, raw data, symbol table (order above,
   inline-vs-strtab by the len-8 rule, back-patch strtab offsets), string
   table.
4. Thread the real `-Fo` path into S_OBJNAME — do not normalize it away.
5. TimeDateStamp is arbitrary (normalized in compare); **every other byte
   must genuinely match**.
