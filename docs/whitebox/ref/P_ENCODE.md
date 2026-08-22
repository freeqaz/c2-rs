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

**Coverage of this page, stated first.** **73 of the 79 distinct arms read**
(92.4 %). The 6 unread are single-opcode VMX128 arms, named in §7. Of the 660
machine opcodes, **653 reach an arm this page states a rule for** (98.9 %).

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

| helper | sites | emits | reached from |
|---|---:|---|---|
| `0x10bf976d` | 1 | **6** = `REL24`, then stashes the symbol in `DAT_10c6fd5c` | form 7 (`bl`) |
| `0x10bf96d9` | 1 | **0x0d** = `IFGLUE`, on the symbol `0x10bf976d` stashed | form 37, only when the opcode is `0x280` (`rsttoc`) — the *"nop after the call"* glue slot |
| `0x10bf96ea` | 2 | **0x10** = `REFHI`, then `0x12` = `PAIR` | form 51 when the opcode is `addis`; form 30 (`lau`) |
| `0x10bf9721` | 1 | **0x11** = `REFLO`, then `0x12` = `PAIR` | form 29 (`lal`) |
| `0x10bf9808` | 2 | `REFLO` + `PAIR`, if the memory operand carries a symbol | both `D`-form memory composers |
| `0x10bf9758` | 1 | **0x0f** = `SECREL16` | form 34 (`loffs`) |
| `0x10bd470b` + inline | 1 | **2** = `ADDR32` | form 65 (`DCD`) |

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

**The 6 unread arms**, all single-opcode VMX128, all in the family §5.7
describes: `0x10bfaa42` (form 101, `vcfpsxws128`/`vcfpuxws128`), `0x10bfaadd`
(103, `vperm128`), `0x10bfabf0` (97, `vrlimi128`), `0x10bfac63` (105,
`vsldoi128`), `0x10bfaccd` (106, `vupkd3d128`), `0x10bfacee` (107,
`vpkd3d128`). Their spans are in [`ENCODE_ARMS.txt`](ENCODE_ARMS.txt); a later
session continues there.

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

The 3,909 residuals are **not disagreements**: they are words of forms this
page did not write a mask for (primary 30 `rld*`, primary 2/3 `tdi`/`twi`,
`mftb`). No word of a form stated above went unexplained.

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
   *builds* the tuple is `FUN_10bc2d7a`'s 189 arms — read **R5**, unstarted.
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
