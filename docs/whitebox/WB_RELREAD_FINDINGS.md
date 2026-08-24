# WB_RELREAD — the relation-code enum is **19 names read out of the image**, `0x10b189b8` is **reflection and not strictness**, and `#3490`'s terminal code is **UGT**

> **PROVENANCE — DISASSEMBLY-DERIVED.** Every address here is an absolute VA in
> `compilers/X360/16.00.11886.00/c2.dll`, sha256
> `c80981c015166effecc71ad8112d5577a065b2300891dfdb02b9c13787a66258`
> (**verified by this lane** against `C2_MAP_METHOD.md` §0 before the first
> read, on both copies reachable from this box). Nothing here is copied into
> `crates/` — `w-relread` ships **zero `crates/` and zero `fixtures/` bytes** —
> so no `DISCLOSURE.md` row is due. A lane that adopts any byte below owes one.

    Lane:      w-relread (characterization)
    Date:      2026-08-24
    Prereg:    WB_RELREAD_PREREG.md — frozen as this lane's FIRST commit
    Assignment: WB_RELATION_FINDINGS.md §5's three ranked follow-up reads
    Method:    READ. No probe grid, no cl.exe run, no obj compiled by this lane.
    Tool:      scripts/dump_relnames.py — sha256-fenced, watched refusing

**Marker convention, used on every claim and never blurred:**

| | |
|---|---|
| **`[R]`** | read out of the pinned image by this lane |
| **`[O]`** | measured over objs — by *another* lane, cited; this lane compiled nothing |
| **`[I]`** | an inference joining `[R]` and `[O]`. **Not a finding.** Marked so it can be attacked separately |

---

## 0. The one-paragraph answer

`WB_RELATION_FINDINGS.md` §2's relation-code assignment is **wrong on eight of
its ten codes**, and the correction was already in this tree, on the board, at
`#2207`. c2 carries its own **19-entry name array** for the enum, at
`0x10c38690`; this lane decoded it from **raw image bytes** and it reads
`0 ILLEGAL, 1 EQ, 2 NE, 3 LT, 4 GT, 5 LE, 6 GE, 7 ULT, 8 UGT, 9 ULE, 10 UGE,
11 SO, 12 NSO, 13 S, 14 NS, 15 VALL, 16 NVALL, 17 VNONE, 18 NVNONE` **[R]**.
Six independent *consumer-side* reads confirm it. `0x10b189b8` is **operand
exchange (reflection)**, not the "strictness flip" `WB_RELATION_FINDINGS.md` §1
names it — and since §2's fourth constraint *assumes* strictness to derive the
assignment, the "over-determined four ways" claim is circular in exactly the
place it went wrong. **`#3490` is SETTLED: the terminal code of `FUN_10c1ac5c`
is 8 and 8 is `UGT`** — so `#2102`'s *"ULE"* is wrong, `w-c7`'s *"unsigned LT"*
is wrong, and `#2207` / `WB_SELECT_RECONCILED.md` §8 is right. `#423`'s 36-cell
grid is **retired as a dispatch rule** and **not** as a byte prediction; §4.6
says exactly where the line falls. **And `#2207` is itself corrected**: it
decoded the *string pool*, which is missing one name, so its list is right for
codes 0–12 and wrong from 13 on.

---

## 1. The name array — `0x10c38690`, **19 entries**, null-bounded at both ends `[R]`

`dump_relnames.py` walks a pointer array and stops **on its own terms**, not on
a bound. The array resolves at exactly the VA `#2207` cites (prereg **S1a
HIT**).

```
code  arrayVA     strVA       name          code  arrayVA     strVA       name
   0  0x10c38690  0x10b197f4  ILLEGAL         10  0x10c386b8  0x10b197cc  UGE
   1  0x10c38694  0x10b197f0  EQ              11  0x10c386bc  0x10b197c8  SO
   2  0x10c38698  0x10b197ec  NE              12  0x10c386c0  0x10b197c4  NSO
   3  0x10c3869c  0x10b197e8  LT              13  0x10c386c4  0x10b12ba8  S     <=== NOT IN THE POOL
   4  0x10c386a0  0x10b197e4  GT              14  0x10c386c8  0x10b197c0  NS
   5  0x10c386a4  0x10b197e0  LE              15  0x10c386cc  0x10b197b8  VALL
   6  0x10c386a8  0x10b197dc  GE              16  0x10c386d0  0x10b197b0  NVALL
   7  0x10c386ac  0x10b197d8  ULT             17  0x10c386d4  0x10b197a8  VNONE
   8  0x10c386b0  0x10b197d4  UGT             18  0x10c386d8  0x10b197a0  NVNONE
   9  0x10c386b4  0x10b197d0  ULE
```

`0x10c3868c` (one word before) → `'vmx'` at `0x10b197fc`; `0x10c38688` and
`0x10c386dc` are both **null**, so the array is bounded at both ends and is
exactly 19 entries. **I do not name what `0x10c3868c` belongs to** — it is
above the pool and outside this read.

### 1.1 **`#2207` and `WB_SELECT_RECONCILED.md` §8 are CORRECTED at code 13, and the mechanism is exact** `[R]`

Both publish a **14-name** list ending `…11 SO, 12 NSO, 13 NS`. The image says
**13 `S`, 14 `NS`**, and continues to 18. The cause is visible in the raw pool:

```
  0x10b197b4  4c 00 00 00 56 41 4c 4c 00 00 00 00 4e 53 00 00  |L...VALL....NS..|
  0x10b197c4  4e 53 4f 00 53 4f 00 00 55 47 45 00 55 4c 45 00  |NSO.SO..UGE.ULE.|
  0x10b197d4  55 47 54 00 55 4c 54 00 47 45 00 00 4c 45 00 00  |UGT.ULT.GE..LE..|
  0x10b197e4  47 54 00 00 4c 54 00 00 4e 45 00 00 45 51 00 00  |GT..LT..NE..EQ..|
  0x10b197f4  49 4c 4c 45 47 41 4c 00                          |ILLEGAL.|
```

The pool descending from `0x10b197f4` holds **18** strings and **`S` is not one
of them** — code 13's name is a one-character string interned at
`0x10b12ba8`, far outside the pool. So decoding *the pool in address order*
yields `…, SO, NSO, NS, VALL, …` and assigns `13 = NS`: **right through code
12, off by one from 13 on, and silently short by five.** `#2207`'s phrasing —
*"the pool descending from `0x10b197f4`"* — names the artifact it actually read.

**This is why the prereg forbade me to decode from `data.tsv` or from the
findings file (M5).** The control fired: p was registered at 0.20 that the raw
decode would disagree with `#2207`, and it disagrees.

**Codes 0–12 are unaffected, and code 8 is inside that range** — so `#2207`'s
bearing on `#3490` survives its own correction intact.

### 1.2 The instrument-defect control (M1) fired on the length `[R]`

| `--entries` | ENTRIES reported | stop condition |
|---:|---:|---|
| 14 | **14** | `BOUND 14 EXHAUSTED — count is my parameter, not the image's` |
| 20 | **19** | null pointer at entry 19 (`0x10c386dc`) |
| 32 | **19** | null pointer at entry 19 |
| 64 | **19** | null pointer at entry 19 |
| 256 | **19** | null pointer at entry 19 |

At the bound a 14-name decode reproduces exactly, and **the tool says so in
those words rather than reporting 14 as a fact.** Denominator: the walk's own
stop condition. Per **`#3483`**, this proves *reproducibility*, not
*attribution* — the attribution is §2's six consumer reads.

### 1.3 **Nothing in the image references this array** `[R]` — the caveat, stated because it cuts against me

The 4-byte little-endian literal `0x10c38690` occurs **0 times** anywhere in the
image, and **0** words at any offset in any raw section (aligned or not) point
into `[0x10c38690, 0x10c386dc)`. Denominator: every byte offset of every raw
section, unaligned included.

So this is **unreferenced data** — a name table with no live consumer in this
build. That is a real reason for caution: an unreferenced table can be stale
with respect to the enum the code actually uses. **It is not stale, and §2 is
how that is known** — six consumer-side reads agree with it and with nothing
else. The names are the compiler author's own labels; the *authority* for using
them comes from the consumers, not from the table.

---

## 2. Six CONSUMER reads that confirm the enum — none of which needs the strings `[R]`

Each of these is read out of executable code and is independent of §1.

| # | site | what it shows |
|---|---|---|
| **C1** | `0x10b189cc` (negation) 2-cycles `(11 12) (13 14) (15 16) (17 18)` | the names pair `SO↔NSO`, `S↔NS`, `VALL↔NVALL`, `VNONE↔NVNONE` — **the `N` prefix is negation, on all four pairs at once.** A wrong assignment would have to reproduce four independent `N`-prefix pairings by accident |
| **C2** | `0x10c1ac0c[8]` = `0x10c1ac0c[0]` | against zero, **code 9 and code 1 share one arm**: `x <=u 0` ≡ `x == 0` — true for `ULE`, false for `w-c7`'s `9 = UGE` |
| **C3** | `0x10c1ac0c[7]` = `0x10c1ac0c[1]` | against zero, **code 8 and code 2 share one arm**: `x >u 0` ≡ `x != 0` — true for `UGT` with **no operand exchange**. Under `w-c7`'s `8 = unsigned LT` this arm asserts `x != 0 ≡ x <u 0`, which is **constant false** |
| **C4** | `0x10c1aa91` `push 0x0` | against zero, **code 7 is folded to constant FALSE** — true for `ULT` (`x <u 0`), impossible for `w-c7`'s `7 = ULE` (`x <=u 0` is `x == 0`, not a constant) |
| **C5** | `0x10c1aa95` `push [ebp-0x10]` | against zero, **code 10 is folded to constant TRUE** — true for `UGE`, impossible for `w-c7`'s `10 = UGT` |
| **C6** | `0x10c1ac34[2..9]` | in the **general** block, codes `(3 4)`, `(5 6)`, `(7 8)`, `(9 10)` each share **one emitter** and differ only in which operand slot is swapped — i.e. each pair is an **operand exchange**. `(LT,GT)`, `(LE,GE)`, `(ULT,UGT)`, `(ULE,UGE)` are mirror pairs; `w-c7`'s `(LE,LT)` and `(GE,GT)` are **not** |

### 2.1 A seventh confirmation, and it is `[O]` — the port's own byte-graded table

`crates/c2-core/src/codegen/leaf/compare.rs` carries a mandatory `k == 0` table,
byte-graded green against real c2 (`w-c7` §5.1, `w9_cmp_zero_le.cpp`):

```text
  (Rel::Lt, unsigned)  a <  0  ->  constant false        [O]
  (Rel::Ge, unsigned)  a >= 0  ->  constant true         [O]
  (Rel::Le, unsigned)  a <= 0  ->  exactly a == 0        [O]
  (Rel::Gt, unsigned)  a >  0  ->  exactly a != 0        [O]
```

Set beside C2–C5 **[R]**: constant-false is code **7**, constant-true is code
**10**, `≡ ==` is code **9**, `≡ !=` is code **8**. So `Rel::Lt`↔7 = `ULT`,
`Rel::Ge`↔10 = `UGE`, `Rel::Le`↔9 = `ULE`, `Rel::Gt`↔8 = `UGT` **[I]**.

**This is the strongest class of evidence this directory recognises** —
`C2_MAP_METHOD.md` §7: *"a white-box finding confirmed by an independent route
is in a different and much stronger category."* The route here is a black-box
fixture table that was derived with no access to the image at all, and it
lands on the same four codes.

**It also refutes `WB_RELATION_FINDINGS.md` §2 using the project's own oracle
output**: unsigned `LT` is code **7**, so `w-c7`'s *"terminal code 8 is
unsigned LT"* cannot stand.

---

## 3. `0x10b189b8` is **REFLECTION**, not "strictness" — and that is where §2 went wrong `[R]`

Read out of the image, all three tables, with their algebra computed rather than
asserted:

```text
  a4  0x10b189a4  00 01 02 07 08 09 0a 07 08 09 0a 00 00 00 00 0f 10 11 12 00
  b8  0x10b189b8  00 01 02 04 03 06 05 08 07 0a 09 00 00 00 00 0f 10 11 12 00
  cc  0x10b189cc  00 02 01 06 05 04 03 0a 09 08 07 0c 0b 0e 0d 10 0f 12 11 00
```

| table | 2-cycles | fixed points | under the image names |
|---|---|---|---|
| `a4` | none (not an involution) | 0,1,2,7,8,9,10,15,16,17,18 | `LT→ULT, GT→UGT, LE→ULE, GE→UGE`; `SO,NSO,S,NS → ILLEGAL` — **signedness** |
| `b8` | `(3 4) (5 6) (7 8) (9 10)` | 0,1,2,15,16,17,18 | `LT↔GT, LE↔GE, ULT↔UGT, ULE↔UGE`; EQ,NE fixed — **operand exchange** |
| `cc` | `(1 2)(3 6)(4 5)(7 10)(8 9)(11 12)(13 14)(15 16)(17 18)` | 0 | `EQ↔NE, LT↔GE, GT↔LE, ULT↔UGE, UGT↔ULE, …` — **negation** |

**`b8` swaps direction, not strictness.** `swap(a < b) = b > a` fixes `EQ` and
`NE` — and so does a strictness flip, which is exactly why the tables alone
cannot tell them apart (prereg **S3c**, registered before looking). Two
independent things settle it: the names (§1) and **C6**, where the general
block implements each `b8` pair as *the same emitter with the operands
exchanged* — the mechanism, not the label.

### 3.1 The circularity, named precisely

`WB_RELATION_FINDINGS.md` §2 says the assignment is *"over-determined, which is
why it can be stated without a probe"*, on four constraints. Constraints 1–3
(`a4` fixes `{1,2}`; `cc` pairs `(1 2)`; `cc` pairs `(3 6)` and `(4 5)`) are
sound and are satisfied by **both** candidate assignments. Constraint 4 —
*"`b8` pairs `(3 4)` and `(5 6)` — a **strictness** flip within one direction —
which fixes the assignment the rest of the way"* — is the only one that
discriminates, and it discriminates by **assuming what `b8` is**. The name
"strictness flip" appears in §1's table as a finding and is used in §2 as a
premise. **Over-determined three ways, and the fourth way is the answer restated
(prereg S3b HIT).**

This is a general hazard for table-algebra reads and it deserves the sharper
statement: *the algebra of a permutation table constrains the assignment only up
to the automorphisms of that algebra.* Here the relation lattice has an
order-2 automorphism (exchange ↔ strictness on `{3,4,5,6}`) that no amount of
table-reading breaks. **A consumer or a name is required, and this lane needed
both to be sure.**

### 3.2 A consequence: `code = IL opcode − 0x1E` is **FALSE** `[I]`

`crates/c2-il/src/func/mod.rs:1411-1416` reads `0x1F => Eq, 0x20 => Ne,
0x21 => Le, 0x22 => Lt, 0x23 => Ge, 0x24 => Gt` **[O]** — the port's IL model,
graded byte-exact against real c2. Set beside §1 **[R]**, the map is a
**permutation, not a subtraction**:

| IL opcode | port `Rel` | c2 relation code |
|---|---|---|
| `0x1F` | `Eq` | **1** `EQ` |
| `0x20` | `Ne` | **2** `NE` |
| `0x21` | `Le` | **5** `LE` |
| `0x22` | `Lt` | **3** `LT` |
| `0x23` | `Ge` | **6** `GE` |
| `0x24` | `Gt` | **4** `GT` |

`w-c7`'s §2 **title** — *"the relation code, recovered — and it is **the IL
opcode minus `0x1E`**"* — is wrong on codes 3–6 and, through `unsigned = signed
+ 4`, on 7–10 as well. Its own prereg **W2** already scored a MISS for
recovering a value instead of a location; **the value was also wrong**, and
nothing in that lane could have caught it, because the value is what its
constraint 4 was built to produce.

### 3.3 **REFUSED: I did not find the opcode → code site either** `[R]`

Prereg **S2c**, registered at p = 0.40 — the same target `w-c7` missed.
Searched: the byte pattern `01 02 05 03 06 04` (a table indexed by
`opcode − 0x1F`) — **0 hits**; the inverse `1f 20 22 24 21 23` — **0 hits**.
Denominator: every byte offset in the 1 347 072-byte image.

**So there is no contiguous byte table performing this permutation, and I am not
naming a site.** `w-c7`'s W2 stands as an open miss, now with two lanes' worth
of negative evidence and one search space eliminated. Prereg **S2b** (*"the
site is a byte table"*) is **REFUTED** for the contiguous-table form; a
per-opcode `mov` inside a decode switch remains open and is a ranked follow-up
(§6).

---

## 4. `FUN_10c1a908` — the ten arms, read from the JUMP TABLES `[R]`

The assignment's first follow-on. **Ghidra's rendering of this function is not
usable** — it emits `WARNING (jumptable): Sanity check requires truncation of
jumptable` and `Could not find normalized switch variable`, and its case labels
are consequently wrong. Everything below is read from the raw tables and the
objdump.

### 4.1 The dispatch is unguarded and the tables are exactly 10 entries

```
10c1aa15:  0f b6 45 ff        movzx eax, BYTE PTR [ebp-0x1]     ; the relation code
10c1aa19:  48                 dec   eax
10c1aa1a:  ff 24 85 0c ac c1 10  jmp DWORD PTR [eax*4+0x10c1ac0c]   ; T1, against zero
...
10c1aaa7:  0f b6 45 ff        movzx eax, BYTE PTR [ebp-0x1]
10c1aaab:  48                 dec   eax
10c1aaac:  ff 24 85 34 ac c1 10  jmp DWORD PTR [eax*4+0x10c1ac34]   ; T2, general
```

* **`T1` @ `0x10c1ac0c`** and **`T2` @ `0x10c1ac34`** — 40 bytes apart, so `T1`
  is exactly **10 entries**; `T2` ends at `0x10c1ac34 + 40 = 0x10c1ac5c`, which
  is **the start of `FUN_10c1ac5c`**, so `T2` is exactly **10 entries** too.
  Both are indexed by **`code − 1`**, covering codes **1..10** and nothing else.
* **There is no `cmp`/`ja` bound check** before either jump. Codes `0` and
  `11..18` are **not handled** — code 0 indexes one dword *before* `T1`.
  Ghidra renders that as `case 0: halt_baddata()`; it is not an arm, it is an
  out-of-range read. **The guarantee that only codes 1–10 arrive is upstream and
  this lane did not read it.**
* **There is no `default` arm.** `WB_RELATION_FINDINGS.md` §5 names
  `FUN_10c198d2`/`FUN_10c19bc0` as *"(default)"* — **it is `T1[0]`, i.e. code 1
  `EQ`**, shared with code 9 `ULE`. `w-c7` copied Ghidra's mislabel.

### 4.2 `T1` — the **against-zero** block (`0x10c1ac0c`)

Reached when the second compare operand is the constant 0.

| code | name | target | what it is |
|---:|---|---|---|
| 1 | `EQ` | `0x10c1aa21` | `FUN_10c198d2` / `FUN_10c19bc0` |
| 2 | `NE` | `0x10c1aa3c` | `FUN_10c19936` / `FUN_10c19c87` |
| 3 | `LT` | `0x10c1aa4d` | `FUN_10c199bc` / `FUN_10c19d50` |
| 4 | `GT` | `0x10c1aa5e` | `FUN_10c19a07` / `FUN_10c19da9` |
| 5 | `LE` | `0x10c1aa6f` | `FUN_10c19a7f` / `FUN_10c19e9a` |
| 6 | `GE` | `0x10c1aa80` | `FUN_10c19af9` / `FUN_10c19f69` |
| 7 | `ULT` | `0x10c1aa91` | **`push 0x0`** → the materialiser: **constant FALSE** |
| 8 | `UGT` | `0x10c1aa3c` | **= code 2's target** (`x >u 0` ≡ `x != 0`) |
| 9 | `ULE` | `0x10c1aa21` | **= code 1's target** (`x <=u 0` ≡ `x == 0`) |
| 10 | `UGE` | `0x10c1aa95` | **`push [ebp-0x10]`** → the materialiser: **constant TRUE** |

**Ten entries, eight distinct targets, six emitter pairs and two constant
folds.** Prereg **S4g** (*"exactly ten arms"*) is a **PARTIAL**: ten is the
*table length*; the arm count is eight.

### 4.3 `T2` — the **general** block (`0x10c1ac34`), and it is `b8` made executable

| code | name | target | emitter (`+1` / else) | operand slot replaced |
|---:|---|---|---|---|
| 1 | `EQ` | `0x10c1aab3` | `FUN_10c19fdb` / `FUN_10c1a494` | `puVar8` |
| 2 | `NE` | `0x10c1aacb` | `FUN_10c1a038` / `FUN_10c1a4f1` | `puVar8` |
| 3 | `LT` | `0x10c1aadc` | `FUN_10c1a1bb` / `FUN_10c1a677` | `puVar8` |
| 4 | `GT` | `0x10c1aaed` | **same pair as 3** | `puVar7` ← **exchanged** |
| 5 | `LE` | `0x10c1aafe` | `FUN_10c1a0ab` / `FUN_10c1a564` | `puVar8` |
| 6 | `GE` | `0x10c1ab0f` | **same pair as 5** | `puVar7` ← **exchanged** |
| 7 | `ULT` | `0x10c1ab20` | `FUN_10c1a396` / `FUN_10c1a838` | `puVar8` |
| 8 | `UGT` | `0x10c1ab31` | **same pair as 7** | `puVar7` ← **exchanged** |
| 9 | `ULE` | `0x10c1ab42` | `FUN_10c1a2d8` / `FUN_10c1a79d` | `puVar8` |
| 10 | `UGE` | `0x10c1ab5a` | **same pair as 9** | `puVar7` ← **exchanged** |

then `uVar5 = (*pcVar4)(puVar7, puVar8);`.

**Ten codes, six distinct emitter pairs.** `EQ` and `NE` are symmetric and get
their own; the four ordering pairs are `b8`'s four 2-cycles, each realised as
*one emitter called with the operands the other way round*. This is **C6**, and
it is the cleanest possible statement of what `0x10b189b8` means.

### 4.4 The within-pair flag — **it is the value of the TRUE operand, `+1` vs `−1`** `[R]`

`WB_RELATION_FINDINGS.md` §5: *"the pair being selected by a flag this lane did
not identify."* It is `edx`:

```
10c1a99f:  8a 48 08     mov cl, BYTE PTR [eax+0x8]     ; operand A's kind
10c1a9a5:  83 ca ff     or  edx, 0xffffffff            ; edx := -1   (the default)
10c1a9ab:  89 55 f0     mov DWORD PTR [ebp-0x10], edx
10c1a9ae:  80 f9 07     cmp cl, 0x7                    ; kind 7 = constant?
10c1a9b1:  75 25        jne 0x10c1a9d8                 ;   no  -> materialise, edx stays -1
10c1a9b3:  83 78 18 01  cmp DWORD PTR [eax+0x18], 0x1  ; value64 == 1 ?
10c1a9b7:  75 0e        jne 0x10c1a9c7
10c1a9b9:  83 78 1c 00  cmp DWORD PTR [eax+0x1c], 0x0
10c1a9bd:  75 08        jne 0x10c1a9c7
10c1a9bf:  33 d2        xor edx, edx
10c1a9c1:  42           inc edx                        ; edx := +1
...
10c1a9c7:  cmp cl,0x7 / cmp [eax+0x18],-1 / cmp [eax+0x1c],-1 / je 0x10c1a9f4   ; value64 == -1
```

and every pair selects with `cmp edx,0x1 ; je`.

So the flag is: **is the value to produce when the relation is TRUE the constant
`+1`?** `+1` takes the first emitter; `−1` **and every non-constant value** take
the second. `edx` is *also* the value pushed by `T1[9]`'s constant-TRUE arm, so
the same word is both the selector and the payload.

`FUN_10c1a908` is therefore the lowering of **`rel ? A : 0`** — a select, not a
bare compare — with a fast path when `A ∈ {+1, −1}` and a materialisation
(`FUN_10bd42ff(0x1004)` + `FUN_10c19859`) otherwise.

**Prereg S4c (width), S4d (value-vs-branch), S4e (polarity) are all MISSES;
S4f ("none of the above") is the HIT.** The calibration note: I registered a
distribution over three named mechanisms and the truth was a fourth. It is not
an anti-tidy miss (`w-r8idiom`'s pattern) and not an artefact-existence miss
(`w-2e4`'s) — **it is a miss of category: I predicted a property of the
INSTRUCTION and it was a property of the OPERAND.**

**`[I]`, and marked as such because I did not read the emitter bodies:** the
`−1` arm most likely produces a 0/−1 **mask** — which is why `−1` and
"arbitrary value" can share one arm, since a mask composes with any payload by
`and`. **I refuse to publish that as a finding.** The emitters at
`FUN_10c19bc0`, `FUN_10c1a494` etc. are unread.

### 4.5 The normalisation preamble — `w-c7`'s three addresses all VERIFY `[R]`

| VA | instruction | guard |
|---|---|---|
| `0x10c1a947` | `mov al, BYTE PTR [eax+0x10b189a4]` | iff `(*(u16*)(type+0xa) & 0xf000) == 0x2000` |
| `0x10c1a96d` | `mov al, BYTE PTR [eax+0x10b189cc]` | iff operand **A** is kind 7 with `value64 == 0` |
| `0x10c1a98f` | `mov cl, BYTE PTR [ecx+0x10b189cc]` | iff operand **B** is kind 7 with `value64 == 0`; **also exchanges** |

`WB_RELATION_FINDINGS.md` §3.1/§3.2's mechanism is **correct and verified** —
only its *names for the codes* were wrong. The relation code is the byte at
**`[node+0x34]`** (`mov cl, BYTE PTR [edi+0x34]` @ `0x10c1a91c`), and here it is
used **unmasked** — no `& 0x1f`. §3.3's *"the relation code is the low 5 bits of
a byte that carries other flags above it"* is read off `FUN_10bd50b7`, which
operates on a **different record** (`[param_1+0xa]`). Two record layouts; the
claim should not be carried across. **Not resolved by this lane.**

### 4.6 **Does this retire `#423`'s 36-cell grid?** — yes as a *rule*, no as a *byte prediction*

`#423`'s grid is 6 relations × {signed, unsigned} × `k ∈ {0,1,2}`. After §4 the
**dispatch** for all 36 cells is a stated rule:

1. if the operand type-class nibble is `0x2000`, `code := a4[code]` — so
   unsigned `LT,GT,LE,GE` become `ULT,UGT,ULE,UGE` (**3,4,5,6 → 7,8,9,10**);
2. for each operand that is the constant 0, `code := cc[code]`, and on the
   second also exchange;
3. `k == 0` → dispatch `T1[code−1]`; `k ≠ 0` → dispatch `T2[code−1]`.

and the four cells `#423` found special are exactly `T1`'s four degenerate
unsigned entries: `7 → false`, `10 → true`, `9 ≡ EQ`, `8 ≡ NE`. **`#423`'s
"three-way interaction of (relation, signedness, literal)" is fully explained,
and it is not three-way — it is `a4` composed with a `k == 0` table selection.**

**What the read does NOT give**: the *bytes* each emitter produces.
`FUN_10c199bc`, `FUN_10c19a07`, `FUN_10c19a7f`, `FUN_10c19af9`, `FUN_10c19936`,
`FUN_10c198d2` and their eight `−1`-arm siblings are **unread**. A grid that
was measuring *which cells are special* is retired; a grid measuring *emitted
size and relocations per cell* is not.

**Prereg S4h was registered at p = 0.45 for exactly this reason and scores
PARTIAL.** `WB_RELATION_FINDINGS.md` §5's *"would retire `#423`'s grid
entirely"* is **too strong** — the word is "the grid's question", not "the
grid".

---

## 5. `#3490` — **SETTLED. The terminal code is 8 and 8 is `UGT`** `[R]`

The second follow-on. `FUN_10c1ac5c` @ `0x10c1ac5c` normalises by a
**re-entrant `dec`/`je` chain**, not a table:

```
10c1acad:  mov dl,[ecx+0x10b189a4]      ; signedness remap, iff the type nibble is 0x2000
10c1acb3:  movzx ecx,dl                 ; <== DISPATCH HEAD, re-entered after every rewrite
10c1acb6:  dec ecx / je 0x10c1ad0c      ; code 1  EQ
10c1acb9:  dec ecx / je 0x10c1aced      ; code 2  NE
10c1acbc:  sub ecx,5 / je 0x10c1acde    ; code 7  ULT
10c1acc1:  dec ecx / je 0x10c1ad10      ; code 8  UGT   <== TERMINAL
10c1acc4:  dec ecx / je 0x10c1acd2      ; code 9  ULE
10c1acc7:  dec ecx / jne 0x10c1af23     ; code 10 UGE, else REJECT
10c1acce:  mov dl,0x9 ; jmp 0x10c1ace0
```

| from | at | rewrite | which swap | table it matches |
|---|---|---|---|---|
| 1 `EQ` | `0x10c1ad0c` | `dl := 2` (`NE`) | **result values** (`0x10c1acd4`) | `cc[1] = 2` — negation |
| 2 `NE` | `0x10c1aced` | `dl := 8` (`UGT`) | none; **requires the other operand be kind 7 with `value64 == 0`, else REJECT** | the identity `x != 0` ≡ `x >u 0` |
| 7 `ULT` | `0x10c1acde` | `dl := 8` (`UGT`) | **compare operands** (`0x10c1ace0`) | `b8[7] = 8` — reflection |
| 8 `UGT` | `0x10c1ad10` | — | — | **TERMINAL: the emit path** |
| 9 `ULE` | `0x10c1acd2` | `dl := 8` (`UGT`) | **result values** (`0x10c1acd4`) | `cc[9] = 8` — negation |
| 10 `UGE` | `0x10c1acce` | `dl := 9` (`ULE`) | **compare operands** (`0x10c1ace0`) | `b8[10] = 9` — reflection |
| 0, 3, 4, 5, 6, 11–18 | `0x10c1af23` | — | — | `mov eax,0x1f4 ; ret` — **500, "impossible"** |

**The two swaps are at two different addresses and touch two different variable
pairs**, and that is what makes reflection and negation separable *inside a
single function*:

* `0x10c1ace0` exchanges `[ebp-0x4]` ↔ `[ebp-0x8]` — the **compare operands**;
  every use of it accompanies a `b8` pair. **Reflection.**
* `0x10c1acd4` exchanges `eax` ↔ `[ebp-0x10]` — the **result values**; every use
  of it accompanies a `cc` pair. **Negation.**

### 5.1 The verdict on the three published readings

| reading | claim about code 8 | verdict |
|---|---|---|
| **`#2102`** / `WB_SELECT_FINDINGS_R2.md` `W-SELECT-3` | *"normalises every unsigned relation to **ULE**"* | **WRONG on the name.** The terminal is `UGT`. `ULE` (code 9) is a *source* the chain rewrites **away from**, at `0x10c1acd2` |
| **`w-c7`** / `WB_RELATION_FINDINGS.md` §2 | terminal code 8 is *"unsigned **LT**"* | **WRONG.** Unsigned LT is code **7**, and `0x10c1acde` is the arm that converts it away by exchanging the operands |
| **`#2207`** / `WB_SELECT_RECONCILED.md` §8 | code 8 is **`UGT`**; *"R2's row `9 UGT` swap, code := 8 should read `9 ULE` **swap the RESULT values**, code := 8"* | **RIGHT, word for word**, and independently re-derived here from the objdump without re-reading their text |

**`#3490` is closed.** It should also be recorded that `#3490` was *stated
against the wrong pair*: it framed the disagreement as `#2102` vs `w-c7` when a
third, correct reading was already on the board at `#2207`, filed by a
reconciliation lane whose whole purpose was to settle this.

### 5.2 Two corrections to `#2102`'s *scope*, beyond the name `[R]`

* *"every unsigned relation"* is right about `{7,8,9,10}` and **silent about
  `EQ`/`NE`**, which the chain also accepts — but **only against a constant
  zero** (`0x10c1aced`'s guard). `x == 0` and `x != 0` are in the carry
  expander's domain; `x == 5` is not.
* `W-SELECT-3`'s *"condition codes 3–6 return impossible"* **verifies exactly**:
  `0x10c1af23` is `mov eax,0x1f4 ; ret`, and `0x1f4 = 500`. Since `a4` maps
  `3,4,5,6 → 7,8,9,10` for unsigned types, codes 3–6 reach the chain **only**
  when the compare is signed. The expander is unsigned-only, as published.

---

## 6. Codes 11–18 — named, and their algebra read; **the meanings REFUSED** `[R]`

The third follow-on. `WB_RELATION_FINDINGS.md` §2 calls 11–18 *"eight further
relations, negation-paired `(11 12)(13 14)(15 16)(17 18)`, left **fixed** by both
`a4` and `b8` — the FP ordered/unordered set"*.

* The **pairing is right** `[R]` — and §1 explains it: `SO↔NSO`, `S↔NS`,
  `VALL↔NVALL`, `VNONE↔NVNONE`, an `N`-prefix on all four.
* *"left fixed by both `a4` and `b8`"* is **wrong for 11–14** `[R]`.
  `a4[11..14] = 00` and `b8[11..14] = 00`, and `00` is **`ILLEGAL`**, not a
  fixed point. So: **asking for the unsigned form, or the mirror, of `SO`,
  `NSO`, `S` or `NS` is an error the tables encode explicitly.** Only
  `15..18` are true fixed points of both (`0f 10 11 12` = identity) —
  `VALL`/`NVALL`/`VNONE`/`NVNONE` are invariant under signedness **and** under
  operand exchange. Prereg **S6b HIT**.
* *"the FP ordered/unordered set"* is **not a name I can confirm and not one I
  will repeat.** `SO` is the PowerPC condition-register bit's conventional
  spelling and on an FP compare that bit is the unordered result, which makes
  `SO`/`NSO` a plausible ordered/unordered pair — **that is an `[I]`, and I am
  not publishing it as a finding.** For `S`/`NS` I have **no reading at all**,
  only the string. For `VALL`/`NVALL`/`VNONE`/`NVNONE` the shape ("all" vs
  "none") and the `'vmx'` string sitting immediately above the pool at
  `0x10b197fc` both point somewhere, and **pointing is not reading.**

**REFUSED, deliberately, per `w-r8idiom`/`w-2e4`:** I will not name what `S`,
`NS`, `VALL`, `NVALL`, `VNONE` or `NVNONE` *mean*. What is published is the
strings, their addresses, their negation pairs, and the fact that four of them
are `ILLEGAL` under both remaps. Prereg **S6c** (*"15–18 are floating-point"*)
is scored a **MISS** — the names do not read as FP — and **S6d** (*"I can name
14–18 from a string"*) is a **HIT** on the strings only.

Neither table is consulted for 11–18 by `FUN_10c1a908` (§4.1: the jump tables
are 10 entries) or by `FUN_10c1ac5c` (§5: they reject to 500). **Every consumer
this lane read handles codes 1–10 only.**

---

## 7. Instrument defects found, all by running the registered controls

| # | control | what it found |
|---|---|---|
| **D1** | M3 — watch the fence refuse | the fence refuses a **truncated** image and a **one-bit-flipped** image of identical size (exit 3, "Nothing was read"). It **crashed with an unhandled `FileNotFoundError` traceback** on an unreadable path. Fixed; re-watched. Prereg registered p = 0.25 that my first fence would be wrong — **it was** |
| **D2** | M1 — vary a parameter it must not depend on | `--entries 14` reports **14** and says *"BOUND EXHAUSTED — count is my parameter"*; 20/32/64/256 all report **19**, stopping on a null. A 14-name answer is reproducible and wrong |
| **D3** | M1 — two independent sources | `xrefs.tsv` and `objdump` **disagree**: `0x10b189cc` is **31** vs **33** sites, `0x10b189b8` is **10** vs **12**. Ghidra misses `0x10b4e101` and `0x10b4e111`, which objdump shows as plain `mov dl,BYTE PTR [eax+0x10b189cc]`. **`WB_RELATION_FINDINGS.md`'s "31 xrefs" is Ghidra's under-count; the number is 33.** Its *"from 26 functions"* reproduces as **23** from both sources — I cannot reconstruct 26 |
| **D4** | M2 — traversal invariance | **the control FAILED on the first run** — chunking the same VA window in 1/3/7 pieces gave **616 / 617 / 625** lines, because arbitrary chunk boundaries land mid-instruction and objdump resynchronises differently. Re-run with boundaries **taken from the single-pass instruction-start set**: **616 lines, identical sha256, all three.** `w-2e4`'s invariance control is sound *only for boundary-aligned chunking*, and that qualification was not in the write-up |
| **D5** | M5 — decode from raw bytes, not from the prior lane's artifact | `#2207`'s list is **wrong at code 13 and short by five**, because it decoded the string pool and the pool is missing `S` (§1.1) |

**Corrected counts, with denominators.** Denominator for all three: every
instruction in `objdump_intel.asm` over the whole 1 347 072-byte image;
containing function attributed by address range from `functions.tsv`.

| table | sites | distinct containing functions |
|---|---:|---:|
| `0x10b189a4` signedness | **6** | **5** |
| `0x10b189b8` reflection | **12** | **11** |
| `0x10b189cc` negation | **33** | **23** |

---

## 8. What this is worth, and what it is not

**Worth.** The enum is now **read**, not derived, and confirmed by six
consumers plus one independent black-box route. Every relational decision in
`FUN_10c1a908` is a two-entry table lookup and a ten-way indexed jump, both
tables read out at their own addresses. `#3490` is closed. `#423`'s grid is
retired as a dispatch question. Three published documents are corrected — and
so is the board row that was supposed to be authoritative.

**Not worth.** Predicted reach was **0** and delivered **0**. Nothing here
converts a TU, moves the census, or changes a byte of the port. The enum's
*names* are cosmetic to the port: `w-c7` §3.1 is right that `#2109` closed the
code lane's need for an address. **What was not cosmetic is that a wrong
assignment was being used to reason about mechanism**, and mechanism is goal (1).

**Also not.** Nothing is adopted; `crates/` and `fixtures/` are byte-identical
to base. `DISCLOSURE.md` is unchanged **on purpose**. A lane that bakes the
enum, `a4`, `b8` or `cc` into a port table owes a row per table — **four now,
not three**, because the name array at `0x10c38690` is a fourth adoptable
artifact.

**A methodological result worth more than any address here.** Two lanes read
the same three tables carefully and correctly and reached opposite assignments,
because the algebra of a permutation table determines the labelling only up to
the automorphisms of that algebra (§3.1). *"Over-determined by N independent
constraints"* is worth checking constraint by constraint for one that assumes
the conclusion — here, one of four did, and it was the only one that
discriminated.

---

## 9. Pre-registration score — **17 hits, 8 misses, 4 partials**

| | registered | measured | |
|---|---|---|---|
| **S1a** | the array resolves at exactly `0x10c38690` (p 0.65) | it does; `.data`, null-bounded both ends | **HIT** |
| **S1b** | names = `#2207`'s 14 (derived, p 0.80) | right 0–12, **wrong at 13**, and 19 entries not 14 | **MISS** — and the most useful one |
| **S1c** | longer than 14 entries; names 14–18 (p 0.55) | **19 entries**, 14–18 named | **HIT** |
| **S1d** | a second independent naming array (p 0.30) | none; and **nothing references even this one** | **MISS** |
| **S2a** | `code = IL opcode − 0x1E` is FALSE (p 0.75) | it is a permutation (§3.2) | **HIT** |
| **S2b** | the mapping site is a byte table (p 0.55) | **0 hits** for either byte pattern over the whole image | **REFUTED** for the contiguous form |
| **S2c** | I name the site with a VA (p 0.40) | **I did not.** Registered below `w-c7`'s 0.5 and still missed | **MISS** |
| **S3a** | `b8` is reflection, not strictness (p 0.75) | names **and** C6 | **HIT** |
| **S3b** | §2 is over-determined ≤ 3 ways (p 0.70) | constraint 4 assumes the answer | **HIT** |
| **S3c** | the tables alone cannot distinguish (p 0.85) | registered before looking; exactly why `w-c7` erred | **HIT** |
| **S4a** | the six emitter-pair VAs resolve (p 0.60) | all six resolve — but one is **mislabelled** "(default)" | **HIT**, with a correction |
| **S4b** | the switch selector is the normalized code (p 0.80) | `[ebp-0x1]`, post-`a4`/`cc` | **HIT** |
| **S4c** | the pair flag is operand **width** (p 0.35) | no | **MISS** |
| **S4d** | the pair flag is **value vs branch** (p 0.30) | no | **MISS** |
| **S4e** | the pair flag is **polarity** (p 0.20) | no | **MISS** |
| **S4f** | none of the above (p 0.15) | **the TRUE-value operand being `+1` vs `−1`** | **HIT** |
| **S4g** | exactly ten arms (p 0.55) | ten *entries*, **eight** distinct targets | **PARTIAL** |
| **S4h** | the read retires `#423`'s grid (p 0.45) | retires the **dispatch question**, not the byte prediction (§4.6) | **PARTIAL** |
| **S5a** | the terminal code is 8 (p 0.75) | `0x10c1acc1 / je 0x10c1ad10` | **HIT** |
| **S5b** | 8 is `UGT`; both `#2102` and `w-c7` wrong (derived, p 0.80) | confirmed from the objdump | **HIT** (derived) |
| **S5c** | `#3490` ends SETTLED with a named arm VA (p 0.60) | `0x10c1ad10`, terminal | **HIT** |
| **S5d** | the normalisation is a swap **plus a table lookup** (p 0.65) | **two different swaps** and **no table** — a `dec`/`je` chain with literal `mov dl,N` | **PARTIAL** — the swap half right, the table half wrong |
| **S6a** | 11/12 are overflow, not "FP ordered/unordered" (p 0.70) | `SO`/`NSO` | **HIT** |
| **S6b** | §2's "fixed by both" is wrong for 11–14 (p 0.85) | they map to `ILLEGAL` | **HIT** |
| **S6c** | 15–18 are floating-point (p 0.55) | `VALL`/`NVALL`/`VNONE`/`NVNONE` — not FP | **MISS**, and I refuse to name them |
| **S6d** | I can name 14–18 from a string (p 0.45) | all five | **HIT** (strings only) |
| **M1** | at least one count moves under a parameter it must not depend on (p 0.40) | **three did** (D2, D3) | **HIT** |
| **M2** | traversal invariance | **failed first, passed boundary-aligned** (D4) | **FIRED** |
| **M3** | my first fence is wrong (p 0.25) | it was (D1) | **HIT** |
| **M5** | the raw decode disagrees with `#2207` (p 0.20) | it does, at code 13 (D5) | **HIT** |
| **M6** | I refuse at least one name (p 0.6) | `S`, `NS`, the `vmx` slot's array, the `−1` emitter's mask | **HIT** |

**The misses that matter.** **S1b** is the best one: I registered `#2207`'s list
at p = 0.80 as *derived*, and the derived part is exactly what broke — the
prior lane read a **pool** and I read an **array**, and those differ by one
interned string. **S2c** is the second: I registered a location prediction at a
*lower* credence than the lane that had already missed it, and still missed it;
the search eliminated one hypothesis and named no address. **S4c/d/e** are a
category miss — three guesses about the instruction when the flag was a
property of the operand. And **S6c** I would rather have missed than hit,
because hitting it would have meant repeating "FP" without a read behind it.

---

## 10. Ranked follow-ons

1. **The IL-opcode → relation-code site is STILL UNNAMED** (`w-c7`'s W2, now
   missed twice). This lane eliminated the contiguous-byte-table hypothesis
   over the whole image (§3.3). The remaining shape is a per-opcode literal
   inside the IL decode switch — find the reader of the IL opcode stream and
   read the `0x1F..0x24` arms. **~½ day**, and it would close a two-lane miss.
2. **The six against-zero emitters and their eight `−1` siblings** (§4.2/§4.4),
   which is what a *byte-level* retirement of `#423` needs. `FUN_10c19936`
   (`!= 0`) and `FUN_10c198d2` (`== 0`) are the two highest-value, since they
   serve four of the ten codes between them. **~1 day.**
3. **The `[node+0x34]` vs `[node+0xa]` record split** (§4.5) — `FUN_10c1a908`
   uses the relation byte unmasked, `FUN_10bd50b7` masks it to 5 bits. Two
   layouts, and `WB_RELATION_FINDINGS.md` §3.3's "low 5 bits of a byte carrying
   flags" is stated of one and quoted of both. Cheap.
4. **`0x10b4dbbc` carries five `cc` lookups**, more than any other function
   (§7), including the two Ghidra cannot see. Unread, and it is the densest
   negation consumer in the image.
5. **`S` / `NS` / `VALL` / `NVALL` / `VNONE` / `NVNONE`** — named but not
   understood (§6). The read that would earn them is a consumer, and this lane
   found none: every consumer it read dispatches codes 1–10 only.
