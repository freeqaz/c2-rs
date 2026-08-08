# WB-F `wb-eh` — factor D's machinery, read off c2's own EH emitter and graded by objs

> **PROVENANCE — DISASSEMBLY-DERIVED.** Every address below is an absolute VA in
> the exact image pinned in [`C2_MAP_METHOD.md`](C2_MAP_METHOD.md) §0
> (`sha256 c80981…6258`), verified in this lane before the first grep. This file
> is **navigation**. Nothing here is adopted into `crates/` — see
> [`DISCLOSURE.md`](DISCLOSURE.md) and §7 for the pre-drafted rows a later code
> lane would carry.
>
> **`high` confidence means "I read the instructions correctly", not "this is
> what c2 does"** (method doc §7, the `.bss` retraction). §5 is where the objs
> get their say, and **one registered prediction of this lane was refuted by
> them** (§5.3, P3.3) and is retracted rather than defended.

Lane `wb-eh` / branch `wt-wb-eh`, branched at master **`9ed20248`**.
PREREG: [`WB_EH_PREREG.md`](WB_EH_PREREG.md), committed **`70f18db4`**, before
the first grep of `~/ghidra-projects/export/c2/` and before the first `cl.exe`.
Scored in §8. Board rows **#1860**–**#1873**; **#1874**–**#1879** left
explicitly unminted.

---

## §0 Result, up front

**Three things, in the order they matter.**

1. **The Main.cpp chain's stuck rung is NAMED, and it is not `0x2C` and not
   EH.** `src/Main.cpp` has exactly **one** emitted function; its `.ex` body is
   **41 tokens** and this lane tokenised **every one of them** with c2's own
   operand-class table, ending cleanly on the `4D` end-of-stream. `0x2C` is a
   `TYPE` plus **one raw byte that c2 reads and discards** — it carries no
   source operand at all, because `.ex` is a stack machine and the convert's
   source is the stack top. The port's `expr-convert-no-value` is therefore a
   fact about the port's `cstack`, not about the stream, exactly as board #1354
   said. And at master the row does not even reach `2C`: its first blocker is
   **`param-width-undetermined:mid`, in the formals header, four tokens before
   the body opens** (§4).
2. **The EH machinery is located and read**, prefix to funclet to record set
   (§2, §3): the `.pdata` word is
   `hasHandler<<31 | 1<<30 | (len_words&0x3FFFFF)<<8 | prolog_words&0xFF`
   computed in a **deferred patch pass** at `0x10bff811`; the funclet split is a
   walk that cuts `.text` at every `__catch$`/`__unwind$` label
   (`0x10c22046`–`0x10c2205f`); and **bit 31 is the same predicate that emits
   the two-word handler prefix**, masked to zero for an `__unwind$` region at
   `0x10c22080`. E3's three-record obj separates that reading from both
   registered rivals (§5.2).
3. **The EH state map is NOT in the IL — but two of its counts are.** `5C` —
   the token the project calls the EH live-state marker — is `TYPE` + `i32c`, and
   across **five objs / nine tokens** its operand takes exactly **two** values,
   `1` (a destructible-object region) and `0x101` (a `try` block). Those values
   are **neither the states in the ip-to-state map nor `maxState`**: c2 *derives*
   the map in its IR (`0x10c219a6` seeds `-2`, `0x10c219c4` propagates,
   `0x10c220c9` emits one 8-byte entry per change), and 4 `5C` tokens produce 18
   map entries. **PREREG P3.3's ip2state link is a MISS and is retracted.** What
   the tokens *do* determine, five for five, is
   **`nTryBlocks` = n₂₅₇** and **`maxState` = n₁ + 2·n₂₅₇** (§5.3).

---

## §1 Where the EH machinery is

Entry: the minted format strings. **c2 mints every EH symbol name** — none of
them travels in the IL.

| string VA | literal | referenced at | in function |
|---|---|---|---|
| `0x10b18c14` | `__ehfuncinfo$%s` | `0x10be0a84` | `FUN_10be0a0a` |
| `0x10b18c70` | `__unwindtable$%s` | `0x10be1323` | **`FUN_10be12c7`** |
| `0x10b18c5c` | `__tryblocktable$%s` | `0x10be13a8` | `FUN_10be12c7` |
| `0x10b18c38` | `__catchsym$%s$%d` | `0x10be18c6` | `FUN_10be12c7` |
| `0x10b18c4c` | `__estypeinfo$%s` | `0x10be1494` | `FUN_10be12c7` |
| `0x10b18c24` | `__estypeinfo$%s$%d` | `0x10be1d23` | `FUN_10be12c7` |
| `0x10b18c04` | `__catch$%s$%d` | `0x10be0534` | `FUN_10be04e7` |
| `0x10b18d00` | `__unwind$%s$%d` | `0x10be341c` | `FUN_10be32de` |
| `0x10b18c84` | `__unwindfunclet$%s$%d` | `0x10be21ed` | `FUN_10be1f3f` |
| `0x10b16624` | `__unwind$` | `0x10b99f3d` | **`FUN_10b99dfe`** |
| `0x10b16630` | `__catch$` | `0x10b99f36` | `FUN_10b99dfe` |
| `0x10b264e4` | `__CxxFrameHandler` | `0x10c22007` | **`FUN_10c21fd2`** |
| `0x10b18dc4` | `.pdata` | `0x10be76ac` | `FUN_10be76a8` |
| `0x10b18dbc` | `.xdata` | `0x10be7b4f` | `FUN_10be7b4b` |

**The PPC funclets use the SHORT forms.** The obj's symbols are `__catch$2554`
and `__unwind$2561` — prefix plus a bare number, not `%s$%d`. Those come from
`FUN_10b99dfe`, which is c2's **symbol-name formatter**, not from the `%s$%d`
strings above (which are the x86-shaped names and are not what this target
emits). §3.4.

| VA | what | confidence |
|---|---|---|
| **`0x10be76a8`** | **the `.pdata` section getter** — `FUN_10be7473(PTR_s__pdata_10c37c40, "PDATA", …, 0xe)`, then alignment `[+0x43] = 4` and `[+0x4b] = 1` | high |
| `0x10be7b4b` | the `.xdata` section getter, class `"XDATA"`, index `0x10`, same alignment | high |
| **`0x10c217fd`** | **the `.pdata` RECORD writer** — allocates an **8-byte** COMDAT (`FUN_10b9c655(6, 8, 4, 0, 4, 0x80)`), emits `{ADDR32 reloc, u32 immediate}`, defers the immediate | high |
| **`0x10bff785`** | **defer** — pushes a 6-field, `0x18`-byte fixup record onto the array at `DAT_10c385e4` | high |
| **`0x10bff811`** | **the deferred `.pdata` patch pass** — computes every unwind word once label addresses are final, then frees the array. Called once, from `0x10b34325` in `FUN_10b3421b` | high |
| **`0x10c21fd2`** | **the C++ EH `.pdata` driver** — the funclet split and the bit-31 predicate | high |
| `0x10c21b03` | the **SEH** `.pdata` driver (`__C_specific_handler`, `__try`/`__except`) — and the path a *non-EH* function takes (`FUN_10c217fd(…, 0, 0, …)`) | high |
| **`0x10be12c7`** | **the `__ehfuncinfo$` record builder** — 3 022 bytes; mints `__unwindtable$`, `__tryblocktable$`, `__catchsym$`, `__estypeinfo$` and writes the `FuncInfo` fields | high |
| **`0x10c220c9`** | **the ip-to-state (`$T…`) array emitter** | high |
| `0x10c219a6` | seeds every state-bearing node to `-2` | high |
| `0x10c219c4` | the state **propagation** pass (`node[+0x24]`, monotone max) | medium — read; its full fixpoint is not established |
| **`0x10b99dfe`** | **the symbol-name formatter** — `$T` `$S` `$SG` `$M` `$E` `$L{C,L,N}` `__unwind$` `__catch$` `__annotation$`, all from **one** field | high |

---

## §2 The `.pdata` record, field by field

### 2.1 The record is exactly two u32, and c2 builds it as `{reloc, immediate}`

`0x10c217fd` allocates a section of **8 bytes at alignment 4** and fills it with
one relocation item and one integer item:

* **`BeginAddress`** — an `ADDR32` against the symbol reached through
  `param_2[+0x28]`. For the main body that resolves to the **function symbol**
  (`Value = 8`); for a funclet, to its `__catch$`/`__unwind$` label. The obj
  carries the difference as an **addend** (`?f@@YAHH@Z+72`, `+128`, `+164` in the
  three cells below), so the port's model must be *function symbol + offset*,
  not one symbol per region.
* **the unwind word** — written as `0` at `0x10c218ca`/`0x10c218ce` and patched
  later.

**No third word and no `.xdata` for the body.** `.xdata` exists in the image
(`0x10be7b4b`) but the workload's only users of it are throw-side records
(w-eh5: 67 objs, all STLport). PREREG **P2.1 HIT.**

### 2.2 The unwind word — the whole computation, at `0x10bff811`

```
   ebx = base.addr                                  ; [ecx]      = p2, the region's base symbol
   eax = (prologEnd.addr - base.addr) >> 2           ; 0x10bff83a..0x10bff845
   if (handler != 0 && entry5 == 0) eax -= 2         ; 0x10bff848..0x10bff852
   edi = eax & 0xff                                  ; 0x10bff85f      <- PROLOG, 8 bits
   eax = (end.addr - base.addr) >> 2                 ; 0x10bff859..0x10bff869
   if (handler != 0 && entry5 == 0) eax -= 2         ; 0x10bff86c..0x10bff876
   ebx = (handler != 0)                              ; 0x10bff87d  setne bl
   eax &= 0x3fffff                                   ; 0x10bff880      <- LENGTH, 22 bits
   ebx = (ebx << 23) | eax                           ; 0x10bff88b, 0x10bff88e
   ebx |= 0x400000                                   ; 0x10bff893      <- UNCONDITIONAL
   word = edi | (ebx << 8)                           ; 0x10bff899, 0x10bff89c
   [item + 0x0e] = word                              ; 0x10bff8a1
```

> **`word = (hasHandler << 31) | (1 << 30) | ((len_words & 0x3FFFFF) << 8) | (prolog_words & 0xFF)`**

* **Bit 30 is set unconditionally**, at `0x10bff893`, for **every** `.pdata`
  record c2 emits — EH or not. The port already ships this constant
  (`c2-core::coff::pdata::UNWIND_THIRTY_TWO_BIT = 0x4000_0000`), reached from
  captures; this is an **independent confirmation**, not new information, and no
  DISCLOSURE row is owed for it.
* The length field is **22 bits**, not "the rest of the word" — `and eax,0x3fffff`
  at `0x10bff880`. The prolog field is **8 bits**. Nothing in the published
  black-box doc pins either width; both are new.
* **`hasHandler` is the same value that decides whether the two-word prefix is
  emitted** — `param_4` of `0x10c217fd`, tested at `0x10c21907` for the prefix
  relocations and passed through as `p5` to the deferral. One predicate, two
  consequences. §5.2 is the obj check.

### 2.3 The `−2`, and why it is degenerate black-box

The two `sub eax,2` (`0x10bff852`, `0x10bff876`) fire iff
`handler != 0 && param_1->kind == 0x18`, i.e. **only for the main-body record of
a function that has a handler prefix** (`0x18` is the function-start node kind;
`0x10c218da` computes the flag, `0x10bff7fd` stores its complement).

The two words removed are the prefix. Both models fit every obj this lane read —
*base = COMDAT start, minus 2* and *base = function symbol (`Value = 8`), no
correction* — because the prefix is always exactly two words, so black-box work
**cannot** separate them. The whitebox reading picks the first. Filed as
navigation under PREREG **D1**; a port only needs the observable, which is that
the main record's length starts at the *code*, not at the COMDAT.

### 2.4 Section order: reverse

E3 (§5.1) has three regions and therefore three `.pdata` COMDATs. In the obj they
appear **unwind funclet, catch funclet, main body** — the exact reverse of their
`.text` order — while `cl /FAsc`'s listing emits them **main, catch, unwind**
(`work/wb-eh/e3.cod`, `$T2596` / `$T2599` / `$T2602`, ascending). So:

> **The `.pdata` COMDAT order in the obj is the REVERSE of the `.text` region
> order, and the reverse of the label-counter order.**

`EH_CRITICAL_PATH.md` §2 observed this on two records and called it "the same
reverse-emission order as the `.rdata` pool"; three records make it a rule with a
direction rather than a pair.

---

## §3 The record set

### 3.1 `__ehfuncinfo$` — 9 dwords, magic first

`0x10be12c7` writes, in order (each `FUN_10b989e2(1)` is a 4-byte immediate item,
each `FUN_10b989e2(2)` a 4-byte `ADDR32` item):

| # | field | source in `0x10be12c7` |
|---:|---|---|
| 0 | **`magic = 0x19930522`** | `mov DWORD PTR [eax+0xe],0x19930522` at **`0x10be1425`** |
| 1 | `maxState` | `DAT_10c434cc` |
| 2 | `pUnwindMap` | the `__unwindtable$%s` COMDAT's symbol, or `0` |
| 3 | `nTryBlocks` | `DAT_10c434c8`, **gated on option-word bit `0x400000`** (`DAT_10c2e2f4[0x25]`) |
| 4 | `pTryBlockMap` | the `__tryblocktable$%s` COMDAT's symbol, or `0` |
| 5 | `nIPMapEntries` | written by `0x10c220c9` |
| 6 | `pIPtoStateMap` | ditto |
| 7 | `pESTypeList` | the `__estypeinfo$%s` COMDAT, gated on `param_1[0x97] & 1` |
| 8 | `EHFlags` | `(param_1[0x25] & 8) != 0` |

PREREG **P2.4 HIT**, including the exact magic: `0x19930522` is the **only**
`19930xxx` immediate in the image.

`__unwindtable$` is `maxState × 8` bytes (`DAT_10c434cc << 3`);
`__tryblocktable$` is `nTryBlocks × 0x14`; each `__catchsym$` handler array is
`n × 0x10`. All three are created by the same `FUN_10b9c655(6, size, 7, 0, 4,
0x80)` call shape as `.pdata` but with the third argument **7** rather than 4.

### 3.2 The ip-to-state array (`0x10c220c9`)

```
prev = -1
for each node in the function's list:
    if (node[+9] & 1) == 0: continue                 # not a state-bearing node
    s = node[+0x24]
    if s == prev or s == -2: continue                # dedup, and skip "unset"
    emit  ADDR32 -> label(node)     (4 bytes)
    emit  i32 s                     (4 bytes)
    prev = s ; n += 1
if n: section = FUN_10b9c655(6, n << 3, 7, 0, 4, 0x80)   # 8 bytes per entry
      FuncInfo[5] = n ; FuncInfo[6] = &section
else: FuncInfo[5] = 0 (kind 2) ; FuncInfo[6] = 0
```

**Two dedup rules and a sentinel**, all three of which a port must reproduce:
consecutive equal states collapse, `-2` means "no state assigned", and the array
is omitted entirely at `n == 0`.

### 3.3 The funclet split and bit 31 — `0x10c21fd2`

```
if (hasCxxEH) { h = extern("__CxxFrameHandler"); h[0x31] = 'V'; }   0x10c22007
base = the function-start node (kind 0x18); isUnwind = false
for each node:
    cut = (node.kind == 0x1b && node.sym[0x31] in {'V','T'})        0x10c22046..0x10c2205a
       || (node.kind == 0x19)                                       0x10c2205c
    if !cut: continue
    emit_pdata(regionStart, base, node.label,
               isUnwind ? 0 : h,                                    0x10c22080..0x10c22089
               isUnwind ? 0 : ehdata,                               0x10c2208a..0x10c22095
               prologMark, ...)                                     0x10c2209b
    if node.kind == 0x1b:
        base = node.sym; isUnwind = (node.sym[0x31] == 'T')         0x10c220ab  cmp [ebx+0x31],0x54
        regionStart = node
```

Read in words:

> **c2 cuts `.text` into `.pdata` regions at every `__catch$` (`'V'`) and
> `__unwind$` (`'T'`) label, plus the function end. A region that begins at an
> `__unwind$` label gets its handler and handler-data arguments forced to zero —
> so it gets NO two-word prefix and its unwind word's bit 31 is CLEAR. Every
> other region of an EH function gets both.**

The masking is the idiom `neg / sbb / not / and` at `0x10c22080`–`0x10c22095`,
applied twice, to the two arguments that `0x10c21907` also tests to decide
whether to emit the prefix relocations. **Bit 31 and the prefix are one
predicate**, which is why the four-record reading in `EH_CRITICAL_PATH.md` §2
held: it was reading the same bit from the other side.

One correction to a decompiler artifact, because a code lane would trip on it:
at `0x10c2206b` the handler-**data** argument is chosen by
`cmp BYTE PTR [ecx+8],0x18` — the main body and a funclet pass **different**
values. Ghidra collapses both into one variable.

### 3.4 The funclet symbol numbers come from the label counter — `0x10b99dfe`

`FUN_10b99dfe` formats a symbol's text name into a `0x1020` buffer by dispatching
on `sym[+0x30]` (kind) and `sym[+0x31]` (sub-kind):

| kind | sub-kind | name |
|---:|---|---|
| 3 | `'T'` (0x54) | `__unwind$` + decimal (`0x10b99f3d`) |
| 3 | `'V'` (0x56) | `__catch$` + decimal (`0x10b99f36`) |
| 3 | `'W'` (0x57) | `$M` + decimal |
| 3 | `'Z'` (0x5a) | `__annotation$` + decimal |
| 3 | `0` | `<name>$` + decimal |
| 3 | other, unnamed | `$L` + `C`\|`L`\|`N` + decimal(`sym[+0x3f]`) [+ `@` + TU] |
| 1 | `'$'` / `'%'` / other | `$S` / `$SG` / **`$T`** + decimal (`0x10b9a04d`–`0x10b9a07d`) |
| 4 | — | `$E` + decimal |

**Every one of them takes its number from `sym[+0x28]`, converted by the same
`FUN_10c1e739(v, buf, cap, 10)` at `0x10b9a08e`.** PREREG **P2.5 CONFIRMED**:
`__catch$N` / `__unwind$N` are not a private EH counter — they are `$M`/`$T`'s
counter, so `docs/LABEL_COUNTER.md`'s arithmetic is the arithmetic that must
predict them. E3's obj is consistent: `$M2585…$M2590` (the six ip2state labels),
`$T2591` (the ip2state array), `$T2596`/`$T2599`/`$T2602` (the three `.pdata`
COMDATs, stride 3), `__catch$2575`, `__unwind$2576`.

---

## §4 Main.cpp — the chain, un-stuck

### 4.1 The row, re-measured at this tip

`c2rs gap --list` on `src/Main.cpp` at the workload flags: **one** emitted
function, verdict `vocab-gap`, blocked at **`param-width-undetermined:mid`**,
class `cflow-straight` / `eh-state1` / `calls-2plus` / `disp-formals-width`.
`expr-convert-no-value-0x2C` is **not** its first blocker at master; it is where
the *hatched* ladder lands (w-front3 §2.1, board #1354/#1469).

The blocking window the census prints, with `>` on the blocking byte:

```
00 4f 01 03 53 53 26 0c 0a 46 2d 0b 0a 2d 0a 0a >4c< 4f 11 53 4f 01 04 26 fb 09 26 0e 0a 2c a6 43 81 20 00 99 86 43 8b 20
```

The port runs out of formals-header before the body marker. **The `4C` is not a
construct it fails to decode — it is the end of a header it did not finish.**

### 4.2 The whole body, tokenised with c2's own table

`work/wb-eh/extok.py` reads the 192-entry class table straight out of the pinned
image and applies the 29 class arms plus the primitives. On Main.cpp's body it
walks **41 tokens** from the `4C 4F 11` marker to the `4D` end-of-stream with no
desync:

```
4C · 4F 11 · 53 · 4F 01 04
26 fb 09 · 26 0e 0a · 2C a6 43 81 20 00 · 99 86 43 8b 20 00 · BD a6 43 81 20 00 80 06 10 00 00
B9 0b 0a 86 43 84 20 · 55 86 43 84 20 · B9 0a 0a 86 41 74 · 55 86 41 74
4C · 26 fd 09 · 26 0e 0a · 2C … · 99 … · BD 82 07 03 00 80 03 10 00 00
4C · 5C a6 43 81 20 01            <-- the ONLY EH token in the body
4B · 4F 01 05 · 26 01 0a · 26 0e 0a · 2C … · 99 … · BD …
4C · 4B · 4F 01 06 · 5E 01 21 · 4B · 54 02 · 29 0d 0a · 4F 12 · 47 · 54 01 · 54 00
4F 02 20 00 · 4F 01 07 · 4D
```

It is **three member calls in a straight line, one `5C`, and scope markers.**
That is the whole of the "highest-worth frontier row".

### 4.3 The NAMED rung

> **The construct is `2C a6 43 81 20 00`. Per c2's reader it is opcode `0x2C`,
> operand class `05` (`DAT_10b25e48[0x2C] = 0x05`), whose arm at `0x10b3d6b2`
> reads one `TYPE` and then falls into `0x10b3d694`, a single `GetByte` whose
> result is stored NOWHERE. The `TYPE` decodes to the 3-byte word `0x432681` —
> class `1` (signed), size index `3` (4 bytes) — i.e. `int`; the `20` is the
> globally gated id skip; the `00` is the discarded byte. `0x2C` is a CONVERT
> whose source operand is not in the stream at all: `.ex` is a stack machine and
> the convert takes the top of stack.**
>
> **So `expr-convert-no-value` is not a rung of the IL. It is the port's `cstack`
> being empty, and it is empty because the port does not model the value the two
> preceding `26` (call-in-expr) tokens leave. The rung is `26`'s VALUE, not
> `2C`'s type — and one rung earlier still, at master, it is the formals header.**

Three consequences, each checkable:

1. **The port's width rule for `0x2C` is wrong and the disagreement is live in
   principle.** c2 reads **one raw byte**; the port reads a varint. They agree
   below `0x80`. Every `0x2C` this lane decoded (Main.cpp ×3, E1, E2, E3 ×4)
   carries payload `0x00`. **PREREG P3.5 is a MISS**: this lane found **no** site
   with a payload `≥ 0x80`, so the desync stays latent (§8).
2. **The byte is discarded**, so a port needs only its width — one byte —
   and never its value. That is a *cheaper* fix than the port's current model.
3. `0x2C` shares class `05` with `0x34`, and both are `TYPE` + discarded byte.

### 4.4 An instrument observation, reported not worked around

`c2rs census src/Main.cpp` names the single emitted function
**`?Run@App@@QAAXXZ`**. The reference obj's only `.text` COMDAT defines
**`main`**, and `?Run@App@@QAAXXZ` appears in it as an **external `bl` target**.
The three calls in the body (§4.2) line up with `main`'s three call sites
(`??0App`, `?Run@App`, `??1App`), so the body is `main` and the *name* is
mis-attributed. `WB_READER_FINDINGS.md` §1's own table names the same row's
example function `main`. Board **#1870**; nothing in this lane depends on it.

---

## §5 The obj-checks

Real `c2.dll` under wibo at the workload profile
`/nologo /wd4355 /wd4164 /c /GR /O1 /Oi /EHsc`, decoded with the project's
existing `scripts/gt_eh.py`. Sources in `work/wb-eh/src/` (uncommitted fixture
cells, deliberately **not** promoted to `fixtures/cpp/`).

### 5.1 The cells

| cell | source | funclets |
|---|---|---|
| **E1** | `int f(int a){ try { return g(a); } catch(int e){ return e+1; } }` | catch |
| **E2** | `struct S{S();~S();int m;}; int f(int a){ S s; return g(a)+s.m; }` | unwind |
| **E3** | both — a destructible local **and** a `try`/`catch`; plus `int leaf(int a){return a+1;}` as the E4 control | catch **and** unwind |
| **M** | the workload's own `src/Main.cpp`, at the workload flags | unwind |
| **E6** | the §5.5 third-witness cell, run in-lane: **two** destructible locals, a **nested** `try`, `catch(int)` **and** `catch(...)` | 2 catch **and** 2 unwind |

### 5.2 E3 separates the bit-31 rivals — the discriminating cell

E3's `?f@@YAHH@Z` is 212 bytes of `.text` with **three** `.pdata` COMDATs:

| region | `.text` | `BeginAddress` | word | bit31 | prefix at |
|---|---|---|---|:-:|---|
| `__unwind$2576` | `0xac..0xd4` | `?f+164` | `0x40000a04` | **0** | — |
| `__catch$2575` | `0x88..0xac` | `?f+128` | `0xc0000902` | **1** | `.text+0x80` |
| main body | `0x08..0x80` | `?f+0` | `0xc0001e08` | **1** | `.text+0x00` |

| rival, registered before the run | predicts | verdict |
|---|---|---|
| **the prefix reading** (registered as expected to survive) | SET · SET · CLEAR | **SURVIVES** |
| **(R-a)** bit 31 = "the function has a language handler" | SET · SET · SET | **REFUTED** |
| **(R-b)** bit 31 = "the region has a prologue" | SET · SET · SET (the unwind funclet's prolog field is **4**, non-zero) | **REFUTED** |
| **(R-c)** bit 31 = "the region saves LR" | not separated by this cell | **not established**, and said so |

**Two of the three rivals fall on one cell**, which meets the PREREG's asserted
minimum of 2 discriminating cells. E1 and E2 replicate the two ends separately
(E1 catch-only: SET/SET; E2 unwind-only: SET/CLEAR), so the reading has **six**
records over three objs, plus **two** more from `Main.cpp`.

**E6 replicates it at multiplicity.** `?f@@YAHH@Z` there is 356 bytes with
**five** `.pdata` COMDATs — and every one falls where the reading says:

| region | `.text` | word | bit31 | prefix |
|---|---|---|:-:|---|
| `__unwind$2589` | `0x13c..0x164` | `0x40000a04` | **0** | — |
| `__unwind$2588` | `0x114..0x13c` | `0x40000a04` | **0** | — |
| `__catch$2587` (`catch(...)`) | `0x100..0x114` | `0xc0000501` | **1** | `.text+0xf8` |
| `__catch$2586` (`catch(int)`) | `0xd4..0xf8` | `0xc0000902` | **1** | `.text+0xcc` |
| main body | `0x08..0xcc` | `0xc0003108` | **1** | `.text+0x00` |

**Thirteen records over five objs, two catch funclets and two unwind funclets in
one function, zero exceptions.** The five COMDATs are again in exact reverse
`.text` order (`0x13c`, `0x114`, `0x100`, `0xd4`, `0x08`), which takes §2.4's
rule from three witnesses to five. A `catch(...)` behaves as a `catch` for this
purpose (`adjectives = 0x40`, `pType = 0`), which is the one thing a reader might
have guessed the other way.

**E4, the null control**: `?leaf@@YAHH@Z` is emitted into its own 8-byte `.text`
COMDAT in the same obj and gets **no `.pdata` record at all**. A frameless leaf
is not a `.pdata` row, so E3's three records cannot be a constant.

**Length arithmetic, checked on all eight records**: every one satisfies
`len_words = (regionEnd − regionBase)/4` and `prolog_words =
(prologMark − regionBase)/4` with the region's own entry symbol as the base
(§2.3 is why this does not separate the two models).

### 5.3 `5C` is NOT the ip-to-state map — P3.3 REFUTED

Five bodies, IL tokenised with `extok.py`, objs decoded with `gt_eh.py`:

| cell | `5C` tokens (i32c operand) | `n₁` | `n₂₅₇` | obj `maxState` | `nTryBlocks` | `nIPMapEntries` | obj ip2state states |
|---|---|---:|---:|---:|---:|---:|---|
| **E1** (catch only) | `0x101` | 0 | 1 | **2** | **1** | 1 | `0` |
| **E2** (unwind only) | `1` | 1 | 0 | **1** | **0** | 2 | `0,-1` |
| **E3** (both) | `1`, `0x101` | 1 | 1 | **3** | **1** | 6 | `1,0,1,-1,0,-1` |
| **M** (`Main.cpp`) | `1` | 1 | 0 | **1** | **0** | 2 | `0,-1` |
| **E6** (2 dtors, nested try, `catch(...)`) | `1`, `1`, `0x101`, `0x101` | 2 | 2 | **6** | **2** | 18 | 18 entries |

**The negative half, which is the one that prices `Main.cpp`:**

* **The count does not match**: 1 `5C` → 2 map entries (E2, M); 2 → 6 (E3);
  **4 → 18** (E6).
* **The values do not match**: E1's only `5C` says `0x101`; its map says `0`.
  `0x101` never appears as a state in any of the five objs.
* So the **ip-to-state map is derived, not transcribed** — `0x10c219a6` seeds
  every state-bearing node to `-2`, `0x10c219c4` propagates forward,
  `0x10c220c9` emits one entry per *change*. A port that reads `5C` and writes it
  out has the wrong architecture, not a missing constant.

**The positive half, and it survived the third-witness cell.** The operand takes
exactly **two** values across five bodies and nine tokens — `1` where the region
covers a destructible object, `0x101` where it is a `try` block — and E6 was
written *specifically* to produce a third (a nested try and a `catch(...)`) and
did not. Four-plus witnesses per side now, which clears #1767's bar, and two
counts fall straight out:

> **`nTryBlocks` = the number of `5C` tokens whose operand is `0x101`** — 1, 0,
> 1, 0, **2** against the objs' 1, 0, 1, 0, **2**. Five for five.
>
> **`maxState` = n₁ + 2·n₂₅₇** — 2, 1, 3, 1, **6** against the objs' 2, 1, 3, 1,
> **6**. Five for five, over four distinct `(n₁, n₂₅₇)` configurations.

`maxState` is a 2-parameter fit on 4 configurations, so it is a *rule with one
degree of freedom left*, not a proof; it is registered here so the next cell
(three trys, or a try with two catches) either confirms or kills it.

One more field falls out of E6 for free: the `5C`'s **TYPE names the type of the
object whose lifetime the region covers** — E6's two unwind `5C`s carry
`a6 43 81` and `a6 43 8a` (its two distinct local types `S` and `T`), while every
`try` `5C` in all five bodies carries the same `86 41 74`.

> **The EH record set is DERIVED, not transcribed.** `0x10c219a6` seeds every
> state-bearing node to `-2`, `0x10c219c4` propagates state numbers forward over
> the node list, and `0x10c220c9` emits one entry per *change*. A port that reads
> `5C` and writes it out has the wrong architecture, not a missing constant.

PREREG **P3.3's** second half ("`0x5C`'s operand is the state that lands in the
ip-to-state map") is **a MISS in the optimistic direction and is retracted.**
The first half (that the chain lands on `0x5C` once `2C` is lifted) was board
#1354's, not this lane's, and is confirmed: `5C` is the body's only EH token.

### 5.4 The listing seam (E5)

`c2rs listing` on E3 (`work/wb-eh/e3.cod`) narrates the whole record set by
name — `EXTRN __CxxFrameHandler:PROC`, the three `.pdata` COMDATs with their
literal words `0c0001e08H` / `0c0000902H` / `040000a04H`, `__unwindtable$`,
`__catchsym$…$3`, `__tryblocktable$`, `__ehfuncinfo$` with `019930522H`, and the
`$T2591` ip2state array with its six `$M`/state pairs. **Every field this lane
read out of the disassembly is printed by c2 itself.** One difference worth
recording: the listing emits `.pdata` **main → catch → unwind**, the obj emits
them **reversed** (§2.4).

### 5.5 Designed and not run

* ~~**A third `5C` value.**~~ **RUN in-lane as E6** — nested `try`, `catch(...)`,
  two destructible locals. **No third value appeared**, and the cell instead
  produced the two counting rules in §5.3. The remaining open cell is narrower:
  **three try blocks, or one try with two `catch` clauses**, which is what
  separates `maxState = n₁ + 2·n₂₅₇` from its rivals.
* **A `0x2C` payload `≥ 0x80`.** Not found in any of the five bodies read here;
  `docs/IL_CAST_CONVERT.md`'s corpus is the place to query. Until one exists the
  port's varint reading of `0x2C` cannot be refuted by an obj.
* **The `−2`.** Unfalsifiable black-box while the prefix is always two words
  (§2.3).
* **`maxState`'s arithmetic.** Three points (1, 2, 3) is a line through anything.

---

## §6 The priced route to `Main.cpp` — a priced DECLINE

Enumerated against this lane's own reading, in the order a conversion lane would
hit them. `Main.cpp` is **one** function.

| # | refusal | where | why it is real |
|---:|---|---|---|
| R1 | `param-width-undetermined:mid` | `c2-il` formals header | measured at master (§4.1); the row never reaches its body today |
| R2 | the `26 … 2C … 99 … BD` member-call spine's **value model**, ×3 | `c2-il` `mcall` | this is what leaves `cstack` empty (§4.3); board #1534 measures the family at 36,751 emitted and records that it has never had a whole-production counterfactual |
| R3 | `0x2C` width: one raw byte, not a varint | `c2-il` `shapes::control_flow::operand` | §4.3; cheap, and latent (§5.5) |
| R4 | `5C`'s meaning | `c2-il` + a new IR pass | §5.3 — **not a decode; a derivation** |
| R5 | the EH **state propagation** pass | `c2-core`, new | `0x10c219a6` + `0x10c219c4`; no port equivalent exists |
| R6 | the ip-to-state array + its two dedup rules and `-2` sentinel | `c2-core/coff`, new | §3.2 |
| R7 | `__ehfuncinfo$` — 9 dwords, magic, five sub-records | `c2-core/coff`, new | §3.1 |
| R8 | `__unwindtable$` (`maxState × 8`), `__tryblocktable$` (`n × 0x14`), `__catchsym$` (`n × 0x10`) | `c2-core/coff`, new | §3.1 — Main.cpp needs the first only |
| R9 | the EH `.rdata` COMDAT at **`Selection = 5`, associative to `.text`** | `c2-core/coff` | `EH_RECORDS.md` §3; the writer's `.rdata` is `Selection = 2` today |
| R10 | the 8-byte `.text` prefix and **function symbol `Value = 8`** | `c2-core/coff` | `EH_CRITICAL_PATH.md` §1: "every consumer of *the function starts at 0* is wrong for an EH function" |
| R11 | the `__CxxFrameHandler` external, minted by c2 and absent from the IL | `c2-core/coff` | §1 |
| R12 | **two** `.pdata` COMDATs, in **reverse** region order, `Selection = 5` | `c2-core/coff` | §2.4; the writer emits one today |
| R13 | the unwind word's bit 31 + the funclet split | `c2-core` | §2.2, §3.3 |
| R14 | the `__unwind$N` funclet itself — a second code region with its own prologue | `c2-core` codegen | E2/M: 10 words, prolog 4. No emitter exists for a second region |
| R15 | the label lead: **six** new labels (`$M`×2, `$T`×1, `$T`×2 for the two `.pdata`, `__unwind$`×1) allocated in an order the counter model has never been graded on | `c2-il::label_lead` | §3.4; `LABEL_COUNTER.md` has no EH row and #1761's rule is already REFUTED (w-xlr §3) |

**Fifteen named refusals, of which eleven are in seams that do not exist.**
PREREG **P3.4 predicted ≥ 12 and a priced decline: HIT.**

> **The decline, stated once.** `Main.cpp` is not a transcription away. R4–R7 are
> a *pass* — c2 derives the EH state map rather than reading it — and R14 is a
> second code region. The correct next lane is **not** `Main.cpp`; it is
> whichever of R2 (the member-call value model, 36,751 emitted functions) or the
> `5C` third-witness cell (§5.5) is cheapest, because R2 is shared with most of
> the frontier and the EH work is shared with none of it until R2 lands.

**What EH is worth, corrected.** Board #1780's ranking puts `Main.cpp` first
"because factor D over 740 objs". Factor D is the *obj-shape* factor, and w-eh5
already established that EH costs factor **C** zero. This lane adds the other
half: the 740 objs share the *record shapes* of §3, but each one still needs R2
and its own function bytes, so **the 740 is a population, not a multiplier.**

---

## §7 Pre-drafted DISCLOSURE rows

**None of these is adopted.** Drafts only, for a later code lane to carry in the
same commit as the code change. Only findings that survived §5 get a row.

| # | Kind | What would be adopted | Address in `c2.dll` | Notes |
|---|---|---|---|---|
| **W-EH-1** | **adoption** | **The `.pdata` unwind word's bit layout**: `hasHandler<<31 \| 1<<30 \| (len_words & 0x3FFFFF)<<8 \| (prolog_words & 0xFF)`, where `len_words` and `prolog_words` are word counts from the region's own entry, and `hasHandler` is the same predicate that emits the two-word `{handler, handlerData}` prefix. The **22-bit** length field and the **8-bit** prolog field are the parts no black-box work has pinned. | **`0x10bff811`** (the pass), `0x10bff87d` (`setne` → bit 31), **`0x10bff880`** (`and eax,0x3fffff`), `0x10bff85f` (`and edi,0xff`), **`0x10bff893`** (`or ebx,0x400000`), `0x10bff899` (`shl ebx,8`), `0x10bff8a1` (the store) | **Obj-confirmed** by §5.2 across eight records / four objs. **The black-box alternative was tried and is partially sufficient**: the port already carries `0x4000_0000` and the `len<<8 \| prolog` shape from captures, so only the two field WIDTHS and the bit-31 predicate would be new. A code lane that needs neither should take **no** row. |
| **W-EH-2** | **adoption** | **The funclet split and bit 31**: c2 cuts `.text` into `.pdata` regions at every `__catch$`/`__unwind$` label plus the function end; a region beginning at an `__unwind$` label has its handler and handler-data arguments forced to zero, so it gets no prefix and bit 31 clear. `.pdata` COMDATs appear in the obj in **reverse** region order. | **`0x10c21fd2`**, `0x10c22046`–`0x10c2205f` (the cut predicate), **`0x10c220ab`** (`cmp [ebx+0x31],0x54` — the `'T'` test), **`0x10c22080`–`0x10c22095`** (the two maskings), `0x10c22007` (`__CxxFrameHandler`) | **Obj-confirmed** by §5.2's three-record cell, which refutes two registered rivals. |
| **W-EH-3** | **adoption** | **`__ehfuncinfo$` is 9 dwords beginning with `0x19930522`**, in the field order of §3.1, with `__unwindtable$` = `maxState × 8`, `__tryblocktable$` = `n × 0x14`, `__catchsym$` = `n × 0x10`, and an ip-to-state array of `n × 8` `{ADDR32 label, i32 state}` pairs emitted one per state *change*, with `-2` as the "unset" sentinel and the array omitted entirely at `n == 0`. | **`0x10be12c7`** (the builder), **`0x10be1425`** (the magic), **`0x10c220c9`** (the ip2state emitter, its dedup at the `iVar1 != local_10 && iVar1 != -2` test) | **Obj-confirmed** by §5.1–§5.3 and, independently, by `cl /FAsc` printing every field by name (§5.4). **The black-box alternative was tried FIRST and it is what `EH_RECORDS.md` already is** — the field *layout* is fully established black-box; what only the disassembly gives is the **dedup rule and the `-2` sentinel**, which no obj exhibits directly. A code lane should take this row for those two facts alone. |
| **W-EH-4** | **route** | **The funclet symbols `__catch$N` / `__unwind$N` take `N` from the same `sym[+0x28]` label counter as `$M` / `$T` / `$S` / `$E`**, formatted by one function. | **`0x10b99dfe`**, `0x10b99f36` / `0x10b99f3d` (the two prefixes), `0x10b9a04d`–`0x10b9a07d` (`$T`/`$S`/`$SG`), **`0x10b9a08e`** (the shared decimal writer) | Logged `route:` per the grey-zone rule — the port already models one counter and `LABEL_COUNTER.md` was derived black-box. **No value or layout is copied**; the row exists so a lane extending the counter to EH knows the search was not blind. |

**Deliberately not drafted**: a row for `0x2C`'s operand class. It belongs to
`WB_READER_FINDINGS.md`'s class-table family, §4.3 corrects the port in the
*cheaper* direction (a fixed byte, not a varint), and it is **not obj-confirmed**
— no site with a payload `≥ 0x80` exists in anything this lane read.

---

## §8 PREREG scored

Board #770's streak stood at ~10 optimistic / 2 pessimistic / 1 hit.

| # | prediction | outcome |
|---|---|---|
| P1.1 | ≥ 5 of 7 EH prefixes are literal strings with an xref | **HIT** — 7 of 7 found, plus `__estypeinfo$` ×2 and `__unwindfunclet$` (§1) |
| P1.2 | `__CxxFrameHandler` is a literal in `c2.dll` | **HIT** — `0x10b264e4` |
| P1.3 | a single name-minting helper shared by ≥ 3 prefixes | **HIT, twice over** — `FUN_10be12c7` `sprintf_s`-mints four of them; `FUN_10b99dfe` formats **nine** symbol shapes from one field |
| P1.4 | one `.pdata` writer, EH-ness enters as a flag | **HIT** — `0x10c217fd`, reached by both the C++ EH driver and the SEH/no-EH driver; the handler arrives as an argument |
| P1.5 | bit 31 appears as a distinct constant/`or` | **HIT** — `setne bl` + `shl ebx,0x17` at `0x10bff87d`/`0x10bff88b`, separate from the `<<8` |
| P2.1 | two u32, no third word, no body `.xdata` | **HIT** — the section is allocated at literal size 8 |
| P2.2 | `bit31 \| len<<8 \| prolog` | **HIT** — and extended: the field widths are 22 and 8, and **bit 30 is unconditional** |
| **P2.3** | **bit 31 = "preceded by the handler prefix"; rivals R-a/R-b/R-c registered** | **HIT** — survives E3; **R-a and R-b REFUTED** on one cell; R-c reported **not established** |
| P2.4 | `__ehfuncinfo$` opens with a magic; **`0x19930522`** named | **HIT**, to the exact value, and it is the only such immediate in the image |
| P2.5 | funclet numbers come from the `$M`/`$T` counter, not a private one | **HIT** — one field `sym[+0x28]`, one formatter, one decimal writer |
| P2.6 | all EH relocations are `ADDR32` | **HIT** — every relocation in §5's four objs is `ADDR32`; none is `ADDR32NB` or `SECREL` |
| **P3.1** | **the stuck rung is a port-side `cstack` artifact, not an EH construct** (registered *pessimistic*) | **HIT** — §4.3, and the master-tip blocker is earlier still |
| P3.2 | `0x2C`'s byte is a conversion-kind selector, not a symbol id or a length | **MISS, and in the useful direction** — the byte is **discarded** (`0x10b3d694`). Rival (R-d) "the port is right, it is a varint" is refuted by the class table; (R-e) is moot |
| **P3.3** | **the chain lands on `0x5C`; `0x5C`'s operand is the ip-to-state state** | **SPLIT: first half confirmed (and it was #1354's, not mine); second half a MISS, optimistic, REFUTED by five objs** (§5.3) and retracted. What replaced it — `nTryBlocks = n₂₅₇` and `maxState = n₁ + 2·n₂₅₇` — was **not** registered and is scored as an unregistered find, not as a rescue of P3.3 |
| **P3.4** | **≥ 12 named refusals and a priced decline** (registered *pessimistic*) | **HIT** — 15 (§6) |
| P3.5 | ≥ 1 `0x2C` site in the workload with payload ≥ `0x80` | **MISS, optimistic** — 8 sites read, all payload `0x00`; the desync stays latent |
| E1 | E1's obj shape, port refuses without `mismatch` | **HIT** — 2 `.pdata`, 96-byte EH `.rdata`, `??_R0H@8` in `.data`, function symbol `Value = 8`; the port reports a blocker and **no `mismatch`** |
| E2 | unwind funclet bit 31 CLEAR, main SET | **HIT** |
| E3 | catch SET · unwind CLEAR · main SET, separating ≥ 2 rivals | **HIT** — 2 rivals refuted |
| E4 | the leaf control's bit 31 clear | **MISS, in a way worth keeping** — the leaf gets **no `.pdata` record at all**, so the control is stronger than registered but the registered observable did not exist |
| E5 | `/FAsc` names the funclets and the record layout | **HIT** — §5.4 |

**16 hits, 4 misses, 1 split.** Of the four misses, **three are optimistic**
(P3.3, P3.5, E4) and one (P3.2) is a miss whose truth makes the port's job
*easier*. The direction of the optimistic misses is the same one
`WB_READER_FINDINGS.md` §7 named: **assuming the IL contains a fact that c2
actually computes.** P3.3 is that belief at the level of a whole record set, and
it is the most expensive version of it this project has registered.

**The decline clauses**: D1 fired (§2.3, the `−2` is navigation-only). D2 did not
fire (the bit-31 rule survived). D3 did not fire (P3.1 held). D4 held — nothing
under `crates/` was touched. D5 did not fire (the price is 15 ≥ 12). D6 held —
every re-measured number in §5 is labelled as such and agrees with
`EH_CRITICAL_PATH.md` §2 to the byte.

---

## §9 What this lane did NOT do

* **It did not re-tread `.rdata$r`.** w-eh5 settled it; RTTI appears here only as
  `??_R0H@8` in `.data`, which is the type descriptor and not the record set.
* **It did not touch the throw side** (`.xdata$x`, `_TI`/`_CTA`/`_CT`). Located
  (`0x10be7b4b`) and left.
* **It did not PROVE `maxState = n₁ + 2·n₂₅₇`.** Two parameters fitted on four
  distinct configurations, five objs — a rule with one degree of freedom left,
  and §5.5 names the cell that spends it. `nTryBlocks = n₂₅₇` is the stronger of
  the pair (one parameter, five for five, including a zero cell twice).
* **It did not establish that `0x5C` has only two operand values.** E6 was
  written to produce a third and did not; that is evidence, not closure. A
  `__try`, a function-try-block, or a `throw` expression are all untried.
* **It did not model the state-propagation fixpoint** (`0x10c219c4`). It read the
  seed, the monotone update and the consumer, and stopped.
* **It did not adopt anything.** No file under `crates/` was modified, and the
  fixture cells stayed in `work/wb-eh/src/` rather than `fixtures/cpp/`.
* **It did not price any other frontier row.** §6's last paragraph is a
  redirection, not a survey.
