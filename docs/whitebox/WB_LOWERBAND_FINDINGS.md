# `WB_LOWERBAND` — the `[sym+0x50]` chain, enumerated: **there is no reduction, because there is no writer**

> **PROVENANCE — DISASSEMBLY-DERIVED.** Every address below is an absolute VA in
> `c2.dll` sha256 `c80981c015166effecc71ad8112d5577a065b2300891dfdb02b9c13787a66258`,
> verified on this tree before the image was opened. Navigation only; nothing
> here enters `crates/` and this lane proposes no
> [`DISCLOSURE.md`](DISCLOSURE.md) row.
>
> Lane `w-lowerband`, 2026-08-28, decision 21. Prereg
> [`work/w-lowerband/PREREG.md`](../../work/w-lowerband/PREREG.md), committed at
> `19d6c4797` **before** the first `grep` of the export. Board **#3731**–**#3736**.
> Score it against the prereg; a findings document read on its own is a story.

---

## 0. The one-paragraph answer

`P_INLINE` §6.6.1 names, as the first of C8's two missing links, that
`[sym+0x50]` *"is **reduced by every pass that runs between there and
`0x10b5fc8a`**"* and that *"nothing yet located reads that reduction."*
**Nothing has located it because it does not exist.** The field has **exactly
one writer in the whole image** — the `.gl` decode at `0x10b9bf6c`, storing
`il-read-varint16`'s return **verbatim, with no arithmetic** — and **nine
readers**, all enumerated below. Three independent instruments agree, and two
of them were watched failing first. **The reduction chain has length zero.**

**And §2.1b's conclusion survives anyway, which is the part that matters.**
Re-reading `w-sizebracket`'s own 168 committed cells against the ceiling this
lane re-derived from the image (128) gives **eight false positives and eight
false negatives at `/O1`** — so the value C8 tests is definitively **not** the
`.gl` `SIZE`. The *conclusion* is right; the *mechanism* named for it is
refuted. **The missing link is therefore AT the store, not after it** — a much
smaller object than "every pass in between", and §1.4 names the three candidates
in the order the evidence supports.

**What is on the path is a POINTER-SELECTION chain, not a value chain: three
sites, and two of them are INSIDE the inliner band** that §6.6 says cannot
contain the answer.

---

## 1. The reference set, complete

### 1.1 How it was enumerated, and over what population

Three instruments, because *"nothing writes X"* is a claim about a tool until
the reference set is enumerated — this repo's most repeated defect
(`README.md`; four prior instances, most recently `w-regcells`' 213 cells).

| id | instrument | population | result |
|---|---|---|---|
| **E1/E2** | [`work/w-lowerband/f50.py`](../../work/w-lowerband/f50.py) | the **independent objdump boundary set** — `objdump -d -M intel`, **424,232** decoded instructions | **125** instructions with a memory operand at `+0x50`, split by width and direction |
| **E3** | Ghidra's decompiler, `decomp_all.c` (control-flow-driven, not linear) | the whole export | **0** `ushort` assignments at `+0x50` image-wide; **13** read occurrences over **12** lines — which is **9 distinct instructions** (see below) |
| **E5** | [`work/w-lowerband/bytescan.py`](../../work/w-lowerband/bytescan.py) | **all 1,232,384 raw bytes of `.text`**, decode-independent, **2,136** encoding patterns | **exactly one** 16-bit-store encoding present |
| **E4** | [`work/w-lowerband/fieldmap.py`](../../work/w-lowerband/fieldmap.py) | 67 functions referencing `+0x50`, 207 referencing `+0x4c` | **29** touch both; the struct filter in §1.3 |
| **E4b** | [`work/w-lowerband/dwordwrites.py`](../../work/w-lowerband/dwordwrites.py) | all **17** dword stores/RMWs at `+0x50` | **0** are on this record; **0** left needing a hand read |


> **The 424,232 and `#3721`'s 425,871 are the same set counted two ways, and
> the difference is reconciled rather than left to look like a discrepancy.**
> The listing has **425,871** addressed lines; **1,639** of them are
> byte-continuation lines carrying no mnemonic (`10b5e4de:\t03 00 00`, the tail
> of the seven-byte `mov` above it). `425,871 − 1,639 = 424,232` decoded
> instructions. `#3721` counts addressed lines, which is the right denominator
> for an *alignment* question; this lane counts decoded instructions, which is
> the right one for an *operand* question. Neither is wrong.


> **13 expressions, 12 lines, 9 instructions — reconciled, not left to look
> like a discrepancy.** Ghidra re-materialises the load wherever it appears in
> the C it prints, so one instruction can produce several occurrences: the
> `CARRY4` idiom at readers 7 and 8 prints the load **twice** each
> (`bVar = CARRY4(acc, x); acc = acc + x;`), reader 3's compound test prints it
> twice on one line, and readers 4/5 contribute three occurrences across three
> lines. `1+1+2+3+1+2+2+1 = 13` occurrences ← **9 instructions**, which is
> exactly E2's word-read count. The two instruments agree on the object and
> differ only in what they count.

**E4b is P1 form (b) taken to the end**, because a 32-bit store writes `SIZE`
without appearing in any 16-bit enumeration. **Its control went RED on the
first run and the miss is reported rather than repaired away**: the signature
recognises functions that *build* the record (the `.gl` reader touches `+0x37`
thirteen times) and does **not** recognise functions that merely *consume* it
(candidacy and the charge touch only `+0x4c` and `+0x50`). That bounds the
filter to the **writer** question — which is the question — and the bound is
printed in its own output. `+0x30` was dropped from the signature as far too
common a displacement to discriminate; it had been the sole reason all seven of
the first run's flags fired.

**E5 exists because a linear disassembler can desynchronise.** `objdump`
sweeps `.text` from the section start, and `c2.dll` has a ~150 KB data block at
the head of `.text` (Ghidra's first function is `0x10b266d0`); a store hidden
inside a desynchronised run would be invisible to E1/E2. E5 searches for the
*encodings* instead — every `66`-prefixed store and RMW form (`mov`, `add`,
`sub`, `or`, `and`, `xor`, `adc`, `sbb`, `xchg`, group1 imm8/imm16, group3,
group5, the shift groups) at `disp8 = 0x50`, at `disp32 = 0x00000050`, and
through SIB; plus both byte halves at `+0x50`/`+0x51`. It accepts false
positives by construction and found 197 candidate positions, of which **one** is
a 16-bit store.

**Controls, watched before any count above was quoted**
([`work/w-lowerband/controls.out`](../../work/w-lowerband/controls.out), `#3336`):

| control | what it does | verdict |
|---|---|---|
| C1 | E2 must find the known store at `0x10b9bf6c` | **GREEN** |
| C2 | E2 re-run on a listing whose one line reads `+0x51` | **watched RED** |
| C3 | E5 must find exactly one 16-bit store encoding | **GREEN**, count 1 |
| C4 | E5 re-run on a **copy** of the image with that one byte patched `0x50` → `0x51` | **watched RED** |
| C5 | the patched copy's sha256 differs from the tree's; the tree's image is unmodified | **GREEN** |

A control nobody has watched fail is decoration. C2 and C4 were watched failing.

### 1.2 The writer — one, and it is verbatim `[R]`

```
10b9bf50:  81 4e 37 00 00 20 00   or     DWORD PTR [esi+0x37],0x200000   <- marks the record kind
10b9bf57:  e8 8d 3a 08 00         call   0x10c1f9e9   (i32c)
10b9bf5c:  89 46 54               mov    DWORD PTR [esi+0x54],eax
10b9bf5f:  e8 85 3a 08 00         call   0x10c1f9e9   (i32c)
10b9bf64:  89 46 58               mov    DWORD PTR [esi+0x58],eax
10b9bf67:  e8 3a 3a 08 00         call   0x10c1f9a6   (i16c)
10b9bf6c:  66 89 46 50            mov    WORD PTR [esi+0x50],ax          <-- THE ONLY WRITER
10b9bf70:  e8 a6 39 08 00         call   0x10c1f91b
10b9bf75:  83 e0 fb               and    eax,0xfffffffb                 <- ATTR bit 2 CLEARED on load
10b9bf78:  89 46 4c               mov    DWORD PTR [esi+0x4c],eax
10b9bf7b:  e8 26 3a 08 00         call   0x10c1f9a6   (i16c)
10b9bf80:  66 89 46 52            mov    WORD PTR [esi+0x52],ax
```

**There is no instruction between the `call` and the `mov`.** `ax` is the
reader's return value and it is stored unmodified. §2.1a's *"exactly ONE 16-bit
store"* is **CONFIRMED**, now by two further instruments it was not originally
checked against.

**New here, and small:** `0x10b9bf75` **clears ATTR bit 2 (`0x4`)** as the
attribute word is loaded, so the in-memory `[sym+0x4c]` is never equal to the
`.gl` `ATTR` byte for a record that carries that bit. Not pursued.

### 1.3 The nine readers — all of them, with what each does `[R]`

The struct is the `.gl` **function-symbol record**, and the identification is
not by `+0x50` alone: `+0x37 | 0x200000` is set by the writer above, `+0x30`
holds the kind byte `4`, `+0x4c` is `ATTR` (confirmed from the container side by
`w-mmioclose`, C13), `+0x78` is the next-record link.

| # | addr | owner | clause | what it does |
|---:|---|---|---|---|
| 1 | `0x10b56732` | `FUN_10b566e9` | — | returns `[sym+0x50] & 0x3f`, guarded on `[sym+0x30]=='\x04' && [sym+0x37]&0x200000` — a **six-bit** view of the field, and one this page does not interpret |
| 2 | **`0x10b5fc86`** | `FUN_10b5fb5f` | **C8** | the candidacy size test's left operand |
| 3 | `0x10b60a6f` | `FUN_10b60930` | **C17** | `budget < instrs && instrs > 0x28` |
| 4 | `0x10b625b2` | `FUN_10b6242a` | **C18** | the 40-instruction test (`#3721`'s corrected address) |
| 5 | `0x10b625bd` | `FUN_10b6242a` | **C19** | `*budget -= it` and `DAT_10c3f5cc += it` |
| 6 | `0x10b626f7` | `FUN_10b62675` | **C2** | seeds the caller's count, `DAT_10c3f5cc = [*fn+0x50]` |
| 7 | `0x10b72ee6` | `FUN_10b72eca` | — | **whole-module 64-bit SUM** of the field over the record list (`DAT_10c4630c`, next at `+0x78`), for records with `[sym+0x4c] & 0x20`, into `DAT_10c46398:DAT_10c4639c` |
| 8 | `0x10b72f0f` | `FUN_10b72f0f` | — | a second 64-bit accumulator, into `DAT_10c2ebb8:DAT_10c2ebbc` |
| 9 | `0x10b8fbda` | `FUN_10b8fb47` | — | an **IL hash function**: for node kind `4` with the same `+0x30`/`+0x37` guard, the field is mixed into the hash |

Two further `+0x50` word operands exist in the listing — `0x10b02423` (`arpl`)
and `0x10b191e1` (`mov r/m16, Sreg`) — and both are in the head data block, own
no Ghidra function, and are linear-decode artifacts. They are named so the count
is reproducible, not because they are code.

**Readers 7, 8 and 9 are the reason the field's semantics are not settled by
this lane.** A quantity that is summed 64-bit-wide across the module *and* fed
into an IL hash *and* masked to six bits by reader 1 is doing more than one job,
and this page claims none of them.

### 1.4 So the missing link is AT THE STORE, and it is one of three things

§3 below shows the tested value is not the `.gl` `SIZE`. §1.2 shows nothing
changes the field after the store. Both hold only if the disagreement is at the
store itself. In the order the evidence supports:

* **(a) `glsize.py`'s `SIZE` and the value `0x10c1f9a6` returns are different
  quantities.** The harness decodes the `.gl` container; the image decodes it
  with `il-read-varint16`, whose three forms §2.1d already established are
  easy to step wrong (`0x80` → three bytes; `0x81..0xff` → **one signed byte**,
  so `0xff` reaches the consumer's `movzx` as 65,535). **Cheapest to settle**,
  and it needs no new compilation: decode the same `.gl` bytes with the image's
  own reader semantics and diff against the harness's column.
* **(b) The record C8 tests is not the record the `.gl` reader filled.** §2
  enumerates two pointer substitutions on the path.
* **(c) `DAT_10c46318` is not 128 at run time.** §4.

**None of them is "the whole of lowering", and none of them is "every pass in
between".**

---

## 2. What IS on the path: a POINTER-SELECTION chain of three sites `[R]`

The chain the brief asked for exists. It does not change the field's *value*; it
changes *which record's field is read* and *whether the test runs at all*.
**Two of the three are inside `0x10b5b86d`–`0x10b62b00`.**

### S1 — `0x10b5fb6e`: the site→symbol resolution, and it has TWO arms

```
10b5fb6e:  e8 3b c4 ff ff         call   0x10b5bfae
```

`FUN_10b5bfae` is **18 bytes with 13 callers**, and §1 of `P_INLINE` does not
name it:

```
10b5bfae:  8b 41 28               mov    eax,DWORD PTR [ecx+0x28]     ; the site's operand list
10b5bfb1:  80 78 08 04            cmp    BYTE PTR [eax+0x8],0x4       ; node kind
10b5bfb5:  8b 40 18               mov    eax,DWORD PTR [eax+0x18]
10b5bfb8:  74 05                  je     0x10b5bfbf                   ; kind 4 -> done
10b5bfba:  8b 40 08               mov    eax,DWORD PTR [eax+0x8]      ; else TWO more loads
10b5bfbd:  8b 00                  mov    eax,DWORD PTR [eax]
10b5bfbf:  c3                     ret
```

So `sym = [[site+0x28]+0x18]` when the operand node's kind byte is `4`, and
`sym = *[[[site+0x28]+0x18]+0x8]` otherwise. **A single call site with two
resolutions**, and the second dereferences one level further — an indirect or
aliased callee.

### S2 — `0x10b5fbf3`: the alias redirect, **IN BAND** and previously unread

```
10b5fbde:  83 3d 20 de c3 10 01   cmp    DWORD PTR ds:0x10c3de20,0x1
10b5fbe5:  75 12                  jne    0x10b5fbf9
10b5fbe7:  f7 86 94 00 00 00 ...  test   DWORD PTR [esi+0x94],0x400
10b5fbf1:  74 06                  je     0x10b5fbf9
10b5fbf3:  8b b6 90 00 00 00      mov    esi,DWORD PTR [esi+0x90]     <-- C8's OPERAND IS REPLACED
```

When `DAT_10c3de20 == 1` **and** `[sym+0x94] & 0x400`, `esi` — the pointer C8's
`movzx` at `0x10b5fc86` dereferences — becomes **`[sym+0x90]`, a different
record**. §6.5 named `[sym+0x94] & 0x400` and `[sym+0x90]` among *"fields this
page had never named and are NOT pursued"*. They are named here as **the second
source of C8's left operand**, which is a different status.

`DAT_10c3de20` is **BSS, zero at load** (`peread.py`), so its run-time value is
not readable from the image; it has **10 direct writers** and is compared against
`0`, `1` and `2` at **389** reference sites image-wide. The same three-valued
selector §6.5 declined to name.

### S3 — `0x10b624c6`/`0x10b624dc` + `0x10b62557`/`0x10b6255a`: the charge SAVES, OVERRIDES and RESTORES the favour-speed bit — **IN BAND**

`DAT_10c2e310` is the global whose non-zero value makes C8's size test
**skipped** (`0x10b5fc7e`; `ebx` is `0` there, zeroed at `0x10b5fc42`/
`0x10b5fc67` — §2.1's correction is right and §6.5's *"`ebx` holds
`[sym+0x4c]`"* is right for the earlier range, so the two do not conflict).

It has **six writers image-wide, and two of them are inside the inliner**:

```
10b624c6:  a1 10 e3 c2 10         mov    eax,ds:0x10c2e310
10b624cb:  89 45 f4               mov    DWORD PTR [ebp-0xc],eax        <- SAVE
10b624ce:  8b 86 80 00 00 00      mov    eax,DWORD PTR [esi+0x80]       <- the POGO record
10b624d4:  8b 40 76               mov    eax,DWORD PTR [eax+0x76]
10b624d7:  25 00 00 80 00         and    eax,0x800000
10b624dc:  a3 10 e3 c2 10         mov    ds:0x10c2e310,eax              <- OVERRIDE
...
10b62557:  8b 45 f4               mov    eax,DWORD PTR [ebp-0xc]
10b6255a:  a3 10 e3 c2 10         mov    ds:0x10c2e310,eax              <- RESTORE
```

**c2 swaps its favour-speed policy to the callee's profiled preference for the
duration of the expansion, then puts it back.** Both the save/override and the
restore are guarded by `[[sym+0x80]+0xb1] >> 10 & 1`, and `[sym+0x80]` is the
POGO profile record — **zero on this workload**, so S3 is read and **dead
here**. It is recorded because it is the mechanism by which C8's *liveness*, not
just its operand, is per-callee.

> **This is the sharpest thing this lane has against `P_INLINE` §6.6.** §6.6.1
> closes: *"the fit is not replaceable by any read confined to
> `0x10b5b86d`–`0x10b62b00`"*, on the grounds that both missing links are
> outside the band. **Link 1 as described does not exist**, and what stands in
> its place — the selection of C8's operand and the liveness of C8's test — is
> **two-thirds in-band and was unread when §6.6 was written.** The *conclusion*
> that the fit is not replaceable still holds, for the unit reason; **the reason
> §6.6 gives for it is wrong.**

---

## 3. The tested value is NOT the `.gl` `SIZE` — and this is a RE-READ, not a probe `[O]`

Read-before-probe: `w-sizebracket`'s cells are **already measured and
committed** (`work/w-sizebracket/series.jsonl`), so the question is answered by
re-reading them. **Nothing was recompiled by this lane.**
[`work/w-lowerband/ceiling_check.py`](../../work/w-lowerband/ceiling_check.py) ·
[`ceiling_check.out`](../../work/w-lowerband/ceiling_check.out). 168 unique
tags, five families, two profiles.

Grading `.gl SIZE < 128` — the ceiling this lane re-derived from the image
(§4) — against the recorded verdict:

| profile | `SIZE < 128` **and inlined** | `SIZE < 128` but **KEPT** | `SIZE ≥ 128` and kept | `SIZE ≥ 128` but **INLINED** |
|---|---:|---:|---:|---:|
| `/O1` (82 cells) | 49 | **8** | 17 | **8** |
| `/Ox` (86 cells) | 55 | **2** | 21 | **8** |

**Sixteen counterexamples at `/O1`, in both directions.** The rule is not a
one-sided bound either — it fails as a sufficient condition *and* as a necessary
one:

* **kept below the ceiling**: `mix_007/008/009`, `fine_002..006`, all `SIZE`
  103–127, all with emitted `.text` of **116–148 B**;
* **inlined above it**: `arith_014/016/020/024` and `static_014/016/020/024`,
  `SIZE` 131–211, all with emitted `.text` of **24–28 B**.

The split is clean in the *emitted* size and absent in `SIZE`. This reproduces
§2.1c's `/O1` bracket of `(108,116]` from a different direction and extends
§2.1b's two-cell witness to sixteen.

> **§2.1b's one-sided form does NOT survive as published.** §2.1b states
> `.gl SIZE < T ⇒ inlined` *"with zero counterexamples in 105 probe cells at
> `T = 98` (`/O1`)"*. That is true at `T = 98` and this lane does not dispute
> it — `mix`'s first kept cell is `SIZE 103` and `fine`'s is `103`. **What must
> not be done is to raise `T` to the image's 128**, which is the natural move
> once §6.6.1 publishes the ceiling as 128: at `T = 128` the same rule has
> **eight** counterexamples on cells that were already on disk. The gap between
> 98 and 128 is exactly where the rule dies.

---

## 4. The ceiling, re-derived — and one qualification `w-inlfit` could not make `[R]`

```
10b5e4cc:  8b 0d 98 ea c2 10      mov    ecx,DWORD PTR ds:0x10c2ea98    ; k
10b5e4d2:  83 f9 06               cmp    ecx,0x6
10b5e4d5:  7e 0c                  jle    0x10b5e4e3
10b5e4d7:  c7 05 18 63 c4 10 ...  mov    DWORD PTR ds:0x10c46318,0x3e8  ; 1000, when k >= 7
10b5e4e3:  6a 10 / 58 / d3 e0     shl    eax,cl                         ; 0x10 << k
10b5e4e8:  a3 18 63 c4 10         mov    ds:0x10c46318,eax
```

`DAT_10c2ea98` = **3** in raw `.data`, file offset `0x12dc98`, so the ceiling is
`0x10 << 3` = **128**. `#3717` reproduced exactly.

**Two corrections to `#3717`/§6.6.1, both small and both about instruments:**

1. **`k` has THREE readers, not two.** `0x10b5da64`, **`0x10b5dacb`** and
   `0x10b5e4cc`. §6.6.1 names the first and the third; `0x10b5dacb` is a second
   read inside the same unread `FUN_10b5da2f` (573 B, `0x10b5da2f`–`0x10b5dc6c`).
   The image has exactly four references to the address in total — those three
   and the descriptor store at `0x10c29800`.
2. **`"k is never stored by any instruction"` is exact about DIRECT-ADDRESSED
   stores and silent about indirect ones.** `0x10c29800` plants `k`'s **address**
   in the `-vol#` option descriptor (`0x10c46dd0`, kind `0x2401`,
   `work/w-lowerband/optmap.out`), which is precisely a handle for a generic
   numeric-option setter to store through. So `k = 3` is the **load-time**
   value; that it is also the **run-time** value under `/O1` is a reasonable
   expectation and **is not established by a direct-store enumeration**. Since
   `DAT_10c46318` is BSS and `FUN_10b5e4cc` runs before the inliner (`#3717`),
   the ceiling's run-time value is settled only when `k`'s is.

**And a datum nobody has recorded: the favour-speed bit's IMAGE value is 1.**
`DAT_10c2e310`, raw `.data`, file offset `0x12d510`, dword `0x00000001` — and
non-zero means C8's size test is **skipped**. Its neighbours, for the same
reason: `DAT_10c2e2fc = 0`, `DAT_10c2e308 = 0`, `DAT_10c2eab0 = 0`,
`DAT_10c2eaac = 0` (all raw `.data`); `DAT_10c3de20`, `DAT_10c3dddc`,
`DAT_10c6f1c8` and `DAT_10c46318` are **BSS, zero at load**.

**But the image value is only half the story, and the other half corrects a
claim this lane nearly shipped.** `FUN_10b82338` (`0x10b82338`, 374 B) loads a
**per-function option word** `eax = [ctx+0x1c]` and writes
`DAT_10c2e310 = (eax >> 23) & 1` at `0x10b8238d`–`0x10b82392` — §2.1's
*"option-word bit 23, written at `0x10b8238d`"*, confirmed. On that branch the
image default is overwritten unconditionally, so *"the default is ON, therefore
`/O1` must be clearing it"* **does not follow** and is not claimed.

> #### What DOES follow: the bit has THREE homes, and one branch never writes the global at all `[R]`
>
> ```
> 10b82352:  89 41 76               mov    DWORD PTR [ecx+0x76],eax   ; ecx = [[ctx]+0x80]
> 10b8236b:  83 3d 20 de c3 10 02   cmp    DWORD PTR ds:0x10c3de20,0x2
> 10b82378:  74 20                  je     0x10b8239a                 ; -> the OTHER global
> 10b8237a:  39 3d ac ea c2 10      cmp    DWORD PTR ds:0x10c2eaac,edi
> 10b82380:  74 09                  je     0x10b8238b
> 10b82382:  83 3d c8 f1 c6 10 02   cmp    DWORD PTR ds:0x10c6f1c8,0x2
> 10b82389:  74 0f                  je     0x10b8239a
> 10b8238b:  8b c8 / c1 e9 17 / 23 ce
> 10b82392:  89 0d 10 e3 c2 10      mov    DWORD PTR ds:0x10c2e310,ecx  ; the global
> 10b8239a:  8b c8 / c1 e9 17 / 23 ce
> 10b823a1:  89 0d dc dd c3 10      mov    DWORD PTR ds:0x10c3dddc,ecx  ; a DIFFERENT global
> ```
>
> When `DAT_10c3de20 == 2`, or when `DAT_10c2eaac != 0 && DAT_10c6f1c8 == 2`,
> the same bit goes to **`DAT_10c3dddc`** and **`DAT_10c2e310` is never written,
> keeping its image value of `1` — i.e. C8's size test off.**
>
> And `0x10b82352` stores the whole option word into `[[…]+0x80]+0x76`, which is
> **exactly the field §2's S3 reads at `0x10b624d4` with mask `0x800000` — bit
> 23 again**. So S3 is not a profile-weight mechanism: **the charge restores the
> favour-speed bit to the CALLEE's own recorded `/Ot`-vs-`/Os` setting for the
> duration of that callee's expansion**, and puts the caller's back afterwards.
> §1 names `[sym+0x80]` as the POGO profile record; it is also where a
> function's option word is kept.
>
> **Liveness, stated rather than assumed.** S3 needs `[sym+0x80] != 0` **and**
> bit 10 of `[[sym+0x80]+0xb1]`; the `+0x76` fill at `0x10b82352` needs
> `DAT_10c2eaac != 0`, whose image value is `0`. **Read, and not exercised
> here.**

---

## 5. Where this lane STOPS, and it is the boundary the brief drew

Decision 21 and the lane brief scope this lane **off count→bytes**, *"the whole
of lowering"*. §3 walks right up to it: the separation is clean in the emitted
`.text` and absent in `SIZE`, which is a statement about the *converter*.
**This lane does not enter it.** The boundary is `0x10b5fc8a`'s left operand:
everything upstream of the `movzx` is enumerated above; what turns a count into
emitted PPC bytes is not opened, not modelled, and not priced here.

The one thing said about it: §1.4(a) — reconciling the harness's `SIZE` column
with `il-read-varint16`'s three forms — is **upstream of lowering entirely**, is
the cheapest of the three candidates, and needs no compilation.

## 6. Not pursued, named so a later lane can pre-register them

* **`FUN_10b566e9`'s `& 0x3f`** (reader 1). Six bits of a field the inliner
  reads sixteen-wide. Either the field is a union or one of the two readings is
  of something else.
* **`FUN_10b8fb47`** (reader 9) mixes the field into an **IL hash**. If the hash
  feeds COMDAT folding or a dedup key, the field has an emit consequence outside
  the inliner entirely.
* **`DAT_10c3de20`** — 389 references, 10 writers, three values, and it gates
  both S2 and two diagnostic calls in candidacy
  (`0x10b5fc60 call 0x10b9e796` with string `0x10b02588`;
  `0x10b5fc75 call 0x10b9cae6`). **Naming the switch that sets it to 2 would
  make c2 narrate its own inline decisions**, which is the direct measurement of
  the quantity this whole thread is about. It is not in the descriptor table
  `optmap.py` recovers.
* **`0x10b9bf75`'s `and eax,0xfffffffb`** — ATTR bit 2 is cleared at load.
* **`FUN_10b5da2f`** (573 B, unread) — the second consumer of `k`, and it reads
  it **twice**.
