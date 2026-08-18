# `P_INLINE` — `inline.c`: the inline decision function

> **Reference page.** **`[R]`** read from the disassembly, *not* obj-checked —
> a hypothesis. **`[O]`** confirmed against a real obj or `/FAsc` listing, with
> the witness named. **`[I]`** an interpretive step. Navigation only; nothing
> here may enter `crates/` without a [`DISCLOSURE.md`](../DISCLOSURE.md) row.
> Index: [`ADDR.tsv`](ADDR.tsv) · front door: [`README.md`](README.md)

**Coverage: 16 entries against a denominator of 93** — Ghidra functions in the
inliner band `0x10b5b86d`–`0x10b62b00` (`inline.c`'s anchor span plus the gaps
on both sides that hold the parameter tables and the legality check). Not
covered: `ptinl.c` entirely, the expansion's own body rewrite beyond its entry,
and both 46-dword POGO parameter tables (read, unreachable, deliberately not
quoted — §5).

> ### The headline, and it is a warning about fitted rules
>
> **`INLINE-P` — the project's incumbent predicate — is EXACTLY RIGHT inside
> the class it was fitted on and wrong outside it in two measured directions:
> a flag axis it does not have, and a LOOP axis nothing has ever had.** On the
> `keygen_xbox.cpp` anchor it predicts **six** inlines and gets **one** `[O]`.
> Its EXTERNAL clause `s ≤ 112` sits inside the measured `(100,116]`, and its
> STATIC cap `s ≥ 308` matches the measured `(300,308]` **to the word** — every
> one of its misses is at a flag set its corpus never contained.

---

## 1. The chain, top down

| addr | size | callers | callees | TU | cites | what |
|---|---:|---:|---:|---|---:|---|
| `0x10b62675` | 464 | 1 | 11 | `inline.c` gap | 4 | **the pass entry, per function.** Skipped wholesale when `DAT_10c40ec4 == 0` `[R]` |
| `0x10b626d8` | *(in `0x10b62675`)* | — | — | `inline.c` gap | 2 | `DAT_10c3f5cc = (ushort)[fn+0x50]` — the caller's **instruction count**, the running growth total `[R]` |
| `0x10b6276a` | *(in `0x10b62675`)* | — | — | `inline.c` gap | 1 | `FUN_10b61ee1(fn, level=1, budget=B, 0, 100000000, 0)` `[R]` |
| `0x10b61ee1` | 539 | 2 | 16 | `inline.c` gap | 3 | **the driver** — collects the sites, loops over them, returns *budget consumed* `[R]` |
| `0x10b600e6` | 1062 | 1 | 8 | `inline.c` gap | 4 | **the site collector.** One linear scan; instruction kind **`0x0f`** is a call site. Tracks EH-region nesting through opcodes `0x2ee/0x2f0/0x2f1/0x2f4/0x2f6/0x2ff/0x300` and stamps a conditional/EH flag into bit 1 of the candidate `[R]` |
| `0x10b5fb5f` | 377 | 3 | 5 | `inline.c` gap | 3 | **candidacy — where the size ceiling is.** §2 `[R]` |
| `0x10b5c06b` | 60 | 5 | 0 | `hash.c` gap | 5 | **legality.** Refuses on `[sym+0x20] & {0x400, 0x1000, 0x40, 0x100}` and `[sym+0x4c] & {0x80000, 0x200}`; requires bit 6 of `[sym+0x4c]` `[R]` |
| `0x10b61d2c` | 437 | 1 | 10 | `inline.c` gap | 1 | per-site driver `[R]` |
| `0x10b60930` | 358 | 1 | 7 | `inline.c` gap | 4 | **the accept/decline predicate** — depth, budget, POGO `[R]` |
| `0x10b6242a` | 587 | 1 | 6 | `inline.c` gap | 1 | **the charge**, and the second copy of the 40-instruction test `[R]` |
| `0x10b620fc` | 814 | 1 | 24 | `inline.c` gap | 1 | **the expansion**, recursing back into `0x10b61ee1` for the inlined body `[R]` |
| `0x10b5fcd8` | 1038 | 1 | 6 | `inline.c` gap | 7 | **the profitability model — POGO ONLY.** Reached from `0x10b60930` only when a profile record exists `[R]`. §5 |
| `0x10b600c8` | *(in `0x10b5fcd8`)* | — | — | `inline.c` gap | 3 | the per-site-count discount `cost -= (K + cost) / n_sites` `[R]` |
| `0x10b5e4cc` | 101 | 1 | 3 | **`inline.c` anchor** | 4 | **the ceiling itself**: `DAT_10c46318 = 0x10 << DAT_10c2ea98` (16 instructions << k), or `1000` when `k ≥ 7` `[R]` |
| `0x10b5e6a5` | 768 | 3 | 8 | **`inline.c` anchor** | 2 | the savings vector for the POGO model `[R]` |
| `0x10b5b86d` | 34 | 1 | 0 | `hash.c` gap | 3 | selects between the two 46-dword parameter tables (`DAT_10c45e18` / `DAT_10c45ed0`) `[R]` |

Diagnostics: `"INL:\tInlining %s (%d instrs) into "` at `0x10b025ec`, and the
`-optref` pruner's `"INF:\t%s not allowed to be inlined (globally
unreferenced)"` — **the quantity is an instruction count c2 holds before
codegen, not a byte count** `[R]`.

---

## 2. The decision function `[R]`

### 2.1 Candidacy — and the switch that turns the size test off

`0x10b5fb5f`, the arm that returns 1:

```
0x10b5fdfd   cmp DWORD [0x10c2e310], 0      <- THE FAVOR-SPEED BIT
                                                if non-zero the size test is SKIPPED
0x10b5fe0c   movzx eax, WORD [sym+0x50]     <- the callee's INSTRUCTION COUNT
0x10b5fe14   cmp eax, DWORD [0x10c46318]    <- the ceiling; `jl` = candidate
0x10b5fe1e   test DWORD [sym+0x4c], 0x2000  <- __forceinline: bypass
```

`0x10c2e310` is the same option-word bit 23 (written at `0x10b8238d`) that moves
`memcpy`'s inline threshold — **two mechanisms now shown to hang off one bit**.
This is why no grid compiled at a single flag set could ever see the ceiling
move.

> ### ⛔ CORRECTION 2026-08-18 (lane `w-sizebracket`) — the SEQUENCE above is right and ALL FOUR ADDRESSES ARE WRONG. They are in a different function.
>
> `FUN_10b5fb5f` has size **377**, so it spans `0x10b5fb5f`–`0x10b5fcd7`.
> **Every address in the block above is past its end** and lands in
> `FUN_10b5fcd8` — §1's *"profitability model — POGO ONLY"*. The bytes actually
> at those addresses are a different computation:
>
> ```
> 10b5fdfd:  0f b6 c9              movzx  ecx,cl
> 10b5fe0e:  0f af 15 8c f5 c3 10  imul   edx,DWORD PTR ds:0x10c3f58c
> 10b5fe15:  0f af 05 7c f5 c3 10  imul   eax,DWORD PTR ds:0x10c3f57c
> ```
>
> The **real** candidacy test, inside `FUN_10b5fb5f` and reading the same two
> operands the block above names, is at `0x10b5fc7e`:
>
> ```
> 10b5fc7e:  39 1d 10 e3 c2 10     cmp    DWORD PTR ds:0x10c2e310,ebx   <- FAVOUR-SPEED (ebx = 0)
> 10b5fc84:  75 33                 jne    0x10b5fcb9                    <- set => size test SKIPPED
> 10b5fc86:  0f b7 46 50           movzx  eax,WORD PTR [esi+0x50]       <- the callee's count
> 10b5fc8a:  3b 05 18 63 c4 10     cmp    eax,DWORD PTR ds:0x10c46318   <- the ceiling
> 10b5fc90:  7c 27                 jl     0x10b5fcb9                    <- below it => candidate
> 10b5fc92:  8b 46 4c              mov    eax,DWORD PTR [esi+0x4c]
> 10b5fcc1:  f7 46 4c 80 20 00 00  test   DWORD PTR [esi+0x4c],0x2080
> ```
>
> Two things carry forward and one does not. **The reading carries**: the
> favour-speed bit does gate the size test, the tested operand is
> `WORD [sym+0x50]`, and the ceiling is `DAT_10c46318`. **The `0x2000`
> `__forceinline` mask does not carry as written** — the mask at `0x10b5fcc1` is
> `0x2080`, and the test at `0x10b5fc92` is against a mask held in `edi` rather
> than an immediate. §2.3's `0x10b609d3` is a separate, genuine `0x2000` test and
> is unaffected.
>
> This is `README.md` §6.2's lesson in a second place: *"address A is inside
> function F"* is a claim to check against `F`'s entry **and size**, which
> `ADDR.tsv` prints. Amended beside, per §2.1 of the front door; the original
> block is left exactly as it was written.

### 2.1a Where `[sym+0x50]` COMES FROM — it is the `.gl` record's `SIZE` field, read verbatim `[O]`

Lane `w-sizebracket`, 2026-08-18. §1 read the quantity as *"an instruction count
c2 holds before codegen"* from the `INL:\t...(%d instrs)` diagnostic. Its
**origin** is now located, and it is a field of the IL.

**There is exactly ONE 16-bit store to `[reg+0x50]` in the whole image:**

```
10b9bf57:  e8 8d 3a 08 00        call   0x10c1f9e9      (i32c)  -> [esi+0x54]
10b9bf5f:  e8 85 3a 08 00        call   0x10c1f9e9      (i32c)  -> [esi+0x58]
10b9bf67:  e8 3a 3a 08 00        call   0x10c1f9a6      (i16c)
10b9bf6c:  66 89 46 50           mov    WORD PTR [esi+0x50],ax   <-- THE ONLY ONE
10b9bf70:  e8 a6 39 08 00        call   0x10c1f91b
10b9bf78:  89 46 4c              mov    DWORD PTR [esi+0x4c],eax
10b9bf7b:  e8 26 3a 08 00        call   0x10c1f9a6      (i16c)  -> WORD [esi+0x52]
```

It is inside `FUN_10b9b8e9`, which [`ADDR.tsv`](ADDR.tsv) already labels
*"p2symtab `.gl` record reader; reads the emit flag word `+0x4c` at
`0x10b9bf70`"* — the same three instructions, from the other side. `0x10c1f9a6`
is `il-read-varint16` (*"i16c, `0x80` escape"*, 1-or-3 byte) and `0x10c1f9e9` is
`il-read-varint32`.

**And the field order matches the port's own `.gl` reader exactly.**
`c2_il::func::gl::gl_function_attrs`' record comment reads

```text
  00 <name> 00 <TYPE> 80 01 10 00 00 00 00 80 <LE32 offset> <SRCPOS> <SIZE> <ATTR>
```

with `SRCPOS` *"a byte under 0x80, or the escape `80 <LE32>`"* and `SIZE` *"a
byte under 0x80"*. Lined up against the reader above:

| `.gl` field | reader | destination |
|---|---|---|
| `80 <LE32 offset>` | `i32c` `0x10b9bf57` | `[sym+0x54]` |
| `SRCPOS` | `i32c` `0x10b9bf5f` | `[sym+0x58]` |
| **`SIZE`** | **`i16c` `0x10b9bf67`** | **`WORD [sym+0x50]`** |
| `ATTR` | `0x10b9bf70` | `[sym+0x4c]` |

> **So `[sym+0x50]` is the `.gl` function record's `SIZE` field, arriving
> verbatim from the IL — the field the port already walks past to reach the
> attribute byte, and throws away.** `[sym+0x4c]`, the field one step later in
> the same decode, is the one the port reads as `FN_FLAG_INLINABLE` and the one
> §1's legality check `0x10b5c06b` tests. The two are neighbours in one record.

`[O]`: decoded out of real `.gl` bytes by `work/w-sizebracket/glsize.py` and
confirmed **linear in source content** — an empty `int f(int)` is 19, and each
added statement is a fixed increment (`s ^= a;` +4, `s = -s;` +5, `s = s << 3;`
+6, `s = s*3+1;` +8, `if (s>3) s=1;` +13). 105 cells, `/O1` and `/Ox`.

**The `0x80` escape is live on real code and the port refuses it.**
`gl_function_attrs` returns `None` for the **whole file** when the `SIZE` byte is
`>= 0x80`, and `SIZE` crosses 128 at ~14 statements — cell `arith_016` reaches
147. Whatever else follows from this section, that refusal is a measured, live
limit of the port's attribute map rather than a theoretical one.

### 2.1b …and the `.gl` `SIZE` field is NOT the value the decision tests `[O]`

Same lane, and it is the finding that matters most. `[sym+0x50]` is
**initialized** from `SIZE` and is then **reduced by whatever runs before the
inliner**, so `SIZE` is an *upper bound* on the tested quantity and not the
quantity.

The witness is a matched pair at the workload profile
(`/nologo /c /GR /O1 /Oi /EHsc`), graded by whether the caller's `.text` COMDAT
carries a `REL24` naming the callee:

| cell | `.gl` `SIZE` | `.ex` bytes | emitted `.text` | real c2 |
|---|---:|---:|---:|---|
| `arith_012_O1` — 12 × `s = s*K+C;` | **115** | 3,233 | **28** | **inlined** |
| `mix_008_O1` — 8 × `s = (s*K+C)^(s>>j);` | **115** | 3,221 | **132** | **kept** |

**Identical `SIZE`, opposite verdicts.** The `arith` chain composes to a single
affine function, so c2 folds it before the inliner looks; the `mix` chain does
not. Extended: `arith` is **inlined at every rung to `SIZE = 211`** at both
`/O1` and `/Ox`, while `mix` is kept from `SIZE = 103` at `/O1`.

**The one-sided implication survives and is the usable form:** folding only
*reduces* the count, so

> `.gl SIZE < T` ⇒ the tested count `< T` ⇒ **c2 inlined it**

holds with **zero counterexamples in 105 probe cells** at `T = 98` (`/O1`) and
`T = 122` (`/Ox`). The converse — *"`SIZE` large ⇒ c2 kept it"* — is false, and
`arith` is 32 cells of it.

### 2.1c The brackets, this lane's, beside §3's `[O]`

Straight-line EXTERNAL callee, one call site, callee non-`inline`:

| profile | bracket in emitted `.text` | bracket in `.gl` `SIZE` | families |
|---|---|---|---|
| `/O1` (the workload's) | **(108, 116]** | (97, 103] on non-folding bodies | `mix`, `fine` |
| `/Ox` | **not separating** — 320 B inlined beside 196 B kept | (121, 127] on non-folding bodies | `mix`, `fine` |

**`/O1`'s (108, 116] sits inside §3's F2 `(100, 116]` and shares its top**, which
is an independent reproduction of that row on new cells four months of lanes
later.

**`/Ox` is where the units swap and it is the reason this section exists.** At
`/O1` the emitted `.text` separates and `SIZE` does not; at `/Ox` neither does,
and the emitted size is *anti*-correlated at the crossing — `fine_005_Ox` emits
**320 B** and is inlined, `fine_006_Ox` emits **196 B** and is kept. Consistent
with §2.1's favour-speed bit turning this very test off, and with `/Ox`'s growth
transforms running *after* the decision, so the emitted body is no longer a
witness to what the inliner saw. **No single-profile size claim from this page
may be quoted at the other profile.**

### 2.1d The `SIZE` field's `0x80` escape, DECODED and shipped `[O] port`

Lane `w-glattrs`, 2026-08-18, board **#3289**–**#3293**. §2.1a's closing
paragraph — *"the `0x80` escape is live on real code and the port refuses it"* —
is **discharged**. The escape is a **length escape with a two-byte
little-endian payload**: the byte `0x80` *exactly* introduces two further
bytes, so an escaped `SIZE` field is three bytes wide and `ATTR` sits three past
the `0x80`, not one.

**The reader, `[R]`, and it is why the two neighbouring fields escape at
different widths.** `il-read-varint16` at `0x10c1f9a6` — the one §2.1a already
names — reads one byte, compares it against `0x80`, and on equality reads
**exactly two** further bytes:

```
10c1f9b1:  8a 11              mov    dl,BYTE PTR [ecx]     ; the byte
10c1f9ba:  80 fa 80           cmp    dl,0x80
10c1f9bd:  74 06              je     0x10c1f9c5            ; escape
10c1f9bf:  66 0f be c2        movsx  ax,dl                 ; else ONE SIGNED byte
10c1f9c5:  ...                                             ; two more bytes ->
10c1f9d8:  88 55 fc           mov    BYTE PTR [ebp-0x4],dl  ;   low
10c1f9e0:  88 55 fd           mov    BYTE PTR [ebp-0x3],dl  ;   high
10c1f9e3:  66 8b 45 fc        mov    ax,WORD PTR [ebp-0x4]  ; LE16
```

`il-read-varint32` at `0x10c1f9e9` is the **identical shape with a four-byte
payload** (`0x10c1fa07`…`0x10c1fa43`). So `SRCPOS`, read by the 32-bit reader,
escapes to five bytes, and `SIZE`, read by the 16-bit one, escapes to three.
**The port's incumbent code stepped over both as if there were one width**, and
refused rather than guess — correctly, because at the wrong displacement the
attribute is an unrelated byte, and an unrelated byte with bit 6 set is a
*permission* the splice acts on.

**And `0x81..=0xff` is a THIRD form, not part of the escape**: `movsx ax,dl` —
one byte, sign-extended, so `0xff` reaches the consumer's `movzx` as 65,535.
The port still refuses it (zero witnesses in 28,838 workload records).

**The black-box confirmation, `[O]`, which is the half that decides.** A
twin grid of 18 cells over two profiles: each pair is two sources differing only
by `__declspec(noinline) ` versus 21 spaces, so byte-length-identical and
compiled from one path. `__declspec(noinline)` clears exactly this record's
`FN_FLAG_INLINABLE`, so the twins' `.gl` must differ at `ATTR` and nowhere else
structural.

| `SIZE` | form | `ATTR` offset | offset − (`p` + 5) | first differing byte past the source hash | XOR |
|---:|---|---:|---:|---:|---:|
| 55, 79, 103, 127 | direct | 236 | **2** | 236 | `0x40` |
| 139, 163, 211, 259, 379 | **escape** | 238 | **4** | 238 | `0x40` |

**18 of 18**, at `/O1` and `/Ox` alike. The escape moves `ATTR` by exactly two,
measured with no disassembly in the loop.

**The payload's endianness, also black box.** The probe family steps `SIZE` by
12 per statement and the ladder does not break at the boundary:
103 → 127 → **139** → 163 → 211 → 259 → 379, every rung equal to `19 + 12n`.
Read big-endian the first escaped rung would be **35,584**.

**And the workload's own records agree.** The 28,739 direct-form records
establish an `ATTR` vocabulary of ten bytes independently
(`c8 4c cc 68 48 e8 28 6c 69 88`). Scored on the 99 escaped records:

| assumed escape width | `ATTR` lands in the vocabulary |
|---:|---:|
| 1 | 3 / 99 |
| 2 | 0 / 99 |
| **3 — shipped** | **99 / 99** |
| 5 (`SRCPOS`'s width) | 1 / 99 |

Background rate for a `.gl` byte to fall in that vocabulary: **5.9 %**.

**One field further on, and this is new here: `ATTR` is not a byte.** It is a
two-or-four-byte little-endian value with a continuation flag in bit 15, read by
`0x10c1f91b`. `__declspec(noinline)` takes it from `0x1068` to `0x801028`, which
crosses `0x8000` — so the record grows from two bytes to four, and **that is the
mechanism behind `w-target`'s nicmp2 observation that "only `.gl` moves, and by
2 bytes"**. The port reads the low byte, which is correct for bit 6 under both
widths and is documented as such in `gl_function_attrs`.

`[O] port`: shipped in `crates/c2-il/src/func/gl.rs`
(`GL_SIZE_ESCAPE_PAYLOAD`), `DISCLOSURE.md` row **W-GLATTRS-1**, at
`mismatch 0`.

### 2.2 The budget — `B = clamp(2 × caller_instrs, 1000, 35000)`

```
0x10b626f4   uVar7 = 1000
0x10b626fb   if (2*caller_instrs > 1000) uVar7 = 2*caller_instrs
0x10b62708   if (uVar7 > 34999)          uVar7 = 35000

0x10b6249b   cmp WORD [callee+0x50], 0x28      <- 40 instructions
0x10b624a2   *budget -= WORD [callee+0x50]     <- charged only if > 40
0x10b60a04   if (budget < instrs && instrs > 0x28) return DECLINE
```

> **A callee of 40 instructions or fewer is never charged against the budget and
> is never declined for affordability.** The budget is a growth cap for *large*
> callees only. `[R]`

### 2.3 Depth and the categorical arms `[R]`

```
0x10b609ae   0x10 < level - DAT_10c3f50c            -> decline   (16 levels)
0x10b609bd   maxlevel != 0xff && maxlevel < level   -> decline
0x10b609d3   test [sym+0x4c], 0x2000                -> __forceinline bypasses
                                                       every size and budget test
0x10b609ee   35000 < DAT_10c3f5cc                   -> decline
```

---

## 3. The measured boundaries `[O]` — GRID-I, 264 frozen cells

`s` = the callee's own emitted `.text`, measured. The bracket is
*(last inlined, first called]*.

| family | `/O1` | `/O2` | `/O1 /Ot` | `/O2 /Os` | `/O1 /Ob0` |
|---|---|---|---|---|---|
| **STATIC**, straight-line | **(300, 308]** | **(212, 252]** | **(212, 252]** | **(300, 308]** | nothing inlines |
| **EXTERNAL**, straight-line | **(100, 116]** | **(156, 164]** | — | — | — |
| **loop-bodied** (GRID-J, 56 cells) | **(56, 80]** | — | — | — | — |

> **The threshold follows FAVOR-SPEED, not the `/O<n>` level.** `/O1 /Ot`
> behaves as `/O2`; `/O2 /Os` behaves as `/O1`. Same two mixed cells that
> decided `wb-memcpy`'s GRID-W.

Facts, each a statement about c2 rather than about a rival:

| # | fact | cells |
|---|---|---:|
| F3 | `/Ob0` declines **everything**, including `__forceinline` | 34 |
| F4 | `__forceinline` inlines a **980-byte** callee, at `/O1` and `/O2` | 2 |
| F5 | varargs and direct recursion decline categorically at every flag set | 6 |
| F7 | **the caller's own size is NOT an input** — a 48-byte caller and a 5 640-byte caller give identical verdicts at every size and both flag sets | 12 |
| F8 | a **control-dependent** site at `s = 212` declines at `/O1` where the unguarded one inlines; at `/O2` it does not | 6 |
| F9 | a **loop-bodied** callee declines at `(56,80]` where a straight-line one inlines to `(96,120]`, **identically at the workload flags and at `/O1 /GS- /c`** — so it is the loop and not the flags | 56 |

**No rival survives.** Scores: R2-CEILING 226/264, R1-INCUMBENT 218/264,
R5-NOSITES 195, R3-SIZE64 168, R4-OBLEVEL 144. R2's 38 misses are all
*parameter* errors; R1's 46 are all *structural* — it has no flag axis at all.

### 3.1 F7 refutes this page's own §2.2 as a *practical* input

The budget is read correctly — the instructions are there — but moving the
caller from 48 B to 5 640 B (i.e. `B` from 1 000 to ~2 820) changes **nothing on
12 cells** `[O]`. Consistent with §2.2 (everything at `k ≤ 40` is free of the
budget, everything above is already refused by the ceiling), and it means the
budget is **not reachable from the flag/size space anyone has swept**. Recorded
as **READ, NOT CONFIRMED**; no `DISCLOSURE` row proposes it.

---

## 4. `?supershuffle` — the clause that actually fires

On the real `keygen_xbox.cpp` obj the six shuffles are **104 / 60 / 84 / 84 /
88 / 88** bytes and **only the 60-byte one is inlined** `[O]`. All six are under
`INLINE-P`'s published 112-byte EXTERNAL ceiling.

> **The clause is the LOOP-CLASS size ceiling at `(60,80]`** — `cmp [sym+0x50],
> [0x10c46318]` at `0x10b5fe14`. `?shuffle2` is 60 B; the next smallest is 84 B.
> That is why it, and nothing else in the TU.

The registered prediction said the clause was the EXTERNAL `index ≤ 64` arm.
**The first half is wrong** — the real arm is three times tighter — and the
second half is right for a reason the prediction did not have. Scored a miss.

---

## 5. What is NOT known here — and one thing deliberately not quoted

* **The POGO cost model (`0x10b5fcd8`) is unreachable on this workload.** It is
  a full cost/benefit model with ~20 tunable weights copied from one of two
  **46-dword parameter tables** (`DAT_10c45e18` / `DAT_10c45ed0`, selected at
  `0x10b5b86d`). Those tables live above the image's raw `.data`
  (`0x10c3cc00`), so they are **zero at load and written at run time** — none of
  their values is quotable from the image and **this page does not quote them**.
  `0x10b60930` reaches the model only when the callee has a profile record.

  > **This is `C2_MAP_METHOD.md` §7 case 1 in advance: the most model-like code
  > in the inliner is not the code the workload takes**, and a lane that read it
  > and stopped would have published a cost model c2 never runs.

* **The `16 << k` ceiling does not compose into the measured numbers.**
  `0x10c2ea98`'s image value is `3`, giving `16 << 3 = 128` **instructions**;
  the measured straight-line ceilings are 25–29 and 37–41 emitted words
  (EXTERNAL) and 53–65 / 75–77 (STATIC). The reading is **named and not claimed
  as the boundary the workload takes** — something between the two, most
  plausibly the linkage arm and the `[sym+0x50]`-vs-emitted-size gap, is unread.
* The **depth cap of 16** (`0x10b609ae`): no cell nests 16 deep.
* `ptinl.c`: not opened.
* 320 cells is not a total statement about c2.
