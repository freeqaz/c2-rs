# P_ENCODE — the instruction encoder `FUN_10bf9f15`

> **PROVENANCE — DISASSEMBLY-DERIVED.** Everything here was obtained by
> statically disassembling `c2.dll`. See [`../DISCLOSURE.md`](../DISCLOSURE.md):
> nothing on this page may enter `crates/` without a row naming the address.
> Legend: **`[R]`** read from the disassembly and *not* obj-checked — **a
> hypothesis**; **`[O]`** reproduced against real c2 output; **`[I]`** inferred.
> `[R]` says *"the instructions were read correctly"*, never *"this is what c2
> does"* ([`README.md:49`](README.md); `C2_MAP_METHOD.md` §7).

**Image.** `compilers/X360/16.00.11886.00/c2.dll`, sha256
`c80981c015166effecc71ad8112d5577a065b2300891dfdb02b9c13787a66258`, verified
before any address below was read. Every address is an absolute VA in this
exact image.

**Provenance of this page.** Read **R2** of
[`../READ_PLAN_2026-08-21.md`](../READ_PLAN_2026-08-21.md) §3, funded by the
owner 2026-08-22 (`docs/DECISIONS_2026-08-22.md` decision 1), executed by lane
`w-read-r2` under [`../WB_ENCODE_PREREG.md`](../WB_ENCODE_PREREG.md). The
prereg's scoring — including its misses — is in
[`../WB_ENCODE_FINDINGS.md`](../WB_ENCODE_FINDINGS.md); **read that before
trusting a number here.** Two arms (forms 49 and 55) and both table VAs are
`w-ildecode`'s prior art, board **#3358** /
[`../WB_MIDDLE_INTERFACES.md`](../WB_MIDDLE_INTERFACES.md) §5; this lane
re-derived them independently and they agree.

**Coverage of this page, stated first.** **79 of the 79 distinct arms read**,
covering **660 of 660** machine opcodes. The last six — all single-opcode
VMX128 arms of the family §5.7 describes — were read after the first draft;
`WB_ENCODE_FINDINGS.md` §6.1 records the intermediate 73/79 rather than
overwriting it, because the six were read *knowing* the family's idiom and
that is a weaker kind of reading than the first six of it.

**Coverage is not confidence.** Every rule in §5 is `[R]` on the question
*which operand did the arm read*, and §8.2's obj test cannot see that axis —
§9(6) is explicit about it. 79/79 means every arm body was followed; it does
not mean every rule is right.

---

## 1. The function

```
int __fastcall encode(Tuple *t /*ecx*/, u32 *out /*edx*/,
                      u32 pc /*[ebp+0x8]*/, void *sink /*[ebp+0xc]*/);
```
`push ebp / sub esp,0xc / ret 0x8` — two register arguments and two stack
arguments; **the return value is always 4** (`push 4 / pop eax` at
`0x10bfae1e`), the byte length of the one word written. `[R]`

* `t` — the final machine tuple. `t+0x04` = opcode, `t+0x28` = the **source**
  operand list, `t+0x2c` = the **destination** operand list, `t+0x0a` = a
  packed word whose **low 5 bits are the condition code** (§5.2).
* `out` — where the single big-endian-on-target word is stored, once, at
  `0x10bfae20`. **There is exactly one store and exactly one `ret` in the
  whole 3,861-byte body**, so no arm can emit two words or zero words. `[R]`
* `pc` — the address of the instruction being encoded, used only by the
  branch arms to form a displacement (§5.3).
* `sink` — the **relocation sink**. When it is `NULL` every relocation-emitting
  path is skipped and the word alone is produced (§6). This argument is what
  makes §6 exist at all, and `WB_MIDDLE_INTERFACES.md` §5.6's *"relocations —
  0 cells"* is superseded by it.

**Operand lists are singly-linked at offset 0.** The prologue reads
`esi = t[0x28]` and `ecx = [esi]` (guarded by `test esi,esi`), so `esi` is the
first source operand and `ecx` the second; a third is `[[esi]]` and arms that
need one walk it (`0x10bfa4ac`, `0x10bf9fdb`). `eax = t[0x2c]` is the
destination operand. `[R]`

### 1.1 The dispatch

```
10bf9f3c  ebx = base_word[opcode]          ; [opcode*4 + 0x10c3a578]
10bf9f43  edx = form[opcode]               ; [opcode*4 + 0x10c39b18]
10bf9f4a  [ebp-0x8] = edx                  ; the FORM is kept — two arms read it back
10bf9f4d  edx--
10bf9f4e  cmp edx,0x6e
10bf9f51  ja  0x10bfae1b                   ; DEFAULT — store the base word unchanged
10bf9f57  jmp [edx*4 + 0x10bfae2d]         ; 111 entries, 79 distinct targets
```
`[O]` on the two table VAs (#3358); `[R]` on the rest. The arm index is
`form − 1`, so the dispatched range is `form ∈ 1..=111` and **`form = 0`,
`112` and `113` fall to the default** — `0 − 1` is `0xFFFFFFFF`, which is
`> 0x6e` unsigned.

---

## 2. The base-word table — `0x10c3a578`, stride 4

One 32-bit PPC word per opcode, **with every operand field zero**. Dumped in
full at [`ENCODE_OPCODES.txt`](ENCODE_OPCODES.txt) (regenerate with
`python3 docs/whitebox/scripts/dump_opcode_tables.py <c2.dll> --encode`).

### 2.1 The extent, and the trap at the end of it `[O]`

`_last` is opcode **`0x295`**, so the machine opcode space is `0x001..0x294`
and **both tables are exactly that long**. Past it:

| op | mnemonic-table name | base word |
|---:|---|---|
| `0x295` | `_last` | `00000000` |
| `0x296` | `illegal` | `00000000` |
| `0x298` | `_first` | `30081000` — **not a base word** |
| `0x2c0` | *"cmpwi"* | `02000008` — **filler, not a base word** |
| `0x2cc` | *"mtctr"* | `02020202` — **filler** |

The names past `0x297` are `WB_MIDDLE_INTERFACES.md` §2.2's trap: they are the
**second** (stride-16, extended-mnemonic) table's strings read through the
stride-12 walk. This lane walked into it and the **base-word table detected
it** — `02020202` is not a PPC encoding of anything. That is a useful property
in its own right: *the base-word table is a cheap validity check on any claim
that opcode N is a machine opcode.* Neither `cmpwi` nor `mtctr` is a c2 machine
opcode; c2 emits `cmpi` (`0x02e`) and `mtspr` (`0x0f8`).

### 2.2 The five opcodes whose base word is zero

`rlandi` (`0x26e`), `rlandi.` (`0x26f`), `deadtmp` (`0x291`) — all three also
route to the default, so they encode as `00000000`; and **`emit` (`0x290`,
form 18)** and **`DCD` (`0x292`, form 65)**, whose arms replace the word
entirely from the operand (§5.6). A zero base word is therefore **not** a
synonym for "not encodable" — see §3.2.

---

## 3. The encode-form table — `0x10c39b18`, stride 4

One small integer per opcode. Over `0x001..0x294`: **104 distinct values**,
range `0..113`, top value `78` covering **104 opcodes** — reproducing the
survey's measurement exactly (`READ_PLAN` §3 banner).

### 3.1 Distribution

| form | opcodes | arm | what it is |
|---:|---:|---|---|
| 78 | 104 | `10bf9f91` | VMX three-register |
| 49 | 65 | `10bfa456` | `RT,RA,RB` — the classic three-register X/XO form |
| 92 | 35 | `10bfa9f0` | VMX128 three-register |
| 39 | 28 | `10bfa53b` | X-form logical — **destination is `RA`** |
| 25 | 28 | `10bfa4df` | `RT,RB` (`fmr`, `frsp`, `fabs`) |

The full inversion — one row per arm with the forms and the opcode count it
serves — is [`ENCODE_ARMS.txt`](ENCODE_ARMS.txt).

### 3.2 The default arm is an ENCODING, not a refusal — and the refusal is elsewhere

> **This corrects `READ_PLAN_2026-08-21.md` §4's spec-shape**, which asked for
> *"rows whose form hits the `edx > 0x6e` default marked **not encodable by
> this path**"*. That reading is wrong in both directions.

`0x10bfae1b` is the store-and-return tail. Reaching it with `ebx` untouched
emits **the base word with no operand fields**, which is the *correct and
complete* encoding for an operand-free instruction. 24 opcodes take it:
21 through the `ja` default (`isync`, `sync`, `eieio`, `sc`, `rfi`, `rfid`,
`tlbia`, `tlbsync`, `slbia`, `mfsr`, `mfsrin`, `mtsr`, `mtsrin`, `slbie`,
`slbmfee`, `slbmfev`, `slbmte`, `tlbie`, `rlandi`, `rlandi.`, `deadtmp`) and
3 through the jump table (`dss`, `dst`, `dstst`, forms 35/52/53/59/72/73/87/
89/95/96/98/100 — 12 forms, the busiest single target).

**The real refusal is `0x10bfa81d`**, which loads `edx = 0x3d9` and jumps to
the shared ICE call `mov ecx,0x10b19730 / call 0x10b33526` at `0x10bfa531` —
a `(file, line)` internal-error report, line **985**. Seven forms (8, 9, 10,
11, 13, 48, 60) and **19 opcodes** land there: the eight `cr*` logical ops,
`mcrf`, `mcrfs`, `mcrxr`, `mtfsb0/.`, `mtfsb1/.`, `mtfsfi/.`, `lswi`, `stswi`.
Two more ICE sites exist inside arms — line **702** (`0x2be`) in form 51 when a
`D`-form arithmetic operand is neither a constant nor an `addis` relocation
(§5.4), and line **1025** (`0x401`) in form 65. `[R]`

So the honest three-way split is: **encoded with no fields** (24), **encoded
with fields** (617), **ICE** (19). Of the 660 machine opcodes, none is silently
mis-encoded by the default.

---

## 4. The arm jump table — `0x10bfae2d`, 111 entries, 79 distinct targets

All 79 targets lie inside the body. Arms **chain**: 34 of them end by jumping
into another arm's tail rather than to the store, so the "arm body" is a DAG
and its 3.8 KB is not 79 independent blocks. The shared tails are worth naming
because a reimplementation wants them as functions:

| tail | code | meaning |
|---|---|---|
| `0x10bfa45f` | `eax=[eax+0x28]; eax=(eax<<5)\|edx; eax<<=5; ecx=[ecx+0x1c]; eax\|=[ecx+0x28]; eax<<=11` | finish a three-5-bit-field bundle at bits 6..20 |
| `0x10bfa470` | `eax <<= 11` | place a bundle at bits 6..20 |
| `0x10bfa348` | `eax \|= ecx` then join | fold a stray low bit in |
| `0x10bfae19` | `or ebx,eax` | compose and store |
| `0x10bfa3b3` | `or ebx,ecx` | compose and store (the `ecx` convention) |
| `0x10bfa564` | `or ebx,edx` | compose and store (the `edx` convention) |
| `0x10bfae1b` | `*out = ebx; return 4` | **the single exit** |

> **PREREG P3.3 predicted "exactly two join points, no third" and that is a
> MISS**: the final `or` into `ebx` happens at **six** distinct instructions
> (`0x10bfa348`, `0x10bfa371`, `0x10bfa3b3`, `0x10bfa3d4`, `0x10bfa564`,
> `0x10bfa8de`, `0x10bfae19`). What survives is the stronger invariant the
> prediction was reaching for: **one store, one exit, one word.**

**One arm contains a second jump table.** Form 37 (`nop` and friends,
`0x10bfa1ad`, 182 B — the largest arm) dispatches again on the opcode through
**`0x10bfafe9`, 9 entries**, `opcode + 0xfffffd89` bounded at `8`. So the
encoder has **two** jump tables, not one. `[R]`

> **⛔ AMENDED BESIDE — lane `w-r8idiom`, 2026-08-24, board #3482. THE NINE
> ENTRIES, DECODED.** `[R]`. Each arm ORs its own register fields into the base
> word, so the base-word table alone does **not** give what a pseudo-nop emits:
>
> | opcode | mnemonic | arm | emits | = |
> |---|---|---|---|---|
> | `0x277` | `nopmthigh` | `0x10bfa203` | `0x7c631b78` | `or r3,r3,r3` |
> | `0x278` | `nopmtmed` | `0x10bfa20e` | `0x7c421378` | `or r2,r2,r2` |
> | `0x279` | `nopmtlow` | `0x10bfa219` | `0x7c210b78` | `or r1,r1,r1` |
> | `0x27a` | `nopstall` | `0x10bfa1db` | *computed* | `or r28..r31` (below) |
> | `0x27b` | `nopalign` | join `0x10bfae1b` | `0x7c000378` | `or r0,r0,r0` |
> | `0x27c` | `nopvmxperm` | `0x10bfa224` | `0x181b021a` | VMX; falls back to `0x60000000` when `DAT_10c2e978 == 0` — §7's escape flag |
> | `0x27d` | `nopvmxsimp` | `0x10bfa242` | `0x11ef7c84` | `vor v15,v15,v15` |
> | `0x27e` | `nopcapenter` | `0x10bfa24d` | `0x7dad6b78` | `or r13,r13,r13` |
> | `0x27f` | `nopcapexit` | `0x10bfa258` | `0x7dce7378` | `or r14,r14,r14` |
>
> **`nopstall` is the only data-driven one.** Arm `0x10bfa1db` reads
> `operand[0x18]`, caps it at `0xf` (out of range → `0x1f`), indexes the
> **16-byte table `0x10c37dcc`** = `28 ×10, 29, 29, 30, 30, 31, 31`, and
> splices the value into all three register fields
> (`x<<5 | x`, `<<5 | x`, `<<11`) — the Xenon delay-nop encodings, selected by
> requested stall in cycles. **`0x10c37dcc` is recorded here for the first
> time.**
>
> The reason this decode was worth doing: it **excludes** the family. Not one
> of the nine is `or r8,r8,r8`, so the `mr r8,r8` in
> [`WB_R8IDIOM_FINDINGS.md`](../WB_R8IDIOM_FINDINGS.md) is not a c2 nop — it is
> an `emit` (`0x290`) of a baked literal. Tool:
> `dump_movearms.py --nops`.

---

## 5. The arm rules

Notation: `S` is the destination operand (`t+0x2c`), `D0` the first source
(`t+0x28`), `D1 = [D0]`, `D2 = [[D0]]`. **`reg(x)` is `[[x+0x1c]+0x28]`** —
the two-hop path to a hardware register number, which is `w-restim`'s
`sym+0x08→+0x1c` **minus one** (`WB_MIDDLE_INTERFACES.md` §5.3, `[O]`).
`imm(x)` is `[x+0x18]`. `kind(x)` is the byte `[x+0x8]`.

**PREREG P3.2 is a HIT with zero exceptions**: every arm that reads a register
uses `reg()` and no other path.

### 5.1 The workhorse integer forms `[R]`, and see §8 for the obj check

| form | arm | opcodes | rule |
|---:|---|---:|---|
| 49, 22 | `10bfa456` | 77 | `RT=reg(S)`, `RA=reg(D0)`, `RB=reg(D1)`; `w = base \| RT<<21 \| RA<<16 \| RB<<11` |
| 39 | `10bfa53b` | 28 | **`RA=reg(S)` — the destination is the `RA` field**; `RS=reg(D0)`, `RB=reg(D1)`; `w = base \| RS<<21 \| RA<<16 \| RB<<11` |
| 47 | `10bfa4c8` | 20 | `w = base \| reg(S)<<21 \| reg(D0)<<16` (`neg`, `addze`, `subfme`) |
| 38 | `10bfa587` | 9 | `w = base \| reg(D0)<<21 \| reg(S)<<16` (`extsb`, `cntlzw` — source in `RS`, dest in `RA`) |
| 36 | `10bfa549` | 4 | `w = base \| reg(D0)<<21 \| reg(S)<<16 \| reg(D0)<<11` — the `or rA,rS,rS` idiom behind `mr`/`not` |
| 25 | `10bfa4df` | 28 | `w = base \| reg(S)<<21 \| reg(D0)<<11` (`fmr`, `frsp`, `fabs`) |
| 51 | `10bfa4ed` | 6 | `RT<<21 \| RA<<16`, then §5.4's three-way immediate |
| 43 | `10bfa56b` | 6 | `w = base \| reg(D0)<<21 \| reg(S)<<16 \| imm(D1)` — logical immediate. **`imm` is ORed unmasked** |
| 33 | `10bfa5a0` | 2 | `w = base \| reg(S)<<21 \| (u16)imm(D0)` — `li`/`lis` |
| 41 | `10bfa685` | 2 | `srawi`: `reg(D0)<<21 \| reg(S)<<16 \| imm(D1)<<11` |
| 42 | `10bfa6dc` | 2 | `rlwinm`: `RS<<21 \| RA<<16 \| SH<<11 \| MB<<6 \| ME<<1`, each of `SH`,`MB`,`ME` a **byte** at `imm()` of successive operands |
| 56 | `10bfa719` | 2 | `rlwimi`, same shape |
| 40 | `10bfa6a1` | 2 | `rlwnm`, same shape with `RB` in place of `SH` |
| 31 | `10bfa741` | 2 | `lcarry`: `reg(S)<<21 \| reg(D0)<<16 \| reg(D0)<<11` |

**Form 39's field order is the single most safety-critical fact on this
page.** `crates/c2-core/src/codegen/encode.rs:106`'s `encode_logical_x`
already carries it, derived black-box from captures, with a comment saying
that getting it wrong *"produces a valid `and` with the destination and the
left operand exchanged — bytes that assemble, disassemble and fuzz-match, and
compute the wrong thing."* c2's own arm says the same thing.

### 5.2 Condition codes — the helper `0x10bf983a` `[R]`

Called from three arms (forms 4, 5, 1) as
`cl = t[0x0a] & 0x1f; edx = D1; call 0x10bf983a`, returning
`BO<<21 | BI<<16` where `BI = 4*crf(edx) + bit`. The switch:

| `cc` | `bit` | `BO` | reading |
|---:|---:|---:|---|
| 0 | 0 | 0 | none — the caller supplies `BO` itself |
| 1 | 2 | 12 | `EQ` |
| 2 | 2 | 4 | `NE` |
| 3 | 0 | 12 | `LT` |
| 4 | 1 | 12 | `GT` |
| 5 | 1 | 4 | `LE` |
| 6 | 0 | 4 | `GE` |
| `0x0b` | 3 | 12 | `SO` / unordered |
| `0x0c` | 3 | 4 | `NS` / ordered |
| `0x0f` | 0 | 12 | `LT` (second encoding) |
| `0x10` | 0 | 4 | `GE` (second encoding) |
| `0x11` | 2 | 12 | `EQ` (second encoding) |
| `0x12` | 2 | 4 | `NE` (second encoding) |
| others (`7..0x0a`, `0x0d`, `0x0e`, `≥0x13`) | 0 | 0 | default |

The `crf` comes from `[[edx+0x1c]+0x28]`, i.e. `reg()` applied to the CR
operand. **This is `cr_bi()` + `BO_TRUE`/`BO_FALSE` in
`crates/c2-core/src/codegen/encode.rs:1056`, reached from the other side.**

> **A refinement to `WB_MIDDLE_INTERFACES.md` §3.3**, which reports `t+0xa` as
> *"a SIZE, not a condition code"* `[O]`. Both are true: `t+0xa` is a packed
> **16-bit** field. Three arms read it three different ways — `&0x1f` as the
> condition code (`0x10bfa2b0`, `0x10bfa2d1`, `0x10bfa326`), as a whole word
> compared against `0x1008` (`0x10bfa381`, `0x10bfa3e4`), and on the *operand*
> `&0xfff` compared against `8` (`0x10bfa393`). §3.3's measurement is not
> contradicted; its wording is too strong.

### 5.3 Branches `[R]`

| form | arm | rule |
|---:|---|---|
| 3, 55 | `10bfa2a5` | `or ebx,0x2800000` — `BO = 20`, no operand read at all. `blr`/`ret`/`bctr`/`bctrl`/`bid` |
| 4 | `10bfa2b0` | `bclr`/`bcctr`: `base \| helper(cc, D1)` — `BO`,`BI` only, **no displacement** |
| 5 | `10bfa326` | `bc`: `base \| helper(cc,D1) \| ((([D0+0x18][0x18] − pc) >> 2) & 0x3fff) << 2` |
| 1 | `10bfa2c2` | `bdnz`/`bdz`: if `D1` exists and `D1[0xa] == 0x9000`, take the `cc` helper and then `xor` `0x800000` (`bdnz`) or `0xc00000` (`bdz`); otherwise `or` `0x2000000` / `0x2400000`. Then the same 14-bit displacement |
| 6 | `10bfa263` | `b`: if `[D0+0x18][0x30] == 3` it is a **local label** → fall into form 2's 24-bit self-relative displacement; otherwise fall into form 7 |
| 2 | `10bfa26c` | `bgip`: `LI = (((target − pc) >> 2) & 0xffffff) << 2` |
| 7 | `10bfa285` | `bl`: **no `sink` ⇒ base word alone**; with a sink, emit `REL24` (§6) and set `LI = (−pc) & 0x3fffffe` |

`bl`'s `−pc` is the COFF convention: the field holds the displacement from the
instruction to the *section start*, and the linker adds the symbol.

### 5.4 Immediates and the operand kind byte `[R]`

`kind(x) = [x+0x8]` is the operand class. Three values are load-bearing:

* **`7` — an immediate constant.** Form 51 (`0x10bfa503`) takes
  `(u16)[D1+0x18]`; `mfspr`/`mtspr` (`0x10bfa76a`, `0x10bfa7a3`) take the SPR
  number the same way; `DCD` (`0x10bfa84e`) takes the whole word.
* **`2` — a static address.** The D-form load/store composers
  (`0x10bf9e55`/`0x10bf9eb5`) take `RA = 0x1f` when
  `(u16)DAT_10c6fd9c == 0x20` and `[memop+0x18][0x4] != 6`, else `RA = 1`, and
  the displacement from `[memop+0x18][0x24]`.
* **`4`** — distinguishes two symbol shapes inside the relocation helpers
  `0x10bf96ea` / `0x10bf9721`.

Form 51's three-way is the clearest statement of the whole design:

```
      RT<<21 | RA<<16 already in ebx
      if kind(D1) == 7        -> | (u16)imm(D1)             ; a constant
      elif opcode == 0x00e    -> if sink: REFHI + PAIR       ; addis sym@ha
                                 else:    base word alone
      else                    -> ICE(line 702)
```

### 5.5 `D`-form memory — `0x10bf9e55` (load) / `0x10bf9eb5` (store) `[R]`

Forms 21/45/46 (loads, 16 opcodes) and 27/58/71 (stores, 13 opcodes) do
nothing but `edx = sink; ecx = t; call <composer>; or ebx,eax`. The two
composers are mirror images — the load takes its memory operand from `t+0x28`
and its register from `t+0x2c`, the store the other way round:

```
memop = t[0x28]           (load)   /  t[0x2c]  (store)
if kind(memop) == 2:      RA, disp from the static-address path above
else:                     RA = reg(memop[0x2c]); disp = memop[0x28]
                          if sink: call 0x10bf9808   ; REFLO + PAIR, if a symbol
reg  = reg(t[0x2c])       (load)   /  reg(t[0x28])  (store)
return (reg<<5 | RA)<<16 | (u16)disp
```

> **A tooling trap at this exact address, recorded because it costs a reader
> ten minutes.** `objdump -d -M intel` (`C2_MAP_METHOD.md` §4's independent
> disassembly source) **mis-syncs at `0x10bf9e55`** and prints
> `dec ebx / sub BYTE PTR [eax+0x56020879],al` for the load composer's
> prologue. The raw bytes are
> `53 8b d9 8b 4b 28 80 79 08 02 56 57` = `push ebx; mov ebx,ecx;
> mov ecx,[ebx+0x28]; cmp BYTE PTR [ecx+0x8],0x2; push esi; push edi`, and the
> store composer at `0x10bf9eb5` is the same twelve bytes with `2c` for `28` —
> which is how the mirror-image reading above was confirmed. **Read the bytes
> when a linear disassembler's output stops making sense**; the two sources
> disagreeing is the point of having two.

Indexed (`X`-form) memory is forms 26/50 (`0x10bf9788`) and 28/61
(`0x10bf97c8`), same mirror-image pair: a memory operand whose own
`[+0x4] == 0x29f` is base-only (index register 0), otherwise the index comes
from `[memop+0x2c]` and the base from `[memop+0x30]`.

### 5.6 The two arms that are not encodings `[R]`

* **`emit` (opcode `0x290`, form 18, `0x10bfa846`)** — `ebx = imm(D0)`. The
  operand *is* the instruction word. This is why its base word is `0`.
* **`DCD` (opcode `0x292`, form 65, `0x10bfa84e`)** — `kind(D0)==7` ⇒ same as
  `emit`; else with a sink, resolve the symbol via `0x10bd470b` and emit a
  type-**2** (`ADDR32`) relocation; else ICE line 1025.

### 5.7 Split fields `[R]`

* **`mfspr`/`mtspr` (forms 54, 62)** — `SPR` is written **low half first**:
  `w = base | R<<21 | (spr & 0x1f)<<16 | (spr >> 5)<<11`. This is exactly the
  hazard `encode.rs:228`'s `encode_mtctr` documents (*"writing `9 << 11`
  produces a legal-looking `mtspr` naming SPR 288"*), and c2 does the split in
  the arm rather than in the base word — which is why `mtspr`'s base word is
  `7c0003a6` with the field zero (§8.1's residual 5).
* **`mftb` (form 108)** — same shape, with `TBR` chosen by a single
  comparison: `imm(D0) == 0x10c ? 0x188 : 0x1a8`.
* **`mtcrf` (form 17)** — `w = base | reg(D1)<<21 | CRM<<12`.
* **VMX128 (forms 86..107)** — the 7-bit vector register numbers are scattered:
  the low 5 bits go in the classic field and the high 2 are placed by
  `(r >> 2) & 0x18`, `r & 0x60`, `r & 0x40`, `r & 0x20` at various shifts.
  Six arms of this family are the unread residue (§7).

---

## 6. Relocations — the encoder DOES drive them

> **This section supersedes `WB_MIDDLE_INTERFACES.md` §5.6** (*"Relocations …
> this lane observed the relocation/label half of the emit seam **not at
> all** … 0 cells"*) and **refutes this lane's own PREREG P4.3**, which
> predicted the encoder emits none. It also narrows `READ_PLAN` §4's
> spec-shape item 5 (*"the negative section: relocations are not in this
> spec"*): they are not the *whole* seam, but the **request** for every
> `.text` relocation is issued from inside `FUN_10bf9f15`.

Every relocation path is guarded by `cmp [ebp+0xc],0` — with a `NULL` sink the
encoder still produces the correct word and simply asks for nothing. The sink
is reached through one indirect vector, `ds:0x10c433f8`, called as
`push <type>; edx = symbol; ecx = 0; call [0x10c433f8]`. The type codes are
`IMAGE_REL_PPC_*` and they match the architecture exactly: `[R]`

| helper | reaching paths | emits | reached from |
|---|---:|---|---|
| `0x10bf976d` | 1 | **6** = `REL24`, then stashes the symbol in `DAT_10c6fd5c` | form 7 (`bl`) |
| `0x10bf96d9` | 1 | **0x0d** = `IFGLUE`, on the symbol `0x10bf976d` stashed | form 37, only when the opcode is `0x280` (`rsttoc`) — the *"nop after the call"* glue slot |
| `0x10bf96ea` | 2 | **0x10** = `REFHI`, then `0x12` = `PAIR` | form 51 when the opcode is `addis`; form 30 (`lau`) |
| `0x10bf9721` | 1 | **0x11** = `REFLO`, then `0x12` = `PAIR` | form 29 (`lal`) |
| `0x10bf9808` | 2 | `REFLO` + `PAIR`, if the memory operand carries a symbol | both `D`-form memory composers |
| `0x10bf9758` | 1 | **0x0f** = `SECREL16` | form 34 (`loffs`) |
| `0x10bd470b` + inline | 1 | **2** = `ADDR32` | form 65 (`DCD`) |

**"Reaching paths" is not "call sites"** — form 30 (`lau`) reaches
`0x10bf96ea` by `jmp 0x10bfa522` into form 51's tail rather than by its own
`call`, and `0x10bf9808`'s two sites are inside the `D`-form composers, not in
the encoder body. Exact, mechanically counted: the body
`0x10bf9f15..0x10bfae2a` contains **15 direct call sites over 13 distinct
targets, plus 1 indirect** (`0x10bfa86f`, `ds:0x10c433f8`).

`0x10b2930b`, called between the two halves of every `REFHI`/`REFLO` pair, is
inside `coffemit.c`'s recovered range (`0x10b290dc..0x10b2b0dd`,
[`../c2_tus.tsv`](../c2_tus.tsv)) — the pair record's addend.

**What is still not in this spec:** the sink itself
(`0x10c433f8`'s target), how a relocation record reaches the section's reloc
array, and label *placement*. A complete encoder is still not a complete emit
seam — the claim that changes is that the seam **starts here** and is
enumerable, not that it is finished.

---

## 7. The escape flag `DAT_10c2e978` — VMX128 `[R]`

12 references inside the encoder, **all reads**; the image contains no direct
store, and the one non-read cross-reference is
`mov DWORD PTR ds:0x10c47134, 0x10c2e978` at `0x10c2a00a` — the variable's
*address* being registered in a table, the shape of a command-line option.

Its effect is a single, uniform guard in the 20 vector arms:

```
if (DAT_10c2e978 != 0) {
    if (any register operand >= 0x20) {
        opcode = FUN_10bf98ec(t, opcode);   /* substitute the VMX128 opcode */
        goto restart;                        /* re-dispatch on the new opcode */
    }
}
```

`FUN_10bf98ec` (1,385 B) is the substitution table: **72 of the 84 `*128`
opcodes appear in it as immediates** — `lvewx128`, `stvx128`, `vaddfp128`,
`vcmpeqfp128`, … So the flag is *"VMX128 registers 32..127 are available"*,
and the encoder's response to a high register is not a wider field but a
**different opcode**, re-entered at `0x10bf9f26` with the restored operand
lists. **PREREG P4.1 named VMX128 as one of two candidates and is a HIT.**

### 7.1 The VMX128 register split, worked once `[R]`

A VMX128 register is 7 bits and no field holds 7 contiguous bits, so every arm
of the family scatters it the same way: `r & 0x1f` goes in the classic 5-bit
field and bits 5–6 are placed by fixed masks in the instruction's low bits.
Form 86 (`vmr128`, `0x10bfa902`) in full — `VD = reg(S)`, `VB = reg(D0)`:

```
w = base | (VD & 0x1f) << 21 | (VB & 0x1f) << 16 | (VB & 0x1f) << 11
         | (VB & 0x40) << 4                     ; VB bit 6
         | ((VB >> 5) & 0x3)                    ; VB bits 5..6, low
         | ((VD >> 3) & 0xc)                    ; VD bits 5..6, low
         | (VB & 0x20)                          ; VB bit 5
```

The other nineteen arms of the family differ only in how many registers they
carry and which of the four scatter masks each uses; the last six read —
`0x10bfaa42` (form 101, `vcfpsxws128`/`vcfpuxws128`), `0x10bfaadd` (103,
`vperm128`), `0x10bfabf0` (97, `vrlimi128`), `0x10bfac63` (105, `vsldoi128`),
`0x10bfaccd` (106, `vupkd3d128`), `0x10bfacee` (107, `vpkd3d128`) — are all
this shape with an extra immediate from `imm()` of a third or fourth operand.

> **A hazard for a reimplementation, and the reason three arms score as
> impure in `WB_ENCODE_FINDINGS.md` §1's P3.1.** Three of these arms —
> `0x10bfaafc` (`vperm128`), `0x10bfac7f` (`vsldoi128`), `0x10bfad09`
> (`vpkd3d128`) — **write to `[ebp+0xc]`**, the *relocation-sink argument
> slot*, using it as a spare register. It is safe in c2 only because no VMX128
> opcode reaches §6, so the sink is dead by then. A port that keeps the sink
> live past the arm and copies this structure would corrupt it.

---

## 8. What was checked against real output

### 8.1 The base-word table against the port's black-box re-derivation `[O]`

`crates/c2-core/src/codegen/encode.rs` accumulated 89 PPC words one captured
obj at a time, never looking at `c2.dll`. Evaluating each `encode_*` with
**every operand zero** and comparing against `base_word[op]` for the matching
mnemonic: **82 of 89 identical (92.1 %)**, **0 disagreements in a primary
opcode or an extended opcode**, and all 7 residuals are a field the port bakes
that c2 contributes from the arm:

| port | port word | c2 base | xor | the arm that supplies it |
|---|---|---|---|---|
| `encode_blr` | `4e800020` | `4c000020` | `02800000` | form 55 `or ebx,0x2800000` (`BO=20`) |
| `encode_bctrl` | `4e800421` | `4c000421` | `02800000` | same |
| `encode_bdnz` | `42000000` | `40000000` | `02000000` | form 1 `or ebx,0x2000000` |
| `encode_mtctr` | `7c0903a6` | `7c0003a6` | `00090000` | form 62's split `SPR` (§5.7) |
| `encode_srwi31` | `54000ffe` | `54000000` | `00000ffe` | form 42's `SH`/`MB`/`ME` |
| `encode_clrlwi31` | `540007fe` | `54000000` | `000007fe` | same |
| `encode_clrlwi_record` | `5400003f` | `54000001` | `0000003e` | same |

Full listing: `work/w-read-r2/control_p1.txt`.

> **⛔ AMENDED BESIDE — lane `w-encmap`, 2026-08-26, board #3641. THE SENTENCE
> ABOVE IS TRUE AND ITS SCOPE IS NARROWER THAN IT HAS BEEN READ.**
>
> ~~*"`encode.rs` accumulated 89 PPC words one captured obj at a time, never
> looking at `c2.dll`"*~~ — **the tense is the whole point and it is past.** The
> sentence describes **how the port's words were originally obtained**, and it
> stays true of that. It is **not** a statement about the tree it is read on:
> lane `w-s1` (2026-08-22, board `#3379`) moved all 85 primary/extended opcode
> literals into `mop.rs`'s read table, so the bits are no longer accumulated
> — verified on this tree, the live half of `encode.rs` composes **zero** words.
> What is still black-box-derived is the **choice of opcode and operand role per
> helper**, which `w-s1` did not touch. See §11 for the full adjudication of
> `#3634` against this sentence.
>
> **The consequence that matters, because a lane was priced on it.** `#3617`
> quoted this paragraph as the reason the `encode` row's `ported` strength was
> **not defined at all** — *"derived black-box from captured objs, never from
> these arms … so `sites the port implements` is not defined against the 79-arm
> population."* That inference does not hold. **How a port function was
> obtained has no bearing on whether it lands on an arm**; the map is a join on
> the **form**, which both sides carry, and `mop::plan` had already been citing
> the arm addresses one by one since `w-s1`. `ported` is **27 of 79** (§10) and
> the map cost a join, not a read. The residue was real; its stated reason was
> not.
>
> **What is NOT amended:** the 82-of-89 comparison, its seven residuals, and
> their attribution to named forms are `w-read-r2`'s measurement and stand
> unre-taken. §10 cites them; it does not re-derive them.

### 8.2 The arm rules against 500 real objs `[O]`

For every non-padding word of executable `.text` in 500 `dc3-decomp`
reference objs, the test was: does some opcode `op` satisfy
`(word & ~armmask(form[op])) == base_word[op]`, where `armmask` is **this
page's reading** of which bits the arm composes? A misread field width leaves
a bit outside the mask and the residual stops matching any base word.

```
words 634,457   explained 630,548 (99.3839%)   unexplained 3,909
relocation sites 124,700          unexplained AT a relocation site: 0
```

The 3,909 residuals are **not disagreements**: they are words of forms that
pass did not write a mask for (primary 30 `rld*`, primary 2/3 `tdi`/`twi`,
`mftb`). No word of a form stated above went unexplained.

**Second pass, with every read form masked** — including the default arm at
**mask 0**, which is the strongest claim available (the arm owns *no* bits):

```
words 634,457   explained 633,226 (99.8060%)   unexplained 1,231
```

**The second pass is weaker evidence than the first and must not be quoted as
stronger.** Sixteen VMX128 forms are masked at `0x03FFFFFF` — everything below
the primary opcode — because their scatter layout (§7.1) is per-form and this
lane did not transcribe sixteen separate masks. A generous mask cannot fail,
so those forms are *covered*, not *confirmed*. **The 99.38 % figure is the one
with teeth**; 99.81 % is the coverage statement.

**The residual is 1,231 words and it splits two ways:**

* **1,214 of primary opcode 4** — VMX/VMX128 in DC3's Bink and audio code,
  where even the generous mask does not reach. Unresolved, and named as such.
* **17 × `7c2004ac` = `lwsync`** — and this one is a **finding**. c2's machine
  table has **no `lwsync` opcode**: `sync` is `0x196` with base `7c0004ac` and
  form 113, which routes to the default arm, and the default arm sets no
  fields. **c2 cannot produce `7c2004ac` through this encoder at all.** The
  only path in the table that can is `emit` (`0x290`, form 18), whose arm
  copies an operand word verbatim (§5.6) — so `__lwsync()` reaches `.text` as
  a **literal word**, not as an opcode. That is exactly why form 18 is
  excluded from the mask set, and the exclusion turned a catch-all into a
  detector.

**The control was made capable of failing, and shown to be.** Four
deliberately-broken masks:

| mutation | explained |
|---|---|
| *(as read)* | **99.38 %** |
| `D` field 16 → 12 bits | 91.40 % |
| `RB` field 5 → 4 bits | 92.32 % |
| drop the `RA` field | 73.49 % |
| `SPR` unsplit | 95.66 % |

The `RB` mutation is worth its own sentence: on the **small** purpose-built
probe (46 words, §8.3) it changed **nothing**, because no word there used a
register ≥ 16. A control that cannot distinguish the hypothesis from its
mutation on the corpus you ran it on is not a control, and only the 500-obj
population made this one one. Evidence: `work/w-read-r2/p6_scale.log`.

### 8.3 The purpose-built probe `[O]`

`work/w-read-r2/probe/p6.cpp`, compiled by real `c2.dll` under wibo at
`/O1 /GS- /c /GR /Oi /EHsc`, deliberately containing a non-zero `D`-form
displacement, a negative immediate, an external symbol reference, an external
call and `Rc`/`LK`-bearing words: **46 of 46 explained, 7 relocation sites, 0
residuals.**

### 8.4 The form-coverage curve `[O]`

Over the same 500 objs, bucketing each word to its most-constrained matching
form: **27 forms cover 99.0 %** of emitted words and **15 cover 90.0 %**;
38 distinct forms appear at all, out of 104 in the table.

`w-ildecode`'s registered expectation — *"20–40 forms will cover ≥99 % of
emitted words"* (`WB_MIDDLE_INTERFACES.md` §8.1;
`ROADMAP_SLICING_2026-08-21.md`'s encoder row) — is **scored and it is a
HIT**, first time it has been scored.

**The denominator, named rather than left implicit.** These are the
`dc3-decomp` build's objs, not the 878-TU workload manifest, and they are the
tree's *own* compiles at *its* flags. The curve is a statement about real
DC3-shaped C++ compiled by this c2, not about the graded workload; a form the
workload reaches and DC3 does not would not appear.

---

## 9. What this spec does not give I2

Stated so absence is not read as coverage (`STEP5_PRICING_2026-08-21.md`'s
I2 is *"the general lowering to `coff::Function`"*):

1. **The tuple stream.** This page starts at a finished machine tuple. What
   *builds* the tuple is `FUN_10bc2d7a`'s ~~189 arms~~ **61 real arms over 95 opcodes, plus one refusal over 94 ([`WB_ILARMS_MAP.md`](../WB_ILARMS_MAP.md) §1)** — read **R5**, ~~unstarted~~ **DONE 2026-08-23, `ref/P_ILRECORD.md`, #3415**.
   An encoder is a total function of a tuple you do not yet have.
2. **Operand-record layout beyond the fields used here.** `+0x00` next,
   `+0x04` opcode-or-address-mode, `+0x08` kind, `+0x09` flag byte,
   `+0x0a` packed word, `+0x18` immediate, `+0x1c` symbol, `+0x24`/`+0x28`/
   `+0x2c`/`+0x30` address components — each named because an arm reads it,
   none exhaustively enumerated.
3. **Relocation *records*.** §6 gives the request; the sink is unread.
4. **Label placement and block order** — `CEILING` phase 1, still the one
   unserved phase, and reads R3/R8.
5. **`DAT_10c6fd9c`** — a 16-bit target/ABI word compared against `2` and
   `0x20` in three places (`0x10bfa5cc`, `0x10bf9e63`, `0x10bf9ec3`) and
   deciding the static-address base register. Read it before implementing
   §5.4's `kind == 2` path.
6. **The `[R]` bound.** Everything in §5 is `[R]` at instruction level and
   `[O]` only through §8.2's mask test, which confirms *which bits an arm
   owns* and not *which operand it read them from*. A rule that puts the right
   bits in the right place from the wrong operand passes §8.2 and would still
   be wrong. That distinction is exactly `C2_MAP_METHOD.md` §7's, and closing
   it needs the tuple stream from (1).

---

## 10. The arm → PORT map — lane `w-encmap`, 2026-08-26, boards #3636–#3641

**No evidence mark appears in this section, deliberately.** §1–§9 read
`c2.dll`; this section reads **the port**, against §1–§9's result. Marking a
port-side row with an evidence letter would both misuse the legend and silently
move the `encode` row's published agreement census (obj-marks 9 of 28 — the
counter
in `crates/c2-harness/src/subsys.rs` counts every mark after the page's first
`---`). A map of what we built is not new evidence about what c2 does.

**Nothing was re-read for this section.** All 79 arms were already read by
`w-read-r2` (`#3376`), and [`../READ_PLAN_2026-08-21.md`](../READ_PLAN_2026-08-21.md)
carries them in its already-read half. This section is a **join**, not a read.

### 10.1 The headline, with its denominator and why that denominator

**`ported` = 27 of 79 arms (34.18 %).**

The denominator is the **79 arms**, and the choice is published rather than
silent because this page has **three** defensible ones that differ by up to
5.6×:

| candidate | value | why not |
|---|---:|---|
| Ghidra function entries in the encoder band | 14 | `SUBSYS.md` §1's cell. A different population entirely — functions, not rules. `w-submetric` already recorded the 5.6× trap on this row |
| jump-table **entries** | 111 | **re-measured on this tree: the 111 entries ARE 111 forms, and every form belongs to exactly one arm** (0 forms served by two arms). Counting entries counts one arm up to 12 times — `10bfae1b` alone owns 12 |
| **distinct arm targets** | **79** | **chosen.** `read` on this row is already `79 distinct encode arms`, so the tuple's containment `sites ⊇ read ⊇ ported` is well formed only in the arm unit |

### 10.2 The predicate, and the two readings published beside it

An arm counts as **ported** iff some form it serves has **both** a field plan
in `mop::plan` **and** an `OPCODES` row that reaches it. Requiring the second
half is the conservative choice: a plan no opcode reaches composes nothing.

| reading | value | what it means |
|---|---:|---|
| strict — *every* form of the arm reachable | **25** | 3 arms are served partially: `10bfa2a5` (55 yes, 3 no), `10bfa34f` (14 yes, 12 no), `10bfad76` (68 yes, 69 no) |
| **published** — some form reachable, plan **and** opcode | **27** | |
| loose — a plan exists, opcode or not | **28** | the extra arm is `10bfa26c` (form 2, `bgip`): the port has the placement rule and no instruction that uses it |

**The published rule grants an arm on one of its forms, so it OVER-states
partial coverage.** That bias is stated here and re-stated in the instrument's
own caveat; `the_three_ported_readings_are_distinct` fails if the three ever
collapse onto each other, so the caveat cannot decay into decoration.

### 10.3 The map — 27 arms

Port sites are `crates/c2-core/src/codegen/mop.rs`, whose `plan()` **already
cites the c2 arm address it was read from, arm by arm**. This table is the
inverse of those citations, joined against `ENCODE_ARMS.txt` and checked to be
total.

| c2 arm | form(s) | c2 opcodes | port field plan | port mnemonics through it |
|---|---|---:|---|---|
| `10bfa456` | 22,49 | 77 | `mop.rs:673` | 14: `add`, `adde`, `divw`, `divwu`, `fadd`, `fadds`, `fdiv`, `fdivs`, `fsub` … |
| `10bfa4df` | 25 | 28 | `mop.rs:679` | 2: `fmr`, `frsp` |
| `10bfa53b` | 39 | 28 | `mop.rs:685` | 10: `and`, `andc`, `eqv`, `or`, `or.`, `orc`, `slw`, `sraw`, `srw` … |
| `10bfa1a1` | 28,61 | 23 | `mop.rs:731` | 2: `stdx`, `stfsx` |
| `10bfa17f` | 26,50 | 22 | `mop.rs:730` | 3: `lfsx`, `lhzx`, `lwzx` |
| `10bfa4c8` | 47 | 20 | `mop.rs:691` | 3: `addze`, `neg`, `subfze` |
| `10bfa667` | 21,45,46 | 16 | `mop.rs:720/727` | 7: `lbz`, `lbzu`, `ld`, `lfd`, `lfs`, `lhz`, `lwz` |
| `10bfa676` | 27,58,71 | 13 | `mop.rs:721/728` | 10: `stb`, `std`, `stdu`, `stfd`, `stfs`, `stfsu`, `sth`, `sthu`, `stw` … |
| `10bfa587` | 38 | 9 | `mop.rs:693` | 4: `cntlzw`, `extsb`, `extsb.`, `extsh` |
| `10bfad76` | 68 (+69 not reached) | 8 | `mop.rs:715` | 2: `rldicl`, `rldimi` |
| `10bfa56b` | 43 | 6 | `mop.rs:698` | 2: `ori`, `xori` |
| `10bfa4ed` | 51 | 6 | `mop.rs:695` | 6: `addi`, `addic`, `addic.`, `addis`, `mulli`, `subfic` |
| `10bfa2a5` | 55 (+3 not reached) | 5 | `mop.rs:741` | 2: `bctrl`, `blr` |
| `10bfa34f` | 14 (+12 not reached) | 4 | `mop.rs:758` | 2: `cmp`, `cmpl` |
| `10bfa2b0` | 4 | 4 | `mop.rs:743` | 2: `bcctr`, `bclr` |
| `10bfa549` | 36 | 4 | `mop.rs:689` | 1: `mr` |
| `10bfa326` | 5 | 4 | `mop.rs:746` | 1: `bc` |
| `10bfa478` | 23 | 4 | `mop.rs:677` | 2: `fmul`, `fmuls` |
| `10bfa6dc` | 42 | 2 | `mop.rs:703` | 2: `rlwinm`, `rlwinm.` |
| `10bfa801` | 64 | 2 | `mop.rs:761` | 1: `twi` |
| `10bfa2c2` | 1 | 2 | `mop.rs:751` | 1: `bdnz` |
| `10bfa263` | 6 | 2 | `mop.rs:754` | 1: `b` |
| `10bfa685` | 41 | 2 | `mop.rs:700` | 1: `srawi` |
| `10bfa719` | 56 | 2 | `mop.rs:703` | 1: `rlwimi` |
| `10bfa415` | 16 | 1 | `mop.rs:759` | 1: `cmpli` |
| `10bfa7a3` | 62 | 1 | `mop.rs:736` | 1: `mtspr` |
| `10bfa3ba` | 15 | 1 | `mop.rs:759` | 1: `cmpi` |

### 10.4 The 52 arms nothing in the port implements — the load-bearing half

They are **not uniform**, and reading them as one number would be the mistake:

| class | arms | c2 opcodes | note |
|---|---:|---:|---|
| VMX / VMX128 | **25** | **243** | includes the default arm `10bf9f91` alone at **104** opcodes (form 78 — §3.2's *"the default arm is an ENCODING, not a refusal"*). §8.2's residual is the same population: 1,214 of its 1,231 unexplained words are primary opcode 4 |
| everything else | **27** | **100** | enumerated below |

The 27 non-VMX unmapped arms, by c2 opcode count:

| arm | forms | opcodes | family |
|---|---|---:|---|
| `10bfa81d` | 8,9,10,11,13,48,60 | 19 | CR-logical (`crand`, `cror`, …) |
| `10bfa49a` | 24 | 18 | FP multiply-add (`fmadd`, `fmsub`, `fnmadd`, …) |
| `10bfa1ad` | 37 | 13 | the nop family — **its own second jump table** (`0x10bfafe9`, 9 entries), decoded at §4 by `w-r8idiom` |
| `10bfa8ae` | 111 | 9 | cache ops (`dcbf`, `dcbz`, `dcbz128`, …) |
| `10bfad3b` | 66,67 | 4 | `rldcl`/`rldcr` |
| `10bfa75c` | 20,44 | 4 | `mfcr`, `mffs`, `mfmsr` |
| `10bfae1b` | 35,52,53,59,72,73,87,89,95,96,98,100 | 3 | **the single exit** — the no-field arm; every form that composes nothing routes here |
| `10bfa79e` | 57 | 3 | `mtmsr` family |
| `10bfadb7` | 70 | 2 | `sradi` |
| `10bfa83a` | 19 | 2 | `mtfsf` |
| `10bfa7f4` | 63 | 2 | `td`/`tw` |
| `10bfa741` | 31 | 2 | `lcarry` |
| `10bfa6a1` | 40 | 2 | `rlwnm` |
| `10bfa5b4` | 32 | 2 | `lea` |
| **`10bfa5a0`** | 33 | 2 | **`li`/`lis` — and the port DOES emit both** (§10.5) |
| **`10bfa285`** | 7 | 2 | **`bl`/`bla` — and the port DOES emit `bl`** (§10.5) |
| `10bfae00` | 109 | 1 | `mfocrf` |
| `10bfade3` | 110 | 1 | `mtocrf` |
| `10bfa84e` | 65 | 1 | `DCD` |
| **`10bfa846`** | 18 | 1 | **`emit`** — the arm that copies an operand word verbatim. §8.2 found this is how `__lwsync()` reaches `.text`, and `#3482` found `mr r8,r8` is `emit 0x7d084378`. The port implements neither |
| `10bfa827` | 17 | 1 | `mtcrf` |
| `10bfa7d0` | 108 | 1 | `mftb` |
| **`10bfa76a`** | 54 | 1 | **`mfspr` — and the port DOES emit `mflr`** (§10.5) |
| `10bfa646` | 34 | 1 | `loffs` |
| `10bfa61e` | 29 | 1 | `lal` |
| `10bfa602` | 30 | 1 | `lau` |
| `10bfa26c` | 2 | 1 | `bgip` — the port has the **plan** and no opcode (§10.2's loose reading) |

### 10.5 What building the map FOUND: `mop` is not the port's only word composer

Three of the rows above are marked because **the port emits those instructions
and does not go through the arm's rule to do it.** Following that thread found
the finding of this lane, and it is not the one the wave expected.

`crates/c2-core/src/codegen/mop.rs`'s module doc says, twice, and
`codegen/encode.rs`'s says once more:

> *"`base_word` is now the port's **only** source of a primary opcode, so the
> two derivations can no longer drift apart silently."*

**That claim is false on this tree.** Enumerating every live (non-`cfg(test)`)
instruction-word production in `crates/c2-core/src` finds **eleven** outside
`mop::encode_op`:

| site | word | c2 opcode | form | arm | `mop` could compose it? |
|---|---|---|---:|---|---|
| `codegen/calls.rs:36` `encode_tail_branch` | `0x48000000 \| disp` | `b` `0x001f` | 6 | `10bfa263` | **YES — and does, `op::B`** |
| `codegen/calls.rs:142` `encode_call_branch` | `0x48000000 \| disp \| 1` | `bl` `0x002b` | 7 | `10bfa285` | no — no `OPCODES` row |
| `codegen/calls.rs:98` `lis` | `0x3C000000 \| d<<21` | `addis` `0x000e` | 51 | `10bfa4ed` | **YES — and does, `op::ADDIS`** |
| `codegen/calls.rs:102` `addi` | `0x38000000 \| d<<21 \| a<<16` | `addi` `0x000b` | 51 | `10bfa4ed` | **YES — and does, `op::ADDI`** |
| `codegen/calls.rs:106` `li` | `0x38000000 \| d<<21 \| k` | `addi` `0x000b` | 51 | `10bfa4ed` | **YES — and does** |
| `codegen/frame.rs:54` `FRAME_LR_STORE` | `0x9181FFF8` | `stw` `0x017a` | 58 | `10bfa676` | **YES — and does, `op::STW`** |
| `codegen/frame.rs:57` `FRAME_LR_LOAD` | `0x8181FFF8` | `lwz` `0x00d6` | 45 | `10bfa667` | **YES — and does, `op::LWZ`** |
| `codegen/frame.rs:60` `FRAME_MFLR_R12` | `0x7D8802A6` | `mfspr` `0x00e6` | 54 | `10bfa76a` | no — no `OPCODES` row |
| `codegen/frame.rs:63` `FRAME_MTLR_R12` | `0x7D8803A6` | `mtspr` `0x00f8` | 62 | `10bfa7a3` | **YES — and does, `op::MTSPR`** |
| `codegen/frame.rs:70` `FRAME_STWUX` | `0x7C21616E` | `stwux` `0x017f` | 61 | `10bfa1a1` | plan yes, no `OPCODES` row |
| `codegen/frame.rs:74` `FRAME_BACKCHAIN` | `0x80210000` | `lwz` `0x00d6` | 45 | `10bfa667` | **YES — and does, `op::LWZ`** |

**Seven of the eleven are the same word produced twice by two different rules,
both live on the emit path.** Worked by hand from this page's own §5 rules,
`mop` reproduces each of the seven exactly — e.g. `stw r12,-8(r1)` through form
58 is `0x90000000 | 12<<21 | 1<<16 | 0xFFF8` = `0x9181FFF8`, which is
`FRAME_LR_STORE` to the bit. **They agree today**, which is precisely why
nothing has caught them: this is a **latent** duplicate, not a live wrong emit,
and the gate is green.

**This is the failure class the repo has recorded most often** — two
independent producers of one quantity, caught once only by a name collision —
and the eleven sites are exactly the population `mop.rs`'s "no longer drift
apart silently" sentence claims does not exist. Consumers of the pledge should
read it as: *`base_word` is the only source of a primary opcode **for
instructions that go through `MachineOp`***.

**Not repaired here, deliberately.** Routing `calls.rs` and `frame.rs` through
`mop` is an **emit change** and needs its own two-sided price; and
`crates/c2-core/src/codegen/**` is another lane's fence this wave. Filed as
board **#3637** (the duplicate productions) and **#3638** (the false
"only source" claim, which is `#3632`'s class found by a different route).

### 10.6 Two stale counts in `mop.rs`, found by re-measuring rather than carrying

Both are doc comments in a file this lane may not write; reported, not fixed.

| claim | says | this tree | where |
|---|---|---|---|
| the `OPCODES` subset | *"71 of c2's 660 rows"* | **85 rows** | `mop.rs:257` |
| the form reach | *"the port's 71 opcodes reach 24 of c2's 109 forms"* | **85 opcodes reach 34 forms**; `plan()` answers **35** | `mop.rs:664–665` |

Neither is load-bearing for an emitted byte — they are descriptions of a table,
not the table — but both are quoted as denominators, and `#3617` quoted the
`89`/`82` pair from §8.1 as the reason `ported` could not be computed at all.
Filed as **#3639**.

---

## 11. `#3634` adjudicated — both readings are true, and the doc comment is the thing that is wrong

Board `#3634` says `encode.rs`'s black-box re-derivation *"was retired"*, leaving
*"9 S-marks and 2 O-marks, zero F-marks"*. Decision 16 says `encode.rs`'s module doc
*"still declares itself a black-box re-derivation"* and asks which is right.

**Both quote the same doc comment, and the doc comment contradicts itself.**
`crates/c2-core/src/codegen/encode.rs` carries, in one `//!` block:

* **lines 8–25** — *"**This file is a black-box re-derivation of two tables c2
  states plainly, and the read is priced — comment only, nothing here
  changes.**"* Written 2026-08-22 **before** read R2 landed, describing R2 as
  future work (*"Read **R2** (2–4 d) dumps both tables"*).
* **lines 35–58**, twenty-seven lines later — *"**2026-08-22, lane `w-s1` — THE
  BLACK-BOX RE-DERIVATION IS RETIRED.**"*

The first paragraph was **never struck** when the second was appended. `#3634`
read the bottom; decision 16 read the top; neither is misreading the file.

**The adjudication, per object:**

| object | status | evidence |
|---|---|---|
| the **primary/extended opcode literals** (85 of them) | **RETIRED — `#3634` is right** | every one is now `MachineOp::new(op::X)` naming a read table row. Verified on this tree: the live half of `encode.rs` (lines 1–2006) contains **zero** `to_be_bytes` and every `<<` is inside a doc comment |
| the **11 remaining named constants** | **9 S-marks + 2 O-marks, 0 F-marks — `#3634` is right** | not re-counted here; that is the provenance census's key |
| the **89 helper functions' choice of opcode and operand roles** | **STILL black-box-derived — decision 16's concern is right** | `w-s1` moved where the *bits* come from. It did not re-derive *which* c2 opcode a given port lowering should name, or which operand plays which role; those still come from captured objs, and §8.1's per-function evidence notes are still what the port is graded on |
| the module doc's **opening paragraph** | **STALE, and it is the actual defect** | it describes R2 as unstarted work and the file as unchanged by it |

**So `#3634` is a census of constants and is correct as such; it is an over-read
only if quoted as "nothing in `encode.rs` is black-box-derived any more", which
its own text does not say.** The two claims are compatible and the file is what
is wrong.

**The fix site is `crates/c2-core/src/codegen/encode.rs`, which is fenced to
lane `w-disclose` this wave (comment-only edits in `codegen/**`).** This lane
therefore **STOPS and reports** rather than editing it. Filed as **#3640**.
