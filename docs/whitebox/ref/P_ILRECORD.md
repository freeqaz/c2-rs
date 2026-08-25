# P_ILRECORD — the IL-record → codegen dispatch `FUN_10bc2d7a`

> **PROVENANCE — DISASSEMBLY-DERIVED.** Everything here was obtained by
> reading `compilers/X360/16.00.11886.00/c2.dll`, sha256
> `c80981c015166effecc71ad8112d5577a065b2300891dfdb02b9c13787a66258`,
> verified before the first address was quoted. Whitebox analysis is
> authorized and encouraged (`CLAUDE.md`, project owner, 2026-08-17).
> Nothing here may enter `crates/` without a `DISCLOSURE.md` row naming the
> address it came from. **No row is owed by the lane that wrote this page**
> — it adopted nothing.

**Read R5** of [`../READ_PLAN_2026-08-21.md`](../READ_PLAN_2026-08-21.md) §3,
lane `w-read-r5`, board **#3415**–**#3421**. Prereg:
[`../WB_ILRECORD_PREREG.md`](../WB_ILRECORD_PREREG.md) (committed first,
`52fa9d7bd`). Grade:
[`../WB_ILRECORD_FINDINGS.md`](../WB_ILRECORD_FINDINGS.md). Instrument:
[`../scripts/dump_ilrecord.py`](../scripts/dump_ilrecord.py), re-runnable and
sha256-fenced.

---

> ## ⛔ THE READ PLAN'S "189 ARMS" IS AN OPCODE COUNT. THERE ARE 62 ARMS.
>
> `READ_PLAN_2026-08-21.md` §3 and §4, `C2_MAP.md:1012`,
> `STEP5_PRICING_2026-08-21.md:139`, `WHITEBOX_LEVERAGE_2026-08-21.md:89` and
> `ARCHITECTURE_PROPOSAL_2026-08-20.md:968` all say **189 arms**. The number
> `189` is `0xBD − 0x01 + 1`: it is the size of the **opcode** domain, and it
> was read off `labels/W-IL.tsv:36`'s *"table 0x10bc4152, ops 0x01..0xBD"* as
> if the table were indexed by opcode. It is not.
>
> The switch is MSVC's **two-level** form — a byte index table, then a DWORD
> target table (§1.1). **189 opcodes → 62 distinct arms**, and **94 of the
> 189 opcodes route to a single arm that raises C1001** (§1.3). The real
> read is **61 arms serving 95 opcodes**, plus one refusal.
>
> This is the same class of error R2 pre-empted at the encoder (111 entries,
> 79 targets) — `P_ENCODE.md` §4. Here nobody had parsed the table:
> `ADDR.tsv:755` records `0x10bc4152` as `data`, **size 4, `unknown`**.

---

## 1. The function

| | |
|---|---|
| **entry** | `0x10bc2d7a` |
| **extent** | 5,080 B, `0x10bc2d7a` … `0x10bc4152` — the body ends **exactly** where the jump table begins `[R]` |
| **TU** | `e:\bt\278379\vctools\compiler\be\p2\reader.c` (`FUNCS.tsv:2870`; the TU string `0x10b17738` is referenced by the refusal arm itself, §1.3) |
| **callers** | 4 xrefs: `0x10bc159e`, `0x10bc3344`, `0x10bc4356`, `0x10bc497d`. `0x10bc3344` is **inside this function** — arm 20 recurses (§3.4) |
| **driver** | `0x10bc4715`, per-function: seeks `.sy@+0x58` then `.ex@+0x54`, then codegen (`labels/W-IL.tsv:37`) |
| **shape** | a **token loop**, not a one-shot dispatch (§1.2) |
| **calls out to** | **76 distinct direct callees over 174 call sites**, plus itself |

### 1.1 The dispatch `[R]`

```
10bc2e08  mov   edx, dword ptr [ebp - 0x34]        ; the opcode just read
10bc2e0b  lea   eax, [edx - 1]
10bc2e0e  cmp   eax, 0xbc
10bc2e13  ja    0x10bc4143                         ; out of range -> C1001
10bc2e19  movzx eax, byte ptr [eax + 0x10bc424a]   ; BYTE table, 189 entries, values 0..61
10bc2e20  jmp   dword ptr [eax*4 + 0x10bc4152]     ; DWORD table, 62 arm targets
```

| table | VA | extent | stride | entries |
|---|---|---|---:|---:|
| **byte index** | `0x10bc424a` | `…0x10bc4306` | 1 | **189** (opcode `0x01`…`0xBD`, index `op−1`) |
| **arm targets** | `0x10bc4152` | `…0x10bc4249` | 4 | **62** |

The byte table ends at `0x10bc4306` and the next function begins at
`0x10bc4307` — corroborated independently by `c2_strings.tsv:833`, which
listed `0x10bc4307` as a `reader.c` function long before this table was
parsed.

All **62 of 62** targets lie inside the body `[R]`. Arm bodies, measured as
the linear span from each target to the next distinct target: **min 7 B,
median 42.5 B, max 880 B, total 4,907 B** of the 5,080.

### 1.2 It is a loop, and the loop's exit is table-driven `[R]`

The prologue falls into a token loop:

```
10bc2dbb  lea   edx, [ebp - 0x34]
10bc2dbe  mov   ecx, esi
10bc2dc0  call  0x10bbc9ab          ; THE .ex TOKEN FETCH (WB_READER_FINDINGS §2)
...
10bc2e20  jmp   dword ptr [eax*4 + 0x10bc4152]
```

Almost every arm ends `jmp 0x10bc3ff6`, which is arm 0 — simultaneously an
arm (for opcodes `0x01`/`0x2a`/`0x2b`/`0x43`) and **the loop tail**. The tail
decides whether to iterate or return, and it decides it from the *same
per-opcode attribute table the reader uses*:

```
10bc401e  mov   eax, 0x1000
10bc4023  test  word ptr [ecx*2 + 0x10b25f10], ax   ; attr[opcode] & 0x1000
10bc402b  je    0x10bc411e                          ; -> loop back to 0x10bc2db8
```

`0x10b25f10` is the per-opcode `u16` attribute table read by `wb-reader`
(board **#1591**), where it was used with bit `0x400`. **Bit `0x1000` is the
"this opcode ends the record group" bit** — a new fact about a known table.
The prologue uses a third bit, `0x8000`, at `0x10bc2dda`.

### 1.3 94 of 189 opcodes REFUSE — arm 61 `[R]`

```
10bc4143  mov   edx, 0xcdf            ; 3295
10bc4148  mov   ecx, 0x10b17738       ; "…\be\p2\reader.c"
10bc414d  call  0x10b33526            ; the C1001 reporter (WB_READER_FINDINGS:176)
```

Arm 61 is the target of **both** the out-of-range `ja` **and** 94 in-range
opcodes. `0x10b33526` is the same ICE entry the operand-class `0B` arm calls.

**The dispatch is not total over the IL opcode space, and the incompleteness
is enormous**: `0x07 0x14 0x1d 0x1e 0x25 0x2d 0x2e 0x2f 0x31 0x3f 0x45 0x48
0x49 0x4a 0x4d 0x4e 0x50 0x51 0x52 0x57 0x58 0x5b 0x5f 0x63 0x65 0x69
0x6a…0x76 0x78…0x8a 0x8c 0x91…0x98 0x9c 0x9e 0x9f 0xa1 0xa3…0xb8 0xba`.

Those opcodes are legal `.ex` tokens — the operand-class table `0x10b25e48`
assigns most of them class `00` and the reader parses them — but they are
**not legal in the context this walk runs in**. `0x10bc2d7a` is one consumer
of the `.ex` stream, not the consumer.

> **This bounds R5's own subject far more tightly than the read plan
> assumed, and it is the single most price-relevant fact on this page.** It
> is reported as a finding, **not** as a re-pricing — see §8.

---

## 2. The frame — how an arm sees the record `[R]`

The decoded record ("node") lives in the dispatch's own frame at
**`ebp−0x34`**, and `ebp−0x38` holds the walk's current list pointer. Arms
pass `ecx = ebp−0x38`, i.e. **a pointer to a two-word window whose `+0` is
the list cursor and whose `+4` is the node** — which is how the pure-router
arms (§3.1) hand the callee an opcode they never load themselves.

| frame slot | node offset | what, per `WB_READER_FINDINGS` §3 |
|---|---|---|
| `[ebp-0x34]` | `node+0` | **the opcode** |
| `[ebp-0x30]` | `node+4` | the composed **type word**; `>>12` is the type class |
| `[ebp-0x2e]` | `node+6` | the **flag word** — `0x4000` is the `0x27` bit, `0x2000` the `v & 0x10000` bit |
| `[ebp-0x2c]` | `node+8` | `ext`, when the type's top nibble is 6 or 7 |
| `[ebp-0x24]`/`[ebp-0x20]` | `node+0x10`/`+0x14` | the 64-bit immediate pair |
| `[ebp-0x14]` | `node+0x20` | the **symbol** (`sym(id)` from the reader's `varU`→`sym` paths) |
| `[ebp-0x10]` | `node+0x24` | the second symbol / aux operand |
| `[ebp-0xc]` | `node+0x28` | the **`size_index`** (1→1 B, 2→2, 3→4, 4→8) |

> **This mapping is `[R]`, but it is cross-validated against an independently
> read document rather than asserted.** `WB_READER_FINDINGS` §3.2 step 7 says
> the reader stores `size_index` at `node+0x28`; arm 54 reads
> `movzx edx, byte ptr [ebp-0xc]` = `node+0x28` and passes it to a
> symbol-minting call. Step 8 says the reader sets `node[+6] |= 0x2000` from
> `v & 0x10000` and writes `ext` to `node+8` *iff the composed type's top
> nibble is 6 or 7*; arm 54 reads `[ebp-0x2e] & 0x2000` and takes
> `[ebp-0x2c]` on exactly the `>>12 ∈ {6,7}` branch. Two offsets and two
> predicates agree with a lane that never saw this function.

### 2.1 The walk's own state

Six frame slots are **not** part of the record and persist across tokens.
They are what makes this a walk rather than a mapper, and an implementation
of I1 must model them:

| slot | set by | meaning |
|---|---|---|
| `[ebp-0x4c]` | arms 52 (`=1`), 53 (`=2`), cleared by 23, 28 | the **bind mode** — C3 |
| `[ebp-0x58]` | arm 52, consumed by 23, 28 | the bind operand |
| `[ebp-0x54]` | arms 31 (`=1`), 34 | "terminate the walk" |
| `[ebp-0x44]` | arms 33 (`inc`), 34 (`dec`), 55 | **scope depth** |
| `[ebp-0x64]` | cleared by 23 | pending statement info |
| `[ebp-0x40]` | arms 0, 26, 32 | the current output list head |

---

## 3. The 62 arms

**Legend.** `role` is what the arm does; `verdict` answers the read plan's
select-vs-decode question under the rule fixed in the prereg §P2 **before any
arm was read**:

| role | meaning |
|---|---|
| **ROUTE** | the whole body is argument setup + one call + a jump to the tail. The semantics are in the callee |
| **BUILD** | mints one or more IR nodes with a **literal** IR opcode |
| **STATE** | mutates walk state (§2.1), a symbol, or a global; builds nothing |
| **REWRITE** | pattern-matches nodes already built and rewrites them |
| **REFUSE** | C1001 |

| verdict | meaning |
|---|---|
| **DECODE** | one output, no choice; no global, no branch on type/mode |
| **SELECT** | chooses among ≥ 2 distinct outputs |
| **DEFER** | the choice, if any, is below this lane's depth-1 bound — **counted as unresolved, never guessed** |

`class` is the operand-format class from `0x10b25e48` (board #1591), joined
here with the semantic grammar **for the first time**, which is what the read
plan asked for.

| # | arm VA | opcodes | class | B | calls | globals | role | verdict | tuple(s) built / effect |
|--:|---|---|---|--:|--:|---|---|---|---|
| 0 | `10bc3ff6` | `01` `2a` `2b` `43` | 00,03,04 | 333 | 11 | `10c2f064` | STATE | SELECT | **the loop tail + epilogue.** Mints `0x310`/`0x311` scope markers; `attr&0x1000` decides iterate-vs-return |
| 1 | `10bc2ec1` | `02`…`0d` (10) | 00 | 13 | 1 | — | ROUTE | DEFER | → `0x10bc001d` (10 B thunk) |
| 2 | `10bc386f` | `08` | 00 | 18 | 1 | — | ROUTE | DECODE | → `0x10bc0f77(edx=0x2b5)` |
| 3 | `10bc3868` | `0e` | 00 | 7 | 0 | — | ROUTE | DECODE | `edx=0x2b4`, falls into arm 2 |
| 4 | `10bc37c3` | `0f`…`19` (10) | 01 | 16 | 1 | — | ROUTE | DEFER | → `0x10bc2cf2` |
| 5 | `10bc31fb` | `1a` | 00 | 89 | 1 | — | STATE | SELECT | flips `sym+0x15` nibble via `0x10b189cc`, toggles `sym+0x14` bit 0 |
| 6 | `10bc3254` | `1b` `1c` | 00 | 152 | 4 | — | REWRITE | SELECT | branches on `1b` vs `1c` and on `flags&3`; two list splices |
| 7 | `10bc38a1` | `1f`…`24` (6) | 00 | 13 | 1 | — | ROUTE | DEFER | → `0x10bbffbb`. **All six relational operators share one arm and no discriminator is passed** — the callee re-reads the opcode through `ecx+4`  ⚠ **SEE THE BANNER BELOW — this cell's last two clauses are WRONG (board #3547)** |

> ### ⚠ ARM 7's CELL IS CORRECTED — inserted 2026-08-25 by lane `w-relsite`, board **#3547**. The original row above is left exactly as written (the `#3495`-on-`#3468` convention); nothing in it was altered.
>
> **What is RIGHT, and it is the load-bearing part:** the arm VA `10bc38a1`, the
> six opcodes `1f`…`24`, the 13-byte length, the single call, and the callee
> `0x10bbffbb`. All five re-derived from raw image bytes. **That callee is the
> IL-opcode → relation-code site**, unnamed after two subsequent lanes searched
> for it (`w-c7` prereg W2, `w-relread` prereg S2c) — **because neither read this
> row**. `#3098`'s family, seventh instance (**#3546**).
>
> **What is WRONG, in both clauses:**
>
> * *"no discriminator is passed"* — **a discriminator IS passed, in `edx`.**
>   The dispatch head loads it at `0x10bc2e08` (`mov edx,[ebp-0x34]`) and
>   nothing on the path to arm 7 clobbers it, so the 13-byte arm does not need
>   to re-materialise it. `FUN_10bbffbb` reads it three instructions in:
>   `10bbffc6: mov eax,edx`.
> * *"the callee re-reads the opcode through `ecx+4`"* — **it never touches
>   `ecx+4`.** `ecx` on entry is `&[ebp-0x38]`, a pointer to the caller's record
>   cursor; the callee does `mov edi,ecx` then `mov ecx,[edi]` — offset **0** —
>   and writes the cursor back at `10bc0016: mov [edi],esi`.
>
> Together those two clauses say *"the discriminator is somewhere else, go look
> for it"*, which is the opposite of what the arm does and is why the site
> stayed unnamed. Full read, with the six literals and the confirmation probe:
> [`../WB_RELSITE_FINDINGS.md`](../WB_RELSITE_FINDINGS.md) §2, §3.
| 8 | `10bc2e27` | `26` | 02 | 17 | 1 | — | ROUTE | DEFER | → `0x10bc24a6` |
| 9 | `10bc3891` | `27` | 01 | 16 | 1 | — | ROUTE | DEFER | → `0x10bbfebb` (256 B). **C1, 33.3 % of the residue, is a 16-byte trampoline** |
| 10 | `10bc3881` | `28` | 02 | 16 | 1 | — | ROUTE | DEFER | → `0x10bbfe9a` |
| 11 | `10bc3117` | `29` | 02 | 66 | 1 | `10c2f078` `10c2f07c` | STATE | SELECT | threads a global list |
| 12 | `10bc2f75` | `2c` `34` | 05 | 80 | 2 | — | ROUTE | SELECT | branches on type class ∈ {3,4} and mask `0x1e` before → `0x10bc2458` |
| 13 | `10bc2ff1` | `30` `5a` | 01 | 92 | 1 | `10c6f29c` | ROUTE | SELECT | → `0x10bbf134`; opcode `5a` and type class 6/7 take extra paths |
| 14 | `10bc3784` | `32` | 01 | 63 | 1 | — | REWRITE | SELECT | → `0x10bc271a`, then tests `node+4 == 0x2af` |
| 15 | `10bc2fc5` | `33` | 06 | 44 | 1 | — | BUILD | SELECT | type class **5 (real)** → `0x10bd3a86`; else pushes the 64-bit pair |
| 16 | `10bc37d3` | `35` `36` | 01 | 149 | 7 | — | BUILD | SELECT | `0x2c5 + (op != 0x35)` |
| 17 | `10bc2f43` | `37` | 00 | 50 | 3 | — | STATE | DECODE | pops two, links, sets `flags |= 4` |
| 18 | `10bc304d` | `38` `39` | 02 | 159 | 4 | `10c6f298` | BUILD | SELECT | mints `0x2dd`; `op == 0x39` inverts a condition via `0x10b189cc` |
| 19 | `10bc30ec` | `3a` | 02 | 43 | 2 | — | BUILD | DECODE | mints `0x2de` |
| 20 | `10bc32ec` | `3b` | 02 | 500 | 13 | `10c2e2ec` `10c6f2a4` `10c3de20` | REWRITE | SELECT | **recurses into `0x10bc2d7a`**; mints `0x2af`; saves/restores `0x10c2e2ec` |
| 21 | `10bc34e0` | `3c` | 07 | 48 | 3 | `10c6f2a4` | BUILD | DECODE | allocates a `0x48` record, publishes it to `0x10c6f2a4` |
| 22 | `10bc3510` | `3d` | 08 | 69 | 2 | — | BUILD | DECODE | allocates a `0x48` record, copies range fields |
| 23 | `10bc371c` | `3e` `bd` | 09,19 | 85 | 1 | — | ROUTE | DEFER | → `0x10bc0fcc` (2,763 B) with **five** args including the bind mode and scope state |
| 24 | `10bc3697` | `40` | 01 | 133 | 1 | `10c472e8` | STATE | SELECT | **C2 intrinsic.** A `sub`/`je` chain over intrinsic ids `0xd8` `0xdb` `0x11e` `0x14c` `0x5fd`, plus ranges `0x5fe`…`0x607` and `0x65b`…`0x65c`; sets `fn+0x94 |= 0x200/0x400`, `fn+0x98 |= 0x1000/0x2000/0x4000` |
| 25 | `10bc3771` | `41` | 01 | 19 | 1 | — | ROUTE | DEFER | → `0x10bc253b` |
| 26 | `10bc3555` | `42` | 02 | 20 | 1 | — | ROUTE | DEFER | → `0x10bc00a1` (2,282 B) |
| 27 | `10bc3166` | `44` | 00 | 149 | 8 | — | REWRITE | SELECT | list splice keyed on the bind mode `[ebp-0x4c]` |
| 28 | `10bc3621` | `46` | 00 | 36 | 1 | `10c2e950` | ROUTE | DEFER | → `0x10bbe3f1`; publishes the bind operand to `0x10c2e950` |
| 29 | `10bc3651` | `47` | 00 | 70 | 4 | `10c3cf96` `10c3d72c` | STATE | SELECT | **branches on the global `0x10c3cf96`** — two entirely different behaviours |
| 30 | `10bc38ae` | `4b` | 00 | 13 | 1 | — | ROUTE | DEFER | → `0x10bbf6f8` |
| 31 | `10bc3645` | `4c` | 00 | 12 | 0 | — | STATE | DECODE | `[ebp-0x54] = 1` — terminate the walk |
| 32 | `10bc38bb` | `4f` | 0c | 23 | 1 | — | ROUTE | DEFER | → `0x10bbe561`. **The `0x4F` sub-record — read R9's target — enters here** |
| 33 | `10bc3580` | `53` | 00 | 52 | 1 | `10c2f060` | STATE | SELECT | scope **push**; mints `0x310` when depth > 1 |
| 34 | `10bc35b4` | `54` | 0d | 109 | 2 | `10c2f060` `10c2e2ec` | STATE | SELECT | scope **pop**; mints `0x311` |
| 35 | `10bc3570` | `55` | 01 | 16 | 1 | — | ROUTE | DEFER | → `0x10bc1a97` |
| 36 | `10bc38d2` | `56` | 00 | 35 | 1 | `10c2e2e0` `10c2e2ec` `10c6f2a0` | STATE | DECODE | resets three globals — a phase boundary |
| 37 | `10bc2ece` | `59` | 00 | 117 | 5 | — | BUILD | SELECT | mints `0x2b0` and/or `0x2b6` depending on `0x10bd53f2` |
| 38 | `10bc3b00` | `5c` | 13 | 16 | 1 | — | ROUTE | DEFER | → `0x10bc0e25` |
| 39 | `10bc3b10` | `5d` `5e` | 14 | 63 | 2 | — | BUILD | SELECT | `0x2fb + (op != 0x5d)`, type `0x1004`; attaches a `0x10`-byte payload |
| 40 | `10bc3b7d` | `60` `62` | 00 | 42 | 2 | — | BUILD | SELECT | `0x2fe + 2·(op != 0x60)` |
| 41 | `10bc3b4f` | `61` | 15 | 46 | 2 | — | BUILD | DECODE | mints `0x2ff` |
| 42 | `10bc3569` | `64` | 01 | 7 | 0 | — | STATE | DECODE | `flags |= 8` |
| 43 | `10bc3fe0` | `66` | 1a | 22 | 2 | — | BUILD | DECODE | mints a leaf of type `0x1004`; **also the shared push tail** `0x10bc3fec`/`3fef`/`3ff1` |
| 44 | `10bc39b6` | `67` | 1b | 11 | 0 | — | BUILD | DECODE | pushes the 64-bit pair, falls into arm 43 |
| 45 | `10bc3ba7` | `68` | 00 | 117 | 3 | `10c2f074` | BUILD | SELECT | mints `0x2c5`; type class 4 takes a rewrite path |
| 46 | `10bc3a11` | `77` | 01 | 222 | 12 | — | REWRITE | SELECT | a **multi-node sequence** — `0x2d4`, then `0x2c5`, with two list rotations |
| 47 | `10bc3c1c` | `8b` | 00 | 74 | 2 | `10c2f074` | BUILD | SELECT | mints `0x2c6`, then checks the result's own opcode |
| 48 | `10bc38f5` | `8d` | 01 | 94 | 7 | — | BUILD | DECODE | mints `0x2f4` **and** `0x2f5` — **the prologue pair read R6 names** |
| 49 | `10bc3967` | `8e` | 00 | 7 | 0 | — | BUILD | DECODE | `0x2f0`, falls into arm 50 |
| 50 | `10bc3953` | `8f` | 00 | 20 | 1 | — | BUILD | DECODE | `0x2ee`, type `0x1004` |
| 51 | `10bc396e` | `90` | 00 | 7 | 0 | — | BUILD | DECODE | `0x2ef`, falls into arm 50 |
| 52 | `10bc3987` | `99` | 1c | 35 | 0 | `10c472e8` | STATE | SELECT | **C3 bind.** `[ebp-0x4c] = 1`; captures the operand **only if `[0x10c472e8+0xcac] != 0`** |
| 53 | `10bc39aa` | `9a` | 01 | 12 | 0 | — | STATE | DECODE | **C3 bind.** `[ebp-0x4c] = 2`. Twelve bytes, no call, no branch |
| 54 | `10bc39c1` | `9b` | 12 | 80 | 3 | — | BUILD | SELECT | **C3 bind.** Mints a symbol via `0x10b8034a(sym, type, size_index, flags&0x2000)`; type class 6/7 passes `ext` instead of `type & 0xfff` |
| 55 | `10bc3aef` | `9d` | 00 | 17 | 1 | — | ROUTE | DEFER | → `0x10bc4307` (the function immediately after the tables) |
| 56 | `10bc3c66` | `a0` | 01 | 880 | 37 | several | REWRITE | SELECT | **the largest arm.** A deep peephole: matches `0x1a`/`0x1b` kinds, `node+4 == 0x2af`, `0x2e4`/`0x21`/`0x22`, then rewrites |
| 57 | `10bc3fd6` | `a2` | 17 | 10 | 1 | — | ROUTE | DEFER | → `0x10bd3aa8` (the class-`17` string payload) |
| 58 | `10bc2e38` | `b9` | 18 | 137 | 4 | — | BUILD | SELECT | |
| 59 | `10bc3975` | `bb` | 07 | 18 | 0 | — | STATE | DECODE | |
| 60 | `10bc3159` | `bc` | 00 | 13 | 1 | — | ROUTE | DEFER | → `0x10bc1e79` |
| **61** | `10bc4143` | **94 opcodes** | many | 15 | 1 | — | **REFUSE** | — | **C1001, `reader.c` line 3295** |

### 3.1 The counts

| | count | of |
|---|--:|--:|
| distinct arms | **62** | — |
| … real arms (excluding the refusal) | **61** | 62 |
| opcodes dispatched | 189 | — |
| … opcodes that REFUSE | **94** | 189 |
| … opcodes actually handled | **95** | 189 |
| **PURE-ROUTE arms** (≤ 8 instructions, exactly 1 call, 0 conditional branches, 0 globals) | **17** | 61 |
| … opcodes they serve | **40** | 95 |
| arms with ≥ 1 direct call | 53 | 62 |
| arms reading ≥ 1 `.data` global | **17** | 62 (27.4 %) |
| arms with ≥ 1 conditional branch | 26 | 62 |
| distinct `.data` globals referenced | **17** | — |
| **verdict DEFER** | **19** | 61 |
| **verdict DECODE** | **17** | 61 |
| **verdict SELECT** | **25** | 61 |

**DECODE is 17 of 61 = 27.9 %.** The prereg registered ≤ 40 % — a HIT, and in
the predicted direction: this dispatch is *not* the mechanical recipe
`C2_MAP.md:1012` called it.

---

## 4. Context-dependence — which arms read a global `[R]`

The read plan asks each construct to state **whether the arm reads any
global**, "that is what makes an arm context-dependent". Two kinds of
absolute reference must be kept apart, and the distinction is this page's,
not the plan's:

- **`.text` tables** (`0x10b01000`…`0x10c2de00`) — `0x10b25f10`,
  `0x10b25e48`, `0x10b189cc`. Constant maps indexed by the opcode. Reading
  one is **not** context; it is a lookup an implementation can inline.
- **`.data` globals** (`0x10c2e000`…`0x10c70750`) — compiler state. **These
  are the context.**

**17 of 62 arms (27.4 %) read at least one `.data` global; 17 distinct
globals appear across 28 references.** The distribution is concentrated:
**the top 12 globals cover 82.1 %** of all references.

| global | arms | what the reference does |
|---|--:|---|
| `0x10c2e2ec` | 3 | a `u16` saved/restored around arm 20's recursion — a **counter the nested walk must not disturb** |
| `0x10c472e8` | 3 | the **compiler-options block**; `+0xcac` gates C3's operand capture, `+0xcd8 & 0x20000` gates C2's call. Same base and same `+0xcac` offset the reader uses (`WB_READER_FINDINGS` §3.2 step 5) — **shared context, not a private one** |
| `0x10c6f29c` `0x10c6f2a0` `0x10c6f2a4` `0x10c6f298` | 2,1,2,1 | the current aggregate/range record published by arms 21/22 and consumed by 20 |
| `0x10c2f060` `…64` `…6c` `…74` `…78` `…7c` | 1–2 each | the **scope/label chain** — arms 0, 11, 33, 34 |
| `0x10c3cf96` | 2 | a boolean that **switches arm 29 between two unrelated behaviours** |
| `0x10c3d72c` | 2 | the pending record arm 29 threads |
| `0x10c2e950` | 2 | the bind operand handed across from arm 28 |
| `0x10c3de20` | 1 | selects arm 20's rewrite helper |
| `0x10c2e2e0` | 1 | reset by arm 36 |

**The context an I1 implementation must model is small and enumerable** — 17
words, 12 of which carry 82 % of the traffic. That is the answer to the
question the read plan actually asked, and it is a *favourable* one.

---

## 5. The ten residue constructs

Keyed to `ROADMAP_SLICING_2026-08-21.md:162-169` (mass) and `:277-280`
(naming). **C4–C10 have no published opcode set anywhere in the record** —
`ROADMAP_SLICING` pools them into one 241,297-body row — so this section can
only key the three constructs whose opcodes the record states. That gap is a
finding, not an omission here (§8).

| construct | mass | opcode(s) | arm(s) | reads a global? | verdict | what the arm actually does |
|---|--:|---|---|---|---|---|
| **C1 off-add** | 696,164 (33.3 %) | `0x27` | **9** | **no** | DEFER | **a 16-byte trampoline** to `0x10bbfebb` (256 B). The arm makes no decision at all |
| **C2 intrinsic** | 464,172 (22.1 %) | `0x40` | **24** | **yes** — `0x10c472e8+0xcd8` | SELECT | **builds no node.** A `sub`/`je` chain over intrinsic ids that sets five flag bits on the *function* record |
| **C3 bind** | 413,626 (19.8 %) | `0x99` | **52** | **yes** — `0x10c472e8+0xcac` | SELECT | sets bind mode 1; captures the operand only when the option is on |
| | | `0x9a` | **53** | no | DECODE | sets bind mode 2. **12 bytes, no call, no branch** |
| | | `0x9b` | **54** | no | SELECT | mints a symbol; the only one of the three that constructs anything |
| **C4–C10** | 462,880 (24.1 %) | **unpublished** | — | — | — | cannot be keyed; see §8 |

**Three of the ten constructs, and 75.2 % of the residue mass, reach arms
that build no IR node in the arm itself.** C1 defers wholesale, C2 sets
function flags, C3 sets walk state in two of its three opcodes.

### 5.1 The `0x27` special case, carried `[R]`

The read plan: *"Carry the `0x27` special case
(`WB_READER_FINDINGS.md:228-234`) — a spec that omits it is wrong on the
largest single construct."* Carried, and **corrected**.

The inherited, obj-confirmed fact (board **#1595**) lives in the *type*
reader `FUN_10b3d546`, not here:

1. `0x10b3d581`: `if (opcode == 0x27) node[+6] |= 0x4000`
2. `0x10b3d5b9`: `if (opcode == 0x27) return` — the classification tail is
   skipped **for `0x27` and no other opcode**, so a `0x27` node carries **no
   `size_index` at `+0x28`** and **no composed type at `+4`**.

**What this lane can now add: arm 9 does not test bit `0x4000`, and neither
does anything else in `FUN_10bc2d7a`.** The bit is set by the reader and is
read *below* arm 9, inside `0x10bbfebb` or further down. Searched
exhaustively: the constant `0x4000` occurs **exactly once in all 5,080
bytes**, at `0x10bc3765` — `or dword ptr [esi+0x98], 0x4000`, which sits in
arm 23's *linear span* but belongs to arm 24's intrinsic chain (reached by
`jg 0x10bc373f`), and which **writes a function flag rather than testing a
node flag**. Nothing in this function reads bit `0x4000` of `node+6`.

> **Prereg P4.1 predicted arm 9 tests the `0x4000` bit — MISS.** The two
> special cases are **not** the same mechanism at two addresses. The channel
> exists (the bit is set and something must read it) but the consumer is
> below this function. Recorded as a miss, and as the reason §8 lists
> `0x10bbfebb` as the highest-value next read in the project.

P4.2's consequence **holds and is now grounded**: because the classification
tail is skipped, arm 9 *cannot* branch on type or size — it has neither. Its
being a trampoline is therefore not laziness in the arm; it is forced by the
record's shape.

---

## 6. THE SELECT/DECODE BOUNDARY — located, and it is a numeric line

This is what read R5 was funded for. `READ_PLAN` §4: *"that boundary is the
I1/I2 split the whole 15–45 eng-mo estimate rests on, and it has never been
located."*

**It is located. It is the value `0x294` in c2's single opcode numbering.**

### 6.1 The measurement `[R]`, and it is exhaustive

Every immediate operand in the interval `[0x100, 0x400]` across **all 62
arms** was collected mechanically (`dump_ilrecord.py`, reproducible). There
are **26 distinct values**:

```
02af 02b0 02b3 02b4 02b5 02b6 02c5 02c6 02d4 02dd 02de 02e4 02e5 02ee
02ef 02f0 02f4 02f5 02fb 02ff 0305 0306 0310 0311        (24 values)
0200 0400                                                 (2 values)
```

`P_ENCODE.md` §2.1 fixes the **machine** opcode space at `0x001..0x294`
(`_last` is `0x295`). Of the 26, **exactly one is ≤ `0x294`** — and `0x200`
together with `0x400` occurs only as

```
10bc36c5  or  dword ptr [esi + 0x94], 0x200
10bc36d1  or  dword ptr [esi + 0x94], 0x400
```

i.e. as **flag bitmasks in arm 24**, not as opcodes.

> **Zero machine opcodes are named anywhere in `FUN_10bc2d7a`. Every node
> opcode it mints is ≥ `0x2af`.** The lowest is `0x2af`; the encoder's
> highest is `0x294`; the two ranges do not touch, and there is a 26-value
> gap between them.

The constants are confirmed to *be* opcodes rather than arbitrary numbers by
a **round trip within the read**: arm 20 mints `0x2af`
(`mov ecx, 0x2af; call 0x10bd72b0`) and arms 14 and 56 **test for it at the
node's `+4` field** (`cmp dword ptr [esi + 4], 0x2af`). `+0x04` is the same
field `P_ENCODE.md` §9.2 calls *"opcode-or-address-mode"* on a machine
tuple. **One field, one numbering, two disjoint ranges.**

### 6.2 What that means, stated carefully

- `FUN_10bc2d7a` consumes `.ex` tokens and emits **IR tree nodes in the
  `≥ 0x295` space**. It never names a machine instruction.
- `FUN_10bf9f15` (read R2) consumes **machine tuples in the `≤ 0x294`
  space** and emits PPC words. It never sees an IR opcode.
- **Something between them lowers `≥ 0x295` to `≤ 0x294`.** That is
  instruction selection, and it is the `wb-select` lane's subject
  (`WB_SELECT_RECONCILED.md`: 13 operator×type tables, 41 dispatch arms, 18
  expansion arms) at `0x10b022cc`/`0x10b1b1f0`/`0x10bf7c59`/`0x10bfee89` —
  **a disjoint address set from `0x10bc2xxx`**, which is now explained
  rather than merely observed.

**There are three stages, not two.** The project's I1/I2 framing has one
arrow where the binary has two.

### 6.3 This REFINES board #3359 rather than confirming it

Board **#3359** (`w-ildecode`): *"there is no intermediate between the IL
token stream and the machine tuple list — by the time any tap can see a
tuple, selection has already run"*, concluding *"a general op-level IL decode
is not 'read the records'; it is 'reproduce selection'."*

- **The observational half is upheld and explained.** No tap can see the
  boundary because the intermediate is an in-memory tree in a private opcode
  range that never reaches an obj. #3359 was right about what it could see.
- **The structural half is wrong as stated.** There *is* an intermediate —
  the `≥ 0x295` node space — and the decode stage *is* separable from
  selection, by a numeric test on one field.
- **But the pessimistic conclusion survives on different grounds.** A
  general IL decode is still not "read the records", because the records are
  routed into **76 distinct callees** and the walk carries six pieces of
  cross-token state (§2.1). The cost is in the **tree builders and the walk**,
  not in entanglement with selection.

> **Prereg P2.3 predicted #3359 CONFIRMED — selection happens at or below
> this dispatch, and the boundary does NOT fall between it and the encoder.
> MISS, in the most useful possible direction.** The boundary does fall
> between them, and it is sharper than anyone expected.

### 6.4 The boundary is interleaved, not a prefix `[R]`

Prereg P2.4 asked whether I1 could be scoped as a contiguous region of the
opcode space. It cannot. DECODE arms (`0x0e`, `0x37`, `0x3a`, `0x3c`, `0x3d`,
`0x4c`, `0x56`, `0x61`, `0x64`, `0x66`, `0x67`, `0x8d`, `0x8e`, `0x8f`,
`0x90`, `0x9a`, `0xbb`) and SELECT arms alternate across the whole range with
no contiguous partition — `0x8d`/`0x8e`/`0x8f`/`0x90` are DECODE while
`0x8b` and `0x99` on either side are SELECT. **HIT.**

---

## 7. What was checked against something other than the reading

### 7.1 The structural facts, re-measured `[O]`-equivalent

Not obj-confirmable (§8), but confirmed against artifacts produced
independently of this lane:

| claim | independent check |
|---|---|
| the body ends at `0x10bc4152` | Ghidra's own extent, `FUNCS.tsv:2870`, 5,080 B — and `0x10bc2d7a + 5080 = 0x10bc4152` exactly |
| the byte table ends at `0x10bc4306` | `c2_strings.tsv:833` lists `0x10bc4307` as the next `reader.c` function, recorded before any table was parsed |
| the frame map (`node+0x28`, `node+6`, `node+8`) | `WB_READER_FINDINGS` §3.2 steps 7–8, a different lane, different function — two offsets and two predicates agree (§2) |
| `0x10b33526` is the C1001 reporter | `WB_READER_FINDINGS:176`, reached from the operand-class `0B` arm |
| `0x10b25f10` is the per-opcode attribute table | board **#1591**; this page adds bits `0x1000` and `0x8000` to the known `0x400` |
| `0x294` is the machine-opcode ceiling | `P_ENCODE.md` §2.1, read R2, independently |

### 7.2 The corpus probe `[O]`

See `WB_ILRECORD_FINDINGS.md` §5. Reported there with its population and its
limits, including the one it cannot escape (§8.1).

---

## 8. What this spec does NOT give I1

Stated so absence is not read as coverage.

1. **The tree builders — and they are where the work is.** 61 arms route into
   **76 distinct direct callees over 174 call sites**. Depth-1 only was read.
   The four largest are `0x10bc0fcc` (2,763 B), `0x10bc00a1` (2,282 B),
   `0x10bbec18` (1,174 B) and `0x10bbf8f1` (1,128 B). **19 of 61 arms are
   DEFER**, i.e. their semantics are entirely below the bound. `0x10bbfebb`
   (256 B), the C1 `off-add` builder covering 33.3 % of the residue, is the
   single highest-value unread function this read exposes.
2. **What the `≥ 0x295` opcodes MEAN.** This page proves the space exists and
   bounds it. It does not name `0x2af` or `0x2c5`. The mnemonic table
   `0x10b1b260` (stride 12) is indexed by the same opcode number and is the
   obvious next read — `dump_opcode_tables.py` already reads it, and nobody
   has looked above `0x294`.
3. **C4–C10 cannot be keyed to arms** because their opcodes are unpublished.
   `ROADMAP_SLICING` pools 241,297 bodies into one row with no opcode list.
   Keying them needs an IL-census pass, not a read.
4. **The 94 refusing opcodes' real consumer.** They are legal tokens handled
   by some other walk; which one is unread.
5. **The `[R]` bound, and it is harder here than at the encoder.**
   `P_ENCODE.md` §9.6's caveat applies verbatim *and* is compounded: the
   encoder could at least be checked word-for-word against an obj. **This
   seam's output never appears in any artifact** — it is an in-memory tree in
   a private opcode range. `READ_PLAN` §3's *"the tap cannot see this seam"*
   is not a tooling gap; it is structural.
6. **No re-pricing.** §1.3 (95 real opcodes, not 189) and §8.1 (76 callees
   below) push in opposite directions, and R2's discipline governs: a read
   produces a spec, and #1767's rule bars extrapolating a price from it. The
   findings report both numbers and decline to combine them.
