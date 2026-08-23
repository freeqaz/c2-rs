# `P_OPATTR` — the per-opcode attribute byte `0x10c3afd8`, its class field, and the dispatch tail that reads it

> **PROVENANCE — DISASSEMBLY-DERIVED.** See [`../DISCLOSURE.md`](../DISCLOSURE.md).
> Nothing here may enter `crates/` without a `DISCLOSURE.md` row naming the
> address it came from.

Lane `w-tailread`, board **#3460**–**#3463**. Prereg:
[`../WB_TAILCLASS_PREREG.md`](../WB_TAILCLASS_PREREG.md). Grade:
[`../WB_TAILCLASS_FINDINGS.md`](../WB_TAILCLASS_FINDINGS.md). Tooling:
[`../scripts/dump_tailclass.py`](../scripts/dump_tailclass.py),
[`../scripts/probe_selfmove.py`](../scripts/probe_selfmove.py).

**Image.** `compilers/X360/16.00.11886.00/c2.dll`, sha256
`c80981c015166effecc71ad8112d5577a065b2300891dfdb02b9c13787a66258` — verified
before any address here was read. Every address is reproducible from the image
alone; the tooling disassembles the PE directly.

**Provenance legend** ([`README.md`](README.md) §2): `[R]` read, not confirmed
against any obj; `[O]` obj-confirmed; `[I]` inferred. **`[R]` means "the
instructions were read correctly", never "this is what c2 does."**

---

## 0. The answer, in one screen

[`P_EXPAND.md`](P_EXPAND.md) §1.2 named this table as its largest unread hole
and called it *"a per-opcode attribute BYTE table whose low 3 bits are an
opcode class"*, reached by *"767 opcodes"*. Both halves of that sentence needed
work, and the corrections are the finding:

> **The table is not new, and the 767 is not a number.** `0x10c3afd8` is a
> **lossless stride-1 replica of the mnemonic table's `flags` field**, which
> board **#2044**/**#2106**/**#2206** already documented and decoded four bits
> of, in 2026-08-09. Its **low three bits** are genuinely undecoded anywhere,
> and they are an **operand-shape class**: load / store / move / sign-extend.
> The "767 opcodes" is the whole domain of R6's walk, not a measured set — it
> tracks the walk's bound exactly, and reports **1024** if you set the bound to
> `0x400`. And the tail it feeds **emits no instruction at all**: it attaches
> an *operand*.

| question | answer | tier |
|---|---|---|
| what is the table? | the mnemonic table's `flags` byte, denormalised to stride 1; **664 of 664 entries identical** | `[R]` |
| its extent? | **`0x298` = 664 entries**, opcodes `0x000..0x297` — derived, not assumed (§1.1) | `[R]` |
| what do the low 3 bits mean? | an **operand-shape class**: 1 = move/SPR-transfer, 2 = load, 3 = store, 4 = sign-extend, 0 = other (§2) | `[R]` |
| how many consumers? | **38** sites image-wide; the dispatch tail is one of them | `[R]` |
| how many of the 767 are "expanded"? | **the question is malformed** — 767 is the walk's domain (§3) | `[R]` |
| does the dispatch tail expand anything? | **no. It emits zero words and attaches an operand** (§4) | `[R]` |
| does c2 delete instructions here? | **yes**, and the record had no delete primitive until now: `0x10bd5516`, **401 callers** (§5) | `[R]` |
| does c2 emit redundant self-moves? | **`mr r8,r8` yes — 3,792.** `fmr`: 0 of 32,569 (§6) | **`[O]`** |

---

## 1. The table  `[R]`

### 1.1 Extent, derived rather than assumed

The mnemonic table is `0x10b1b260`, stride 12. The **extended-mnemonic** table
begins at `0x10b1d180`. Those two facts pin the first table's length with no
guessing:

```
0x10b1b260 + N*12 = 0x10b1d180   =>   N = 0x298 = 664
0x10c3afd8 + 0x298 = 0x10c3b270  <-  and a SECOND byte table begins exactly there
```

So the attribute table is **664 bytes, covering opcodes `0x000..0x297`, and it
stops.** `_last` sits at index `0x295` (`DISCLOSURE.md` W-MID-1), so
`0x001..0x294` is the machine opcode space and `0x295..0x297` is the sentinel
tail.

### 1.2 It is the mnemonic table's flags field, byte for byte

```
attr[op] == (u8) *(u32 *)(0x10b1b260 + op*12 + 8)   for 664 of 664 entries, 0 differ
entries whose flags word exceeds one byte:          0   (so the replica is lossless)
```

**This table is therefore not a new fact and must not be cited as one.** Board
**#2040** lists it beside the mnemonic and base-word tables; **#2044** decodes
`0x08` = `Rc=1`, `0x10` = has an `Rc` sibling, `0x20` = writes `XER[CA]`,
`0x40` = reads `XER[CA]`; **#2106** and **#2206** turn `0x10` into the
record-form rewrite rule; and `rungs/2026-08-09-wb-select2.md:67` says in one
line *"The same byte is exposed as an array at `0x10c3afd8`, indexed by machine
opcode."*

> **An independent corroboration of #2044, from a direction that lane did not
> use.** Bit `0x08` has **n = 135** and bit `0x10` has **n = 135** — exactly
> equal. If `0x08` marks "is a record form" and `0x10` marks "has a record form
> at `opcode+1`", the two populations must be the same size, and they are. That
> is a check #2044 could have run and did not.

### 1.3 Bits, with their populations

| bit | n | meaning | source |
|---|---:|---|---|
| `0x07` | — | **the operand-shape class (§2)** | **this lane** |
| `0x08` | 135 | `Rc=1`: this opcode *is* a record form | #2044 |
| `0x10` | 135 | has a record-form sibling at `opcode+1` | #2044 / #2106 |
| `0x20` | 41 | writes `XER[CA]` | #2044 |
| `0x40` | 26 | reads `XER[CA]` | #2044 |
| `0x80` | **0** | **unused across all 664 entries** | this lane |

---

## 2. The class field — the part nothing in this repo decoded  `[R]`

`attr[op] & 7`. Five of eight values are populated; **5, 6 and 7 never occur.**

| class | n | members |
|---:|---:|---|
| **0** | 516 | everything else |
| **1** | 35 | **move / special-register transfer** — `fmr fmr. mcrf mcrfs mcrxr mfcr mffs mffs. mfmsr mfocrf mfspr mfsr mfsrin mftb mtcrf mtfsb0 mtfsb0. mtfsb1 mtfsb1. mtfsf mtfsf. mtfsfi mtfsfi. mtmsr mtmsrd mtmsrdee mtocrf mtspr mtsr mtsrin slbmte mr mr. vmr vmr128` |
| **2** | 55 | **load** — every `l*` form, `lbz` through `lwzx`, including the VMX `lv*`/`lv*128` |
| **3** | 52 | **store** — every `st*` form, `stb` through `stwx`, including `stv*`/`stv*128` |
| **4** | 6 | **sign-extend** — `extsb extsb. extsh extsh. extsw extsw.` |

The partition is by **operand shape**, not by expansion behaviour: it separates
the opcodes that carry a memory operand (2, 3), those that are a pure register
copy (1), and those that are a width conversion (4).

### 2.1 An independent cross-check: every peephole arm is class-pure

The class decode above could be a pattern this lane fitted. It is not, and the
image contains its own control. `FUN_10c182b4` dispatches through a **different
byte table** at `0x10c184a8` (`P_EXPAND.md` §5), read by a different lane from
different bytes. Partition its 17 active arms by *this* table's class field
(`dump_tailclass.py --table`):

| arm | n | classes present | |
|---:|---:|---|---|
| 0 / 1 / 2 | 38 / 1 / 11 | `{0}` | three-operand ALU, `cmpi`, `cmpli`… |
| **3 / 4 / 5** | 2 / 2 / 2 | **`{4}`** | `extsb` `extsh` `extsw` ± record — **all six class-4 opcodes, and only those** |
| **6** | 1 | **`{1}`** | `fmr` |
| **7 / 8 / 9** | 4 / 5 / 5 | **`{3}`** | `stb` / `sth` / `stw` families |
| 10 / 11 / 12 / 13 | 108 / 26 / 4 / 2 | `{0}` | VMX, `rlandi` |
| **14 / 15 / 16** | 1 / 1 / 1 | **`{1}`** | `mr` `mr.` `vmr` |
| 17 | 445 | `{0:322, 1:30, 2:55, 3:38}` | the **do-nothing default** |

> **Arms that are not class-pure, excluding the default: 0.** Every arm that
> *does* something serves exactly one class. And the counts close: class 1 is
> 4 named + 30 on the default + `vmr128` (outside the index bound) = **35**;
> class 3 is 14 named + 38 = **52**; class 4 is 6 named + 0 = **6** — every
> sign-extend has a dedicated arm. **All 55 class-2 opcodes are on the
> do-nothing arm**: the peephole never rewrites a load.

Two unrelated tables in the image, one taxonomy.

### 2.2 The 38 consumers

`dump_tailclass.py --consumers`. **28 bit probes, 10 class probes.** Two of the
38 index a **compile-time-constant** opcode — `ds:0x10c3afe4` is `attr[0xc]`,
i.e. *"does `addic` have a record sibling"*, folded by MSVC into a fixed
address at `0x10c0b44a` and `0x10c0b5d5`.

The ten class probes: `0x10bfccd2`, `0x10bff584`, `0x10c09699`, `0x10c0e30b`,
`0x10c0f760`, `0x10c0f7af`, `0x10c11f5c`, `0x10c17825`, `0x10c17caa`,
`0x10c1825a`. Six test `class == 2`, two test `class == 1`, one tests
`class == 3`, and one (`0x10bff584`) compares against a **register**, so its
class is a parameter.

> **`P_EXPAND.md` §1.2 calls this "the dispatch tail's table".** It is not the
> tail's table; the tail is **one of 38** consumers and one of ten that read the
> class. The table is a general per-opcode property of the machine model.

---

## 3. ⛔ "767 opcodes reach the tail" is the walk's domain, not a measurement

`P_EXPAND.md` §1.1 tabulates `opcodes reaching the dispatch TAIL 0x10c0e30b =
767` and §7 warns that *"reaches the tail is not is unchanged"*. The warning is
right; the number is not a number.

`0x2ff` **is** 767, and `dump_expansion.py`'s walk domain is exactly
`1..OPMAX`. Re-running its own `opcode_tree` with the bound raised:

| OPMAX | discriminated opcodes | arm bodies | "reach the tail" |
|---|---:|---:|---:|
| `0x2ff` (R6's) | 69 | 29 | **767** |
| `0x400` | **70** | 29 | **1024** |
| `0x600` | **70** | 29 | **1536** |

**The tail count equals OPMAX at every setting.** It says the tail is reachable
carrying an un-narrowed opcode interval — a fact about the abstract
interpretation, not about c2. Six of R6's ten "shared fall-through bodies"
report `767` for the same reason.

**Consequence for the brief that funded this lane:** *"convert 767 opcodes reach
the tail into opcode X is / is not expanded"* is not a task that can be
completed, because there was never a 767-element set. What replaces it is §4.

### 3.1 …and R6's arm map is short by one, at `0x302`

The same table shows it: raising the bound gains **exactly one** opcode.

```
10c0e2ed:  mov ecx,eax
10c0e2ef:  sub ecx,0x2fe
10c0e2f5:  je 0x10c0e494      <- opcode 0x2fe
10c0e2fb:  dec ecx
10c0e2fc:  je 0x10c0e487      <- opcode 0x2ff
10c0e302:  sub ecx,0x3
10c0e305:  je 0x10c0e479      <- opcode 0x302   ** absent from P_EXPAND §3 **
10c0e30b:  <the tail>
```

`opcode_bound()` takes the largest **literal** in a `cmp`/`sub`/`add`/`lea` and
adds one, giving `0x2fe + 1 = 0x2ff`. `0x302` is reachable only by following the
`dec` / `sub 3` chain and is never a literal, so the bound excluded the very arm
that lies past it. The corrected count is **70 discriminated opcodes**, still
over 29 arm bodies (`0x10c0e479` falls through into `0x10c0e4a4`, already an
arm).

---

## 4. What the dispatch tail actually does — and it is not expansion  `[R]`

```
10c0e30b:  mov cl,BYTE PTR [eax+0x10c3afd8]
10c0e311:  and cl,0x7
10c0e314:  cmp cl,0x2
10c0e317:  je  0x10c0e40f          <- class 2: LOADS
10c0e31d:  cmp eax,0x281
10c0e322:  je  0x10c0e40f          <- and `lea`, explicitly
10c0e328:  cmp cl,0x3
10c0e32b:  jne 0x10c0e4ab          <- everything else: the exit join, untouched
           (falls through)         <- class 3: STORES
```

It is a **three-way** classifier, not the two-way one §1.2 describes. `lea`
(`0x281`) is **class 0**, which is why the class alone does not catch it and the
opcode is named explicitly — the predicate being computed is *"is this a memory
reference"*, i.e. **load-or-lea**.

### 4.1 Both live classes converge, and the shared body attaches an operand

Class 2 walks the instruction's `+0x28` operand list; class 3 walks `+0x2c`;
both filter on the operand kind byte `[edi+8]` and on
`FUN_10c123b9(opcode, x)` — which is `mov eax,[ecx*4+0x10c39b18]`, the
**encode-form table** of [`P_ENCODE.md`](P_ENCODE.md) §3, so the filter is a
*form* predicate. Survivors reach one shared body at `0x10c0e398`:

```
10c0e398:  ecx = 1;      call 0x10b26ecd   -> allocate a SET object
10c0e3a2:  edx = 0xd;    call 0x10b26eda   -> OR element 0xd into it
10c0e3ac:                call 0x10bd3a44   -> wrap it as an OPERAND node (kind 0xb, tag 0x2ac)
10c0e3b7:                call 0x10bd7108   -> append it to the instruction's +0x2c list
```

**Not one of those four is an instruction constructor**, and the one that
inserts into a list inserts into the **operand** list. `0x10bd7108` reaches
`0x10bd6e89`/`0x10bd3ce2`/`0x10bd3d7a`, all operand machinery.

So: **the tail's word delta is zero for every opcode that reaches it.** What it
does is annotate memory-referencing instructions with an extra definition.

### 4.2 The tail reads the table OUT OF ITS EXTENT, and it is benign

The tail applies **no bound check**. The table has `0x298` entries; the switch
dispatches opcodes up to at least `0x302`. Opcodes `≥ 0x298` therefore index
into the **second** table at `0x10c3b270`.

Measured over `0x298..0x2ff`, the bytes landed on decode to classes `{0, 1, 4}`
and **never to 2 or 3** — so the tail takes the exit join for all of them and
the out-of-extent read **changes no behaviour**. It is a latent hazard, not a
live bug, and a port must not "fix" it into a bound check without noticing that
the observable result is identical.

### 4.3 A caution on the minting evidence

`dump_tailclass.py --tail` reports a **minimum hop count** to an instruction
constructor: **8** for all three tail bodies, **1** for three control arms R6
scores as minting. That is evidence, not proof.

**The transitive form of the question is useless and was discarded.** "Can a
constructor be reached from here" answers *yes* for the tail and for every
control alike, because c2's call graph is strongly connected through its arena
and diagnostic machinery. That is the identical defect to §3's 767. The direct
reading of the five callees in §4.1 is the stronger argument and is what this
page rests on.

---

## 5. The delete primitive — which this record did not have  `[R]`

[`P_EXPAND.md`](P_EXPAND.md) §2 establishes the **mint** side: 16 constructors,
all reaching the list-insert wrapper `0x10bd5732`, which calls the
doubly-linked insert-after `0x10bd3824`. There is an exact dual, and no
document here names it:

```
0x10bd3824  INSERT-AFTER          0x10bd5516  UNLINK
  eax = ecx->next                   eax = ecx->prev ; edx = ecx->next
  edx->next = eax                   eax->next = edx
  ecx->next = edx                   eax = ecx->next ; edx = ecx->prev
  ...                               eax->prev = edx
```

`0x10bd5516` is the **delete primitive**, and the asymmetry in how much each is
used is the point:

```
0x10bd3824  insert-after   207 direct calls
0x10bd5516  unlink         401 direct calls     <- deletion is the COMMONER operation
```

### 5.1 So `P_EXPAND.md` §3's word counts are one-sided

`dump_expansion.py --words` counts constructor calls. **It has no notion of
deletion**, so an arm that removes an instruction scores `0..0` — identical to
an arm that does nothing. Inside `FUN_10c0d57e` there is exactly one call to the
delete primitive, at **`0x10c0e4a6`**, in the body at **`0x10c0e4a4`** — which
§3 lists as *"the no-op join"*, `0..0`, opcodes `fmr, mr, 0x2e5, 0x2f7`.

**Its true word delta is −1, not 0.** It is the *delete* join. Two further arms
fall into it: `0x10c0e494` (opcode `0x2fe`) after setting the opcode to `0x297`
(`_last`), and `0x10c0e479` (opcode `0x302`, the arm §3.1 recovers). `0x10c0dfdc`
— §3's `fmr, mr` row, also `0..0` — reaches it too, via
`call 0x10bd2d83 / jmp 0x10c0e4a4`.

### 5.2 The mint set's closure argument is sound, and rests on a premise the page does not state

`P_EXPAND.md` §2 obtains its 16 constructors by **inverting the call graph** on
`0x10bd5732`. That is only closed if the wrapper's address is never taken.
Measured:

```
0x10bd5732  (the wrapper)   address-taken   0 sites     <- so the inversion IS closed
0x10bd3824  (the primitive) address-taken 506 sites     <- passed as a CALLBACK
```

The primitive beneath it is handed as a function pointer to the emitter helpers
(`0x10bd7c10`, `0x10bd790e`, `0x10bd575d`, …) at **506** sites, two of them
inside this very switch (`0x10c0db6b`, `0x10c0e20e`). So the closure holds for
the wrapper and **would not** hold one level down. The premise that makes §2
sound is the `0` on the first line, and it is supplied here rather than assumed
— the same premise `P_LABEL.md` states explicitly for its allocator.

Spot-checked: the callback path's node builder `0x10bd575d` does **not** set
`+9` bit 0, so it is not minting instructions and R6's counts are not disturbed
by it. That is one function checked, not the population.

---

## 6. ⛔ `[O]` — c2 DOES emit redundant self-moves, 3,792 of them

`P_EXPAND.md` §5 records the peephole's arms and this lane read the one arm no
document had — **arm 6, `fmr`, `0x10c1838b`** — plus its class-1 siblings.

### 6.1 What arm 6 is  `[R]`

A 12-byte thunk `mov ecx,esi / call 0x10c16fbd / jmp 0x10c18448`. The handler
**`FUN_10c16fbd`** (191 B, 1 caller, 7 callees) is a **redundant-move eliminator
with a copy-propagation fallback**:

```
src = instr[+0x28] ; dst = instr[+0x2c]
if (dst->[0x1c] == src->[0x1c])          <- SAME REGISTER
      clear bit 0x40 on both operands' descriptors
      tail-call 0x10c16cde               -> and the instruction is UNLINKED
else  0x10c16a46 / 0x10bfc132 / 0x10c16ba5 / 0x10c16c66   <- copy propagation
```

Its siblings: arm 14 `mr` → `0x10c16d83`, arm 15 `mr.` → `0x10c1707c`, arm 16
`vmr` → `0x10c16e59`. Each move form has its own handler.

### 6.2 What the corpus says  `[O]`

`probe_selfmove.py` over the capture cache. The claim under test — *"c2 emits no
`mr rX,rX` and no `fmr fX,fX`"* — is what §6.1 licenses if `[R]` were allowed to
mean "this is what c2 does".

| corpus | non-self move forms | **self-moves** |
|---|---:|---:|
| 6,000 objs | 29,785 | **0** |
| 30,000 objs | 50,439 | **298** |
| **120,000 objs**, 176,969 `.text` sections, 1,726,709 words | **135,218** | **3,792**, in **1,206** objs (1.00 %) |

**REFUTED.** And note it survived 6,000 objs — a lane that had stopped there
would have published a confirmation.

**But the refutation is narrower than the claim, and the split matters:**

| form | non-self | self |
|---|---:|---:|
| `fmr` | 32,569 | **0** |
| `mr.` | 150 | **0** |
| `mr` | 102,499 | **3,792** |

**Arm 6's own opcode is clean.** Across 32,569 emitted `fmr` there is not one
self-move, so on this corpus the `fmr` arm's deletion is consistent with the
output. Every violation is `mr` — arm 14, `0x10c18373` → `FUN_10c16d83`, a
handler this lane did **not** read. The `[O]` result therefore refutes the
*generalisation* to all class-1 opcodes, and leaves §6.1's reading of arm 6
itself unchallenged.

The self-moves are genuine emitted instructions, checked three ways: **no
relocation covers those offsets**, the section carries `CNT_CODE|EXECUTE`, and
the surrounding words decode as a coherent prologue. The unit was compiled
**`/Ox`**, so "the peephole was disabled" does not explain them.

They are also unmistakably an **idiom**: **3,792 of 3,792 name `r8`** and no
other register, and they sit adjacent to branches (`op18`), typically three
before a `bl` and three after. **This lane does not settle what the idiom is**
and deliberately declines to guess in the record.

> **What is refuted is the LICENCE, not the read.** Arm 6 plainly implements
> redundant-move deletion; §6.1 stands as `[R]`. What cannot be said is *"c2
> emits no self-move"*. Whatever produces these runs after the peephole, or is
> not subject to it, or satisfies none of the handler's guards. **This is the
> `.bss`-bump failure mode** (`C2_MAP_METHOD.md` §7) — caught inside the same
> lane that did the reading, by the obj check the `[R]`/`[O]` split exists to
> demand.

---

## 7. `0x10b1d180` — SETTLED, and the contradiction was a malformed question

`P_EXPAND.md` §6 and board **#3432** record a stride-16 table
`{char *name, u32 machine_opcode, u32 BO, u32 BI}` that decodes perfectly, and
**deliberately did not publish** its index mapping, because under the obvious
hypothesis `op = 0x298 + j` the opcode `0x2f0` decodes to the trap mnemonic
`twlti` while `0x2f0`'s arm demonstrably calls the prologue driver. R6 was right
not to publish. The reason the two cannot be reconciled is that **there is no
mapping to publish.**

### 7.1 Nothing indexes the table by an opcode

Its **only** referencing function is `FUN_10c0174b`, which holds all three
references in the image (`0x10c0175e`, `0x10c01774`, `0x10c01790`). It is a
**name lookup**: it starts at row 1 (`xor ebp,ebp / inc ebp`), strides by
`shl eax,4`, string-compares each row's name, and terminates on the `_last`
string `0x10b19ce4` — the twin of `FUN_10c00900`, which does the same to the
*first* table with `imul eax,eax,0xc`.

> **`P_EXPAND.md` §6 says its "only two references are inside `FUN_10c00900` and
> `FUN_10c0174b`."** `FUN_10c00900` references `0x10b1b260`, the **first** table.
> This table has **one** referencing function, not two.

`ebp` is a **search cursor**. It is never an opcode, and no instruction in the
image computes `0x298 + j`.

### 7.2 The row index becomes an opcode by reading field `+4`, at one address

The sole caller, at `0x10c0298d`, makes it explicit:

```
10c0297e:  call 0x10c00900          ; look the name up in the FIRST table
10c02986:  cmp eax,ebx / jge ...    ; found -> that result IS the opcode
10c0298d:  call 0x10c0174b          ; not found -> try the EXTENDED table
10c02995:  cmp eax,ebx / je  ...    ; not found either -> error
10c02999:  shl eax,0x4              ; row index * 16
10c0299c:  mov ecx,[eax+0x10b1d184] ; <-- FIELD +4.  THIS is the opcode.
```

The mapping from a row to an `instr[1]` opcode is **field `+4`**, read by one
instruction at one address. There is no index arithmetic to recover.

### 7.3 And the table can never name a pseudo-op

Over its 122 rows: the `+4` field has **min `0x0`, max `0x295`, 23 distinct
values**, and **0 rows** carry an opcode `≥ 0x298`. So no row of this table can
ever denote `0x2f0`. The premise of the contradiction is empty.

### 7.4 Where `twlti` came from

Reproduced exactly. Row **88** is `twlti`, real opcode `0x19d` (`twi`), BO 0,
BI 16 — and `0x2f0 - 0x298 = 88`. R6's `[R]` reading was arithmetically correct;
the hypothesis it was testing is computed nowhere in c2.

For completeness, the *other* trap — indexing the **first** (stride-12) table
past its extent, which is board **#3357** — gives `0x2f0 → twige` and
`0x2f4 → twlgt`, different garbage again. `dump_tailclass.py`'s `mnemonic()`
refuses above `0x298` for exactly this reason.

**Verdict: SETTLED, and the mapping is published — as "there is none, and here
is what replaces it."** The decline criterion in the prereg (§5.1) required
naming what *does* index the table before publishing; §7.1 and §7.2 do.

---

## 8. What this page does NOT claim

* **§2's class names are labels for a partition, not strings in the image.**
  "load", "store", "move", "sign-extend" are this lane's names for classes 2,
  3, 1 and 4; c2 contains no such text. The *membership* is measured, exactly.
* **§4's "emits zero words" is `[R]`** and rests on reading five callees, not on
  the hop-count number, which §4.3 says explicitly is evidence only.
* **§5's 506 address-takings are counted, not classified.** One of the callback
  paths was spot-checked for the real-instruction bit; the population was not.
* **§5.1 does not re-score `P_EXPAND.md` §3.** It names one arm whose count is
  wrong in sign and the reason the instrument cannot see it. A corrected word
  table is a follow-up, not this page.
* **§6's refutation is about the obj, not about which pass owns it.** An obj is
  post-everything and cannot attribute an absence — or a presence — to the
  peephole rather than to selection.
* **The `r8` idiom in §6.2 is unexplained here, on purpose.** A plausible story
  exists; it is not evidence, and guessing it in the record is how the next lane
  inherits a wrong fact.
* **No `crates/` byte was changed** and no `DISCLOSURE.md` row is owed —
  nothing here was adopted.

## 9. Follow-ups this read hands over, ranked

1. **What is `mr r8,r8`?** §6.2. 3,792 instances, one register, branch-adjacent,
   at `/Ox`. It is a real, reproducible, unexplained emission and it is
   **obj-visible**, so it is cheap to chase and does not need the disassembly.
2. **Re-score `P_EXPAND.md` §3 with a signed word delta.** §5.1. The instrument
   needs a delete oracle beside its mint oracle; both primitives are now named.
3. **The second byte table `0x10c3b270`.** Same extent, `0x298` entries, its own
   two consumers bound-checked at `0x295` with default `0x64` — a per-opcode
   *value* (a cost or latency), never read here.
4. **`0x10c0e479` / opcode `0x302`.** §3.1 recovers the arm; what `0x302` *is*
   was not chased, and it is absent from `P_ILRECORD.md`'s minting arms.
