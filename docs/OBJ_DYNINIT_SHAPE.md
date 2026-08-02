# The `??__E` dynamic-initializer obj — byte-level characterization

Board **#158**, the **obj-shape half** (ROADMAP §10.12 lists it as the larger
half, and not implied by the decode). This is a read-only measurement lane: no
code under `crates/` was touched. Every number below is transcribed from an obj
produced by the real `cl.exe` 16.00.11886.00 / `c2.dll` under wibo. Nothing is
computed or inferred unless it is labelled a rule, and every rule names the cells
it was fitted on and the cells it was tested against.

Companion docs: [`OBJ_FORMAT_MVP.md`](OBJ_FORMAT_MVP.md) (the four-section shell,
CONST/DERIVED classification) and [`OBJ_GY_SHAPES.md`](OBJ_GY_SHAPES.md)
(`_fltused`, pooled FP `.rdata`, framed calls under `/Gy`). This doc covers only
what those two do not: the sections a namespace-scope object with a non-trivial
constructor adds.

Controls for this lane are
[`rungs/_2026-08-02-w-objshape-prereg.md`](rungs/_2026-08-02-w-objshape-prereg.md)
(committed before the first capture) and
[`rungs/_2026-08-02-w-objshape-rules-frozen.md`](rungs/_2026-08-02-w-objshape-rules-frozen.md)
(13 rules + 9 falsifiable held-out predictions, committed before the held-out
cells were compiled).

---

## 0. The headline, before the tables

1. **The fixture is exact.** `fixtures/cpp/il_dyninit_static.cpp` at the
   workload's flags produces an obj whose `.text$yc` payload is **byte-identical**
   to both workload TUs':
   `3d 60 00 00 3d 40 00 00 38 8b 00 00 38 6a 00 00 38 a0 00 00 4b ff ff ec`.
   Same section set, same 24 symbol records, same 9 + 1 relocations. Only
   `.debug$S` (obj path), `.rdata` (the literal) and `.bss` (`sizeof`) differ.

2. **The flags matter, and the fixture gate's default flags are wrong for this
   class.** `/Ox /GS- /c` (what `c2rs compile` and the fixture gate use) does
   **not** imply `/GF`; the workload's `/O1` does. Without `/GF` the literal is a
   `$SG<n>` static in one **non-COMDAT** `.rdata` placed *before* `.text`, the
   `??_C@…` symbol does not exist at all, and multi-literal codegen changes. Any
   emit work graded only at `/Ox` would be graded against the wrong obj.

3. **The string COMDAT hash yielded.** `FIKCJHKP` is **JamCRC** (CRC-32,
   poly `0xEDB88320`, init `0xFFFFFFFF`, **no** final XOR) over the literal's
   bytes *including* the NUL, rendered as base-16 digits `A`..`P`,
   most-significant first, **leading zeros suppressed**. Verified on 6 literals
   including both real workload strings. Fully computable by the port.

4. **One thing is unsolved and is called out as such**: when a TU has ≥3
   namespace-scope objects, the *addresses* c2 assigns them inside `.bss` are a
   name-keyed permutation this lane did not crack (§7). **It does not affect the
   two #158 target TUs**, which have one object each.

---

## 1. Pre-registration, scored

Verbatim from the prereg. Bias registered in advance: *"I want this shape to come
out regular and derivable."* It largely did — which is why the wrong predictions
below are the load-bearing part of this section.

| # | prediction | verdict |
|---|---|---|
| P1 | 8 sections, order `.drectve`, `.debug$S`, 2×`.XBLD$W`, `.text`, `.rdata`, `.bss`, `.CRT$XCU` | **count right, order wrong.** 8 sections ✓. The code section is named **`.text$yc`**, not `.text`, and at `/O1` `.text$yc` precedes `.rdata` (at `/Ox` it is the other way round) |
| P2 | `.text`/`.rdata`/`.bss` COMDAT, `.CRT$XCU` not | **mostly wrong.** `.text$yc` ✓ COMDAT, `.CRT$XCU` ✓ not. `.bss` is **never** COMDAT. `.rdata` is COMDAT **only under `/GF`** |
| P3 | every COMDAT selection = 1 (NODUPLICATES) | **wrong.** The thunk's `.text$yc` and its `.rdata` use **2 (ANY)**. An *ordinary* function's `.text` does use 1 — the distinction is a discriminator, not a constant |
| P4 | 24 symbol records | **right**, and the composition (11 shell + 4×(section+aux) + 4 owned + 1 undefined) was right |
| P5 | 5 relocations on `.text`; alternative "7, PAIR after each REFHI" | **wrong, both.** **9** — a PAIR follows **every** REFHI *and* every REFLO |
| P6 | REFHI/REFLO **not** adjacent, offsets HI,HI,LO,LO at 0,4,8,12 | **right**, and refined in §3.2 |
| P7 | 1 `ADDR32` on `.CRT$XCU` at offset 0 → `??__EsL@@YAXXZ` | **right, exactly** |
| P8 | `.bss` `SizeOfRawData = 0`, size in `VirtualSize` | **wrong.** `SizeOfRawData` carries the size, `VirtualSize = 0`, `PointerToRawData = 0` |
| P9 | `sL$initializer$` is STATIC | **right.** Bonus not predicted: `??__EsL@@YAXXZ` is **also** STATIC |
| P10 | a 32-bit checksum, `A`..`P` nibbles, value `0x58A297AF` | **right on both the encoding and the value**; the algorithm is named in §5 |

**5 clean right, 4 wrong, 1 half.**

Grid predictions Q1–Q6: **Q1 right** (the four-section shell is invariant across
every cell). **Q2 half** — `.bss` size tracks `sizeof` ✓ but the *alignment* moves
too, which I predicted would not. **Q3 half** — arity moves `.text` size ✓, but
the relocation count is unchanged for scalar arguments and does change for
address-valued ones. **Q4 right in direction, understated** — a destructor adds
**2** sections and **10** symbol records, not "+1 and +3". **Q5 wrong** — two
objects yield **one** `.CRT$XCU` with two entries, not two sections. **Q6 wrong**
— see §4.1; section order is *not* first-use order.

### Held-out predictions (committed as a git object before those cells existed)

**6 of 9 right; 2 of the 3 refutations corrected a rule.**

| # | verdict |
|---|---|
| H1 ordinary function then object | **right** |
| H1b object then ordinary function | **WRONG** — the obj is byte-for-byte the same as H1. Section order does **not** track source order across the ordinary/thunk boundary (§4.1) |
| H2 non-zero addend | **WRONG** — the addend is not in the relocation at all; c2 emits an **extra `addi`** (§3.3). The registered alternative was the right one |
| H3 three objects sharing one literal | **right** — 3 `.text$yc`, 1 `.rdata`, `.bss` 3, `.CRT$XCU` 0xc/3 relocs |
| H4 `double` argument | **right** — `__real@3ff8000000000000`, 8 B, `0x40401040`, `_fltused` present |
| H5 64-byte object + destructor | **right** — 10 sections, `.bss` 0x40 ALIGN_8, `.pdata` SEL=5 `Number=5`, `.text$yd` |
| H6 `"xyzzy"` | **right, exact** — `??_C@_05POJHDMIP@xyzzy?$AA@`, 6 B, aux CheckSum `0xb0aa62d3` |
| H7 101-byte literal | **WRONG by one character** — predicted `??_C@_0GF@ALHLJLME@`, actual `??_C@_0GF@LHLJLME@`. The leading `A` (nibble 0) is suppressed. **This is the cell that turned a fixed-width guess into the correct rule** (§5) |
| H8 six-argument constructor | **right** — `.text$yc` size `0x28`, still 9 relocations |
| H9 every COMDAT CheckSum = CRC | **WRONG in scope** — string `.rdata`, `.pdata` and `.XBLD$W` carry it; **FP-constant `.rdata` carries 0** (§2.3) |

---

## 2. The section set

### 2.1 The reference cell

`fixtures/cpp/il_dyninit_static.cpp` at `/O1 /Oi /EHsc /GS- /c`, 1,316 B.
`Machine = 0x01F2`, `NumberOfSections = 8`, `NumberOfSymbols = 24`,
`Characteristics = 0x0180`.

| # | name | SizeOfRawData | Ptr | nrel | Characteristics | decode |
|---|---|---:|---:|---:|---|---|
| 1 | `.drectve` | 0x84 | 0x154 | 0 | `0x00100A00` | LNK_INFO \| LNK_REMOVE \| ALIGN_1 |
| 2 | `.debug$S` | 0x94 | 0x1d8 | 0 | `0x42100040` | CNT_INIT_DATA \| ALIGN_1 \| DISCARDABLE \| READ |
| 3 | `.XBLD$W` (C2) | 0x10 | 0x26c | 0 | `0xC0401040` | CNT_INIT_DATA \| **COMDAT** \| ALIGN_8 \| READ \| WRITE |
| 4 | `.XBLD$W` (C1) | 0x10 | 0x27c | 0 | `0xC2301040` | CNT_INIT_DATA \| **COMDAT** \| ALIGN_4 \| DISCARDABLE \| READ \| WRITE |
| 5 | **`.text$yc`** | 0x18 | 0x28c | **9** | `0x60401020` | CNT_CODE \| **COMDAT** \| ALIGN_8 \| EXECUTE \| READ |
| 6 | **`.rdata`** | 0x4 | 0x2fe | 0 | `0x40301040` | CNT_INIT_DATA \| **COMDAT** \| ALIGN_4 \| READ |
| 7 | **`.bss`** | 0x1 | **0** | 0 | `0xC0100080` | CNT_UNINIT_DATA \| ALIGN_1 \| READ \| WRITE |
| 8 | **`.CRT$XCU`** | 0x4 | 0x302 | **1** | `0xC0300040` | CNT_INIT_DATA \| ALIGN_4 \| READ \| WRITE |

Rows 1–4 are the shell of `OBJ_FORMAT_MVP.md`, unchanged. `VirtualAddress`,
`PointerToLinenumbers` and `NumberOfLinenumbers` are 0 in every section of every
cell measured. `VirtualSize` is 0 in every section **including `.bss`** — the
uninitialized size lives in `SizeOfRawData` with `PointerToRawData = 0`.
Raw data is packed contiguously with no inter-section padding, and `.bss`
contributes none.

### 2.2 The full section vocabulary

| section | when | Characteristics | COMDAT selection |
|---|---|---|---|
| `.text$yc` | one per `??__E<name>@@YAXXZ` thunk | `0x60401020` | **2 (ANY)** |
| `.text$yd` | one per `??__F<name>@@YAXXZ` atexit thunk | `0x60401020` | **2 (ANY)** |
| `.text` | one per **ordinary** function | `0x60401020` | **1 (NODUPLICATES)** |
| `.rdata` | one per distinct string literal or FP constant, under `/GF` | `0x40<a>01040` | **2 (ANY)** |
| `.rdata` | *all* literals pooled, **without** `/GF` | `0x40<a>00040` | none — not a COMDAT |
| `.pdata` | one per **framed** function | `0x40401040` | **5 (ASSOCIATIVE)**, `Number` = the 1-based section index of its `.text*` |
| `.bss` | **exactly one**, all objects | `0xC0<a>00080` | none |
| `.CRT$XCU` | **exactly one**, all initializer pointers | `0xC0300040` | none |

`<a>` is the alignment nibble, §4.2. `.CRT$XCU` was ALIGN_4 in every cell.
No `.CRT$XTX`/`.CRT$XPU` section is ever emitted: destructor registration goes
through a runtime `atexit` **call**, not a section (§4.4).

### 2.3 COMDAT aux `CheckSum`

**CRC-32, polynomial `0xEDB88320`, init `0`, no final XOR, over the section's raw
data.** Known-answer check on two constants that predate this lane:
`.XBLD$W`(C2) `43 32 00 00 00 00 00 00 00 10 00 00 2e 6e 44 00` → `0x92F87AA0` ✓,
`.XBLD$W`(C1) → `0x838510D9` ✓. Then `"abc\0"` → `0x8619B74C` ✓,
`"defg\0"` → `0x06AC9C4E` ✓, `"xyzzy\0"` → `0xB0AA62D3` ✓ (predicted before
capture), the 101-byte literal → `0x5DAE28DE` ✓ (predicted), the two real
workload strings → `0xA4AF7FD1` and `0x468413E7` ✓ (predicted), `.pdata`
`00 00 00 00 40 00 10 03` → `0x48DF1BD6` ✓.

**Scope, corrected out-of-sample (H9 was wrong):** the field is `0` for
`.text$y?`, for `.text`, for every non-COMDAT section, **and for an FP-constant
`.rdata` COMDAT** (`__real@3ff8000000000000` → CheckSum `0`, not the CRC). The
sections that carry a real checksum are the two `.XBLD$W`, the **string**
`.rdata`, and `.pdata`. A plausible reading — *the checksum is written by
whichever component created the section, and c2-created sections get 0* — is a
**hypothesis, not a measurement**; the port only needs the rule as stated.

Note the deliberate contrast with §5: **the same polynomial appears twice with
different initial values.** Section checksum = init `0`; string-name hash =
init `0xFFFFFFFF`. Getting these two swapped is the obvious way to implement
this wrong.

---

## 3. Symbols and relocations

### 3.1 The symbol table, in order

The reference cell, all 24 records. `sc` = StorageClass, `naux` = aux count.

| idx | name | Value | Sec | Type | sc | naux |
|---:|---|---:|---:|---|---:|---:|
| 0 | `@comp.id` | `0xAB2E6E` | −1 | 0 | 3 STATIC | 0 |
| 1,2 | `.drectve` + aux | 0 | 1 | 0 | 3 | 1 |
| 3,4 | `.debug$S` + aux | 0 | 2 | 0 | 3 | 1 |
| 5,6 | `.XBLD$W` + aux | 0 | 3 | 0 | 3 | 1 |
| 7 | `__C2_11886` | 0 | 3 | 0 | 2 EXTERNAL | 0 |
| 8,9 | `.XBLD$W` + aux | 0 | 4 | 0 | 3 | 1 |
| 10 | `__C1_11886` | 0 | 4 | 0 | 2 EXTERNAL | 0 |
| 11,12 | `.text$yc` + aux | 0 | 5 | 0 | 3 | 1 |
| **13** | **`??__EsL@@YAXXZ`** | **0** | **5** | **0x0020** | **3 STATIC** | 0 |
| **14** | `??0L@@QAA@PBDH@Z` | 0 | **0** | 0x0020 | 2 EXTERNAL | 0 |
| 15,16 | `.rdata` + aux | 0 | 6 | 0 | 3 | 1 |
| **17** | `??_C@_03FIKCJHKP@abc?$AA@` | 0 | 6 | 0 | **2 EXTERNAL** | 0 |
| 18,19 | `.bss` + aux | 0 | 7 | 0 | 3 | 1 |
| **20** | `sL` | 0 | 7 | 0 | **3 STATIC** | 0 |
| 21,22 | `.CRT$XCU` + aux | 0 | 8 | 0 | 3 | 1 |
| **23** | **`sL$initializer$`** | **0** | **8** | **0** | **3 STATIC** | 0 |

Aux records for section symbols: `Length` = `SizeOfRawData`,
`NumberOfRelocations` = the section's, `NumberOfLinenumbers` = 0, `CheckSum` per
§2.3, `Number` = 0 except `.pdata`, `Selection` per §2.2, three trailing zero
bytes.

Answering the brief's question directly: **`sL$initializer$` is the last symbol
in the table, storage class 3 (STATIC), `Type = 0`, `Value = 0`, in the
`.CRT$XCU` section, and it is not referenced by any relocation** — it exists so
the linker has a name for the 4-byte slot. It carries **no** `??_`-style
mangling; the name is literally `<identifier>$initializer$` built from the
object's *source* identifier, not its decorated name (`?gL@@3UL@@A` still yields
`gL$initializer$`).

Three storage-class facts that are easy to get backwards:

* `??__E<name>@@YAXXZ` and `??__F<name>@@YAXXZ` are **STATIC** (3), with
  `Type = 0x0020` (function) and `Value = 0` — even when the *object* has
  external linkage. An **ordinary** function's symbol is EXTERNAL (2). The
  workload's `ZlibLicense.cpp` confirms both halves at once:
  `?sLicense@@3VLicenses@@A` is EXTERNAL while `??__EsLicense@@YAXXZ` is STATIC.
* The string-literal COMDAT symbol is **EXTERNAL** (2) with `Type = 0` — it is
  the COMDAT's defining symbol and the linker must be able to fold it. Without
  `/GF` the corresponding `$SG<n>` symbol is **STATIC**.
* A `static` object's `.bss` symbol is **STATIC** and **undecorated** (`sL`,
  `sLicense`); a non-`static` one is **EXTERNAL** and **decorated**
  (`?gL@@3UL@@A`, `?sLicense@@3VLicenses@@A`).

**Ordering rule.** The symbol table follows section order exactly. For each
section: the section symbol + aux, then the symbols it defines, then any
**undefined external** first referenced by that section. In the reference cell
`??0L@@QAA@PBDH@Z` (SectionNumber 0) sits at index 14 — *inside* the `.text$yc`
group and *before* the `.rdata` group. With a destructor the two undefined
externals of `??__E` (`atexit`, `??0L`) both land in the `.text$yc` group and
`??1L@@QAA@XZ` lands in the `.text$yd` group.

Framed functions additionally emit `$M<n>` **LABEL** symbols (storage class 6) in
the `.text` section — one at the body offset and one at the section end — and a
`$T<n>` STATIC symbol in `.pdata`. **The `<n>` counter is a front-end name
counter this lane did not characterize and does not predict** (it shifts with
unrelated source edits: `$M2597/$M2598/$T2599` vs `$M2598/$M2599/$T2600` for two
sources differing only in a member array).

### 3.2 Relocations on `.text$yc`

Nine records, ascending `VirtualAddress`, 10 bytes each
(`VirtualAddress` u32, `SymbolTableIndex` u32, `Type` u16):

```
00 00 00 00  11 00 00 00  10 00     REFHI  0x0000 -> [17] ??_C@_03FIKCJHKP@abc?$AA@
00 00 00 00  00 00 00 00  12 00     PAIR   0x0000 -> 0
04 00 00 00  14 00 00 00  10 00     REFHI  0x0004 -> [20] sL
04 00 00 00  00 00 00 00  12 00     PAIR   0x0004 -> 0
08 00 00 00  11 00 00 00  11 00     REFLO  0x0008 -> [17] ??_C@_03FIKCJHKP@abc?$AA@
08 00 00 00  00 00 00 00  12 00     PAIR   0x0008 -> 0
0c 00 00 00  14 00 00 00  11 00     REFLO  0x000c -> [20] sL
0c 00 00 00  00 00 00 00  12 00     PAIR   0x000c -> 0
14 00 00 00  0e 00 00 00  06 00     REL24  0x0014 -> [14] ??0L@@QAA@PBDH@Z
```

* Types are `IMAGE_REL_PPC_REFHI 0x0010`, `REFLO 0x0011`, `PAIR 0x0012`,
  `REL24 0x0006`.
* **Every** REFHI and **every** REFLO is immediately followed by a `PAIR` whose
  `VirtualAddress` equals the primary's and whose `SymbolTableIndex` is **0**.
  Observed with `SymbolTableIndex = 0` in 100 % of PAIR records across every cell
  measured, including the non-zero-addend cell (§3.3), so the field appears to be
  a constant here rather than an addend carrier.
* `REL24` takes **no** PAIR.

**Adjacency — the brief's question, answered from bytes.** The prior lane's
finding is confirmed *for this shape* but is not a law:

| cell | address operands | REFHI offsets | REFLO offsets | adjacent? |
|---|---:|---|---|---|
| reference (`str`,`int`) | 2 | 0, 4 | 8, 12 | **no** — `lis` block, then `addi` block |
| `L(void)` | 1 | 0 | 4 | **yes** |
| `L(int)` | 1 | 0 | **8** | **no** — a `li r4,7` is scheduled between them |
| `L(const char*, const char*)` | 3 | 0, 4, 8 | 12, 16, 20 | no |
| `L(float)` | 2 | 0, 4 | 8, 12 | no, **and the symbol order differs between the blocks** |

The last row is the one to design against: HI order is `__real@3fc00000`, `sL`;
LO order is `sL`, `__real@3fc00000`, because the LO for the FP constant rides on
an `lfs` displacement field rather than an `addi`. **Relocation records are
ordered by offset; the symbol sequence within the HI block and within the LO
block are independent.**

### 3.3 The `.text$yc` payload and the branch encoding

```
3d 60 00 00   lis   r11, 0      <- REFHI(string)
3d 40 00 00   lis   r10, 0      <- REFHI(sL)
38 8b 00 00   addi  r4, r11, 0  <- REFLO(string)
38 6a 00 00   addi  r3, r10, 0  <- REFLO(sL)
38 a0 00 00   li    r5, 0
4b ff ff ec   b     -0x14       <- REL24(??0L@@QAA@PBDH@Z)
```

**The stored branch word is `0x48000000 | ((-k) & 0x03FFFFFC) | LK`, where `k` is
the section offset of the branch** — i.e. the encoded target is the section
start, and the addend is 0 with the PC-relative bias pre-applied. Verified on
ordinary functions too, which rules out this being a dynamic-initializer quirk:
a leaf tail call at `k = 0` stores `48 00 00 00`, and a framed `bl` at `k = 0xc`
stores `4b ff ff f5` (LK set).

Scalar arguments are materialized in **descending register order**: a
6-argument constructor emits `li r9,5; li r8,4; li r7,3; li r6,2; li r5,1` after
the two address pairs.

**A non-zero addend does not enter the relocation** (H2, refuted). For
`static char buf[64]; static L sL(buf + 32, 0);` c2 emits an extra instruction:

```
3d 60 00 00   lis   r11, 0       <- REFHI(buf)
3d 40 00 00   lis   r10, 0       <- REFHI(sL)
39 6b 00 00   addi  r11, r11, 0  <- REFLO(buf)        r11 = &buf
38 6a 00 00   addi  r3,  r10, 0  <- REFLO(sL)
38 a0 00 00   li    r5, 0
38 8b 00 20   addi  r4,  r11, 32   (no relocation)   r4 = &buf + 32
4b ff ff e8   b     -0x18
```

Still 9 relocations. The REFLO now targets a scratch register and a separate
unrelocated `addi` applies the displacement.

### 3.4 `.CRT$XCU`

Raw data is `00 00 00 00` per entry — **all zero**, the address comes entirely
from the relocation. One `IMAGE_REL_PPC_ADDR32` (`0x0002`) at offset `4i`
targeting the *i*-th `??__E…@@YAXXZ`, in **source order**, with no PAIR.
`.pdata`, when present, likewise carries one `ADDR32` at offset 0 pointing at its
function, over raw data `00 00 00 00 40 00 10 03`.

---

## 4. What varies, and what does not

Grid: object size (`char[1..256]`, `short`, `int`, `double` members), constructor
arity 0–6, argument type (`const char*` / `int` / `float` / `double`), one vs two
distinct literals, a shared literal across 2–6 objects, a destructor, external vs
internal linkage, ordinary functions mixed in, and the flag axis `/Od /O1 /O2
/Ox /Ox /GF /Ox /Gy`. ~60 objs.

### 4.1 Invariant across every cell

* The four-section shell. Checked mechanically over **61 objs** (every cell in
  the grid plus both real workload TUs): `.drectve` raw bytes, both `.XBLD$W`
  raw bytes, the first four Characteristics words, and symbol records 0–10
  (name/Value/SectionNumber/StorageClass/aux-count) are **identical in all 61**,
  and identical across `/Ox` and `/O1` as well. `.debug$S` is the one shell
  section that moves, and only with the embedded output-obj path length
  (0x94 in the probes, 0xac in the workload TUs) — which is also what shows the
  comparison is capable of reporting a difference.
* `@comp.id = 0x00AB2E6E`, `Machine = 0x01F2`, `Characteristics = 0x0180`.
* `.text$y?` characteristics `0x60401020`; `.CRT$XCU` characteristics
  `0xC0300040`; `.pdata` characteristics `0x40401040`.
* Selection: **2 (ANY)** for `.text$yc`/`.text$yd` and their `.rdata`;
  **1 (NODUPLICATES)** for an ordinary function's `.text`; **5 (ASSOCIATIVE)**
  for `.pdata`.
* `.bss` and `.CRT$XCU` are always exactly one each, always last, in that order.

**Section order is two-phase, not source order (Q6 and H1b, both refuted).**
All ordinary-function groups come first (source order among themselves), then all
dynamic-initializer groups (source order among themselves), then `.bss`, then
`.CRT$XCU`. A source with `f1`, then the object, then `f2` yields
`.text`(f1), `.text`(f2), `.text$yc`, `.rdata`, `.bss`, `.CRT$XCU` — the object's
position in the source is invisible in the section order. Moving the object above
or below an ordinary function produces **byte-identical** objs (1,429 B both
ways).

Within a group: the `.text$y?` COMDAT, then each `.rdata` COMDAT it is the
**first** to reference (in first-reference order, so a literal shared by three
objects appears once, in the first object's group), then its `.pdata` if framed.

### 4.2 Alignment — one rule for `.bss` and `.rdata`

For a blob of `n` bytes with natural alignment `t`:

> **`align = max(t, 1 if n < 2 else 4 if n < 64 else 8)`**

Measured thresholds, both sides:

| n | 1 | 2 | 3..63 | 64 | 65..256 |
|---|---|---|---|---|---|
| `.bss` (`char[n]` member) | ALIGN_1 | ALIGN_4 | ALIGN_4 | **ALIGN_8** | ALIGN_8 |
| `.rdata` (literal of n bytes incl. NUL) | ALIGN_1 | ALIGN_4 | ALIGN_4 | **ALIGN_8** | ALIGN_8 |

`t` shows up independently: a `double` member gives ALIGN_8 at `n = 8` where a
`char[8]` gives ALIGN_4; an FP `double` constant `.rdata` is 8 B ALIGN_8 while an
FP `float` constant is 4 B ALIGN_4. The `.bss` section's nibble is the max over
the objects it holds (`buf[64]` + a 1-byte object → size 0x41, ALIGN_8).

### 4.3 What moves with what

| axis | moves | does **not** move |
|---|---|---|
| object size / type | `.bss` `SizeOfRawData`, `.bss` align nibble | everything else — the `.text$yc` bytes are unchanged |
| constructor arity (scalars) | `.text$yc` size, +4 B per argument; the `REL24` offset | the **relocation count** (stays 9), the section set, the symbol count |
| argument type `const char*` | +1 `.rdata` COMDAT, +2 symbol records, +4 relocations | — |
| argument type `int` | nothing structural — one `li` | — |
| argument type `float`/`double` | +1 `.rdata` COMDAT (`__real@…`), +2 symbols, **+1 undefined `_fltused`**, and the REFLO rides an `lfs`/`lfd` displacement | — |
| distinct literals | one `.rdata` COMDAT each, in first-reference order | `.bss`, `.CRT$XCU` |
| **shared** literal across N objects | nothing — **one** `.rdata`, in the first referencing group | — |
| linkage (`static` vs not) | the `.bss` symbol's storage class (STATIC↔EXTERNAL) and decoration | the thunk symbol stays **STATIC**; `<name>$initializer$` stays STATIC and undecorated |
| destructor | **+2 sections** (`.pdata`, `.text$yd`), **+10 symbol records**, `??__E` becomes **framed** (0x40 B, 14 relocations) and gains a `bl atexit` | the `.CRT$XCU` size stays 4 — **no `.CRT$XTX`** |
| N objects | N `.text$yc`, one `.bss` of the summed size, one `.CRT$XCU` of `4N` with N relocations | the section *count* rule; everything stays single-`.bss`, single-`.CRT$XCU` |
| ordinary functions in the TU | `.text` groups **before** every `.text$yc` | the thunk groups' relative order |
| `/GF` (implied by `/O1`, `/O2`; **not** by `/Ox`) | literal becomes a `??_C@…` COMDAT `.rdata` **after** `.text`; without it, a single **non-COMDAT** `$SG<n>` `.rdata` **before** `.text`, and the multi-literal codegen changes (the second literal is addressed as a displacement off the first, 5 relocations instead of 9) | the thunk, `.bss`, `.CRT$XCU` |

### 4.4 The destructor shape, in full

`struct L { L(const char*, int); ~L(); }; static L sL("abc", 0);` → 10 sections,
34 symbol records.

```
.text$yc  0x40  14 rel   ??__EsL@@YAXXZ, framed
.rdata    0x04   0 rel   ??_C@_03FIKCJHKP@abc?$AA@
.pdata    0x08   1 rel   Selection 5 ASSOCIATIVE, Number = 5 (-> .text$yc)
.text$yd  0x0c   5 rel   ??__FsL@@YAXXZ
.bss      0x01
.CRT$XCU  0x04   1 rel   -> ??__EsL@@YAXXZ
```

```
7d 88 02 a6   mflr  r12
91 81 ff f8   stw   r12, -8(r1)
94 21 ff a0   stwu  r1, -0x60(r1)
3d 60 00 00 / 3d 40 00 00 / 38 8b 00 00 / 38 6a 00 00 / 38 a0 00 00   (as §3.3)
4b ff ff e1   bl    ??0L@@QAA@PBDH@Z
3d 60 00 00   lis   r11, 0        <- REFHI(??__FsL@@YAXXZ)
38 6b 00 00   addi  r3, r11, 0    <- REFLO(??__FsL@@YAXXZ)
4b ff ff d5   bl    atexit
38 21 00 60 / 81 81 ff f8 / 7d 88 03 a6 / 4e 80 00 20   epilogue
```

`??__F` is registered by **calling `atexit` with its address**, not by a
`.CRT$XT?` section — so a destructor costs an undefined `atexit` external, a
`.text$yd` COMDAT, a `.pdata` COMDAT, and four extra relocations on `.text$yc`.
`??__FsL@@YAXXZ` itself is a 3-instruction leaf: `lis`/`addi` for `sL`, tail `b`
to `??1L@@QAA@XZ`.

---

## 5. The string COMDAT name — solved

`??_C@_03FIKCJHKP@abc?$AA@` decomposes as `??_C@` `_0` `<L>` `@` `<H>` `@`
`<escaped text>` `@`:

* `_0` — a narrow (`char`) string literal.
* `<L>` — the byte length **including the NUL**, as an MSVC-mangled number:
  `1..10` → `'0'..'9'` (i.e. `n-1`), otherwise base-16 digits `A`..`P` followed
  by `@`. `"abc"` → 4 → `3`. `"Hello, world!"` → 14 → `O@`. 26 → `BK@`.
* `<H>` — **the hash**, see below.
* the escaped text, truncated at 32 characters, `?$AA` for the NUL, `?1` for `/`.

> **`<H>` = JamCRC of the literal's bytes including the NUL** — CRC-32,
> polynomial `0xEDB88320`, **init `0xFFFFFFFF`, no final XOR** (equivalently
> `~crc32(bytes)`) — rendered in base 16 with digits `A`=0 … `P`=15,
> **most-significant digit first, leading zeros suppressed**.

Byte evidence, all seven literals measured:

| literal | bytes incl. NUL | JamCRC | `<H>` from the rule | `<H>` in the obj |
|---|---:|---|---|---|
| `abc` | 4 | `0x58A297AF` | `FIKCJHKP` | `FIKCJHKP` ✓ |
| `defg` | 5 | `0x3F7194AC` | `DPHBJEKM` | `DPHBJEKM` ✓ |
| `` (empty) | 1 | `0x2DFD1072` | `CNPNBAHC` | `CNPNBAHC` ✓ (in `??_C@_00CNPNBAHC@?$AA@`) |
| `Hello, world!` | 14 | `0x647FB1F9` | `GEHPLBPJ` | `GEHPLBPJ` ✓ |
| `xyzzy` | 6 | `0xFE973C8F` | `POJHDMIP` | `POJHDMIP` ✓ — **predicted before capture** |
| `q`×100 | 101 | `0x0B7B9BC4` | `LHLJLME` | `LHLJLME` ✓ — **7 digits; the held-out cell that found the leading-zero rule** |
| `system/src/synth/tomcrypt` | 26 | `0xF4BC3E1C` | `PELMDOBM` | `PELMDOBM` ✓ (real workload TU) |
| `system/src/zlib` | 16 | `0x55C0A74D` | `FFMAKHEN` | `FFMAKHEN` ✓ (real workload TU) |

The 101-byte cell is why the fixed-8-nibble form registered in the prereg is
**wrong**: `0x0B7B9BC4` would be `ALHLJLME` at fixed width, and the obj carries
`LHLJLME`. The leading zero is suppressed, exactly as in `<L>`'s number
mangling. Without a held-out cell whose CRC happened to have a zero top nibble,
this lane would have shipped a rule that is right on ~15 of 16 literals and
silently wrong on the rest.

The port can compute this name from the literal bytes alone. **The decline-floor
condition "the string COMDAT name cannot be computed" is not met.**

---

## 6. What the `/FAsc` listing gets wrong about this obj

Recorded because the project has already been bitten by treating the listing as
byte-faithful, and because this fixture is one of the cases that discriminates —
it contains a relocated branch, which is what makes the check meaningful.

The listing for `il_dyninit_static.cpp` (`c2rs listing`, same `/O1` flags that
produced the obj):

```
  00014	48000000	 b            ??0L@@QAA@PBDH@Z
```

The obj carries **`4b ff ff ec`** at `.text$yc + 0x14`. The listing prints the
canonical unrelocated word and the target's *name*; the obj carries a
self-relative displacement back to the section start. They are not the same
bytes and no transformation of the listing text recovers the obj word without
knowing the offset.

The listing's **section order also disagrees with the obj's**: it prints the
`.rdata` string COMDAT *before* the `??__EsL` `PROC`, while the `/O1` obj puts
`.text$yc` first (the listing order matches the `/Ox` obj instead). Section order
is one of the things this lane was asked to establish, and the listing is not
evidence for it.

Control on the probes themselves: `c2rs listing` reports
`1 PROC / 1 .text COMDAT / 3 PUBLIC` for the fixture, confirming the source made
the single-thunk structure assumed throughout — checked rather than presumed.

---

## 7. The one thing that did not yield, and the verdict for #158

### 7.1 Unsolved: multi-object `.bss` address assignment

With N ≥ 3 namespace-scope objects, the offsets c2 assigns inside `.bss` are
**not** source order and not reverse source order. Sources identical except for
the object count, all objects 1 byte:

| N | `.bss` layout, by ascending address |
|---:|---|
| 2 | `s1 s2` |
| 3 | `s3 s1 s2` |
| 4 | `s4 s3 s1 s2` |
| 5 | `s4 s3 s5 s1 s2` |
| 6 | `s6 s4 s3 s5 s1 s2` |

Each new object is spliced into a position that depends on its **name** — the
signature of a hash-table walk. The *symbol table* order is the clean part: the
`.bss` symbols are listed in strictly **descending address** order in every
same-kind cell (and the `.CRT$XCU` symbols in ascending order, always matching
source order). Mixing kinds breaks even that: `static char buf[64]` +
`static L sL` lists `buf`(0) then `sL`(64), ascending.

This lane **declines** the multi-object `.bss` permutation: it is a name-keyed
ordering that would need the front end's hash reproduced, and no amount of
further black-box probing of this shape resolves it into a stated rule. Nothing
above is fitted to it — it is called out so a later lane does not rediscover it
as a mismatch.

**It does not block #158.** Both target TUs, and the fixture, have exactly one
namespace-scope object, where the permutation is the identity.

### 7.2 The two workload TUs, measured

`src/system/synth/tomcrypt/TomCryptLicense.cpp` and
`src/system/zlib/ZlibLicense.cpp`, from their cached reference objs:

| | TomCryptLicense | ZlibLicense | the fixture |
|---|---|---|---|
| sections | 8, same names/order | 8, same | 8, same |
| symbol records | 24 | 24 | 24 |
| `.text$yc` size / relocs | 0x18 / 9 | 0x18 / 9 | 0x18 / 9 |
| `.text$yc` bytes | **identical** | **identical** | **identical** |
| `.CRT$XCU` | 4 B, 1 `ADDR32` | same | same |
| `.rdata` | 0x1a, `??_C@_0BK@PELMDOBM@…` | 0x10, `??_C@_0BA@FFMAKHEN@…` | 0x4 |
| `.bss` | 0xc, ALIGN_4, `sLicense` **STATIC** | 0xc, ALIGN_4, `?sLicense@@3VLicenses@@A` **EXTERNAL** | 0x1, ALIGN_1 |
| `.debug$S` | 0xac | 0xac | 0x94 |

The `.text$yc` payload
`3d 60 00 00 3d 40 00 00 38 8b 00 00 38 6a 00 00 38 a0 00 00 4b ff ff ec` is
byte-for-byte the same in all three. The only structural difference between the
two workload TUs is the `.bss` symbol's linkage — which §4.3 already covers and
the `extlink` probe reproduces exactly. This is the `?sLicense@@3VLicenses@@A`
"extra data run" ROADMAP §10.11 saw in `.gl`.

### 7.3 Verdict

**The obj shape is fully determined for the #158 target class** — a TU whose only
emitted function is one `??__E` thunk for one namespace-scope object. Every
field is either constant, or a function of quantities the port already has
(section sizes, the literal's bytes, the object's `sizeof`/alignment, the
constructor's mangled name). Specifically:

* the section set and order — determined (§2.2, §4.1);
* every Characteristics word — determined (§2.2, §4.2);
* all 24 symbol records, their order, storage classes and aux contents —
  determined (§3.1, §2.3);
* all 10 relocations — determined (§3.2, §3.4);
* the string COMDAT's name — **computable** (§5);
* `<name>$initializer$` — mechanical from the source identifier.

The **decline floor registered in the prereg is not tripped for this class**: the
order is a function of things the IL has, and the name is computable. Two
caveats that belong in any emit rung's brief:

1. **Grade at `/O1`, not `/Ox`.** `/Ox` does not imply `/GF`, and the `/Ox` obj
   is a different shape (§4.3). The fixture's own `NotImplemented` today is
   recorded at `/Ox`.
2. **Do not widen past one object per TU without solving §7.1**, and do not widen
   past `??__E`/`??__F` on the assumption that the rest of the `??_` family
   behaves the same — ROADMAP §10.12 already recorded `??_G` behaving the other
   way on the decode side.

---

## 8. Reproducing this

```sh
# the reference cell (workload flags)
printf '/O1 /Oi /EHsc /GS- /c\n' > /tmp/f.txt
c2rs compile il_dyninit_static.cpp --cwd <dir> --flags-file /tmp/f.txt \
     --keep-obj /tmp/dyninit.obj
python3 work/objshape/coffdump.py /tmp/dyninit.obj    # scratch reader, this lane

# c2's own narration, for contrast (see §6 before trusting it)
c2rs listing <abs cpp> --out /tmp/base.cod
```

`work/objshape/` holds the ~60 probe sources (`p_*.cpp`), the batch driver
(`mk.sh`), and the two scratch readers (`coffdump.py`, `summarize.py`). The objs
themselves are gitignored scratch, per the project rule; every byte quoted above
is transcribed here so the doc stands without them.
