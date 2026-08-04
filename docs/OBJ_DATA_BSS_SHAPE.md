# `.data` and `.bss` — byte-level characterization

Lane **`w-bss`**, the section-shape rung. ROADMAP §10.19 factored Phase 7 into four
predicates and found the tightest is **C**, the section shape; the greedy ladder's
single largest step is **`.bss`, worth +402 TUs**. This document is the byte-level
specification a later writer rung needs, in the shape of
[`OBJ_DYNINIT_SHAPE.md`](OBJ_DYNINIT_SHAPE.md).

This is a **read-only measurement lane**: nothing under `crates/` was touched. Every
number below is transcribed from an obj produced by the real `cl.exe`
16.00.11886.00 / `c2.dll` under wibo. Nothing is computed or inferred unless it is
labelled a **rule**, and every rule names the cells it was fitted on and the cells
that refute it.

Controls: [`rungs/_2026-08-02-w-bss-prereg.md`](rungs/_2026-08-02-w-bss-prereg.md),
committed before the first capture and scored verbatim in §1.

**Flags.** Everything below is at the **workload's own flags**,
`work/dc3-workload/flags.txt` = `/nologo /wd4355 /wd4164 /c /GR /O1 /Oi /EHsc`.
The `c2rs` CLI's default set does **not** carry `/GR`, and the prereg registered
`/O1 /Oi /EHsc /GS- /c` — a probe at either of those silently lacks RTTI. Where a
result depends on the flag set it says so.

---

## 0. The headline, before the tables

Everything here is measured twice where it can be: on a designed probe grid, and on
the **871 real workload objs** (§11) — 9,139 `.data` and 14,916 `.bss` sections.

1. **`.data` and `.bss` are emphatically not "one each, at the end".** In the
   workload a single obj reaches **101 `.data`** and **235 `.bss`** sections
   (`src/system/hamobj/HamDirector.cpp` holds both records), and **only 50 of the
   754 objs that have any `.data` have exactly one**. A writer built on a singular
   model fails on **704 of 754**. `.bss` is emitted **between the two `.XBLD$W`
   watermark COMDATs**; `.data` **after** the second one, in **754 of 754** objs.

2. **`.data` in this workload is 92.4 % RTTI by symbol count**, and the section name
   does not tell you what is in it. Of 9,287 symbols defined in a `.data`,
   **8,581 are `??_R0` type descriptors** and only 652 are ordinary decorated data.
   Classify by the **symbols defined in the section**, never by its name — this is
   the §10.20 defect (`.rdata$r` called "EH" for days when it is RTTI) arriving from
   the opposite direction. The RTTI record set is **split across two section names**:
   `??_R0` appears **only** in `.data`, `??_R1`–`??_R4` **only** in `.rdata$r`, so any
   RTTI rung must emit both (§4.3).

3. **The multi-object address permutation is solved, and the port never has to
   reproduce a hash.** It is not c2's — the order is already in the IL that c2 is
   handed:

   > **`.bss` ascending address = the IL `.gl` symbol-record order** for objects
   > without a dynamic initializer, and the **exact reverse** of it for objects with
   > one; the two groups never interleave, eager first (§5.2).
   > **`.data` ascending address = declaration (source) order** (§5.3).

   `OBJ_DYNINIT_SHAPE.md` §7.1 **declined** this permutation as "a name-keyed
   ordering that would need the front end's hash reproduced". It does not. The hash
   is the *front end's*, it runs before c2, and its result is a readable input.
   That section should be revised — see §9.

4. **One thing is only partly determined and is called out as such**: the address
   *allocator* itself. A bump allocator whose alignment padding becomes a reusable
   hole reproduces **14 of 18** random `.bss` cells and **12 of 14** random `.data`
   cells exactly; the residual is characterised, with two verbatim counterexamples,
   in §5.5. It is exact whenever the objects share one size/alignment, and it
   predicted **11 of 11** held-out layouts in the §4.2 grid.

5. **The terminal ceiling is 871, not 878.** The 7 workload TUs that never produce
   an obj fail in **`c1xx`, the front end** — C2084/C2512 duplicate bodies, C1189
   wrong-platform guards, C1083 missing `windows.h`. No obj exists for them at any
   settings, so they are unmeasurable at any effort. That is a property of the
   corpus, not an instrument fault, and the denominator for every section-shape
   metric should be **871**.

---

## 1. Pre-registration, scored

Verbatim from the prereg. The registered bias was *"I expect `.data`/`.bss` to be
boringly regular … which makes me likely to under-vary"*, with the mitigation that
`selectany`, `const`, `extern` and thread-local went into the grid **because** I
predicted they changed nothing. That mitigation is what earned P3 and P4 their
refutations, and they are the load-bearing rows here.

| # | prediction | verdict |
|---|---|---|
| P1 | `.bss` = `0xC0<a>00080`, `.data` = `0xC0<a>00040` | **right on both**, and `<a>` is the alignment nibble as registered (§3.1, §4.1). Understated: **both can additionally carry `LNK_COMDAT` (`0x1000`)**, which the prediction did not allow for |
| P2 | `.data` has a real `PointerToRawData`; `.bss` keeps `PointerToRawData = 0` with its size in `SizeOfRawData` | **right, exactly.** `VirtualSize = 0` in every section of every cell, `.bss` included |
| P3 | at most **one** `.data` and **one** `.bss` per obj, both **after every code group**, `.data` **before** `.bss`, `.bss` before `.CRT$XCU` | **wrong on every clause.** Many `.data` per obj; `.bss` sits **inside** the `.XBLD$W` pair; `.data` comes **after** `.bss`, not before; and both can precede *and* follow code groups (§2.2) |
| P4 | neither is ever COMDAT, **including** under `selectany`, which is instead diverted to a separate COMDAT section. *Named alternative: `selectany` makes `.bss`/`.data` itself a COMDAT with Selection 2* | **wrong; the named alternative is right.** `__declspec(selectany)` makes the section itself a COMDAT — `.bss` `0xC0301080` / `.data` `0xC0301040`, Selection **2 (ANY)**. RTTI `??_R0` does the same to `.data` without any `selectany` in the source (§3.3) |
| P5 | `Value` = byte offset, `SectionNumber` = 1-based index, `Type = 0`, no aux; `static` ⇒ STATIC + undecorated, non-`static` ⇒ EXTERNAL + decorated | **right, all five clauses** (§6.1) |
| P6 | a `const` object with a constant initializer lands in **`.rdata`, non-COMDAT**, and if unreferenced is dropped | **right**, with a refinement the prediction did not make: *internal*-linkage `const` is dropped when unreferenced, but `extern const` is **kept** even when unreferenced, in a non-COMDAT `.rdata` (§4.4) |
| P7 | `extern`-declared-not-defined ⇒ no section, one undefined EXTERNAL, `SectionNumber = 0`, `Value = 0` | **right, exactly** (§6.3) |
| P8 | `.data` uses the **same** permutation as `.bss` for the same name set — one ordering rule, two sections. *Named alternative: `.data` is source order and only `.bss` permutes* | **wrong; the named alternative is right.** Five cells, including declaration orders chosen so source order and sorted order differ: `.data` is **always** declaration order (§5.3) |
| P9 | inter-object padding is the minimum needed for each object's natural alignment, applied in layout order | **half right.** The alignment is not the *natural* alignment — it is promoted by size, `max(natural, 1 if n<2 else 4 if n<64 else 8)` — and the padding is **not dead**: it becomes a hole a later object can be placed into, so "applied in layout order" is wrong (§5.4) |
| P10 | zero-initialized ⇒ `.bss`, non-zero ⇒ `.data`, explicit `= 0` indistinguishable from no initializer | **right.** `int z1=0; int z2; int z3={0};` yields one `.bss` of 0xc and no `.data` at all |

**5 clean right (P2, P5, P7, P10, and P1 on its literal claim), 4 wrong (P3, P4, P8,
and P9's second clause), 1 right-with-refinement (P6).** Two of the four wrong ones
— P4 and P8 — were wrong in exactly the way their own registered alternatives
predicted, which is the only reason those alternatives were written down.

### The permutation hypotheses, scored

| # | hypothesis | verdict |
|---|---|---|
| H-A | subset stability — one total order over names, independent of the subset and of N | **CONFIRMED.** `a`..`z` gives `y d w k x m j n z r t i b v s u h c o e a l q p f g`; five subsets (halves, random 10, random 5, every other letter) each reproduce their restriction exactly, and the 26 singles keep that order inside a 699-name set |
| H-B | hash-bucket walk with push-front; reversing declaration order flips same-bucket pairs only | **CONFIRMED as a mechanism, and superseded.** Shuffling declarations permutes only adjacent runs, which yields a clean 1024-bucket partition (§7.3) — but the table being walked is the **front end's**, not c2's, and the port reads its output instead of recomputing it |
| H-C | the key is the source identifier, not the decorated name | **CONFIRMED.** `static char sN` (undecorated `sN`) and `char sN` (decorated `?sN@@3DA`) give **identical** orders at every N from 1 to 10 (§7.1, families B and C) |
| H-D | the key includes declaration order, type, size or alignment | **REFUTED for declaration order** (four random declaration permutations of the same 8 names give one order, and the `.gl` order is likewise invariant), and **refuted for type**: eight objects renamed nothing but retyped keep their order. **NOT refuted for size** — see §5.5, where mixed sizes do move addresses |
| H-E | the bucket count is fixed, no rehash | **CONFIRMED at 1024** — 11,000 names occupy exactly 1024 buckets, and an 8,000-name set occupies 1023 (§7.3) |

**The registered decline floor for question 4 is not reached, and the question is
moot.** The floor said: decline if no member of a named hash family reproduces the
partition. No member did — 7,452 configurations of 12 accumulator forms × 9 input
transforms × 7 output folds × 12 shifts scored **nothing above 0.08** against a
chance baseline of 0.03. But the hash does not need to be reproduced, because it is
not c2's (§7.4).

---

## 2. The section set

### 2.1 The shell, and where the data sections sit in it

`OBJ_FORMAT_MVP.md` describes a four-section shell: `.drectve`, `.debug$S`, and two
`.XBLD$W` COMDAT watermarks (`C2` then `C1`, distinguishable by their first two raw
bytes). The data sections are **interleaved into that shell**, not appended after
it. Measured on 17 TU shapes:

| TU | section order |
|---|---|
| (nothing but a declaration) | `.drectve .debug$S .XBLD$W(C2) .XBLD$W(C1)` |
| one function | `… .XBLD$W(C2) .XBLD$W(C1) .text` |
| **`char b1;`** | `… .XBLD$W(C2)` **`.bss`** `.XBLD$W(C1)` |
| **`char d1=1;`** | `… .XBLD$W(C2) .XBLD$W(C1)` **`.data`** |
| **both** | `… .XBLD$W(C2)` **`.bss`** `.XBLD$W(C1)` **`.data`** |
| string literal + `.bss` | `.drectve .debug$S` **`.rdata`** `.XBLD$W(C2)` **`.bss`** `.XBLD$W(C1)` **`.data`** |
| `__declspec(thread) int t1;` | `… .XBLD$W(C2)` **`.tls$`** `.XBLD$W(C1)` |
| `__declspec(thread) int t1=4;` | `… .XBLD$W(C2) .XBLD$W(C1)` **`.tls$`** |
| `.bss` + a function | `… .XBLD$W(C2)` **`.bss`** `.XBLD$W(C1) .text` |
| a function + `.bss` (order swapped in source) | *identical bytes* — `… .XBLD$W(C2)` **`.bss`** `.XBLD$W(C1) .text` |
| dyninit object | `… .XBLD$W(C2) .XBLD$W(C1) .text$yc` **`.bss`** `.CRT$XCU` |
| dyninit object + `char b1;` | `… .XBLD$W(C2)` **`.bss`** `.XBLD$W(C1) .text$yc .CRT$XCU` |
| dyninit object + `char d1=1;` | `… .XBLD$W(C2) .XBLD$W(C1)` **`.data`** `.text$yc` **`.bss`** `.CRT$XCU` |
| everything at once | `.drectve .debug$S .rdata .XBLD$W(C2)` **`.bss`** `.XBLD$W(C1)` **`.data`** `.rdata .text .text$yc .CRT$XCU` |

### 2.2 The placement rule

> **Rule S1.** There are three insertion points in the shell, and each data kind has
> exactly one:
> * **before `.XBLD$W(C2)`** — a `/GF` string-literal `.rdata` COMDAT that the *front
>   end* created (present only when a literal exists);
> * **between `.XBLD$W(C2)` and `.XBLD$W(C1)`** — the **uninitialized** section:
>   `.bss`, or `.tls$` if the objects are thread-local and uninitialized;
> * **after `.XBLD$W(C1)`** — the **initialized** section: `.data`, or `.tls$` for
>   initialized thread-locals; then every code group and every section a code group
>   drags with it, then `.CRT$XCU` last.

Fitted on the 17 rows of §2.1; **not refuted by any cell in this lane**. Two
consequences worth stating because they are counter-intuitive:

* **`.bss` precedes `.data`** in file order — the *uninitialized* section comes
  first, which is the opposite of the usual link-order intuition and the opposite of
  what the prereg guessed.
* The **dyninit `.bss` is not the same insertion point.** Compare rows 11 and 12: a
  TU whose only object is a dynamic-initializer object puts `.bss` *after*
  `.text$yc`, but adding one plain `char b1;` moves the single `.bss` back into the
  `.XBLD$W` pair and both objects share it. So the position is decided by the
  section's *earliest* contributor, not by any per-object rule. Row 13 shows all
  three at once: `.data` (after C1), `.text$yc`, `.bss` (after the code), `.CRT$XCU`.

`OBJ_DYNINIT_SHAPE.md` §4.1's "`.bss` and `.CRT$XCU` are always exactly one each,
**always last**, in that order" is **true only for the dyninit-only TU class it was
measured on**. It is false in general and false on this workload.

### 2.3 Multiplicity

> **Rule S2.** There is **at most one non-COMDAT `.bss`** and **at most one
> non-COMDAT `.data`** per obj, and they hold every ordinary object of their kind.
> **COMDAT `.data`/`.bss` sections are unbounded in number** — one per COMDAT
> object. Every `??_R0…` RTTI type descriptor is one, and so is every
> `__declspec(selectany)` object.

`d_rtti_dyncast` (a two-class hierarchy plus one `dynamic_cast`) emits 16 sections
including **two** `.data`, at indices 8 and 16, holding `??_R0?AUB@@@8` and
`??_R0?AUD@@@8`. This is the clause of P3 that is most likely to break a writer
built on the prereg's assumption, and the workload confirms it at scale:

| | `.data` | `.bss` |
|---|---:|---:|
| objs with 0 | 117 | 181 |
| objs with exactly 1 | **50** | 66 |
| objs with 2 | 44 | 21 |
| objs with 3 | 40 | 22 |
| most in one obj | **101** | **235** |
| sections in the workload | 9,139 | 14,916 |

Both records are `src/system/hamobj/HamDirector.cpp`.

**The between-the-watermarks slot is a rule, not a tendency.** Across all 871 objs
the only section that ever appears between `.XBLD$W(C2)` and `.XBLD$W(C1)` is a
`.bss` — **zero exceptions** — and when one appears there is exactly one of it, and
it is always the TU's ordinary non-COMDAT `.bss`. 139 objs use the slot; the other
732 leave it empty. Every COMDAT `.bss` is emitted elsewhere. Symmetrically,
`.data` appears **after** `.XBLD$W(C1)` in **754 of the 754** objs that have one.

---

## 3. `.bss` — the byte-level spec

### 3.1 Section header

| field | value |
|---|---|
| `Name` | `.bss`, NUL-padded to 8 bytes (never a `/n` string-table name) |
| `VirtualSize` | **0** — in every cell measured, including `.bss` |
| `VirtualAddress` | 0 |
| `SizeOfRawData` | the total allocated size (see §5); **this is where the size lives** |
| `PointerToRawData` | **0** |
| `PointerToRelocations` | 0 |
| `PointerToLinenumbers` | 0 |
| `NumberOfRelocations` | **0** — a `.bss` never carries a relocation |
| `NumberOfLinenumbers` | 0 |
| `Characteristics` | `0xC0<a>00080` ordinary, `0xC0<a>01080` COMDAT |

`0x00000080` = `CNT_UNINITIALIZED_DATA`, `0x40000000` = `MEM_READ`, `0x80000000` =
`MEM_WRITE`, `0x00001000` = `LNK_COMDAT`. `<a>` is the alignment nibble
(`1` = ALIGN_1, `3` = ALIGN_4, `4` = ALIGN_8, `5` = ALIGN_16, `6` = ALIGN_32).

`.bss` contributes **no bytes** to the file: raw data is packed contiguously across
the sections that have a `PointerToRawData`, and `.bss` is skipped.

**Invariant, zero counterexamples across all 14,916 workload `.bss` sections**:
`PointerToRawData == 0`, `VirtualSize == 0`, `NumberOfRelocations == 0`, and the
size lives in `SizeOfRawData`. Every section carries exactly **one** section symbol
(true of all 9,139 `.data` too). Sizes run 1 … 1,245,344 bytes, median 4.

**In the workload the COMDAT form is the common one, not the exception.** The
characteristics distribution over all 14,916 `.bss`:

| Characteristics | count | decode |
|---|---:|---|
| `0xC0301080` | **14,618** | COMDAT, ALIGN_4 |
| `0xC0300080` | 182 | ordinary, ALIGN_4 |
| `0xC0400080` | 50 | ordinary, ALIGN_8 |
| `0xC0401080` | 30 | COMDAT, ALIGN_8 |
| `0xC0101080` | 21 | COMDAT, ALIGN_1 |
| `0xC0100080` | 15 | ordinary, ALIGN_1 |

and over all 9,139 `.data`: `0xC0301040` 8,382 (COMDAT ALIGN_4), `0xC0400040` 454,
`0xC0401040` 253, `0xC0300040` 44, `0xC0101040` 3, `0xC0100040` 3. **Only two
Selection values occur anywhere**: 0 for non-COMDAT and **2 (ANY)** for every COMDAT
in both sections — 8,638 `.data` and 14,669 `.bss`. No NODUPLICATES, no ASSOCIATIVE,
no LARGEST. The probe grid reaches the COMDAT form only through
`__declspec(selectany)`; the workload reaches it overwhelmingly through templates
and RTTI, so a writer must treat COMDAT `.bss` as the default case rather than an
edge.

### 3.2 The section alignment nibble

> **Rule B1.** The `.bss` characteristics' alignment nibble is the **maximum over the
> objects the section holds** of `align(obj)` from §5.4 — i.e.
> `max_i max(natural_i, 1 if n_i < 2 else 4 if n_i < 64 else 8)`, and any
> `__declspec(align(k))` raises it to `k`.

Measured, one object per cell:

| object | size | nibble |
|---|---:|---|
| `char a1` | 1 | ALIGN_1 (`0xC0100080`) |
| `short a2` | 2 | ALIGN_4 (`0xC0300080`) |
| `int a4` | 4 | ALIGN_4 |
| `char a3[3]` | 3 | ALIGN_4 |
| `char a5[5]` | 5 | ALIGN_4 |
| `char a63[63]` | 63 | ALIGN_4 |
| `char a64[64]` | 64 | **ALIGN_8** (`0xC0400080`) |
| `char a65[65]` | 65 | ALIGN_8 |
| `double a8` | 8 | ALIGN_8 |
| `int bz[1024]` | 4096 | ALIGN_8 |
| `__declspec(align(32)) char` | 1 | ALIGN_32 (`0xC0600080`) |

The `n = 63 → 64` step and the `n = 1 → 2` step are the two thresholds; they are the
same thresholds `OBJ_DYNINIT_SHAPE.md` §4.2 measured for `.bss` and `.rdata`, now
confirmed to be a property of the **object**, not of the section, because §5.4 needs
the per-object value to place anything correctly.

### 3.3 When `.bss` is a COMDAT

Only under `__declspec(selectany)` on an **uninitialized** object:

```
.bss   size=0x4  ptr=0  ch=0xC0301080  CNT_UNINIT|LNK_COMDAT|READ|WRITE|ALIGN_4
       aux: Length=4 NumberOfRelocations=0 NumberOfLinenumbers=0
            CheckSum=0x00000000 Number=0 Selection=2 (ANY)
       sym ?sb@@3HA  Value=0 Type=0 SC=2 EXTERNAL
```

`Selection = 2 (ANY)`, `Number = 0`, and the aux `CheckSum` is **0** — a `.bss`
never has raw data to check-sum. `__declspec(selectany)` on an *initialized* object
produces the `.data` COMDAT of §4.3 instead.

### 3.4 Aux record for the `.bss` section symbol

`Length = SizeOfRawData`, `NumberOfRelocations = 0`, `NumberOfLinenumbers = 0`,
`CheckSum = 0`, `Number = 0`, `Selection = 0` (or 2 for the `selectany` COMDAT),
three trailing zero bytes.

---

## 4. `.data` — the byte-level spec

### 4.1 Section header

Identical to §3.1 except: `PointerToRawData` is **real**, `SizeOfRawData` is the raw
byte count, `NumberOfRelocations` can be non-zero, and the characteristics word is
`0xC0<a>00040` ordinary / `0xC0<a>01040` COMDAT (`0x00000040` =
`CNT_INITIALIZED_DATA`).

### 4.2 The aux `CheckSum` — and a correction to `OBJ_DYNINIT_SHAPE.md` §2.3

> **Rule D1.** The `.data` section symbol's aux `CheckSum` is **CRC-32, polynomial
> `0xEDB88320`, init `0`, no final XOR**, over the section's raw data — and it is
> written for **non-COMDAT** `.data` as well as COMDAT `.data`.

Verified on 9 sections:

| raw bytes | obj CheckSum | CRC-32(init 0) |
|---|---|---|
| `01` | `0x77073096` | `0x77073096` ✓ |
| `07` | `0x9E6495A3` | ✓ |
| `00 00 00 02` | `0xEE0E612C` | ✓ |
| `00 00 00 03` | `0x990951BA` | ✓ |
| `00 00 00 04` | `0x076DC419` | ✓ |
| `00 00 00 05` | `0x706AF48F` | ✓ (the `selectany` COMDAT) |
| `00 00 00 09` | `0x79DCB8A4` | ✓ |
| `00 00 00 00 00 00 00 00 2E 3F 41 55 42 40 40 00` (`??_R0?AUB@@@8`) | `0x37DE6EDE` | ✓ |
| `00 00 00 00 00 00 00 00 2E 48 00` (`??_R0H@8`) | `0x0A73AEE7` | ✓ |

`OBJ_DYNINIT_SHAPE.md` §2.3 states the field is *"`0` … for every non-COMDAT
section"*. That is **false for `.data`** — every non-COMDAT `.data` above carries a
real CRC. The rest of that section's scope claim stands (`.bss` is 0 because it has
no raw data; `.text$y?` is 0).

**The known-answer control that licenses this.** Two independent derivations agree.
This lane fitted the algorithm on 9 probe cells; a blind re-derivation over the
871-obj workload census, done without sight of that fit, produced the same
polynomial and the same init-0/no-final-XOR convention, while the standard
`zlib.crc32` convention (init `0xFFFFFFFF`, final XOR) matched **0 of 9,139**
sections. Both also agree with the project's prior independent characterization of
the same polynomial in `OBJ_DYNINIT_SHAPE.md` §2.3. Rule D1 rests on that
agreement, not on either fit alone.

### 4.2.1 The float exclusion — specification, then hypothesis

**52 of the 9,139 workload `.data` sections do not match Rule D1**, and every one
contains floating-point initializers. This lane's own §4.2 grid had already hit one:
`double d8 = 8.0;` gives raw `40 20 00 00 00 00 00 00` with `CheckSum = 0`, where
the CRC is `0xE620FB71`.

The two claims are kept apart on purpose, because one is measured and one is
inferred:

> **Specification (this is what a writer implements).** The `CheckSum` is
> CRC-32/`0xEDB88320`/init-0/no-final-XOR over a **subset** of the section's raw
> bytes. On 9,087 of 9,139 workload sections that subset is all of them; on 52 it
> omits the bytes of floating-point initializers.
>
> **Hypothesis (fenced).** The omission is c2's floating-point initializer path
> writing its bytes into the section without feeding the running CRC.

Found cases cannot separate byte- from word-granularity, or say whether padding is
omitted too. So the predicate was settled with a **designed grid whose predicted
CheckSums were committed as a git object before any cell was compiled** —
[`rungs/_2026-08-04-w-bss-fpcrc-prereg.md`](rungs/_2026-08-04-w-bss-fpcrc-prereg.md),
11 cells varying only the count, size and placement of float vs int initializers,
with four candidate variants scored against each other.

**Result: 11 of 11 cells hit the registered primary prediction (VAR-A), and 11 of
11 predicted layouts were also exact.**

| cell | source | raw | registered VAR-A | measured |
|---|---|---|---|---|
| f0 | `int a=1; int b=2;` | `00000001 00000002` | `0xD36E489C` | `0xD36E489C` ✓ |
| f1 | `float f=1.0f;` | `3F800000` | `0x00000000` | `0x00000000` ✓ |
| f2 | `double d=1.0;` | `3FF00000 00000000` | `0x00000000` | `0x00000000` ✓ |
| f3 | `int a=1; float f=1.0f;` | `00000001 3F800000` | `0x77073096` | `0x77073096` ✓ |
| f4 | `float f=1.0f; int a=1;` | `3F800000 00000001` | `0x77073096` | `0x77073096` ✓ |
| f5 | `int; float; int` | `00000001 3F800000 00000002` | `0xD36E489C` | `0xD36E489C` ✓ |
| f6 | `int; double; int` | `00000001 00000002 3FF00000 00000000` | `0xD36E489C` | `0xD36E489C` ✓ |
| f7 | `float f; float g;` | `3F800000 40000000` | `0x00000000` | `0x00000000` ✓ |
| **f8** | `char c=1; float f=1.0f;` | `01 000000 3F800000` | **`0xB8BC6765`** | `0xB8BC6765` ✓ |
| **f9** | `char c; char e; float f; char g;` | `01 02 03 00 3F800000` | **`0x9015E0C8`** | `0x9015E0C8` ✓ |
| f10 | `float p[2]={..}; int a=1;` | `3F800000 40000000 00000001` | `0x77073096` | `0x77073096` ✓ |

f8 and f9 are the discriminating cells: VAR-B (omit the padding as well as the FP
bytes) predicted `0x77073096` and `0xAAFD590F` and is **refuted**. So the omitted
set is exactly the FP objects' own byte ranges — **alignment padding stays in the
CRC**. Secondary predictions P-A (control), P-B (FP-only ⇒ 0), P-C (f3 = f4,
placement-independent), P-D (FP array behaves as FP scalar) and P-E (the layouts,
including f6's hole reuse) all hold.

**f6 is worth its own line**: the registered layout was `a@0 b@4 d@8` — `b` placed
*inside* the hole that `d`'s 8-alignment opens rather than after `d` — and that is
what the obj carries. §5.4's hole reuse is confirmed out of sample.

**Granularity — registered as still open, then settled by an exploratory cell.**
The prereg said in advance that a VAR-A hit would leave byte-vs-word granularity
undetermined, because every FP object in the grid is 4-aligned and 4k-sized, so
VAR-A and VAR-W make identical predictions. Three cells were then added
**after** the grid, and are labelled here as *not pre-registered*:

| cell (exploratory) | raw | CheckSum | the subset that reproduces it |
|---|---|---|---|
| `#pragma pack(1) struct P{char c; float f;}; P p={1,1.0f};` | `01 3F800000` | `0x77073096` | **bytes 1..5 dropped** — a non-word-aligned range |
| `struct Q{int i; float f;}; Q q={7,1.0f};` | `00000007 3F800000` | `0x9E6495A3` | the float **member's** 4 bytes dropped |
| `struct R{int i; int j;}; R r={7,9};` (control) | `00000007 00000009` | `0xCBFC64B4` | none — the full CRC |

The packed-struct cell puts a `float` at byte offset 1 and the omitted range starts
at 1, so the omission is **byte-granular**; VAR-W is refuted. The `struct Q` cell
shows the omission is **per initializer member, not per object** — an aggregate
with one int and one float member contributes its int member's bytes to the CRC and
not its float member's. Both findings raise the fenced hypothesis's standing
considerably, and it is still labelled a hypothesis: nothing here observes the CRC
call site, only its output.

### 4.3 When `.data` is a COMDAT

Two producers, both with `Selection = 2 (ANY)`, `Number = 0`, characteristics
`0xC0301040`:

* `__declspec(selectany)` on an initialized object;
* **every RTTI type descriptor `??_R0…`**, with no `selectany` in the source.

```
.data  size=0x10  ch=0xC0301040  Selection=2 (ANY)  CheckSum=0x37DE6EDE
       raw: 00 00 00 00 00 00 00 00 2E 3F 41 55 42 40 40 00
       sym ??_R0?AUB@@@8   Value=0  Type=0  SC=2 EXTERNAL
       rel off=0x0 type=0x0002 (ADDR32) -> ??_7type_info@@6B@   [undefined external]
```

The layout is: 4 bytes of vftable pointer (relocated to `??_7type_info@@6B@`),
4 bytes of zero (the runtime's spare field), then the NUL-terminated decorated type
name `.?AUB@@`. Size = `8 + strlen(name) + 1`; `??_R0H@8` (for `int`) is `0xb` bytes
with name `.H`. `??_R0` is the **only** RTTI record that lands in `.data` — `??_R1`,
`??_R2`, `??_R3` and `??_R4` all go to `.rdata$r` (`0x40301040`, read-only, no
`MEM_WRITE`), which is why they are separated: the type descriptor is writable
because the runtime patches its vftable pointer, and the rest is not.

**Confirmed on the workload, and the separation is total.** Of the 9,139 `.data`
sections, **8,581 hold exactly one `??_R0` and nothing else**, 558 hold only
ordinary data, and **zero mix the two** — RTTI always gets its own COMDAT `.data`.
`??_R1`–`??_R4` never appear in a `.data` at all. Two further independent routes
reached the same conclusion: this lane's `d_rtti_dyncast` probe, and a separated-axis
`/GR`-on/off probe in another lane that put `??_R0?AUA@@@8` and `??_R0?AUB@@@8` in
`.data` while `??_R1`–`R4` went to `.rdata$r`. Treat it as settled.

**Emission-order fact for whoever owns the RTTI rung.** In **5,530 of the 9,136**
positionally-interior `.data` sections, the section immediately before *and* the
section immediately after are both `.rdata$r`. The `??_R0` COMDAT is emitted
**inside the RTTI group** — between the `.rdata$r` records of the same type — not
batched with the initialized data. That is a different lane's question, but it is
this census's finding and it belongs where they will find it.

### 4.4 What does *not* go to `.data`

| source | lands in | note |
|---|---|---|
| `const int ci = 7;` (internal linkage), **unreferenced** | **nowhere** — dropped entirely | zero sections added |
| `const int ci = 7;` referenced | `.rdata`, **non**-COMDAT, `0x40300040` | folded with other `const` in one `.rdata` |
| `extern const int ce = 9;` | `.rdata`, non-COMDAT | **kept even when unreferenced** — the linkage forces emission |
| `static const int k` class member | `.rdata` **COMDAT** `0x40301040`, Selection ANY | |
| `volatile int v2 = 3;` | `.data`, ordinary | `volatile` changes only the symbol's decoration (`?v2@@3HC`) |
| `__declspec(thread) int t2 = 4;` | **`.tls$`**, `0xC0300040` | initialized and uninitialized thread-locals **share one `.tls$`**, laid out together; there is no `.tls$` equivalent of the `.bss`/`.data` split |
| string literal, under `/GF` (implied by `/O1`) | `.rdata` COMDAT `??_C@…` | `OBJ_DYNINIT_SHAPE.md` §5 |

The `.rdata` cell is worth one extra line because it discriminates: `const int ci=7;
extern const int ce=9; const char cs[4]="abc";` with all three referenced yields
**one** `.rdata` of 8 bytes, `00 00 00 09 61 62 63 00` — `ce` at 0 and `cs` at 4,
and `ci` **not present at all** because it was constant-folded into the code. So
"`const` is dropped when unreferenced" is really "`const` is dropped when every use
was folded", and the address-taken case is what keeps it.

---

## 5. Address assignment — the core of the spec

This is the part a writer cannot guess, and the part §7 of `OBJ_DYNINIT_SHAPE.md`
declined.

### 5.1 The two questions, separated

Laying out a section needs (a) an **order** in which to walk the objects and (b) an
**allocator** that turns that walk into offsets. They are independent, and the
project's earlier confusion came from reading a permuted output and assuming the
allocator was doing the permuting. It is not: the allocator is a straight bump, and
the *order* is an input.

### 5.2 `.bss` walk order — the rule

> **Rule A1.** Partition the namespace-scope objects into **eager** (no dynamic
> initializer) and **deferred** (has one). Walk the eager objects in **IL `.gl`
> symbol-record order**, then the deferred objects in **reverse `.gl` record
> order**. The two groups never interleave; every eager object gets a lower address
> than every deferred object.

Measured, at workload flags, with the `.gl` read directly out of the captured IL
(`c2rs capture --keep-il`), independently of anything the disassembly says:

| cell | `.gl` record order | `.bss` ascending | verdict |
|---|---|---|---|
| 8 uninit externs | `d2 p3 p2 d3 d4 d1 p1 p4` | `d2 p3 p2 d3 d4 d1 p1 p4` | **= `.gl`** |
| 6 uninit externs, word names | `charlie delta foxtrot bravo echo alpha` | same | **= `.gl`** |
| 8 uninit statics + one function each | `d2 p3 p2 d3 d4 d1 p1 p4` | same | **= `.gl`** |
| 8 `static L(1)` dyninit | `d2 d1 d4 p3 p2 d3 p4 p1` | `p1 p4 d3 p2 p3 d4 d1 d2` | **= reverse(`.gl`)** |
| 6 `static L(1)` dyninit, word names | `alpha echo charlie delta foxtrot bravo` | `bravo foxtrot delta charlie echo alpha` | **= reverse(`.gl`)** |
| 4 plain + 4 dyninit | `d2 d1 d4 p3 p2 d3 p1 p4` | `p3 p2 p1 p4` ∥ `d3 d4 d1 d2` | **eager = `.gl`↾p, deferred = reverse(`.gl`↾d)** |

The mixed row is the one that pins the mechanism rather than a correlation: the
eager block is the `.gl` order **restricted to eager objects** and the deferred
block is the reverse of the `.gl` order **restricted to deferred objects** — a
single interleaved list could not produce that.

**Where the names live in the `.gl`.** External-linkage objects appear under their
decorated name (`?p1@@3DA`); internal-linkage objects appear as **`$` + the source
identifier** (`$p1`). Both are in the same record stream and the walk does not
distinguish them — the third row above is a static-only TU and behaves identically
to the extern-only first row.

**Declaration order is irrelevant.** Four random source permutations of the same 8
names give one `.gl` order and one `.bss` order:

```
decl d4 d3 d1 p4 d2 p3 p1 p2   .gl = d2 p3 p2 d3 d4 d1 p1 p4   .bss = d2 p3 p2 d3 d4 d1 p1 p4
decl d4 d2 d1 p2 p1 d3 p3 p4   .gl = d2 p3 p2 d3 d4 d1 p1 p4   .bss = d2 p3 p2 d3 d4 d1 p1 p4
decl p2 d1 d2 p1 p4 d4 d3 p3   .gl = d2 p3 p2 d3 d4 d1 p1 p4   .bss = d2 p3 p2 d3 d4 d1 p1 p4
decl p2 p1 d3 d4 d1 p3 d2 p4   .gl = d2 p3 p2 d3 d4 d1 p1 p4   .bss = d2 p3 p2 d3 d4 d1 p1 p4
```

### 5.3 `.data` walk order — the rule

> **Rule A2.** `.data` objects are walked in **declaration (source) order**, not in
> `.gl` order.

Five cells, with names and declaration orders chosen so that source order, sorted
order and `.gl` order are pairwise different:

```
decl zulu alpha mike bravo yankee charlie  ->  .data zulu alpha mike bravo yankee charlie   (=decl, !=sorted)
decl charlie yankee bravo mike alpha zulu  ->  .data charlie yankee bravo mike alpha zulu   (=decl, !=sorted)
decl s9 s1 s7 s3 s5 s2                     ->  .data s9 s1 s7 s3 s5 s2                      (=decl, !=sorted)
```

with the `.gl` for the same sources being `charlie delta foxtrot bravo echo alpha`
— so the same TU permutes its `.bss` and does **not** permute its `.data`. The
prereg's P8 named this as its alternative; it is the right one. The same holds for
`static` initialized objects.

**Trap.** For one of the name sets above the `.bss` order came out
`zulu yankee mike charlie bravo alpha`, which is exactly reverse-alphabetical. It is
a coincidence of that name multiset; the other cells kill "sorted" outright.

### 5.4 The allocator

> **Rule A3.** One cursor per section, starting at 0. For each object in walk order,
> with size `n` and declared/natural alignment `t`:
>
> ```
> align(obj) = max(t, 1 if n < 2 else 4 if n < 64 else 8)
> ```
>
> Round the cursor up to `align(obj)`; the skipped bytes become a **hole**. Place the
> object, advance the cursor past it. Before taking from the cursor, first try to
> place the object in an existing hole — **lowest-addressed hole that fits at the
> object's alignment** — splitting the hole around it. The section's
> `SizeOfRawData` is the final cursor value.

The size-promotion term is the same one §3.2 measures for the section nibble, and it
is what makes the model work: a plain `natural` alignment scores 7/18 where the
promoted one scores 14/18 (§5.5).

Worked example — `.data` of
`char d1=1; short d2=2; int d4=4; double d8=8.0; const char* dp="hi"; char* dq=&d1; int arr[4]={1,2,3,4};`
walked in declaration order:

| object | n | align | cursor before | placed at | why |
|---|---:|---:|---|---|---|
| `d1` | 1 | 1 | 0 | **0x00** | |
| `d2` | 2 | 4 | 1 | **0x04** | hole `[1,4)` created |
| `d4` | 4 | 4 | 6 | **0x08** | hole `[6,8)` created |
| `d8` | 8 | 8 | 0xc | **0x10** | hole `[0xc,0x10)` created |
| `dp` | 4 | 4 | 0x18 | **0x0c** | **fills the hole** — this is the row that refutes "layout order" |
| `dq` | 4 | 4 | 0x18 | **0x18** | |
| `arr` | 16 | 4 | 0x1c | **0x1c** | |

Final cursor `0x2c` = the measured `SizeOfRawData`, and all seven offsets match the
obj. Two relocations, both `IMAGE_REL_PPC_ADDR32` (`0x0002`) with **no PAIR**, at
`0x0c → ??_C@_02PCEFGMJL@hi?$AA@` and `0x18 → ?d1@@3DA` — i.e. an address-valued
initializer stores zero bytes and carries the address entirely in the relocation,
exactly as `.CRT$XCU` does.

A second worked example, `.bss`, showing the hole reuse across an over-aligned
object — `char da; __declspec(align(16)) char db; char dc; __declspec(align(16)) char dd;`
with `.gl` order `da dd db dc`:

| object | align | cursor | placed at |
|---|---:|---|---|
| `da` | 1 | 0 | **0x00** |
| `dd` | 16 | 1 | **0x10**, hole `[1,0x10)` |
| `db` | 16 | 0x11 | **0x20**, hole `[0x11,0x20)` |
| `dc` | 1 | 0x21 | **0x01** — lowest hole that fits |

Measured: `da@0 dc@1 dd@0x10 db@0x20`, `SizeOfRawData = 0x21`, nibble ALIGN_16. ✓

### 5.5 What the allocator model does **not** cover

Scored on random cells drawn from 11 object types (sizes 1…100, natural alignments
1/2/4/8), walk order taken from the actual `.gl` (for `.bss`) or the declaration
(for `.data`), against the real obj's offsets **and** its `SizeOfRawData`:

| section | model | exact cells |
|---|---|---|
| `.bss` | `align=size-promoted`, lowest-fit hole | **14 / 18** |
| `.bss` | `align=size-promoted`, no hole reuse | 12 / 18 |
| `.bss` | `align=natural`, lowest-fit hole | 7 / 18 |
| `.data` | `align=size-promoted`, lowest-fit hole | **12 / 14** |
| `.data` | `align=size-promoted`, no hole reuse | 4 / 14 |

**It is exact on every cell whose objects share one size** — that includes all of
§7.1's three families at every N from 1 to 10, and the uniform-`int` control
(`u7@0 u2@4 u1@8 u3@c u5@10 u4@14 u8@18 u6@1c`, section size `0x20`). The failures
all involve mixed sizes, and in each the *order* deviates from the walk, which a
bump allocator cannot do. Both counterexamples verbatim, so a later lane starts from
data rather than re-deriving:

```
cell 10   .gl : vk2(1,1) vk5(64,1) vk0(2,2) vk4(16,8) vk6(5,1) vk1(3,1) vk3(5,1)
          obs : vk2@0 vk5@8 vk0@48 vk1@4c vk4@50 vk6@60 vk3@68     size 0x6d
          — vk1 is allocated BEFORE vk4, though it is later in the .gl;
            the hole [1,8) left by vk5's 8-alignment is never used.

cell 11   .gl : vl5(100,1) vl1(2,2) vl4(2,2) vl3(8,8) vl7(16,4) vl2(100,1) vl0(4,4) vl6(16,4)
          obs : vl5@0 vl1@64 vl4@68 vl7@6c vl0@7c vl3@80 vl2@88 vl6@ec   size 0xfc
          — vl3 (the only 8-aligned object) is deferred past vl7 and vl0.
```

Both look like the walk yields to alignment: an object whose alignment would force
padding is passed over in favour of a later one that fits. A "skip and retry" walk
was **not** fitted here — it would need its own held-out set, and the lane's brief
puts the byte-level spec ahead of it.

**Independent check against lane w-map.** A parallel lane reading `c2.dll`'s
disassembly reports the `.bss` allocator as *"align 8, then `+(8−size)` for size ∈
{1,2,4}"* — a right-justification into 8-byte slots. **That does not reproduce these
objs.** `int z1=0; int z2; int z3={0};` gives `z2@0 z1@4 z3@8`, and right-justifying
4-byte objects in 8-byte slots would give `z2@4 z1@0xc z3@0x14` with a section three
times the size. Either that routine is not the one on this path, or it is guarded.
Reported as a disagreement, not resolved.

---

## 6. Symbols

### 6.1 The defined-data symbol

`Value` = the byte offset within the section. `SectionNumber` = the section's 1-based
index. `Type` = `0x0000` (data; contrast `0x0020` for a function). `NumberOfAuxSymbols`
= 0. Storage class and name follow linkage:

| linkage | StorageClass | name |
|---|---|---|
| `static` / anonymous namespace | **3 (STATIC)** | the **undecorated** source identifier — `s1`, `cs` |
| external | **2 (EXTERNAL)** | the **decorated** name — `?d1@@3DA`, `?m@S@@2HA`, `?v2@@3HC` |

A `__declspec(selectany)` object is EXTERNAL and is the COMDAT's defining symbol.
`??_R0…` is EXTERNAL with `Type = 0`.

Workload storage-class histograms, over the symbols defined in each section kind:

| | EXTERNAL (2) | STATIC (3) |
|---|---:|---:|
| `.data` | 9,176 | 111 |
| `.bss` | 5,318 | **10,056** |

`.bss` is majority-STATIC and `.data` overwhelmingly EXTERNAL, which is the expected
consequence of §4.4: an internal-linkage object with a constant initializer is
usually folded away or diverted to `.rdata`, so what survives into `.data` is mostly
external, while file-scope `static` scratch buffers land in `.bss`.

### 6.2 Symbol-table order

The symbol table follows section order, and within a section's group the section
symbol + aux comes first. The order of the *defined* symbols inside the group is
**linkage- and eagerness-dependent**, and this is the one field in this document
that is not reduced to a single rule:

| section | objects | symbol-table order | equals |
|---|---|---|---|
| `.data` | initialized, either linkage | `p1 p2 p3 p4 d1 d2 d3 d4` | **declaration order** (= ascending address, since §5.3) |
| `.bss` | eager, **EXTERNAL** | `p4 p1 d1 d4 d3 p2 p3 d2` | **reverse `.gl`** (= descending address) |
| `.bss` | eager, **STATIC** | `p1 p2 p3 p4 d1 d2 d3 d4` | **declaration order** (neither ascending nor descending) |
| `.bss` | deferred (dyninit), **either** linkage | `d2 d1 d4 p3 p2 d3 p4 p1` | **`.gl` order** (= descending address) |
| `.bss` | eager, **mixed** linkage | `p1 d1 d3 p3` ∥ `p2 p4 d2 d4` | **all externals first in reverse `.gl`, then all statics in declaration order** |

> **Rule Y1 (eager `.bss`).** Emit every EXTERNAL symbol first, in reverse `.gl`
> record order; then every STATIC symbol, in declaration order.
>
> **Rule Y2 (deferred `.bss`).** Emit in `.gl` record order regardless of linkage.

Y1 was fitted on the extern-only and static-only cells and **confirmed
out-of-sample by the mixed cell**, which it predicts exactly and which no simpler
rule (ascending, descending, or declaration) matches. Y2 is fitted on two cells
(static and extern dyninit) and is **not** independently confirmed.

`OBJ_DYNINIT_SHAPE.md` §7.1's *"the `.bss` symbols are listed in strictly descending
address order in every same-kind cell"* is true for the three shapes above where it
happens to coincide, and **false for eager statics**, which are in declaration order
— the shape that document did not have.

### 6.3 Undefined externals

`extern int eonly;` used but not defined contributes **no section** and one symbol:
`SectionNumber = 0`, `Value = 0`, `StorageClass = 2`, `Type = 0`, no aux. Exactly as
predicted (P7).

### 6.4 A storage class this lane did not expect

`??_EB@@UAAPAXI@Z` (the vector-deleting-destructor alias emitted with RTTI) appears
as an undefined symbol with **`StorageClass = 105`** (`IMAGE_SYM_CLASS_CLR_TOKEN` in
the modern header, used here as a weak-external-ish marker). It is out of this
lane's seam but a `.data`/RTTI writer will meet it, so it is recorded.

---

## 7. The permutation — bounded, keyed, and then dissolved

### 7.1 The table across N

Three families, N = 1…10, at workload flags, each object 1 byte so the allocator is
the identity and only the order shows. `.gl` relation checked on every row.

| N | A: `static L sN(1)` (dyninit) | B: `static char sN` + fn ref | C: `char sN` (extern) |
|---:|---|---|---|
| 1 | `s1` | `s1` | `s1` |
| 2 | `s1 s2` | `s2 s1` | `s2 s1` |
| 3 | `s3 s1 s2` | `s2 s3 s1` | `s2 s3 s1` |
| 4 | `s4 s3 s1 s2` | `s2 s3 s1 s4` | `s2 s3 s1 s4` |
| 5 | `s4 s3 s5 s1 s2` | `s2 s5 s3 s1 s4` | `s2 s5 s3 s1 s4` |
| 6 | `s6 s4 s3 s5 s1 s2` | `s2 s5 s3 s1 s6 s4` | `s2 s5 s3 s1 s6 s4` |
| 7 | `s6 s4 s3 s5 s7 s1 s2` | `s2 s5 s3 s7 s1 s6 s4` | `s2 s5 s3 s7 s1 s6 s4` |
| 8 | `s6 s4 s3 s5 s7 s8 s1 s2` | `s2 s5 s3 s7 s8 s1 s6 s4` | `s2 s5 s3 s7 s8 s1 s6 s4` |
| 9 | `s6 s4 s3 s5 s7 s8 s9 s1 s2` | `s2 s9 s5 s3 s7 s8 s1 s6 s4` | `s2 s9 s5 s3 s7 s8 s1 s6 s4` |
| 10 | `s6 s4 s10 s3 s5 s7 s8 s9 s1 s2` | `s2 s9 s5 s3 s7 s10 s8 s1 s6 s4` | `s2 s9 s5 s3 s7 s10 s8 s1 s6 s4` |

Every A row is `reverse(.gl)`; every B and C row is `.gl`.

* **B ≡ C at every N.** Internal vs external linkage changes the decorated name
  completely (`s1` vs `?s1@@3DA`) and changes nothing about the order — the key is
  the **source identifier** (H-C).
* **A ≠ B.** Same identifiers, different order, because the front end interns
  dynamic-initializer objects at a different point; and A is additionally reversed by
  c2's deferred-list drain.
* **Fixed points.** N = 1 is the identity in all three families. N = 2 is the
  identity in family **A only** (`s1 s2`), and is the transposition in B and C.
  §10.16 recorded `2: s1 s2` from family A and `6: s6 s4 s3 s5 s1 s2` from family A;
  both reproduce.
* §10.16's declined N = 6 row is family A's, and it reproduces exactly.

### 7.2 What it keys on

| axis varied, everything else fixed | order changes? |
|---|---|
| **the names** (rename in place, `alpha bravo charlie delta` ↔ `delta charlie bravo alpha`) | **no** — both give `charlie delta bravo alpha`. The order follows the *name set*, not the position |
| **declaration order** (4 random permutations of 8 names) | **no** |
| **linkage** (`static` vs extern, and mixed in one TU) | **no** |
| **type / decoration** (same identifiers, 8 different types) | **no** |
| **object size** (all 1 byte vs `double`, and mixed) | **no** for the walk; **yes** for the resulting addresses (§5.4), and in 4/18 mixed-size cells the address order deviates from the walk (§5.5) |
| **run-to-run** (3 recompiles of one source) | **no** — identical |
| **dynamic initializer present** | **yes** — reverses the group and changes the `.gl` order itself |
| **`.data` vs `.bss`** | **yes** — `.data` does not permute at all (§5.3) |

### 7.3 The bucket partition, as data

Before the `.gl` correspondence was found, this lane extracted the partition
black-box, and it is kept because it is a **front-end** fact that a `c1xx` lane will
want and that constrains any white-box claim about that hash:

* Shuffling declaration order permutes only **adjacent runs** of the output — the
  signature of a chained hash table walked bucket-ascending with head insertion.
  Using that, the boundaries are recoverable: 26 random declaration orders over
  11,000 names give a partition that is **consistent on all 26** (asserted, not
  eyeballed).
* **1024 buckets**, exactly: 11,000 names occupy 1024 distinct buckets; an
  independent 8,000-name run occupies 1023.
* The single-letter bucket indices are not linear in the character
  (`a→802 b→537 c→669 d→45 e→702 f→965 g→969 h→641 i→531 j→223 k→91 l→876 m→127
  n→371 o→687 p→912 q→904 r→477 s→580 t→526 u→628 v→545 w→48 x→98 y→2 z→443`), and
  a GF(2)-linearity test on the character bits fails
  (`h(a)^h(b)^h(d)^h(g) ≠ 0` where `'a'^'b'^'d'^'g' = 0`), which rules out CRC-like
  and pure shift-xor forms for the single-character case.
* **7,452 hash configurations produce nothing**: 12 accumulator forms (`mul`, `xmu`,
  `mux`, `rox`, `roa`, `rrx`, `rra`, `shx`, `sha`, `sxr`, `sdbm`, one-at-a-time) ×
  19 multipliers × 6 initial values × 9 input transforms (raw, NUL-terminated,
  reversed, length-prefixed, upper-cased, `?`/`_`-prefixed, `@`-suffixed) × 7 output
  folds × 12 shifts, scored by partition agreement (walk-order agnostic). Best score
  **0.08** against a chance baseline of **0.03**. CRC-32 in three polynomials and
  both bit orders, FNV-1/1a, PJW and the PDB `LHashPbCb` are all in that set and all
  miss.

The full 11,000-name → bucket map is regenerable with `work/w-bss/bucket.py`.

### 7.4 Why it does not matter for the port

The 1024-bucket table is **`c1xx`'s**, not c2's. Its output — the order of the name
records in the `.gl` stream — is part of the IL that c2 consumes, and §5.2 shows c2
does nothing to it but walk it (forwards for eager objects, backwards for deferred
ones). **A port that reads the `.gl` in file order reproduces the permutation
exactly and never computes a hash.**

This is also why the search in §7.3 was doomed in a way that is worth recording: it
was fitting a c2 hash to a c1xx artefact. The corroborating disassembly evidence from
lane w-map agrees — c2's own symbol table is keyed by a sequential integer id
(`bucket = id & 0x3ff`), not by name, so no name-keyed order can come out of it.
Two independent methods, one conclusion.

**Scope.** This dissolves the permutation for **c2**. It does **not** dissolve it for
a front-end port; `docs/` already tracks a `c1xx` lane, and §7.3 is that lane's
starting data.

---

## 8. What a writer still cannot build from this document

Stated plainly, because the value of the document is bounded by this list.

1. **Mixed-size allocation, 4 cells in 18.** §5.5. Exact for uniform-size objects
   and for the two hand-worked mixed cells in §5.4; not a closed rule. A writer
   restricted to TUs whose `.bss`/`.data` objects share one size is safe; one that
   is not needs the skip-and-retry walk fitted and held out.
2. **The FP `.data` CheckSum predicate is now determined** (§4.2.1: byte-granular,
   per initializer member, padding retained, 11/11 held-out plus 3 exploratory
   cells) — but *why* it happens is still a hypothesis. Nothing here observes the
   CRC call site. A writer implements the subset rule and does not need the reason.
3. **Deferred-`.bss` symbol order (Rule Y2)** is fitted on two cells and has no
   held-out confirmation. Y1 does.
4. **`.tls$`** is characterised only to the level of "one section, initialized and
   uninitialized share it, `0xC0300040`". Its walk order was not measured.
5. **Whether "has a dynamic initializer" is exactly the deferral predicate**, versus
   something broader. Every cell here is consistent with the narrow reading; none
   discriminates it from, say, "address taken by a COMDAT".
6. **`.data` relocations beyond `ADDR32`-with-no-PAIR.** Only pointer-valued
   initializers were exercised. Member-pointer, vftable-pointer and cross-section
   initializers were not.
7. **The `??_R0` payload's spare word** is `00 00 00 00` in every cell measured; no
   cell was built that would make it non-zero, so it is *observed constant*, not
   *known constant*.
8. **The workload census covers headers, characteristics, symbols and section
   order — not addresses.** `sections.jsonl` does not carry the raw section bytes,
   so §5's allocator is validated on probes only. Re-running the census with raw
   bytes retained would let §5.4/§5.5 be graded on all 24,055 real sections, and is
   the obvious next gate.
9. **`.tls$` multiplicity and COMDAT behaviour** were not censused; only the probe
   cells in §4.4 speak to it.

---

## 9. Proposed board rows

Next free number is **#162**.

* **#162 — `.data`/`.bss` writer.** Build the COFF writer for the two sections from
  §2–§6. Gate: byte-exact on the probe grid, then on the workload TUs whose only
  missing sections are `.data`/`.bss`. Blocked-by: §8.1 for mixed-size TUs.
* **#163 — the skip-and-retry walk.** Close §5.5 / §8.1. Prereg a rule, commit
  predictions for a held-out mixed-size set, then measure.
* **#164 — revise `OBJ_DYNINIT_SHAPE.md` §7.1 and §2.3.** §7.1's decline is
  superseded by §5.2 here; §2.3's "non-COMDAT sections carry CheckSum 0" is refuted
  for `.data` by §4.2; §4.1's "`.bss` and `.CRT$XCU` always last" is refuted by §2.2.
* **#165 — `c1xx` name-hash lane.** §7.3 is its data: 1024 buckets, an 11,000-name
  partition, and a refuted-family list.

---

## 10. Reproducing this

```sh
cd .claude/worktrees/w-bss/work/w-bss
printf '/nologo /wd4355 /wd4164 /c /GR /O1 /Oi /EHsc\n' > flags-w.txt

WBSS_FLAGS=flags-w.txt python3 bat3.py      # §2,§3,§4 — section order, headers, RTTI
WBSS_FLAGS=flags-w.txt python3 glorder.py   # §5.2,§5.3 — .gl order vs .bss/.data order
python3 alloc.py  20260804 18               # §5.5 — .bss allocator fit
python3 allocd.py 7 14                      # §5.5 — .data allocator fit
python3 bucket.py                           # §7.3 — the 1024-bucket partition (slow)
```

`probe.py` compiles one source with the real toolchain and reads the obj back;
`glorder.py` additionally captures the IL and reads the `.gl` record order.
`coffdump.py` is the scratch COFF reader (a copy of `tools/coffdump.py` with a
dict-based `Obj`). Probe sources and objs are gitignored scratch per the project
rule; every byte quoted above is transcribed here so the document stands without
them.

---

## 11. The workload census

`work/w-bss/census/sections.jsonl` — **committed** (with `git add -f`; `work/` is
gitignored) so the measurement is reproducible without re-deriving it. One record
per obj: source path, section count, the full ordered section-name list (with the
two `.XBLD$W` distinguished as `:C2` / `:C1`), and for every `.data` and `.bss` the
header fields, decoded characteristics, COMDAT flag, the section symbol's aux
record, and every symbol defined in it. 871 objs, 9,139 `.data`, 14,916 `.bss`.
No absolute machine paths, in the data or in the scripts.

Regenerate with `census/one.sh` + `census/census.py`; the aggregates quoted above
come from `census/agg*.py`. The reference objs themselves (102 MB) are **not**
committed and were deleted after extraction, per the project rule.

**Why 871 and not 878.** Seven TUs never produce an obj, and they fail in `c1xx`
before c2 is ever reached: C2084/C2512 (duplicate function bodies), C1189
(`#error` wrong-platform guards) and C1083 (missing `windows.h`). They are
unmeasurable at any effort and at any flag setting, so **871 is the terminal
denominator** for section-shape work, not a temporary instrument limit.

**A caution about how this census was gathered.** The capture cache holds 943,194
entries. A shell glob over it (`work/capture-cache/*/`, `**`, `grep -r`, `find`
from the repo root) makes the shell materialise every path and allocates tens of
gigabytes; doing so once OOM-killed this machine. Iterate an **explicit resolved
list** built from `work/dc3-workload/files.txt`, never a wildcard over the cache
root.

---

## 12. Controls

* **Every probe is checked for the section it was supposed to produce**, its symbol
  count and its section size, *before* any ordering is read off it
  (`probe.order_extern` asserts the name multiset and
  `SizeOfRawData == len(names)`). A probe whose object was silently optimized away
  would otherwise read as a permutation.
* **Bytes come from the obj, never from a `/FAsc` listing.** §10.16 was misled by a
  listing once; `OBJ_DYNINIT_SHAPE.md` §6 records why the listing is not evidence
  for section order.
* **Grade at the workload's flags.** `/GR` is in them and is not in the CLI default
  set; a probe compiled with defaults silently lacks RTTI, which would have hidden
  every `??_R0` finding in §4.3.
* **The real `c2.dll` under wibo is the only judge.** No expected obj is constructed
  anywhere in this lane, and no rule in this document was accepted on the strength
  of a disassembly reading — §5.5 records one place where the disassembly and the
  objs disagree, and the objs win.
