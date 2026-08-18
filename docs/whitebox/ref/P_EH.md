# `P_EH` — EH state synthesis: `ehexcept.c`, `except.c`, and the `.pdata` drivers

> **Reference page.** **`[R]`** read from the disassembly, *not* obj-checked —
> a hypothesis. **`[O]`** confirmed against a real obj or `/FAsc` listing, with
> the witness named. **`[I]`** an interpretive step. Navigation only; nothing
> here may enter `crates/` without a [`DISCLOSURE.md`](../DISCLOSURE.md) row.
> Index: [`ADDR.tsv`](ADDR.tsv) · front door: [`README.md`](README.md)

**Coverage: 19 entries against a denominator of 47** — Ghidra functions in the
EH band `0x10be04e7`–`0x10be3800` (`ehexcept.c`'s anchor and the record
builders in the gap before it). The `.pdata` drivers (`0x10c217fd`…) and the
deferred patch pass (`0x10bff785`/`0x10bff811`) are outside that band and are
listed anyway because the record cannot be read without them. Not covered:
`ssa_seh.c`, the `.xdata` throw-side records (67 workload objs, all STLport),
and the state-propagation fixpoint's proof.

> ### This page is the strongest section of the reference, and the reason is
> **`src/Main.cpp` converted on it.** Lane `w-main2` (`#2970`–`#2978`) emitted a
> byte-exact obj, so §2 and §3 below are not a reading that happens to survive a
> grid — they are a reading the port **reproduced**. Where a claim has that
> status it is marked `[O] port`.

---

## 1. Entries

| addr | size | callers | callees | TU | cites | what |
|---|---:|---:|---:|---|---:|---|
| `0x10be12c7` | 3022 | 1 | 13 | *(gap before `ehexcept.c`)* | 12 | **the `__ehfuncinfo$` record builder.** Mints `__unwindtable$%s`, `__tryblocktable$%s`, `__catchsym$%s$%d`, `__estypeinfo$%s` and writes the nine `FuncInfo` fields `[R]` · §3 `[O] port` |
| `0x10be0a0a` | 169 | 1 | 7 | *(gap)* | 1 | mints `__ehfuncinfo$%s` (string `0x10b18c14`, referenced at `0x10be0a84`) `[R]` |
| `0x10be04e7` | 102 | 1 | 3 | *(gap)* | 1 | mints `__catch$%s$%d` (`0x10b18c04` @ `0x10be0534`) — the **x86-shaped** name `[R]` |
| `0x10be1f3f` | 1220 | 1 | 28 | *(gap)* | 1 | mints `__unwindfunclet$%s$%d` (`0x10b18c84` @ `0x10be21ed`) `[R]` |
| `0x10be32de` | 594 | 1 | 5 | **`ehexcept.c`** | 1 | mints `__unwind$%s$%d` (`0x10b18d00` @ `0x10be341c`) `[R]` |
| `0x10b99dfe` | 682 | 3 | 6 | **`p2symtab.c`** | 14 | **the symbol-name formatter, and the PPC funclets use THIS, not the `%s$%d` strings.** §4 `[R]` · `[O]` |
| `0x10c21fd2` | 247 | 2 | 4 | *(gap)* | 6 | **the C++ EH `.pdata` driver** — mints `__CxxFrameHandler` at `0x10c22007`, sets sub-kind `'V'`, then walks the node list cutting `.text` into regions `[R]` |
| `0x10c21b03` | 752 | 1 | 15 | *(gap)* | 1 | the **SEH** driver (`__C_specific_handler`, sub-kind `'S'`) — **and the path a NON-EH function takes**: `FUN_10c217fd(…, 0, 0, …)`, handler null `[R]` |
| `0x10c217fd` | 405 | 2 | 10 | *(gap)* | 6 | **the `.pdata` RECORD writer.** Allocates an 8-byte COMDAT at align 4 (`FUN_10b9c655(6, 8, 4, 0, 4, 0x80)`), emits `{ADDR32 BeginAddress, u32 unwind}` and defers the immediate `[R]` · `[O] port` |
| `0x10bff785` | 140 | 1 | 2 | `code.c` gap | 2 | **defer** — pushes a 6-field `0x18`-byte fixup `{item, base, end, prologEnd, handler, !isMainBody}` onto `DAT_10c385e4` `[R]` |
| `0x10bff811` | 187 | 1 | 1 | `code.c` gap | 6 | **the deferred unwind-word pass**, run once from `0x10b34325`. §2 `[R]` · `[O] port` |
| `0x10c220c9` | 418 | 1 | 4 | *(gap)* | 9 | **the `$T` ip-to-state array emitter** — one 8-byte `{ADDR32 label, i32 state}` per state **change**. §3.2 `[R]` |
| `0x10c219a6` | 30 | 1 | 0 | *(gap)* | 7 | seeds `node[+0x24] = -2` on every node with `[+9] & 1` — the "unset" sentinel the emitter skips `[R]` |
| `0x10c219c4` | 97 | 1 | 0 | *(gap)* | 9 | the forward monotone state propagation. **Read; its fixpoint is NOT established** `[R]` |
| `0x10c22046` | *(in `0x10c21fd2`)* | — | — | *(gap)* | 5 | **the region-cut predicate**: a kind-`0x1b` LABEL node whose symbol sub-kind is `'V'` (`__catch$`) or `'T'` (`__unwind$`) at `0x10c22054`/`0x10c22058` `[R]` |
| `0x10c22080` | *(in `0x10c21fd2`)* | — | — | *(gap)* | 6 | **the handler mask** — `neg/sbb/not/and` applied twice: an `__unwind$` region passes `handler = 0` and `handlerData = 0`, so it gets **no prefix** `[R]` · `[O]` E3 |
| `0x10be76a8` | 44 | 4 | 2 | **`emit.cpp`** | 2 | creates `.pdata` (group `"PDATA"`, index `0xe`, alignment `[+0x43] = 4`, `[+0x4b] = 1`) `[R]` |
| `0x10be7b4b` | 44 | 1 | 2 | **`emit.cpp`** | 5 | creates `.xdata` (group `"XDATA"`, index `0x10`, same alignment) `[R]` |
| `0x10b3421b` | 382 | 1 | 27 | `dag.c` gap | 4 | the one caller of the patch pass (`0x10b34325`) `[R]` |

**c2 mints every EH symbol name — none of them travels in the IL** `[R]`. The
format strings live at `0x10b18c04`…`0x10b18d00` (x86 shapes, `%s$%d`) and
`0x10b16624`/`0x10b16630` (`__unwind$`, `__catch$` — the short PPC forms).

---

## 2. The `.pdata` record

### 2.1 Two u32, built as `{reloc, immediate}` `[O]`

* **`BeginAddress`** — an `ADDR32` against the symbol reached through
  `param_2[+0x28]`. For the main body that resolves to the **function symbol**
  (`Value = 8`); for a funclet, to its `__catch$`/`__unwind$` label. The obj
  carries the difference as an **addend** (`?f@@YAHH@Z+72`, `+128`, `+164` on the
  three E3 cells), so a port's model must be *function symbol + offset*, not one
  symbol per region.
* **the unwind word** — written as `0` and patched later.

**No third word and no `.xdata` for the body.**

### 2.2 The unwind word — `0x10bff811`

```
eax = (prologEnd.addr - base.addr) >> 2                 ; 0x10bff83a..
if (handler != 0 && entry5 == 0) eax -= 2               ; 0x10bff848..
edi = eax & 0xff                                        ; 0x10bff85f   <- PROLOG, 8 bits
eax = (end.addr - base.addr) >> 2                       ; 0x10bff859..
if (handler != 0 && entry5 == 0) eax -= 2               ; 0x10bff86c..
ebx = (handler != 0)                                    ; 0x10bff87d   setne bl
eax &= 0x3fffff                                         ; 0x10bff880   <- LENGTH, 22 bits
ebx = (ebx << 23) | eax                                 ; 0x10bff88b, 8e
ebx |= 0x400000                                         ; 0x10bff893   <- UNCONDITIONAL
word = edi | (ebx << 8)                                 ; 0x10bff899, 9c
```

> **`word = (hasHandler << 31) | (1 << 30) | ((len_words & 0x3FFFFF) << 8) |
> (prolog_words & 0xFF)`** `[R]`, `[O] port`

* **Bit 30 is set unconditionally** for **every** `.pdata` record c2 emits, EH
  or not. The port already ships this constant from captures — an **independent
  confirmation**, not new information, and **no `DISCLOSURE` row is owed**.
* The length field is **22 bits** and the prolog field **8 bits**. Nothing in
  the published black-box doc pins either width; both are new `[R]`.
* **`hasHandler` is the same value that decides whether the two-word prefix is
  emitted** — one predicate, two consequences. E3's three-record obj separates
  that from both registered rivals `[O]`.

### 2.3 The `−2`, and why it is degenerate black-box

The two `sub eax,2` fire iff `handler != 0 && param_1->kind == 0x18` (the
function-start node kind) — i.e. **only for the main-body record of a function
that has a handler prefix** `[R]`. Two models fit every obj — *base = COMDAT
start, minus 2* and *base = function symbol (`Value = 8`), no correction* —
because the prefix is always exactly two words, so **black-box work cannot
separate them**. The whitebox reading picks the first; a port only needs the
observable, which is that the main record's length starts at the *code*.

### 2.4 Section order: reverse `[O]`

> **The `.pdata` COMDAT order in the obj is the REVERSE of the `.text` region
> order, and the reverse of the label-counter order.**

E3's three regions appear in the obj as **unwind funclet, catch funclet, main
body**, while `cl /FAsc`'s listing emits them **main, catch, unwind**
(`$T2596`/`$T2599`/`$T2602`, ascending). Two records made this look like "the
same reverse-emission order as the `.rdata` pool"; three make it a rule with a
direction.

---

## 3. The record set

### 3.1 `__ehfuncinfo$` — 9 dwords, magic first `[R]`, `[O] port`

| # | field | source in `0x10be12c7` |
|---:|---|---|
| 0 | **`magic = 0x19930522`** | `mov DWORD PTR [eax+0xe],0x19930522` at **`0x10be1425`** — the **only** `19930xxx` immediate in the image |
| 1 | `maxState` | `DAT_10c434cc` |
| 2 | `pUnwindMap` | the `__unwindtable$%s` COMDAT symbol, or `0` |
| 3 | `nTryBlocks` | `DAT_10c434c8`, **gated on option-word bit `0x400000`** |
| 4 | `pTryBlockMap` | the `__tryblocktable$%s` COMDAT symbol, or `0` |
| 5 | `nIPMapEntries` | written by `0x10c220c9` |
| 6 | `pIPtoStateMap` | ditto |
| 7 | `pESTypeList` | the `__estypeinfo$%s` COMDAT, gated on `param_1[0x97] & 1` |
| 8 | `EHFlags` | `(param_1[0x25] & 8) != 0` |

Sizes: `__unwindtable$` is `maxState × 8`; `__tryblocktable$` is
`nTryBlocks × 0x14`; each `__catchsym$` handler array is `n × 0x10`. All three
use the same `FUN_10b9c655(6, size, 7, 0, 4, 0x80)` shape as `.pdata` but with
third argument **7** rather than 4 `[R]`.

### 3.2 The ip-to-state array — `0x10c220c9` `[R]`

```
prev = -1
for each node:
    if (node[+9] & 1) == 0: continue          # not state-bearing
    s = node[+0x24]
    if s == prev or s == -2: continue          # dedup, and skip "unset"
    emit ADDR32 -> label(node);  emit i32 s;  prev = s;  n += 1
if n: section = FUN_10b9c655(6, n << 3, 7, 0, 4, 0x80)
      FuncInfo[5] = n ; FuncInfo[6] = &section
else: FuncInfo[5] = 0 (kind 2) ; FuncInfo[6] = 0
```

**Two dedup rules and a sentinel, all three of which a port must reproduce**:
consecutive equal states collapse, `-2` means "no state assigned", and the array
is omitted entirely at `n == 0`.

### 3.3 The EH state map is NOT in the IL — and `P3.3` is retracted

> ⛔ **RETRACTION.** PREREG `P3.3` said the `5C` token links to the ip-to-state
> map. **It does not.** `5C` is `TYPE` + `i32c`, and across **five objs / nine
> tokens** its operand takes exactly two values, `1` (a destructible-object
> region) and `0x101` (a `try` block) `[O]`. Those are **neither** the states in
> the map **nor** `maxState` — c2 *derives* the map in its IR (`0x10c219a6`
> seeds, `0x10c219c4` propagates, `0x10c220c9` emits), and **4 `5C` tokens
> produce 18 map entries**.

What the tokens *do* determine, five for five `[O]`:

> **`nTryBlocks` = n₂₅₇** and **`maxState` = n₁ + 2·n₂₅₇**

---

## 4. Funclet numbers come from the label counter, not a private EH counter

`0x10b99dfe` dispatches on `sym[+0x30]` (kind) × `sym[+0x31]` (sub-kind):

| kind | sub-kind | name |
|---:|---|---|
| 3 | `'T'` (`0x54`) | `__unwind$` + decimal (`0x10b99f3d`) |
| 3 | `'V'` (`0x56`) | `__catch$` + decimal (`0x10b99f36`) |
| 3 | `'W'` | `$M` + decimal |
| 3 | `'Z'` | `__annotation$` + decimal |
| 3 | `0` | `<name>$` + decimal |
| 3 | other, unnamed | `$L` + `C`\|`L`\|`N` + decimal(`sym[+0x3f]`) [+ `@` + TU] |
| 1 | `'$'` / `'%'` / other | `$S` / `$SG` / **`$T`** + decimal |
| 4 | — | `$E` + decimal |

> **Every one takes its number from `sym[+0x28]`**, converted by the same
> `FUN_10c1e739(v, buf, cap, 10)` at `0x10b9a08e` `[R]`. So `__catch$N` /
> `__unwind$N` are **not** a private EH counter — they are `$M`/`$T`'s counter,
> and `LABEL_COUNTER.md`'s arithmetic is what must predict them.

E3's obj is consistent `[O]`: `$M2585…$M2590` (six ip2state labels), `$T2591`
(the array), `$T2596`/`$T2599`/`$T2602` (three `.pdata` COMDATs, stride 3),
`__catch$2575`, `__unwind$2576`.

**The PPC funclets use the SHORT forms** — `__catch$2554`, not `%s$%d`. The
`%s$%d` strings in §1 are the x86 shapes and are not what this target emits
`[O]`.

---

## 5. What is NOT known here

* **`0x10c219c4`'s fixpoint is not established** — the propagation is read; its
  termination and monotonicity were not proved. `medium`.
* **`ssa_seh.c`** (`0x10bcee41`): not opened.
* **`.xdata`**: exists (`0x10be7b4b`) but the workload's only users are
  throw-side records — 67 objs, all STLport. Not read.
* **The `5C` token's own semantics** beyond the two observed operand values.
* Five objs / nine tokens is the whole of §3.3's evidence base.
