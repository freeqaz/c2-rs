# `src/system/utl/EncryptXTEA.cpp` — the price, re-derived from the obj

Lane `w-xtea`, 2026-08-09. **Every row below was read off a disassembly this
lane produced**, which is the thing board **#1792** says of its own frontier
re-survey it did not do (*"No row here was compiled or disassembled by this
lane"*).

Reference obj: `work/w-frame/refobj.sh src/system/utl/EncryptXTEA.cpp` at the
workload's own `flags.txt` (`/GR /O1 /Oi /EHsc …`); dump in
`work/w-xtea/xtea_dump.txt`. `.text` = **272 B** over five COMDAT sections,
matching the scan's `bytefrac-denominator 272`.

## 0. What is already accepted

`fnbyte-exact 1 of 5`, `bytefrac-accepted 16 of 272 = 5.9 %`. The accepted 16
bytes are the whole of `??0XTEABlockEncrypter` — `li 11,0 · std 11,0(3) ·
std 11,8(3) · blr`. **Price 0, and it is already paid.**

Note what that does to the byte-fraction ranking that put this TU at frontier
rank 2: the 5.9 % is one four-instruction constructor. It is the same shape
board **#480** found in `mmio`'s 72.7 %, one unit down — a fraction that is real
and still says nothing about the 256 bytes it is a fraction of.

## 1. The four blocked bodies

| # | function | B | cflow | calls | reader blocker (base) |
|---|---|---|---|---|---|
| 1 | `??0XTEABlockEncrypter` | 16 | straight | 0 | — **in class, byte-exact** |
| 2 | `SetKey` | 12 | straight | 0 | `expr-intrinsic-memcpy` |
| 3 | `SetNonce` | 32 | straight | 0 | `expr-op-0x27` |
| 4 | `Encipher` | 116 | loop | 0 | `expr-load-type-8882` |
| 5 | `Encrypt` | 96 | loop | **1** | `expr-op-0x27` |

## 2. `SetKey` — 12 bytes, 3 facts

```
addi 3, 3, 16          ; this + offsetof(mKey)
li   5, 16             ; the constant size
b    memcpy            ; REL24, a TAIL branch — no frame, no `bl`
```

| # | fact | already in the tree? |
|---|---|---|
| S1 | `/Oi` does **not** expand this `memcpy`; c2 emits a call. The intrinsic must be lowered **as a call**, and `docs/IL_INTRINSIC_CALL.md` records that none of the family can be lowered today | no |
| S2 | argument 2 (`uc`) is **already in `r4`** and costs zero instructions — an in-place argument elision, which needs the formal→register map to suppress a `mr` | no |
| S3 | argument 1 is a member-array address, `addi` off `this` | no |

`encode_tail_branch` (`codegen/calls.rs:32`) and the REL24 emission already
exist and are general, so the branch itself is free. **Price 3.**

## 3. `SetNonce` — 32 bytes, 4 facts

```
ld     10, 0(4)
clrldi 11, 5, 32       ; zero-extend `unsigned int shift` to 64 bits
add    10, 10, 11
std    10, 0(3)
ld     10, 8(4)
add    11, 10, 11      ; the SAME r11, reused
std    11, 8(3)
blr
```

| # | fact | already in the tree? |
|---|---|---|
| N1 | an **8-byte operand type in the value model**. `ValueClass` has `Int1u`/`Int4`/`Ptr4` and no 8-byte member; this is what `expr-load-type-8882` refuses | no |
| N2 | `clrldi` — the zero-extension of the `unsigned int` argument to 64 bits. **No 64-bit rotate/mask encoder exists anywhere in `c2-core`** (checked: `rldicl`, `rldimi`, `clrldi`, `sradi`, `srdi`, `sldi`, `extsw` — zero hits) | no |
| N3 | the extension is computed **once** and used by both elements — a CSE the port does not do | no |
| N4 | scratch allocation `r10`/`r11`, with the second `add` writing into `r11` because that is the shared value's last use | no |

`encode_ld`/`encode_std` exist and 64-bit `add` is the same instruction as
32-bit, so those cost nothing. **Price 4.**

## 4. `Encipher` — 116 bytes, ≥ 9 facts *and* a missing pass

29 instructions, a CTR loop of 4 trips, leaf.

| # | fact | already in the tree? |
|---|---|---|
| E1 | `expr-shr-mixed-sign` — the reader's terminal refusal on this body, and the ladder's real exit (§7) | no |
| E2 | the 64→32 split at entry: `slwi 10,4,0` (a **shift by zero** standing in for a move — a naive emitter writes `mr`) and `rldicl 9,4,32,32` for the high half | no |
| E3 | the 32→64 repack at exit: `clrldi 3,10,32` + `rldimi 3,9,32,0` | no |
| E4 | a CTR loop whose body **contains a memory reference** (`lwzx`). `counted_accum_loop.rs` shipped at board #1980, and board **#1981** defines that class to contain no memory reference and DECLINES the update-form pass by name | no |
| E5 | `rlwinm 8,11,2,28,29` — `(sum & 3) * 4` fused into one rotate-and-mask: a mask and an index scale in a single instruction | no |
| E6 | `rlwinm 7,11,23,28,29` — `((sum >> 11) & 3) * 4`, the same fusion folding a **shift** in as well. A second, distinct instance | no |
| E7 | `lwzx 8,8,5` — indexed load off the fused scaled index | no |
| E8 | `sum += 0x9E3779B9` emitted as `addis 11,11,-25033` + `addi 11,11,31161` **accumulating into `r11` itself**, not materialise-then-add | no |
| E9 | register assignment `r6`–`r11` across the loop, with `key` held in `r5` | no |

**And one thing that is not a fact.** The body is **software-pipelined**: the
`lwzx` for round *n* is hoisted above the adds it feeds, and the two halves of
the XTEA round are interleaved rather than emitted in source order. That is an
instruction scheduler. The port has none, and no count of "facts" prices a pass.

**Price ≥ 9, plus a scheduler.**

## 5. `Encrypt` — 96 bytes, ≥ 10 facts

```
mflr 12 ; bl __savegprlr_26 ; stwu 1,-144(1)      ; frame Class C, 6 saved GPRs
sub  26, 5, 4                                     ; (char*)out - (char*)in
addi 30, 3, -8                                    ; BIASED base pointer
li   29, 2                                        ; the trip count, NOT in CTR
…
addic. 29, 29, -1 ; … ; bf 2, .-48                ; the loop back edge
stdu 11, 8(30)                                    ; store AND advance
addi 1,1,144 ; b __restgprlr_26
```

| # | fact | already in the tree? |
|---|---|---|
| C1 | `__savegprlr_26`/`__restgprlr_26`, `FrameLayout` Class C | **yes** — `codegen/frame.rs`, board #1783, minted by `comdat.rs` |
| C2 | frame size **144** for this shape | no |
| C3 | `sub 26,5,4` — the pointer difference held live across the loop | no |
| C4 | `addi 30,3,-8` — a **biased** base so `ld 4,8(30)` reads element *i* and the store both writes and advances. An induction-variable rewrite | no |
| C5 | `stdu` — the update-form 64-bit store. Board **#1981** declines the update-form pass by name | no |
| C6 | `stdx 11,26,31` — indexed 64-bit store | no |
| C7 | a counted loop that is **not** a CTR loop, because the body contains a call — the complement of the class #1980 shipped | no |
| C8 | `addic.` setting CR0 and `bf 2` reading it | no |
| C9 | `Encipher` is **not inlined** into `Encrypt` despite `/O1` — an inline-decline fact, and the call-bearing admission has to agree with the obj's own 3 `.text` relocations | no |
| C10 | register assignment over `r26`–`r31` | no |
| C11 | the `.pdata` record | **yes** — `coff/pdata.rs` is general; it needs C2 to be right |

**Price ≥ 10.**

## 6. THE BINDING CONSTRAINT — and it is not codegen at all

The obj carries **`$M2756` and `$M2757`** (both in `Encrypt`'s section) and
**`$T2758`** in `.pdata`. Those numbers come from c2's single running
compiler-label counter, and a byte-exact obj has to reproduce them.

`crates/c2-core/src/codegen/labels.rs` invariant 2 **refuses every backward
reference by name**, because a body with a backward branch charges that counter
and the port cannot predict by how much: the module's own note records **four
distinct magnitudes over eleven cells with no rule that survives all of them**,
and two candidate rules fitted to that table and **both refuted by it**.

Board **#741/#742** made the consequence conditional — a wrong `$M` needs a
framed function in the same TU to land on, and three frontier TUs (`Sort`,
`Primes`, `IPP_basicmath_xbox`) are `label-free` and so escape it.

**`EncryptXTEA.cpp` does not escape it.** It has a framed function, that
function carries the `$M` pair, and *both* loop bodies sit in the same TU. So
even a hypothetical port that emitted all 272 bytes correctly would still miss
the obj on six bytes of symbol table.

### 6.1 The charge, MEASURED — the table would have been wrong

Not read from `docs/LABEL_COUNTER.md`. Measured by counterfactual over TUs one
body apart, at `/O1` **and** `/Ox`, in `work/w-xtea/lab/` — `A` is the real TU
made self-contained, and each cell changes `Encipher`'s control flow **and
nothing else**. `Encipher` is function 4 of 5 and the framed `Encrypt` is
function 5, so the cell is **LIVE**: a wrong charge here moves a later
function's `$M` (`w-blockir` §6 — a wrong charge on the *last* function is
inert, and this is not that).

| cell | `Encipher`'s control flow | obj B | `Encrypt`'s `$M` pair, `/O1` |
|---|---|---|---|
| **A** | `for (i < 4)` | 1950 | **2643 / 2644** |
| E | `for (i < 2)` | 1950 | 2643 / 2644 — **no move** |
| F | `while` | 1950 | 2643 / 2644 — **no move** |
| G | `for` **+ 4 extra straight-line statements** | 1966 | 2643 / 2644 — **no move** |
| **D** | no loop at all | 1882 | **2642 / 2643** |
| **H** | `do { … } while` | 1950 | **2642 / 2643** |

**Controls, all held.** `G` is the size control — 16 more bytes of object and
the counter does not move, so the `A`-vs-`D` step is not a code-volume artifact.
`E` says the charge does not key on the trip count. `F` says `while` and `for`
charge alike.

**Result at `/O1`, relative to the same TU with no loop:**

```
   for   +1        while  +1        do/while  +0
```

**`docs/LABEL_COUNTER.md` §4 records `for` +2, `while` +2, `do/while` +1.**
Every one of this lane's numbers is **one less**. Whether that is a different
baseline or a genuinely different charge for this body shape is not decidable
from six cells — but it does not need to be, because the operative conclusion is
the same either way: **taking the charge from the table would have put `$M2756`
and every later label in this TU off by one.** That is the fifth consecutive
lane to measure that table wrong, and the first to do it on this TU.

**And `/Ox` is not the same experiment.** At `/Ox` the TU packs into a single
`.text`, and `Encipher` becomes framed and mints a triple of its own in `A`,
`E`, `F` and `G` (two triples) while staying unframed in `D` and `H` (one). So
the *minting* is mode-dependent as well as the charge, and no single number
covers both profiles. `H.Ox` (2639) and `D.Ox` (2632) differ by 7 where their
`/O1` counterparts are equal.

## 7. The reader price, and #1792's `LIFTED→LIMIT`

Base ladder (`work/w-front3/ladder.py`, this tree, before any change):

```
net=17  stepped=16  EXIT: expr-chain-noform-0x00 (noform)   RENAME: op:00
```

That reproduces board **#1465** and **#1430** to the unit. **The exit is an
artifact, and not the one #1465 named.** `0x00` is not an unpinned opcode — it
is byte 5 of the 8-byte payload of `nonce & 0xFFFFFFFF`, and the walk is
standing inside a literal it mis-measured (`33 88 82 23 80 ff ff ff ff 00 00 00
00 0B`). #1465's *conclusion* is right — the `op:00` rung buys zero decode
distance — and its *mechanism* is wrong in the direction that matters: no entry
in a width table can repair a desync, so that rung was unfixable-by-pinning
rather than merely unpinned.

After the repair (commit `79cabd78`):

```
net=17  stepped=17  EXIT: expr-shr-mixed-sign:mid          RENAME: none
```

**So the answer to #1792 for this TU is: it WAS the instrument's limit, and it
is not any more.** Three of the four blocked bodies now walk to the function
tail (`expr-chain-noform-0x4F`, which `ladder.py` treats as a terminal because
`4F 12` *is* the tail); the fourth exits on `expr-shr-mixed-sign`, a real,
named, unlifted refusal in the port. The published `net` is unchanged at 17 —
which is the point of quoting the **exit key** and not the depth (**#1103**):
the number stood still while the thing it is a proxy for changed completely.

Note also that the board row and the rung disagree about this TU. Board #1792's
summary sweeps `EncryptXTEA` into a five-TU `LIFTED→LIMIT` list;
`rungs/2026-08-08-w-xlr.md` §10 names **four** TUs as `LIFTED→LIMIT` and prices
`EncryptXTEA` separately at **≥ 26**. The board row was right that the ladder
was instrument-bounded, and §10 was right that ≥ 26 is a genuine lower bound.

## 8. The total

| function | facts | notes |
|---|---|---|
| `??0XTEABlockEncrypter` | **0** | already byte-exact |
| `SetKey` | 3 | |
| `SetNonce` | 4 | |
| `Encipher` | ≥ 9 | **plus an instruction scheduler** |
| `Encrypt` | ≥ 10 | |
| the TU | **+1** | the `$M` label charge — §6, and it is the binding one |
| **total** | **≥ 27** | |

Counted by the rule *"what varies between these two? if nothing, it is one
refusal."* **No discount factor is applied anywhere**, per the standing
instruction that every discount applied in this project has been wrong.

**≥ 27 against `w-xlr` §10's published `≥ 26`** — an independent re-derivation
landing one above a number produced without compiling the TU. The agreement is
worth less than it looks (both are lower bounds and neither is tight), but it is
the first time that figure has been checked against the object code.

## 9. Verdict — DECLINED, at a named count

The standing clause declines a frontier TU at **≥ 4** independent facts. This
one is at **≥ 27**, needs a pass the port does not have, and is gated behind the
project-wide label-counter problem that `labels.rs` refuses on principle and
that this lane has now measured to be **+1 for this TU at `/O1`, a value the
published table does not give**.

Converting it would have meant admitting a backward reference with a guessed
label charge — which is the exact defect `labels.rs` invariant 2, board #285 and
`LABEL_COUNTER.md` §4.1 all exist to prevent, and it would have shown up as six
wrong bytes in a symbol table inside an obj that still links.
