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

---

## 6. THE INLINER'S SCOREBOARD — clause-by-clause conformance, and the 4-tuple `[R]`/`[O]`

> **Added 2026-08-26 by lane `w-inlmetric`** (decision 15, the owner's named
> exemplar: *"inliner is extremely valuable to understanding how that logic
> works in the compiler"*). **Amend-beside**: §1–§5 are unchanged, including
> §2.1's struck block and its correction.
>
> **This is a PROGRESS INSTRUMENT and never a gate** (`FUNCTION_BYTE_MATCH.md`
> §0). It licenses no emit, moves no admitted set, and adopts nothing into
> `crates/`. The lane wrote **zero `crates/` bytes**.

### 6.0 The coverage line at the top of this page mixes two units

> **`Coverage: 16 entries against a denominator of 93`** — re-measured on this
> tree against [`FUNCS.tsv`](FUNCS.tsv) over the band
> `0x10b5b86d`–`0x10b62b00`, which `SUBSYS.md` §1's inliner row publishes as
> `16 / 93`:

| quantity | measured | note |
|---|---:|---|
| functions in the band | **93** | replicates the published denominator **exactly** |
| …attributed to `inline.c` / `hash.c` | 61 / 32 | the band spans both, as §1 says |
| …carrying `page = P_INLINE.md` | 63 | the band is *assigned* to this page |
| **…with `cover = paged`** | **13** | **the functions this page actually reads** |
| …with `cover = cited` | 4 | named in passing, not read |
| **§1 table rows** | **16** | 13 functions **+ 3 addresses interior to another row** |

**The 16 and the 93 are not the same unit.** Three of §1's sixteen rows are
sub-addresses of a row already present — `0x10b626d8` and `0x10b6276a` inside
`0x10b62675`, and `0x10b600c8` inside `0x10b5fcd8` — and the page marks each
*(in `0x…`)* itself. So `16 / 93` reads as 17.2 % and the **function** coverage
is **13 / 93 = 14.0 %**, or 17/93 = 18.3 % if `cited` counts as covered.

> **Nothing is wrong with the 16**; what is wrong is reading it against 93 as
> though it were a rate. The three readings are published together above so the
> next lane picks one deliberately. `work/w-inlmetric/band_count.txt`.

### 6.1 The conformance table — 24 clauses

Machine-checked source: [`work/w-inlmetric/CLAUSES.tsv`](../../../work/w-inlmetric/CLAUSES.tsv).
Grader: `work/w-inlmetric/check_table.py`, **watched failing on three planted
verdicts before this table's green was quoted**
(`work/w-inlmetric/POSITIVE_CONTROL.md`). Every address below is verified inside
the function named, mechanically, against `FUNCS.tsv`'s entry+size.

**State** — `[R]`-derived: the port's counterpart comes from the same field c2
tests, with a `DISCLOSURE`-grade trail · `fitted`: a counterpart exists and is a
black-box fit, not a reading of this clause · `absent`: no counterpart in
`crates/`, and the named token is **verified absent** rather than assumed ·
`unexercisable`: no compilation this project runs reaches the clause, so
`absent` would mis-read it. **Ties break toward `absent`** (`PREREG.md` §5).

| # | clause | addr | state | witness | exercised by the workload |
|---|---|---|---|---|---|
| C1 | pass entry per function; skipped wholesale when `DAT_10c40ec4 == 0` | `0x10b62675` | **absent** | — | yes |
| C2 | caller instruction count seeded, `DAT_10c3f5cc = (ushort)[fn+0x50]` | `0x10b626d8` | **absent** | — | not separable (F7) |
| C3 | growth budget `B = clamp(2 × caller_instrs, 1000, 35000)` | `0x10b626f4` | **absent** | — | not separable (F7) |
| C4 | driver entry `FUN_10b61ee1(fn, level=1, budget=B, 0, 1e8, 0)` | `0x10b6276a` | **absent** | — | not separable |
| C5 | site collector: one linear scan, instruction kind `0x0f` is a call site | `0x10b600e6` | **absent** | — | yes |
| C6 | site collector: EH-region nesting, conditional/EH flag into bit 1 | `0x10b600e6` | **absent** | — | yes (F8, 6 cells) |
| C7 | ceiling **value**: `DAT_10c46318 = 0x10 << DAT_10c2ea98`, or `1000` at `k ≥ 7` | `0x10b5e4d7` | **absent** | — | yes |
| C8 | candidacy **size test**: `cmp WORD [sym+0x50], DAT_10c46318`; `jl` = candidate | `0x10b5fc8a` | **fitted** | `splice.rs:INLINE_UNBOUNDED_BYTES` | yes |
| C9 | favour-speed bit `0x10c2e310` non-zero ⇒ **the size test is SKIPPED** | `0x10b5fc7e` | **absent** | — | **no** — `/O1` pins the bit |
| C10 | `__forceinline`: `test [sym+0x4c], 0x2000` bypasses every size and budget test | `0x10b609d3` | **absent** | — | yes (F4, 2 cells) |
| C11 | legality: refuse on `[sym+0x20] & {0x400, 0x1000, 0x40, 0x100}` | `0x10b5c06b` | **absent** | — | **no** |
| C12 | legality: refuse on `[sym+0x4c] & {0x80000, 0x200}` | `0x10b5c06b` | **absent** | — | **no** |
| C13 | legality: **REQUIRE bit 6 of `[sym+0x4c]`** | `0x10b5c06b` | **`[R]`-derived** | `gl.rs:FN_FLAG_INLINABLE` (`0x40`) | yes |
| C14 | depth cap: `0x10 < level - DAT_10c3f50c` ⇒ decline (16 levels) | `0x10b609ae` | **absent** | — | **no** — no cell nests 16 deep |
| C15 | `maxlevel != 0xff && maxlevel < level` ⇒ decline | `0x10b609bd` | **absent** | — | **no** — `#pragma inline_depth` in 0/100 TUs |
| C16 | caller-huge decline: `35000 < DAT_10c3f5cc` | `0x10b609ee` | **absent** | — | **no** |
| C17 | budget accept/decline: `budget < instrs && instrs > 0x28` | `0x10b60a04` | **absent** | — | not separable (F7) |
| C18 | the 40-instruction test, **second copy** | `0x10b6249b` | **absent** | — | not separable |
| C19 | the charge: `*budget -= WORD[callee+0x50]`, and the growth total | `0x10b624a2` | **absent** | — | not separable |
| C20 | the expansion **recurses back into the driver** for the inlined body | `0x10b620fc` | **fitted** | `splice.rs:S6-chain` | yes (#1020, 150 witnesses) |
| C21 | POGO profitability model, entered only on a profile record | `0x10b5fcd8` | **unexercisable** | — | unexercisable |
| C22 | POGO per-site discount `cost -= (K + cost) / n_sites` | `0x10b600c8` | **unexercisable** | — | unexercisable |
| C23 | parameter-table selection, `DAT_10c45e18` / `DAT_10c45ed0` | `0x10b5b86d` | **unexercisable** | — | unexercisable |
| C24 | the tested quantity `WORD [sym+0x50]` **is the `.gl` `SIZE` field**, verbatim | `0x10b9bf6c` | **`[R]`-derived** | `gl.rs:GL_SIZE_ESCAPE_PAYLOAD` (`W-GLATTRS-1`) | yes (99 escaped records) |

**Per-state split: `[R]`-derived 2 · fitted 2 · absent 17 · unexercisable 3.**
**Exercised: yes 9 · no 6 · not separable 6 · unexercisable 3.**

### 6.2 What the table says that a percentage would not

1. **Seventeen of twenty-four clauses have no counterpart in the port, and the
   absence is VERIFIED rather than assumed.** Each `absent` row names a token
   the grader confirms is not in `crates/`. That is the whole point of the
   w-root mitigation pattern: an absent clause is now **visible**, where before
   it was inferred from the port not mentioning it.

2. **The two `fitted` rows are fitted to a DIFFERENT QUANTITY than c2 tests.**
   c2's C8 compares a **pre-codegen instruction count** (`WORD [sym+0x50]`,
   and the diagnostic string is literally `"%d instrs"`); the port's three
   ceilings — `INLINE_UNBOUNDED_BYTES = 64`, `INLINE_DECLINE_BYTES = 128`,
   `INLINE_DECLINE_LOOP_BYTES = 80` — are **lowered byte counts**, every one of
   them fitted to an obj bracket. §5's *"`16 << k` does not compose into the
   measured numbers"* and §2.1b's *"`SIZE` is an upper bound on the tested
   quantity and not the quantity"* are two views of that same gap, and the gap
   is why `INLINE_DECLINE_LOOP_BYTES` has to exist at all: a loop body priced
   in emitted bytes is over-credited by ≈ 1.55 (§5 / F9).

3. **C24 is the sharpest row on the page.** The port already **decodes the
   field c2's decision tests** — `GL_SIZE_ESCAPE_PAYLOAD` shipped it, at
   `mismatch 0`, with a `DISCLOSURE` row — **and then discards the value.**
   §2.1a's *"the field the port already walks past to reach the attribute byte,
   and throws away"* is a live statement about `crates/` today. **This is not a
   recommendation to consult it**: §2.1b measured `SIZE` as an *upper bound* on
   the tested quantity, with `arith_012` and `mix_008` at an identical `SIZE`
   of 115 and opposite verdicts. Consuming it would be adopting a bound as
   though it were the quantity.

4. **C13 is the one clause where two independent derivations MET.**
   `WB_INLINE_FINDINGS` §1 read c2's legality test at `0x10b5c06b` as
   *"requires bit 6 of `[sym+0x4c]`"* off the disassembly; `w-mmioclose`
   located `__declspec(noinline)` in the `.gl` as bit 6 / `0x40` from the
   container side, 9-of-9 and 11-of-11, and closed a **shipped wrong emit**
   (`w10`, #2402). `gl.rs`'s own comment records the meeting. It is the only
   row on this table with that property.

5. **`__forceinline` is the biggest asymmetry, and it is directional.** c2's
   C10 is an **accept** clause that bypasses every size and budget test — F4
   inlines a **980-byte** callee. The port has no accept path anywhere: it
   reads the record's `0x20` flags byte only to **keep** a wholesale refusal.
   §7's *"the accept side is not offered"* is therefore not a policy the port
   adopted, it is the port's entire relationship with this subsystem.

### 6.3 The 4-tuple, instantiated (decision 15's metric shape)

Every strength with its denominator, and the tree it was taken on.

| # | strength | value | denominator, re-measured here |
|---|---|---|---|
| 1 | **read** | **13 / 93 = 14.0 %** functions read (`16` §1 entries; `17/93` if `cited` counts) | `FUNCS.tsv`, band `0x10b5b86d`–`0x10b62b00`, this tree |
| 2 | **agreement** | **4 / 24 clauses have any port counterpart** (2 `[R]`-derived + 2 fitted); `INLINE-P` **0.9678** on the re-frozen hold-out | 24 clauses (§6.1); **8,936** graded callees (§6.4) |
| 3 | **exercised** | **9 / 24 yes · 6 no · 6 not separable · 3 unexercisable** | the 100-TU hold-out's 8,936 callees, dc3 `15a64d92f` |
| 4 | **byte-owned** | **CITED, NOT RE-MEASURED** — `#3534`, 2026-08-25 | see below |

**On strength 4, and this is a fence rather than a footnote.** Decision 15
forbids re-taking `#3534`'s read, and nothing in this section reverses its
finding. `#3534` **flipped OFF the inline-decision permuter**, on a measurement
in both directions on one tree on one day: the port's wrong bodies are **99.87 %
opcode substitutions with 0 reorderings**, while the permuter's actual working
set is **2.14 % opcode, 52.50 % register, 7.90 % pure reorderings**.
`INLINE_PREDICATE.md`'s model and `splice.rs`'s S7 are **right for the port's
population and wrong for the permuter's, both by measurement.** A richer inline
scoreboard is not an argument for building that permuter, and this section is
not offered as one.

### 6.4 `exercised` — three structural facts, each measured here

Measured on the re-frozen 100-TU hold-out, **8,936** graded callees, dc3
`15a64d92f` (0 dirty), `/nologo /wd4355 /wd4164 /c /GR /O1 /Oi /EHsc`.
`work/w-inlmetric/exercised.txt`.

**(a) `INLINE-P` degenerates to a single threshold on this workload.**

```
   N_max == UNBOUNDED : 6,397 = 71.59 %
   N_max == 0         : 2,539 = 28.41 %
   N_max FINITE and non-zero (SCHEDULE D's graduated middle FIRING) : 0 of 8,936
```

**The whole of SCHEDULE D — `min(9, 1 + floor(19/(i−16)))`, the most elaborate
object in `INLINE_PREDICATE.md` §2 and the subject of round 31's §6.18.9 — fires
on ZERO of 8,936 workload callees.** It is STATIC-only, and:

**(b) The STATIC arm has a workload population of ONE.**

```
   EXTERNAL : 8,935        STATIC : 1        varargs : 0
```

The single STATIC callee is `?ModChan@@YAHH@Z` in `src/system/rndobj/ColorXfm.cpp`
(`size 28`, `index 28`, `N_max = inf`, `sites 10`, HIT). So F1's `(300,308]`
STATIC ceiling, `#3066`'s non-overlap, and SCHEDULE D all describe a region the
workload does not visit. `n_sites > 1` **is** exercised — 2,910 callees, up to
31 sites — but only in the EXTERNAL class, where `N_max` is UNBOUNDED-or-0 and
the site count therefore changes no verdict.

> **`#3066` is CITED and deliberately NOT re-derived.** It reads *"the port's
> largest lowered body is 152 B and c2's static-inline floor is > 308 B, so the
> windows do not overlap"* — a claim about the tree at that rung, and `#3063`'s
> standing lesson is that such a control must be re-derived before it is made
> mandatory. Re-deriving the **port** half needs a full port scan this lane did
> not run, so the port-side number is quoted as `#3066`'s and not as this
> lane's. What **is** measured here is the same non-overlap from **c2's** side
> and it is stronger: the STATIC clause has a population of 1 in 8,936
> regardless of what the port's ceiling is.

**(c) Four clauses are unexercised for a reason in the flags, not in the code.**
C9 (favour-speed) cannot move because the workload is pinned to `/O1` — GRID-I
moved it at `/O2` on 60 cells, so the clause is exercis**able** and merely
unexercised **here**. C15 is `0xff` throughout: `#pragma inline_depth` and
`#pragma auto_inline` appear in **0 of the 100** hold-out TUs. C21–C23 are the
only genuinely **unexercisable** rows — `/GL` and `profile-guided` appear in
**0** dc3 source files, and the two 46-dword tables sit above the image's raw
`.data`, zero at load.

**An unexercisable cell is not a covered one, and neither is an unexercised
one.** Nine of twenty-four is what this workload can grade.

### 6.5 The ONE read this lane took `[R]` — `FUN_10b5fb5f` end to end, and it discharges an open half of §2.1's correction

**Read-before-probe** (`WHITEBOX_LEVERAGE_2026-08-21.md`), priced before it was
taken (`work/w-inlmetric/read_price.txt`). The band's unread remainder is **80
functions / 22,840 B — 3.4× the bytes already read** — and this lane did **not**
enter it. What it read instead is the **whole of `FUN_10b5fb5f`**, 377 B, a
function already inside the read 13, of which §2.1 had quoted seven lines.
Image `sha256 c80981c0…a66258`, verified. Listing:
`work/w-inlmetric/FUN_10b5fb5f.asm`.

> #### The finding: **§2.1's `0x2000` `__forceinline` mask DOES carry. It is `edi`.**
>
> §2.1's CORRECTION block closes with *"the `0x2000` `__forceinline` mask does
> not carry as written — the mask at `0x10b5fcc1` is `0x2080`, and the test at
> `0x10b5fc92` is against a mask held in `edi` rather than an immediate."*
> **`edi` is named now:**
>
> ```
> 10b5fc31:  bf 00 20 00 00     mov    edi,0x2000        <- THE MASK, materialised
> 10b5fc36:  85 df              test   edi,ebx           <- ebx = [sym+0x4c]
> ...
> 10b5fc92:  8b 46 4c           mov    eax,DWORD PTR [esi+0x4c]
> 10b5fc95:  85 c7              test   edi,eax           <- __forceinline, edi still 0x2000
> ```
>
> `edi` is callee-saved and nothing between the two writes it. So the
> **original** §2.1 block's reading — *"`test DWORD [sym+0x4c], 0x2000` —
> `__forceinline`: bypass"* — was **right in substance** and wrong only in its
> address and its encoding. `w-sizebracket` retracted the address correctly and
> could not settle the mask; this settles it. **Two of the three things the
> correction put in doubt now carry, and the third (`0x2080` at `0x10b5fcc1`) is
> a genuinely different test on a different path.**

**Four more things the full read establishes, each `[R]` and none adopted:**

1. **The legality function is CALLED FROM CANDIDACY, and the edge is now read.**
   `0x10b5fc13: call 0x10b5c06b`, immediately followed by `test eax,eax` / `je`
   to the `return 0` arm. §1 lists `0x10b5c06b` as its own row without stating
   which caller reaches it; C11–C13 are therefore **inside** C8's function, not
   beside it.

2. **`ebx` holds `[sym+0x4c]` across the whole middle of the function**
   (`0x10b5fbf9: mov ebx,[esi+0x4c]`), and **at least five distinct bits of it
   are tested**: `0x10` (`0x10b5fbfc`), `0x100` (`0x10b5fc25`), `0x2000`
   (`0x10b5fc36`), `0x200` (`0x10b5fc3a`), then `0x2` and `0x10` of the low byte
   again at `0x10b5fca9`/`0x10b5fcb5`, and `0x2080` at `0x10b5fcc1`. **§1's
   legality row names `0x200` as a refusal bit at `0x10b5c06b`; it is tested a
   second time here, on a different path, to a different end.**

3. **`ds:0x10c3de20` is a THREE-VALUED selector and it is tested three times** —
   against `1` at `0x10b5fbde` and against `2` at `0x10b5fc4c` and `0x10b5fc69`
   — and the `== 2` arms call `0x10b9e796` with the string pointer `0x10b02588`
   and `0x10b9cae6`. A diagnostic/verbosity or `/Ob`-level selector is the
   obvious hypothesis and **this page does not claim it**; no cell has moved it.

4. **THERE IS NO LINKAGE ARM IN THE CANDIDACY FUNCTION — a negative result, and
   it narrows §5.** §5 says of the `16 << k` gap: *"something between the two,
   most plausibly **the linkage arm** and the `[sym+0x50]`-vs-emitted-size gap,
   is unread."* All 377 bytes are now read and **no storage-class or linkage
   field is tested anywhere in them.** Whatever produces `INLINE_PREDICATE.md`
   §6.17.3's measured STATIC/EXTERNAL split, it is not a branch in the function
   that owns the size test. **One of §5's two named candidates is eliminated;
   the other — the `[sym+0x50]`-vs-emitted-size gap — is untouched and is
   independently corroborated by §2.1b.**

**Fields this page had never named appear here and are NOT pursued:**
`[sym+0x94] & 0x400` (`0x10b5fbe7`), `[sym+0x90]` (`0x10b5fbf3`, `0x10b5fba7`),
`[sym+0x80]` (`0x10b5fca1` — §1 names it as the POGO profile record), and the
option globals `0x10c2e308`, `0x10c2eab0`, `0x10c2eaac`. Naming them is the
whole of what this lane does with them.

> **THE TABLE IN §6.1 IS NOT GROWN BY THIS READ, DELIBERATELY.** `PREREG.md` §5
> fixes the clause list before any measurement, and at least three clauses above
> (`[sym+0x4c] & 0x10` gating `[ebp+0xc] & 0xf00`, the second `0x200` test, the
> `0x10c3de20` selector) would be new rows. Adding rows discovered *after* the
> split was predicted is fitting the instrument to its own result. They are
> filed as a **named follow-up** in
> [`../../rungs/2026-08-26-w-inlmetric.md`](../../rungs/2026-08-26-w-inlmetric.md)
> §8 instead, for a lane that pre-registers them.

### 6.6 THE TWO `fitted` CELLS, READ — and both stay `fitted`, for two different reasons

> **Added 2026-08-27 by lane `w-inlfit`** (decision 20, board **#3717**–**#3722**).
> **Amend-beside**: §1–§6.5 are unchanged, including §2.1's struck block, its
> correction, and §6.1's table — **no clause row is added, removed, renumbered or
> restated**, and the reachable denominator is still **21 of 24**.
>
> Prereg `work/w-inlfit/PREREG.md`, registered before the image was opened.
> **Predicted reach 0, delivered 0**: this lane wrote **zero `crates/` bytes** and
> proposes no `DISCLOSURE` row. Both predictions held, so the value here is the
> located *reason* each fit cannot yet be replaced.

#### 6.6.1 C8 — the size ceiling is now READ END TO END, and it still does not reach the port's constant `[R]`

Four facts, none of which was on this page:

1. **`DAT_10c46318` has exactly ONE reader in the entire image, and it is C8's
   own `cmp` at `0x10b5fc8a`.** It has exactly two writers, both inside
   `FUN_10b5e4cc`: `0x10b5e4d7` stores `0x3e8` (1000) and `0x10b5e4e8` stores
   `0x10 << k`. Three references in 22 MB of disassembly, and that is all of
   them. **C8's right-hand operand is therefore wholly determined by C7's
   producer**, and nothing else in c2 can perturb it between the two.
2. **`DAT_10c46318` is above the image's raw `.data`** (which ends at
   `0x10c3cc00`), so it is **zero at load**. Since c2 demonstrably inlines,
   `FUN_10b5e4cc` necessarily runs before the inliner — a structural conclusion
   from the writer set, not an assumption.
3. **`k = DAT_10c2ea98` is `3`, and it is a real initialised datum**: `.data`,
   file offset `0x12dc98`, inside the raw region. So the ceiling is
   `0x10 << 3` = **128**, in the units of `WORD [sym+0x50]` — a pre-codegen
   **instruction count** (§2.1a, and the diagnostic string is `"%d instrs"`).
4. **`k` is never stored by any instruction; its ADDRESS is planted in an
   option-descriptor record**, at `0x10c29800`. The record is
   `[name_ptr, value_ptr, kind]` at stride 12, and its name field resolves to
   the UTF-16 string **`-vol#`** — an undocumented numeric switch. So c2's
   ceiling is command-line-settable, the workload never sets it, and `k = 3`
   holds for every compilation this project runs.

`work/w-inlfit/optmap.py` recovers the descriptor table from the run of stores
that builds it — the table itself is zero at load and unquotable, exactly as §5
requires. The record phase is **anchored on the `-EHs`/`-EHa` pair** rather than
assumed, and the recovery is self-checking: `-Gs#`/`-Gt#` land on adjacent
dwords, and `-MLd`/`-MDd`/`-MTd`/`-ML`/`-MD`/`-MT` land on six consecutive ones
in source order. The same block names **21 further undocumented `-inl*#`
switches** whose value words occupy `0x10c45db4`–`0x10c45e10`, immediately below
§5's two POGO parameter tables at `0x10c45e18`/`0x10c45ed0`. `k` is also read at
`0x10b5da64` as a **multiplier** (`(n+2) * k`) inside the unread
`FUN_10b5da2f`, so it is a general inliner scaling knob and not solely this
ceiling's shift. *That `-vol#` reads as an inline **vol**ume control is the
obvious gloss and this page does not claim it* `[I]`.

> **The arithmetic, registered in the prereg before the read and held.** c2's
> ceiling is **128 instructions**. The port's `INLINE_UNBOUNDED_BYTES` is **64
> bytes = 16 PPC words**, and its relatives are 128 B and 80 B. At one word per
> instruction that is **8×**; against §2.1c's measured `/O1` bracket of
> `(108,116]` — 27–29 words — it is still **4.4–4.7×**. §5's *"`16 << k` does
> not compose into the measured numbers"* is now the same statement with its
> operand's provenance closed on both ends.

**What is missing is NAMED, and it is two links, neither of them in this band:**

* `[sym+0x50]` is initialised from the `.gl` `SIZE` field at `0x10b9bf6c` and is
  **reduced by every pass that runs between there and `0x10b5fc8a`**. §2.1b
  measures the consequence — `arith_012` and `mix_008` at an identical `SIZE` of
  115 with opposite verdicts — and **nothing yet located reads that reduction.**
* Even given the reduced count, turning a count into emitted PPC bytes is the
  whole of lowering, which is what the port's `s` measures.

**So C8 stays `fitted`, and the fit is not replaceable by any read confined to
`0x10b5b86d`–`0x10b62b00`.** The port's constant is in the wrong unit, and the
converter is two subsystems away. Adopting 128 would be an emit change priced
against a number that does not mean what the port's constant means.

#### 6.6.2 C20 — the recursion, its six arguments, and the division nobody had `[R]`

**The edge is real and it is at `0x10b62402`**, 774 bytes into `FUN_10b620fc`
(§6.1 cites the function's entry, which is where the row's address points).
`FUN_10b61ee1` has **exactly two callers** — the pass entry `0x10b62675` and
this one — so the driver is entered once per function and once per expansion,
and by nothing else.

The pass entry's own call is at **`0x10b6276e`**. Reading the two call sites
against each other fixes all six parameters:

| driver parameter | pass entry `0x10b6276e` | the recursion `0x10b62402` |
|---|---|---|
| `ecx` — the function | the function being compiled | `esi` |
| `edx` — **level** | `1` | **`BYTE [site+0x18] + level`** (`0x10b623f2`, `0x10b623f9`) |
| stack 1 — **budget**, by value | `B` = §2.2's clamp | **`*budget / remaining_sites`** — `idiv` at `0x10b623ec` |
| stack 2 | `0` | threaded through unchanged |
| stack 3 | `100000000` | `[site+0x10]` |
| stack 4 | `0` | `[site+0x14]` |

`ret 0x10` and `sub eax,[ebp+0x8]` at `0x10b620f2` confirm §1's *"returns budget
consumed"*: the driver saves its incoming budget at entry and returns the
difference.

> #### The finding: **THE GROWTH BUDGET IS DIVIDED EVENLY AMONG THE REMAINING CALL SITES.**
>
> The divisor is traced through four frames rather than guessed — expansion
> `[ebp+0x14]` ← charge `[ebp+0x10]` ← per-site driver `[ebp+0x1c]` ← the
> driver's **local** `[ebp-0xc]`. That local is the **out-parameter of the site
> collector**: `lea edx,[ebp-0xc]` at `0x10b61f99`, immediately before
> `call 0x10b600e6` — C5's function. It is decremented once per site at
> `0x10b620c8`, at the bottom of the loop that walks the collector's list.
>
> **So at site *i* of *n*, the nested pass receives `remaining_budget / (n − i + 1)`.**
> Later sites are divided by less, against a smaller remainder.

Three further reads at the same call, each `[R]` and none adopted:

1. **Stack 3/4 are one 64-bit quantity, and it HALVES.** `100000000` at the top
   level, and `shrd eax,edi,0x1` / `shr edi,1` at `0x10b6204e` shifts the pair
   `[site+0x10]`/`[site+0x14]` right by one, in place. A second budget-like
   quota, on a different schedule from the instruction budget.
2. **A `__forceinline` callee is charged NOTHING for its nested expansion.**
   `0x10b6240f` tests `[sym+0x4c] & 0x2000` and `jne` skips **both**
   `sub DWORD [ebx],eax` (`0x10b62418`) and `add ds:0x10c3f5cc,eax`
   (`0x10b6241a`). §2.2's exemption is for callees at or under 40 instructions;
   this is a second, orthogonal one, and it exempts the **global growth total**
   as well as the local budget.
3. **§5's *"the only site-count arithmetic in the image is this division"* — said
   of C22's POGO discount — is FALSE.** There is a second site-count division,
   at `0x10b623ec`, on the **non-POGO** path, and §6.4 measures 2,910 workload
   callees with more than one site and up to 31. C21–C23 remain `unexercisable`;
   it is the *exclusivity* clause in the prose that does not survive.

**And the port's counterpart is still not derived from any of it.**
`splice.rs`'s `S6-chain` has **no level, no budget, no site count and no
division**. It walks to the chain's end and asks its size clause once. c2
re-enters the entire decision at each level, under a level that strictly
increases (C14's cap), a budget that is divided (C17), and C8's size test — the
first two of which are `absent` from the port and the third of which is §6.6.1.
These are different rules.

> **Why the fit nevertheless works, which is the part the read buys** `[I]`.
> The port admits only chains in which every link has **exactly one** call site
> (`S6`/`S2`) and whose end has **none** (`S6-chain-open`). On that population
> `n = 1` at every level, so **c2's division is the identity** and the nested
> budget is the parent's, undivided. The bodies are at most 64 emitted bytes, far
> under §2.1b's one-sided `SIZE < 98 ⇒ inlined` at `/O1` (zero counterexamples in
> 105 cells), so C18's `jbe 0x28` means nothing is charged and C17 cannot bind.
> **The port's fixpoint is right on its own admitted set for a reason that is now
> read rather than only measured** — but a coincidence located on a subset is a
> soundness argument for a fit, **not** a derivation of it, and §6.1's ties break
> toward the weaker state. **C20 stays `fitted`.**

#### 6.6.3 The grader's blind spot: eight of the twenty-four addresses are MID-INSTRUCTION

`check_table.py`'s ADDRESS check asks whether an address lies inside the function
its `owner` column names. It cannot fail on an address that is inside the right
function and inside the middle of an instruction — and **eight of the 24 are**:
C2, C3, C4, C14, C16, C17, C18, C19. Measured against the **independent objdump
boundary set** (425,871 instruction starts), not against the Ghidra database the
addresses came from. `work/w-inlfit/addr_align.py`, watched **green** on a
two-row table and **red** on a one-byte planted shift before either verdict was
quoted (`#3336`).

Three verified by hand, with the address the clause actually describes:

| row | cited | what is really there | the clause's real address |
|---|---|---|---|
| C4 | `0x10b6276a` | +6 into `mov ds:0x10c46330,0x10c46334` | **`0x10b6276e`** |
| C18 | `0x10b6249b` | +1 into `mov ecx,[eax+0x4]` | **`0x10b625b6`** — `cmp eax,0x28`, the only one in the function |
| C19 | `0x10b624a2` | +1 into `cmp ds:0x10c6f1c8,0x1` | **`0x10b625bb`** and **`0x10b625c1`**, the only two |

**C18/C19's citations are `0x11b` bytes early because they landed in a DUPLICATE
of the wrong function.** `0x10b62488`–`0x10b624be` in the charge is an
instruction-for-instruction copy of `0x10b5fb85`–`0x10b5fbbb` in candidacy — an
inlined helper emitted twice — and the two cited addresses fall inside the copy.

**Neither of this lane's own rows is affected**: `0x10b5fc8a` (C8) and
`0x10b620fc` (C20) are both genuine instruction starts, verified. **No row is
edited here** — the table is another lane's frozen instrument and its green is
quoted on its own tree. The checker is left beside it so the next lane can act
under its own prereg.

**Filed as follow-ups, NOT pursued and NOT adopted** (§6.5's convention):
C10's cited `0x10b609d3` decodes to `call 0x10b5e64d`, not a `0x2000` test —
aligned but describing something else, which is a defect class the new checker
does **not** reach; and `FUN_10b5da2f` (573 B, unread) is the second consumer of
`k`.

### 6.7 THE `[sym+0x50]` REDUCTION DOES NOT EXIST — §6.6.1's first missing link is refuted, and §2.1b's conclusion survives without it

> **Added 2026-08-28 by lane `w-lowerband`** (decision 21, board **#3731**–**#3736**).
> **Amend-beside**: §1–§6.6 are unchanged — including §2.1's struck block and its
> correction, §2.1a's *"exactly ONE 16-bit store"*, §2.1b, and §6.1's table.
> **No clause row is added, removed, renumbered or restated**; the reachable
> denominator is still **21 of 24** and the split is still
> `absent 17 · fitted 2 · [R]-derived 2 · unexercisable 3`.
>
> Prereg `work/w-lowerband/PREREG.md`, committed at `19d6c4797` **before the
> image was opened**. **Predicted reach 0, delivered 0**: zero `crates/` bytes,
> no `DISCLOSURE` row, no `gate.sh` row (`#3691`).
> Full record: [`../WB_LOWERBAND_FINDINGS.md`](../WB_LOWERBAND_FINDINGS.md).

§6.6.1 names as C8's first missing link that `[sym+0x50]` *"is **reduced by
every pass that runs between there and `0x10b5fc8a`**"* and that *"nothing yet
located reads that reduction."*

> #### **Nothing has located it because it does not exist. The field has ONE writer and NINE readers, and the writer stores `il-read-varint16`'s return verbatim.**

**Three instruments, and the two that could miss were watched missing**
(`work/w-lowerband/controls.out`, `#3336`):

| instrument | population | result |
|---|---|---|
| `f50.py` over the **independent objdump boundary set** | **424,232** decoded instructions (= 425,871 addressed lines − 1,639 byte-continuation lines; `#3721`'s denominator is the former) | 125 operands at `+0x50`; **1** 16-bit write |
| Ghidra's decompiler (control-flow-driven, not linear) | the whole export | **0** `ushort` assignments at `+0x50` image-wide; 13 read occurrences over 12 lines = the same **9** instructions E2 finds (the decompiler re-materialises the load inside `CARRY4` idioms and compound tests) |
| `bytescan.py`, **decode-independent** | **all 1,232,384 bytes of `.text`**, 2,136 encoding patterns (`mov`/`add`/`sub`/`or`/`and`/`xor`/`adc`/`sbb`/`xchg`/group1/3/5/shifts, disp8 + disp32 + SIB, and both byte halves) | **exactly one** 16-bit-store encoding present |

`bytescan.py` exists because `objdump` sweeps `.text` linearly and c2 has a
~150 KB data block at its head; a store inside a desynchronised run would be
invisible to the listing. It is not.

**The nine readers, complete:** `0x10b56732` (`FUN_10b566e9`, returns the field
`& 0x3f`), **`0x10b5fc86` (C8)**, `0x10b60a6f` (C17), `0x10b625b2` (C18),
`0x10b625bd` (C19), `0x10b626f7` (C2), `0x10b72ee6` and `0x10b72f0f` (two
**64-bit whole-module accumulators**, `DAT_10c46398` and `DAT_10c2ebb8`),
`0x10b8fbda` (`FUN_10b8fb47`, an **IL hash**).

#### 6.7.1 §2.1b's conclusion survives — and its one-sided form must not be raised to 128 `[O]`

**Read-before-probe: `w-sizebracket`'s cells are already measured and committed,
so this is a RE-READ and nothing was recompiled.** 168 unique tags,
`work/w-lowerband/ceiling_check.out`. Grading `.gl SIZE < 128` — §6.6.1's own
ceiling — against the recorded verdict:

| profile | `< 128` inlined | `< 128` but **KEPT** | `≥ 128` kept | `≥ 128` but **INLINED** |
|---|---:|---:|---:|---:|
| `/O1` | 49 | **8** | 17 | **8** |
| `/Ox` | 55 | **2** | 21 | **8** |

Sixteen counterexamples at `/O1`, **in both directions**. So the tested value is
**not** the `.gl` `SIZE` — §2.1b is right — but not for the reason §6.6.1 gave.
§2.1b's `T = 98` form is untouched and holds; **raising `T` to the image's 128,
which is the natural move once §6.6.1 publishes the ceiling, breaks it on eight
cells that were already on disk.**

**Therefore the missing link is AT THE STORE, not after it**, and it is one of
three named things — the cheapest being that the harness's `SIZE` column and
`il-read-varint16`'s three forms (§2.1d: `0x80` → three bytes; `0x81..0xff` →
**one signed byte**) are not the same quantity. **None of them is "the whole of
lowering" and none is "every pass in between."**

#### 6.7.2 What IS on the path: a POINTER-SELECTION chain, and TWO OF ITS THREE SITES ARE IN THE BAND

Nothing changes the field's value; three sites change **which record's field is
read** and **whether the test runs at all**.

| site | addr | in band? | what |
|---|---|---|---|
| **S1** | `0x10b5fb6e` → `FUN_10b5bfae` | yes | site→symbol, **18 bytes, 13 callers, TWO arms**: `sym = [[site+0x28]+0x18]` when the operand node's kind byte is `4`, else `sym = *[[[site+0x28]+0x18]+0x8]` |
| **S2** | **`0x10b5fbf3`** | **yes** | `mov esi,[esi+0x90]` — **C8's operand is REPLACED by another record**, gated `DAT_10c3de20 == 1 && [sym+0x94] & 0x400`. §6.5 named both fields as *"NOT pursued"*; they are the second source of C8's left operand |
| **S3** | `0x10b624c6`/`0x10b624dc` + `0x10b62557`/`0x10b6255a` | **yes** | the charge **saves, overrides and restores the favour-speed global `DAT_10c2e310`** around the expansion, from `[[sym+0x80]+0x76] & 0x800000` — so C8's *liveness* is per-callee. POGO-gated, therefore **read and dead on this workload** |

> **This is what contradicts §6.6.** §6.6.1 concludes *"the fit is not
> replaceable by any read confined to `0x10b5b86d`–`0x10b62b00`"* **because both
> missing links are outside the band.** Link 1 as described does not exist, and
> what stands in its place is **two-thirds in-band and was unread**. The
> conclusion still holds — for the *unit* reason, §6.6.1's second link — but the
> reason given for it does not.

#### 6.7.3 Two corrections to §6.6.1, both about instruments `[R]`

1. **`k` has THREE readers, not two**: `0x10b5da64`, **`0x10b5dacb`** (a second
   read inside the same unread `FUN_10b5da2f`) and `0x10b5e4cc`. Four references
   in total, the fourth being the descriptor store at `0x10c29800`.
2. **`"k is never stored by any instruction"` is exact about DIRECT-ADDRESSED
   stores and silent about indirect ones.** `0x10c29800` plants `k`'s *address*
   in the `-vol#` descriptor — precisely a handle for a generic numeric-option
   setter to store through. `k = 3` is the **load-time** value; that it is the
   **run-time** value under `/O1` is not established by a direct-store
   enumeration, and since `DAT_10c46318` is BSS the ceiling's run-time value is
   settled only when `k`'s is.

**And a datum nobody had read: the favour-speed bit's IMAGE value is `1`** —
`DAT_10c2e310`, raw `.data`, file offset `0x12d510` — and non-zero means C8's
size test is **skipped**. `FUN_10b82338` writes it from **bit 23 of a
per-function option word** (`0x10b8238d`–`0x10b82392`), confirming §2.1's
attribution; **so the default being ON does NOT license "therefore `/O1` clears
it"**, and this page does not claim it.

**What does follow is that the bit has THREE homes.** When `DAT_10c3de20 == 2`,
or `DAT_10c2eaac != 0 && DAT_10c6f1c8 == 2`, the same bit is written to a
**different global `DAT_10c3dddc`** at `0x10b823a1` and **`DAT_10c2e310` is
never written, keeping `1`** — C8's size test off. And `0x10b82352` stores the
option word into `[[…]+0x80]+0x76`, **exactly the field §6.7.2's S3 reads at
`0x10b624d4` with mask `0x800000` — bit 23 again**. So **S3 restores the
favour-speed bit to the CALLEE's own `/Ot`-vs-`/Os` setting for the duration of
its expansion**, rather than being a profile-weight mechanism. S3 needs
`[sym+0x80] != 0` and bit 10 of `[[sym+0x80]+0xb1]`, and the `+0x76` fill needs
`DAT_10c2eaac != 0` (image `0`): **read, not exercised here**.

Neighbours, same read: `DAT_10c2e2fc`, `DAT_10c2e308`, `DAT_10c2eab0`,
`DAT_10c2eaac` are all `0` in raw `.data`; `DAT_10c3de20`, `DAT_10c3dddc`,
`DAT_10c6f1c8` and `DAT_10c46318` are **BSS, zero at load**.

---

### 6.8 THE 24 `-inl*` SWITCHES ARE THE POGO TABLES' OWN OVERRIDES — and §5's "not quotable" is too strong

> **Added 2026-08-28 by lane `w-inlswitch`** (decision 22, board **#3768**–**#3773**).
> **Amend-beside**: §1–§6.7 are unchanged — including §2.1's struck block and
> its correction, §2.1a, §2.1b, §6.1's table, §6.6's two `fitted` verdicts and
> §6.7's refutation. **No clause row is added, removed, renumbered or
> restated**; the reachable denominator is still **21 of 24** and the split is
> still `absent 17 · fitted 2 · [R]-derived 2 · unexercisable 3`.
>
> Prereg `work/w-inlswitch/PREREG.md`, committed at `5eec1a7d5` **before the
> image was opened**. **Predicted reach 0, delivered 0**: zero `crates/` bytes,
> no `DISCLOSURE` row, no `gate.sh` row (`#3691`).
> Full record: [`../WB_INLSWITCH_FINDINGS.md`](../WB_INLSWITCH_FINDINGS.md).
> Instrument: [`../scripts/dump_inlswitch.py`](../scripts/dump_inlswitch.py).

#### 6.8.0 The count `#3718` published is wrong, and it is 24 `[R]`

`optmap.py`, re-run unmodified in this lane's tree and **byte-identical** to
`w-inlfit`'s committed output, names **24** `-inl`-prefixed switches over **24
distinct** value words. They tile `0x10c45db4`–`0x10c45e10` **contiguously**:
`(0x10c45e10 − 0x10c45db4)/4 + 1 = 24`, so there is no gap and no screen under
which the answer is 21. 23 are numeric (kind `0x2401`); **`-inlnlw` at
`0x10c45db8` is a boolean** (kind `0x0101`), which is what the `-inl*#`
spelling silently drops — but 23 is not 21 either.

#### 6.8.1 §5's two POGO tables ARE recoverable, by §5's own method `[R]`

§5 says of `DAT_10c45e18`/`DAT_10c45ed0` that *"none of their values is
quotable from the image and this page does not quote them."* The premise —
BSS, zero at load — is exactly right. The conclusion is **too strong**, for the
same reason the descriptor table is recoverable: **the code that fills them is
in the image.**

**`FUN_10b5b88f`** (`0x10b5b88f`, 335 B) scatters **37** switch value words —
the contiguous block `0x10c45d80`–`0x10c45e10` — into `[ecx+0x00…0xb4]`, and it
has **exactly two callers**, each passing one table:

| filler | `ecx` | then |
|---|---|---|
| `FUN_10b5ba71` (`0x10b5ba78`) | `0x10c45ed0` — **table B** | 33 zero-guarded default stores |
| `FUN_10b5bc6e` (`0x10b5bc73`) | `0x10c45e18` — **table A** | 33 zero-guarded default stores |

The guard is `cmp ds:F,0 / jne skip / mov ds:F,<imm>`: **a switch left unset
falls through to a default that differs between the two tables.** Both 33-value
sets are printed in `WB_INLSWITCH_FINDINGS.md` §3. 46 − 33 = the 13 fields that
get no default in either table, and they are exactly the 13 that no switch
names and nothing reads.

**So a `-inl*` switch's "load-time default" is not a value at its own address**
— that is always `0`, by BSS — **it is the value its DESTINATION field receives
when the switch is absent.** The lane's own prereg predicted ≥ 12 initializing
stores at the switch words; there are **0 of 24**, and locating the mechanism
instead is what the miss bought.

#### 6.8.2 Which table is live, and the two gates `[R]` `[O]`

`FUN_10b5e4cc` — §6.6.1's producer of `DAT_10c46318` — is also the whole
parameter initialisation, and it runs both fillers before copying **46 dwords**
into the live record at `0x10c3f510`:

```
10b5e4cc  k = DAT_10c2ea98
10b5e4d2  DAT_10c46318 = (k <= 6) ? 0x10 << k : 1000     ; §6.6.1, re-read here
10b5e4ed  call FUN_10b5ba71                              ; fill table B
10b5e4f2  call FUN_10b5bc6e                              ; fill table A
10b5e4f7  if DAT_10c462c4 == 0: return                   ; GATE 1 — not read
10b5e50a  call FUN_10b5b9de(size)                        ; module-size trim of A
10b5e50f  esi = (DAT_10c6f1c8 == 0) ? 0x10c45e18 : 0x10c45ed0    ; GATE 2
10b5e52a  rep movsd 0x2e dwords -> 0x10c3f510
```

`DAT_10c6f1c8` is the **requested POGO mode**, and §6.8.4 measures it `0` here.
**Table A is the live table on this workload.**

#### 6.8.3 All 24 have a reader, all 24 are dead here `[R]` `[O]`

**24 of 24** have at least one read of their live field — the prereg predicted
at most 11, and at most 6 tied to a named decision. **39 read instructions
serve the 24 switches** (58 over all 46 fields), and **25 of the 39** are
inside **`FUN_10b5fcd8`**, §5's POGO cost model, which
accumulates a score and returns `score < -inlS#`. The decisions, in arithmetic:

* **`-inlS#`** (`+0x00`, A **60** / B **2**) is the **accept threshold** —
  `0x10b600d6`: `cmp esi,ds:0x10c3f510` / `setl al`.
* **`-inlcsw#` `-inldasw#` `-inlcasw#` `-inlflcsw#` `-inlfcsw#`** are five
  linear weights summed in one run at `0x10b5fdba`–`0x10b5fdf1` and subtracted
  from the score. **Four of the five default to 0 in both tables** — a dormant
  cost model inside the cost model.
* **`-inlnlw`** (boolean, default 0) gates the whole banded rational-scale
  block, so **`-inlniln#`/`-inlnild#`/`-inlnoln#`/`-inlnold#` are four numeric
  switches behind one boolean that is off by default.**
* **`-inlocsa1#`…`4#`** are flat credits by call-count band (25 M / 50 M /
  100 M); **`ocsa2`/`ocsa3` default to 0 in both tables** while `ocsa1`/`ocsa4`
  are 96 (A) / 15 (B).
* **`-inlcrmax#`** (10) caps the repeat count above which **`-inlfcsa#`**'s
  credit `esi -= (fcsa + esi)/ecx` is skipped.
* **`-inluserinl#`** (A 8 / B 2) credits bit 7 of the caller's argument byte;
  **`-inlnobr#`** (A **48** / B 3) credits bit 7 of `[sym+0xb1]` and is table
  A's largest single credit.
* **`-inlmlsa#`** (A 32 / B 15) is the one read outside the model —
  `FUN_10b5dc6c` at `0x10b5dca9` bails when a byte counter exceeds it.

**Every one of them is read and dead on this workload.** `FUN_10b5fcd8` is
entered only from `0x10b60a50` under §5's profile-record gate, and
`FUN_10b5dc6c`'s caller `FUN_10b60727` opens with `cmp ds:0x10c3de20,0x1`
(`0x10b60730`) and `cmp ds:0x10c6f1c8,0x1` (`0x10b60767`). **This is a
characterization of c2's decision surface, not a candidate for adoption, and
nothing in it licenses an emit.**

#### 6.8.4 `DAT_10c3de20` is the EFFECTIVE POGO mode — and it narrates nothing `[R]`

`w-lowerband` §7 filed it as *"389 refs, 10 writers, three values"* with the
follow-up that *"naming the switch that sets it to `2` would make c2 narrate
its own inline decisions."*

**The writer count is 19 instructions in 13 owner functions**, and the two
instruments agree to the address: Ghidra's `WRITE` (13) plus `READ_WRITE` (6,
the `and ds:…,0x0` clears) is the same set the objdump listing yields. Neither
instrument produces 10.

**The chain, complete.** The only literal `2` is `0x10b9e2bb`, inside
`FUN_10b9e1d2`, which returns at once unless `DAT_10c6f1c8 == 2`
(`0x10b9e229`). `DAT_10c6f1c8`'s enabling writers are three instructions in
`FUN_10b848dc`, immediately after the option-table walk:

| site | condition | stores |
|---|---|---|
| `0x10b84b47` | `[0x10c46bcc] != 0` — **`-pgo#`** / **`-po#`** | `2` |
| `0x10b84b58` | `[0x10c46bc4] != 0` — **`-pgu#`** | `2` |
| `0x10b84b80` | `[0x10c46bd0] != 0` — **`-pgi#`** / **`-pi#`** | `1` |

`0x10c46bcc` and `0x10c46bd0` are the value words `optmap.py` prints as
`(reg)`; resolved here by tracking the two registers loaded at `0x10c29c23`
and `0x10c29c28`, which also shows **`-pgo#`/`-po#` and `-pgi#`/`-pi#` are
alias pairs on one word each.** `FUN_10bae79c`'s two stores write `0` only.

`0x10b9e07d` is a bare mirror `DAT_10c3de20 := DAT_10c6f1c8`, and the two
neighbouring stores zero it right after a diagnostic — `"ERR:\t%s was not
profiled; Pogo disabled\n"` (`0x10b16788`) and `"WRN:\t%s was not probed; Pogo
disabled\n"` (`0x10b16724`). **So `DAT_10c3de20 ∈ {0,1,2}` is
`{no POGO, instrument, optimize/update}`, and it is the mode that took effect
where `DAT_10c6f1c8` is the mode requested.**

> **The follow-up's premise is FALSE.** The switch exists and is named, but
> setting it does not make c2 report anything — it puts c2 in profile-guided
> optimization, which **swaps the live parameter record from table A to table
> B**, which differs from A on **13 of its 46 fields (8 of the 24 switch-fed
> ones) by 2.1× to 30×, B tighter in every one**, and turns on a cost model
> gated on profile data. It is a
> mode selector that **changes** the inline decision, so using it to observe
> the decision would be measuring a different compiler. The narration seam this
> was reaching for already exists and is `cl /FAsc`.

#### 6.8.5 `FUN_10b5da2f` READ — and its "second reader of `k`" is a loop reload `[R]`

573 B, **one caller `0x10b5eb27` inside `FUN_10b5e9a5`**, which is in the band.
It is a **budgeted statement-cost test returning 1 when the cost exceeds the
budget**:

* **budget** = `k · (n + 2 + [DAT_10c2e310 != 0] + 2·[attr 0x500000 & 8])`,
  where `n` counts the operand nodes of kind 1/2 with a non-null `[[node+0x18]+0x14]`
  (`0x10b5da47`–`0x10b5da98`);
* **cost** accumulates over the statement list by node kind — `0x0d` +1 (plus a
  structural match ending in `call 0x10b4cc87` that re-anchors the walk),
  `0x0f` +2 **and a `2k` refund to the budget**, `0x13` +2;
* the tail is `cmp [ebp-4],esi; jg` → return 1.

**`k` has three read instructions and two semantic uses.** `0x10b5dacb` is the
loop-head reload made necessary by the `0x0f` arm's `neg ecx` at `0x10b5daed`.
`#3734` is correct as filed; its implication that `k` is *"a general inliner
scaling knob"* at two independent places is one place, read twice — the other
use is §6.6.1's `16 << k`.

#### 6.8.6 `k`'s RUN-TIME value is settled at 3 — `#3734`'s open question is closed `[R]` `[O]`

`#3734` left it open because `0x10c29800` plants `k`'s *address* in the
`-vol#` descriptor, *"precisely a handle for a generic numeric-option setter to
store through."* Both halves are now closed.

**Read.** `FUN_10c1f746` walks the table from `0x10c46bd8` at **stride 12**
(`add esi,0xc` at `0x10c1f7a4` — the stride confirmed independently of
`optmap.py`), terminating on `BYTE [esi+9] == 0`, and calls `FUN_10c1f572`
**only on a name match**. `FUN_10c1f572`'s kind-`0x24` arm is three
instructions — `call 0x10c1f34c` (parse), `mov ecx,[edi+4]` (the value_ptr),
`mov [ecx],eax` — and **it is the only store through a numeric descriptor's
value pointer in the image. There is no initialisation sweep.**

**Measured** `[O]`, witness `work/w-inlswitch/cl_argv_modes.out`: `cl /Bd`
prints each pass's own command line, and over **every row of
`scripts/lanes.txt`** plus `/Os` `/Ot` `/Ox /Ob0` `/Ox /Ob1`, the c2 argv is
`-il … -typedil -Fo… -W 1 -Gs4096 -G604 -QVMX128 -QDD2 -MT -Fdvc100.pdb -f … [-Og] [-Ob0|-Ob1|-Ob2] [-Gy] [-EHs]`
and **contains no `-vol`, no `-inl*`, and no `-pgi`/`-pgo`/`-pgu`/`-pi`/`-po`/
`-pv` at any mode.** The only inline switch cl ever passes is `-Ob<n>`
(`0x10c46bc0`), which is not one of the 24.

> **`k = 3` at run time, so `DAT_10c46318 = 0x10 << 3 = 128`, on every
> compilation this project runs.** Same evidence gives `DAT_10c6f1c8 = 0`
> (table A live) and `DAT_10c3de20 = 0`, so **§6.7.2's S2 gate
> (`DAT_10c3de20 == 1`) and §6.7.3's `DAT_10c3dddc` arm (`== 2`) are both dead
> here** — which sharpens §6.7's *"read, not exercised"* into a measurement.
>
> **This does not make 128 adoptable, and this lane does not adopt it**
> (decision 22 §3; `#3732`'s 8 counterexamples in each direction; §6.7.1's
> `/O1` table). Settling `k` closes a **provenance** question. C8's remaining
> defect is the **unit** — §6.6.1's second missing link — and §6.6.1's verdict
> is untouched.

#### 6.8.7 A control this lane failed, and caught before publishing `[O]`

An earlier probe reported that `cl /Ox /Gy` does not pass `-Gy` to c2 — which
would have made two `scripts/lanes.txt` rows byte-identical duplicates. **It
was the instrument.** The loop wrote the mode as an unquoted `$m` and **zsh
does not word-split unquoted parameter expansions**, so `cl.exe` received
`/Ox /Gy` as one argument and parsed only `/Ox`; the same defect dropped
`/EHsc` from every multi-flag row. Re-derived with separate argv entries,
`-Gy` **is** passed at every ordering. No lane is a duplicate and no row is
owed. Recorded because the false reading was one command from publication, and
because it is the same class as `#3731`: an enumeration of one addressing form
— here, one *argument* form — quoted as an enumeration of the thing.
