# WB_MIDDLE_INTERFACES — the opaque middle's two edges, addressed

> **PROVENANCE — DISASSEMBLY-DERIVED.** Every address is an absolute VA in the
> image pinned in [`C2_MAP_METHOD.md`](C2_MAP_METHOD.md) §0
> (`sha256 c80981…6258`). **`[R]`** read from the disassembly, not confirmed
> against any artifact — a hypothesis. **`[O]`** confirmed against a live tap
> row or a real obj, with the witness named. `[R]` means *"the instructions were
> read correctly"*, **never** *"this is what c2 does"*.
>
> Nothing here changes what the port emits. The port stays **I/O-behavioral**:
> this file documents c2's internals to make two interfaces legible, and every
> claim marked `[O]` is graded by OUTPUT equality — tap rows and obj bytes —
> never by matching c2's own instruction bytes.

Lane `w-ildecode` / `wt-w-ildecode`, branched at master `72207b86f`, rebased
onto `edd882f96` after `w-restim` landed. PREREG:
[`WB_MIDDLE_PREREG.md`](WB_MIDDLE_PREREG.md), committed before the confirming
runs; scored in §7. Board rows **#3357**–**#3360**.
Runnable proof: `crates/c2-reference/tests/middle_interfaces.rs` (4 tests).

---

## §0 Why this file exists, and what it is not

[`../ARCH_REVIEW_2026-08-21.md`](../ARCH_REVIEW_2026-08-21.md) finding 3 put two
unbudgeted prerequisites on step 5's critical path and priced them at **3–9
engineer-months**: a **general op-level IL decode** and a **general lowering to
`coff::Function`**. Neither is here. What is here is the **minimum subset that
makes those two interfaces legible** — the record layouts, the driver
addresses, the transform in prose, one worked example traced end to end, and
just enough graded code to show each documented fact is real.

The value is negative-space value: the review's estimate was made against an
opaque middle, and an estimate against a *documented* middle is a different
estimate. §8 re-states it.

**What this file does not claim.** It does not claim a general decode, a general
lowering, or that the subset generalizes. It claims that on three functions
across two fixtures the port already emits byte-exact, both interfaces are
reproducible from c2's own tables, and it names precisely which of its rules are
DERIVED from a table in the image and which are TRANSCRIBED from one snapshot.

---

## §1 The map — where the middle's two edges are

```
   .ex / .gl  ──►  the READER  ──►  … selection, lowering …  ──►  TUPLE LIST
   (K1 codec)      0x10bbc9ab                                    (tuple.c)
                   0x10b3d610                                        │
                                                                     ▼
   INTERFACE 1 is this whole arrow.                          sched1 ─ globregs
   c2 does not expose an intermediate.                       sched2 ─ COLOR
                                                             sched3
                                                                     │
                                                          the lowering band
                                                                     │
                                                              sched0 (run 4)
                                                                     │
                                                                after0  ◄── the
                                                                     │   last
                                                                     ▼   point
   COFF .text  ◄──  the ENCODER  ◄──  the final tuple order    observable
                    0x10bf9f15
   INTERFACE 2 is this arrow, and it is SHORT.
```

The asymmetry between the two arrows is the single most useful thing in this
document. **Interface 2 is a 30-line function over two array lookups.**
Interface 1 is the whole compiler.

| what | where | conf |
|---|---|---|
| the `.ex` token fetch | `0x10bbc9ab` (`reader.c`) | `[R]` `WB_READER_FINDINGS.md` §2 |
| the `.ex` operand decoder + class table | `0x10b3d610`, table `0x10b25e48` | `[O]` `WB_READER_FINDINGS.md` §5.3(1) |
| the tuple IR | `tuple.c` `0x10bd398a` | `[R]` `WB_REGALLOC_FINDINGS.md` §1 |
| the per-function phase pipeline | `0x10b7d85e`, orchestrator `0x10b7e6af` | `[R]` `P_DAG.md` §1 |
| the four scheduler runs | driver `0x10be6382`, sites `0x10b7dc9f`/`0x10b7dcde`/`0x10b7dd1d`/`0x10b7e00c` | `[O]` `P_DAG.md` §1's 2026-08-20 correction |
| the observation point after run 4 | `0x10b7e701` (`after0`) | `[O]` `w-restim`, `W-STAGETAP-3` |
| **the instruction encoder** | **`0x10bf9f15`** (`code.c`) | **`[O]` §5** |

---

## §2 THE OPCODE-NUMBER SPACE — the thing that made the stream unreadable

`ARCH_REVIEW` finding 4(b) says c2's opcode numbering *"is already erased by the
port's encoded-byte blocks, so the map is not a function"*. That is a statement
about the port. On c2's side the numbering is **one table, and it is named**.

### 2.1 The machine mnemonic table — `0x10b1b260`, stride 12 `[O]`

`{ char *name, u32 form, u32 flags }`, indexed by the opcode. Index `0` is
`_first`; index **`0x295` is `_last`**, so the machine opcode space is
`0x001 … 0x294`.

| op | name | op | name | op | name |
|---:|---|---:|---|---:|---|
| `0x001` | `add` | `0x0d6` | `lwz` | `0x271` | `lis` |
| `0x00b` | `addi` | `0x17a` | `stw` | `0x272` | `mr` |
| `0x00e` | `addis` | `0x21` | `bc` | `0x284` | **`ret`** |
| `0x01f` | `b` | `0x2d` | `cmp` | `0x285` | `blr` |
| `0x02b` | `bl` | `0x2e` | `cmpi` | `0x270` | `li` |

`P_DAG.md` §2.1's list (`addi` = 11, `lis` = 625, `blr` = 645) is **exactly
right and 0-based**, re-derived here by
`docs/whitebox/scripts/dump_opcode_tables.py`.

### 2.2 A SECOND table nobody had named — `0x10b1d180`, stride 16 `[R]`

Immediately after the first (`0x10b1b260 + 0x298 × 12 = 0x10b1d180`), and it is
**not a continuation of the same index space**: the inline-asm mnemonic parser
at `0x10c0174b` walks it from index **1** with `shl eax,4`, while
`0x10c00900` walks the first table with `imul eax,eax,0xc`. It is the
**extended/simplified-mnemonic table**: `{ char *name, u32 real_opcode, u32 BO,
u32 BI }`.

```
  1 subi   -> 0x00b (addi)       8 blt  -> 0x21  BO=12 BI=0
  2 subis  -> 0x00e (addis)      9 ble  -> 0x21  BO=4  BI=1
  4 subic. -> 0x00d (addic.)    10 beq  -> 0x21  BO=12 BI=2
  5 sub    -> 0x181 (subf)      13 bne  -> 0x21  BO=4  BI=2
 28 cmpw   -> 0x02d (cmp)       31 cmplw-> 0x02f (cmpl)
```

That the `(BO, BI)` pair sits in the table is the reason `Terminator::Bc`
carrying `(BO, BI)` — this repo's own precedent, cited by `ARCH_REVIEW` finding
4 as the pattern IR3 should follow — is the shape c2 itself uses.

> **The trap this table sets, recorded because this lane walked into it.**
> Read as a *continuation* of the first table at index `0x298`, it decodes
> tuple opcode `0x30f` as `tdlngi` — a trap instruction — in a function that is
> `return a+b+c`. It is not. Tuple opcodes above `0x297` are **structural
> pseudo-ops with no mnemonic at all** (§2.3), and the second table has its own
> index space. Anyone reading a tuple stream will hit this.

### 2.3 Two disjoint bands, and the flag byte tells you which `[O]`

**Measured over 4 fixtures, 75 real-instruction tuples and 213 structural ones,
zero counterexamples** (`the_opcode_space_is_c2s_own_mnemonic_table`):

* **`+0x9` bit 0 clear** ⇒ the opcode is **above `0x297`** — a structural
  pseudo-op, no mnemonic. Seen: `0x309`, `0x30a`, `0x30b`, `0x30d`, `0x30f`.
  (`0x30f` at category `0x17` is the region terminator the region finder tests
  at `0x10be5d55` — the layout predicts a value the code branches on.)
* **`+0x9` bit 0 set, at `sched0`/`after0`** ⇒ the opcode is a **machine
  opcode** with a mnemonic. **36 of 36.**
* **`+0x9` bit 0 set, before the lowering band** ⇒ **not necessarily.**
  6 counterexamples: `0x2f8`, flagged as a real instruction, past `_last`.

That last bullet **refutes PREREG P0.1 as registered** and is the more useful
statement: *the lowering band is exactly the place where instruction-carrying
pseudo-ops become machine opcodes.* On `mvp_add3` you can watch the single
tuple `0x2f8` become `0x284` (`ret`) across it.

---

## §3 INTERFACE 1 — IL record → tuple

### 3.1 The record layout

The `.ex` side is `WB_READER_FINDINGS.md` §3 and is not restated. What matters
for the correspondence:

| `.ex` field | how it is read | address |
|---|---|---|
| opcode byte | `GetByte`, one per token | `0x10bbc9ab` |
| operand class | `movzx eax,BYTE PTR [ecx+0x10b25e48]` | **`0x10b3d626`** |
| symbol token | `varU` (2 or 4 bytes, bit-15 continuation) → TU symtab | `0x10c1f91b`, `0x10b99977` |
| TYPE word | 1/2/3-byte variable-length integer | `0x10c1fe40` |
| TYPE size index | `(v >> 9) & 7` → 1/2/4/8 bytes | **`0x10b3d5c1`**, stored `0x10b3d5ea` |

The tuple side (`W-STAGETAP-2`, `W-STAGETAP-4`, `W-STAGETAP-5`):

| off | field | conf |
|---|---|---|
| `+0x00` | `next` | `[O]` |
| `+0x04` | **opcode** (§2) | `[O]` |
| `+0x08` | category byte | `[O]` |
| `+0x09` | flags; **bit 0 = real instruction** | `[O]` §2.3 |
| `+0x0a` | `& 0x1f` — **the operand size in bytes**, not a condition code (§3.3) | `[O]` |
| `+0x10` | `prev` | `[R]` `0x10be626c` |
| `+0x28` | operand list **D** — the source side | `[O]` §5 |
| `+0x2c` | operand list **S** — the destination side | `[O]` §5 |

### 3.2 The transform, for the closed subset

There is **no intermediate exposed between the two**: by the time any tap can
see a tuple, selection and the first lowering have already run. So the
"correspondence" documented here is a correspondence between *token counts and
kinds* and *tuple counts and kinds*, not a per-record mapping — and saying so is
the point, because it is exactly why a general decode is expensive.

| `.ex` token | class | tuples produced | derivation |
|---|---|---|---|
| `B9 <varU sym> <TYPE>` — parameter load | `0x18` | **none** | DERIVED: the parameter is already in its ABI register |
| `02` — binary add | `0x00` | **one**, opcode `0x001` (`add`), category `0x0d`, flags `0x01` | DERIVED: n−1 for an n-leaf chain, graded at n = 2, 3, 4 |
| `41 <TYPE>` — return value | `0x01` | **one**, opcode `0x2f8`, category `0x15`, flags `0x01`, plus a four-row structural tail | TRANSCRIBED |
| `3A <varU sym>` — jump to the epilogue | `0x02` | **none** at `/Ox` | DERIVED (the target is the next label) |
| `54 <i32c>` — scope close | `0x0d` | **none** | DERIVED |
| `29 <varU sym>` — epilogue label | `0x02` | **none**; the decode ends here | DERIVED |
| `4F 01 <byte>` — source-line record | `0x0c` | **none** | TRANSCRIBED (one sub-opcode only; every other `0x4F` **refuses**) |

`0x4F`'s payload is a sub-record read by `FUN_10b9761e` off an 8-byte-stride
descriptor table at **`0x10b26268`** and then a ~14-arm switch. Decoding it is
outside this lane, so exactly one sub-opcode is pinned and everything else
refuses — which is why only `mvp_add3` (a one-line definition, no interior line
record) could be traced by hand, and why the multi-line fixtures needed that
one rule to be graded at all.

### 3.3 `+0xa` is a SIZE, not a condition code — and it is `[O]`

`c2host/stagetap.c`'s own comment and `WB_DAGORDER_FINDINGS.md` §2 both call
`+0xa & 0x1f` a condition code. On every real-instruction tuple of every
`int`-typed function traced here it is **4**, and on every structural tuple it
is **0**. Four is the byte width of the operands, and the TYPE word `0x641` that
produced them has size index `(0x641 >> 9) & 7 = 3`, which
`0x10b3d5c1`'s ladder maps to **4 bytes**.

This is registered as PREREG **P1.4** and asserted by the interface-1 test. Its
*discriminating* half — P1.5, a `double` fixture showing `8` and a `short`
fixture showing `2` — **did not run**, so what is established is that the field
is 4 wherever the operands are 4 bytes wide and 0 where there are none, which is
consistent with a size and also with several other readings. **The condition-code
reading is not refuted here.** See §7.

### 3.4 Worked example — `fixtures/cpp/mvp_add3.cpp` `[O]`

`int add3(int a, int b, int c) { return a + b + c; }`, at `/Ox /GS- /c`, a
function the port already emits **byte-exact** (`c2rs diff` → `Port=Match`).

The `.ex` body, from its `4C 4F 11 53` marker:

```
 b9 e3 09  86 41 74      load  sym 0x9e3 (a)   TYPE word 0x641, size index 3 -> 4 bytes
 b9 e4 09  86 41 74      load  sym 0x9e4 (b)
 02                      ADD                    class 00, zero operand bytes
 b9 e5 09  86 41 74      load  sym 0x9e5 (c)
 02                      ADD
 41 86 41 74             return value
 3a e7 09                jump  sym 0x9e7        the epilogue label
 54 02                   scope close, depth 2
 29 e7 09                define label 0x9e7
```

The tuple list at `sched1`, region block 0 — the live tap, verbatim:

```
  idx  opcode   cat  flg  +0xa
    0  00000001  0d   01   04      add
    1  00000001  0d   01   04      add
    2  000002f8  15   01   04      <return-value pseudo-op, no mnemonic>
    3  0000030f  17   00   00      region terminator
    4  00000309  1a   00   00
    5  0000030b  19   00   00
    6  00000309  1a   00   00
```

Two `02` tokens, two `add` tuples. Three `B9` loads, zero tuples. One `41`, one
`0x2f8`. **Row for row, this is what
`the_il_subset_decoder_reproduces_the_tuple_rows` reproduces** from the `.ex`
bytes using c2's own operand-class table — and it reproduces `mvp_two.cpp`'s
`add2` (one `add`) and `add4` (three) with the same rules, so the count is a
prediction and not a transcription of `add3`'s two.

---

## §4 The lowering band, watched

Same function, the tuple list after the lowering band, at `after0` — the
**whole-function** walk, so it also carries the two tuples ahead of the first
region:

```
  opcode   cat  flg
  0000030a  18   00
  0000030d  17   00
  00000001  0d   01     add        <-- unchanged
  00000001  0d   01     add        <-- unchanged
  0000030f  17   00
  00000284  10   01     ret        <-- 0x2f8 became a MACHINE opcode
  0000030b  19   00
  00000309  1a   00
```

One tuple changed opcode; one structural tuple was deleted; the two `add`s and
their operands are untouched. That is the whole lowering band on this shape, and
it is the smallest legible instance of what `ARCH_REVIEW` calls the general
lowering.

---

## §5 INTERFACE 2 — the final tuple order → COFF `.text`

### 5.1 The encoder — `FUN_10bf9f15` @ `0x10bf9f15` `[O]`

**The sole reader of the base-encoding table**, and it is short:

```
0x10bf9f20  eax = tuple[+0x4]                    ; the opcode
0x10bf9f26  esi = tuple[+0x28]                   ; operand list D (sources)
0x10bf9f2c  eax = tuple[+0x2c]                   ; operand list S (destination)
0x10bf9f33  ecx = [esi]                          ; the second source
0x10bf9f3c  ebx = [opcode*4 + 0x10c3a578]        ; THE BASE WORD
0x10bf9f43  edx = [opcode*4 + 0x10c39b18]        ; THE ENCODE FORM
0x10bf9f4d  edx-- ; if (edx > 0x6e) default
0x10bf9f57  jmp  [edx*4 + 0x10bfae2d]            ; a 111-arm jump table
            … the arm ORs the operand fields onto ebx at 0x10bfae19/0x10bfae1b
```

| table | VA | stride | contents |
|---|---|---:|---|
| **base word** | **`0x10c3a578`** | 4 | the 32-bit PPC encoding with every operand field zero |
| **encode form** | **`0x10c39b18`** | 4 | `1 … 0x6f`; the arm index is `form − 1` |
| the arm table | `0x10bfae2d` | 4 | 111 entries |

Spot-checked against the architecture, and every one is right:
`add` → `0x7C000214`, `addi` → `0x38000000`, `addis` → `0x3C000000`,
`b` → `0x48000000`, `bl` → `0x48000001`, `bc` → `0x40000000`,
`cmp` → `0x7C000000`, `lwz` → `0x80000000`, `stw` → `0x90000000`,
`li` → `0x38000000` (= `addi`), `lis` → `0x3C000000` (= `addis`),
`mr` → `0x7C000378` (= `or rA,rS,rS`), **`ret` and `blr` → `0x4C000020`**
(= `bclr`, with no `BO`).

### 5.2 The two arms this lane needs, read instruction for instruction `[R]`

**Form `0x31` — three registers** (`add`, and `add`'s form). Arm `0x10bfa456`:

```
10bfa456  edx = [[esi+0x1c] + 0x28]     ; esi = tuple[+0x28]  -> RA
10bfa459  eax = [[eax+0x1c] + 0x28]     ; eax = tuple[+0x2c]  -> RT
10bfa462  eax = (eax << 5) | edx
10bfa46a  eax = (eax << 5) | [[ecx+0x1c] + 0x28]   ; ecx = [tuple[+0x28]] -> RB
10bfa470  eax <<= 11
10bfa473  -> 0x10bfae19                 ; word = base | eax
```

so **`word = base | (RT << 21) | (RA << 16) | (RB << 11)`**, with each register
read as `operand+0x1c` → `+0x28`.

**Form `0x37` — `ret` / `blr`.** Arm `0x10bfa2a5` is **one instruction**:

```
10bfa2a5  or ebx, 0x2800000             ; BO = 20 ("branch always") at bit 21
10bfa2ab  -> 0x10bfae1b
```

No operand is read at all. `0x4C000020 | 0x02800000 = 0x4E800020` — `blr`.

### 5.3 Where the register actually lives, and why it is not in the tuple

`P_DAG.md` §2's 2026-08-20 correction: a 128-byte window of the tuple record is
**byte-identical across COLOR**. `w-restim` then measured *where* it is:
`op+0x1c` → symbol, `sym+0x08` → `+0x1c` = the **physical** register, in c2's
own numbering where index 1 is `r0` (`WB_REGALLOC_FINDINGS.md` §2). The encoder
reaches the same number by a shorter path — `operand+0x1c` → `+0x28`, which is
the **hardware** number the 5-bit field wants — so

> **`sym+0x08 → +0x1c` == `operand+0x1c → +0x28` + 1.** `[O]`

That relation is what lets §5.4's check run on `w-restim`'s instrument without
this lane touching the tap at all.

### 5.4 Worked example, all 32 bits `[O]`

`mvp_add3`'s `after0` rows, with the operand walk on. `D` is `tuple+0x28`,
`S` is `tuple+0x2c`; the number shown is the physical register in c2's `n = r+1`
numbering, so `hw = n − 1`.

| tuple | D0 | D1 | S0 | → RT | RA | RB | encoded | obj |
|---|---:|---:|---:|---:|---:|---:|---|---|
| `0x001` `add` | `4` | `5` | `0x0c` | `11` | `3` | `4` | `0x7C000214 \| 11<<21 \| 3<<16 \| 4<<11` = **`0x7D632214`** | `7d632214` ✅ |
| `0x001` `add` | `0x0c` | `6` | `4` | `3` | `11` | `5` | = **`0x7C6B2A14`** | `7c6b2a14` ✅ |
| `0x284` `ret` | — | — | — | — | — | — | `0x4C000020 \| 20<<21` = **`0x4E800020`** | `4e800020` ✅ |

`.text` is 12 bytes and there are exactly three real-instruction tuples.
`the_final_tuple_order_reproduces_the_text_words` does this for `mvp_add3`'s one
function and `mvp_two`'s two: **9 words, 32 bits of 32, from the tuple order and
c2's own two tables.**

### 5.5 The site matters, and it is `after0` and not `sched0`

The region tap fires at region-finder **entry** and run 4 has no successor run,
so every `sched0` block is the final schedule's **input**
(`ARCH_REVIEW` finding 1). A byte check built on `sched0` would be grading the
order that goes *into* the last schedule. On these three functions the two
happen to agree — the final schedule is a no-op on a three-instruction body —
which is precisely why the distinction has to be made by construction rather
than by observation. `w-restim`'s `after0` site is what makes §5.4 a statement
about the emitted order.

### 5.6 What is NOT in §5

Relocations. `mvp_add3` and `mvp_two` have **zero** `.text` relocations, so this
lane observed the relocation/label half of the emit seam **not at all**. The
brief asked for it; it is **not delivered**, and §8 prices it. Everything known
about it remains in `LABEL_COUNTER.md` and `P_COFF.md`.

---

## §6 The ratio, and how it relates to `w-restim`'s Probe C

`w-restim` measured on `w5_chain.cpp` that c2 carries **19–22 tuples and 29
regions where the port emits 4–5 instructions**, and concluded there is no
common coordinate between the port's output and c2's tuple stream. That stands
and is not weakened here. Two things are added:

1. **Most of the ratio is bookkeeping, not structure.** On `mvp_add3` the
   region-walk payload is **36 rows across 7 region blocks** for a **three-word**
   function — but the whole-function list at `after0` is **8 rows**, of which
   **3** carry an instruction. The 36 is suffix re-reads (65.1% of the payload,
   `ARCH_REVIEW` §1) times four phases.
2. **At the last observable point the instruction-carrying tuples are in
   bijection with the emitted words** — 3 functions, 9 words, exact. So the
   projection Probe C found undefined is undefined *in the region coordinate*;
   in the **instruction** coordinate at `after0` it is the identity.

Both are true and they answer different questions. A port cannot emit c2's
region trace; a port's instruction sequence is nevertheless comparable to c2's
final tuple list, one row per word, at one site.

---

## §7 PREREG scored

`H` hit · `M` miss · `U` unscoreable (did not run).

| # | prediction | verdict |
|---|---|---|
| P0.1 | every real-instruction tuple carries a machine opcode | **M — REFUTED.** True at `sched0`/`after0` (36/36), false before the lowering band (6 counterexamples). §2.3 carries the repair |
| P0.2 | every structural tuple is above the machine space | **H** — 213/213 |
| P0.3 | P0.1/P0.2 hold on ≥ 4 further fixtures | **H for P0.2, M for P0.1** — 4 fixtures, and the fixtures are what produced the refutation |
| P0.4 | the mnemonic matches the emitted word | **H** — §5.4, via the encoding rather than the string |
| P1.1 | two `02` tokens → two `add` tuples | **H** |
| P1.2 | three `B9` loads → zero tuples | **H** |
| P1.3 | `41` → one non-machine tuple pre-lowering, `0x284` after | **H** |
| P1.4 | `+0xa & 0x1f` is the operand size in bytes | **H on the graded part** (4 on every 4-byte tuple, 0 on every structural one) and **NOT DISCRIMINATED** — see P1.5 |
| P1.5 | a `double` fixture shows 8, a `short` fixture shows 2 | **U — DID NOT RUN.** This is the cell that would have made P1.4 a finding rather than a consistency, and its absence is why §3.3 does not claim the condition-code reading is refuted |
| P1.6 | the token→tuple count is not 1:1 in general | **U** — no constant-operand fixture was graded; §6 supports it from a different direction |
| P2.1 | the final order, restricted to real instructions, is the emitted order | **H** — 3 functions, 9 words |
| P2.2 | `word = base_word[op] \| fields`, tables at `0x10c3a578`/`0x10c39b18` | **H**, and the form values were **wrong in the prereg**: the registered arm indices `0x03`/`0x1e` were read off the wrong jump-table entry; the real forms are `0x31` and `0x37`. Reported as a correction, not folded in silently |
| P2.3 | `ret`/`blr` share `0x4C000020`; the emitted word is base \| BO=20 | **H** — and arm `0x10bfa2a5` is literally `or ebx,0x2800000` |
| P2.4 | the same on ≥ 4 further fixtures | **U — DID NOT RUN.** 3 functions across 2 fixtures, not 4 further ones |
| P2.5 | the registers are not in the tuple; a tuple-only check cannot supply them | **H**, and it stopped mattering: `w-restim`'s operand walk supplies them, so §5.4 is 32 bits of 32 rather than the masked check this lane planned |
| P3.1 | 0 TUs converted, 0 obj bytes moved | **H** — §9 |
| P3.2 | match 26 / mismatch 0 / fnbyte-exact 35894 unmoved | **H** — §9 |
| P3.3 | at least one prediction is refuted | **H** — P0.1, plus P2.2's form values |

**Score: 13 H · 3 M · 3 U.** The three misses are all this lane's own readings
being too confident, and two of them (P0.1, P2.2's form values) were caught by
the graded code rather than by re-reading — which is the argument for writing
the code at all.

---

## §8 What the integration row still costs after this

`ARCH_REVIEW` finding 3 priced the two prerequisites at **3–9 engineer-months**
before CEILING §5's ~5:1 optimism calibration. This lane does not re-price the
whole row; it moves specific parts from black box to documented, and the honest
form of the update is a **lower bound**, per lane kind.

**Attribution first, because two lanes landed on the same day.** Rows marked
*(w-restim)* below are `w-restim`'s work, not this lane's; they are in the table
because the integration row's price depends on the whole state of knowledge and
not on who moved it. This lane's own contribution is the opcode numbering, the
encoder and its two tables, the two arms, the reconciliation of the two register
paths, and the interface-1 subset.

| part of the row | before | after this lane | why |
|---|---|---|---|
| **the tuple record's shape** | unknown beyond 5 bytes | **documented**, 8 fields + 2 operand lists | §3.1, `W-STAGETAP-2`; the operand and function-record layouts are *(w-restim)*, `W-STAGETAP-4/-5` |
| **the opcode numbering** | *"erased, the map is not a function"* | **one named table + one trap named** | §2 |
| **which tuples are instructions** | unknown | **`+0x9` bit 0, 288 rows, 0 counterexamples** | §2.3 |
| **tuple → PPC word** | black box | **`base_word[op] \| form-fields`, 2 tables, 1 function, 9 words graded** | §5 |
| **the register fields** | *"not in the tuple"* | **two pointer hops** *(w-restim)*, **and the two paths reconciled** | §5.3 |
| **IL token → tuple** | black box | **legible for a 7-token closed subset, on a shape with no calls, no branches, no memory, no constants** | §3 |
| **relocations / labels at the emit seam** | black box | **still black box — 0 cells** | §5.6 |
| **the other ~110 encode forms** | black box | **still black box — 2 of 111 arms read** | §5.2 |
| **selection and the lowering band** | black box | **still black box** — one pseudo-op watched turn into one machine opcode | §4 |

**The estimate does not move down by much, and the reason is worth stating
plainly.** Interface 2 is genuinely small — a real general lowerer from a final
tuple list to `.text` words is `0x10bf9f15`'s 111 arms plus relocations, which
is *weeks*, not months, and this lane has 2 of the 111 and none of the
relocations. **Interface 1 is where the 3–9 months lives**, and nothing here
touches it: the subset decoded is 7 tokens on a body with no call, no branch, no
memory reference and no constant, and the two rules that carry the shape of the
answer (`0x41` → `0x2f8` + a four-row tail, and `4F 01` being three bytes) are
**transcriptions from one snapshot, not derivations**.

So, as a lower bound under a ~5:1 calibration:

* **Interface 2 (general lowering): the review's share of the estimate should
  come down.** Two tables and a 111-arm switch is a bounded, enumerable job
  against a table that can be read exhaustively out of the image in an
  afternoon, and it is gradeable per-word against a real obj — which is the
  cheapest grading surface this project has. Call it the smaller half by a
  wide margin. **This lane declines to put a number on it**: it graded 2 arms
  and 9 words, and a 3-cell measurement extrapolated to 111 arms is exactly the
  kind of estimate `#1767` refuses.
* **Interface 1 (general op-level decode): unchanged, and if anything the case
  for the upper half of 3–9 months is stronger.** There is **no intermediate
  exposed between the IL token stream and the machine tuple list** (§3.2) —
  selection has already happened at every point a tap can observe. A general
  decode is therefore not "read the records"; it is "reproduce selection", and
  the reader half that *is* pure decoding was already measured to convert **0
  of 48** frontier functions on its own (`WB_READER_FINDINGS.md` §4).
* **The row's real risk is not either interface. It is that the two do not meet
  in the middle** — and §6 is the evidence: the port's coordinate system and
  c2's are related by the identity at `after0` and by nothing at all in the
  region coordinate, so an integration built on region-level parity has no
  target while one built on instruction-level parity at `after0` has a
  well-defined one that nobody has built.

**One concrete, cheap thing this lane recommends and did not do**: dump all 111
encode-form arms and the `0x10c39b18` histogram over the workload's emitted
opcodes. That converts "2 of 111 arms" into a coverage number, costs an
afternoon, and is a prerequisite for anyone putting a real figure on interface
2's half of the row.

---

## §9 Gate

The lane's `crates/` delta is **one new test file**. No `crates/` source is
touched, no `c2host/` source is touched (the tap extension this lane wrote was
deleted in favour of `w-restim`'s — see the commit that says so), so the
required-zero metrics cannot move and the gate is a no-regression control.
Counts in [`../rungs/2026-08-21-w-ildecode.md`](../rungs/2026-08-21-w-ildecode.md) §Gate.
