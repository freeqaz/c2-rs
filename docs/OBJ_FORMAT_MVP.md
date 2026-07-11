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
