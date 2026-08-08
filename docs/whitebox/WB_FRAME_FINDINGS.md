# WB_FRAME — the frame-opening predicate and the frame-size arithmetic

> **PROVENANCE — DISASSEMBLY-DERIVED.** Lane WB-B of
> [`CAMPAIGN_2026-08-08.md`](CAMPAIGN_2026-08-08.md). Every address below is an
> absolute VA in the exact image pinned in [`C2_MAP_METHOD.md`](C2_MAP_METHOD.md)
> §0 — `sha256 c80981c015166effecc71ad8112d5577a065b2300891dfdb02b9c13787a66258`,
> verified at the top of this lane. This is **navigation** until a row lands in
> [`DISCLOSURE.md`](DISCLOSURE.md). **The obj is the sole judge** (method doc §7).

PREREG for this lane is in [`../rungs/2026-08-08-wb-frame.md`](../rungs/2026-08-08-wb-frame.md),
committed at `2e1d858` before the first grep of the flat export.

---

## 0. The headline, stated first because it changes the follow-on lane's job

**Board #1477's diagnosis of `?supershuffle@@YAXPAD@Z` is wrong, and this lane
retracts it.** The row reads *"c2 opens a 96-byte frame where the port does
not"*. The port opens exactly the same 96-byte frame, one word later. §1 has the
two word streams side by side. The follow-on code lane (task `w-frame`, board
#1477/#151) would have shipped a frame RULE that moves this function by **zero
bytes**.

The frame reading in §2–§4 is still delivered, still obj-checked (§5), and is
still worth having — it is just not what the anchor was about.

---

## 1. The anchor, measured — the frame is NOT the defect

`?supershuffle@@YAXPAD@Z`, `src/keygen_xbox.cpp`, workload flags
(`/O1 /Oi /EHsc /GR …`). Port stream from `c2rs gap --fnbyte-diff-jsonl`;
reference stream from the same record, cross-read against c2's own `/FAsc`
listing.

| # | port | | # | c2 (ref) | |
|---:|---|---|---:|---|---|
| 0 | `7d8802a6` | `mflr r12` | 0 | `7d8802a6` | `mflr r12` |
| 1 | `9181fff8` | `stw r12,-8(r1)` | 1 | `9181fff8` | `stw r12,-8(r1)` |
| **2** | **`fbe1fff0`** | **`std r31,-16(r1)`** | | *(absent)* | |
| **3** | **`9421ffa0`** | **`stwu r1,-96(r1)`** | **2** | **`9421ffa0`** | **`stwu r1,-96(r1)`** |
| 4 | `7c7f1b78` | `mr r31,r3` | 3 | `4bfffff5` | `bl ?shuffle1` |
| 5 | `4bffffed` | `bl ?shuffle1` | 4–17 | 14 words | *inlined `?shuffle2` loop* |
| 6,8,10,12,14 | `7fe3fb78` ×5 | `mr r3,r31` | | *(absent)* | |
| 7,9,11,13,15 | `4bffff…` ×5 | `bl ?shuffle2…6` | 18–21 | `4bffffb9…ad` ×4 | `bl ?shuffle3…6` |
| 16 | `38210060` | `addi r1,r1,96` | 22 | `38210060` | `addi r1,r1,96` |
| 17 | `8181fff8` | `lwz r12,-8(r1)` | 23 | `8181fff8` | `lwz r12,-8(r1)` |
| 18 | `7d8803a6` | `mtlr r12` | 24 | `7d8803a6` | `mtlr r12` |
| **19** | **`ebe1fff0`** | **`ld r31,-16(r1)`** | | *(absent)* | |
| 20 | `4e800020` | `blr` | 25 | `4e800020` | `blr` |

`9421ffa0` is **present in both** and identical. Both frames are 96 bytes; the
two get there by different routes and the routes happen to collide —
`align16(80 + 8·0 + 8) = 96` for c2 and `align16(80 + 8·1 + 8) = 96` for the
port, because 88 and 96 both round to 96. That coincidence is why the row's
author, reading only "first differing word", concluded the frame was missing.

**The three real defects, in size order:**

1. **c2 inlined `?shuffle2`** and the port did not — 14 words of `lbz/stbu/bdnz`
   loop replacing one `bl`. This is the whole `ins/del` mass and 12 of the 12
   substitutions. It is an **inliner** question, not a frame question.
2. **The port needs a callee-saved GPR for the incoming pointer and c2 does
   not.** c2 leaves `c` in the *volatile* `r3` across all six calls and reads it
   back afterwards (`addi r11,r3,-1` at `+0x14`, *after* `bl ?shuffle1`). It can
   do that because the callees are in-TU and provably do not write `r3` —
   `?shuffle1` is emitted frameless and never assigns `r3` (verified in the same
   `.cod`). Cost to the port: `std r31,-16(r1)` + `ld r31,-16(r1)`.
3. **Five `mr r3,r31`** the port emits to re-materialise the argument — a
   consequence of 2, not an independent defect.

`.pdata` corroborates the frame decision independently: c2's second
`RUNTIME_FUNCTION` word for this symbol is `0x40001A03` → `PrologLen = 3`
instructions, `FunctionLen = 26` words, `32-bit = 1`, `ExceptionFlag = 0`. Three
prologue instructions is `mflr / stw / stwu` — no saved register.

---

## 2. The predicate, read off the disassembly

Three functions, all in the `p2\ppc\code.c` range of
[`c2_tus.tsv`](c2_tus.tsv) (`code.c` anchor `10bf9f15` … `inlnasm.c` anchor
`10c01d50`), plus the allocator in the `lower.c`…`mdlist.c` gap.

### 2.1 `FUN_10bff95c` — the prologue driver

```
local_b8 = FUN_10bfebf7(blocklist);   /* callee-saved GPR bitmask   */
uVar1    = FUN_10bff507();            /* THE PROLOGUE FLAG WORD     */
if (DAT_10c3de20 == 1 && FUN_10b9c8cb(fn) != 0)   /* POGO/instrumented build */
        { local_b8 |= 0xc000; uVar1 |= 4; }
bVar7 = (byte)(uVar1 | local_b8);
FUN_10bfec72(fn, blocklist, ip, bVar7, local_b8, 1);
```

`local_b8`'s set bits are all in 14..31 (§2.3), so **the low byte of `bVar7` is
`FUN_10bff507()`'s return value**, plus a forced bit 2 on the POGO path.

### 2.2 `FUN_10bff507` @ **`0x10bff507`** — the flag word, and the frame bit

One linear scan of the function's instruction list. It sets three bits; **bit 2
is the frame bit** and it is set at exactly one instruction in the whole image:

| VA | instruction | meaning |
|---|---|---|
| `10bff544` | `cmp BYTE PTR [ecx+0x8],0x1b` | block-terminator kind; with `[+0x24]+0x31 ∈ {'V','T'}` the scan **stops** (`10bff550`, `10bff559`) |
| **`10bff565`** | **`cmp BYTE PTR [ecx+0x8],0xf`** | **instruction kind `0x0f` → jump to the frame bit** |
| **`10bff56e` / `10bff573`** | `cmp bl,0x12` / `cmp DWORD PTR [ecx+0x4],0x2e0` | **kind `0x12` with opcode `0x2e0` (an EH pseudo-op) → same** |
| `10bff57c` | `cmp bl,0x14` | kind `0x14` contributes nothing |
| `10bff5dd` | `or eax,0x1` | bit 0 — LR must be spilled (opcode `0x281`, or a class-2/3 instruction whose destination operand names the LR pseudo-register `DAT_10c6fd9c`) |
| `10bff5f2` | `or eax,edx` (`edx = 2`) | bit 1 alone — opcodes `0x2df` / `0x301`, and a class-2 destination symbol id of 3 |
| **`10bff5f6`** | **`or eax,0x6`** | **the only site in `c2.dll` that sets bit 2** |

`0x2e0` is confirmed EH-adjacent independently: `FUN_10bd5228`'s tuple switch
routes kind `0x12` opcode `0x2e0` to `FUN_10be4598`, inside the `ehexcept.c` /
`except.c` band of `c2_tus.tsv`.

### 2.3 `FUN_10bfebf7` @ **`0x10bfebf7`** — the callee-saved GPR mask

Scans every instruction with `[+9] & 1`, walks its operand list, and for each
operand that is a register definition (`[op+8] == 1`) whose register number
`n` satisfies `LO <= n <= 0x20` sets bit `n-1`. `LO` is `0x0f` normally and
`0x12` when `DAT_10c2e980 != 0`. Register numbers are 1-based, so `0x0f..0x20`
is **`r14..r31`** — the callee-saved GPR file, exactly. `popcount(mask)` is the
`nSaved` the size arithmetic uses.

### 2.4 `FUN_10bfec72` @ **`0x10bfec72`** — the prologue emitter

| VA | instruction | what it gates |
|---|---|---|
| `10bfec7d` | `test BYTE PTR [ebp+0xc],0x1` | bit 0 → the LR spill (`FUN_10c07910(LR,-8,…)` at `10bfeca1`, then `FUN_10bfeb52`) |
| `10bfecbf`–`10bfed10` | the `0x20 → 0x0f` descending loop | inline callee-saved stores at `-8·k`, highest register first, starting one slot below the LR slot |
| **`10bfed12`** | **`test BYTE PTR [ebp+0xc],0x4`** | **bit 2 → establish the frame** |
| `10bfed27` | `call 0x10c07910` with register `0x53` | the frame-establish pseudo-register — emits no PPC instruction (`0x53` takes `FUN_10c07910`'s `0xe6` arm), it is the unwind/`.endprolog` record |
| **`10bfed2f`** | **`mov eax,DWORD PTR [eax+0x68]`** | **the base frame size, `fn+0x68`** |
| **`10bfed35`** | **`lea edx,[eax+edi*8+0x8]`** | **`size = base + 8·nSaved + 8`** |
| `10bfed39` | `call 0x10c0b6fa` | the allocator |

---

## 3. The size arithmetic, read off the disassembly

### 3.1 `FUN_10c0b6fa` @ **`0x10c0b6fa`** — the frame allocator

| VA | instruction | meaning |
|---|---|---|
| **`0x10c0b706` / `0x10c0b708`** | `test esi,esi` / `jle 0x10c0b8c7` | **nothing at all is emitted when the computed size is ≤ 0** |
| **`0x10c0b71f` / `0x10c0b722`** | `lea ecx,[esi+0xf]` / `and ecx,0xfffffff0` | **`F = align16(size)`** |
| `0x10c0b72e`–`0x10c0b73f` | `and esi,0xffff8000`, `cmp esi,edx` | does `−F` fit a signed 16-bit immediate |
| **`0x10c0b745`–`0x10c0b752`** | `mov ebx,ds:0x10c386f4` / `imul edx,edx,0x5` / `cmp ecx,edx` / `jge` | **`F >= 5 × PAGE` takes the `_RtlCheckStack12` path** |
| `0x10c386f4` | data | **`0x1000` — the page, initialised in the image** |
| `0x10c0b799` | `mov ecx,0x29f` | the per-page probe (`ld r12,−k·PAGE(r1)`), one per page below `F` |
| `10c0b7f5`-ish | `piVar6[1] = 0x17e` | `stwu r1,−F(r1)` |
| `0x10c0b83b`-ish | `piVar6[1] = 0x17f` | `stwux r1,r1,r12`, after `bl _RtlCheckStack12` (string `_RtlCheckStack12` at `0x10b19700`, marked `\|= 0x10800` at `10c0b7xx`) |

### 3.2 What this says about the published rule

Composing §2.4 and §3.1:

> **`F = align16( [fn+0x68] + 8·nSaved + 8 )`**

`docs/CODEGEN_FRAMED_CALLS.md` §1.2, derived entirely black-box from 44
witnesses plus a 441/480 refutation sweep, reads

> `F = align16( max(16 + 8·max(nOutSlots,8), localsBase + localsBytes) + 8·nSaved + 8 )`

These are **the same expression** with `[fn+0x68]` naming the base term. The
disassembly adds three things the sweep could not see and does **not** contradict
anything it did:

* the `+8` LR slot and the `8·nSaved` term are *separate additions at one site*
  (`10bfed35`), not a fitted constant;
* `align16` happens **once, in the allocator, after** both additions
  (`10c0b722`) — so an implementation that rounds the base first is wrong on any
  input where the base is not already 16-aligned;
* the page constant is a **datum** (`0x10c386f4 = 0x1000`) and the
  `_RtlCheckStack12` threshold is literally `5 ×` it, both matching
  `crates/c2-core/src/codegen/frame.rs`'s `FRAME_PAGE` and its 5-page comment.

The 39 refutations at `nSaved ≥ 18` are **not** explained by this reading: the
spill area they see enters through `[fn+0x68]`, computed elsewhere, and this lane
did not read that site. Stated so absence does not read as coverage.

---

## 4. `?supershuffle` specifically — why 96

`nOutSlots = 1` (every callee takes one pointer) → floored to 8 →
base `= 16 + 8·8 = 80`. No stack locals, so the base stays 80.
`nSaved = 0` (§1: `r3` carries the pointer, no `r14..r31` is defined, so
`FUN_10bfebf7` returns 0). Then `10bfed35` computes `80 + 0 + 8 = 88`, and
`10c0b722` rounds: **`align16(88) = 96 = 0x60`**, immediate `0x10000 − 0x60 =
0xffa0` → `9421ffa0`. Bit 2 is set because the body contains calls.

---

## 5. THE OBJ-CHECK GRID — frozen before the first `cl.exe`

Sources: [`grids/wb-frame/frame_grid.cpp`](grids/wb-frame/frame_grid.cpp), one
COMDAT per cell. Compiled with the real `cl.exe` 16.00.11886.00 under wibo at the
workload mode (`/O1 /Oi /EHsc /GR`) and at `/O1 /GS- /c`.

**Measured quantity**: *is a frame open* — does the emitted `.text` for the cell
contain `stwu r1,−F(r1)` (`0x9421xxxx`) or `stwux r1,r1,r12` (`0x7C21616E`)?
Recorded beside it: `F`, `nSaved` read from the prologue, and the `.pdata`
`PrologLen`.

### 5.1 The rivals

| id | predicate |
|---|---|
| **R0** | **(this lane's reading)** frame iff the body contains a call — any `bl`, including a compiler helper — or an EH pseudo-op, or the build is POGO-instrumented. Locals, saved registers and FPR use are **not** inputs. |
| R1 | frame iff the body contains a **non-tail** call (a pure tail call needs no LR slot, so no frame) |
| R2 | frame iff the function has any stack local (`localsBytes > 0`) |
| R3 | frame iff any callee-saved GPR/FPR is saved (`nSaved > 0`, as measured in the emitted prologue) |
| R4 | frame iff the function uses FPRs |
| R5 | frame iff `localsBytes > 64` (a small-leaf red zone below `r1`) |

### 5.2 Frozen predictions

`Y` = a frame is open, `N` = none. **This table was committed before the grid was
compiled.**

| cell | shape | R0 | R1 | R2 | R3 | R4 | R5 |
|---|---|---|---|---|---|---|---|
| C1 | leaf, no locals, no calls | N | N | N | N | N | N |
| C2 | leaf, 256 B runtime-indexed local array | N | N | **Y** | N | N | **Y** |
| C3 | leaf, `double` arithmetic, no locals | N | N | N | N | **Y** | N |
| C4 | leaf loop, 16 accumulators (forces callee-saved) | N | N | N | **Y** | N | N |
| C5 | one call, result consumed | **Y** | **Y** | N | N | N | N |
| C6 | pure tail call | **Y** | N | N | N | N | N |
| C6b | tail call, transformed argument | **Y** | N | N | N | N | N |
| C6c | void tail call | **Y** | N | N | N | N | N |
| C7 | 64-bit divide — helper call, none in source | **Y** | **Y** | N | N | N | N |
| C9 | leaf, one 4-byte escaping local | N | N | **Y** | N | N | N |
| C9b | leaf, 16 B escaping local array | N | N | **Y** | N | N | N |

R3's column is defined against the **measured** `nSaved`, so C2/C3/C5/C7 are
entered as `N` on the assumption that those shapes save nothing; if a cell
turns out to save a register the cell is reported as **assumption unmet** and
excluded from every pair it was carrying, and separation is re-asserted on what
remains. That rule is registered here, before the run.

### 5.3 Separation assertion — every rival pair, minimum 2 discriminating cells

w-clear's confound was a grid whose cells could not tell two live hypotheses
apart. Asserted from the table above, **before running**:

| pair | discriminating cells | n |
|---|---|---|
| R0–R1 | C6, C6b, C6c | 3 |
| R0–R2 | C2, C5, C6, C6b, C6c, C7, C9, C9b | 8 |
| R0–R3 | C4, C5, C6, C6b, C6c, C7 | 6 |
| R0–R4 | C3, C5, C6, C6b, C6c, C7 | 6 |
| R0–R5 | C2, C5, C6, C6b, C6c, C7 | 6 |
| R1–R2 | C2, C5, C7, C9, C9b | 5 |
| R1–R3 | C4, C5, C7 | 3 |
| R1–R4 | C3, C5, C7 | 3 |
| R1–R5 | C2, C5, C7 | 3 |
| R2–R3 | C2, C4, C9, C9b | 4 |
| R2–R4 | C2, C3, C9, C9b | 4 |
| R2–R5 | C9, C9b | **2** |
| R3–R4 | C3, C4 | **2** |
| R3–R5 | C2, C4 | **2** |
| R4–R5 | C2, C3 | **2** |

Minimum over all 15 pairs = **2**. The assertion holds; the grid runs.

**Declared NOT separated, so absence does not read as coverage.** The EH arm of
R0 (`kind 0x12 / opcode 0x2e0` at `10bff573`) and the POGO arm (`10bff95c`'s
`or …,4`) have **no cell**: EH without any call is not expressible in C++ that
this toolchain will emit, and the workload is not a POGO build. Those two
disjuncts of R0 are **read but unchecked** and are claimed as navigation only —
no DISCLOSURE row is proposed for either.

### 5.4 Results

*(filled in from the run; see §5.5 for the survivor)*

---

## 6. Pre-drafted DISCLOSURE rows

*(filled in after §5.4)*
