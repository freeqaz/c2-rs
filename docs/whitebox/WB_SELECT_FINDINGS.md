# WB-I `wb-select` — how c2 selects PPC instructions

> **PROVENANCE — DISASSEMBLY-DERIVED.** Obtained by statically disassembling
> Microsoft's `c2.dll` — the exact image pinned in
> [`C2_MAP_METHOD.md`](C2_MAP_METHOD.md) §0, sha256 verified at the top of this
> lane as `c80981…6258`. It is **navigation only** until a row is added to
> [`DISCLOSURE.md`](DISCLOSURE.md). **The obj is the sole judge**: §7 grades
> every reading here against real `c2.dll` under wibo, and §7.6/§8 record what
> the objs refuted.

Lane `wb-select` (WB-I), campaign 2026-08-08, run 2026-08-09. PREREG:
[`WB_SELECT_PREREG.md`](WB_SELECT_PREREG.md), committed in `18a2ec45`
**before the first grep of `~/ghidra-projects/export/c2/`** — one `ls -la` of
that directory ran earlier, to confirm the export exists at all, and is
disclosed here because it is the only thing that touched the path before the
freeze. Grid:
[`grids/wb-select/select_grid.cpp`](grids/wb-select/select_grid.cpp) +
[`frozen.tsv`](grids/wb-select/frozen.tsv), committed in `3624bb93` **before
the first `cl.exe` of the grid**. Calibration:
[`grids/wb-select/calib.cpp`](grids/wb-select/calib.cpp), unscored, §6.1.

WB-D §9.1 and WB-H §9 item 3 both end on the same sentence: *the pattern set —
instruction selection — is the only "no" left.* This lane went and read it.

---

## 0. The answer in one screen

**c2's instruction selection is table-driven, and the table is in the image.**

There is **one opcode field** — `instr[1]`, the second word of every tuple — and
**one opcode space**. Values `≤ 0x298` are PPC machine instructions and
pseudo-ops; values `> 0x298` are the machine-independent IR operators.
*Selection is the act of overwriting that field from the upper half with a value
from the lower half*, in place, on a list that is never reordered (WB-D §4).

Three image-resident arrays share the lower half's index — c2's machine opcode
number — and together they are the Rosetta stone that makes every selection arm
readable:

| VA | stride | contents |
|---|---|---|
| **`0x10b1b260`** | 12 | `{char *mnemonic; int operand_format; int flags}`, 665 entries, `_first`…`_last` |
| **`0x10c3a578`** | 4 | the **32-bit big-endian PPC encoding skeleton** |
| **`0x10c3afd8`** | 1 | a property byte (`0x20` = writes `XER[CA]`) |

Above them sit **sixteen per-type opcode tables** installed by `FUN_10c04cb9` @
`0x10c04cb9` — move, load, load-indexed, store, store-indexed, negate, add,
subtract, multiply, divide, compare-immediate, compare-register, and four
`-QVMX128` variants — each 26 ints indexed by the operand type. **`cmpw` vs
`cmplw` (#1788) is a table lookup**, not a branch, and so is `divw` vs `divwu`.

The per-operator dispatch is `FUN_10c0f882` @ **`0x10c0f882`**, a **46-arm
jump-table switch** on the tuple opcode. Most arms are three lines long.

And the one place where selection is genuinely an *algorithm* rather than a
table is the case the brief named: a **relational used as a value**. That is
tuple opcode `0x2ea`, and c2 races **two branchless expanders against each
other by cost** and takes the cheaper, ties to the second:

| | VA | idiom |
|---|---|---|
| profitability | `FUN_10c1b315` @ **`0x10c1b315`** | if-convert only when `2·cost < 20` at `/O1` |
| driver | `FUN_10c1b517` @ **`0x10c1b517`** | `min(costA, costB)`, **ties to B** |
| **A** | `FUN_10c1ac5c` @ **`0x10c1ac5c`** | `[li] · subfic\|subfc · subfe · [rlwinm] · [addi]` |
| **B** | `FUN_10c1af2d` @ **`0x10c1af2d`** | `[addi] · [cntlzw] · cntlzw · rlwinm 27,31,31 · [xori] · [addi]` |

`cost` is the **word count**, and `500` means *infeasible* — which the
profitability gate then reads as "do not if-convert at all", so the same
routine that produces WB-D's four-word carry idiom also decides when there is a
branch instead.

**The grid: 10 of 12 primary predictions hit, 6 of 10 word-exact including
every register.** Four cells were predicted *instruction-word for
instruction-word, sight unseen*. Two missed and are retracted in §7.6.

---

## 1. The opcode space, and the tables that decode it (deliverable 1)

### 1.1 Finding the table

`strings.tsv` types only a handful of the mnemonics (`addis` at `0x10b1b1f0`
with zero xrefs). The array was found by scanning the raw image for a pointer
to that string: exactly one, at file offset `0x1a708` = VA `0x10b1b308`. Walking
outward on a 12-byte stride recovers a table of **665 entries** at
`0x10b1b260`, bracketed by the sentinels `_first` / `_last` and alphabetical.

Alphabetical means it is a *name lookup* table, and its only readers are in the
inline-asm parser (`0x10c00913`, `0x10c029dd`, `0x10c032b8` — `p2\ppc\inlnasm.c`
band). That would have made it a dead end, except that a **second** array
shares its index.

Scanning the image for the PPC encoding skeletons `0x7c000110` (`subfe`) and
`0x7c000034` (`cntlzw`) lands in a dense `.data` block. Aligning it against the
mnemonic table pins the base:

```
index(`addze`)  = (0x10b1b35c − 0x10b1b260)/12 = 21
0x10c3a578 + 4·21 = 0x10c3a5cc  =  0x7c000194  =  addze         ✓
index(`and`)    = 25 → 0x10c3a5dc = 0x7c000038 = and            ✓
index(`addzeo`) = 23 → 0x10c3a5d4 = 0x7c000594 = addze with OE  ✓
```

Three independent alignments on instructions whose encodings are fixed by the
architecture. **`0x10c3a578` is machine-opcode → encoding**, and therefore
`0x10b1b260`'s index *is* c2's machine opcode number.

The `flags` word at `+8` of the mnemonic table decodes cleanly:

| bit | meaning | evidence |
|---|---|---|
| `0x08` | `Rc = 1` (sets `CR0`) | every `.`-suffixed entry has it, no other |
| `0x10` | has an `Rc` sibling | `add` `0x10` / `add.` `0x08` |
| `0x20` | writes `XER[CA]` | `addc` `0x30`, `subfic` `0x20`, `srawi` `0x30` |
| `0x40` | reads `XER[CA]` | `adde` `0x70`, `subfe` `0x70`, `addze` `0x70` |

and the same information exists as a **byte array at `0x10c3afd8`**, indexed by
the same opcode number, which is what the *selectors* read (§2.2).

**This answers the brief's "record-form vs plain forms" question directly.** A
record form is not a separate selection decision: it is the **next opcode
number** (`add` `0x001` / `add.` `0x002`, `rlwinm` `0x133` / `rlwinm.` `0x134`,
`cntlzw` `0x032` / `cntlzw.` `0x033`), and `flags & 0x10` marks which opcodes
have a sibling to switch to.

### 1.2 The opcode space is ONE space

`FUN_10c0d57e` @ **`0x10c0d57e`** is the 3899-byte in-place expansion switch
WB-D §4 saw. Its discriminants are `0x0b`…`0x0d` (`addi`/`addic`/`addic.`),
`0x21` (`bc`), `0x2e`/`0x30` (`cmpi`/`cmpli`), `0x270` (`li`) — *machine*
opcodes — **and** `0x2f0` / `0x2f4` at `0x10c0e266`…`0x10c0e291`, which are the
prologue/epilogue *tuple* opcodes WB-D quoted.

So the field is one space:

> **`instr[1] ∈ [1, 0x298]` is a PPC machine instruction or pseudo-op (the
> 665-entry table). `instr[1] > 0x298` is a machine-independent IR operator.
> Selection rewrites the field downward, in place.**

**A CORRECTION to WB-D §4, offered gently.** WB-D wrote that the expansion
switch's `0x2f4`/`0x2f0` arms "call the prologue driver `0x10bff95c` via
`0x10c216f5`/`0x10c21719`". The call sites are real and are inside
`FUN_10c0d57e`, but the *same* two opcodes are also arms of `FUN_10c0f882` (§2),
where they dispatch to `0x10bfee89`/`0x10bfee9a`. WB-D's walk is right; what was
missing is that there are **two** switches on the one field, one before
selection and one after, and the numbers belong to the IR half of a space WB-D
had no reason to know was shared.

---

## 2. The pattern set (deliverable 2)

### 2.1 The dispatch — `FUN_10c0f882` @ `0x10c0f882`

A 46-arm jump-table `switch (instr[1])`. The arms this project's IL actually
carries:

| tuple op | operator | handler |
|---:|---|---|
| `0x2af` | assign / copy | `0x10c053e7`, else `0x10c053a8` (`li`) |
| `0x2b4` | `~` | `0x10c09d9e` (`not`) |
| `0x2c5`, `0x327`, `0x328` | `+` | `0x10c0634b` |
| `0x2c6`, `0x329`, `0x32a` | `−` | **`0x10c064cb`** |
| `0x2c7` | `*` | `0x10c067f1` (`mulli` / table) |
| `0x2ca` | `&` | `0x10c0711d` (`rlandi` / `and`) |
| `0x2cb` | `\|` | `FUN_10c0718f(t, 0x11d or, 0x121 ori)` |
| `0x2cc` | `^` | `FUN_10c0718f(t, 0x26a xor, 0x26c xori)` |
| `0x2cd` | `/` | `0x10c068ee` |
| `0x2ce` | `%` | `0x10c06eb1` |
| `0x2d4` | compare → CR | **`0x10c0eb17`** |
| `0x2dc` | call | `0x10c0e4b9` |
| `0x2dd`, `0x2e7` | conditional branch | `0x10c0b690` (`bc`) |
| `0x2de` | computed branch | `0x10c07899` (`mtspr ctr` + `bctr`) |
| **`0x2ea`** | **relational as a VALUE** | **`0x10c1b517`** (§3) |
| `0x2eb` | intrinsic | `0x10bf7c59` — `p2\ppc\cgintrin.c` |
| `0x2f0` / `0x2f4` | prologue / epilogue | `0x10bfee9a` / `0x10bfee89` |

Note that `|` and `^` are handled by **one shared routine parameterised with two
opcode numbers**. That is the shape of the whole file: the arms are thin, and
the knowledge lives in the tables.

**PREREG P1.2 scored a hit and it matters**: `cgintrin.c` is *not* the operator
selector. It is one arm of `FUN_10c0f882` (`0x2eb`), the intrinsic arm.

### 2.2 The sixteen per-type opcode tables — `FUN_10c04cb9` @ `0x10c04cb9`

> **CORRECTION 2026-08-09, lane `wb-tables` (WB-J),
> [`WB_TABLES_FINDINGS.md`](WB_TABLES_FINDINGS.md) §1.** The installer was
> counted instruction by instruction: it contains **17 stores into 13 distinct
> pointer slots** (`DAT_10c6fdac`…`DAT_10c6fddc`), the last four overwriting
> four of the thirteen under `DAT_10c2e978`. **"Sixteen" is a real count — of
> the table *bodies* contiguous in `.data` at `0x10c38f30`…`0x10c395b0`, stride
> `0x68`** — but the list below is *twelve named plus four `-QVMX128`* and
> therefore **omits the thirteenth installed table, convert/widen at
> `0x10b1fd08`** (`extsb`/`extsh`/`extsw`/`mr`), which is in `.text`, not in
> the `.data` block. A port adopting this list drops the narrowing operator.
> The full 17-body enumeration with all 26 slots decoded, and the slot map that
> shows only 17 of 26 are ever live, are in `WB_TABLES_FINDINGS.md` §1.3–§1.4.
> Nothing else in this section is corrected; every entry below that this lane
> re-decoded is right.

```
DAT_10c6fddc = 0x10c38f30   move        DAT_10c6fdc8 = 0x10c392d8   negate
DAT_10c6fdd8 = 0x10c38f98   load        DAT_10c6fdc4 = 0x10c39340   add
DAT_10c6fdd4 = 0x10c39068   load-x      DAT_10c6fdc0 = 0x10c393a8   subtract
DAT_10c6fdd0 = 0x10c39138   store       DAT_10c6fdbc = 0x10c39410   multiply
DAT_10c6fdcc = 0x10c391a0   store-x     DAT_10c6fdb8 = 0x10c39478   divide
                                        DAT_10c6fdb4 = 0x10c394e0   compare, immediate
                                        DAT_10c6fdb0 = 0x10c39548   compare, register
```

plus four `-QVMX128` alternates (`0x10c39000`, `0x10c390d0`, `0x10c39208`,
`0x10c39270`) selected on `DAT_10c2e978`. Each table is 26 ints indexed by
`FUN_10bd7c10(type)`. Decoded through §1's mnemonic table:

| type slot | move | load | store | add | mul | div | cmp-reg | cmp-imm |
|---:|---|---|---|---|---|---|---|---|
| 1 | `mr` | `lbz` | `stb` | `add` | `mullw` | `divw` | `cmp` | `cmpi` |
| 2 | `mr` | `lhz` | `sth` | `add` | `mullw` | `divw` | `cmp` | `cmpi` |
| 4 | `mr` | `lwz` | `stw` | `add` | `mullw` | `divw` | `cmp` | `cmpi` |
| 6 | `mr` | `ld` | `std` | `add` | `mulld` | `divd` | `cmp` | `cmpi` |
| 7 / 8 / 10 | `mr` | `lbz`/`lhz`/`lwz` | `stb`/`sth`/`stw` | `add` | `mullw` | **`divwu`** | **`cmpl`** | **`cmpli`** |
| 12 | `mr` | `ld` | `std` | `add` | `mulld` | `divdu` | `cmpl` | `cmpli` |
| 13 / 14 | `fmr` | `lfs`/`lfd` | `stfs`/`stfd` | `fadds`/`fadd` | `fmuls`/`fmul` | `fdivs`/`fdiv` | `fcmpu` | `fcmpu` |
| 16 / 17 | `mr` | `lwz` | `stw` | `add` | `mullw` | `divwu` | `cmpl` | `cmpli` |
| 20…23 | `mr` | `lbz`…`ld` | `stb`…`std` | `add` | `mullw`/`mulld` | `divwu`/`divdu` | `cmpl` | `cmpli` |
| 25 | `vmr` | `lvx` | `stvx` | `vaddfp` | — | — | — | — |

> **#1788 is closed as a mechanism.** The comparison's signedness is not
> recomputed and not inferred: it is `DAT_10c6fdb0[type]` versus
> `DAT_10c6fdb4[type]`, and the *only* thing the selector decides is
> register-form versus immediate-form. WB-D §5's operand-nibble→class map
> `0x10b022cc` is a different, coarser projection of the same type word.

### 2.3 The compare selector — `FUN_10c0eb17` @ `0x10c0eb17`

Reads the tuple's type word `*(u16*)(t+10)`, takes the **nibble** `>>12`, and:

* nibble `1` → the constant must fit a **signed** 16-bit field
  (`v & 0xffff8000 == 0` or `== 0xffff8000` with the high word `-1`);
* nibbles `2`, `3`, `4` → **unsigned** 16-bit fit (`v & 0xffff0000 == 0`);
* fit ⇒ `instr[1] = DAT_10c6fdb4[idx]` (`cmpi`/`cmpli`); otherwise the constant
  is forced into a register and `instr[1] = DAT_10c6fdb0[idx]` (`cmp`/`cmpl`).

The CR field is assigned literally at the tail:

```
*(void**)(op2 + 0x1c) = &DAT_10c2f088 + ((-(FUN_10c0b300(t) != 0) & 0xfffffffa) + 0x49) * 0x60;
```

`0x49` is **`cr6`** on WB-D §2's register table; `0x49 − 6 = 0x43` is **`cr0`**.
So `cr6` is the default and `cr0` is the exception, gated by `FUN_10c0b300`
which this lane did **not** read — recorded as `unknown` in `W-SELECT.tsv`
rather than guessed. This *replaces* WB-D §7.5's retracted `cr0` claim with a
site, and cell `wbs_s5` re-confirms `cr6` black-box.

### 2.4 Arithmetic, shifts and logicals — one-to-one, with two exceptions

**WB-H §9 item 3's narrowing HOLDS (PREREG P4.1, hit).** `+`, `−`, `*`, `&`,
`|`, `^` and the shifts are one-to-one lowerings through §2.2's tables. The
selectors `0x10c0634b` (add) and `0x10c064cb` (subtract) are mirror images and
their only real work is the **carry chain**: they pick `adde`/`subfe` when a
carry-in operand is present, `addc`/`subfc` when a carry-out is consumed,
`addi`/`subfic` for a 16-bit constant, and `addze`/`subfze`/`addme`/`subfme`
for the `0` and `−1` cases — then, at `0x10c06689`:

```
if (DAT_10c2ecf0 == 0 && ((&DAT_10c3afd8)[instr[1]] & 0x20) && no carry consumer)
        attach a dead XER[CA] def
```

i.e. the property byte of §1.1 is read to keep the carry register honest.

The two exceptions, both obj-confirmed in §7:

* **`/` by a power of two** (`FUN_10c068ee` @ `0x10c068ee`) is `srawi` + `addze`
  — two words for one operator, coupled through `XER[CA]`, and the `addze` is
  the round-toward-zero correction. The same routine mints the two `twi`
  division traps and the `neg`/`rldicl`/`rlwinm` cases.
* **`&` with a constant** goes through the `rlandi` pseudo-op (`0x26e`), whose
  expansion `FUN_10c0a2e2` @ `0x10c0a2e2` chooses between `rlwinm`, `rlwinm.`,
  `andi.`, `andis.`, `rldicl` and `li`+`and`.

---

## 3. THE VALUE-PRODUCING RELATIONAL — the flagship (deliverable 2, continued)

Tuple opcode `0x2ea`. This is WB-D's `x < 10u ? 1 : 2` four-word carry idiom,
and it is not a peephole: it is a **costed race between two expanders**.

### 3.1 The driver — `FUN_10c1b517` @ `0x10c1b517`

```
if (type nibble == 5)                       FUN_10c194b8(t);           // bool-typed
else if (FUN_10c1b2fa(DAT_10c2e2f4) &&      // enabled
         one compare operand is constant 0) FUN_10c1a908(t);           // zero fast path
else {
    a = FUN_10c1ac5c(t, 0);                 // COST only
    b = FUN_10c1af2d(t, 0);                 // COST only
    if (b <= a) FUN_10c1af2d(t, 1); else FUN_10c1ac5c(t, 1);   // EMIT — TIES TO B
}
```

The `param_2` flag is a **dry-run / emit** switch: the identical routine walks
the identical decision tree twice, first returning a cost and then emitting. A
port can copy that structure exactly.

### 3.2 Strategy A — the CARRY idiom, `FUN_10c1ac5c` @ `0x10c1ac5c`

**Normalisation.** The relation byte lives at `t+0x34`. If the *compare's* type
nibble is `2` it is first remapped through the table at **`0x10b189a4`**, which
maps the signed family `3,4,5,6` onto the unsigned family `7,8,9,10`. Then a
small state machine drives everything to the single canonical form
**`local_c >u local_8`** (relation `8`):

| relation | code | action |
|---|---:|---|
| `==` | 1 | swap the two RESULT values, become `!=` |
| `!=` | 2 | **require** the compare's 2nd operand to be constant `0`; become `>u` |
| `<u` | 7 | swap the COMPARE operands; become `>u` |
| `>u` | 8 | canonical |
| `<=u` | 9 | swap the two RESULT values; become `>u` |
| `>=u` | 10 | swap the COMPARE operands, become `<=u`, then as above |
| `<`,`>`,`<=`,`>=` **signed** | 3,4,5,6 | **`return 500`** — infeasible |
| the float relations | `0x0b`…`0x12` | `return 500` |

**Cost, which is the word count.** With `base` = the second result operand and
`delta` = first − second (after any swap):

```
cost  = 2
if (local_c is a constant)              cost = 3      // an extra `li`
if (delta != -1)                        cost += mask_cost(delta)
if (base  != 0)                         cost += 1
```

**Emission.**

```
[ li     rT, local_c ]                      if local_c is a constant
  subfic rD, local_c, K                     if local_8 is a 16-bit constant
  subfc  rD, local_c, local_8               otherwise
  lcarry (0x28c)  ->  subfe rD, rD, rD      == XER[CA] − 1
[ rlandi rD, 0, delta ]                     unless delta == −1
[ addi   rD, rD, base ]                     unless base == 0
```

The whole thing is one identity. `subfic rD,rA,SIMM` computes `SIMM − rA` and
sets `CA` iff there is no borrow, i.e. iff `SIMM ≥u rA`, i.e. iff
**`¬(local_c >u local_8)`**. `subfe rD,rD,rD` is `¬rD + rD + CA = CA − 1`, so:

> **mask = `−1` exactly when the canonical relation is TRUE, `0` otherwise, and
> the result is `base + (mask & delta)`.**

`delta == −1` is the case where `mask & delta == mask`, which is why the `and`
disappears — and that, not a special case, is why WB-D's `x < 10u ? 1 : 2`
comes out in exactly four words.

`0x28c` / `0x28d` (`lcarry`, `lcarry.`) are pseudo-ops whose encodings in
`0x10c3a578` are `subfe`'s own `0x7c000110` / `0x7c000111`.

### 3.3 Strategy B — the CNTLZW idiom, `FUN_10c1af2d` @ `0x10c1af2d`

Feasible only when the compare's second operand is the constant `0` **or** the
relation is `==`/`!=`; and the two result values must differ, and either
`|delta| == 1` or one of them is `0` and the other a power of two. Otherwise
`return 500`.

```
[ addi rD, x, −K   or   subf ]     bring the comparison to zero
[ cntlzw rD, rD ]                  an EXTRA one when the relation is signed (bVar5)
  cntlzw rD, rD
  rlandi rD, 27, 1                 == rlwinm rD,rD,27,31,31 == srwi rD,rD,5
[ xori rD, rD, 1 ]  [ neg rD, rD ]  [ addi rD, rD, base ]  [ rlandi log2(delta) ]
```

The double `cntlzw` is the signed trick: `cntlzw(x) == 0` iff `x < 0`, and
`cntlzw(0) == 32`, so `cntlzw(cntlzw(x)) >>> 5` is exactly the sign bit.

### 3.4 The profitability gate — `FUN_10c1b315` @ `0x10c1b315`

Called from `p2\misc.c` at `0x10b813f1` — i.e. **the branch-to-value conversion
is a machine-independent decision that asks the machine-dependent expanders
what it would cost**:

```
c = FUN_10c1ac5c(t, 0);  if (!favour_speed) c *= 2;   if (c < 20) return true;
c = FUN_10c1af2d(t, 0);  if (!favour_speed) c *= 2;   return c < 20;
```

(`DAT_10c2e310` is the favour-speed word wb-memcpy found at `0x10c2e310`; under
favour-speed the threshold is `c < 10`, the same predicate.) A `500` from both
expanders is therefore `1000 ≥ 20` and **no if-conversion happens at all** —
c2 emits a real `cmpwi` and a conditional branch. §7 cell `wbs_s5` is that
prediction, and it landed.

### 3.5 The relation-code space, and the four remap tables

Four 20-byte tables sit adjacent at `0x10b18990`, `0x10b189a4`, `0x10b189b8`,
`0x10b189cc`. Three of them are unambiguously **relation** maps, from their
fixed points and their involutions:

| VA | map | fingerprint |
|---|---|---|
| `0x10b189a4` | **signed → unsigned** | `3,4,5,6 → 7,8,9,10`; `1,2` fixed |
| `0x10b189b8` | **commute (swap operands)** | `3↔4, 5↔6, 7↔8, 9↔10`; `1,2` fixed; `0x0b…0x0e → 0` |
| `0x10b189cc` | **negate** | `1↔2, 3↔6, 4↔5, 7↔0x0a, 8↔9, 0x0b↔0x0c, 0x0f↔0x10` |

from which the relation codes fall out:

| 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 10 | 0x0b…0x12 |
|---|---|---|---|---|---|---|---|---|---|---|
| `==` | `!=` | `<` | `>` | `<=` | `>=` | `<u` | `>u` | `<=u` | `>=u` | the ordered/unordered float forms |

`0x10b18990` (`3,4,5,6` fixed, `7,8,9,10 → 3,4,5,6`) is the **signedness-erasing**
member of the same family, but it is indexed by `type & 0x1f` at three sites
including WB-H's `mtctr` guard. **This lane does not resolve whether the two
19-value spaces are one enum**, so WB-H's "size table, must be 4 or 2" gloss
is left standing and flagged `unknown`, not corrected. Naming it as an open
question is the honest outcome; §9 lists it.

### 3.6 The zero-operand fast path — `FUN_10c1a908` @ `0x10c1a908`

When one compare operand is the constant `0`, the driver skips the cost race
and dispatches through a **20-arm switch on the relation code** to one of about
twenty small expanders in `0x10c198d2`…`0x10c1a838`, each with a second variant
chosen when the other operand is `1` or `−1`. `FUN_10c19936` @ `0x10c19936` is
the `subfe`-based member of that family. This lane located it, named it, and
**did not enumerate its twenty arms** — §9 counts that as unread ground.

---

## 4. Idiom recognisers that fire on a COMBINATION

The brief asked specifically for these. Three exist and are readable:

1. **The relational-as-value machinery of §3 itself.** It is not one operator:
   it fires on the *pair* (a relational, a two-way select over two constants),
   and its output depends on **both result constants** through `base` and
   `delta`. `x > 7u ? 12 : 4` and `x > 7u ? 1 : 2` produce different word
   counts from the same comparison.
2. **The carry chain in `0x10c0634b` / `0x10c064cb`.** `a + b + carry` is not
   two operators plus a fixup; the presence of a carry *operand* selects
   `adde`/`subfe` outright, and `0x10c3afd8`'s `0x20` bit then decides whether
   a dead `CA` def has to be manufactured.
3. **`/` by a power of two** → `srawi` + `addze`, where the second instruction
   consumes the first's `XER[CA]`. One C operator, two coupled words.

What is **not** an idiom recogniser, contrary to PREREG P2.5: no `(a<<k)+b`
folding and no `x != 0 → cntlzw` peephole was found as a *pattern match*. The
`cntlzw` form is a *strategy*, chosen by cost, not a rewrite rule. P2.5 scored
a miss and is retracted.

---

## 5. What the port already models, restated against this reading

`c2-core` emits integer add-chains, tail calls, one framed non-leaf call, and
four transcribed body shapes. Against §2's tables that is: the `mr`/`add`/`addi`
slots of two of the sixteen tables, at one type. **None of §3 exists in the port
at all**, and §3 is what a `bool` costs.

---

## 6. THE OBJ-CHECK — FROZEN BEFORE THE FIRST `cl.exe` OF THIS GRID

Source: [`grids/wb-select/select_grid.cpp`](grids/wb-select/select_grid.cpp),
sha256 `69affbacc6a67a939bc6b2d7e0c97393f980bfad34982a421c8588b5d37d005c`.
Predictions: [`grids/wb-select/frozen.tsv`](grids/wb-select/frozen.tsv),
sha256 `d05b4cc1531bbb6c59beea144cbbcd47e4bb26ac2ac99fb7141ba2f72f3fd6fa`.
Both committed in `3624bb93` **before** the grid's first compile; the run is
`work/wb-select/run/grid.obj` (not committed — it is an obj).

12 cells, one COMDAT each, `wibo cl.exe /nologo /c /GR /O1 /Oi /EHsc /Gy`
(WB-D's workload mode plus `/Gy`). Two graded columns, scored separately:
**primary** = the mnemonic sequence (or, for the two cells where block order
would have been WB-H's subject, a stated class); **secondary** = the exact
instruction words including every register.

Six rivals were frozen: `R-LC1`/`R-LC2` (does a peephole fold `subfe`+`addi`
into `li`+`subfze`?), `R-M1`/`R-M2` (`rlwinm` or `andi.` for a contiguous
mask?), `R-SB1`/`R-SB2` (is `x < 0` as a bool one word or three?). The
separation assertion and the "cells the port does not emit" floor (asserted 4,
actual 11) are in `frozen.tsv` and were checked before the run.

### 6.1 The calibration pass, and what it changed

[`grids/wb-select/calib.cpp`](grids/wb-select/calib.cpp) (6 cells) was compiled
**first and is unscored**, because wb-inline's v1 grid was refuted by its own
cells. It shares **no relation and no constant** with the graded grid.

**Full disclosure, because it matters to how much §7 is worth.** The PREREG
(P6.1) said the calibration would be read with `--no-disasm`, i.e. sizes only.
`scripts/gt_dump.py --no-disasm` **still prints the raw words**, so this lane
saw the calibration cells' instruction words. What it saw and what it did:

* `wbk_1` (`x <= 3u ? 7 : 9`) came out `subfic r11,r3,3 / subfe r11,r11,r11 /
  rlwinm r11,r11,0,30,30 / addi r3,r11,7`. This **confirmed the §3.2 template
  the disassembly had already produced** — the derivation of `mask = CA − 1`,
  `base`, `delta` and the cost-is-word-count identity is above in this document
  and was written from `FUN_10c1ac5c` before anything was compiled.
* `wbk_2` (the `if` spelling of `wbk_1`) is **byte-identical** to `wbk_1`. The
  grid therefore does not waste a cell on the spelling.
* `wbk_4` (`a >= b ? 1 : 0`) came out `li r10,-1 / subc r11,r3,r4 /
  subfze r3,r10` — a shape `FUN_10c1ac5c` does **not** contain. That is the
  entire reason rival `R-LC2` exists, and `R-LC2` was frozen as a live
  possibility rather than dismissed.
* `wbk_3` (a signed relational) came out six words with `eqv`/`srawi`-free
  arithmetic that neither expander in §3 produces, which is why cell `wbs_s5`
  was written to test the *profitability* claim rather than a word sequence.

The honest statement of what this costs: **§7's carry-idiom cells are not
"blind"** — the template was corroborated by `wbk_1` before the graded cells
were frozen. What §7 still tests, and what calibration could not have given, is
the *cost function*, the *normalisation table*, the *A-vs-B race*, the *tie
rule*, and the *500-means-branch* claim — none of which is visible in any obj.

---

## 7. RESULTS

```
-- wbs_s1   li 11,10 / subc 11,3,11 / subfe 11,11,11 / addi 3,11,2 / blr
-- wbs_s2   subc 11,3,4 / subfe 11,11,11 / addi 3,11,2 / blr
-- wbs_s3   subfic 11,3,7 / subfe 11,11,11 / rlwinm 11,11,0,28,28 / addi 3,11,4 / blr
-- wbs_s4   cntlzw 11,3 / rlwinm 11,11,27,31,31 / xori 11,11,1 / addi 3,11,5 / blr
-- wbs_s5   cmpwi 6,3,10 / li 3,1 / bclr 12,24 / li 3,2 / blr
-- wbs_s6   cntlzw 11,3 / cntlzw 11,11 / rlwinm 11,11,27,31,31 / xori 11,11,1 / addi 3,11,1 / blr
-- wbs_b1   li 11,10 / subc 11,3,11 / subfe 11,11,11 / clrlwi 3,11,31 / blr
-- wbs_b2   subc 11,4,3 / subfe 11,11,11 / clrlwi 3,11,31 / blr
-- wbs_b3   srwi 3,3,31 / blr
-- wbs_k1   srawi 11,3,3 / addze 3,11 / blr
-- wbs_k2   clrlwi 3,3,24 / blr
-- wbs_k3   subc 11,3,4 / subfe 11,11,11 / clrlwi 11,11,31 / add 3,11,5 / blr
```

| cell | primary (mnemonic sequence / class) | secondary (exact words + registers) |
|---|---|---|
| `wbs_s1` | **HIT** — `li ; subc ; subfe ; addi` | miss (predicted `r10` for the `subc` dest, c2 reuses `r11`) |
| `wbs_s2` | **HIT** | **HIT — word for word** |
| `wbs_s3` | **HIT** | **HIT — word for word**, all four words |
| `wbs_s4` | **HIT** | **HIT — word for word** |
| `wbs_s5` | **HIT** (class) | not predicted |
| `wbs_s6` | **MISS** (§7.6) | not predicted |
| `wbs_b1` | **HIT** | miss (same one register as `s1`) |
| `wbs_b2` | **HIT** | **HIT — word for word** |
| `wbs_b3` | **MISS** (§7.6) | miss |
| `wbs_k1` | **HIT** | miss (registers only: c2 routes through `r11`) |
| `wbs_k2` | **HIT** | **HIT — word for word** |
| `wbs_k3` | **HIT** | **HIT — word for word** |

**Primary: 10 of 12. Secondary: 6 of 10 predicted word-exact.** PREREG P6.2
registered "at least 2 cells will MISS" — exactly 2 did.

### 7.1 What the carry reading actually predicted, and got

`wbs_s3` is the cell worth stating in full, because every number in it came out
of `FUN_10c1ac5c` and none of it out of an obj:

| predicted | emitted |
|---|---|
| `subfic r11,r3,7` `0x21630007` | `0x21630007` |
| `subfe r11,r11,r11` `0x7d6b5910` | `0x7d6b5910` |
| `rlwinm r11,r11,0,28,28` `0x556b0738` | `0x556b0738` |
| `addi r3,r11,4` `0x386b0004` | `0x386b0004` |

The mask `28,28` is bit `0x8`, which is `delta = 12 − 4`; the `addi` immediate
is `base = 4`; there is no `li` because `local_c` is the register operand; and
there are four words because `cost = 2 + mask + base = 4`. `wbs_s2`, `wbs_b2`,
`wbs_k2` and `wbs_k3` are the same story with different arithmetic.

### 7.2 The rivals, decided

* **`R-LC1` beats `R-LC2`.** No cell folded `subfe`+`addi` into `li`+`subfze`.
  `wbs_s2` (`base = 2`, `delta = −1`) is the cell that would have shown it and
  emitted the literal `FUN_10c1ac5c` sequence. The calibration cell `wbk_4`'s
  `subfze` shape therefore comes from somewhere else — most plausibly
  §3.6's zero-operand table, since `wbk_4`'s result pair is `{1,0}` — and is
  **not** a peephole over strategy A. Recorded as unread ground, not explained.
* **`R-M1` beats `R-M2`, 5 for 5.** A contiguous mask is always `rlwinm`,
  never `andi.` — c2 does not clobber `CR0` for a value.

  > **CORRECTION 2026-08-09, lane `wb-tables` (WB-J),
  > [`WB_TABLES_FINDINGS.md`](WB_TABLES_FINDINGS.md) §3.5.** The first sentence
  > holds and survives 17 further cells. **The second is wrong.** `andi.` is
  > exactly what a **non-contiguous** 16-bit mask gets, and c2 clobbers `CR0`
  > for a value without hesitation: cell `m3_split16` (`x & 0x8001`) emits
  > `andi. 3,3,32769`, and `c_m_101`, `c_m_f0f0`, `k_plain_2nd` and
  > `k_101_bias` do the same. This grid contained no non-contiguous mask, so
  > its five cells could not have seen it. The rule is in
  > `WB_TABLES_FINDINGS.md` §3.3 (rule S) and is graded 6/6 there.
* **`R-SB1` beats `R-SB2`** on `wbs_b3`. §7.6.

### 7.3 The A-vs-B race, and the tie rule

`wbs_s4` (`x == 0 ? 5 : 6`) was frozen as a **4–4 tie** with the prediction that
**B wins**, from `if (costB <= costA)`. B won, word for word. That single cell
is the only black-box evidence in this project that the tie-break exists, and
it is worth what it cost: no obj can distinguish "ties to B" from "B was
cheaper" unless the costs are computed independently first, which is exactly
what §3.2/§3.3 make possible.

### 7.4 `wbs_s5` — the 500 claim, and `cr6` again

`int x < 10 ? 1 : 2` is a signed relational against a non-zero constant: A
refuses (relation `3`), B refuses (no zero operand, relation not `==`/`!=`), so
`FUN_10c1b315` sees `1000` and declines. c2 emitted

```
cmpwi 6, 3, 10 ; li 3,1 ; bclr 12,24 ; li 3,2 ; blr
```

— `cmpwi` **on `cr6`** (§2.3), a conditional return (`bclr 12,24` = `bltlr cr6`),
and no carry or `cntlzw` anywhere. Every conjunct of the frozen class
prediction held. It also re-confirms `cmpwi` (signed) from the type table for
an `int`, which is #1788 at the obj.

### 7.5 The one that is nearly free: `wbs_k1`

`x / 8` → `srawi r11,r3,3 ; addze r3,r11`. Mnemonic sequence exactly as read
from `FUN_10c068ee`; the registers differ from the prediction only because c2
routes the intermediate through `r11` rather than computing in `r3`.

### 7.6 THE TWO MISSES, stated as misses

**`wbs_b3` — RETRACTED.** Predicted (rival `R-SB2`, from `FUN_10c1af2d`'s
`bVar5` arm): `cntlzw ; cntlzw ; rlwinm 27,31,31`, three words. c2 emitted
**one** word, `srwi r3,r3,31`. The reading of strategy B is not wrong about
strategy B — `wbs_s6`, the *same* comparison with the result pair `{1,2}`
instead of `{0,1}`, did take the double-`cntlzw` form. What is wrong is the
claim that strategy B is what handles `x < 0` as a **bool**. It is not:
`(int)(x < 0)` has type nibble `5` and goes to `FUN_10c194b8` @ `0x10c194b8`
(§3.1's first branch), an 890-byte routine this lane **located but did not
read**. `R-SB1` wins and `R-SB2` is retracted.

**`wbs_s6` — MISS on a conjunct.** Predicted "exactly two `cntlzw`, one
`rlwinm 27,31,31`, no carry, **4 or 5 words including `blr`**". The mechanism
clauses all held; the word count is **6**. The extra word is the `xori 11,11,1`
that `FUN_10c1af2d`'s `bVar3` arm emits, which this lane's cost trace did not
carry through the `bVar15 == 3` normalisation. A prediction with a false
conjunct is a miss, so it is scored as one; the useful residue is that **§3.3's
emission list is right and this lane's cost arithmetic for the signed arm is
not**.

### 7.7 The register column, and whose rule it vindicates

Three of the four secondary misses are one and the same error: this lane
predicted a *fresh* register (`r10`) where the previous value's live range had
already ended, so `r11` was free again. WB-D §3.4's rule — *minimum cost over
the interference-allowed candidates, ties to the earliest of
`r11, r10, …, r3, r31, …`* — gives the **emitted** answer in all three. The
miss is this lane's, not WB-D's, and every cell in this grid is consistent with
§3.4. Selection → order → registers held again, on 12 more cells.

---

## 8. PREREG SCORE

| # | claim | outcome |
|---|---|---|
| P0.1 | the floor: a pattern-set reading survives a frozen check on an idiom the port does not emit | **HIT** — 10/12 primary, 11 of 12 cells outside the port |
| P0.2 | the carry idiom is selected directly, not peepholed from a compare | **HIT** — `FUN_10c1ac5c` never mints a compare opcode |
| P0.3 | judgment comes out "yes", pattern set < 120 rules | **HIT**, §9 |
| P1.1 | selection is at least partly table-driven | **HIT** — §1, §2.2; stronger than registered |
| P1.2 | `cgintrin.c` is **not** the operator selector | **HIT** — it is arm `0x2eb` of `FUN_10c0f882` |
| P1.3 | the per-operator selection is in `p2\ppc\lower.c` | **HIT** — `0x10c0f882` is in the `lower.c` band |
| P1.4 | the final expansion switch is a later pass, dispatched by a jump table, and its table gets named | **PARTIAL** — `FUN_10c0d57e` named, but it is a **binary decision tree**, not a jump table, and it spans both halves of one opcode space (§1.2). Registered as partial. |
| P1.5 | an image-resident mnemonic table indexed by the machine opcode exists and is the Rosetta stone | **HIT** — `0x10b1b260` + `0x10c3a578` + `0x10c3afd8` |
| P1.6 | the arithmetic tuple opcodes are dense in one band | **HIT** — `0x2c5`…`0x2d4` contiguous |
| P2.1 | hand-written `switch` + `if`-ladders, no BURG cost table | **HIT** for the per-operator arms; **but** §3 *is* a cost model, so registered as a half-miss and the half is recorded |
| P2.2 | signed-16 fit for `addi`/`cmpwi`, unsigned-16 for the logicals | **HIT** — `FUN_10c0eb17`'s two fit tests, by type nibble |
| P2.3 | `*` by a constant strength-reduced below a threshold | **NOT TESTED** — no cell; withdrawn, not scored |
| P2.4 | `/` by a constant is a magic multiply | **NOT TESTED** for non-powers of two; the power-of-two case is `srawi`+`addze` (`wbs_k1`) |
| P2.5 | an idiom recogniser fires on `(a<<k)+b` and on `x!=0 → cntlzw` | **MISS** — the `cntlzw` form is a costed *strategy*, not a rewrite; no shift-add folding found. Retracted (§4). |
| P3.1 | carry-setting subtract + `subfe rD,rD,rD` mask + one fixup | **HIT**, word-exact on four cells |
| P3.2 | the idiom is unsigned-only; signed does not get it | **HIT** — relations `3,4,5,6` return `500` (`wbs_s5`) |
| P3.3 | it generalises over the two result constants via the fixup | **HIT** — `base`/`delta`, `wbs_s3` word-exact |
| P3.4 | branch context selects `cmplwi`+`bc`; there is a value-vs-branch bit, and it gets named | **HIT** — the bit is the *tuple opcode*: `0x2d4` compare vs `0x2ea` relational-as-value |
| P3.5 | a variable bound also gets a carry idiom, with `subfc` | **HIT** — `wbs_s2`, `wbs_b2` |
| P3.6 | a `-QX` switch isolates it | **NOT TESTED** — `DAT_10c2e2f4`/`DAT_10c2ed00` gate `FUN_10c1b2fa`; no counterfactual run. Withdrawn. |
| P4.1 | WB-H's narrowing holds: the arithmetic operators are one-to-one | **HIT** (§2.4) |
| P4.2 | signed `/` by a power of two emits `srawi`+`addze` | **HIT**, `wbs_k1` |
| P4.3 | `char`/`short` narrowing is `extsb`/`extsh`, `unsigned char` is `rlwinm` | **NOT TESTED** — no cell; withdrawn |
| P4.4 | record forms are a *fusion* peephole | **MISS** — a record form is a **separate opcode number** with `flags & 0x10` marking the sibling (§1.1). Retracted. |
| P4.5 | under 120 selection arms for the integer scalar operators | **HIT** — 46 dispatch arms total, of which ~20 are integer scalar |
| P5.1 | "yes, with a boundary set by the combinations" | **HIT**, §9 |
| P5.2 | first class `expr_straightline_int`, predicted reach **0** | **HIT** on the reach half; §9.3 |
| P5.3 | a general `lower_expr` under 800 lines of Rust | §9.4 — estimated **≈ 600**, not measured. Registered, unscored. |
| P5.4 | at least one operator readable but not predictable without a further unread pass, and named | **HIT** — `FUN_10c194b8`, named in §7.6 by the cell that refuted the prediction |
| P6.1 | calibration reads sizes only | **MISS on the method** — `--no-disasm` prints words. Disclosed in §6.1 rather than quietly absorbed. |
| P6.2 | at least 2 graded cells miss | **HIT** — exactly 2 |
| P6.3 | registers follow WB-D §3.4 unchanged, so a word miss is a selection miss | **HIT for §3.4, MISS for this lane** — §7.7; the three word-level misses are register misses, and WB-D's rule predicts them correctly |

**33 registered, 22 hits, 5 misses, 2 partial, 4 withdrawn untested.** Every
miss is above, in the row it belongs to.

---

## 9. THE JUDGMENT — can the port lower an ARBITRARY straight-line body from a derived pattern set? (deliverable 4)

### 9.1 The answer

> **Yes — for a stated, checkable class, and the class is much larger than a
> transcription. The pattern set for the operators the IL actually carries is
> about 60 rules plus two cost models, and it is READ, not guessed. What it is
> NOT yet is complete: three named routines carry the remaining cases, and one
> of them broke a cell in this very grid.**

This is the first "yes" this campaign has produced for a question that is not
about registers. WB-D's §9.1 said *"a correct register policy assigns correct
registers to the wrong instructions until the pattern set is right."* The
pattern set is now right for the operators the corpus is made of, and the proof
is that four cells came out **word for word** from a reading with no obj in the
loop.

The reason it generalises where a transcription does not is structural, and it
is the single most useful sentence in this document:

> **The knowledge is in tables, not in code.** Sixteen 26-entry arrays give
> operator × type → opcode. One 665-entry array gives opcode → encoding. Three
> 20-byte arrays normalise relations. A port that transcribes those four
> families has c2's selection for arithmetic, loads, stores, moves, compares
> and the logicals — *all types at once*, because the type axis is the table's
> own index.

### 9.2 What a general `lower_expr` needs, concretely, in order

1. **The opcode → encoding map** for the ~120 opcodes the corpus uses. Free:
   it is the PowerPC architecture, and `0x10c3a578` only confirms which
   spelling c2 picked. **No DISCLOSURE row needed** — a port can write
   `subfic = 0x20000000` from the ISA manual.
2. **The operator × type tables** (§2.2). ~16 × 26, of which maybe 12 × 10 are
   live. This is where the port stops guessing about `cmpw`/`cmplw`,
   `divw`/`divwu`, `lbz` vs `lha`. **Black-box re-derivable** — every entry is
   one two-line fixture.
3. **The immediate-fit rule** (§2.3): signed-16 for nibble 1, unsigned-16 for
   nibbles 2–4, else force to a register. Black-box re-derivable.
4. **The `rlandi` expansion** (§2.4): contiguous mask → `rlwinm`, and *not*
   `andi.`. Obj-confirmed 5 for 5 here.
5. **The relational-as-value cost race** (§3). This is the only piece that is a
   genuine algorithm, and it is ~120 lines: two normalisation state machines,
   two cost functions, one `<=` comparison. It is also the piece with the
   highest leverage, because **every `bool` in C++ goes through it.**
6. **Then** WB-D §3.4 for registers, which §7.7 re-confirms is free.

### 9.3 The first general class, and its predicted reach

**`expr_straightline_int`** — one basic block; integer scalar operators from
§2.2's live slots; relationals as values via §3; loads and stores of locals and
parameters; no calls; no floats, no VMX, no aggregates.

**Predicted reach over the 124-TU reach-pool: `0` on the first scan, and a lane
that expects otherwise will report a failure.** This is WB-D P5.4 and WB-H §9.1
unchanged and untouched by anything here: 48 of the frontier's 59 functions die
at the port's **IL reader** before any emitter question is reachable. Nothing
in §1–§7 moves that.

What *does* change is the price of every future emitter class, and by more than
the loop lane changed it. WB-H shipped one *shape*. This lane ships the
*index*: the reason a shape was ever needed is that nobody had the operator ×
type map, and now it is on the page with its address.

### 9.4 What `lower_expr` costs in port terms

Roughly, and registered as an estimate rather than a measurement:

| piece | Rust |
|---|---|
| opcode enum + encoders for ~120 opcodes | ~250 lines, mostly a table |
| the operator × type map | ~80 lines of `const` |
| the per-operator dispatch (§2.1's 20 integer arms) | ~150 lines |
| the relational-as-value race (§3.2 + §3.3 + §3.4) | ~120 lines |
| the `rlandi` expansion | ~40 lines |

**≈ 640 lines**, against four transcribed body shapes that cover four
functions. PREREG P5.3 registered "under 800"; this is inside it but it is an
estimate and is scored as unmeasured.

### 9.5 What is explicitly NOT claimed

* **`FUN_10c194b8` @ `0x10c194b8`** — the bool-typed (`type nibble 5`) relational
  expander, 890 bytes, **located and not read**. It refuted `wbs_b3`. Any
  `lower_expr` that handles `bool` must read it first, and the honest form of
  the class predicate today is *"result values that are not `{0,1}`"*.
* **`FUN_10c1a908`'s twenty arms** (§3.6) — the zero-operand fast path,
  located, dispatch read, arms not enumerated. It probably explains the
  calibration cell `wbk_4`'s `subfze`.
* **`FUN_10c0b300`** — the `cr0`-instead-of-`cr6` decision. `unknown`.
* **The magic-number divide** for non-power-of-two constants, and the multiply
  strength reduction. No cell.
* **Float and VMX selection.** The tables carry slots 13/14/25 and this lane
  read none of them.
* **Whether `0x10b18990`'s 19-value space is the same enum as the relation
  codes** (§3.5), which is what would settle WB-H's "size table" gloss. Left
  open, deliberately.

---

## 10. Pre-drafted DISCLOSURE rows

Per `DISCLOSURE.md` step 5 the black-box alternative is preferred, and **for
most of this lane it is available and should be used**: the operator × type
tables, the immediate-fit rule and the `rlandi` expansion are all re-derivable
from two-line fixtures against real `c2.dll` with no address at all. What is
**not** black-box re-derivable is the *cost model* and the *tie rule* — no obj
separates "B was cheaper" from "ties go to B" without an independent cost.

| # | Kind | What would be adopted | Address in `c2.dll` | Adopted into | Commit | Notes |
|---|---|---|---|---|---|---|
| **W-SELECT-1** | **route** | **The machine opcode space and its three parallel arrays** — mnemonic/format/flags, encoding skeleton, property byte — and that record forms are the *next opcode number* with `flags & 0x10` marking the sibling. | **`0x10b1b260`** (665 × 12), **`0x10c3a578`** (× 4), **`0x10c3afd8`** (× 1) | *(nothing — this lane adopts no code)* | *(pending)* | **A port needs no row for the encodings** — they are the PowerPC ISA. Carry this row only if the *opcode numbering* or the *flag bit assignments* are copied. |
| **W-SELECT-2** | **adoption-ready** | **The sixteen operator × type opcode tables**: `cmp`/`cmpl`, `cmpi`/`cmpli`, `divw`/`divwu`, `lbz`/`lhz`/`lwz`/`ld`, `stb`/`sth`/`stw`/`std`, `mr`/`fmr`/`vmr`, `add`/`fadds`/`fadd`/`vaddfp`, `subf`, `mullw`/`mulld`. **This is #1788's mechanism.** | **`0x10c04cb9`** (the installer), `0x10c39548` (cmp-reg), `0x10c394e0` (cmp-imm), `0x10c393a8` (sub), `0x10c39340` (add), `0x10c39478` (div), `0x10c38f30`…`0x10c391a0` (move/load/store), `0x10c39000`/`0x10c390d0`/`0x10c39208`/`0x10c39270` (`-QVMX128` variants) | *(nothing)* | *(pending)* | **The black-box alternative is complete and should be preferred**: one fixture per (operator, type) re-derives every live entry, and cell `wbs_s5` already exhibits `cmpwi` for `int`. Carry the row only if the *table layout* or the *type-slot numbering* is copied. |
| **W-SELECT-3** | **route** | **The value-producing relational is a COSTED RACE between two branchless expanders, decided by word count, ties to the `cntlzw` strategy; `500` means infeasible and the caller then emits a real branch.** Includes the relation-code normalisation (`==`→`!=`, `<u`→`>u` by swapping the compare operands, `<=u`/`>=u` by swapping the result pair) and the identity `result = base + ((CA − 1) & delta)`. | **`0x10c1b517`** (the driver and the tie), **`0x10c1ac5c`** (carry), **`0x10c1af2d`** (cntlzw), **`0x10c1b315`** (the `2·cost < 20` gate), `0x10b813f1` (its caller in `p2\misc.c`), `0x10c1b2fa` + `DAT_10c2e2f4`/`DAT_10c2ed00` (the enable), `0x10c1a908` (the zero-operand table), `0x10c194b8` (the bool-typed path, unread) | *(nothing)* | *(pending)* | **This one genuinely needs the row.** The *emitted shapes* are black-box (`select_grid.cpp` exhibits five of them with no address), but the **cost function**, the **tie rule** and the **500 ⇒ branch** rule are not visible in any obj: cell `wbs_s4` is a 4–4 tie and no obj can tell you it was a tie. Grey-zone rule applies. |
| **W-SELECT-4** | **route** | **The relation-code space** `1 ==, 2 !=, 3/4/5/6 signed < > <= >=, 7/8/9/10 unsigned`, and its three remap tables (signed→unsigned, commute, negate). | `0x10b189a4`, `0x10b189b8`, `0x10b189cc` (and `0x10b18990`, `unknown`) | *(nothing)* | *(pending)* | **No obj exposes these numbers** — they are c2's internal encoding and a port has its own. The row exists because §3.2's normalisation table is *defended* by quoting them. A port that states the rule as "canonicalise every relation to unsigned `>`" needs no row. |
| **W-SELECT-5** | **adoption-ready** | **`/` by a power of two is `srawi` + `addze`**, and **`&` with a contiguous mask is `rlwinm`, never `andi.`**. | `0x10c068ee` (divide), `0x10c0a2e2` (the `rlandi` expansion) | *(nothing)* | *(pending)* | **The black-box alternative is complete and should be used instead**: `wbs_k1` and `wbs_k2`/`wbs_s3`/`wbs_b1`/`wbs_b2`/`wbs_k3` exhibit both with no address. Row recorded only so the *site* is on file. |

**Held, not proposed.** The 46-arm dispatch map of `FUN_10c0f882` (§2.1) is
navigation of the most useful kind and is *not* offered as a row, because a
port's own IR has its own opcodes and only the *grouping* transfers — and the
grouping (`|` and `^` share a routine; `+` and `−` are mirrors; the intrinsics
are one arm) is re-derivable from any C compiler's structure.

**Not claimed.** This lane did not read `FUN_10c194b8`, `FUN_10c1a908`'s twenty
arms, `FUN_10c0b300`, the magic-number divide, the multiply strength reducer,
float or VMX selection, or `dag.c`'s tree-to-tuple walk. §9.5 is the list, and
§7.6 is the cell that paid for the first entry on it.
