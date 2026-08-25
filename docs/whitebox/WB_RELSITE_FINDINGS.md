# WB_RELSITE — the IL-opcode → relation-code site is **`FUN_10bbffbb`**, it is a six-arm literal chain, and it stores the **COMPLEMENT**

> **PROVENANCE — DISASSEMBLY-DERIVED.** Every address here is an absolute VA in
> `compilers/X360/16.00.11886.00/c2.dll`, sha256
> `c80981c015166effecc71ad8112d5577a065b2300891dfdb02b9c13787a66258`
> (**verified by this lane** against `C2_MAP_METHOD.md` §0 before the first
> read). Nothing here is copied into `crates/` — `w-relsite` ships **zero
> `crates/`, `fixtures/` and `c2host/` bytes** — so **no `DISCLOSURE.md` row is
> due**. A lane that adopts any byte below owes one; §11 says how many.

    Lane:       w-relsite (characterization)
    Date:       2026-08-25
    Prereg:     WB_RELSITE_PREREG.md — frozen as this lane's FIRST commit (b96459476)
    Assignment: w-relread §9 item 1 = WB_RELREAD_FINDINGS.md §10 item 1 = w-c7 prereg W2
    Method:     READ, then ONE confirmation capture. No probe grid.
    Tool:       scripts/dump_relsite.py — sha256-fenced, watched refusing
    Board:      #3546-#3550

**Marker convention, used on every claim and never blurred:**

| | |
|---|---|
| **`[R]`** | read out of the pinned image by this lane |
| **`[O]`** | measured against real `c2.dll` — by this lane's capture (§5) or by another lane, cited |
| **`[I]`** | an inference joining them. **Not a finding.** Marked so it can be attacked separately |

---

## 0. The one-paragraph answer

**The site is `FUN_10bbffbb` @ `0x10bbffbb`** — 98 bytes, reached from exactly
one place, arm 7 of the IL-record dispatch at `0x10bc38a1`, which is a 13-byte
`lea`/`call`/`jmp` router. It converts the IL opcode with a **`sub eax,0x1f`
plus four `dec eax`/`je` plus a fallthrough**, six arms, each a single
`mov bl,imm8`: `1F→2, 20→1, 21→4, 22→6, 23→3, 24→5` **`[R]`**. There is **no
table** — `w-relread`'s image-wide elimination of the contiguous-table form
holds, and the remaining shape it predicted (a per-opcode literal in the decode
switch) is exactly what is there. **`relation code = IL opcode − 0x1E` is
refuted from the code, on 6 of 6 arms and not 4 of 6** — including `EQ` and
`NE`, the two `w-c7` §2's heading got right by luck, because **this site stores
the COMPLEMENT of the source relation, not the relation**. A confirmation
capture against real `c2.dll` under wibo pins the namespace at the strongest
resolution the artifact allows: over six one-function TUs differing only in the
relational operator, **exactly one byte of the 2 760-byte `.ex` differs**, and
it takes the values `1F 20 21 22 23 24` for `== != <= < >= >` **`[O]`**. And the
answer was reachable from a landed document in this tree the whole time —
`ref/P_ILRECORD.md`'s arm table named arm 7 and its callee — which makes this
**`#3098`'s family, seventh instance**, and the first one where the prior art
was a `docs/whitebox/` artifact rather than a board row.

---

## 1. WHAT WAS DECLARED IN THE PREREG, AND WHAT IT COST TWO LANES

The prereg §1 declares, before any disassembly, that orientation had already
found `ref/P_ILRECORD.md`'s arm table row:

> | 7 | `10bc38a1` | `1f`…`24` (6) | 00 | 13 | 1 | — | ROUTE | DEFER | → `0x10bbffbb`. **All six relational operators share one arm and no discriminator is passed** — the callee re-reads the opcode through `ecx+4` |

`w-read-r5` (boards `#3415`–`#3421`) read **62 of 62 arms and 189 of 189
opcodes** and published that row. **`w-c7` then missed this site, and
`w-relread` then missed it again**, and between them they spent a table-algebra
derivation that produced a false identity and an image-wide byte search that
produced a null. Neither grepped `docs/whitebox/`.

**This is `#3098`'s family and it is the SEVENTH recorded instance**
(`#3517` was the sixth). What is new in this one, and worth the row:

* the six prior instances were all **board rows** that a topic grep missed. This
  one is a **`docs/whitebox/` artifact** — a different shelf, missed the same
  way, which means "grep the board for the row you were told about" (`#3517`'s
  operational rule) would **not** have found it either;
* the way it *was* found is not clever and is repeatable: **the brief said
  "check `docs/whitebox/` for an existing artifact covering your subject"**, and
  the check took under a minute. That instruction is in the brief because
  briefing a lane to create a doc that already exists has happened twice here.
  It has now paid for itself a third way — by finding an *answer*, not a
  duplicate.

**The operational rule this adds to `#3517`'s: before a location read, grep the
`docs/whitebox/ref/P_*.md` specs for the opcode, address or field you are
hunting. They are structured tables and a topic grep does hit them — nobody
ran one.** Board **#3546**.

---

## 2. The routing — exactly six opcodes reach arm 7, denominator 189 `[R]`

`FUN_10bc2d7a`'s dispatch head, read from the objdump:

```
10bc2e08:  8b 55 cc              mov   edx,DWORD PTR [ebp-0x34]     ; the IL opcode
10bc2e0b:  8d 42 ff              lea   eax,[edx-0x1]
10bc2e0e:  3d bc 00 00 00        cmp   eax,0xbc
10bc2e13:  0f 87 2a 13 00 00     ja    0x10bc4143                   ; the C1001 default
10bc2e19:  0f b6 80 4a 42 bc 10  movzx eax,BYTE PTR [eax+0x10bc424a] ; opcode-1 -> arm #
10bc2e20:  ff 24 85 52 41 bc 10  jmp   DWORD PTR [eax*4+0x10bc4152]  ; arm # -> arm VA
```

Two levels: a **189-byte index** at `0x10bc424a` (opcodes `0x01`..`0x bd`) and a
**dword arm table** at `0x10bc4152`. Decoded from raw bytes:

| fact | value | denominator |
|---|---|---|
| opcodes whose index byte is `7` | **`0x1f 0x20 0x21 0x22 0x23 0x24` — exactly six, no others** | all **189** index entries |
| distinct arm numbers in the index | **62** | all 189 entries (reproduces `WB_ILRECORD_FINDINGS.md` §0's count from a second decode) |
| arm-table entry 7 | `0x10bc38a1` | the 8th dword at `0x10bc4152` |
| neighbours, for orientation | `0x1a`→arm 5, `0x1b`/`0x1c`→arm 6, **`0x1d`/`0x1e`→arm 61** (the C1001 catch-all) | — |

Arm 7 itself, raw:

```
10bc38a1:  8d 4d c8              lea  ecx,[ebp-0x38]     ; &the record cursor
10bc38a4:  e8 12 c7 ff ff        call 0x10bbffbb         ; THE SITE
10bc38a9:  e9 48 07 00 00        jmp  0x10bc3ff6         ; the loop tail
```

**Thirteen bytes, and the opcode is not among them.** It does not need to be:
`edx` still holds it from `0x10bc2e08` and is not clobbered on the path.

### 2.1 `ref/P_ILRECORD.md`'s arm-7 row is WRONG in both of its clauses `[R]`

The row says *"no discriminator is passed — the callee re-reads the opcode
through `ecx+4`."*

* **A discriminator IS passed**: `edx`, inherited live from the dispatch head.
  `FUN_10bbffbb` reads it in its second instruction after the prologue
  (`10bbffc6: mov eax,edx`).
* **The callee does not read `ecx+4`.** `ecx` on entry is `&[ebp-0x38]`, a
  pointer to the caller's record cursor; the callee does `mov edi,ecx` then
  `mov ecx,[edi]` — it dereferences `ecx`, at offset **0**, to get the cursor
  value, and writes it back at `10bc0016: mov [edi],esi`.

This is **prereg control M1 firing** — registered at p = 0.25 that at least one
prior artifact I navigated by would disagree with the raw decode. It did, and
the disagreement is not cosmetic: *"no discriminator is passed"* is precisely
the sentence that would send a reader looking for the discriminator somewhere
else, which is what happened twice. **Amended beside, never edited** (the
`#3495`-on-`#3468` convention): `P_ILRECORD.md` keeps its text and carries an
inserted banner. Board **#3547**.

---

## 3. THE SITE — `FUN_10bbffbb`, 98 bytes, six literal arms `[R]`

`0x10bbffbb` .. `0x10bc001c` inclusive, `0x62` bytes. Raw:

```
55 8b ec 51 53 56 57 8b f9 8b 0f 8b c2 83 e8 1f 89 4d fc 74 20 48 74 19 48 74
12 48 74 0b 48 74 04 b3 05 eb 12 b3 03 eb 0e b3 06 eb 0a b3 04 eb 06 b3 01 eb
02 b3 02 ba d4 02 00 00 8d 4d fc e8 f2 f8 ff ff 8b 75 fc 8b 46 08 8b 40 2c 8a
48 15 c0 e3 04 80 e1 0f 0a cb 88 48 15 89 37 5f 5e 5b c9 c3
```

Disassembled:

```
10bbffbb:  55                push ebp
10bbffbc:  8b ec             mov  ebp,esp
10bbffbe:  51 53 56 57       push ecx / ebx / esi / edi
10bbffc2:  8b f9             mov  edi,ecx             ; edi = &cursor
10bbffc4:  8b 0f             mov  ecx,[edi]           ; ecx = *cursor
10bbffc6:  8b c2             mov  eax,edx             ; edx = THE IL OPCODE
10bbffc8:  83 e8 1f          sub  eax,0x1f            ; <== the dispatch INDEX
10bbffcb:  89 4d fc          mov  [ebp-0x4],ecx       ; flag-preserving; local cursor
10bbffce:  74 20             je   0x10bbfff0          ; opcode 0x1f
10bbffd0:  48 / 74 19        dec eax ; je 0x10bbffec  ; opcode 0x20
10bbffd3:  48 / 74 12        dec eax ; je 0x10bbffe8  ; opcode 0x21
10bbffd6:  48 / 74 0b        dec eax ; je 0x10bbffe4  ; opcode 0x22
10bbffd9:  48 / 74 04        dec eax ; je 0x10bbffe0  ; opcode 0x23
10bbffdc:  b3 05             mov  bl,0x5              ; FALLTHROUGH — opcode 0x24
10bbffe0:  b3 03             mov  bl,0x3
10bbffe4:  b3 06             mov  bl,0x6
10bbffe8:  b3 04             mov  bl,0x4
10bbffec:  b3 01             mov  bl,0x1
10bbfff0:  b3 02             mov  bl,0x2
10bbfff2:  ba d4 02 00 00    mov  edx,0x2d4           ; the node opcode to mint
10bbfff7:  8d 4d fc          lea  ecx,[ebp-0x4]
10bbfffa:  e8 f2 f8 ff ff    call 0x10bbf8f1          ; mint the node
10bbffff:  8b 75 fc          mov  esi,[ebp-0x4]
10bc0002:  8b 46 08          mov  eax,[esi+0x8]       ; the node just minted
10bc0005:  8b 40 2c          mov  eax,[eax+0x2c]      ; its operand/info record
10bc0008:  8a 48 15          mov  cl,[eax+0x15]
10bc000b:  c0 e3 04          shl  bl,0x4              ; <== the code goes in the HIGH NIBBLE
10bc000e:  80 e1 0f          and  cl,0xf
10bc0011:  0a cb             or   cl,bl
10bc0013:  88 48 15          mov  [eax+0x15],cl       ; <== THE STORE
10bc0016:  89 37             mov  [edi],esi           ; cursor written back
10bc0018:  5f 5e 5b c9 c3    pop / leave / ret
```

### 3.1 The mapping it performs

| IL opcode | arm VA | instruction | relation code | name (`#3518`'s enum) |
|---|---|---|---:|---|
| `0x1F` | `0x10bbfff0` | `mov bl,0x02` | **2** | `NE` |
| `0x20` | `0x10bbffec` | `mov bl,0x01` | **1** | `EQ` |
| `0x21` | `0x10bbffe8` | `mov bl,0x04` | **4** | `GT` |
| `0x22` | `0x10bbffe4` | `mov bl,0x06` | **6** | `GE` |
| `0x23` | `0x10bbffe0` | `mov bl,0x03` | **3** | `LT` |
| `0x24` (and the fallthrough) | `0x10bbffdc` | `mov bl,0x05` | **5** | `LE` |

Three structural notes, each read rather than assumed:

* **There is no bound check and no default arm.** The sixth arm is a
  *fallthrough*, so any opcode other than `0x1F`..`0x23` arriving here would
  silently take code 5. The guarantee that only `0x1F`..`0x24` arrive is §2's
  index table, and it is exact.
* **The code lands in a 4-BIT field** — `shl bl,4` into the high nibble of
  `[record+0x15]`. Codes **16, 17, 18** (`NVALL`, `VNONE`, `NVNONE`) **cannot be
  represented in this carrier at all.** That is a real bound on the enum's use
  and it is new; `w-relread` §6 could find no consumer for 11–18 and this says
  where three of them cannot live.
* **All six mint the same node opcode `0x2d4`.** The relation is carried
  entirely by the nibble, not by the node kind.

### 3.2 Independently decoded, three ways `[R]`

Prereg control **M2**. The six literals reproduce from:

1. **GNU objdump, Intel syntax**, whole-image dump;
2. **GNU objdump, AT&T syntax** over just the arm block (`mov $0x5,%bl`, …) — a
   different renderer of the same decoder;
3. **`scripts/dump_relsite.py`**, this lane's own decoder, which walks the chain
   from raw bytes via the PE section headers and never sees objdump's output.

And **`w-relread`'s D4 boundary-alignment control**: dumping the site from three
instruction-aligned start VAs (`0x10bbffbb`, `0x10bbffc4`, `0x10bbffc8`) gives
**byte-identical instruction text**, differing only in objdump's own section
header line.

---

## 4. `relation code = IL opcode − 0x1E` is REFUTED FROM THE CODE — 0 of 6 `[R]`

| IL opcode | `opcode − 0x1E` | the site emits | agree? |
|---|---:|---:|---|
| `0x1F` | 1 | **2** | no |
| `0x20` | 2 | **1** | no |
| `0x21` | 3 | **4** | no |
| `0x22` | 4 | **6** | no |
| `0x23` | 5 | **3** | no |
| `0x24` | 6 | **5** | no |

**Zero of six.** This is *stronger* than `w-relread` §3.2's finding, which had
the identity holding on `EQ`/`NE` and failing on the four orderings. At the
actual site it fails on all six, because the site complements (§6) and
complementation moves `EQ`↔`NE` too.

**And the sharpest part is that a subtraction IS there.** `0x10bbffc8` is
`sub eax,0x1f` — one off from `0x1E`, on the opcode, inside the function that
produces the code. **It is the switch index and its value is discarded**: `eax`
is dead after the last `je`, and every arm loads an unrelated literal. A read
that stopped at the first arithmetic instruction on the opcode would have
"confirmed" `w-c7`'s heading. `dump_relsite.py` prints the disagreement count
mechanically so this cannot be eyeballed wrong.

**What this does NOT retract**: `w-c7` never saw this function, so this is not
a transcription error — its heading came from `WB_RELATION_FINDINGS.md` §2's
circular constraint 4 (`#3518`), which was *built* to make the map a
subtraction. The site is where that construction meets the binary and loses.
Board **#3548**.

---

## 5. THE CONFIRMATION PROBE — one byte of 2 760, and it is the opcode `[O]`

Prereg **M4**, and the standing caveat's whole point: `[R]` means *the
instructions were read correctly*, not *this is what c2 does*.

Six one-function TUs, identical but for the operator, captured with
`c2rs capture` at the default `/Ox /GS- /c` profile:

```cpp
int only_lt(int a, int b) { return a <  b; }     /* and ==, !=, <=, >=, > */
```

Every `.ex` is **2 760 bytes**. Across all six, the number of differing byte
offsets is **one**:

| offset | `==` | `!=` | `<=` | `<` | `>=` | `>` |
|---|---|---|---|---|---|---|
| `0x0aa7` | `0x1f` | `0x20` | `0x21` | `0x22` | `0x23` | `0x24` |

Denominator: all 2 760 byte positions, compared pairwise against the `<`
capture. **Prereg U1 is CONFIRMED at the strongest resolution the artifact
allows** — the IL opcode byte for a source relation is measured directly, and
the measurement does not go through the port's naming at all. `Rel::from_opcode`
(`crates/c2-il/src/func/mod.rs:1411-1416`) is independently correct.

Regenerate:

```sh
for r in eq ne le lt ge gt; do ./target/release/c2rs capture <tu>.cpp --keep-il il_$r; done
```

**This is the class of evidence `C2_MAP_METHOD.md` §7 calls much stronger** — a
white-box read confirmed by a route with no access to the image.

---

## 6. THE SITE STORES THE **COMPLEMENT**, and here is how many ways that is over-determined — checked constraint by constraint

Join §3.1 `[R]` with §5 `[O]`:

| source | IL opcode `[O]` | the site stores `[R]` | complement of the source? |
|---|---|---|---|
| `a == b` | `0x1F` | 2 `NE` | ✔ |
| `a != b` | `0x20` | 1 `EQ` | ✔ |
| `a <= b` | `0x21` | 4 `GT` | ✔ |
| `a <  b` | `0x22` | 6 `GE` | ✔ |
| `a >= b` | `0x23` | 3 `LT` | ✔ |
| `a >  b` | `0x24` | 5 `LE` | ✔ |

**Six for six.** Equivalently, the site computes `cc[R]` where `cc` is the
negation table at `0x10b189cc` (`#3489`/`#3518`) and `R` is the source relation:
`cc = 00 02 01 06 05 04 03 …`, and `cc[1]=2, cc[2]=1, cc[5]=4, cc[3]=6,
cc[6]=3, cc[4]=5`. Marked **`[I]`** — it joins a read to a measurement — but the
measurement is this lane's own and the read is mechanical.

### 6.1 The alternative, and how many of my constraints it also satisfies

**`w-relread` §2's rule, applied to my own claim before publishing it.** The
rival is **H2: the site stores the relation un-complemented, and it is the
*labels* that are shifted.** Four constraints bear on it:

| # | constraint | kills H2? |
|---|---|---|
| **1** | §5's capture: source `<` ⇒ IL `0x22` ⇒ code 6, and 6 is `GE` by the name array + six consumers (`#3518`) | **YES** — it measures source→opcode with no labelling assumption |
| **2** | `FUN_10c194b8` @ `0x10c194ef`/`0x10c1953b` rewrites `+0x34` **4→3** and **5→6**, each with an intervening `call 0x10bd3f17`, i.e. `b8[4]=3`, `b8[5]=6` with an operand action | **NO.** This is about the *algebra* of `{3,4,5,6}`, not its sense. **Satisfied by both.** Named as non-discriminating rather than counted |
| **3** | The transfer at `0x10bc0339` applies `cc` on the way to the `+0x34` carrier (§7), and `FUN_10c1a908`'s against-zero folds on that carrier are *semantically true statements* for the un-complemented relation (`x <u 0` ≡ false, `x >=u 0` ≡ true, `x <=u 0` ≡ `x==0`, `x >u 0` ≡ `x!=0` — `w-relread` C2–C5 `[R]`). `cc∘cc = id` | **YES** — under H2 those four folds would each be false, i.e. wrong code |
| **4** | `FUN_10bbfd7c`'s coercion of a non-condition value builds a `0x2d4` node with nibble **1 = `EQ`** (`0x10bbfe8e: or dl,0x10`). C truthiness is `v != 0` | **YES** `[I]` — under H2 c2 would be lowering a truth test as `v == 0` |

**Three of four discriminate; the fourth does not, and I say which.** That is
the exact discipline `WB_RELATION_FINDINGS.md` §2 skipped when it counted a
constraint that restated its conclusion.

### 6.2 Why complement — stated as `[I]` and not as a finding

The natural reading is a **branch-around encoding**: `if (cond) …` lowers to a
branch taken when `!cond`, so the record that will become the branch carries the
complement. §6.1's constraint 4 is consistent with it and constraint 3 explains
why the value position sees the un-complemented relation. **I am not publishing
"branch-around" as a finding** — I did not read the branch emitter, and
`w-r8idiom`/`w-2e4`'s rule is that a guess between the read and the check is how
an identity stops being one.

---

## 7. THREE CARRIERS OF THE SAME ENUM, AND THE TRANSFER BETWEEN THE FIRST TWO `[R]`

`w-relread` §4.5 left this open as ranked follow-on 3 (*"the `[node+0x34]` vs
`[node+0xa]` record split"*). It is a **three**-carrier question and this lane
closes the first two ends of it.

| carrier | where | width | written by | read by |
|---|---|---|---|---|
| **A** | high nibble of `[[node+0x2c]+0x15]` on a `0x2d4` node | **4 bits** — so codes 16–18 cannot live here | **`0x10bc0013`** — this lane's site, and **five others** (§7.1) | `0x10bbfe23` (`shr al,0x4`), `0x10bbdbe9`, `0x10b49767`, `0x10b497c4` — **4 nibble readers, and there are exactly 4 `shr r8,0x4` in the whole image** |
| **B** | `[node+0x34]`, a whole byte | 8 bits | `0x10bd74e7`, inside `FUN_10bd748b` (a kind-`0x11` node constructor, `ret 0x1c`, the code is its **5th** stack arg) | `FUN_10c1a908` @ `0x10c1a91c` (`w-relread` §4.5), `FUN_10c194b8` @ `0x10c194d6` |
| **C** | low 5 bits of `[param_1+0xa]` | 5 bits | unread | `FUN_10bd50b7` (`WB_RELATION_FINDINGS.md` §3.3) — **still unread, still not resolved** |

**The A → B transfer, read:**

```
10bc0335:  0f b6 45 e0                 movzx eax,BYTE PTR [ebp-0x20]   ; carrier-A code
10bc0339:  0f b6 80 cc 89 b1 10        movzx eax,BYTE PTR [eax+0x10b189cc]  ; <== cc, NEGATION
10bc0343:  57 / 68 35 38 bd 10 / 50    push edi / push 0x10bd3835 / push eax
10bc034d:  b9 ea 02 00 00              mov  ecx,0x2ea
10bc0358:  e8 20 76 01 00              call 0x10bd797d      ; -> FUN_10bd748b -> [node+0x34]
```

`[ebp-0x20]` is loaded at `0x10bc01b4` from the return value of
`call 0x10bbfd7c` @ `0x10bc01aa` — and `FUN_10bbfd7c` is the **carrier-A nibble
reader** (`0x10bbfe20: mov al,[eax+0x15]; shr al,0x4`), with the truthiness
fallback of §6.1 constraint 4. So:

**IL opcode →(`FUN_10bbffbb`, complement) carrier A →(`FUN_10bbfd7c`, read)
→(`cc`, complement again) carrier B**, and `cc∘cc = id`, so **carrier B holds
the source relation** — which is exactly the sense `FUN_10c1a908`'s folds
require. The loop closes.

**Also read, and it is the neatest cross-check on carrier A:** IL opcode `0x1a`
(arm 5, `0x10bc31fb`) **negates carrier A in place through the same `cc`
table** —

```
10bc321e:  8a 51 15   mov   dl,BYTE PTR [ecx+0x15]
10bc3221:  0f b6 c0   movzx eax,al
10bc3224:  8a 80 cc 89 b1 10   mov al,BYTE PTR [eax+0x10b189cc]
10bc322a:  80 e2 0f / c0 e0 04 / 0a c2 / 88 41 15   ; and 0xf, shl 4, or, store
```

— and then toggles bit 0 of `[record+0x14]`, which is the *"negate + flip the
taken bit"* shape `WB_RELATION_FINDINGS.md` §3.3 names at `FUN_10bd507f`.

### 7.1 CARRIER A HAS SIX WRITERS AND ONLY ONE OF THEM READS AN IL OPCODE — with the denominator `[R]`

**This is stated because it is the way §3 could be over-read.** "The site" is
the only place an *IL opcode* becomes a relation code (§2's index table settles
that, denominator 189). It is **not** the only place carrier A is written.

Enumerated over **all 24 byte stores to `[reg+0x15]` in the image**, keeping the
**6** that are high-nibble merges (an `and r8,0xf` within six instructions
before the store):

| VA | the code it writes | where |
|---|---|---|
| **`0x10bc0013`** | **six literals selected by the IL opcode** | **`FUN_10bbffbb` — THE SITE**, IL arm 7 (`0x1F`..`0x24`) |
| `0x10bbfe93` | literal **1 `EQ`** (`or dl,0x10`) | `FUN_10bbfd7c`'s coercion of a non-condition value (§6.1 constraint 4) |
| `0x10bc1ff7` | literal **4 `GT`** (`or cl,0x40`) | `0x10bc1fe0`, not an arm |
| `0x10bc3232` | **`cc[code]`** — negation, in place | IL arm **5**, opcode **`0x1a`** |
| `0x10bc3a82` | literal **1 `EQ`** (`or cl,0x10`) | IL arm **46**, opcode **`0x77`** |
| `0x10bc3def` | a **variable** in `bl`, `and bl,0x1f` then `shl bl,4` | IL arm **56**, opcode **`0xa0`** |

The last row is worth one sentence and no more. `and bl,0x1f` masks to **five**
bits and `shl bl,4` then discards bit 4, so a code of 16 or above is silently
truncated on the way into a four-bit field. That is the only place in this read
where a **5-bit** mask and carrier A meet, and `WB_RELATION_FINDINGS.md` §3.3's
*"low 5 bits of a byte that carries other flags"* is a 5-bit claim about
**carrier C**. **I am not joining them.** I did not read arm 56, I do not know
that `bl` is a relation code there, and opcode `0xa0` is outside this lane.

The complementary denominator: there are exactly **4** `shr`/`sar r8,0x4`
instructions in the entire image, and **all four** read `[reg+0x15]` — so the
nibble has four readers and they are enumerable.

### 7.2 A SEVENTH independent read that `0x10b189b8` is REFLECTION `[R]`

`FUN_10c194b8` normalises carrier B:

```
10c194ef:  3c 04                 cmp   al,0x4          ; GT
10c194f9:  e8 19 aa fb ff        call  0x10bd3f17      ; the operand swap, below
10c19501:  c6 47 34 03           mov   BYTE PTR [edi+0x34],0x3   ; -> LT
10c19539:  3c 05                 cmp   al,0x5          ; LE
10c19543:  e8 cf a9 fb ff        call  0x10bd3f17
10c1954b:  c6 47 34 06           mov   BYTE PTR [edi+0x34],0x6   ; -> GE
```

**And `FUN_10bd3f17` is read, not assumed — it is a 17-byte two-element
operand-list swap and nothing else** `[R]`:

```
10bd3f17:  8b 41 28   mov eax,[ecx+0x28]   ; the operand list head  = operand 1
10bd3f1a:  8b 10      mov edx,[eax]        ;                        = operand 2
10bd3f1d:  8b 32      mov esi,[edx]        ;                        = operand 3
10bd3f1f:  89 51 28   mov [ecx+0x28],edx   ; head := 2
10bd3f22:  89 02      mov [edx],eax        ; 2->next := 1
10bd3f24:  89 30      mov [eax],esi        ; 1->next := 3
10bd3f27:  c3         ret
```

So `4→3` and `5→6` are `b8[4]` and `b8[5]`, each with the compare's two operands
**exchanged**: `a > b` becomes `b < a`, and `a <= b` becomes `b >= a`. Both are
valid rewrites. Under the retracted *"strictness flip"* labelling the same two
rewrites would read `LT→LE` and `GE→GT` **with the operands exchanged**, and
`a < b` is not `b <= a`. **This is a site outside `w-relread`'s six**, it needs
no strings, and it agrees with them. Board **#3549**.

---

## 8. WHAT THIS LANE REFUSED TO NAME

Per `w-r8idiom` / `w-2e4` / `w-relread` §5, and prereg **M6**.

* **"Branch-around"** as the *reason* for the complement (§6.2). The
  complementation is read and measured; the explanation is `[I]` and the branch
  emitter is unread.
* **Carrier C** (`[param_1+0xa]`, low 5 bits, `FUN_10bd50b7`). Untouched by this
  lane. `w-relread`'s follow-on 3 is **two thirds closed, not closed**, and
  `WB_RELATION_FINDINGS.md` §3.3's *"low 5 bits of a byte carrying other flags"*
  is still a claim read off one record and quoted of three.
* **What node opcode `0x2d4` *is*.** It is minted by `0x10bbffbb`,
  `0x10bbfe74` and `0x10bf086f`, and matched by `0x10bbdba9`. That is four
  sites and not a definition.
* **The origin of carrier B's code on paths that do not come through carrier
  A.** `FUN_10bd748b` has **3** callers; `0x10bf08a0` passes a hardcoded
  `push 0x6`, `0x10bd79a7` is a forwarding wrapper (`FUN_10bd797d`, **7**
  callers), and `0x10b81a5f`'s argument was **not traced**. Denominator
  published; the trace is not.
* **Whether codes 16–18 are reachable at all.** §3.1 shows they cannot live in
  carrier A. That is a bound, not a proof of unreachability, and I do not claim
  one.

---

## 9. Instrument defects — two, both found by running the registered controls

`CLAUDE.md`'s instrument-defect rule. Both are in **this lane's own** tool.

| # | control | what it found |
|---|---|---|
| **D1** | M3 — watch the instrument on its first honest run | The chain walker **terminated at the `mov [ebp-0x4],ecx` sitting between the `sub` and the first `je`** and reported a chain of **length 0** — then crashed formatting a `None`. The store is flag-preserving and belongs inside the chain. Fixed: such stores are stepped over and **named in the output**, never silently. Prereg registered p = 0.25 that my first fence would be wrong — **the sha256 fence was right and the decoder was wrong**, which is a distinction worth keeping |
| **D2** | M1 — vary a parameter it must not depend on | `--site 0x10c1ac5c` and `--site 0x10c1a908` **crashed** on the same `None` format instead of refusing. Both are relation dispatchers in *other* shapes (`dec ecx` / `sub ecx,5`; a jump table). Fixed: the decoder now prints `DECODER REFUSED … Nothing was decoded` and exits 1 — **it refuses rather than emitting a table that would look like an answer** |

**The generalisation worth carrying**: `w-relread`'s D1 was a fence defect and
mine were **decoder** defects behind a working fence. A fence that refuses a
corrupt image says nothing about whether the parse behind it is right. Both
have to be watched, and the second one is watched by pointing the tool
somewhere it should refuse.

### 9.1 Fence evidence — watched refusing before any output was trusted

`dump_relsite.py` verifies sha256 **in `__init__`, before any PE parse**.

| case | result |
|---|---|
| truncated to 600 000 of 1 347 072 bytes | `IMAGE FENCE REFUSED … Nothing was read.` **exit 3** |
| **one bit flipped** at file offset `0x40000`, size identical (1 347 072) | `IMAGE FENCE REFUSED … Nothing was read.` **exit 3** |
| unreadable path | `IMAGE FENCE REFUSED — cannot read it: …` **exit 3** |
| the pinned image | proceeds, prints the digest as `(PINNED, verified)`, **exit 0** |

### 9.2 The parameter-independence control (`#3483`)

| `--max-arms` | result |
|---:|---|
| 3 | **`BOUND EXHAUSTED — count is my parameter, not the image's`**, exit 1 |
| 5, 6, 32, 256 | **six arms**, identical table, exit 0 |

---

## 10. Pre-registration score — **12 hits, 3 misses, 2 partials, 2 controls fired**

> This header first read **"13 hits"** and was **wrong** — I wrote the tally by
> hand before counting the table, which is the *exact* thing `w-relread` §9
> caught itself doing and wrote down so the next lane would not. The next lane
> did it anyway. Corrected from
> `grep -oE '\*\*(HIT|MISS|PARTIAL|FIRED)\*\*' | sort | uniq -c` over the
> verdict column (12 / 2 / 2 / 2; the third MISS is S6, whose verdict cell
> carries trailing prose inside the bold and so is not matched by that regex —
> **which is itself an instrument defect in the counting method, and the reason
> the hand count and the machine count must agree before either is published**).
> The miscount is left on the record: a lane whose own `#3550` is *"do not
> substitute a description for a read"* does not get to hand-wave its own
> arithmetic.

Counted mechanically from the verdict column, not by hand (`w-relread` §9's
lesson — learned late, see above).

| | registered | measured | |
|---|---|---|---|
| **S1** | arm 7 verifies: ~13 B at `0x10bc38a1`, only call → `0x10bbffbb` (p 0.80) | 13 B exactly; `lea`/`call`/`jmp` | **HIT** |
| **S2** | the conversion is inside `FUN_10bbffbb`, not a level deeper (p 0.55) | it is, 13 bytes into the body | **HIT** |
| **S3** | I name the site with a VA and the materialising instruction (p 0.70) | `0x10bbffbb`; six `mov bl,imm8`; the store at `0x10bc0013` | **HIT** |
| **S4** | a compare/`sub`+`je` chain with per-opcode literals, not a table (p 0.65) | `sub eax,0x1f` + 4 `dec`/`je` + fallthrough | **HIT** |
| **S5** | the conversion is not arithmetic on the opcode (p 0.85) | the only arithmetic is the dead switch index | **HIT** |
| **S6** | the literals are `1,2,5,3,6,4` for `1F..24` (p 0.75) | they are **`2,1,4,6,3,5`** — the **complement** of what I registered | **MISS, and the lane's most valuable result** |
| **S7** | the code is stored at node `+0x34` (p 0.40) | high nibble of `[[node+0x2c]+0x15]`, a **4-bit** field; `+0x34` is a *second* carrier | **MISS** |
| **S8** | `FUN_10bbffbb` < 1 500 bytes (p 0.60) | **98** | **HIT** |
| **S9** | the same function mints the node opcode (p 0.50) | `mov edx,0x2d4` @ `0x10bbfff2` | **HIT** |
| **T1** | exactly one opcode→code site for the six (p 0.55) | exactly six opcodes reach exactly one arm, denominator 189 | **HIT** |
| **T2** | guard and value positions share the conversion (p 0.45) | they share **the site**, and are separated by the A→B transfer's second `cc` (§7) | **PARTIAL** — right that there is one converter, wrong that nothing intervenes |
| **U1** | the dispatch's opcode space is the container's (p 0.80) | one byte of 2 760, `1F`..`24` | **HIT** `[O]` |
| **M1** | ≥ 1 prior artifact I navigate by disagrees with the raw decode (p 0.25) | `P_ILRECORD.md` arm 7, both clauses (§2.1) | **HIT** |
| **M2** | two independent sources disagree somewhere (p 0.30) | three sources, three boundary-aligned starts, **no disagreement** | **MISS** |
| **M3** | my first fence is wrong (p 0.25) | the **fence** was right; the **decoder** was wrong, twice (§9) | **PARTIAL** |
| **M4** | the confirmation probe confirms (p 0.70) | one byte, exactly the six opcodes | **HIT** |
| **M5** | publish the denominator with any null | 189 index entries; 2 760 `.ex` bytes; 3 callers; 4 nibble readers | **FIRED** |
| **M6** | I refuse ≥ 1 name (p 0.6) | five, §8 | **HIT** |
| **M7** | check "over-determined" constraint by constraint | 3 of 4 discriminate; the 4th named as not doing so (§6.1) | **FIRED** |

### The misses that matter

* **S6 is the one to read.** I registered the literals as `1,2,5,3,6,4` — the
  map `w-relread` §3.2 derived by joining the port's `Rel` names to the enum
  names. That join is an `[I]`, correctly marked as such by that lane, and I
  registered it at p = 0.75 as though it were the answer. **The site emits the
  complement of it, on all six.** Nobody was wrong: §3.2's table is the
  *semantic* map and the site's is the *stored* map, and the two are separated
  by a negation nobody had read. **Registering a derived quantity at high
  credence is what made the gap legible** — `w-relread`'s S1b lesson, repeating
  in a different subject and with a bigger consequence.
* **S7 at p = 0.40 was my lowest site prediction and it still over-reached** —
  I predicted the field a *consumer* reads and the site writes a different one.
  The lesson is narrow and useful: **a consumer's field is not the producer's
  field**, and in this compiler there are at least three carriers of one enum.
* **M2 is a MISS I would rather have.** I registered p = 0.30 that two
  disassembly sources would disagree, because they did for `w-relread` (D3).
  They did not here. Recording a control that *passed* matters as much as one
  that fired — `#3483`'s point is that a green control is a statement about
  what it could have caught, and this one could have caught `w-relread`'s exact
  defect.

---

## 11. What this is worth, and what it is not

**Worth.** A two-lane open miss is closed with a VA, a byte listing and a
mechanical decoder. `w-c7`'s prereg **W2** and `w-relread`'s prereg **S2c** are
both answered. `code = opcode − 0x1E` is refuted at the place it would have had
to be true, on 6 of 6 rather than 4 of 6. The IL dispatch's routing for the
relational family is exact with a published denominator. A **third** carrier
question is two-thirds closed. `0x10b189b8` gets a seventh independent
confirmation as reflection. And the enum gains a hard structural bound: three
of its nineteen codes cannot fit in the field the front half of the compiler
stores them in.

**Not worth.** Predicted reach **0**, delivered **0**. Nothing here converts a
TU, moves the census, or changes a byte of the port. The port does not need the
complement: `Rel::from_opcode` is graded on *behaviour*, and it is confirmed
correct by §5 rather than corrected by it.

**Also not.** Nothing is adopted; `crates/`, `fixtures/` and `c2host/` are
byte-identical to base. **`DISCLOSURE.md` is unchanged on purpose.** The first
lane to bake any of this into a port table now owes **five** rows, not four:
the name array `0x10c38690`, the three tables `0x10b189a4`/`b8`/`cc`, **and the
site's six literals at `0x10bbffdc`..`0x10bbfff2`**.

**The transferable result.** `w-relread` closed with *"the algebra of a
permutation table determines the labelling only up to the automorphisms of that
algebra — a consumer or a name is required."* This lane adds the next term:
**a name and a consumer fix the labelling; neither fixes the SENSE.** The enum
was read correctly, six consumers agreed, and the map from the IL was still off
by a global negation, because every one of those consumers reads a carrier two
hops downstream of the producer. **The producer had to be read.** That is the
argument for location reads over value reads in one sentence.

**And it is why `w-c7`'s W2 was right to score a MISS for recovering a value
instead of a location — a verdict that would still have been right if the value
had been correct.** `w-c7`'s value was wrong (`opcode − 0x1E`). `w-relread`'s
§3.2 replacement is *semantically* right — it is the map from the IL opcode to
the relation the source wrote, and this lane's §5 capture confirms every row of
it. **It is still not what the site stores**, and no amount of getting the value
more correct would have surfaced that, because a value read has nowhere to put
the observation *"and then it is complemented on the way in."* Only an address
does. Board **#3550**.

---

## 12. Ranked follow-ons

1. **Carrier C — `FUN_10bd50b7` / `[param_1+0xa]`, low 5 bits** (cheap).
   `w-relread`'s follow-on 3, now the only unclosed third of it. It is also the
   only carrier wide enough for codes 11–18 *and* narrow enough to be masked,
   which makes it the best candidate for the consumer §8 could not find.
2. **The branch emitter that consumes carrier A's complement** (~½ day) — it
   would turn §6.2's `[I]` into a read, and it is the last step between "the
   site stores the complement" and "here is why".
3. **`0x10b81a5f`'s relation argument to `FUN_10bd748b`** (cheap) — the one
   carrier-B origin this lane did not trace, named with its denominator in §8.
4. **What node opcode `0x2d4` is**, from its four sites (§8).
5. **The six against-zero emitters** — unchanged from `w-relread` §10 item 2,
   and still what a *byte-level* retirement of `#423` needs.
