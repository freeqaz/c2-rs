# WB-A `wb-reader` — the frontier's 48 reader refusals, read off c2's own `.ex` reader

> **PROVENANCE — DISASSEMBLY-DERIVED.** Every address below is an absolute VA in
> the exact image pinned in [`C2_MAP_METHOD.md`](C2_MAP_METHOD.md) §0
> (`sha256 c80981…6258`); verify it before trusting one. This file is
> **navigation**. Nothing here is adopted into `crates/` — see
> [`DISCLOSURE.md`](DISCLOSURE.md) and §6 for the pre-drafted rows a later code
> lane would carry.
>
> **`high` confidence means "I read the instructions correctly", not "this is
> what c2 does"** (method doc §7, the `.bss` retraction). §5 is where the objs
> get their say; two readings in §3 were **refuted or corrected by them** and are
> marked as such rather than defended.

Lane `wb-reader` / `wt-wb-reader`, branched at master `c34c388c`.
PREREG: [`WB_READER_PREREG.md`](WB_READER_PREREG.md) (committed `7d671a8`, before
the 48 were grouped) and [`WB_READER_PREREG_R2.md`](WB_READER_PREREG_R2.md)
(committed before round 2 ran). Scored in §7. Board rows **#1590–#1596**,
**#1599**, **#1600** — **#1597, #1598 and #1601–#1609 are left explicitly
unminted**.

---

## §0 Result, up front

**The premise of the lane is half-wrong, and the half that is wrong is the
important half.**

The 48 frontier functions behind `fnbyte-refused-parse` are **not blocked on
grammar**. The port's own width scanner (`shapes::control_flow::scan_full`) walks
**48 of 48** of those bodies from the `4C 4F 11` marker to the seven-byte
function tail with the depth invariant intact — every one reports a
`cflow-<shape>` key and **not one** reports a `cf-<production>-0xNN`. There is no
token in any of the 48 whose *width* the port does not already know.

So the thing the campaign wanted decoded — "what the IL constructs behind them
*mean*, per c2's own IL reader" — is decodable, and **is decoded below**, and it
recovers **zero** functions on its own. What blocks the 48 is the *modeled
expression class*: acceptance, i.e. codegen semantics. That is measured, not
argued, in §4: admitting the entire relational family moves the frontier's reader
column from **48 to 48**, and admitting the entire intra-body control-flow
vocabulary moves it from **48 to 48**.

What the reading **is** worth is separate and is not small: c2's `.ex` operand
grammar turns out to live in **one 190-entry byte table** (`DAT_10b25e48`), which
gives the operand width of *every* opcode at once — including **nine positions
where the port's published width table is silent, guessed, or wrong**, three of
which are latent desyncs waiting in the corpus (§3.4). Three of those readings
survived an obj-check against real `c2.dll` (§5), including one that **refutes
the port's `TYPE = <tag><kind><LEB id>` grammar outright**.

---

## §1 The listing — all 48, by first-blocker key

Derived from board #1474's instrument (`GapReport::frontier_codegen`,
`crates/c2-harness/src/gap/factors.rs:813`) plus `TuResult::fn_blockers`
restricted to the frontier, via a scratch-only print in this lane's worktree
(**never committed**, reverted before the gate in §8). Reproduces the master
baseline exactly: FRONTIER 16 TUs, 59 emitted functions = 10 exact + 1 wrong +
0 codegen-refused + **48 reader-refused** + 0 ungraded.

| key | n | TUs | example function |
|---|---:|---:|---|
| `expr-cmp-eq` | **11** | 6 | `?Add_InPlace@IPP@@YAXIPBMPAM@Z` |
| `expr-jump` | **10** | 3 | `?NextHashPrime@@YAHH@Z` |
| `assign-store-type-8643` | **4** | 2 | `?FindNodeA@@YAPBUCharGraphNode@@W4PlayBlend@@PAXM@Z` |
| `expr-op-0x27` | **4** | 2 | `?SetNonce@XTEABlockEncrypter@@QAAXPB_KI@Z` |
| `expr-brfalse` | **3** | 3 | `?GetBuffer@JsonWriter@@QAAJPAGPAK@Z` |
| `assign-rhs-call-0x26` | 1 | 1 | `CXLrcImpl_CreateClientWithTransport` |
| `call-arg-lit-permuted:mid` | 1 | 1 | `vsprintf_s` |
| `call-arg-outer-formal:eof` | 1 | 1 | `?getKeyImpl@@YAXPAEPAD0@Z` |
| `expr-brtrue` | 1 | 1 | `?Free@Pool@@QAAXPAX@Z` |
| `expr-call-in-expr-data-addr-then-plain-call-and-op-more` | 1 | 1 | `?getKey@KeyChain@@YAXIPAE0@Z` |
| `expr-call-in-expr-op-0x1F` | 1 | 1 | `?getMasher@KeyChain@@YAXPAE@Z` |
| `expr-call-in-expr-recv-load-then-plumbing-0x3A` | 1 | 1 | `??0Biquad@DSP@@QAA@PAM@Z` |
| `expr-cmp-ge` | 1 | 1 | `_free_osfhnd` |
| `expr-cmp-ne` | 1 | 1 | `?append@DName@@QAAXPAVDNameNode@@@Z` |
| `expr-intrinsic-memcpy` | 1 | 1 | `?SetKey@XTEABlockEncrypter@@QAAXPBE@Z` |
| `expr-lit-type-9641` | 1 | 1 | `?opaquePredicate@@YAXXZ` |
| `expr-load-type-8211` | 1 | 1 | `?asciiDigitToHex@@YAED@Z` |
| `expr-load-type-8882` | 1 | 1 | `?Encipher@XTEABlockEncrypter@@AAA_K_KPAI@Z` |
| `expr-op-0x0F` | 1 | 1 | `?roll@@YAHH@Z` |
| `expr-op-0x30` | 1 | 1 | `?swap@@YAXAAD0@Z` |
| `param-width-undetermined:mid` | 1 | 1 | `main` |
| **total** | **48** | 16 | |

Which TUs each key touches (file names only; the full 48-row table is
reproducible from the instrument):

* `expr-cmp-eq` — `wordwrap`, `Biquad`, `IPP_basicmath_xbox`, `vsnprnc`,
  `vswprnc`, `mmio`
* `expr-jump` — `keygen_xbox` (8), `Primes`, `wordwrap`
* `assign-store-type-8643` — `keygen_xbox` (2), `negate_test` (2)
* `expr-op-0x27` — `EncryptXTEA` (2), `Pool` (2)
* `expr-brfalse` — `keygen_xbox`, `wordwrap`, `jsonwriter`

`src/keygen_xbox.cpp` alone contributes **18** of the 48 across **10 distinct
keys**, of which `expr-jump` is 8.

### §1.1 The finding that makes the rest of this document what it is

The same instrument also carries each body's **control-flow decode**
(`FnCensus::cflow`, `FnCensus::cflow_off`). Crossed against the blocker key:

| blocker key | cflow class | `cflow_off` (why it left `CfResidue::Modeled`) | n |
|---|---|---|---:|
| `expr-jump` | `cflow-loop` | `rmw` | 9 |
| `expr-cmp-eq` | `cflow-loop` | `compare` | 5 |
| `expr-cmp-eq` | `cflow-if-n` | `compare` | 4 |
| `expr-op-0x27` | `cflow-loop` | `off-add` | 2 |
| `assign-store-type-8643` | `cflow-loop` / `cflow-if-n` | `store-type` | 2 / 2 |
| … 21 further rows, one to three each | | | |

**Every one of the 48 carries a `cflow-<shape>` key. None carries a
`cf-<production>-0xNN`.** `control_flow::walk` returns `Ok` only when the walk
"consumed every byte of the body through a decoded field and landed exactly on
the function tail with the depth invariant intact", so this is a statement about
tokenisation and not about acceptance: **the port's width grammar already covers
100 % of the 48.** **33 of the 48** additionally sit on a CFG class the
emitter cannot express (`cflow-loop` 21, `cflow-if-n` 11, `cflow-if-2` 1); the
other 15 are `cflow-straight` 8, `cflow-straight+expr-modeled` 3, `cflow-if-1` 3
and `cflow-if-1+expr-modeled` 1, held by their `cflow_off` reason alone.

That is why §4's counterfactual is the deliverable that actually prices these 48,
and why the grammar work below is filed as *correction of the port's width table*
rather than as *the frontier's price*.

---

## §2 Where c2's `.ex` reader is

| VA | what | confidence |
|---|---|---|
| `0x10bbc9ab` | **the `.ex` token fetch.** `GetByte`; `0xFF` → `FUN_10c1eeb6(1)` (C1001); `0x4D` → end of stream (sets `DAT_10c2f068`); otherwise zeroes `[node+4]` and `[node+6]` (both `u16`) and calls the operand decoder. One call per token. | high |
| `0x10b3d610` | **the operand decoder.** `*node = opcode` (`0x10b3d624`), then dispatch. | high |
| **`0x10b3d626`** | **`movzx eax,BYTE PTR [ecx+0x10b25e48]`** — the **operand-class table**, indexed by the opcode byte. *This is the whole `.ex` operand grammar in one array.* | high |
| `0x10b3d631` | `cmp eax,0x1c` / `ja 0x10b3d941` — **29 classes**, `0x00`–`0x1C`; anything else is C1001. | high |
| `0x10b3d63a` | `jmp DWORD PTR [eax*4+0x10b3d954]` — the 29-entry class jump table at **`0x10b3d954`**. | high |
| `0x10b3d941` | the out-of-range / class-`0x10` / class-`0x16` arm: `FUN_10c1eeb6(1, 0x10b02140, 0x1eb)` — C1001, **line 491**. | high |
| `0x10b3d546` | **the TYPE reader** (`FUN_10b3d546`), called from ten class arms. | high |
| `0x10c1fe40` | **the TYPE *word*** — a 1/2/3-byte variable-length integer. Not previously named in `docs/whitebox/`. | high |
| `0x10b3d56e` | `test WORD PTR [ecx*2+0x10b25f10],dx` — a **per-opcode `u16` attribute table** at `0x10b25f10`, bit `0x400`, gated on type-word bit `0x1000`. | medium (read; its meaning is not established) |

The scalar primitives are the ones `DISCLOSURE.md` already names as navigation
(`0x10c1f8fc` GetByte, `0x10c1f91b` varU, `0x10c1f9a6` i16c, `0x10c1f9e9` i32c,
`0x10c1fae7` i64c, `0x10c1f90a` skip-continuation, `0x10c1fc5b` bounded string).
Read as bodies rather than as labels, two of them matter below:

* **`0x10c1f91b` (`varU`)** — reads **two** bytes LE unconditionally; if the
  second has bit 7 set it clears bit 15 and reads **two more**, folding them in
  at `<<15`. So a `varU` is 2 or 4 bytes and **never 1**.
* **`0x10c1f90a` (skip)** — `do { b = GetByte } while (b < 0)`: a plain
  LEB-style continuation run, 1..n bytes.

---

## §3 The operand grammar — the 29 classes

Decoded from `0x10b3d641`–`0x10b3d94d`. `→[k]` is a store at `node+k`.
`sym(id)` is `FUN_10b99977(TU[+0x14], id)`, the TU symbol-table lookup.

| class | arm VA | operand grammar |
|---:|---|---|
| `00` | `0x10b3d92e` | **nothing.** The opcode is the whole token. |
| `01` | `0x10b3d641` | `TYPE` |
| `02` | `0x10b3d64d` | `if (DAT_10c67fc0 == 0 && op == 0x42) nothing; else` → class `08` |
| `03` | `0x10b3d676` | `varU`→`sym`→`[0x20]`; `TYPE`; `GetByte`; `GetByte` |
| `04` | `0x10b3d69e` | `varU`→`sym`→`[0x20]`; `GetByte` |
| `05` | `0x10b3d6b2` | `TYPE`; **`GetByte`** (one raw byte — *not* a varint) |
| `06` | `0x10b3d6bb` | `TYPE`; then payload **by decoded type**: class `5` (real) → the 12-byte decimal path `FUN_10c1feef`/`FUN_10be6caf`; else if the type's low 12 bits are `8` → `i64c`; else → `i32c` |
| `07` | `0x10b3d7a8` | `TYPE`; then class `08` |
| `08` | `0x10b3d65f` | `varU`→`sym`→`[0x20]` |
| `09` | `0x10b3d7b4` | `TYPE`; `GetByte`→`[0x24]` (byte) |
| `0A` | `0x10b3d7bb` | `GetByte`→`[0x24]` (byte) |
| `0B` | `0x10b3d7c8` | **unconditional C1001** — `FUN_10b33526(0x10b02140, 0x12b)`, line 299 |
| `0C` | `0x10b3d7d7` | `i16c`→`[0x24]` (u16); then `FUN_10b9761e(node)` — the `0x4F` sub-record reader, descriptor table `0x10b26268` |
| `0D` | `0x10b3d922` | `i32c` → `[0x10]`, sign-extended into `[0x14]` |
| `0E` | `0x10b3d7ec` | `TYPE`; `varU`→`sym`→`[0x20]`; `varU`→`sym`→`[0x24]` |
| `0F` | `0x10b3d81c` | `i16c`, discarded |
| `10` | `0x10b3d941` | **C1001** |
| `11` | `0x10b3d922` | = class `0D` |
| `12` | `0x10b3d826` | `TYPE`; **raw `varU`** (not symbol-mapped) → `[0x24]` |
| `13` | `0x10b3d834` | `TYPE`; `i32c`→`[0x24]` |
| `14` | `0x10b3d842` | `i32c`→`[0x10]`; `i32c`→`[0x24]` |
| `15` | `0x10b3d84c` | `varU`→`sym`→`[0x10]`; `varU`→`sym`→`[0x20]`; `i16c` sign-extended →`[0x24]` |
| `16` | `0x10b3d941` | **C1001** |
| `17` | `0x10b3d878` | `i32c` = *n*; allocate *n*+1; read *n* bytes as a bounded NUL-terminated string; NUL-terminate; ptr→`[0x10]` |
| `18` | `0x10b3d8a1` | `varU`→`sym`→`[0x20]`; `TYPE` |
| `19` | `0x10b3d8b8` | `TYPE`; `GetByte`→`[0x24]`; then `DAT_10c2edc4 ? i32c : i16c`(sign-extended) → `[0x20]` |
| `1A` | `0x10b3d8e5` | `i32c` = *n*; then *n* × skip-continuation |
| `1B` | `0x10b3d8fa` | `i32c`→`[0x10]`/`[0x14]`; then `varU`, discarded |
| `1C` | `0x10b3d90d` | `TYPE`; then **if `[DAT_10c472e8+0xcac] != 0`**: `i32c`→`[0x10]`/`[0x14]` |

### §3.1 The class table itself (`0x10b25e48`, opcodes `0x00`–`0xBF`)

```
class 00 (109) 00-0E 1A-1E 1F-25 2E 37 43 44 46 47 48 4A 4B 4C 4D 4E 53 56 57 59
               5F 60 62 68 6A-76 79-7C 80 84 85 86 88-98 9D A3-B7 BC BE BF
class 01 ( 32) 0F-19 27 2F 30 31 32 35 36 40 41 55 58 5A 64 77 7D 87 8D 9A 9C A0 A1
class 02 ( 15) 26 28 29 2D 38 39 3A 3B 42 49 63 65 69 B8 BA
class 03 (2A)   class 04 (2B)   class 05 (2C 34)   class 06 (33)
class 07 (3C BB) class 08 (3D)  class 09 (3E 7E)   class 0A (3F)
class 0B (45)   class 0C (4F)   class 0D (50 51 52 54 5B 9E 9F)
class 0E (78)   class 0F (7F)   class 10 (81 82 83) class 12 (9B)
class 13 (5C)   class 14 (5D 5E) class 15 (61)     class 17 (A2)
class 18 (B9)   class 19 (BD)   class 1A (66)      class 1B (67)  class 1C (99)
```

### §3.2 The TYPE word (`FUN_10c1fe40`, `0x10c1fe40`) — the port's grammar is wrong

```
b1 = GetByte
if (b1 & 0x80) == 0:  return b1                                  /* 1 byte  */
b2 = GetByte
if (b1 & 0x40):
    b3 = GetByte
    return ((b2 & 0x7F) << 16) | ((b1 & 0x7F) << 8) | b3          /* 3 bytes */
return ((b1 & 0x7F) << 8) | b2                                    /* 2 bytes */
```

`FUN_10b3d546` then, in order (`0x10b3d550` … `0x10b3d60b`):

1. `class = v & 0xF` (`0x10b3d557`) — 1 signed · 2 unsigned · 3 data pointer ·
   4 code pointer · 5 real · 6 aggregate · 7 void. **Exactly the low nibble of
   the byte the port calls the `kind`.**
2. `if (v & 0x1000 && attr[opcode] & 0x400) node[+6] |= 0x800` (`0x10b3d55f`–`0x10b3d57d`).
3. **`if (opcode == 0x27) node[+6] |= 0x4000`** (`0x10b3d581`, `0x10b3d586`, `0x10b3d58b`).
4. `ext = (v >> 4) & 0x1F`; **if `class == 6 && ext == 0` read an `i32c`** — the
   aggregate's out-of-line size (`0x10b3d58f`–`0x10b3d5a4`).
5. **`if ([DAT_10c472e8 + 0xcac] != 0) skip-continuation`** (`0x10b3d5a6`–`0x10b3d5b4`).
   *This* is the field the port models as the TYPE's `<LEB128 id>`. It is a
   **separate, globally gated read**, not part of the type word.
6. **`if (opcode == 0x27) return`** (`0x10b3d5b9`) — the whole classification tail
   below is **skipped for opcode `0x27` and for no other opcode**.
7. `size_index = (v >> 9) & 7` (`0x10b3d5c1`), stored at `node[+0x28]`
   (`0x10b3d5ea`) — 1 → 1 byte, 2 → 2, 3 → 4, 4 → 8. **This is what the port's
   `tag` byte carries**: `0x82`/`0x84`/`0x86`/`0x88` ↔ `(tag − 0x80)/2`.
8. `FUN_10b3d40a(class, ext, size_index − 1, FUN_10c1fe9d(v))` → `node[+4]`;
   `if (v & 0x10000) node[+6] |= 0x2000`; if the result's top nibble is 6 or 7,
   `node[+8] = ext`.

So the port's `TYPE = <tag> <kind> <LEB128 id>` is a **re-spelling of the
two-byte case only**: `tag = 0x80 | (v >> 8)`, `kind = v & 0xFF`. The wide bit
the port calls `TAG_WIDE` (`0x40`, `docs/IL_TYPE_WIDE_TAG.md`) is **`b1 & 0x40`,
the type word's own three-byte continuation** — the port's reading of it is
independently confirmed here. What the port does **not** have is:

* the **one-byte short form** (`b1 < 0x80`), which it cannot parse at all; and
* the fact that the "id" is **not part of the type** and is read only when a
  global is set.

§5 puts both of these in front of the oracle. The one-byte form **wins**.

### §3.3 The three largest families, decoded

**`expr-cmp-eq` (11) and the relational family `expr-cmp-ne`/`-ge` (2 more).**
Opcodes `0x1F`–`0x24` (`==`, `!=`, `<=`, `<`, `>=`, `>`, from
`body::expr_opcode_name`, which the project pinned by compiling a probe per
relation). All six are **class `0x00`: zero operand bytes.** The token *is* the
opcode. Confirmed by inspection of a real stream — `?Add_InPlace`'s body reads
`B9 <n> <86 42 75>  33 <86 42 75> 00  1F  38 <fa 09>`, i.e. `if (n == 0) goto L`
with nothing between the compare and the branch.
**Record layout:** `node[0] = 0x1F`; `node[+4]`, `node[+6]` zeroed by the fetch
at `0x10bbc9ab`; nothing else written.
*Grammar cost to the port: none — its width table already has these six at
width 1.*

**`expr-jump` (10), and `expr-brfalse`/`expr-brtrue` (4 more).** Opcodes `0x3A`,
`0x38`, `0x39` — all **class `0x02`**, which is class `0x08` plus a guard that
fires only for opcode `0x42`. So each is `<op> <varU>`, and the `varU` is
resolved **through the TU symbol table** (`FUN_10b99977(TU[+0x14], id)`,
`0x10b3d669`) into `node[+0x20]`. A branch target is a **symbol token**, not an
offset and not a table index — which is why the port's `Site { tok, at }` model,
which decides forward/backward by *where the `29` happens to sit*, is the right
shape.
**Operand encoding:** `varU` = 2 bytes LE, and if bit 15 is set, clear it and
read 2 more, `v = lo15 | hi16 << 15` — so **2 or 4 bytes, never 1**.
*Grammar cost to the port: none.*

**`expr-op-0x27` (4).** Opcode `0x27` is **class `0x01`: exactly one TYPE and
nothing else.** And `0x27` is the **only opcode the TYPE reader special-cases**:
at `0x10b3d581` it sets `node[+6] |= 0x4000`, and at `0x10b3d5b9` it **returns
before the classification tail**, so a `0x27` operand's type word is *read for
its width and never decoded* — no class, no size index, no `node[+4]`.
Semantically that fits the construct the port calls `off-add`: a designator built
on a designator, where the type names the *pointee being stepped over* and the
node needs a flag, not a lowered type.
*Grammar cost to the port: none — it already reads `27 <TYPE>`. But see §5: the
TYPE **width** rule it uses to do so is wrong in general.*

**`assign-store-type-8643` (4) and the `*-type-*` family (3 more).** These are
not opcode refusals at all: the opcode is `0x32` (store, class `0x01`) or `0xB9`
(load, class `0x18`) or `0x33` (literal, class `0x06`), all of which the port
reads correctly, and the key names the **type class it will not accept** —
`8643` is a 4-byte **data pointer** (`kind & 0xF == 3`), `8882` an 8-byte
unsigned, `8211` a 1-byte signed, `9641` a volatile 4-byte signed. c2's reader
accepts all of them without comment; the refusal is `is_int4_type ||
is_ptr_to_4`, a port-side acceptance gate.

### §3.4 Where the port's width table and c2's table disagree

Nine positions. Three are latent desyncs, four are refusals the table can lift,
two are structural.

| opcode | port (`shapes::control_flow::operand`) | c2 | consequence |
|---|---|---|---|
| **`0x2C`** convert | `2C <TYPE> <varint>` | **class `05` = `TYPE` + one raw `GetByte`** (`0x10b3d694`) | **latent desync** at any payload ≥ `0x80` |
| **`0x54`** scope close | `+2`; §12.1 records byte-vs-varint as **UNKNOWN** | **class `0D` = `i32c`** (`0x10b3d922`) | **§12.1 is resolved**; agree below `0x80` |
| **`0x28`** subscript | hard-coded `28 00 00`, refuses anything else | **class `02` = `varU` symbol token** | `00 00` *is* a `varU` of 0; a token with bit 15 set is 4 bytes and the port desyncs |
| `0x14` | refuses — "unwitnessed, and §5 says not to fill it" | **class `01` = `TYPE`** | a refusal the table lifts |
| `0x31` | refuses — "`IL_CALL_GRAMMAR.md` §7 lists it as unidentified" | **class `01` = `TYPE`** | ditto |
| `0x07 0x08 0x1D 0x1E 0x25` | refuse — unwitnessed | **class `00` = payload-free** | ditto (the width the port declined to guess is the width) |
| `0x43` escape | `43 <sub>`; `42` → `+4`, `37` → `+2`; every other sub refuses | **`43` is class `00` — there is no escape.** `42` is class `02` with a `DAT_10c67fc0`-gated zero-operand arm; `37` is class `00` | the port's two witnessed cases fall out exactly (`43`+`42`+2-byte `varU` = 4; `43`+`37` = 2) and its refusal of other sub-opcodes is unnecessary |
| `0x45`, `0x81`–`0x83` | not mentioned | **unconditional C1001** (class `0B` at `0x10b3d7c8`, line 299; class `10` → `0x10b3d941`, line 491) | these bytes cannot occur in a valid `.ex` |
| TYPE width | `<tag> <kind> <LEB128 id>`, always ≥ 3 | **1/2/3-byte word + a globally gated LEB skip** (§3.2) | **refuted by obj-check**, §5 cell R2-C3 |

Agreements worth recording, because they are independent confirmations of
readings the port reached from captures alone: `0x44` payload-free · `0x66
<n> <n LEB tokens>` (class `1A`) · `0x67 <i32c slot> <varU tok>` (class `1B`, and
c2's first field is an `i32c`, which is exactly the `67 80 80 00 00 00 04 0A`
escape `work/WDR/probe/p3.cpp` measured) · `0x9A <TYPE>` with no trailing field
(class `01` — c2 agrees with the 13,024-body corpus measurement and **not** with
the analogy to `0x99`) · `0x99 <TYPE> <i32c>` where the trailing field is
*conditional* (class `1C`) · `0x5C <TYPE> <i32c>` · `0x5D`/`0x5E <i32c> <i32c>` ·
`0xB9 <varU> <TYPE>` · `0x33 <TYPE> <payload-by-class>`.

---

## §4 Recovered vs renamed — measured, not predicted

The project already ships the counterfactual instruments for two of the top three
families: `C2RS_SINK_REL=expr` consumes all six relationals and
`C2RS_SINK_BRANCH=cflow` consumes `38`/`39`/`29`/`3A`/`4B`. Both are
**measurement-only by construction** — they push no `IlOp` and poison the walk,
so they cannot move an obj byte. Run over the 16 frontier TUs at the workload
flags:

| arm | frontier | `frontier-codegen-reader` | the 48's top keys |
|---|---:|---:|---|
| baseline | 16 | **48** | `expr-cmp-eq` 11 · `expr-jump` 10 · `assign-store-type-8643` 4 · `expr-op-0x27` 4 · `expr-brfalse` 3 |
| `C2RS_SINK_REL=expr` | 16 | **48** | `expr-brfalse` **14** · `expr-jump` 10 · `assign-store-type-8643` 4 · `expr-op-0x27` 4 · `expr-brtrue` 3 |
| `C2RS_SINK_BRANCH=cflow` | 16 | **48** | `expr-cmp-eq` 11 · `expr-call-in-expr-op-0x35` **9** · `assign-store-type-8643` 4 · `expr-op-0x27` 4 · `expr-op-0x53` 3 |

**Recovered 0. Renamed 21** — the 11 `expr-cmp-eq` merge into a 14-strong
`expr-brfalse` row, and the 10 `expr-jump` become 9 `expr-call-in-expr-op-0x35`
plus 3 `expr-op-0x53`. This is board #150's eighth and ninth confirmation, taken
on the frontier itself rather than on the whole census.

**An instrument defect found on the way, reported rather than worked around:**
setting *both* sinks returns the key histogram to baseline exactly (`expr-cmp-eq`
11, `expr-jump` 10). The two are not composable, and `c2rs census` reproduces it
on one TU (`REL` alone → `expr-brfalse` ×5; `REL`+`BRANCH` → `expr-cmp-eq` ×5).
Nothing in this lane depends on the composition — the two single-sink arms above
are each valid on their own — but a lane that reads the `both` arm as "admitting
everything recovers nothing" would be reading an instrument that did not run.
Board **#1600**.

### §4.1 What the port's reader would need, per family

| family | n | what the *reader* needs | what actually blocks it |
|---|---:|---|---|
| `expr-cmp-eq` + `-ne` + `-ge` | 13 | **nothing.** Width 1, already correct. | a value model for a relational, and then a lowering: `cmplw`/`cmplwi` + `crXX` + a materialisation. `cflow_off = compare` on all 13. |
| `expr-jump` + `-brfalse` + `-brtrue` | 14 | **nothing.** `<op> <varU>` symbol token, already correct. | a CFG class: 9 of the 14 are `cflow-loop` with `cflow_off = rmw`, i.e. they need a *compound-assignment* value model **and** a backward branch, and `codegen::labels` invariant 4 refuses every backward branch on the label-counter argument (`docs/LABEL_COUNTER.md` §4.2). |
| `expr-op-0x27` | 4 | **nothing for the opcode**; the TYPE **width rule** is wrong in general (§3.2/§5) and `27`'s type is never classified, so the port may skip it entirely rather than gate on its class | an `off-add` designator lowering; 2 of the 4 are additionally `cflow-loop`. |
| `assign-store-type-*` / `expr-load-type-*` / `expr-lit-type-*` | 7 | **nothing.** These are acceptance gates (`is_int4_type \|\| is_ptr_to_4`), not decodes. | a lowering for a 4-byte pointer store, an 8-byte unsigned load, a 1-byte signed load, a volatile int literal. |
| everything else | 10 | mixed; `call-arg-*`, `param-width-*`, `expr-call-in-expr-*` are `mcall` productions, one function each | one function each — no family. |

**Predicted recovered / renamed on the 48 if every reading in §3 were adopted
into the port's reader tomorrow: 0 recovered, 48 renamed.** Two of the three
top families' shares of that (21 of 48) are **measured** above rather than
predicted; the rest follows from §1.1 — a body whose every token already
tokenises cannot be recovered by a width fix.

---

## §5 The obj-checks

Real `c2.dll` under wibo, via `c2host`, on real captured bundles at the workload
flags. Method: capture the TU's `_CL_*` bundle with `/Bd` (keeping it alive with
`strace -e inject=unlink…:retval=0`), edit **one or two bytes** of the `.ex` at a
site located by the census's own blocking-hex window, replay, compare.
Harness: `work/wb-reader/probe.py` (scratch, not committed).

**The baseline gate.** Every TU's replayed baseline reproduced its pipeline obj
**byte-identically** with `TimeDateStamp` zeroed, on all five TUs — after one
harness correction that is worth recording: c2 writes its **`-Fo` path string
into the obj**, so a replay to a different filename differs in exactly those
bytes and nothing else. Round 0 read that as a `DROP` on all five TUs (decline
clause 1 firing correctly on a harness defect, not on c2).

### §5.1 Round 1 — the registered grid, and why four cells missed

| cell | edit | predicted | observed |
|---|---|---|---|
| A0 | `1F` → `1F` | `IDENT` | **`IDENT`** ✓ |
| A1 | `1F` → `20` (same class) | `DIFF` | **`DIFF`** ✓ |
| A1b | `1F` → `20` at a second site | `DIFF` | **`DIFF`** ✓ |
| A2 / A2b | `1F` → `27` (cross class) | `NOOBJ` | `DIFF` ✗ |
| A3 | `1F` → `26` (cross class) | `NOOBJ` | `DIFF` ✗ |
| B0 / B0b | jump token rewritten over itself | `IDENT` | **`IDENT`** ✓✓ |
| B1 | jump token → another label of the same body | `DIFF` | **`DIFF`** ✓ |
| B2 / B2b | jump token bytes swapped | `NOOBJ` | `DIFF` ✗ |
| C0 / C0b | `27`'s TYPE rewritten over itself | `IDENT` | **`IDENT`** ✓✓ |
| C1 / C1b | `27`'s TYPE class nibble `43` → `41` | `IDENT` | **`IDENT`** ✓✓ |
| C2 / C2b | `27`'s TYPE tag `A6` → `C6` | `NOOBJ` | `IDENT` ✗ |

**Four misses, all optimistic, all the same mistake: `NOOBJ` is not c2's failure
mode.** A desynchronised `.ex` operand stream does not raise C1001 — c2 decodes
whatever the shifted bytes say and emits an obj. (`C2` misses for a second,
separate reason my own §3.2 had already corrected before the run: `A6 43 8b 20` →
`C6 43 8b 20` moves the third byte from the LEB skip into the type word and is
**width-preserving**, so `IDENT` was the right answer and the registered
prediction was simply stale.)

`{IDENT, DIFF, NOOBJ}` therefore cannot separate a same-class substitution from a
cross-class one. Round 1 is scored as it stands.

### §5.2 Round 2 — the leader/body delta

Registered in [`WB_READER_PREREG_R2.md`](WB_READER_PREREG_R2.md) before it ran.
Outcome is `Δleaders` (symmetric difference of `.text` COMDAT leader names) and
`Δbodies` (leaders in both whose section bytes differ); `STRUCTURE-BROKEN` is the
third value that emerged — `c2-obj`'s COMDAT walk, ported verbatim, **fails
closed** on the mutant obj.

| cell | edit | predicted | observed | verdict |
|---|---|---|---|---|
| R2-A0 | `1F` → `1F` | `Δl=0 Δb=0` | `Δl=0 Δb=0` | ✓ |
| R2-A1 | `1F` → `20` (class 0 → 0) | `Δl=0 Δb=1` | **`Δl=0 Δb=1`** | ✓ |
| R2-A1b | `1F` → `23` (class 0 → 0) | `Δl=0 Δb=1` | **`Δl=0 Δb=1`** | ✓ — *metric validated; the decline clause did not fire* |
| R2-A2 | `1F` → `27` (class 0 → **1**) | `Δl>0 ∨ Δb>1` | `Δl=0 Δb=1` | ✗ — **not discriminating**, see below |
| R2-A3 | `1F` → `26` (class 0 → **2**) | `Δl>0 ∨ Δb>1` | **`STRUCTURE-BROKEN`** | ✓ |
| R2-B1 | jump token → another label | `Δl=0 Δb=1` | **`Δl=0 Δb=1`** | ✓ |
| R2-B2 | jump token bytes swapped | `Δl>0 ∨ Δb>1` | **`STRUCTURE-BROKEN`** | ✓ |
| R2-C1 / C1b | `27`'s TYPE class nibble `43` → `41` | `Δl=0 Δb=0` | **`Δl=0 Δb=0`** | ✓✓ |
| R2-C3 / C3b | `27`'s TYPE tag `A6` → `26` | `Δl>0 ∨ Δb>1` | **`STRUCTURE-BROKEN`** | ✓✓ |

Plus two **controls** whose prediction is the same under this reading and under
the port's, so they cannot be cherry-picked, run after the fact and labelled:

| control | edit | observed |
|---|---|---|
| R2-C4 / C4b | `27`'s TYPE `A6 43` → `26 C3` — a **one-byte** type word plus a **three-byte** LEB skip, restoring the original four bytes | **`IDENT` / `IDENT`** |
| R2-A4 | `1F` → `26` with the following `varU`'s continuation cleared | `STRUCTURE-BROKEN` — my compensating arithmetic was off by one byte (`26` then consumes 3 where 4 were needed). Reported as a failed control, not as evidence. |

### §5.3 What survived, and what it establishes

1. **The class table is real, at one site, one byte, three ways.** At
   `?Add_InPlace`'s `1F`, substituting a **class-`00`** opcode (`20`, `23`)
   changes exactly one function body and leaves the obj structurally intact;
   substituting the **class-`02`** opcode `26` — which the table says consumes a
   `varU` — breaks the obj's COMDAT structure outright. Same site, same
   single-byte edit, opposite outcomes, in the direction `DAT_10b25e48` predicts.
   **Confirmed.** This is the success floor for the largest frontier family:
   `expr-cmp-eq` is opcode `0x1F`, class `00`, **zero operand bytes**, and the
   obj agrees.
2. **`varU` is little-endian with a bit-15 continuation.** Swapping the two bytes
   of a jump token (`EC 09` → `09 EC`) sets bit 15 and makes the reader eat two
   more bytes: the obj's structure breaks. A **big-endian** rival predicts
   `IDENT` (it would name the same label) and a **plain-2-byte-LE** rival
   predicts a merely-different obj. Both are refuted. **Confirmed.**
3. **The TYPE word has a one-byte short form, and the port's
   `<tag><kind><LEB id>` grammar is REFUTED.** `A6 43 8B 20` (2-byte word +
   2-byte skip = 4) → baseline. `26 43 8B 20` → **structure broken**: under the
   port's grammar this is `tag=26 kind=43 id=LEB(8B,20)`, still four bytes, still
   aligned, and it is not. `26 C3 8B 20` (1-byte word + 3-byte skip = 4) →
   **byte-identical**. Three cells, two TUs, and no fixed-width `<tag><kind>`
   model can produce that pattern. **Confirmed, and the port's rule is the one
   that has to move.**
4. **Opcode `0x27`'s TYPE is read for width and never classified.** Changing its
   class nibble from `43` (data pointer) to `41` (signed) yields a
   **byte-identical** obj in two TUs; so does replacing the whole type word with
   a different-valued one of the same width (R2-C4). The rival — "a `27`'s type
   is classified like every other operand's" — predicts a different obj and is
   refuted. **Confirmed.**
5. **Not established, and said so.** R2-A2 (`1F` → `27`) is not discriminating:
   the reading says `27` then consumes `38 FA 09` (a one-byte type word `0x38`
   plus a two-byte skip) and re-aligns on the following `53` — which is what the
   obj shows, but a `0`-width and a `3`-width rival both predict the same
   observable. It is reported as a miss, not converted into support.
6. **c2 does not ICE on a desynchronised operand stream.** Nine cells that
   shifted the stream all produced an obj. This is a fact about the oracle's
   error surface that any future mutation lane needs: **absence of a C1001 is not
   evidence the edit was legal.**

The rest of §3 is **read but unconfirmed**: 25 of the 29 classes have no cell,
and every "NEW" row of §3.4 (`0x14`, `0x31`, `0x07`, `0x08`, `0x1D`, `0x1E`,
`0x25`, `0x2C`'s raw byte, `0x54`'s `i32c`) is navigation until an obj says
otherwise. §5.4 is the design; none of it ran.

### §5.4 Designed and not run

* **`0x2C`'s payload.** Find or synthesise a convert whose payload byte is
  ≥ `0x80` (`docs/IL_CAST_CONVERT.md`'s corpus can be queried for one). c2 reads
  one raw byte; the port reads a varint. Predict: the port's *scanner* desyncs
  where c2 does not, so the discriminator is a **port-side** decode-reach delta,
  no `cl.exe` needed. Cheapest of the lot; it is a query, not an experiment.
* **`0x54`'s `i32c`.** Rewrite a scope-close depth from `2A` to `80 2A 00 00 00`
  — not length-preserving, so it needs a companion deletion; or find a body whose
  natural depth exceeds `0x7F`. Predict `IDENT` under `i32c`, structure-broken
  under a plain byte.
* **`0x14` / `0x31` / `0x07` / `0x08` / `0x1D` / `0x1E` / `0x25`.** Substitute
  each for an opcode of the class the table assigns it, at a site where a length
  change would break, and require `Δleaders = 0 ∧ Δbodies ≤ 1`.
* **The `43` escape.** Substitute the sub-opcode `42` → `37` and check that the
  obj re-aligns two bytes earlier rather than breaking — the port's escape table
  says `37` is a legal sub-opcode with a *different* width, c2's says they are
  two ordinary tokens.

---

## §6 Pre-drafted DISCLOSURE rows

**None of these is adopted.** They are drafts, formatted per
[`DISCLOSURE.md`](DISCLOSURE.md)'s checklist, for a later code lane to carry **in
the same commit as the code change**. Only findings that survived §5 get a row;
`W-EX-1` is deliberately absent because the class table as a *whole* is not
obj-confirmed — only the three cells §5.3 names are.

| # | Kind | What would be adopted | Address in `c2.dll` | Notes |
|---|---|---|---|---|
| **W-EXT-1** | **adoption** | **The `.ex` TYPE word is a 1/2/3-byte variable-length integer, and the "id" is a separate, globally gated LEB skip.** `b1 < 0x80` → the word is `b1` (**one byte** — the form the port cannot parse); `b1 & 0x40` → three bytes, value `((b2 & 0x7F) << 16) \| ((b1 & 0x7F) << 8) \| b3`; else two bytes, `((b1 & 0x7F) << 8) \| b2`. The trailing continuation run the port models as `<LEB128 id>` is read **only** when `[DAT_10c472e8 + 0xcac] != 0`. Type class = `v & 0xF`; **size index = `(v >> 9) & 7`** (1→1, 2→2, 3→4, 4→8), which is what the port's `tag` byte carries. This is a **bit layout**, so it is adoption. | **`0x10c1fe40`** (the word), `0x10b3d550` (its one call site in the type reader), **`0x10b3d5a6`–`0x10b3d5b4`** (the gated skip), **`0x10b3d5c1`** (`shr ebx,9` / `and ebx,7`), `0x10b3d5ea` (the store) | **Obj-confirmed** by §5.3(3): a three-cell width demonstration across two TUs that no `<tag><kind><LEB id>` model can produce. **The grey-zone alternative was tried and is insufficient**: the port reached `<tag><kind><LEB>` from captures alone and it is *correct on every byte the corpus contains*, which is exactly why black-box work cannot find the one-byte form — no workload TU emits one. |
| **W-EXT-2** | **adoption** | **Opcode `0x27`'s TYPE is read for its width and never classified.** The TYPE reader tests the *opcode* twice: once to set `node[+6] \|= 0x4000`, once to return before the classification tail. So a `0x27` operand's type contributes no class, no size index and no `node[+4]`, and the port's acceptance gate on that type is testing a field c2 discards. | **`0x10b3d581`** (`cmp DWORD PTR [esi],0x27`), **`0x10b3d586`**/`0x10b3d58b` (`\| 0x4000`), **`0x10b3d5b9`** (the second test, the early return) | **Obj-confirmed** by §5.3(4): the class nibble `43` → `41` is byte-identical in two TUs, and so is a wholesale replacement of the type word at constant width. |
| **W-EXT-3** | **route** | **A branch/jump/label operand is a `varU` resolved through the TU symbol table** — a symbol token, not an offset, not a table index — and `varU` is little-endian with a bit-15 continuation (2 or 4 bytes, never 1). | `0x10b3d64d` (class-`02` entry and its `0x42` guard), **`0x10b3d65f`** (`call 0x10c1f91b`), **`0x10b3d669`** (`call 0x10b99977` — the symbol lookup), `0x10b3d66e` (the store) | Logged `route:` per the grey-zone rule: the port already models these as tokens and already reads `varU` this way (`readers.rs`, re-derived from black-box IL before any disassembly). §5.3(2) **confirms the encoding against real `c2.dll`** and refutes both rivals. **No value or layout needs to be copied** — the row exists so a reader knows the search was not blind. |

A fourth row is **deliberately not drafted**: the operand class table
`DAT_10b25e48` itself. Copying 190 table entries into the port would be the
largest single adoption this project has contemplated, §4 shows it converts
nothing, and §5 confirms three cells of it and not the table. If a future lane
wants it, it should want the **nine rows of §3.4** individually, each with its own
obj-check and its own row.

---

## §7 PREREG scored

Board #770's streak stood at ~10 optimistic / 2 pessimistic / 1 hit.

### Round-1 PREREG ([`WB_READER_PREREG.md`](WB_READER_PREREG.md))

| # | prediction | outcome |
|---|---|---|
| P1.1 | largest key holds 8–16 of the 48 | **HIT** — 11 (`expr-cmp-eq`) |
| P1.2 | top 3 keys hold ≥ 24 | **HIT** — 11 + 10 + 4 = 25 |
| P1.3 | ≥ 12 distinct keys | **HIT** — 21 |
| P1.4 | keygen contributes 18, not one family | **HIT** — 18 across 10 keys |
| P2.1 | width dispatches off a per-opcode table | **HIT** — `DAT_10b25e48` at `0x10b3d626` |
| P2.2 | ≥ 1 top-3 opcode takes zero operand bytes | **HIT** — `0x1F`, class `00` |
| P2.3 | ≥ 1 port/c2 width disagreement | **HIT, understated** — nine (§3.4) |
| P2.4 | the largest family's reading is obtainable | **HIT** |
| **P3.1** | **the 48 are not grammar-bound; ≥ 40 of 48 already tokenise** | **HIT** — 48 of 48 |
| **P3.2** | recovered 0, renamed ≥ 44 | **HIT** on the measured part (0 recovered on both sink arms; 21 renamed measured, the rest inferred from P3.1) |
| P3.3 | 0 frontier TUs converted by this lane | **HIT** |
| A0 | `IDENT` | HIT |
| A1 | `DIFF` | HIT |
| **A2** | **`NOOBJ`** | **MISS (optimistic)** — `DIFF` |
| **A3** | **`NOOBJ`** | **MISS (optimistic)** — `DIFF` |
| B0 | `IDENT` | HIT |
| B1 | `DIFF` | HIT |
| **B2** | **`NOOBJ`** | **MISS (optimistic)** — `DIFF` |
| C0 | `IDENT` | HIT |
| **C1** | **`IDENT`** (the discriminating cell) | **HIT**, replicated |
| **C2** | **`NOOBJ`** | **MISS (optimistic)** — `IDENT`; stale, see §5.1 |

**18 hits, 4 misses, all four misses optimistic and all four the same mistake.**

### Round-2 PREREG ([`WB_READER_PREREG_R2.md`](WB_READER_PREREG_R2.md))

8 hits (R2-A0, A1, A1b, A3, B1, B2, C1×2, C3×2 — counting the replicates, 10 of
11 cells), **1 miss** (R2-A2, and it is reported as non-discriminating rather
than reinterpreted). The decline clause on the metric's validity did **not**
fire: both same-class cells came back `Δleaders = 0, Δbodies = 1` exactly.

### The direction of the misses

Five misses across both rounds, **all optimistic, and four of them one belief**:
*a wrong-width read of an IL stream will make c2 refuse.* It does not. That
belief is not in any published document, which is why it survived to be
registered; it is now in this one, and in board **#1599**.

---

## §8 Gate

`scripts/gate.sh --require-graded` — this is a docs lane and the gate is a
no-regression control. The scratch instrument of §1 was reverted
(`git checkout -- crates/c2-harness/src/gap/fnbytes.rs`) and the harness rebuilt
**before** the gate ran; nothing under `crates/` is committed by this lane.

See §8 of [`docs/rungs/2026-08-08-wb-reader.md`](../rungs/2026-08-08-wb-reader.md)
for the counts.
