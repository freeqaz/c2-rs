# `P_COFF` — the obj writer: `coff.c` (model) and `coffemit.c` (every `fwrite`)

> **Reference page.** Provenance marks are mandatory and mean exactly this:
> **`[R]`** read from the disassembly and *not* obj-checked — a hypothesis;
> **`[O]`** confirmed against a real obj or a `/FAsc` listing, with the witness
> named; **`[I]`** an interpretive step on top of an `[R]` or `[O]`.
> `[R]` means *"the instructions were read correctly"*, **never** *"this is what
> c2 does"* — those come apart, and `C2_MAP_METHOD.md` §7 is the priced example.
> Navigation only: nothing on this page may enter `crates/` without a
> [`DISCLOSURE.md`](../DISCLOSURE.md) row naming the address.
>
> Addresses are absolute VAs in `c2.dll`
> `sha256 c80981c015166effecc71ad8112d5577a065b2300891dfdb02b9c13787a66258`.
> Index: [`ADDR.tsv`](ADDR.tsv) · front door: [`README.md`](README.md)

**Coverage: 21 of the 120 functions in the `coff.c`/`coffemit.c` band**
(`0x10b281af`–`0x10b2b0dd` inclusive; the denominator is Ghidra function
entries in that span). Not covered: the relocation writer, `.debug$S` record
construction beyond the gate at `0x10b28548`, and the string-table builder.

**That `coff.c` and `coffemit.c` are separate translation units is itself a
finding** — it predicted a model layer distinct from a writer layer, and the
split is clean: `coff.c` opens the file and owns the section model, `coffemit.c`
does every `fwrite`. Both halves were named in advance and landed
(`C2_MAP.md` §6 control P4, the most exact result in that set: two of the five
predicted routines *are* the two files' anchor addresses).

---

## 1. The write path, in order

| # | what | address |
|---|---|---|
| 1 | open the obj, choose `Machine`, create `.drectve` | `0x10b28586` |
| 2 | latch the callbacks (`Machine`, `FILE*`, alloc, error, **time**) | `0x10b2ae0e` |
| 3 | per-section finalize: kind → chars, alignment → nibble | `0x10b287b8` → `0x10b982d6` ([`P_SECTION.md`](P_SECTION.md)) |
| 4 | write the file header (`0x14`, or `0x38` BIGOBJ) | `0x10b2b0dd` |
| 5 | write each 40-byte section header | `0x10b2b02d` |
| 6 | write each 18-byte symbol, and its aux | `0x10b2a936`, `0x10b2948b` |
| 7 | encode names ≤ 8 inline, longer through the string table | `0x10b2ad50`, `0x10b2a265` |

---

## 2. Entries

`size` and the caller/callee counts are Ghidra's, from the flat export.
`cites` counts how many times the record already mentions the address — a high
count means a page here is *replacing* re-derivation, not adding to it.

| addr | size | callers | callees | TU | cites | what |
|---|---:|---:|---:|---|---:|---|
| `0x10b281af` | 34 | 0 | 1 | `coff.c` gap | 1 | **the `TimeDateStamp` callback** — the image's sole `_time64` caller, and it **returns 0 when `DAT_10c2ead8` is set** `[R]`. That global is `-Brepro`: c2 already implements the determinism the harness gets by zeroing file offsets 4..8 `[I]` |
| `0x10b281f7` | 32 | 1 | 0 | `coff.c` gap | 1 | the COMDAT `Selection` byte map — an **identity map over 1..8** `[R]`. See §4 |
| `0x10b28261` | 163 | 1 | 0 | `coff.c` gap | 1 | **alignment bytes → `IMAGE_SCN_ALIGN_*`**: `(log2(a)+1) << 20`, ladder `1 → 0x100000` … `0x1000 → 0xD00000` `[R]`. This is U1's converter — see [`P_SECTION.md`](P_SECTION.md) §2 |
| `0x10b28304` | 172 | 1 | 0 | `coff.c` gap | 1 | the inverse of `0x10b28261` (nibble → byte count) `[R]` |
| `0x10b28548` | 62 | 1 | 1 | `coff.c` gap | 1 | the `.debug$S` record gate — record kinds `0xb`/`0xc`, gated on kind + `flags37 & 0x200` + **the emit bit** `[R]`. `coffemit.c` only *consumes* the emit bit; the decision is in `main.c` (§5) |
| `0x10b28586` | 548 | 1 | 10 | **`coff.c` anchor** | 5 | **the obj opener.** `_wfopen_s(objpath, "wb")`; sets `Machine` = `0x1F2` (`POWERPCFP`) or `0x0C13` under LTCG; writes `@comp.id`; creates `.drectve` `[R]` · `Machine = 0x01F2` **`[O]`** on two live objs (`C2_MAP.md` §7.3) |
| `0x10b287b8` | 183 | 4 | 10 | `coff.c` gap | 1 | **per-section finalize**: `kind = FUN_10be7727(sect)`, `align = FUN_10be77a3(kind, sect) → sect+0x43`, then `FUN_10b982d6`; ORs in `IMAGE_SCN_LNK_COMDAT` `[R]` |
| `0x10b2888e` | 48 | 5 | 4 | `coff.c` gap | 1 | emits a section then pads raw data to `sect[0x18]` `[R]` |
| `0x10b289fd` | 46 | 0 | 3 | `coff.c` gap | 1 | **ORs the alignment nibble into `Characteristics` — but only when `(sect[0x53] & 0xF00000) == 0`** `[R]`, i.e. only when the IL's own override did not already carry one. **`[O]`**: a `.gl` mutation setting the override to `0xC0500040` came back **verbatim** (`C2_MAP.md` §3F mutation matrix) |
| `0x10b291b1` | 45 | 3 | 1 | **`coffemit.c`** | 1 | section-number ceiling check at `0xFEFF` `[R]` |
| `0x10b291de` | 62 | 1 | 1 | **`coffemit.c`** | 1 | non-COMDAT output-section descriptor ctor: `+4` name, `+8` chars, `+0x10` index `[R]` |
| `0x10b2921c` | 76 | 1 | 1 | **`coffemit.c`** | 1 | COMDAT output-section descriptor ctor: `+0x16` `Selection`, `+0x18` associated number `[R]` |
| `0x10b29268` | 21 | 1 | 1 | **`coffemit.c`** | 1 | ORs alignment bits into descriptor `+8` (the `Characteristics` word) `[R]` |
| `0x10b2948b` | 104 | 2 | 0 | **`coffemit.c`** | 2 | **the 18-byte aux section-definition record.** `Number` and `Selection` are written **only when `Characteristics & 0x1000`** — i.e. only for COMDATs `[R]`. See §4 |
| `0x10b2a265` | 78 | 2 | 3 | **`coffemit.c`** | 1 | symbol `Name` encoder: ≤ 8 bytes inline zero-padded, else `{0, string-table offset}` `[R]` |
| `0x10b2a936` | 1050 | 0 | 9 | **`coffemit.c`** | 2 | **`fwrite` of the 18-byte `IMAGE_SYMBOL`.** The `.file` record memcpys 8 bytes, `SectionNumber = 0xFFFE`, `StorageClass = 0x67` `[R]`. Driven by `FUN_10b8303c(g_symList @ 0x10c2e234, …)` — **the iteration order of that list is the open R6 question** (§5) |
| `0x10b2ad50` | 159 | 1 | 5 | **`coffemit.c`** | 2 | the COFF long-name encoder: ≤ 8 inline zero-padded; else `'/'` + decimal string-table offset; else `"//"` + 6 base64 characters above `9999999` `[R]` |
| `0x10b2ae0e` | 321 | 1 | 4 | **`coffemit.c`** | 1 | **latches the eight-entry callback table** into `0x10c44bf0 … 0x10c44c0c` (`Machine`, `FILE*`, allocator, error, time) `[R]`. `DAT_10c44c08` is the `FILE*` every `fwrite` on this page uses; `DAT_10c44bf8` is the error raiser |
| `0x10b2b02d` | 176 | 0 | 3 | **`coffemit.c`** | 2 | **`fwrite(buf, 0x28, 1)` — the 40-byte `IMAGE_SECTION_HEADER`.** All ten fields mapped in §3 |
| `0x10b2b0dd` | 787 | 0 | 12 | **`coffemit.c` anchor** | 5 | **the `IMAGE_FILE_HEADER` writer**, and the 56-byte BIGOBJ variant. `fseek(n*0x28 + 0x14)`; BIGOBJ ClassID GUID `{D1BAA1C7-BAEE-4BA9-AF20-FAF66AA4DCB8}` at `.data 0x10b01be4`; `fwrite` of `0x14` or `0x38` `[R]`. `SizeOfOptionalHeader = 0`, `Characteristics = 0x0180` **`[O]`** on two live objs |
| `0x10b283b0` | 74 | 4 | 2 | `coff.c` gap | 0 | **the COMDAT spin-off**: `FUN_10be74cf(base->name+4, base->class+0x3b, newidx, base->kind+0x4d, selection, sym)`, copying `+0x43` and `+0x53` `[R]`. COMDAT-ness is **not** in the IL's tag-9 record; this is where it is minted `[O]` (`C2_MAP.md` §3F) |

---

## 3. `IMAGE_SECTION_HEADER`, field by field — `0x10b2b02d`

Read from the decompiled body; the stack slot order is the struct order.
**`[R]` for the source field, `[O]` for the emitted value where a real obj
pins it.**

| offset | field | source | provenance |
|---:|---|---|---|
| `0x00` | `Name[8]` | `FUN_10b2ad50(sect->name)` | `[R]` |
| `0x08` | `VirtualSize` | `sect + 0x1c` | `[R]`; **`= 0` in every section including `.bss`** `[O]` (`OBJ_DYNINIT_SHAPE.md` §P8, every cell) |
| `0x0c` | `VirtualAddress` | `sect + 0x18` | `[R]`; `= 0` `[O]` |
| `0x10` | `SizeOfRawData` | `sect + 0x14` | `[R]`; carries the size **even for `.bss`** `[O]` |
| `0x14` | `PointerToRawData` | `sect + 0x20` | `[R]`; `0` for `.bss` `[O]` |
| `0x18` | `PointerToRelocations` | `sect + 0x24` | `[R]` |
| `0x1c` | `PointerToLinenumbers` | **hard `0`** | `[R]` |
| `0x20` | `NumberOfRelocations` | `min(sect[0x30], 0xFFFF)` | `[R]` |
| `0x22` | `NumberOfLinenumbers` | **hard `0`** | `[R]` |
| `0x24` | `Characteristics` | `sect->hdr[8]`, **`\|= 0x01000000`** (`LNK_NRELOC_OVFL`) when `nrel > 0xFFFE` | `[R]` |

A short write raises `0xa3f` through `DAT_10c44bf8` `[R]`.

---

## 4. `Selection` and `CheckSum` — the aux record

This is usability question **U2**; the one-lookup answer lives here.

### 4.1 `Selection` — written at all only for COMDATs

> **`0x10b2948b` writes `Number` and `Selection` only when
> `Characteristics & 0x1000` (`IMAGE_SCN_LNK_COMDAT`).** For a non-COMDAT
> section both bytes are zero because the record was zeroed, not because
> anything computed a zero. `[R]`

The value itself comes from the section descriptor's `+0x16`, set by the COMDAT
ctor `0x10b2921c` from `FUN_10be74cf`'s `selection` argument, minted at
`0x10b283b0` — **not** from the IL, whose tag-`0x09` section record has no
COMDAT field at all `[O]` (`C2_MAP.md` §3F: `.text$yc` has an identically
shaped record and still emits `0x60401020`).

`0x10b281f7` maps the byte and is an **identity map over `1..8`** `[R]`.

**Which values actually occur — and the scope trap.** Three documents state
different things and *all three are right about their own population*; the
reference carries them side by side rather than arbitrating:

| population | `Selection` values seen | witness |
|---|---|---|
| `.data` / `.bss`, 8 638 + 14 669 COMDATs | **only `0` and `2` (`ANY`)** | `OBJ_DATA_BSS_SHAPE.md` §3.1 `[O]` |
| `.text` under `/Gy` | `1` (`NODUPLICATES`) | `OBJ_GY_SHAPES.md` §·25 `[O]` |
| `.pdata` | **`5` (`ASSOCIATIVE`)**, with `Number` = the associated section | `OBJ_GY_SHAPES.md` §·246, `CODEGEN_FRAMED_CALLS.md` `[O]` |

> **Do not read "only two Selection values occur anywhere" as a statement about
> the obj.** It is a statement about `.data` and `.bss`. `.pdata` alone
> falsifies it, and `FUN_10be76d4`'s base-section resolver special-cases
> `selection != 5` — the binary itself knows `5` is different `[R]`.

**Selection code 8** is mapped by `0x10b281f7` and special-cased at emit, and it
is **not** a documented `IMAGE_COMDAT_SELECT_*`. Do not assume it is a valid
on-the-wire value `[R]`, open (`C2_MAP.md` §7.2).

### 4.2 `CheckSum` — computed *outside* c2's own bytes

> **The algorithm is reflected CRC-32, polynomial `0xEDB88320`, init `0`, no
> final inversion, over the section's raw data.** `[O]`

Two independent derivations agree — a 9-cell probe fit and a blind
re-derivation over the 871-obj workload census — and the standard `zlib.crc32`
convention (init `0xFFFFFFFF`, final XOR) matches **0 of 9 139** sections
(`OBJ_DATA_BSS_SHAPE.md` §4.2). That is the strongest evidence class this
project produces.

**And it is not in `c2.dll`.** The `0xEDB88320` table is absent at every
4-aligned offset for both bit orders; the immediate `20 83 b8 ed` and its three
equivalents occur **nowhere** in the 1 347 072-byte image; the two `.XBLD$W`
checksums the port hardcodes (`0x92F87AA0`, `0x838510D9`) are absent as
immediates yet a fresh obj carries them `[R]`, search-method-controlled
(`C2_MAP.md` §6 P1). The computation reaches the writer through the eight-entry
callback table `0x10b2ae0e` latches into `DAT_10c44bf4 … 0x10c44c0c` `[R]`.

**Consequence, and it is why no `crc` label is published anywhere in this
directory:** pattern-matching "hash-shaped code near an emit site" would have
produced a confident wrong address.

**When it is nonzero — two documents disagree, and the later one is narrower
and better witnessed. Both are recorded; neither is rewritten.**

| dated claim | says | status |
|---|---|---|
| `OBJ_FORMAT_MVP.md` §·163 | *"nonzero **ONLY** for COMDAT sections"*; `.drectve`/`.debug$S`/`.text` store 0 | true on the MVP fixtures it was written against; **too strong as a general rule** |
| `OBJ_DYNINIT_SHAPE.md` §2.3 | `0` for every non-COMDAT section | **corrected below**; its own §H9 already found FP-constant `.rdata` COMDATs carry `0`, so the field is not simply "COMDAT ⇒ CRC" either |
| `OBJ_DATA_BSS_SHAPE.md` §4.2 **Rule D1** (later) | CRC over raw data, **written for non-COMDAT `.data` as well**, 9/9 | the operative rule for `.data` |

The consistent reading across all three `[I]`: **the field follows the raw
data, not the COMDAT bit** — `.bss` is `0` because it has *no* raw data, and
`.text$y?`/FP-constant `.rdata` are the cases still unexplained. Treat
"non-COMDAT ⇒ 0" as **refuted for `.data`** and unresolved elsewhere.

---

## 5. What is NOT known here

* **COFF symbol-table order (R6) — open.** The writer is `0x10b2a936`, driven
  by `FUN_10b8303c(g_symList @ 0x10c2e234, FUN_10b2a936)`; the *iteration order
  of that list* is the entire question. Three probes gave three answers:
  all-dyninit → IL order; no-dyninit → **source** order (matching neither IL nor
  address order); mixed → ascending address. The published "strictly descending
  address" holds only in the first case and is coincidence there
  (`C2_MAP.md` §7.2) `[O]`.
* **Section emission order — open, `medium`.** Kind-ordered, not name-ordered.
  Mutation shows kind `0x1D` defers to the end while all six other kinds tested
  landed at the *same* index, so the key is `0x1D`-vs-not, not the kind value
  `[O]`. Start at `0x10b287b8`.
* **`msobjXX.dll` does not write the obj** — its single import
  `FCreateFromBytesW` has exactly one call site (`0x10be83f2`), which opens an
  *existing* file `GENERIC_READ` and only reads through vtable slots `[R]`. The
  earlier worry that the COFF writer lived in msobj is **refuted**.
* Not covered by this page: the relocation writer, the string-table builder,
  `.debug$S` record construction.

---

## 6. Retrieval

The flat export is machine-local and regenerates in minutes
(`C2_MAP_METHOD.md` §3–4). `E=$C2RS_GHIDRA_EXPORT` (default
`~/ghidra-projects/export/c2`):

```sh
A=10b2b02d
grep -P "^$A\t"  "$E/calls.tsv"                 # callees
grep -P "\t$A\t" "$E/calls.tsv"                 # callers
awk "/^\/\/ ===== FUNC $A /{p=1} p; /^\/\/ ===== FUNC /&&p&&!/$A/{exit}" "$E/decomp_all.c"
grep -n "^$A" "$E/objdump_intel.asm"            # raw bytes at the right VA
grep -P "^$A\t" docs/whitebox/ref/ADDR.tsv      # everything already known
```
