# P_SUB4F — the `.ex` `0x4F` sub-record grammar (`FUN_10b9761e`)

> **PROVENANCE — DISASSEMBLY-DERIVED.** Everything here was obtained by
> reading `compilers/X360/16.00.11886.00/c2.dll`, sha256
> `c80981c015166effecc71ad8112d5577a065b2300891dfdb02b9c13787a66258`,
> verified before the first address was quoted. Whitebox analysis is
> authorized and encouraged (`CLAUDE.md`, project owner, 2026-08-17).
> Nothing here may enter `crates/` without a `DISCLOSURE.md` row naming the
> address it came from. **No row is owed by the lane that wrote this page**
> — it adopted nothing and changed no `crates/` byte.

**Read R9** of [`../READ_PLAN_2026-08-21.md`](../READ_PLAN_2026-08-21.md) §3,
lane `w-read-r9`, board **#3442**–**#3444**. Prereg:
[`../WB_SUB4F_PREREG.md`](../WB_SUB4F_PREREG.md) (committed first,
`fe9a08f39`). Grade: [`../WB_SUB4F_FINDINGS.md`](../WB_SUB4F_FINDINGS.md).
Instruments: [`../scripts/dump_sub4f.py`](../scripts/dump_sub4f.py) (the read)
and [`../scripts/sub4f_probe.py`](../scripts/sub4f_probe.py) (the
confirmation probe), both re-runnable and sha256-fenced.

---

> ## ⛔ `0x10b26268` IS NOT A WIDTH TABLE. IT IS A TABLE OF FORMAT STRINGS.
>
> `DISCLOSURE.md:89` (`W-MID-4`), `WB_MIDDLE_INTERFACES.md:203-208`,
> `READ_PLAN:103,178` and `crates/c2-reference/tests/middle_interfaces.rs:689`
> all describe *"an 8-byte-stride descriptor table at `0x10b26268` and then a
> ~14-arm switch"* and infer that **the record's widths live in the table**.
> They do not. The table's first dword is a **`const char *`** to a
> NUL-terminated string of **field-type codes**, and `FUN_10b9761e` is a
> **format-string interpreter** that walks that string one byte at a time
> (`0x10b97860` fetch, `0x10b9785a` advance) dispatching each code through the
> cascade at `0x10b9766c`.
>
> **So the "~14 arms" figure is RIGHT and its referent is WRONG.** There are
> exactly **14** arms — 13 field-type codes plus one default — but they
> enumerate **field types**, not sub-opcodes. The sub-opcode domain is
> **64**. This is R5's error (`P_ILRECORD.md`'s "189 arms is an opcode count")
> in mirror image: there, the count was wrong; here, the count is right and
> the level it describes is wrong. A width is not a table cell — it is the sum
> over a string of codes, and three of those codes are data-dependent.

---

## 1. The route — and `0x4F` owns this table alone

| step | address | what happens | mark |
|---|---|---|---|
| record opcode `0x4F` read | — | operand-format class byte at `0x10b25e48 + 0x4F` is **`0x0C`** | `[R]` |
| class-`0x0C` arm | **`0x10b3d7d7`** | `call 0x10c1f9a6` (VI16) → `mov word ptr [ebx+0x24], ax` → `call 0x10b9761e` | `[R]` |
| sub-opcode fetch | **`0x10b9763d`** | `movsx eax, byte ptr [esi+0x24]` — **the LOW BYTE, SIGNED** | `[R]` |
| descriptor load | **`0x10b97641`** | `mov eax, dword ptr [eax*8 + 0x10b26268]` | `[R]` |
| null check | `0x10b9764c`–`0x10b9764e` | `test eax,eax / jne` — a null descriptor means **no payload**, return | `[R]` |
| code fetch | **`0x10b97860`** | `mov al, byte ptr [eax]` — the next field-type code | `[R]` |
| terminate | `0x10b97862`–`0x10b9786a` | `test al,al / jne` — NUL ends the record | `[R]` |
| advance | **`0x10b9785a`** | `inc dword ptr [ebp-0x7c]` — one byte along the format string | `[R]` |
| dispatch | **`0x10b9766c`** | `movsx eax,al` then the 13-way compare cascade | `[R]` |

**`0x4F` is the only opcode in the image whose operand-format class is
`0x0C`** — checked over the whole 192-entry class table `0x10b25e48`
(`dump_sub4f.py --selftest`). So this table is the `0x4F` sub-record table and
nothing else, and `FUN_10b9761e` has **exactly one caller** (`0x10b3d7e2`),
matching `ref/FUNCS.tsv:2084`'s "1 caller / 13 callees". `[R]`

**`0x10b26268` appears exactly once as an immediate anywhere in the image.**
There is no second reader, and **the entry's second dword is read nowhere at
all** — no instruction in the image forms the address `base + 8i + 4`. It is
reported below as *unread*, not as absent. `[R]`

### 1.1 The containing TU, from the code's own argument

Both refusal arms call the ICE reporter `0x10b33526` with
`ecx = 0x10b163a0`, which is the wide string
`L"e:\bt\278379\vctools\compiler\be\p2\p2pragma.c"` — independently
confirming `c2_tus.tsv:29`'s attribution of `FUN_10b9761e` to `p2pragma.c`
from inside the function. `edx` is the source line. `0x10b33526` ends in
`int3`, so **both refusals are fatal**. `[R]`

---

## 2. The descriptor table

```
0x10b26268   64 entries, stride 8, ends 0x10b26468
entry        { const char *fmt ;  u32 <READ NOWHERE IN THE IMAGE> }
```

**The extent is fixed two independent ways** (both in `--selftest`):

1. entries `0x34..0x3f` carry no format pointer — the table's zero tail;
2. `0x10b26268 + 64 × 8 = 0x10b26468` is **exactly** where the next object
   begins: the wide string `L"…\be\common\vlines.c"`. A table of any other
   length would not land on an object boundary.

`ref/ADDR.tsv:93` records `0x10b26268` as `data`, size **4**, confidence
**`unknown`** — nobody had parsed it. That row is now wrong in its size field
and this page supersedes it. `[R]`

The format strings live in the **80 bytes immediately BELOW the table**,
`0x10b26218`–`0x10b26267`, four bytes apart. A stride check is in
`--selftest`: no descriptor points *inside* the table, which a wrong stride
would produce (`WB_MIDDLE_INTERFACES.md` §2.2's stride-12/stride-16 trap, by
name).

**Population of the 64 slots:** 29 carry a format string · 19 are null with
the second dword set to 1 · 16 are entirely zero. **At run time the last two
groups are indistinguishable** — the reader tests only the first dword — so
the table encodes a distinction the image never consults.

---

## 3. The 14 arms — field-type codes, not sub-opcodes

The cascade at `0x10b9766c` decides 13 codes and one default. `reads` is the
payload each arm consumes from the IL cursor `0x10c46310`, in stream order.

| code | ASCII | arm VA | reads | effect | mark |
|---:|:--:|---|---|---|:--:|
| `0x0b` | | `0x10b97706` | **BYTE** | → `0x10b97d47`, **which reads further stream** — DEFER, see §7 | `[R]` |
| `0x0c` | | `0x10b976c7` | **STR** | counted byte string via `0x10c1fca9` into `0x10c6b040`, cap `0x1020` | `[R]` |
| `0x0d` | | `0x10b9786f` | — | **ICE `p2pragma.c:88`** — a dedicated NOT-IMPLEMENTED arm | `[R]` |
| `0x0e` | | `0x10b976b9` | **VI16** | `mov word ptr [esi+0x10], ax` — low half only | `[R]` |
| `0x14` | | `0x10b976a8` | **VI32** | `cdq` → `node+0x10` / `node+0x14`, sign-extended to 64 | `[R]` |
| `0x15` | | `0x10b97718` | **VI16** | `cwde` → `cdq` → `node+0x10` / `node+0x14` | `[R]` |
| `0x16` | | `0x10b977fd` | **VARU** | → `DAT_10c2eaa0` **and `DAT_10c2edd0`**, the TU label counter — §6.1 | `[R]` |
| `0x17` | | `0x10b977d4` | **LIST-B** | `0x10b97584` into a `0x270`-cap buffer, then alloc + copy | `[R]` |
| `0x1a` | | `0x10b97762` | **VI16 + loop** | count `n` → `node+8`; then `n` × (VARU, VI16) into two arrays | `[R]` |
| `0x1d` | | `0x10b9774f` | **gated** | `DAT_10c2eb5c != 0` ? as code `0x15` : **ICE `:160`** | `[R]` |
| `0x1e` | | `0x10b9773d` | **LIST-A** | `0x10b97502` into a `0x270`-cap buffer, then alloc + copy | `[R]` |
| `0x6c` | `l` | `0x10b9780e` | **gated** | `DAT_10c2eb4c != 0` ? **VI32** : VI16 zero-extended; also → `DAT_10c2e2e4` | `[O]` |
| `0x73` | `s` | `0x10b9783d` | **VARU** | symbol token → `0x10b9880d` (hash `% 0x7f` over `[0x10c472e8]+0x3c`); sets `node+0x0 := 0x56` | `[O]` |
| *default* | | `0x10b97758` | — | `mov edx,0xa0` → **ICE `p2pragma.c:160`** (fatal, `int3`) | `[R]` |

`l` for **line** at sub-opcode `0x01` and `s` for **symbol** at `0x02` are not
coincidences: the two ASCII codes in the set are the two the corpus actually
exercises, and both decode to exactly what the black-box record already said
they were.

### 3.1 The scalar readers, all driven off the IL cursor `0x10c46310`

All five live in **`ioin.c`**, not in `p2pragma.c` — `0x10c1fca9`'s own ICE
argument is `0x10b261b8` = `L"…\be\common\ioin.c"`. So the record grammar
(`p2pragma.c`) and the byte-level varint decoding (`ioin.c`) are separate
TUs, which is why the widths are expressible as five reusable primitives.

| name | VA | width | rule | mark |
|---|---|---|---|:--:|
| **BYTE** | `0x10c1f8fc` | **1** | one raw byte, no escape | `[R]` |
| **VARU** | `0x10c1f91b` | **2 or 4** | reads a `u16`; if **bit 15** is set, reads two more and combines | `[O]` |
| **VI16** | `0x10c1f9a6` | **1 or 3** | one byte sign-extended; **`== 0x80`** escapes to two more | `[O]` |
| **VI32** | `0x10c1f9e9` | **1 or 5** | one byte sign-extended; **`== 0x80`** escapes to four more | `[O]` |
| **STR** | `0x10c1fca9` | **VI16 + n** | VI16 count `n`, then `n` raw bytes; `n > cap` → ICE **`ioin.c:300`** (`0x10b261b8`) | `[R]` |

VARU's "2 or 4 with a continuation flag in bit 15" reproduces
`DISCLOSURE.md`'s **`W-GLATTRS-1`** row exactly, from a different call site and
a different lane. That row was established black box; this is the instruction
sequence behind it.

### 3.2 The two list siblings, both in `p2pragma.c`

- **`0x10b97502`** (LIST-A) — repeats { **VI16** tag; if tag `== -1` stop;
  **BYTE** } . Overflow past the caller's cap → ICE `:195`; a cap `<= 1` →
  ICE `:176`. `[R]`
- **`0x10b97584`** (LIST-B) — repeats { **BYTE** `b`; if `b == 0x0f` stop;
  **VARU**; then **VI32** repeatedly while non-zero } . Overflow → ICE `:237`;
  cap `<= 1` → ICE `:215`. `[R]`

---

## 4. THE WIDTH TABLE — every one of the 64 sub-opcodes

Record layout: **`4F` · VI16(sub) · fields per the format string.**
"payload" below is the field part only. Regenerate with
`dump_sub4f.py <c2.dll> --table`.

| sub | codes | payload | total record | mark |
|---:|---|---|---|:--:|
| `0x01` | `6c` | **VI32: 1 or 5** | **3 or 7** | **`[O]`** |
| `0x02` | `73` | VARU: 2 or 4 | **4 or 6** | **`[O]`** |
| `0x03` | `64 4b` | — | **ICE `:160`** | `[R]` |
| `0x04` | `6f` | — | **ICE `:160`** | `[R]` |
| `0x05` | `43` | — | **ICE `:160`** | `[R]` |
| `0x06` | `49` | — | **ICE `:160`** | `[R]` |
| `0x07` | *(empty)* — see note | none | **2** | `[R]` |
| `0x0a` | `07` | — | **ICE `:160`** | `[R]` |
| `0x0b` | `0b` | BYTE + DEFER (§7) | ≥ 3, **unbounded** | `[R]` |
| `0x0c` | `4d` | — | **ICE `:160`** | `[R]` |
| `0x0d` | `41` | — | **ICE `:160`** | `[R]` |
| `0x0e` | `46` | — | **ICE `:160`** | `[R]` |
| `0x0f`,`0x10`,`0x11`,`0x12` | *(null)* | none | **2** | **`[O]`** for `0x11`,`0x12` |
| `0x13` | `08` | — | **ICE `:160`** | `[R]` |
| `0x14` | `09` | — | **ICE `:160`** | `[R]` |
| `0x15` | `0a` | — | **ICE `:160`** | `[R]` |
| `0x16` | `0c` | STR: VI16 + n | 2 + 1..3 + n | `[R]` |
| `0x17` | `0d` | — | **ICE `:88`** (dedicated) | `[R]` |
| `0x18` | `0e` | VI16: 1 or 3 | 3 or 5 | `[R]` |
| `0x19`,`0x1a`,`0x1b`,`0x1c` | *(null)* | none | **2** | `[R]` |
| `0x1e` | `62` | — | **ICE `:160`** | `[R]` |
| `0x1f` | `14` | VI32: 1 or 5 | 3 or 7 | `[R]` |
| `0x20` | `15` | VI16: 1 or 3 | 3 or 5 | `[R]` |
| `0x21` | `71` | — | **ICE `:160`** | `[R]` |
| `0x22` | *(null)* | none | **2** | `[R]` |
| `0x23` | `17` | LIST-B | variable | `[R]` |
| `0x24` | `02` | — | **ICE `:160`** | `[R]` |
| `0x25` | `18` | — | **ICE `:160`** | `[R]` |
| `0x26` | `19` | — | **ICE `:160`** | `[R]` |
| `0x28` | `1a` | VI16 `n`, then `n`×(VARU,VI16) | variable | `[R]` |
| `0x32` | `1d` | gated: VI16 or **ICE `:160`** | 3 or 5 | `[R]` |
| `0x33` | `1e` | LIST-A | variable | `[R]` |
| `0x35`..`0x3e` | *(null)* | none | **2** | `[R]` |
| `0x00`,`0x08`,`0x09`,`0x27`,`0x29`..`0x31`,`0x34`,`0x3f` | *(zero)* | none | **2** | `[R]` |

**Summary of the domain: 12 sub-opcodes decode a payload · 17 are FATAL
(16 → ICE `:160`, `0x17` → ICE `:88`) · 35 read nothing.** `12 + 17 + 35 =
64`, checked arithmetically rather than asserted.

> **Note on `0x07`, because "empty string" would overstate it.** Its
> descriptor is `0x10b01bbe`, which is **not** the address of an empty string
> literal — it points at a NUL *inside* another literal's padding
> (`… 61 74 61 00 | 00 00 20 20 …`). Behaviourally the interpreter reads a
> NUL and consumes no payload, so the record is 2 bytes; but the pointer looks
> like a compiler-emitted alias onto a shared zero byte rather than a
> deliberate empty format. Stated precisely so a later reader does not infer a
> distinct "empty format" concept that the data does not support. `[R]`

### 4.1 The corpus-witnessed seven, and the port's three constants

`IL_STMT_GRAMMAR.md` §12.6 names seven `0x4F` sub-opcodes seen in real IL.
**All seven are admitted and none reaches a refusal arm** — and three of the
port's own transcribed constants fall straight out of the table:

| sub | table says | the port already had | agrees? |
|---|---|---|---|
| `0x01` | `4F` + VI16 + **VI32** | `4f 01 80 c8 00 00 00` at line 200 (`readers.rs:391-408`) | **yes** — VI32's escape is `0x80` + 4 |
| `0x02` | `4F` + VI16 + **VARU** = **4 bytes** | `BLOCK_START = [0x4F,0x02,0x20,0x00]` (`codec.rs:103`) | **yes**, exactly 4 |
| `0x11` | null → **2 bytes** | `LO_RECORD = [0x4F,0x11]`, `(ExToken::LoRecord, 2)` | **yes**, exactly 2 |
| `0x12` | null → **2 bytes** | `FN_TAIL = [0x4F,0x12,0x47,0x54,0x01,0x54,0x00]`, 7 bytes | **no** — see §5 |
| `0x1f` | VI32 | `FN_START = [0x4F,0x1F]`, a 2-byte *marker* only | not contradicted |
| `0x20` | VI16 | `4F 20` descriptor, "not yet field-typed" (`IL_BUNDLE_MVP.md:259`) | now typed |
| `0x33` | LIST-A | `4F 33` metadata payload, "not yet field-typed" | now typed |

---

## 5. The `47` that `IL_STMT_GRAMMAR.md` §12.6 calls unexplained

> *"The single byte `47` between `4F 12` and the outer scope closes is
> **unexplained**; it is a fixed byte in every one of the ~5300 bodies
> examined."*

**It is the next record's opcode.** Sub-opcode `0x12`'s descriptor is null, so
`4F 12` consumes **two bytes and nothing else**; `47 54 01 54 00` is a
separate token sequence that the port's 7-byte `FN_TAIL` literal swallows as
one unit. The literal still *matches* — it is a correct transcription of a
byte sequence — but it mis-describes the grammar, and a decoder that treats
those seven bytes as one record cannot generalise to a tail followed by
anything else. `[R]`, and consistent with `[O]` across the corpus below.

---

## 6. Two globals worth naming

### 6.1 Code `0x16` writes `DAT_10c2edd0` — the TU label counter

`0x10b977fd` reads a **VARU** and stores it to **both** `DAT_10c2eaa0`
(`0x10b97802`) and **`DAT_10c2edd0`** (`0x10b97807`).

`ref/P_LABEL.md:51` (read R3) already records `0x10b97807` as *"seed install,
IL directive `0x16`"*, `[R]`. **The address is right, the number `0x16` is
right, and the word "directive" is wrong — `0x16` is a field-type code, not a
sub-opcode.** The distinction is not pedantry, because of what follows from
it.

**Exactly one of the 13 handled field-type codes is selected by no descriptor
in the table, and it is this one.** Enumerating the 29 format strings gives 29
distinct code bytes; intersecting with the cascade's 13:

```
handled by the cascade (13):  0b 0c 0d 0e 14 15 16 17 1a 1d 1e 6c 73
used by some descriptor (29): 02 07 08 09 0a 0b 0c 0d 0e 14 15 17 18 19 1a
                              1d 1e 41 43 46 49 4b 4d 62 64 6c 6f 71 73
handled but NEVER selected:   16      <-- the label-seed arm, and only it
used but NOT handled (17):    02 07 08 09 0a 18 19 41 43 46 49 4b 4d 62 64
                              6f 71   <-- all reach ICE :160
```

So **`0x10b977fd`/`0x10b97807` is unreachable through this table**: no `0x4F`
sub-record can select it. The code is real and R3 read it correctly; it is
simply **not on the path any `0x4F` record takes** — precisely the failure
mode `ref/README.md:54-60` prices with the `.bss` bump rule, and precisely
what `[R]` is defined not to claim.

**This does not touch `LABEL_SEED_GAP`.** `w-seedgap`'s coefficients were
measured black box from a 22-cell obj grid reading `u32_le(.gl[7..11])`
(`DISCLOSURE.md` row `W-SEEDGAP-1`), so nothing shipped depends on this arm.
What it removes is a *mechanism story*, not a number.
`P_LABEL.md` is **amended beside, never rewritten** (`ref/README.md:72+`);
the grade is [`../WB_SUB4F_FINDINGS.md`](../WB_SUB4F_FINDINGS.md) §P7. `[R]`

**Checked by the instrument**, so it cannot rot: `dump_sub4f.py --selftest`
asserts that `0x16` is the unique handled-but-unselected code.

### 6.2 The two mode gates

- **`DAT_10c2eb4c`** (`0x10b9780e`) selects VI32 vs VI16 for code `l`. The
  grid proves it is **non-zero** in every configuration this project
  compiles: line 1 000 000 round-trips, which a 16-bit field cannot hold.
  `[O]`
- **`DAT_10c2eb5c`** (`0x10b9774f`) gates code `0x1d`; when clear, the arm
  falls into the default **ICE**. Never observed set or clear by this lane.
  `[R]`

---

## 7. The index is UNBOUNDED — there is no range check

`0x10b9763d`–`0x10b97641` is `movsx` then an indexed load with **no `cmp` in
between**, and the caller at `0x10b3d7d7` does not bound the value either.
Consequences, all `[R]`:

- the caller stores a **VI16** (`mov word ptr [ebx+0x24], ax`) and the reader
  takes the **signed low byte**, so sub-opcode `0x100` aliases `0x00`;
- sub-opcodes `0x40..0x7f` index **past** the table into `vlines.c`'s path
  string and dereference a wild pointer;
- sub-opcodes `0x80..0xff` sign-extend **negative** and index *below* the
  table, into the format-string pool.

Safety rests entirely on the front end never emitting a sub-opcode above
`0x3f`. This is a property of c2 as shipped, not a defect this project can
act on — it is recorded because a port that *generates* IL must respect it,
and because "the table has 64 entries" is only a bound on the **data**, never
on the **index**.

---

## 8. What this page does NOT establish

- **Semantics of the 12 decoding sub-opcodes beyond field types.** This is a
  *grammar*, not a meaning. What `0x1f`'s VI32 or `0x33`'s list *are* is not
  here.
- **Code `0x0b`'s true width.** `0x10b97d47` (a 0x20c4-byte-frame function)
  calls `0x10c1f8fc`, `0x10c1fc5b` and `0x10c20342` and therefore consumes
  further stream. Read to depth 2 and **DEFER**red, R5's convention.
- **Anything about `0x10bbe561`.** That is the record→**codegen** side
  (`P_ILRECORD.md:238`, arm 32). §P8 of the findings shows the two are
  disjoint and dispatch the same field at two different widths.
- **The second dword of every entry.** Never read anywhere in the image.
- **Any emitted byte.** This lane graded no obj, ported no pass and touched
  no `crates/` byte. Nothing here is `[O] port`.
