# `P_SECTION` — the section and symbol model: `p2symtab.c` and `emit.cpp`

> **Reference page.** **`[R]`** read from the disassembly, *not* obj-checked —
> a hypothesis. **`[O]`** confirmed against a real obj or `/FAsc` listing, with
> the witness named. **`[I]`** an interpretive step. `[R]` means *"the
> instructions were read correctly"*, never *"this is what c2 does"*.
> Navigation only; nothing here may enter `crates/` without a
> [`DISCLOSURE.md`](../DISCLOSURE.md) row.
> Index: [`ADDR.tsv`](ADDR.tsv) · front door: [`README.md`](README.md)

**Coverage: 24 entries against a denominator of 137** — 102 Ghidra functions in
`p2symtab.c`'s anchor span (`0x10b97dfb`–`0x10b9b8e9`) plus 35 in `emit.cpp`'s
(`0x10be71c9`–`0x10be7e81`). Not covered: the rest of the `.gl` record
dispatcher's 27 arms (see `WB_READER_FINDINGS.md` for the `.ex` side), the
symbol hash table, and `.debug$S`/`$$TYPES` construction beyond the section
creators.

**Why this page is separate from [`P_COFF.md`](P_COFF.md).** The three open
questions the project kept aiming at `coffemit.c` are **not in `coffemit.c`**:
section selection and naming is `p2symtab.c`'s `FUN_10b982d6`, the section
*constructors* are `emit.cpp`'s, and the name/kind/class/override all arrive
**in the IL**. Aiming at the writer would have failed, and did.

---

## 1. Entries

| addr | size | callers | callees | TU | cites | what |
|---|---:|---:|---:|---|---:|---|
| `0x10b982d6` | 385 | 4 | 2 | **`p2symtab.c`** | 5 | **THE section-kind decision**: kind → (name, `Characteristics`). Remaps through `FUN_10be7727` first; **the IL override at `sect+0x53` wins when nonzero** (`LAB_10b98435`) `[R]` · every arm obj-checked by `.gl` mutation `[O]` (§3) |
| `0x10b98457` | 108 | 2 | 0 | **`p2symtab.c`** | 0 | storage-class normaliser. `test [ecx+0x20],0x480` at **`0x10b9849f`** decides `.bss` (6) vs `.data` (2) `[R]` — §4 |
| `0x10b9a143` | 733 | 12 | 10 | **`p2symtab.c`** | 0 | assigns `sym[0xC]` for `.bss`/`.data`/`.rdata` globals; picks COMDAT selection for new COMDATs `[R]` |
| `0x10b9a4a7` | 190 | 1 | 4 | **`p2symtab.c`** | 3 | synthetic-datum builder; corroborates `sym[0x20] = 0x180` / `sym[0x1c] = size` `[R]` |
| `0x10b805b3` | 166 | 2 | 5 | `misc.c` | 0 | **`SetInitialData`**: `sym[0x20] \|= 0x180`, `sym[0x1c] = cb`, then `memcpy` the bytes `[R]`. **No code anywhere in `.text` clears bit `0x80`** — there is no `and […+0x20], ~0x80` `[R]`; §4 |
| `0x10b9b8e9` | 3307 | 3 | 48 | **`p2symtab.c` anchor** | 0 | **the `.gl` record dispatcher.** `GetByte` tag, `dec`+`cmp 0x1a` guard at `0x10b9b92e`, byte-index table `0x10b9c615` (27 entries), jump table `0x10b9c5d5` (16) `[R]` |
| `0x10b9bdcf` | *(in `0x10b9b8e9`)* | — | — | `p2symtab.c` | 11 | the **shared** handler for tags `0x04`/`0x0E`/`0x10` `[R]`. Tag `0x0E` carries the **emit flag word** into `sym+0x4c` at `0x10b9bf78`, verbatim from the IL `[O]` (single-bit `.gl` mutation, `C2_MAP.md` §3E) |
| `0x10b9c212` | *(in `0x10b9b8e9`)* | — | — | `p2symtab.c` | 3 | **tag `0x09` = the SECTION DEFINITION record** (jump-table slot 7): `varU` id, cstr name, `u8` kind, cstr class, `u32` chars-override `[R]` · **every field mutated in real `.gl` bytes and replayed** `[O]` (§3) |
| `0x10b9c5ca` | *(in `0x10b9b8e9`)* | — | — | `p2symtab.c` | — | **not a no-op arm** — `mov edx,0x7ba; jmp 0x10b9bd1a`, the **fatal-error path** `[R]`, confirmed live by a one-byte desync producing `C1001 … p2symtab.c, line 1978` (= `0x7BA`) `[O]`. An earlier "shared no-op arm" reading was wrong |
| `0x10b99093` | 263 | 1 | 8 | **`p2symtab.c`** | 1 | drains the deferred dyninit list **head-first** — the source of the `.bss` reversal `[R]` · reversal rule `[O]` twice, §5 |
| `0x10b99dfe` | 682 | 3 | 6 | **`p2symtab.c`** | 14 | **the symbol-name formatter** — `$T` `$S` `$SG` `$M` `$E` `$L{C,L,N}` `__unwind$` `__catch$` `__annotation$`, all from **one** field pair `sym[0x30]`×`sym[0x31]` `[R]`. See [`P_EH.md`](P_EH.md) |
| `0x10be7473` | 92 | 14 | 4 | **`emit.cpp`** | 0 | `CreateSection(name, group, idx, kind)` → **non-COMDAT**; queries `0x10b982d6`, stores chars at `+0x53` `[R]` |
| `0x10be74cf` | 131 | 5 | 4 | **`emit.cpp`** | 0 | `CreateComdatSection(…)`: `+0x4c = 7`, `+0x63 = selection`, `+0x5f = base` `[R]` |
| `0x10be7727` | 124 | 3 | 2 | **`emit.cpp`** | 0 | returns `sect[0x4d]`; **only kinds `1` and `0x1D` are re-mapped**, by a `$`-aware name prefix (`.rdata`/`.xdata`/`.const` → 4, `.text` → 0, `.drectve` → 0x10) `[R]` |
| `0x10be76d4` | 19 | 4 | 0 | **`emit.cpp`** | 0 | base-section resolver: a COMDAT (`sect[0x4c] == 7`) **with `selection != 5`** → `sect+0x5f`, else identity `[R]`. The `!= 5` is the binary knowing `ASSOCIATIVE` is different |
| `0x10be76e7` | 64 | 1 | 1 | **`emit.cpp`** | 0 | the `$`-aware prefix match: equal, or the next character is `'$'` `[R]` |
| `0x10be77a3` | 53 | 2 | 0 | **`emit.cpp`** | 0 | **the alignment chooser**: kinds `0` / `0x1B` / `0x20` (and `0x12` when `DAT_10c2e310`, the favor-speed bit) → the global `DAT_10c2e9b4`; kind `0xA` → the `.drectve` section's; **else `sect+0x43`** `[R]`. §2 |
| `0x10be7552` | 124 | 2 | 5 | **`emit.cpp`** | 0 | the **only** runtime name composer: strips at `'$'`, appends `"$zz"` / `"$zy"` `[R]` |
| `0x10be75de` | 101 | 2 | 1 | **`emit.cpp`** | 0 | creates `.debug$S` (group `"DEBSYM"`, kind `0x13`) `[R]` |
| `0x10be7643` | 101 | 2 | 1 | **`emit.cpp`** | 0 | creates the DEBTYP section (default name `"$$TYPES"`, kind `0x14`) `[R]` |
| `0x10be794d` | 173 | 3 | 5 | **`emit.cpp`** | 0 | creates `<base>$zz`, kind `0x1B` — **PGO-gated** `[R]` |
| `0x10be79fa` | 175 | 2 | 5 | **`emit.cpp`** | 0 | creates `<base>$zy`, kind `0x20` — **PGO-gated** `[R]` |
| `0x10be7b07` | 68 | 2 | 4 | **`emit.cpp`** | 0 | creates `.text` → `g_state[0x2cc]` `[R]` |
| `0x10be7b9e` | 234 | 1 | 5 | **`emit.cpp`** | 0 | creates `.data` / `.rdata` / `.bss` / `.tls$`, latching `0x10c45f88 … 0x10c45f98` `[R]` |
| `0x10c27b56` | 326 | 7 | 5 | `smdmisc.c` | 5 | stores the allocated address to `sym+0x18` and sets the **first-touch flag `0x800`** — which is why zero-init and dyninit objects never interleave `[R]` |

---

## 2. The alignment nibble — usability question **U1**, answered here

> **The chain, end to end.**
>
> 1. **Per object**: `align(obj) = max(t, 1 if n < 2 else 4 if n < 64 else 8)`
>    where `t` is the declared/natural alignment and `n` the size.
>    `__declspec(align(k))` raises `t` to `k`. **`[O]`** — `OBJ_DATA_BSS_SHAPE.md`
>    §5.4, one object per cell.
> 2. **Per section**: `sect+0x43` accumulates the **maximum** over the objects
>    the section holds. **`[O]`** — Rule B1, `OBJ_DATA_BSS_SHAPE.md` §3.2.
>    Chosen by `0x10be77a3`, which overrides `sect+0x43` with the global
>    `DAT_10c2e9b4` for kinds `0`/`0x1B`/`0x20` (code and the PGO ordering
>    sections) `[R]`.
> 3. **Byte count → nibble**: `0x10b28261`, `(log2(a) + 1) << 20`, a ladder over
>    `1 … 0x2000` — so `1 → 0x100000` (ALIGN_1), `4 → 0x300000` (ALIGN_4),
>    `8 → 0x400000` (ALIGN_8), `0x1000 → 0xD00000` `[R]`.
> 4. **OR it in — conditionally**: `0x10b289fd` ORs the nibble into the emitted
>    `Characteristics` **only when `(sect[0x53] & 0xF00000) == 0`**, i.e. only
>    when the IL's own `Characteristics` override does not already carry one
>    `[R]`. Proven by an override carrying nibble `5`: `0xC0500040` came back
>    **verbatim** `[O]` (`C2_MAP.md` §3F).

### 2.1 The measured table — the thing that cost a lane

Size → nibble, one object per cell, **`[O]`** (`OBJ_DATA_BSS_SHAPE.md` §3.2):

| object | `n` | nibble | `.bss` `Characteristics` |
|---|---:|---|---|
| `char a1` | 1 | **ALIGN_1** | `0xC0100080` |
| `short a2` | 2 | **ALIGN_4** | `0xC0300080` |
| `int a4` | 4 | **ALIGN_4** | `0xC0300080` |
| `char a3[3]` | 3 | ALIGN_4 | |
| `char a63[63]` | 63 | ALIGN_4 | |
| `char a64[64]` | 64 | **ALIGN_8** | `0xC0400080` |
| `double a8` | 8 | **ALIGN_8** | `0xC0400080` |
| `int bz[1024]` | 4096 | ALIGN_8 | |
| `__declspec(align(32)) char` | 1 | ALIGN_32 | `0xC0600080` |

The two thresholds are `n = 1 → 2` and `n = 63 → 64`, and they are a property
of the **object**, not the section `[I]`. The `{1→1, 2→3, 4→3, 8→4}` nibble
sequence a past lane recovered by probing is rows 1, 2, 3 and 7 of this table.

**Do not read the nibble off the type alone.** The promotion term is why:
a plain "natural alignment" model scores **7/18** where the promoted one scores
**14/18** on the same cells (`OBJ_DATA_BSS_SHAPE.md` §5.5) `[O]`.

---

## 3. The IL tag-`0x09` section record — name, kind, class, override

`0x10b9c212`. Node size `0x68`. **Every field was mutated in real `.gl` bytes
and replayed through real `c2.dll`** — which is what makes this a fact rather
than a reading `[O]` (`C2_MAP.md` §3F).

| # | primitive | → field | meaning |
|---|---|---|---|
| 0 | `GetByte` | — | tag `0x09` |
| 1 | `varU` `0x10c1f91b` | `+0x28` | **section index** — what symbols reference |
| 2 | `GetCStr` `0x10c1fc5b` | `+0x04` | **name** (interned) |
| 3 | `GetByte` `0x10c1f8fc` | `+0x4d` | **kind** |
| 4 | `GetCStr` `0x10c1fc5b` | `+0x3b` | class/group (`"CODE"`, `"DATA"`, `""`) |
| 5 | `GetU32` `0x10c1fb8b` | `+0x53` | **`Characteristics` override** |

Then `0x10b982d6` computes `(name, chars)` and writes back to `+0x53`.

The mutation matrix, all `[O]`:

| mutation | emitted `Characteristics` | reads |
|---|---|---|
| baseline (`.CRT$XCU`, kind `0x1D`) | `0xC0300040` | |
| name `.` → `Z` | `ZCRT$XCU`, chars unchanged | the name is IL-borne |
| kind `1D` → `00` | `0x60400020` | `.text`, align forced to 8 |
| kind → `03` / `04` / `13` | `0xC0300080` / `0x40300040` / `0x42300040` | `.bss` / `.rdata` / `.debug$S` |
| override → `0x40000040` | `0x40300040` | **override beats kind** |
| override → `0xC0500040` | `0xC0500040` **verbatim** | c2 skips its align OR when the nibble is set — §2 step 4 |
| class `DATA` → `CODE` | no change | |
| swap the ids of `.CRT$XCU` / `.CRT$XCL` | initializer lands in `.CRT$XCL` | the `varU` **is** the section index |

**Kind `0x1D` means "named data section, keep my name"**: kind `1` resolves
through `0x10be76d4` and substitutes `".data"`; `0x1D` takes `sect+4` `[R]`.
Both yield `0xC0000040`.

**COMDAT-ness is not in tag 9 at all** — `0x10b283b0` spins a COMDAT child off
the tag-9 base via `0x10be74cf`; `.text$yc` has an identically shaped record and
emits `0x60401020` `[O]`.

Source-side corroboration `[O]`: `#pragma section(".mysec", read, write,
discard)` yields kind `01` with `chars = 0xC2000040` — **the u32 is exactly the
`#pragma section` attribute set**; `__declspec(align(64))` on the *symbol* gave
`0xC2700040`, i.e. alignment comes from the symbol, not the record.

> **The `.gl` header is 26 bytes only when all four `i16c` version fields fit in
> one byte.** The build number `11886` escapes (`80 6e 2e`), making it **28** and
> moving the first record from `0x1A` to `0x1C`. This document's predecessor got
> that wrong; the variable-width trap is the standing hazard of every `.gl`
> offset claim.

---

## 4. `.bss` vs `.data` — the predicate

```
10b9849f:  f7 41 20 80 04 00 00    test  DWORD PTR [ecx+0x20],0x480
10b984a6:  74 0c                   je    0x10b984b4          ; -> .bss
```

> **The predicate is "does this symbol carry initializer bytes as it reaches
> c2", not a zero-scan.** `[R]`

Bit `0x80` at `sym+0x20` is set by `0x10b805b3` (`SetInitialData`) and **no code
anywhere in `.text` clears it** `[R]`. So `static int x = 0;` landing in `.bss`
means the zero-folding happened in **c1xx** and c2 never received bytes — high
confidence on the c2-side predicate, **medium** on the c1xx attribution, which
was never verified there `[I]`.

An explicit IL section (`#pragma data_seg`, `__declspec(allocate)`) sets
`sym[0xC]` and **short-circuits all of the above** `[R]`.

---

## 5. `.bss` object order — the reversal rule, and the retracted bump rule

> **`.bss` ascending address = the exact REVERSE of the IL `.gl` record order**
> for objects **with** a dynamic initializer; **= `.gl` order** for plain
> zero-init statics; and in a mixed TU every non-dyninit object precedes every
> dyninit one. `[O]`

Mechanism `[R]`: no initializer → **eager**, allocated as the record streams
past (`0x10b9b161` / `0x10b9b6a4` → `0x10c27b56` → the bump allocator
`0x10c2757d`); has one → **deferred**, head-inserted onto `DAT_10c2f064` and
drained head-first by `0x10b99093`. Head insert + head-first drain = reversal.
The first-touch flag `0x800` in `0x10c27b56` is why the two groups never
interleave.

**Independently confirmed black-box by lane `w-bss` from the IL alone**, across
6 cells, 4 declaration-order permutations and `N = 1…10` in three families —
two independent routes to one mechanism, the strongest evidence class this
project has. It also refuted the hash hypothesis in both directions: a
9-decoration × modulus × shift/mask brute force returned **0 hits**, and
`w-bss`'s own 7 452-configuration search scored 0.08 against a 0.03 baseline,
with the right diagnosis — it was fitting a **c2** hash to a **c1xx** artefact.

> ### ⛔ RETRACTED — the `.bss` bump rule, and it is this directory's calibration datum
>
> `0x10c2757d` reads, cleanly and completely:
>
> ```
> cur = (cur + 7) & ~7;
> if (size - 1 < 7 && (size & (size - 1)) == 0) cur += 8 - size;
> cur += size;
> ```
>
> **That rule does not reproduce the real objs** (`w-bss` §5.5). It was marked
> `high`, it was read correctly, and it was wrong about c2. The claim is
> withdrawn to `unknown`; what replaces it is
> `OBJ_DATA_BSS_SHAPE.md` §5.4's **Rule A3** — one cursor per section,
> promoted per-object alignment, and *first-fit into the lowest hole that fits*
> — which is `[O]` at 14/18 and reproduces both worked examples exactly.
>
> Three live explanations for the failure, none yet distinguished: the read path
> is not the one real inputs take (`0x10c27b56` has **seven** callers); a guard
> was not modelled (`0x800`, `sym[0xC]`); or a later pass re-lays-out. **The
> amendment stands beside the original reading, which stays as written.**

---

## 6. What is NOT known here

* **Section emission order** — open, `medium`. Kind-ordered, not name-ordered;
  the key is `0x1D`-vs-not rather than the kind value `[O]`. Candidate sorters
  `0x10b98b00` / `0x10b9aaa8` / `0x10b9acfa` inspected, none clearly it. Start at
  `0x10b287b8`.
* **Kind `9`** — `0x10b982d6` handles it, **no creator found**. `unknown`.
* **The owner-index `varU`** in the tag-`0x0E` record — Ghidra and raw asm both
  say it is read only when `+0x20 & 0x200`, yet `+0x20` decoded to
  `0x005`/`0x105`/`0x405` in **every** record across three bundles while the two
  bytes are unambiguously consumed. One record in 61 breaks the chain. Open,
  `low`; start at `0x10b9be72`.
* **c1xx's zero-initializer folding** — asserted, **not verified**, and it is a
  c1xx fact.

---

## 7. The `.gl` record dispatcher's ARMS, and the port map over them

**Lane `w-secported`, 2026-08-26, boards `#3661`–`#3666`.** Funded by
[`../../DECISIONS_2026-08-22.md`](../../DECISIONS_2026-08-22.md) § Decision 17.

> **THIS SECTION USES NO EVIDENCE-MARK TOKENS AND THAT IS DELIBERATE.**
> Board `#3641`: `subsys.rs`'s `count_marks` counts every literal bracketed
> `R`/`O`/`I` after a page's first `---`, and it cannot tell an annotation
> from a sentence *about* one — so `w-encmap` moved the encoder's own
> agreement census from 9/28 to 13/34 by writing four sentences that
> mentioned the legend. Everything below says **read-tier** or
> **obj-checked** in words instead, so this section carries **zero** counted
> tokens by construction and the section row's census is unmoved by it.
> Measured across this edit: `section-marks-obj` **17 → 17**,
> `section-marks-total` **53 → 53**.
>
> **Respelling is the mitigation because the general repair is refuted.** Lane
> `w-provaudit` measured, this same wave, that **481 of the 488 marks across
> the ten `ref/P_*.md` pages are already backticked** — so a delimiter rule
> ("backticked means a mention") would zero nine rows' agreement census rather
> than repair it — and that a position rule fails on `P_ENCODE.md` itself (**0**
> marks in table rows, **28** in prose). The only convention that works is a
> **distinct token**, i.e. migrating all 488 marks. Priced and declined by
> `w-provaudit`, and by this lane. **Until it is taken, checking the row's
> census before and after every edit is the only guard.**

### 7.1 The population — and there are not 27 arms

§1's row for `0x10b9b8e9` records a *"byte-index table `0x10b9c615` (27
entries), jump table `0x10b9c5d5` (16)"*, and `labels/W-GLREC.tsv` records the
same. **Neither committed the table contents**, so the arm population had
never been enumerated in this tree, and everything downstream — including
this row's own former residue and Decision 17's brief — described it as *"the
27-arm dispatcher"*.

Re-derived from the pinned image (`work/w-secported/dump_glrec.py`, sha256
`c80981c0…a66258`, decoding every table address and bound from the operand
bytes at the dispatch head `0x10b9b922` so nothing carried can survive):

| claim in this tree | verdict |
|---|---|
| byte-index table `0x10b9c615`, 27 entries | reproduces |
| jump table `0x10b9c5d5`, 16 entries | reproduces |
| fatal arm `0x10b9c5ca` (§1) | reproduces |
| `W-GLREC.tsv`: `byteidx[0x04] == byteidx[0x0E] == byteidx[0x10]` | reproduces (all slot 2) |
| **"27 arms"** | **DOES NOT REPRODUCE** |

**27 is a count of TAG VALUES, not of arms.** `dec eax; cmp eax,0x1a; ja`
bounds the tag at `0x01 … 0x1B` — 27 values — which index a 27-entry byte
table into **16** jump slots. **One of those sixteen slots is the fatal path**
`0x10b9c5ca` (`mov edx,0x7ba; jmp`, the `C1001 … p2symtab.c, line 1978` §1
already records), and it serves **eight** tags. So:

> **15 live record handlers over 19 live tags, plus one refusal over 8.**
> Calling it 27 arms overstates the handler population by **1.8×**.

Committed enumeration: [`../../../work/w-secported/GLREC_ARMS.tsv`](../../../work/w-secported/GLREC_ARMS.tsv).

| arm | extent (B) | tags | tags served | port |
|---|---:|---:|---|---|
| `0x10b9b945` | 1016 | 3 | `0x01` `0x02` `0x1A` | — |
| `0x10b9bd3d` | 146 | 1 | `0x03` | — |
| **`0x10b9bdcf`** | 667 | 3 | `0x04` `0x0E` `0x10` | **`c2_il::func::glalias`**, `DISCLOSURE` **W-ALIAS-1** |
| `0x10b9c06a` | 104 | 1 | `0x05` | — |
| `0x10b9c0d2` | 177 | 1 | `0x07` | — |
| `0x10b9c183` | 143 | 1 | `0x08` | — |
| **`0x10b9c212`** | 143 | 1 | **`0x09` — the SECTION DEFINITION record** | **—** |
| `0x10b9c2a1` | 122 | 1 | `0x0B` (undecorated name — `glalias.rs`'s own note) | — |
| `0x10b9c31b` | 27 | 1 | `0x0D` | — |
| `0x10b9c336` | 124 | 1 | `0x12` | — |
| `0x10b9c3b2` | 146 | 1 | `0x0A` | — |
| `0x10b9c444` | 26 | 1 | `0x06` | — |
| `0x10b9c45e` | 261 | 1 | `0x19` | — |
| `0x10b9c563` | 30 | 1 | `0x18` | — |
| `0x10b9c581` | 73 | 1 | `0x1B` | — |
| `0x10b9c5ca` | 11 | **8** | `0x0C` `0x0F` `0x11` `0x13` `0x14` `0x15` `0x16` `0x17` — **fatal** | n/a |

**Only the arm's tag routing and extent are read here.** Thirteen of the
fifteen live arms have **no semantics in this tree at all** — the tag values
they serve are known and their bodies are not. That is the honest state, and
it is the largest single unread block the section subsystem owns.

### 7.2 `ported` = **1 of 15**, and the load-bearing half is the 14

The metric is `subsys-metric section-ported 1` / `section-ported-den 15`,
recomputed on every `cargo test` by `subsys::recount_section_ported`.

**The one**: `0x10b9bdcf`, the shared tag-`0x04`/`0x0E`/`0x10` handler,
decoded by `crates/c2-il/src/func/glalias.rs` under `DISCLOSURE.md`
**W-ALIAS-1** and **W-ALIAS-2**, with `W-OBJPLAN-1` (the emit-seed bit `0x20`
at `sym+0x4c`) and `W-STAGETAP-6` (the name route through `0x10b99dfe`)
landing on the same handler and on §1's name formatter. **All three adopted
rows in this subsystem sit on two of the twenty-five §1 entries.**

**The fourteen, named** — this is the residue, and it is worth more than the
ratio:

1. **`0x10b9c212`, tag `0x09`, the section-definition record.** §3 reads all
   six of its fields and obj-checks **every one** by mutating real `.gl`
   bytes. **The port does not read it.** It carries seventeen fully-resolved
   `(name, Characteristics)` constants — `CH_TEXT`, `CH_BSS_BASE`,
   `CH_CRT_XCU`, `CH_DRECTVE`, `CH_DEBUGS`, `CH_XBLD_C1/C2`, … — where c2 has
   an IL-borne name, an IL-borne kind, an IL-borne override, a kind switch
   (`0x10b982d6`), a remapper (`0x10be7727`), a base resolver (`0x10be76d4`)
   and an alignment chooser (`0x10be77a3`). **The port's section model is the
   OUTPUT of c2's section model, tabulated.** That is the single most
   consequential sentence on this page for anyone pricing goal (2): every
   section name the port can emit is one somebody measured, and a TU whose IL
   names a section outside that set has no path through the port at all.
2. **`0x10b9b945` (tags `0x01`/`0x02`/`0x1A`).** `glalias.rs` names `0x01`
   and `0x02` as `KIND1_TAGS`, but as **pattern locators to step past**, not
   as a decode — and it names `0x1A` nowhere. Uncited, unported.
3. **The twelve remaining live arms** — `0x10b9bd3d` `0x10b9c06a`
   `0x10b9c0d2` `0x10b9c183` `0x10b9c2a1` `0x10b9c31b` `0x10b9c336`
   `0x10b9c3b2` `0x10b9c444` `0x10b9c45e` `0x10b9c563` `0x10b9c581` — of
   which this tree knows **only the tag each serves**.

**The port has no `.gl` record-stream decoder.** `c2_il::func::gl` — 227 KB,
the largest reader in the workspace — scans name runs and TYPE tags and
**never consumes a record tag**. `glalias` is the only module that decodes a
record *grammar*. That is why the citation predicate and the behavioural
reading agree on 15 of 15 cells here, and the agreement was checked rather
than assumed.

### 7.3 The denominator, chosen out loud

`w-encmap` found the encoder had three defensible denominators up to 5.6×
apart. This row has five. All were measured on this tree; the choice is the
first and the reason is beside it.

| # | denominator | measured | why not chosen |
|---|---|---:|---|
| **A** | **live dispatcher arms** | **15** | **CHOSEN** |
| B | arms including the refusal | 16 | a port that also refuses agrees with c2 by doing nothing; counting a refusal as an implemented site drifts the number upward for free, and `../../PROGRESS_METRIC.md` runs the other way |
| C | tag values | 27 (19 live) | the port names 5 (`0x01` `0x02` `0x04` `0x0E` `0x10`), **three of them as locators rather than decoders**, so the numerator would be 5 or 2 depending on a distinction the unit cannot express |
| D | this page's read entries | 25 rows / 22 entries / 20 in-band | **an address grep gives 2 and is WRONG** — see §7.4 |
| E | the band / the TU attribution | 137 / 327 | the port maps onto neither; this is the row's `sites`, a different population |

**The containment survives the unit change.** All fifteen arms live *inside*
`0x10b9b8e9`, which is one of this page's read entries, which is one of the
137 band sites. So `sites ⊇ read ⊇ ported` holds as a containment of **site
sets**; the three *counts* are in three granularities and their **ratios must
not be compared**. That is a strictly better-formed containment than the
`encode` row's, where `read` (79 arms) exceeds `sites` (14 functions).

### 7.4 Why the ENTRY unit was rejected — two rules the port implements and cites nothing for

A citation predicate normally measures documentation discipline rather than
implementation. On the arm unit that objection was tested and came back zero.
**On the entry unit it is real, and here are the two cases:**

| rule | where the port has it | cites |
|---|---|---|
| §2 step 3 — the alignment nibble, `log2(a) + 1` over a `1 … 0x2000` ladder (`0x10b28261`) | `c2_core::coff::container::align_nibble` | **nothing** — fitted from `../../OBJ_DYNINIT_SHAPE.md` §4.2 |
| §5 — `.bss` ascending address is the reverse of `.gl` record order for dyninit objects (`0x10b99093`) | `c2_core::coff::data`, Rule Y1 / A3 | **nothing** — §5 itself records it as *"independently confirmed black-box by lane `w-bss` from the IL alone"* |

Both were derived **black-box and only afterwards found to agree with the
read**. An address grep over the twenty-five entries scores **2** where the
honest answer on those two cells is **1** each. **The entry unit is therefore
not a denominator this page will defend**, and it is published as a rival
reading rather than suppressed.

This is also the sharper form of a claim `#3617` made and `#3636` refuted.
`#3617` said `ported` was ill-formed because the port is I/O-behavioral;
`#3636` answered that *how* a port function was obtained has no bearing on
whether it lands on an arm. **Both are right, on different units.** On the
arm unit `#3636` holds. On the rule unit `#3617` holds, and §7.4 is the
measurement that shows it: two rules obtained black-box, agreeing with the
read, joinable to it by nothing.

### 7.5 This page's own coverage line does not reproduce, and never did

The banner reads **`24 entries against a denominator of 137`**. Recounted on
this tree:

| reading | count |
|---|---:|
| rows in §1's table | **25** |
| of those, addresses that are Ghidra function entries | **22** (three — `0x10b9bdcf`, `0x10b9c212`, `0x10b9c5ca` — are addresses *inside* `0x10b9b8e9`, and §1 says so) |
| of those, entries inside the two bands that give the 137 | **20** (`0x10b805b3` is `misc.c`, `0x10c27b56` is `smdmisc.c`) |

**24 reproduces under none of them**, and `git log -S` puts the line at
**25 rows in the file's first commit** — so it was wrong when it was written,
not rotted since. Same family as `#3643` (`mop.rs`'s *"71 of c2's 660 rows"*
against a true 85, also wrong from the file's first commit), which is the
defect class lane `w-provaudit` is building an instrument for this wave.

**The line is left as written.** `subsys.rs` verifies it verbatim as the
`section` row's `den_probe`, and the standing convention here (§5's own
retraction is the model) is that an amendment stands *beside* the original
reading rather than replacing it. `subsys-metric section-read 24` is
therefore a **carried page figure that this page now flags**, not a recount.
