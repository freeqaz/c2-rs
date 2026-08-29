# `WB_INSTRCOUNT` — what `[fn+0x50]` is, where it comes from, and why F7 measured zero

> **Lane `w-instrcount`, 2026-08-29. Kind: characterization. Outcome: `built`.**
> Prereg: [`work/w-instrcount/PREREG.md`](../../work/w-instrcount/PREREG.md),
> frozen before the image was opened. Board **#3824**–**#3830**.
> Image: `compilers/X360/16.00.11886.00/c2.dll`, sha256
> `c80981c015166effecc71ad8112d5577a065b2300891dfdb02b9c13787a66258`.
> Predicted reach **0**; census **+0**; byte delta **0 by construction** — this
> lane changes no compiled file.
>
> **Evidence tiers**, as everywhere in this directory: **`[R]`** read from the
> disassembly · **`[O]`** confirmed against a real obj / captured IL, witness
> named · **`[I]`** an interpretive step.
>
> **This page does not edit `docs/whitebox/ref/P_INLINE.md` or
> `work/w-inlmetric/CLAUSES.tsv`.** Both belong to `w-clausegen` this wave, by
> construction, because splitting them caused `#3814`. Everything below that
> would change a clause row is stated here as a **proposal for a later wave**,
> with the address a future editor needs.

---

## 0. The answer in six lines

* **`[fn+0x50]` is `WORD [sym+0x50]`, a 16-bit field of the `.gl` symbol
  record — reached by ONE indirection the clause text omits** (`0x10b626f5`
  loads `[fn+0x00]` first). `[fn]` is the symbol; the count is on the symbol.
* **It is the `.gl` function record's `SIZE` field, arriving verbatim from the
  front end.** Its sole writer in the image is `0x10b9bf6c`, the `.gl` reader.
* **The unit is the FRONT END's count**, not machine instructions and not
  bytes. c2 never computes it; c2 only reads, sums and compares it.
* **The `ushort` at the seed is a no-op**; the real 16-bit ceiling is imposed
  by the IL encoding, and the `0x81..0xff` single-byte form makes a function
  read as **32,896..65,535** at the consumer — which is a live hazard, not a
  theoretical one.
* **F7 is a property of the grid, not of c2**: the caller count reaches exactly
  two predicates, and on the D family they had **≥ 7.9× and 6.1× of slack**.
  The published *"`B` from 1000 to ~2,820"* is arithmetic in the wrong unit —
  **measured, it is 1,000 → 9,846**.
* **The read unblocks 2 of the 4 rows the brief names**, not 4. C4 and C20's
  binding blocker was never the count.

---

## 1. The load, read instruction by instruction `[R]`

`FUN_10b62675`, the inline pass entry. `ecx` on entry is the `fn` object.

```
10b62675:  55                    push  ebp
10b62676:  33 ed                 xor   ebp,ebp            ; ebp = 0 for the rest
10b62678:  56                    push  esi
10b62679:  8b f1                 mov   esi,ecx            ; esi = fn
10b6267b:  39 2d c4 0e c4 10     cmp   ds:0x10c40ec4,ebp  ; C1: pass enabled?
10b62681:  75 3e                 jne   0x10b626c1
...
10b626f5:  8b 06                 mov   eax,DWORD PTR [esi]     ; <-- [fn+0x00] = THE SYMBOL
10b626f7:  0f b7 40 50           movzx eax,WORD PTR [eax+0x50]  ; <-- THE COUNT, zero-extended
10b626fb:  83 0d c8 f5 c3 10 ff  or    ds:0x10c3f5c8,0xffffffff
10b62702:  53                    push  ebx
10b62703:  a3 cc f5 c3 10        mov   ds:0x10c3f5cc,eax        ; C2: seed the running total
10b62708:  03 c0                 add   eax,eax                  ; C3: 2 x count
10b6270a:  bb e8 03 00 00        mov   ebx,0x3e8                ;     1000
10b6270f:  3b c3                 cmp   eax,ebx
10b62711:  7e 02                 jle   0x10b62715                ;     floor
10b62713:  8b d8                 mov   ebx,eax
10b62715:  b8 b8 88 00 00        mov   eax,0x88b8                ;     35000
10b6271a:  3b d8                 cmp   ebx,eax
10b6271c:  7c 02                 jl    0x10b62720                ;     ceiling
10b6271e:  8b d8                 mov   ebx,eax
...
10b6276e:  e8 6e f7 ff ff        call  0x10b61ee1                ; C4: the driver, ebx = B
```

**Two corrections to the clause text, both small and both load-bearing.**

1. `CLAUSES.tsv` C2 and `P_INLINE` §1 write the quantity as `(ushort)[fn+0x50]`.
   The instruction pair is `mov eax,[esi]` **then** `movzx eax,WORD [eax+0x50]`
   — `[[fn]+0x50]`. The count lives on the **symbol record**, which is the
   *same struct* the callee-side clauses (C8, C17, C18, C19, C24) read at
   `[sym+0x50]`. Caller and callee read one field of one struct type; that
   unification is not stated anywhere in the tree today.
2. The same `[esi]` object is used at `0x10b62731`–`0x10b62733`
   (`mov eax,[esi]; mov eax,[eax+0x37]; shr eax,0x1b`) — the 3-bit COFF linkage
   word `P_GLOBREGS` reads at `[gl+0x37]`. **Independent confirmation that
   `[fn+0x00]` is the `.gl` symbol record**, from a second field.

The `(ushort)` in the decompiler's rendering is the `movzx` and nothing else:
the field is 16 bits **at rest**, so the seed narrows nothing. See §4 for where
the 16-bit ceiling actually bites.

---

## 2. The producer chain, and a write census that states its blind spots `[R]`

### 2.1 The census, because `#3505` is six for six

`P_INLINE` §2.1a asserts a universal negative — *"there is exactly ONE 16-bit
store to `[reg+0x50]` in the whole image"*. `#3505`'s sharpest instance is an
xref census that returned **60 refs / 0 writes correctly**, because the write
went through `rep movsd` and `EDI`. A grep for `mov WORD PTR [reg+0x50]`
reproduces that defect exactly: it cannot see a DWORD store (which covers
`0x50..0x53`), a store through an advanced base, a `stosw`, or a block copy.

So the census was re-run with the classes enumerated rather than assumed
([`census_p50.py`](../../work/w-instrcount/census_p50.py),
[`classify_p50.py`](../../work/w-instrcount/classify_p50.py); outputs beside
them). Two filters, both necessary:

* **c2.dll has no `.rdata` section.** Strings and tables live inside `.text`
  (VMA `0x10b01000`–`0x10c2dc7c`, `objdump -h`), so objdump disassembles data
  as instructions and the raw census is polluted with `arpl WORD PTR [ecx+0x50]`
  and `add BYTE PTR [ebp+ebp*2+0x50],al`. **125 raw operands → 105 inside a
  Ghidra function extent.**
* **Displacement `0x50` is not a struct identity.** Attribution to the `.gl`
  record requires the same function, on the same base register, to also touch a
  field already identified on that record (`+0x20`, `+0x37`, `+0x4c`, `+0x52`,
  `+0x54`, `+0x58`).

**Result over the 105 in-function operands:**

| | count |
|---|---:|
| WORD **write** at `+0x50` | **1** — `0x10b9bf6c` |
| WORD **read** at `+0x50` | 9 |
| DWORD / BYTE writes at `+0x50` | 22 |
| DWORD / BYTE reads at `+0x50` | 73 |

**The single WORD write is `0x10b9bf6c`, `mov WORD PTR [esi+0x50],ax`, in
`FUN_10b9b8e9` (`p2symtab.c`, 3,307 bytes)** — and it carries the richest
corroboration of any row in the census: the same function, on the same base
register, touches `+0x20`, `+0x37`, `+0x4c`, `+0x52`, `+0x54` **and** `+0x58`.
No other row in the census reaches more than **four**, and the runner-up (`FUN_10c1c3f7`, `0x20,0x4c,0x54,0x58`) is missing exactly the two fields that are specific to this record — `+0x37`, the unaligned linkage word, and `+0x52`, the WORD that only exists if `+0x50` is 16 bits wide.

**None of the 22 DWORD/BYTE writes is on this struct.** The one that looks
closest — `FUN_10b3f454`'s three stores at `0x10b3f557`/`0x10b3f568`/`0x10b3f5b3`,
which corroborate on `+0x4c`, `+0x54`, `+0x58` — is refuted by reading it: its
zero-init arm at `0x10b3f5aa`–`0x10b3f5bf` writes `+0x44 +0x48 +0x4c +0x50 +0x54
+0x58 +0x5c +0x60` as **eight consecutive DWORDs**, so its `+0x50` is a 32-bit
field and there is no `+0x52`. **`+0x4c`/`+0x54`/`+0x58` are generic offsets and
the heuristic false-positives on them; only reading the body settles it.** That
is worth saying out loud, because a census that had stopped at the
corroboration column would have published three phantom writers.

### 2.2 The block-copy blind spots, searched rather than asserted

* **`rep movsd`: 28 sites, 0 candidates.** Every site's `ecx` was read
  (contexts in the lane's notes). The only destinations that extend past
  `+0x50` are `0x10b89576` (`ecx = 0x338` dwords from the global template
  `0x10c40220` into an object with a field at `+0xcd0` — a >3 KB global state
  block, not a `0x94`-byte symbol record), `0x10bffa7c` (`0x2b` dwords,
  stack→stack) and `0x10bf3d06` (`0x20` dwords, stack→stack). The largest
  heap-destination copy is `0x10b7636e` at `0x14` dwords = **`0x00..0x4f`,
  stopping one byte short of the field**.
* **`rep stosd`: 32 sites.** These zero-fill; a fill at allocation precedes the
  `.gl` read and cannot be a *reducer*. Recorded as a bounded class, not
  cleared individually.
* **`memcpy` / `memset` thunks (`0x10c2885c` / `0x10c28862`): 119 call
  sites — NOT individually cleared.** This is the one blind spot this lane
  leaves open, and it is named rather than papered over. Closing it is a
  bounded follow-up read (§7).

> **`[R]` — `[sym+0x50]` has exactly one writer in c2.dll, `0x10b9bf6c`, the
> `.gl` record reader, subject to one named and unclosed blind spot (119
> memcpy/memset call sites).**

### 2.3 The reader, field by field `[R]` — reproduced, not relayed

```
10b9bf57:  call 0x10c1f9e9   (il-read-varint32)  -> [esi+0x54]   `80 <LE32>` offset
10b9bf5f:  call 0x10c1f9e9   (il-read-varint32)  -> [esi+0x58]   SRCPOS
10b9bf67:  call 0x10c1f9a6   (il-read-varint16)
10b9bf6c:  66 89 46 50       mov WORD PTR [esi+0x50],ax          <-- SIZE, THE COUNT
10b9bf70:  call 0x10c1f91b                       -> [esi+0x4c]   ATTR
10b9bf7b:  call 0x10c1f9a6   (il-read-varint16)  -> WORD [esi+0x52]
```

This confirms `P_INLINE` §2.1a's decode independently.

### 2.4 The tension this lane was dispatched to settle — and it settles one half

Prereg §2 registered that two `[O]` claims in `P_INLINE` cannot both be true as
written:

* **§2.1a** — *"there is exactly ONE 16-bit store to `[reg+0x50]` in the whole
  image"*;
* **§2.1b** — *"`[sym+0x50]` is **initialized** from `SIZE` and is then
  **reduced by whatever runs before the inliner**"*, on a matched pair with
  identical `SIZE = 115` and opposite verdicts.

**§2.1a survives the proper census (§2.1–§2.2). So there is no writer in the
image to be §2.1b's reducer**, and c1xx does not fold before emitting either —
both cells' counts are exactly `19 + 8×12` and `19 + 12×8`, to the unit.

§2.1b's **headline** — *"the `.gl` `SIZE` field is NOT the value the decision
tests"* — is nevertheless still standing, and this lane can say something
stronger than §2.1b could, from `w-sizebracket`'s own raw
`series.jsonl` rather than from its prose:

| cell | `gl_size` | **`gl_attr`** | `caller_gl_size` | profile | arm |
|---|---:|---:|---:|---|---|
| `arith_012_O1` | **115** | **104 = `0x68`** | 21 | `/GR /O1 /Oi /EHsc` | **inlined** |
| `mix_008_O1` | **115** | **104 = `0x68`** | 21 | `/GR /O1 /Oi /EHsc` | **kept** |

> **`[O]` — EVERY input to `FUN_10b5fb5f` that this project has identified is
> IDENTICAL across the pair.** The count is the same (115), the `ATTR` word is
> the same (`0x68` — so `& 0x2080` is **0** in both, and §2.5's escape is not
> the separator either), the caller is the same (21), the globals are the same
> compilation. **Candidacy therefore returns the same verdict for both, whatever
> that verdict is, and `P_INLINE` §2.1b's separation is provably DOWNSTREAM of
> `0x10b5fc8a`.** It is not the size test, and it is not `0x2080`.

**And there is no writer in the image to be its stated reducer** (§2.1–§2.2),
nor does c1xx fold before emitting — both counts are exactly `19 + 8×12` and
`19 + 12×8`. So §2.1b's headline is right and **its mechanism is not**.

*(A cross-dataset comparison suggests itself here and this lane declines it:
§6's brackets are measured at GRID-I's `/O1 /GS- /c`, the pair above at the
workload's `/GR /O1 /Oi /EHsc`. `w-sizebracket` §2.1c's own `/O1` bracket for
non-folding bodies is `(97,103]`, which agrees with §6's `[93,99]` to within
family — but two flag sets are two flag sets, and `P_INLINE` §2.1c's own rule
is that no single-profile size claim may be quoted at another profile.)*

### 2.5 `jl` is NOT accept, and over-ceiling is NOT refuse `[R]`

Read as control flow rather than as a sequence, `FUN_10b5fb5f`'s tail says
something every page in this tree gets wrong, including `P_INLINE` §2.1's own
2026-08-18 correction, which renders `0x10b5fc90` as *"below it => candidate"*.
**It is not.**

```
10b5fc7e:  cmp   ds:0x10c2e310,ebx        ; favour-speed (ebx = 0)
10b5fc84:  jne   0x10b5fcb9               ;   set -> skip the size test
10b5fc86:  movzx eax,WORD PTR [esi+0x50]  ; THE COUNT
10b5fc8a:  cmp   eax,ds:0x10c46318        ; the ceiling
10b5fc90:  jl    0x10b5fcb9               ; UNDER  -> not accept; a SECOND gate
10b5fc92:  mov   eax,DWORD PTR [esi+0x4c] ; OVER   -> not refuse either:
10b5fc95:  test  edi,eax                  ;   edi is a CALLER-SUPPLIED ATTR mask
10b5fc97:  jne   0x10b5fcb9               ;   and it reaches the same gate
...
10b5fcb9:  cmp   ds:0x10c2e2fc,ebx
10b5fcbf:  jne   0x10b5fcce               ; -> return 1
10b5fcc1:  test  DWORD PTR [esi+0x4c],0x2080
10b5fcc8:  jne   0x10b5fcce               ; -> return 1
10b5fcca:  xor   eax,eax                  ; -> return 0
```

> **`[R]` — the size test is NEITHER NECESSARY NOR SUFFICIENT.**
> **Under the ceiling still refuses** unless `DAT_10c2e2fc != 0` or
> `[sym+0x4c] & 0x2080`. **Over the ceiling still passes** when
> `[sym+0x4c] & edi` is non-zero, and `edi` is one of `FUN_10b5fb5f`'s five
> parameters — **a caller-supplied `ATTR` mask, i.e. a decision point c2 itself
> exposes as a parameter.**

`0x2080` is `__forceinline` (`0x2000`) **or bit 7**, and **bit 7 of the `.gl`
`ATTR` word is unread**. It is a front-end bit, which is where a "this body is
trivial" mark would live — **and §2.4 already rules it out as the explanation
of `P_INLINE` §2.1b's pair, because both of those cells carry `ATTR = 0x68`
and bit 7 is clear in both.** That is registered here deliberately: it was
this lane's first hypothesis, and its own data killed it inside an hour. The
bit is still worth reading; it is no longer worth reading *for that reason*.

**What §2.5 does establish is structural and does not depend on the pair:** the
size test is not the accept/refuse boundary anyone has been quoting, and one of
its two escapes is a **parameter**. **This lane does not settle where the
boundary actually is** — that needs the `edi` mask traced to `FUN_10b5fb5f`'s
three callers and `DAT_10c2e2fc`'s writer found — and the page it would amend
is `w-clausegen`'s this wave. Ranked in §8.

---

## 3. What the count is FOR — three consumers, and the unit falls out of them `[R]`

The nine WORD reads of `[sym+0x50]` partition into three groups. The third
group is new here and it is what settles the unit.

**(a) The inliner — five sites, all already clause-covered.**

| addr | owner | clause | what |
|---|---|---|---|
| `0x10b5fc86` | `FUN_10b5fb5f` | C8 | candidacy: `cmp eax, DAT_10c46318` |
| `0x10b60a6f` | `FUN_10b60930` | C17 | `cmp [ebp+0x10],eax` — budget vs count |
| `0x10b625b2` | `FUN_10b6242a` | C18 | `cmp eax,0x28` — the 40 test |
| `0x10b625bd` | `FUN_10b6242a` | C19 | the charge |
| `0x10b626f7` | `FUN_10b62675` | C2 | the seed |

**(b) Two other back-end readers.** `0x10b56732` in `FUN_10b566e9`
(`globregs.c`) and `0x10b8fbda` in `FUN_10b8fb47` (`mod.c`), both corroborating
on `+0x37`. Not read further by this lane; recorded so the field's consumer set
is not mis-stated as inliner-only.

**(c) TWO PROGRAM-WIDE ACCUMULATORS, and they are 64-BIT.** `list.c`:

```
10b72eca:  and  ds:0x10c46398,0x0          ; FUN_10b72eca — zero a 64-bit total
10b72ed1:  and  ds:0x10c4639c,0x0
10b72ed8:  mov  ecx,ds:0x10c4630c          ; the function list head
10b72ee0:  test BYTE PTR [ecx+0x4c],0x20   ; an ATTR bit selects the subset
10b72ee6:  movzx eax,WORD PTR [ecx+0x50]   ; <-- THE COUNT
10b72eea:  cdq
10b72eeb:  add  ds:0x10c46398,eax          ; 64-bit accumulate
10b72ef1:  adc  ds:0x10c4639c,edx
10b72ef7:  mov  ecx,DWORD PTR [ecx+0x78]   ; next
10b72efc:  jne  0x10b72ee0

10b72f0f:  movzx eax,WORD PTR [ecx+0x50]   ; FUN_10b72f0f — the same, into a
10b72f13:  cdq                             ; second 64-bit pair
10b72f14:  add  ds:0x10c2ebb8,eax
10b72f1a:  adc  ds:0x10c2ebbc,edx
```

`0x10c46398`/`0x10c4639c` and `0x10c2ebb8`/`0x10c2ebbc` are read together at
`0x10b72f21`–`0x10b72f2d` and handed to `0x10bec828`. The image's own
diagnostic vocabulary supplies the noun: **`" %I64u dynamic instrs"`**
(`0x10b131d8`, referenced at `0x10b724be`).

> **`[R]` — c2 sums `WORD [sym+0x50]` over every function in the compiland into
> a 64-bit total, and prints that class of total with the literal word
> `instrs`. The field is a per-function INSTRUCTION COUNT in c2's own
> vocabulary**, and it is summed at 64 bits precisely because a whole-program
> total does not fit in the 16 bits each function's field carries.

### 3.1 The unit is the FRONT END's, and the `%d instrs` diagnostic reads a DIFFERENT field `[R]`

**Registered prediction P2 said the `INL:\tInlining %s (%d instrs) into `
diagnostic formats `[sym+0x50]`. That half is WRONG and the miss is recorded
here.** At both call sites (`0x10ba1c79` and `0x10ba1d33`, in `FUN_10ba1c2d`)
the argument is:

```
10ba1d2b:  mov  ecx,DWORD PTR [edi]              ; edi -> the symbol
10ba1d2d:  mov  eax,DWORD PTR [ecx+0x80]         ; [sym+0x80] = the function body object
10ba1d33:  push DWORD PTR [eax+0x8e]             ; <-- the printed count, 32-bit
10ba1d41:  push 0x10b025ec                       ; "INL:\tInlining %s (%d instrs) into "
```

`[sym+0x80]` is the same pointer the inliner dereferences at `0x10b625f5`, and
`[edi]`'s identity is fixed two instructions later by `test DWORD PTR
[eax+0x4c],0x2000` at `0x10ba1d70` — the `__forceinline` bit, on the symbol.

**So c2 has TWO counts, and they are different objects:**

| | `WORD [sym+0x50]` | `DWORD [fnbody+0x8e]` |
|---|---|---|
| width | 16 bits | 32 bits |
| written by | `0x10b9bf6c`, the `.gl` reader — **the front end's number** | `0x10ba2335`, in `FUN_10ba1eca` (`p2symtab.c`) — **c2's own recount** |
| read by | the inliner's 5 sites, `globregs.c`, `mod.c`, the two 64-bit totals | the `%d instrs` diagnostics, `0x10ba2035`, `0x10ba3737`, `0x10bb6c31` |
| **the inline decision** | **this one** | never |

`FUN_10ba1eca` is where the recount lives, and it is worth naming because it is
the function a reader would otherwise assume feeds the inliner:

```
10ba2335:  mov  DWORD PTR [ebx+0x8e],eax        ; store c2's own count
10ba233b:  cmp  eax,0xffff
10ba2340:  jl   0x10ba238b
             -> "INF:\t%s won't be profiled (too big)\n"   (0x10b16a78)
10ba23a2:  cmp  eax,0x96                        ; 150
10ba23a7:  jle  0x10ba23cf
10ba23c5:  or   DWORD PTR [edi+0x94],0x100
             -> "INF:\t%s won't be inlined (too big)\n"    (0x10b16b60)
```

**`[sym+0x94] & 0x100` is tested at exactly two addresses in the image,
`0x10b9e5d8` and `0x10ba3b7b`, and NEITHER is inside the inliner band
`0x10b5b86d`–`0x10b62b00`** (checked by dumping every `+0x94]` operand in the
band — 43 of them, none with mask `0x100`). So *"won't be inlined (too big)"*
is **not** this inliner's decision; it is a separate, later gate, and
`FUN_10ba1eca` carries `only-from:pgo-client` in `FUNCS.tsv`.

> **`[R] + [I]` — the quantity the inline decision tests is produced by
> `c1xx`, transported in the `.gl` record's `SIZE` field, and read by c2
> without modification. c2's OWN instruction count exists, is 32-bit, is what
> the `%d instrs` diagnostic prints, and never reaches the inline decision.**
> The `[I]` step is "produced by `c1xx`": the read establishes that no c2 code
> writes the field, which leaves the IL's producer, but this lane did not open
> `c1xx.dll`.

`[O]` support, from `w-sizebracket`'s 105 cells and re-measured here: the field
is **linear in source statements** and c1xx does **not** fold before emitting
it — a 12-rung `s = s*K+C` chain reads 115 = 19 + 8×12, and an 8-rung
`s = (s*K+C)^(s>>j)` chain reads 115 = 19 + 12×8, to the unit.

---

## 4. The 16-bit ceiling, and where it actually bites `[R]`

The seed's `movzx` narrows nothing — the field is 16 bits at rest. The ceiling
is imposed **upstream, by the IL encoding**, and it has three forms, all in
`il-read-varint16` at `0x10c1f9a6` (decode reproduced in `P_INLINE` §2.1d):

| `.gl` byte | form | value delivered to the consumer's `movzx` |
|---|---|---|
| `0x00..0x7f` | direct | `0..127` |
| `0x80` | escape, two further LE bytes | `0..65535` |
| **`0x81..0xff`** | `movsx ax,dl` — **one SIGNED byte** | **`32,896..65,535`** |

The third row is the hazard and it is a behavioural one, not a curiosity:

* A caller whose `SIZE` byte lands in `0x81..0xff` seeds `DAT_10c3f5cc` with a
  value **above 35,000**, so **C16 (`0x10b60a63`) declines the very first
  site** and the caller inlines nothing at all.
* A callee in the same state reads as ≥ 32,896, so C8 refuses it and C19 would
  charge 32,896+ against a budget of at most 35,000.
* A function whose true count reaches 65,536 cannot be represented; what
  happens then is decided by **c1xx's encoder**, not by c2. This lane did not
  read c1xx and does not guess.

Measured incidence: `w-sizebracket`/`w-glattrs` found **99 escaped records in
28,838 workload records** and **zero** in the `0x81..0xff` form. The hazard is
real and unwitnessed on this corpus, which is the honest way to state it.

---

## 5. F7 — the interesting half

`WB_INLINE_FINDINGS` **F7**: *"the caller's own size is NOT an input. A 48-byte
caller and a 5,640-byte caller give identical verdicts at every size and both
flag sets"*, 12 cells. §4.1 explains the null as *"the D family moves the
caller from 48 B to 5,640 B, i.e. `B` from 1000 to ~2,820"*.

### 5.1 The published numbers are in the wrong unit, and the right ones are 3.5× larger `[O]`

`48` and `5,640` are the D cells' **emitted caller `.text` bytes** — they are
the literal `bytes` field of
[`grids/wb-inline/measured.json`](grids/wb-inline/measured.json). The step to
`B ≈ 2,820` divides that by two, i.e. it treats an "instruction" as 2 bytes.
Both moves are wrong: PPC instructions are 4 bytes, and **the tested quantity
is not machine instructions at all** (§3).

Re-measured in the read unit by rebuilding the D family from `grid.py`'s own
frozen generators and reading `WORD [sym+0x50]` out of the captured `.gl`
([`f7_units.py`](../../work/w-instrcount/f7_units.py); raw at
`work/w-instrcount/f7/f7_units.jsonl`):

| cell | caller src | **caller count** | **callee count** | `B` | C17 slack at the site | C16 total | C16 slack |
|---|---:|---:|---:|---:|---:|---:|---:|
| `D_*_k24_b0` | 495 B | **23** | 183 | **1,000** | 817 | 206 | 34,794 |
| `D_*_k24_b700` | 13,095 B | **4,923** | 183 | **9,846** | 9,663 | 5,106 | 29,894 |
| `D_*_k50_b0` | 911 B | 23 | 365 | 1,000 | 635 | 388 | 34,612 |
| `D_*_k50_b700` | 13,511 B | 4,923 | 365 | 9,846 | 9,481 | 5,288 | 29,712 |
| `D_*_k120_b0` | 2,051 B | 23 | 855 | 1,000 | 145 | 878 | 34,122 |
| `D_*_k120_b700` | 14,651 B | 4,923 | 855 | 9,846 | 8,991 | 5,778 | 29,222 |

Identical at `O1` and `O2`. **The measured `B` range is 1,000 → 9,846, not
1,000 → 2,820.** So the axis *was* varied, and P4a's clamp-floor hypothesis is
**FALSIFIED**: `2 × 4,923 = 9,846` is well clear of the 1,000 floor.

### 5.2 Why it still could not move — a theorem from the read, not a fit `[R]`

The caller count reaches exactly two predicates. Both were read this lane, in
full:

```
; FUN_10b60930, the accept/decline predicate
10b60a63:  cmp  DWORD PTR ds:0x10c3f5cc,0x88b8   ; C16: running total vs 35000
10b60a6d:  jg   0x10b609f3                       ;      decline
10b60a6f:  movzx eax,WORD PTR [edi+0x50]         ;      the CALLEE's count
10b60a73:  cmp  DWORD PTR [ebp+0x10],eax         ; C17: budget REMAINING vs count
10b60a76:  jge  0x10b60a81                       ;      affordable -> continue
10b60a78:  cmp  eax,0x28                         ;      40
10b60a7b:  ja   0x10b609f3                       ;      count > 40 -> decline

; FUN_10b6242a, the charge
10b625a6:  test DWORD PTR [esi+0x4c],0x2000      ; __forceinline bypasses BOTH
10b625b0:  jne  0x10b625c7
10b625b2:  movzx eax,WORD PTR [esi+0x50]
10b625b6:  cmp  eax,0x28
10b625b9:  jbe  0x10b625bd                       ; <= 40: NOT charged to the budget
10b625bb:  sub  DWORD PTR [edi],eax              ; *budget -= count
10b625bd:  movzx eax,WORD PTR [esi+0x50]
10b625c1:  add  DWORD PTR ds:0x10c3f5cc,eax      ; running total += count -- NOT
                                                 ;   gated by the 40 test (only by
                                                 ;   __forceinline, at 0x10b625a6)
```

> **THE FIRST-SITE THEOREM `[R]` — stated so it does not lean on §2.5's
> unsettled candidacy.** `B = clamp(2 × caller_count, 1000, 35000)`
> (`0x10b62708`–`0x10b6271e`), so **`B ≥ 1000` for every caller, including a
> caller of size zero**. At the first call site the budget is un-drained, so
> C17's `cmp [ebp+0x10],eax` compares at least 1000 against one callee's count.
> **Therefore C17 cannot decline the first site of any caller whose callee
> counts below 1000 — and the caller's own size only scales `B` UPWARD from
> that floor, so it cannot change the answer in either direction.**

**On the D family that is arithmetic on measured numbers, not an argument.**
The three callee counts are **183, 365 and 855** (§5.1), all below the floor
`B = 1000`, so C17 passes at the single site in all 12 cells at `B = 1000` and
equally at `B = 9,846`. The tightest cell, `k=120` at 855, still has **145** of
headroom. No assumption about candidacy is needed.

*(The looser general bound is worth having too, with its caveat: if candidacy
did refuse everything at or above `DAT_10c46318 = min(0x10 << k, 1000) ≤ 1024`
— `0x10b5e4cc`–`0x10b5e4e8`, `k = 3` in the image's raw `.data`, giving 128 —
then the first charged site of any caller is affordable **7.9×** over. §2.5
shows candidacy has an over-ceiling escape, so this bound is quoted as the
typical case and not as a theorem.)*

To make C17 bind at all, accumulated charges must drive the remaining budget
below a later callee's count. From `B = 1000` against callees at the typical
ceiling of 127 that is **at least ⌈(1000 − 127) / 127⌉ = 7** charged sites,
and up to 22 if the callees are small (a callee at 41 charges 41). **Every D
cell has exactly one call site.**

And C16 needs a running total above 35,000. The largest D cell reaches
**5,778** — **6.1× of slack**, and a caller would need ≈ 35,000 count units. The D family's own rate is
`4,923 / 700 = 7.03` counts per statement, so that is about **5,000**
statements of the `x += tbl[i];` form — **7.1× the largest cell the grid
contains**.

> **F7's answer: the D family varied the right variable and read it through two
> predicates that had 7.9× and 6.1× of slack. The null is a property of the
> grid's design — one call site per cell — and says nothing about whether the
> count is an input.** F7 should be read as *"one call site cannot reach the
> budget"*, which is a fact about c2 and a useful one; `P_INLINE` §3.1 and
> `WB_INLINE_FINDINGS` §4.1's stronger reading — *"the caller's own size is NOT
> an input"* — is not supported by these 12 cells.

### 5.3 What a grid that COULD reach the budget looks like

Directly from the theorem: **one caller, ≥ 8 call sites, each to a distinct
callee whose count is just under the ceiling**, swept against caller size. The
grid's own `C` family already varies `n ∈ {1,3,9}` and F6 records the shape a
budget would produce — *at `s = 212` static, `n = 1` inlines and `n = 3`,
`n = 9` decline* — so **the site-count effect the incumbent `INLINE-P` carries
as a fitted term may be the budget**. This lane does **not** claim that: the
arithmetic does not close (see §6), and `#3505` is what happens to a lane that
banks a mechanism because it is the nearest one. It is ranked in §7 as the next
read.

---

## 6. A contradiction this lane found and did NOT resolve

`WB_INLINE_FINDINGS` §4.2 records that the ceiling reading *"does not compose
into the measured numbers"*, comparing `16 << 3 = 128` **instructions** against
*"25–29 and 37–41 emitted words"*. That comparison is between two different
units and could never have closed. With §3's unit in hand it can be restated
properly, so this lane restated it — `ceiling_units.py`, rebuilding GRID-I's
two boundary pairs from `grid.py`'s frozen generators and reading each callee's
`.gl` `SIZE` `[O]`:

| family | linkage | `k` | callee `.gl` count | emitted `s` | frozen verdict |
|---|---|---:|---:|---:|---|
| A | static | 34 | 253 | 292 B | inlined |
| A | static | **35** | **260** | 300 B | **inlined** |
| A | static | **36** | **267** | 308 B | **called** |
| A | static | 37 | 274 | 324 B | called |
| B | extern | 10 | 85 | 92 B | inlined |
| B | extern | **11** | **92** | 100 B | **inlined** |
| B | extern | **12** | **99** | 116 B | **called** |
| B | extern | 13 | 106 | 124 B | called |

`0x10b5fc90` is `jl`, so **if** the size test is what moves along each ladder,
a candidate satisfies `count < DAT_10c46318` and the windows are

* **STATIC: the verdict flips in `count ∈ [261, 267]`**
* **EXTERNAL: the verdict flips in `count ∈ [93, 99]`**

**Neither contains any `0x10 << k`** — `256` misses the static window by 5 and
`128` misses the external one by 29 — and **no single value satisfies both**.
So, in the correct unit:

> **`[O]` — `DAT_10c46318` alone cannot be the boundary at both linkages, and
> `0x10 << k` is not the boundary at either.** §4.2's complaint survives the
> unit correction, with a much smaller and much more informative gap: the
> static window is **5 counts** above `16 << 4`, not "a reading that does not
> compose at all".

**Consistency check, free and worth printing.** The D family (§5.1) is the same
static `chain` generator at `k ∈ {24, 50, 120}`, counts **183 / 365 / 855**, and
its frozen verdicts are **inlined / called / called** at both `O1` and `O2`.
The static window `[261, 267]` predicts exactly that split — `183 < 261`,
`365 > 267`, `855 > 267` — on **12 cells the window was not fitted to**.

**The "if" in the first line is load-bearing and §2.5 is why.** These are
**measured verdict boundaries in the read unit**, which is a stronger and
narrower claim than "these bracket `DAT_10c46318`": attributing them to the
ceiling requires §2.5's second gate (`DAT_10c2e2fc`, `ATTR & 0x2080`, the
caller's `edi` mask) to be constant along each ladder, and nothing has shown
that. The brackets are published as what they are — the numbers any reading of
the ceiling or of the linkage arm now has to reproduce.

Two candidate explanations, neither taken: a **linkage arm** adjusts the
compared value or selects a different ceiling (`0x10b60a81`'s
`test DWORD PTR [edi+0x37],0x400`, on the linkage word, sits immediately after
C17 and is covered by no clause row), or the compared value is `count − d` for
a small `d ∈ [5, 11]`. **`P_INLINE` §5 already names the linkage arm as
unread**; this lane agrees and adds the bracket that would grade a reading of
it.

---

## 7. What this read unblocks — and what it does not

**The brief's framing is that `no-instr-count` is "ONE missing link, four
rows". It is one missing link, and the read closes it — but it was the BINDING
blocker on only two of the four.** Stated row by row, because overstating this
list costs a wave.

| clause | before | **after this read** | why |
|---|---|---|---|
| **C2** — seed `DAT_10c3f5cc = [fn+0x50]` | `absent`, `no-instr-count` | **UNBLOCKED — derivable today** | The producing field is the `.gl` `SIZE` the port **already decodes** (`crates/c2-il/src/func/gl.rs`, `GL_SIZE_ESCAPE_PAYLOAD`, `DISCLOSURE` **W-GLATTRS-1**) and then discards, which is C24's own note. Every field of a counterpart carries a `PROV[R]` address: the load `0x10b626f5`+`0x10b626f7`, the store `0x10b62703`, the producer `0x10b9bf6c`. |
| **C16** — decline when `35000 < DAT_10c3f5cc` | `absent`, `no-instr-count` | **UNBLOCKED — derivable today, and measured slack-bounded** | Both terms are now read: the seed (C2) and the `add` at `0x10b625c1`, which — unlike the budget subtract one instruction above it — is **not** gated by the 40 test, only by `__forceinline` at `0x10b625a6`. `CLAUSES.tsv` C19 states the two as one clause and records neither asymmetry. The threshold is an immediate at `0x10b60a63`. On this corpus the largest measured total is 5,778 against 35,000, so an adoption is **byte-neutral by construction**, like C15 — which is a reason to adopt it cheaply, not a reason to skip it. |
| **C17** — `budget < instrs && instrs > 0x28` | `absent`, `no-instr-count` | **BLOCKER REMOVED, STILL NOT ADOPTABLE** | Both operands are now derivable (`B` from C3, already `R-derived`; `instrs` from the same field). But `[ebp+0x10]` is the budget **threaded through the driver's recursion**, and the port has no driver to thread it through. C17's binding blocker moves from `no-instr-count` to whatever C4's is. Also proven unreachable at one call site (§5.2). |
| **C4** — driver entry `FUN_10b61ee1(fn,1,B,0,1e8,0)` | `absent`, `no-instr-count` | **NOT UNBLOCKED** | The budget **argument** is fully derivable now. C4's own note says the real absence: *"no depth/budget parameters exist to pass"* — there is no driver, no site collector and no per-site loop. That is `no-instr-stream`'s absence (C5/C6), not the count's. |
| **C20** — the expansion recurses into the driver | `fitted`, `no-instr-count` | **NOT UNBLOCKED** | What stands between `fitted` and `R-derived` for C20 is the **driver**, exactly as for C4. Removing the count from its blocker column would be honest; promoting the row would not. |

**Score: 2 of 4 rows unblocked in the strong sense (C2, C16), 1 blocker
removed without becoming adoptable (C17), 2 untouched (C4, C20).** The
`no-instr-count` label was accurate as a *label* on all four and was the
binding constraint on two — and the difference matters, because C4/C20's real
blocker is the same one C5/C6 already name, which means the four-row group is
really **two rows plus two more instances of `no-instr-stream`**.

*(Proposals only. `CLAUSES.tsv` and `P_INLINE.md` are `w-clausegen`'s this
wave; this lane edits neither, per `WAVE20_BRIEF` §4.)*

---

## 8. Found and not taken, ranked

1. **`FUN_10b5fb5f`'s SECOND gate — `DAT_10c2e2fc` and the caller-supplied
   `edi` mask** (§2.5). Highest-value item on the page, because it makes
   *"the size test decides candidacy"* false in **both** directions and because
   one escape is a **parameter** — which is exactly the shape
   `GOAL_DECISION_2026-08-21` § "AMENDED" says general layers should surface.
   Two small reads: `FUN_10b5fb5f`'s three callers for `edi`, and
   `DAT_10c2e2fc`'s writer. **It will NOT explain `P_INLINE` §2.1b's pair** —
   §2.4 rules that out on `ATTR = 0x68` in both cells — and a lane that takes
   this on should not expect it to.
2. **What separates `P_INLINE` §2.1b's pair, given that candidacy cannot.**
   Every identified input to `FUN_10b5fb5f` is identical across the two cells
   (§2.4), so the mechanism is downstream and no clause of the 24 is a
   candidate: C17 cannot bind at one site, C16 is 30× away, POGO is
   unreachable. **This is a genuinely open question that this lane narrowed
   rather than answered**, and narrowing it is most of the work: the search
   space is now "downstream of `0x10b5fc8a`, sensitive to whether the body
   folds, insensitive to the count and to `ATTR`."
3. **The linkage arm at `0x10b60a81`** (`test DWORD PTR [edi+0x37],0x400`, then
   `0x10b5de82`). It sits between C17 and the POGO call, is covered by no clause
   row, and §6's two brackets — static `[261,267]`, external `[93,99]` — are a
   ready-made grade for any reading of it. It is the last unread thing between
   the read ceiling and the measured boundaries.
4. **Does the budget explain F6's site-count effect?** §5.3's grid is 8–12
   cells and would either confirm C3/C17 for the first time (both are *READ,
   NOT CONFIRMED* today) or refute the budget as the mechanism behind
   `INLINE-P`'s fitted `n_sites` term. Cheap, and it is the only known route to
   an `[O]` on the budget.
5. **Close the 119 memcpy/memset call sites** (§2.2). Bounded, mechanical,
   and it is the one thing standing between this page's write census and a
   clean universal negative.
6. **`FUN_10ba1eca`'s recount and `[sym+0x94] & 0x100`.** A second, 32-bit
   instruction count with its own 150-instruction *"won't be inlined (too
   big)"* gate that this inliner does not consult. Two readers, `0x10b9e5d8`
   and `0x10ba3b7b`. Worth knowing where it *does* bite before anyone models
   c2's inlining end to end.

## 9. Prereg scorecard

| | prediction | outcome |
|---|---|---|
| **P1** | one initializing writer, at most one further | **HIT** — exactly one (`0x10b9bf6c`), zero further, with three blind-spot classes searched and one (119 memcpy sites) left open and named |
| **P2** | the unit is a front-end count, and the `%d instrs` diagnostic reads this field | **SPLIT: unit HIT, diagnostic MISS.** The diagnostic reads `[[sym+0x80]+0x8e]`, a different, 32-bit count. The unit claim survives on the two 64-bit `list.c` accumulators instead — better evidence than the one it replaced |
| **P3** | the field is 16-bit at rest, so the seed's `ushort` is a no-op | **HIT**, and the real ceiling located upstream in `il-read-varint16`, with the `0x81..0xff` hazard quantified |
| **P4a** | both F7 callers clamp to the budget floor | **FALSIFIED** — `B` measured 1,000 → 9,846. The sub-prediction that the published arithmetic was in the wrong unit is a **HIT** (it was emitted `.text` bytes ÷ 2) |
| **P4b** | the budget is unreachable on a one-site grid | **HIT**, and strengthened into the first-site theorem: ≥ 7 charged sites are needed before caller size can matter at all |
| **P5** | C2 unblocked · C4 not · C16 unblocked-as-unreachable · C17 partly | **HIT on all four rows**, with C16 sharper than predicted (derivable *and* slack-bounded) and C20 added as a second row the count does not unblock |
