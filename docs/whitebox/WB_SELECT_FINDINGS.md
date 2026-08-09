# WB-I `wb-select` — how c2 selects PPC instructions

> **PROVENANCE — DISASSEMBLY-DERIVED.** Obtained by statically disassembling
> Microsoft's `c2.dll` — the exact image pinned in
> [`C2_MAP_METHOD.md`](C2_MAP_METHOD.md) §0, sha256 verified at the top of this
> lane as `c80981c015166effecc71ad8112d5577a065b2300891dfdb02b9c13787a66258`
> (and the `c2.dll` the grid was compiled against hashes the same). It is
> **navigation only** until a row is added to [`DISCLOSURE.md`](DISCLOSURE.md).
> **The obj is the sole judge** (method doc §7): §6 grades every reading here
> against real `c2.dll` under wibo, and §6.3 records what the objs refuted.

Lane `wb-select` (WB-I), campaign 2 (`CAMPAIGN_2026-08-08_GENERATORS.md`).
PREREG: [`WB_SELECT_PREREG.md`](WB_SELECT_PREREG.md) — **salvaged by
cherry-pick from an aborted run of this same lane** (commit `18a2ec45` on the
dead branch `worktree-agent-a31667159c896762b`, killed by an expired login, not
by anything about its work). It was frozen *before that run's first grep of the
flat export*; this run treats P0–P6 as its own registered predictions and
scores them unchanged in §8. Grid:
[`grids/wb-select/select_grid.cpp`](grids/wb-select/select_grid.cpp) +
[`frozen.tsv`](grids/wb-select/frozen.tsv), committed in `3968991` **before the
grid's first `cl.exe`**. Calibration:
[`grids/wb-select/calib.cpp`](grids/wb-select/calib.cpp), unscored, read for
section sizes only. Post-grid diagnostics:
[`grids/wb-select/diag.cpp`](grids/wb-select/diag.cpp), unscored.

`lur.c` is WB-H's and is not re-tread here.

---

## 0. The answer in one screen

**Instruction selection in c2 is a two-layer table walk with a small
hand-written idiom layer bolted onto one operator.** It is far smaller and far
more mechanical than WB-D §9.1 assumed when it called the pattern set the last
"no".

| layer | what it is | where |
|---|---|---|
| **1. the machine-opcode enum** | one alphabetically-ordered array of `{const char *mnemonic; u32 operand_format; u32 attributes}`, 662 real PPC entries then ~40 **pseudo-ops** | **`0x10b1b260`**, 12-byte stride; mnemonic pool `0x10b19700`–`0x10b1b25f` |
| **2. the per-operator opcode tables** | **13 tables × 26 entries**, indexed by an operand **type index**; entry = a machine opcode | installed by **`FUN_10c04cb9` @ `0x10c04cb9`** into `DAT_10c6fdac`…`DAT_10c6fddc`; bodies at `0x10c38f30`…`0x10c39548` and `0x10b1fd08` |
| **3. the dispatch** | a `switch` on `tuple->opcode`, base `0x27e`, **41 distinct arms** over 174 opcodes | **`FUN_10c0f882` @ `0x10c0f882`**; byte index `0x10c0fbd6`, jump table `0x10c0fb32` |
| **4. the idiom layer** | **two rival hand-written expanders for a value-producing relational, chosen by predicted word count** | **`FUN_10c1b517` @ `0x10c1b517`** picking between **`FUN_10c1ac5c`** (carry) and **`FUN_10c1af2d`** (`cntlzw`) |
| **5. the in-place expansion switch** | a `switch` on the **machine** opcode that rewrites pseudo-ops and narrowings in situ, 18 arms | **`FUN_10c182b4` @ `0x10c182b4`**; byte index **`0x10c184a8`**, jump table **`0x10c18460`** |

The reading survived **9 of 12** frozen obj cells on its graded core. The three
misses are all instructive and all retracted rather than hedged (§6.3): the
carry idiom's *operand orientation*, the `rlandi` mask's *expanded form*, and —
the big one — **PREREG P3.4's "value-vs-branch context bit", which does not
exist**.

---

## 1. The Rosetta stone: the machine-opcode table (deliverable 1)

PREREG **P1.5** predicted "at least one image-resident table of PPC opcode
mnemonics whose index is c2's internal machine-opcode number, and it is the
Rosetta stone that makes the selection arms readable." It exists and it is
exactly that.

**`0x10b1b260`** is an array of 12-byte records:

```c
struct mdop {              /* stride 12 */
    const char *mnemonic;  /* +0  into the pool 0x10b19700..0x10b1b25f */
    u32         format;    /* +4  operand-shape class (0x31 = XO rD,rA,rB;
                                  0x33 = D rD,rA,SIMM; 0x2f = rD,rA; ...)   */
    u32         attrs;     /* +8  see below                                 */
};
```

Index 0 is the sentinel `_first`; index **661** is `_last`; index 662 is
`illegal`. The real PowerPC mnemonics occupy 1…~620 **in alphabetical order**,
which is what makes the numbers readable at all: `add`=1, `addi`=11,
`addze`=21, `and`=25, `bc`=33, `cmp`=45, `cmpi`=46, `cmpl`=47, `cmpli`=48,
`cntlzw`=50, `mulli`=272, `mullw`=273, `neg`=279, `or`=285, `ori`=289,
`rlwinm`=307, `slw`=319, `sraw`=325, `srawi`=327, `srw`=331, `subf`=385,
`subfc`=387, `subfe`=391, `subfic`=395, `xor`=618, `xori`=620.

Indices **622…660 are c2's own pseudo-ops**, and they are the interesting half:

| # | pseudo-op | what it is |
|---:|---|---|
| 622 / 623 | **`rlandi` / `rlandi.`** | AND-with-constant, before it becomes `rlwinm`/`andi.`/`li`+`and` |
| 624 / 625 | `li` / `lis` | constant materialisation |
| 626 / 627 / 628 / 629 | `mr` / `mr.` / `not` / `not.` | copies |
| 641 / 642 / 643 | `lea` ×2 / `loffs` | address forms |
| 646 / 647 | `lau` / `lal` | the two halves of an address |
| 648 / 649 | `bdnz` / `bdz` | WB-H's loop latch |
| **652 / 653** | **`lcarry` / `lcarry.`** | **materialise XER[CA] as a 0/−1 mask** — the flagship idiom's engine |
| 654 / 655 | `rldtoc` / `retaddr` | |
| 657 / 658 | `deadtmp` / `DCD` | |

### 1.1 The attribute word, decoded

The `attrs` field is a small effect model, and every bit of it is load-bearing
downstream:

| bit | meaning | evidence |
|---|---|---|
| `0x01` | copy / store-side | `mr` = `0x11`, `vmr` = `0x11`, `stw` = `0x03` |
| `0x02` | memory reference | `lwz`/`lbz`/`lha` = `0x02`, `stw` = `0x03` |
| `0x04` | sign-extending narrow | `extsb`/`extsh` = `0x14` |
| **`0x08`** | **writes CR0 (is a record form)** | every `.` mnemonic |
| **`0x10`** | **HAS a record form** (and it is at **opcode + 1**) | `add`=`0x10`/`add.`=`0x08`; `rlwinm`=307/`rlwinm.`=308 |
| **`0x20`** | **defines XER[CA]** | `addc`=`0x30`, `subfc`=`0x30`, `subfic`=`0x20`, `srawi`=`0x30` |
| **`0x40`** | **uses XER[CA]** | `adde`/`addze`/`addme`/`subfe` = `0x70`, **`lcarry` = `0x50`** |

The same attribute byte is available to the lowering as a **byte array indexed
by machine opcode at `0x10c3afd8`** — `(&DAT_10c3afd8)[op] & 0x20` is how
`FUN_10c064cb` decides to attach a carry-def operand, and
`(&DAT_10c3afd8)[op] & 0x10` is how `FUN_10c0b300` decides a record form
exists. **Record form = opcode + 1** and bit `0x10` says the +1 is legal:
that is the entire record-form model, and PREREG **P4.4** ("record forms are a
*fusion*, not a selection") is right about the mechanism.

A second image table, **`0x10b1d190`** (16-byte stride: name, real opcode,
fixed operand field, flag), is the **simplified-mnemonic** layer — `subi`→`addi`
(11), `subis`→`addis` (14), `subic`→`addic` (12), `sub`→`subf` (385),
**`subc`→`subfc` (387)**, `blt`→`bc`(33) with BO `0x0c`, `cmpw`→`cmp`,
`cmpwi`→`cmpi`, `cmplw`→`cmpl`, `cmplwi`→`cmpli`. This is what makes WB-D's
`subc` and this lane's `subfc` the same instruction.

---

## 2. The pattern set (deliverable 2)

### 2.1 The type index — one map, 26 slots

Every operand carries a 16-bit type word `(nibble << 12) | size_in_bytes` at
operand+10 (the same word WB-D §5 saw the class map `0x10b022cc` read).
**`FUN_10bd7c10` @ `0x10bd7c10`** turns it into a **type index 0…25** by a
jump table on the nibble (**`0x10bd7cf0`**) and a size ladder inside each arm:

| nibble | meaning | size → index |
|---:|---|---|
| 1 | **signed integer** | 1→1, 2→2, 4→4, 6→5, else→6 |
| 2 | **unsigned integer** | 1→7, 2→8, 4→10, 6→11, else→12 |
| 3 | pointer | 4→16, 6→18, else→19 |
| 4 | pointer (second flavour) | 4→17, else 18/19 |
| 5 | **floating point** | 4→13, 8→14, else→15 |
| 6 | (fourth integer family) | →20…23 |
| 12 | VMX | →25 |

### 2.2 The thirteen tables

`FUN_10c04cb9` @ `0x10c04cb9` installs them. Each is 26 `int`s = `0x68` bytes.
Decoded through §1 (`-` = no opcode for that type):

| variable | table VA | operator | 1 (i8) | 2 (i16) | 4 (i32) | 6 (i64) | 7 (u8) | 8 (u16) | 10 (u32) | 12 (u64) | 13 (f32) | 14 (f64) | 16 (ptr) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| `DAT_10c6fddc` | `0x10c38f30` | **copy** | `mr` | `mr` | `mr` | `mr` | `mr` | `mr` | `mr` | `mr` | `fmr` | `fmr` | `mr` |
| `DAT_10c6fdd8` | `0x10c38f98` | **load, D-form** | `lbz` | `lhz` | `lwz` | `ld` | `lbz` | `lhz` | `lwz` | `ld` | `lfs` | `lfd` | `lwz` |
| `DAT_10c6fdd4` | `0x10c39068` | **load, X-form** | `lbzx` | `lhzx` | `lwzx` | `ldx` | `lbzx` | `lhzx` | `lwzx` | `ldx` | `lfsx` | `lfdx` | `lwzx` |
| `DAT_10c6fdd0` | `0x10c39138` | **store, D-form** | `stb` | `sth` | `stw` | `std` | `stb` | `sth` | `stw` | `std` | `stfs` | `stfd` | `stw` |
| `DAT_10c6fdcc` | `0x10c391a0` | **store, X-form** | `stbx` | `sthx` | `stwx` | `stdx` | … | … | … | … | `stfsx` | `stfdx` | `stwx` |
| `DAT_10c6fdc8` | `0x10c392d8` | **negate** | `neg` | `neg` | `neg` | `neg` | `neg` | `neg` | `neg` | `neg` | `fneg` | `fneg` | `neg` |
| `DAT_10c6fdc4` | `0x10c39340` | **add** | `add` | `add` | `add` | `add` | `add` | `add` | `add` | `add` | `fadds` | `fadd` | `add` |
| `DAT_10c6fdc0` | `0x10c393a8` | **sub** | `subf` | `subf` | `subf` | `subf` | `subf` | `subf` | `subf` | `subf` | `fsubs` | `fsub` | `subf` |
| `DAT_10c6fdbc` | `0x10c39410` | **mul** | `mullw` | `mullw` | `mullw` | `mulld` | `mullw` | `mullw` | `mullw` | `mulld` | `fmuls` | `fmul` | `mullw` |
| `DAT_10c6fdb8` | `0x10c39478` | **div** | `divw` | `divw` | `divw` | `divd` | **`divwu`** | **`divwu`** | **`divwu`** | **`divdu`** | `fdivs` | `fdiv` | `divwu` |
| `DAT_10c6fdb4` | `0x10c394e0` | **cmp, immediate** | `cmpi` | `cmpi` | `cmpi` | `cmpi` | **`cmpli`** | **`cmpli`** | **`cmpli`** | **`cmpli`** | `fcmpu` | `fcmpu` | `cmpli` |
| `DAT_10c6fdb0` | `0x10c39548` | **cmp, register** | `cmp` | `cmp` | `cmp` | `cmp` | **`cmpl`** | **`cmpl`** | **`cmpl`** | **`cmpl`** | `fcmpu` | `fcmpu` | `cmpl` |
| `DAT_10c6fdac` | `0x10b1fd08` | **convert / widen** | `extsb` | `extsh` | `extsw` | `mr` | `extsb` | `extsh` | `extsw` | `mr` | — | — | `mr` |

Four of them have `-QVMX128` alternates (`0x10c39000`, `0x10c390d0`,
`0x10c39208`, `0x10c39270`), selected on `DAT_10c2e978`; the *only* difference
is index 25 (`lvx`→`lvx128`, `stvx`→`stvx128`).

> **This table is the whole answer to #1788.** Signedness is not a per-site
> decision: the operand's type nibble picks the row, and rows 1/2/4/6 give
> `cmp`/`cmpi` while rows 7/8/10/12 give `cmpl`/`cmpli`. It is one array
> lookup, and cells S2 / S12 / `d_if_s` / `d_if_u_call` confirm it black-box
> (`cmpwi 6,3,10` from `int`, `cmplwi 6,3,10` from `unsigned`, same source
> shape).

**Pointers compare UNSIGNED** (rows 16/17 → `cmpl`/`cmpli`) — which is where
WB-D §7.6's "the switch value is compared unsigned even though the C type is
`int`" comes from: the switch value has been re-typed, not mis-selected (see
§2.5).

### 2.3 The dispatch — 41 arms, and that is the whole selector

`FUN_10c0f882` @ `0x10c0f882`:

```
op = tuple[1];  if ((u32)(op - 0x27e) > 0xad) return;
arm = byte_at(0x10c0fbd6 + (op - 0x27e));   jmp [0x10c0fb32 + 4*arm];
```

174 tuple opcodes, **41 distinct arms**, of which two are "do nothing". The
arms this lane identified:

| tuple op | arm target | what it lowers |
|---|---|---|
| `0x2af` | `FUN_10c053e7` / `FUN_10c053a8` | assign / copy |
| `0x2b3` | **`FUN_10c0f1ed`** | **convert / widen** (table `0x10b1fd08`) |
| `0x2c5`,`0x327`,`0x328` | **`FUN_10c0634b`** | **add** (table `0x10c39340`) |
| `0x2c6`,`0x329`,`0x32a` | **`FUN_10c064cb`** | **sub** (table `0x10c393a8`) |
| `0x2c7` | **`FUN_10c067f1`** | **mul** (table `0x10c39410`) |
| `0x2ca` | **`FUN_10c0711d`** | **AND** |
| `0x2cb` | `FUN_10c0718f(t, 0x11d, 0x121)` | **OR** (`or` / `ori`) |
| `0x2cc` | `FUN_10c0718f(t, 0x26a, 0x26c)` | **XOR** (`xor` / `xori`) |
| `0x2cd` | **`FUN_10c068ee`** | **divide** |
| `0x2ce` | `FUN_10c06eb1` | remainder |
| `0x2d4` | **`FUN_10c0eb17`** | **compare** (branch-consumed) |
| `0x2ea` | **`FUN_10c1b517`** | **value-producing relational** — the idiom layer |
| `0x2f0`,`0x2f4` | `FUN_10bfee9a` / `FUN_10bfee89` | epilogue / prologue |

> **PREREG P1.2 scored a hit and P1.3 a partial**: `cgintrin.c` is *not* the
> main operator selector, and the dispatch does live in the `p2\ppc\lower.c`
> band (`0x10c053e7`…`0x10c11060`) — but the **idiom layer is not there**, it
> is in the unattributed `p2\ppc\` band `0x10c19xxx`–`0x10c1bxxx`, next to the
> machine peepholes.

**PREREG P1.6 hits**: the arithmetic/compare operators are dense in one band —
`0x2c5` add, `0x2c6` sub, `0x2c7` mul, `0x2ca` and, `0x2cb` or, `0x2cc` xor,
`0x2cd` div, `0x2ce` rem, `0x2d4` cmp — so the selector really does switch on
`op − base`.

### 2.4 The per-operator arms, read

**OR / XOR — `FUN_10c0718f` @ `0x10c0718f`.** Nine lines. `op := imm_opcode`
if source 1 is a constant, else `reg_opcode`; then if the chosen opcode's
attribute byte has `0x08`, mint a CR0 operand. **There is no 16-bit fit test**
— which is why `x | 0x12345` becomes `oris`+`ori` (cell S8, hit) rather than
`lis`+`or`. PREREG **P2.2 is half right**: the 16-bit fit test exists, but only
in the *compare* lowering, not in the logical one.

**AND — `FUN_10c0711d` @ `0x10c0711d`.** Asymmetric on purpose, because
`andi.` is record-form-only: a constant operand does **not** select `andi.`, it
mints the **pseudo-op `rlandi` (622)** and defers. A register operand selects
`and` (25). Cell S7 (hit) shows the pseudo-op coming out as
`clrlwi r3,r3,24` = `rlwinm r3,r3,0,24,31`.

**MUL — `FUN_10c067f1` @ `0x10c067f1`.** Constant operand: power of two →
`FUN_10c06786` (strength-reduce to a shift); otherwise **`mulli` (272)**,
full stop. **There is no shift/add decomposition**, so PREREG **P2.3 is
refuted** as written — the threshold it predicted is a two-case test, not a
population count.

**DIV — `FUN_10c068ee` @ `0x10c068ee`.** The signed power-of-two arm is the
one worth quoting, because it is a *combination* and the port cannot emit it:

```
if (nibble == 1 && divisor is a positive power of two) {
        emit  0x147 (srawi)  /  0x143 (sradi) for 64-bit,  shift = log2(d)
        attach an XER[CA] DEF operand   (FUN_10bd42c2(0x50, 0xb000))
        tuple->opcode = 0x15            /*  addze  */
}
```

i.e. **`srawi` + `addze`**, the sign-bias idiom, exactly as PREREG **P4.2**
registered. A negative power of two takes the same path and negates.
Everything else falls through to the div table — **including a non-power-of-two
constant**, so `x / 3` is `li`+`divw` and **there is no magic-number multiply
at `/O1`**. PREREG **P2.4 is refuted** (cell S6, hit).

**COMPARE — `FUN_10c0eb17` @ `0x10c0eb17`.** This is where the immediate-fit
logic lives, and it is signedness-asymmetric exactly as PPC's encodings are:

* nibble 1 (signed) → the constant must fit **signed 16 bits**
  (`(v & 0xffff8000) == 0` or `== 0xffff8000` with the high word `-1`);
* nibble 2, 3, 4 (unsigned / pointer) → it must fit **unsigned 16 bits**
  (`(v & 0xffff0000) == 0` and the high word `0`).

Fit → `DAT_10c6fdb4` (`cmpi`/`cmpli`); no fit → `DAT_10c6fdb0`
(`cmp`/`cmpl`). PREREG **P2.2's asymmetric-immediate claim hits**.

### 2.5 The signedness FLIP, which nobody had listed

If the constant does not fit the form its own type wants, `FUN_10c0eb17` calls
**`FUN_10c07803`** (a value-range query on the other operand) and, when the
range makes it safe, **rewrites the comparison's type nibble**:

```c
uVar9 = (u16)(((nibble == 1) + 1) * 0x1000 | size);   /* 1 -> 2, else -> 1 */
```

— a signed compare becomes unsigned, or an unsigned one signed, purely to reach
an immediate form. **This is the mechanism behind WB-D §7.6's unexplained
"unsigned compare of a signed C type" in its switch cell.** It is navigation
here; this lane did not build a cell that isolates it, and says so rather than
claiming it.

### 2.6 Record forms and the CR field

`FUN_10c0b300` @ `0x10c0b300` decides both. It requires the compare's second
operand to be the **constant 0**, walks *backwards* down the tuple list to the
instruction that defined the compared value, and accepts iff that opcode's
attribute byte has `0x10` (has a record form). Two consequences:

* the CR field is then **`cr0`** (descriptor `0x10c2f088 + 0x60·0x43`),
  otherwise **`cr6`** (`+ 0x60·0x49`) — which is WB-D §7.5's retraction
  *derived* rather than observed: `cr6` is the default and `cr0` is the
  record-form path;
* there is a **special promotion** `addi` (11) → `addic` (12) with a minted
  carry operand before the record form is taken, gated on `(&DAT_10c3afd8)[12]
  & 0x10`. That is where WB-D's `addic. r31,r31,-1` came from.

---

## 3. The idiom layer — a value-producing relational (deliverable 2, flagship)

Tuple opcode **`0x2ea`** → **`FUN_10c1b517` @ `0x10c1b517`**, 140 bytes, and
it is the most surprising thing in the lane:

```c
if (type nibble == 5) return float_path(t);          /* FUN_10c194b8 */
if (<one side is the constant 0> && FUN_10c1b2fa(...))
        return zero_path(t);                          /* FUN_10c1a908 */
cost_carry  = FUN_10c1ac5c(t, 0);      /* DRY RUN — returns a WORD COUNT */
cost_cntlzw = FUN_10c1af2d(t, 0);      /* DRY RUN                        */
if (cost_cntlzw <= cost_carry) FUN_10c1af2d(t, 1); else FUN_10c1ac5c(t, 1);
```

**Two rival hand-written expanders, run first in cost-only mode and then the
cheaper one for real, ties to the `cntlzw` one.** PREREG **P2.1** said "not a
generic bottom-up rewriter, no BURG/iburg cost table" — correct on the
mechanism (it is hand-written C) but **wrong that there is no cost model**:
there is one, it is just two-way and it counts words.

### 3.1 The carry expander — `FUN_10c1ac5c` @ `0x10c1ac5c`

Reads the relational's **condition code** from `tuple + 0xd`. The code enum
(names at `0x10b197c0`–`0x10b197f4`) is
`0 ILLEGAL, 1 EQ, 2 NE, 3 LT, 4 LE, 5 GT, 6 GE, 7 ULT, 8 ULE, 9 UGT, 10 UGE`,
and if the operand type is unsigned the code is first remapped through the
byte table **`0x10b189a4`** = `[0,1,2,7,8,9,10,7,8,9,10,…]` — i.e. **LT→ULT,
LE→ULE, GT→UGT, GE→UGE**.

The normalisation is a small state machine, and it is the whole applicability
predicate:

| code | action |
|---|---|
| 1 `EQ` | swap operands, code := 2 |
| 2 `NE` | **require the other operand to be the constant 0**, code := 8 |
| 7 `ULT` | code := 8, swap |
| 8 `ULE` | **expand** |
| 9 `UGT` | swap, code := 8 |
| 10 `UGE` | code := 9 → swap → 8 |
| **3,4,5,6 (signed LT/LE/GT/GE)** | **`return 500`** — not applicable |

> **PREREG P3.2 hits, and it is the sharpest result in the lane: the carry
> idiom is UNSIGNED-ONLY.** A signed value-producing `<` never gets it, because
> the signed predicate is not a carry-out. Cell S2 confirms it against a real
> obj.

The emission, with `A` = the true-value constant and `B` = the false-value
constant:

```
   [ li      t, K      ]        if the compared bound is itself a constant  (0x270 = li)
     subfic  t, x, K   |        if the other side is a constant fitting SIMM16 (0x18b)
     subfc   t, x, t   |        otherwise                                     (0x183)
                       + an XER[CA] DEF operand
     lcarry  t                  (0x28c) -- materialise CA as a 0/-1 mask
   [ rlandi  t, t, A-B ]        skipped iff A-B == -1                       (0x26e)
   [ addi    t, t, B   ]        skipped iff B == 0                          (0x0b)
```

and the **cost** it returns is `2 + (compared side is a constant) + cost(A−B
mask) + (B != 0)`.

**PREREG P3.1 hits on mechanism** (`lcarry` is exactly "a `subfe` producing a
0/−1 mask from CA, plus an arithmetic fixup"), and **P3.3 hits outright**: the
idiom generalises over arbitrary `A`/`B`, the mask is `A−B` and the fixup is
`+B`. Cells S1 and S11 exhibit both.

### 3.2 The `cntlzw` rival — `FUN_10c1af2d` @ `0x10c1af2d`

Handles `EQ`, `NE` (swap), `LT` and `GE` (each with an **extra `cntlzw`**), and
only against a **zero** right-hand side unless the code is EQ/NE. Emission:

```
   [ subf / addi ]              form (x - y) when the RHS is not already 0
   [ cntlzw ]                   the extra one, for LT / GE
     cntlzw  t, v               (0x32)
     rlandi  t, t, rot=27 mask=1        -- i.e. srwi t,t,5: 1 iff v was 0
   [ xori  t, 1 ]               invert
   [ neg   t ]                  (0x117)
   [ addi  t, B ]
   [ rlandi t, log2(A), -1<<log2(A) ]   scale by a power of two
```

Its applicability is narrower than the carry expander's on relations but wider
on result constants: it needs `|A−B| == 1` or a power of two.

### 3.3 What the two rivals mean together

The pair is the answer to "is there an idiom recogniser firing on a
*combination* rather than one operator" (PREREG **P2.5**, hit): **yes, and it
is the only one**. It fires on `relational ⊗ two-constant-select`, which is one
tuple in c2's IR (`0x2ea`) and two or three operators in C. Nothing analogous
exists for `+`, `−`, `*`, `&`, `|`, `^` or the shifts — each of those is one
table lookup plus at most one constant test.

> **WB-H §9 item 3's narrowing (P4.1) is CONFIRMED**: for integer `+ − * ^ & |`
> and the shifts the pattern set is a table, not an algorithm. The idiom
> library bites on **relationals producing values** and on **divide by a
> power of two**, and nowhere else in the scalar integer set.

---

## 4. The in-place expansion switch (deliverable 1, third part)

WB-D §4 saw "a giant `switch` on `instr->opcode` rewriting each pseudo-op in
situ" and did not name its table. It is **`FUN_10c182b4` @ `0x10c182b4`**:

```
op = instr[1] - 1;  if ((u32)op > 0x292) goto done;
arm = byte_at(0x10c184a8 + op);   jmp [0x10c18460 + 4*arm];
```

— indexed by the **machine** opcode over the whole enum (1…659), **byte index
table `0x10c184a8`, jump table `0x10c18460`**, and it has only **18 arms**:

| arm | opcodes | handler |
|---|---|---|
| 0 | 38 three-operand ALU ops (`add`, `and`, `divw`, `eqv`, …) | `FUN_10c17552` |
| 1 | `cmpi` | `FUN_10c18147` |
| 2 | 11 (`cmpli`, `neg`, `ori`, `sradi`, `srawi`, `xori`, `not`, …) | `FUN_10c17146` |
| 3/4/5 | `extsb`, `extsh`, `extsw` (± record) | `FUN_10c17e78` |
| 7/8/9 | the `stb`/`sth`/`stw` families | |
| 10/11/12 | 138 VMX ops | |
| **13** | **`rlandi` / `rlandi.`** | **`FUN_10c1772b` @ `0x10c1772b`** |
| 14/15/16 | `mr`, `mr.`, `vmr` | `FUN_10c16d83` / `FUN_10c16e59` |
| 17 | 445 opcodes | nothing |

So the "final expansion" pass is **not** a general pseudo-op expander — it is a
narrow set of in-place rewrites, and the one that matters for this lane is
arm 13, `FUN_10c1772b`, which turns `rlandi` into a real instruction.

---

## 5. The obj-check — frozen before the first `cl.exe` of this grid

Source: [`grids/wb-select/select_grid.cpp`](grids/wb-select/select_grid.cpp).
Predictions: [`grids/wb-select/frozen.tsv`](grids/wb-select/frozen.tsv). Both
committed in `3968991` **before** the grid's first compile. 12 cells, one
COMDAT each, `/nologo /c /GR /O1 /Oi /EHsc` (WB-D's workload mode); the run is
`work/wb-select/run/grid.obj` (not committed — it is an obj). The `c2.dll`
driven by wibo hashes `c80981…6258`, the same image the readings come from.

**Grading granularity was fixed in advance** in `frozen.tsv`: each cell is
graded on its **mnemonic sequence and immediate values** ("core") and
separately on register identities. Word *counts* are not graded, because the
calibration pass had already measured them.

### 5.1 The calibration pass, and why it exists

[`calib.cpp`](grids/wb-select/calib.cpp), 34 cells, compiled first, **read for
section sizes only — never a word sequence** — and unscored. The lane brief
required it because wb-inline's v1 grid was refuted by its own cells. It
changed the grid three times before the freeze:

* **`x < 10 ? 1 : 2` is 5 words for BOTH signednesses.** A grid that graded
  word counts would have measured nothing on the lane's flagship question. Every
  prediction in `frozen.tsv` is therefore a *sequence*, and `frozen.tsv` says so.
* **`x / 3` is 3 words.** A magic-number multiply cannot fit in 3; the cell was
  kept precisely because that refutes P2.4 cheaply.
* **`short` load + add is 3 words but `signed char` load + add is 4.** The
  tables predict 4 for both (`lhz`+`extsh`, `lbz`+`extsb`). That gap is what
  made cell S10 a *derived* prediction — "a fusion to `lha` must exist" — rather
  than a transcription of the table.

---

## 6. RESULTS — 9 of 12 on the graded core

| cell | predicted core | emitted | verdict |
|---|---|---|---|
| **S1** `sel_ltu_ab` | carry setter with **immediate** 9 or −10; a **2-word** 0/−1 materialisation containing `subfe`; mask by 4; `addi …,3`; no `cmplwi`/branch/`cntlzw` | `li 11,10` · `subc 11,3,11` · `subfe 11,11,11` · `rlwinm 11,11,0,29,29` · `addi 3,11,3` · `blr` | **MISS** (§6.3a) — the mask, the bias and all three absence claims hit; the **setter form** does not |
| **S2** `sel_lts_ab` | `cmpwi` cr6 vs 10, a conditional branch, `li 7`, `li 3`, `blr`; **no `subfe`/`cntlzw`/`addze`** | `cmpwi 6,3,10` · `li 3,7` · `bclr 12,24` · `li 3,3` · `blr` | **HIT** |
| **S3** `sel_eqz` | `cntlzw` then `rlwinm …,27,31,31` then `blr` | `cntlzw 11,3` · `rlwinm 3,11,27,31,31` · `blr` | **HIT** (core exact) |
| **S4** `sel_nez` | the **carry** expander wins: a carry-setting subtract then `subfe`; no `cntlzw`, no `xori` | `addic 11,3,-1` · `subfe 3,11,3` · `blr` | **HIT** |
| **S5** `sel_divs8` | `srawi …,3` then `addze` then `blr` | `srawi 11,3,3` · `addze 3,11` · `blr` | **HIT** |
| **S6** `sel_divs3` | `li 3` then `divw`; **no `mulhw`/`mulhwu`** | `li 11,3` · `divw 3,3,11` · `blr` | **HIT** |
| **S7** `sel_and_ff` | `rlwinm …,0,24,31` (`clrlwi 24`), **not `andi.`** | `clrlwi 3,3,24` · `blr` | **HIT** |
| **S8** `sel_or_big` | `oris …,1` then `ori …,0x2345`; **no `lis`, no `or`** | `oris 3,3,1` · `ori 3,3,0x2345` · `blr` | **HIT** |
| **S9** `sel_schar` | `lbz` · `extsb` · `addi …,1` · `blr` | `lbz 11,0(3)` · `extsb 11,11` · `addi 3,11,1` · `blr` | **HIT** |
| **S10** `sel_short` | `lha` · `addi …,1` · `blr`; **`extsh` does not appear** | `lha 11,0(3)` · `addi 3,11,1` · `blr` | **HIT** |
| **S11** `sel_ltu_pow2` | setter, `subfe`, mask by 8 **via `rlwinm`**, no `addi` bias | `li 11,10` · `li 10,8` · `subc 11,3,11` · `subfe 11,11,11` · **`and 3,11,10`** · `blr` | **MISS** (§6.3b) — the "no bias" clause hits, the mask **form** does not |
| **S12** `sel_br_u` | `cmplwi` cr6 vs 10, a conditional branch, `li 1`, `li 2`; **no `subfe`** | `li 11,10` · `subc 11,3,11` · `subfe 11,11,11` · `addi 3,11,2` · `blr` | **MISS** (§6.3c) — **there is no branch at all** |

**9 hits · 3 misses on the graded core.** Registers: WB-D §3.4 held on every
cell — `r11` first, then `r10`, result in `r3` — and where my own `regs` column
guessed `r3` throughout it was **my** guess that was wrong, not c2's rule (S3,
S5, S9, S10). No cell needed a register I could not predict from §3.4.

**The success floor is CLEARED several times over.** S5 (`srawi`+`addze`), S3
(`cntlzw`+`rlwinm 27,31,31`), S4 (`addic`+`subfe`), S7 (`clrlwi`), S8
(`oris`+`ori`) and S10 (the `lha` fusion) are each a pattern-set reading that
survived a frozen check on an idiom `c2-core` does not emit today.

### 6.1 What the diagnostics added (unscored)

[`diag.cpp`](grids/wb-select/diag.cpp), run **after** the grid was scored, to
localise the two mechanism misses. Nothing here is a prediction.

* **The carry idiom is not limited to 16-bit bounds.** `x < 100000u` gives
  `lis 11,1` · `ori 11,11,0x86a0` · `subc` · `subfe` · `addi 3,11,2`.
* **`rlandi`'s form tracks its registers, not its mask.** Masks 2, 3, 4, 8 and
  16 with no bias all give `li 10,M` + `and 3,11,10`; masks 4 and 8 **with** a
  bias give `rlwinm 11,11,0,29,29` and `rlwinm 11,11,0,28,28`. The distinguishing
  fact is that in the biased cells `rlandi`'s source and destination land in the
  same register and in the unbiased ones they do not. **The predicate is unread**
  — it is inside `FUN_10c1772b` @ `0x10c1772b` — and this lane names it rather
  than guessing (PREREG **P5.4**, hit).
* **The branch comes back the moment an arm does anything.** `if (x<10u) return
  ext(1); return 2;` → `cmplwi 6,3,10` + `bf 24` + branch. So does a store arm,
  a three-arm ladder (first arm branches, second becomes the idiom), and the
  signed form. A `while (x<10u)` loop gives `cmplwi cr6` + `subfic` + `mtctr` +
  `bdnz`, WB-H's normal form, unchanged.

### 6.2 Both immediate forms of the compare table, confirmed black-box

`cmpwi 6,3,10` from `int` (S2, `d_if_s`) and `cmplwi 6,3,10` from `unsigned`
(`d_if_u_call`, `d_if_u_3arm`, `d_if_u_store`, `d_while_u`) — same source
shape, same constant, same `cr6`. **#1788 now has an obj**, and it needs no
address: the black-box derivation is complete.

### 6.3 The three misses, stated as misses

**(a) S1 — the carry setter takes a REGISTER, not an immediate.** I predicted
`subfic r?,r3,9`. c2 emitted `li 11,10` + `subc 11,3,11` (= `subfc`). The
reading was right and my *instantiation* was wrong: `FUN_10c1ac5c` chooses
`subfic` (`0x18b`) only when **`local_8`** — the operand that survives the
swap dance as the subtrahend — is the constant. For `x <u K` the normalisation
`ULT → swap → ULE` puts the **bound** in the minuend position, so the constant
is materialised by the `0x270` (`li`) arm and the subtract is `subfc`. I could
have derived that from the code I had read and did not. **The prediction as
written names a word (`subfic`) that is not in the obj, so it is scored a
MISS**, per WB-D §7.4's rule. `subfic` does appear in the same compiler on the
same day — as loop trip-count arithmetic (`d_while_u`) — so the opcode is not
dead, it is just not on this path.

**(b) S11 — `rlandi` did not become `rlwinm`.** Retracted: **this lane cannot
predict the expanded form of an AND-with-constant.** Seven cells bound it
(§6.1) and the deciding pass is named (`FUN_10c1772b`), but the predicate is
unread. A port that wants byte-exactness on any expression containing `&` with
a constant **must read `FUN_10c1772b` first**; a port that only wants it in the
biased position can copy S1's shape. This is the single most important
"not claimed" in the lane, because `&`-with-a-constant is everywhere.

**(c) S12 — PREREG P3.4 is REFUTED. There is no value-vs-branch context bit.**
I predicted that a relational feeding a branch selects `cmplwi` + `bc`, per
#1788, and that the selector therefore carries a context bit. It does not.
`if (x < 10u) return 1; return 2;` came out as **the same five-word branchless
carry idiom** as `return x < 10u ? 1 : 2;`. What actually happens is upstream of
selection: **an `if`/`else` whose arms are side-effect-free constant returns is
if-converted into a single value-producing relational tuple (`0x2ea`) before
`FUN_10c0f882` ever runs**, and only then does the idiom layer see it. The
diagnostics pin the boundary: a call, a store, or a signed relation in either
arm and the branch survives.

The replacement rule, obj-grounded on 8 cells:

> **A two-way `if` selects a compare-and-branch iff either arm has a side
> effect, or the relation is signed with a non-zero bound; otherwise it is
> if-converted and goes through the idiom layer.** The "context" that matters is
> the *shape of the arms*, not the *position of the compare*.

---

## 7. THE JUDGMENT — can the port lower an ARBITRARY straight-line body? (deliverable 4)

### 7.1 The answer

> **Yes, for a bounded and namable class — and the pattern set for the operators
> the IL actually carries is roughly 60 rules, not the "under 120" P0.3
> registered and nothing like a compiler-sized problem. But `lower_expr` is
> gated on ONE unread pass (`FUN_10c1772b`, the `rlandi` expander), and the
> class it unlocks converts ZERO TUs on the first scan for exactly the reason
> WB-D §9.3 and WB-H §9.1 already scored.**

This is a real change from WB-D §9.1, which named the pattern set as the last
"no". It is not a "no" any more. Counting the rules a port would need for the
integer scalar operator set:

| what | rules |
|---|---|
| the 13 per-operator tables × the 8 integer type indices actually used (1,2,4,6,7,8,10,12) | **~40 table entries**, but they collapse to **13 operator rules + 1 signedness bit** because every table is constant across the signed rows and across the unsigned rows |
| the constant-operand tests (`ori` vs `or`; `cmpi` vs `cmp` with the two 16-bit fit rules; `mulli` vs `mullw`; power-of-two mul; power-of-two signed div; AND→`rlandi`) | **~9** |
| record-form fusion (opcode+1 gated on attribute bit `0x10`, compare-against-zero, backwards walk) | **1** |
| the CR field (`cr6` default, `cr0` on the record-form path) | **1** |
| the value-producing relational: the condition-code normalisation, the two expanders, and the word-count tie-break | **~12** |
| the narrowing fusions (`lhz`+`extsh` → `lha`; `lbz`+`extsb` stays) | **2** |
| **total** | **≈ 38 core + ~20 for the relational family = ~60** |

**P0.3's "under 120" hits, comfortably.** And P4.5's "under 120 selection arms
for the integer scalar operator set" hits with room: the dispatch has **41 arms
for everything**, VMX and floats included.

### 7.2 What a general `lower_expr` needs, in order

1. **The type index** (§2.1) on every operand — the port already models C types,
   so this is a 26-line mapping and needs no address.
2. **The 13 operator tables** (§2.2). Black-box re-derivable one cell each; the
   table VAs are only needed if the *numbers* are copied.
3. **The constant-operand tests** (§2.4). All black-box.
4. **`rlandi`'s expansion** — **the blocker**. Unread. Until it is read, any
   expression containing `x & K` is a coin flip between `rlwinm` and `li`+`and`.
5. **The if-conversion predicate** (§6.3c) — needed *before* selection, and this
   lane's 8 cells give the rule but not its site.
6. **Then** WB-D §3.4's register rule, which is still free and still last.

### 7.3 The first general class, and its predicted reach

**First class: `expr_int_straightline`** — a single basic block, integer scalar
operators only, **no `&`/`|`/`^` with a constant** (item 4 above), no calls, no
memory beyond loads and stores of locals and parameters, no relational producing
a value. Everything in it is one table lookup plus at most one constant test,
and cells S5–S10 show six different shapes of it coming out byte-exact from the
reading alone.

**Predicted reach on the 124-TU reach pool: `0`.** PREREG **P5.2 registered
exactly this and it stands** — not because the class is wrong but because 48 of
the frontier's 59 functions die at the port's IL reader before any emitter
question is reachable (WB-A; WB-D §9.3/P5.4; WB-H §9.1). A selection rule set is
a **capability**, priced as infrastructure, exactly like WB-D's register rule
and WB-H's three loop passes.

What has changed is the *price*, and it has changed a lot: **the three
generator lanes together now specify a complete scalar-integer emitter**
— selection (§2, this lane), order (WB-D §4, no scheduler), registers (WB-D
§3.4), loops (WB-H §5), frames (WB-B) — with **exactly two named holes**:
`rlandi`'s expansion and the switch-decision-tree pivot algorithm. That is a
much smaller residue than "the pattern set is unread".

### 7.4 Explicitly declined

* **Anything containing `&`, `|` or `^` with a constant** as a *byte-exact*
  class, until `FUN_10c1772b` is read. `|` and `^` are safe (§2.4, cell S8) but
  `&` is not, and a class predicate that admits two of three is a trap.
* **The value-producing relational** as the *first* class. It is the most
  interesting thing here and the reading survived S1's absence claims, S3, S4
  and S11's bias clause — but the cost tie-break between two expanders means a
  port must implement **both** to get either right, and S1 and S11 both missed
  on form. Second class, not first.
* **The signedness flip** (§2.5). Read, never obj-checked by this lane.
* **The switch decision tree.** Still unread; WB-H §5.1 closed the *order*
  question and this lane did not touch the pivot algorithm.

---

## 8. PREREG SCORE

`H` hit · `M` miss · `U` unscoreable (premise did not occur).

| # | prediction | verdict | note |
|---|---|---|---|
| P0.1 | the floor is cleared | **H** | §6, six independent cells |
| P0.2 | the carry idiom is selected directly, not a peephole over a compare | **H** | §3.1 — it is a dedicated tuple opcode (`0x2ea`) with its own expander |
| P0.3 | judgment = "yes", pattern set **under 120 rules** | **H** | §7.1, ≈60 |
| P1.1 | selection is table-driven at least in part, indexed by (opcode) or (opcode × type) | **H** | §2.2 — 13 tables × 26 type indices |
| P1.2 | `cgintrin.c` is **not** the main operator selector | **H** *(registered pessimistic)* | the dispatch is `0x10c0f882` in the `lower.c` band |
| P1.3 | the main selection lives in `lower.c` / `lowersmd.c` | **H, partial** | the dispatch and every per-operator arm do; the **idiom layer** does not (`0x10c19xxx`–`0x10c1bxxx`) |
| P1.4 | the final expansion switch is a **different, later** pass, dispatching through an `.rdata` jump table whose VA this lane names | **H** | `FUN_10c182b4` @ `0x10c182b4`, index `0x10c184a8`, table `0x10c18460` — §4. It expands pseudo-ops and narrowings, not `+`/`<`, as predicted |
| P1.5 | an image-resident mnemonic table indexed by c2's machine-opcode number, the Rosetta stone | **H** | `0x10b1b260` — §1. It was the first thing found and everything else followed from it |
| P1.6 | the arithmetic/compare tuple opcodes are dense in one contiguous band | **H** | `0x2c5`…`0x2d4`; the dispatch literally switches on `op − 0x27e` |
| P2.1 | not a generic bottom-up rewriter; hand-written `switch` + `if`-ladders | **H, qualified** | hand-written, yes — but there **is** a cost model (§3), two-way and word-counting. Registered as "no BURG cost table"; that half is right, "no cost table at all" would have been wrong |
| P2.2 | 16-bit **signed** fit for `-i` forms, **u16** fit for the logical forms | **M** | the asymmetry is real and exactly as described (§2.4) but it lives in the **compare** lowering, not the logical one — `ori`/`xori` have **no fit test at all** (cell S8) |
| P2.3 | `*` by a constant strength-reduced to shifts/adds below a threshold | **M** *(registered optimistic)* | power of two → shift, everything else → `mulli`. No decomposition, no threshold |
| P2.4 | `/` and `%` by a constant → magic-number multiply | **M** | **refuted by cell S6**: `x / 3` is `li 11,3` + `divw`. No `mulhw` anywhere at `/O1` |
| P2.5 | an idiom recogniser fires on a **combination** | **H** | §3 — and it is the only one in the scalar integer set |
| P3.1 | mechanism: carry-setting subtract, then `subfe` producing a 0/−1 mask, then one arithmetic fixup | **H** | §3.1 and cell S1 — `subc` · `subfe rD,rD,rD` · mask · `addi` |
| P3.2 | the idiom is **unsigned-only**; a signed value-producing `<` does not get it | **H** | §3.1's `return 500` for codes 3–6, and cell S2 |
| P3.3 | it generalises over arbitrary result constants `A`/`B`: same mask, different fixup | **H** | §3.1, cells S1 and S11 |
| P3.4 | **not** taken when the comparison feeds a branch; the selector has a value-vs-branch context bit | **M — RETRACTED** | §6.3c. Cell S12 emitted the idiom *with no branch at all*. The context that matters is the shape of the arms, and it is decided **before** selection |
| P3.5 | `x < K` with a **variable** `K` also gets a carry idiom, via `subfc` | **H** | §3.1 — `subfc` (`0x183`) is the default path; `subfic` is the special case |
| P3.6 | the idiom is controlled by an optimisation level or a `-QX` switch, isolable by a counterfactual | **U** | no `-QX` flag reaches `FUN_10c1b517`; this lane found no switch to flip and did not run the counterfactual |
| P4.1 | WB-H §9 item 3's narrowing holds — `+ − * ^ & \|` and the shifts are one-to-one table lowerings | **H** | §2.2, §3.3 |
| P4.2 | signed `>>` by a constant is `srawi` alone; signed `/` by a power of two adds `addze` | **H** | §2.4 and cell S5 — `srawi` + `addze`, exactly |
| P4.3 | `char`/`short` narrowing emits `extsb`/`extsh`; `unsigned char` narrowing emits `rlwinm` (`clrlwi`), not `andi.` | **H** | cells S9, S7; and the convert table `0x10b1fd08` |
| P4.4 | record forms are a **fusion**, not a selection — a peephole fuses `op` + `cmpwi rX,0` into `op.` | **H** | §2.6, `FUN_10c0b300` — a backwards walk from a compare-against-zero, gated on attribute bit `0x10`, with record form = opcode + 1 |
| P4.5 | under 120 distinct selection arms for the integer scalar operator set | **H** | 41 arms for **everything** |
| P5.1 | the judgment is "yes with a named boundary", and the boundary is set by the **idiom recognisers**, not the per-operator table | **M, and the miss is the useful part** | the boundary is set by the **`rlandi` expander** — a *pseudo-op expansion*, not an idiom recogniser, and it sits under `&`-with-a-constant, one of the most common things in any body |
| P5.2 | first class `expr_straightline_int`, predicted reach on the 124-TU pool = **0** | **H** | §7.3 |
| P5.3 | a general `lower_expr` is under 800 lines of Rust for the integer operator set | **U** | this lane wrote no Rust and priced rules, not lines; §7.1's ~60 rules is the honest form. Not claimed as a hit |
| P5.4 | at least one operator readable but **not predictable** without a further unread pass, named rather than papered over | **H** | `rlandi` / `FUN_10c1772b`, §6.3b |
| P6.1 | at `/O1` the graded cells survive folding when operands come from parameters; checked by a size-only calibration | **H** | §5.1 — and calibration earned its keep three times |
| P6.2 | at least **2** graded cells will MISS | **H** *(registered pessimistic)* | exactly 3 |
| P6.3 | register choice follows WB-D §3.4 unchanged, so a word-exact miss is a *selection* miss not a register miss | **H** | §6 — every cell used `r11` then `r10`, result `r3` |

**Score: 25 H · 6 M · 2 U.** Of the six misses, **two were registered
optimistic** (P2.3, P6.2 is a pessimistic hit) and four were not (P2.2, P2.4,
P3.4, P5.1) — of which **P3.4 is a retraction** (§6.3c) and P2.4 is a
refutation by a cell that was in the grid *because* calibration flagged it.

Board #770's streak gains **1 optimistic (P2.3)** and **1 pessimistic (P1.2)**.

---

## 9. Pre-drafted DISCLOSURE rows

Per `DISCLOSURE.md` step 5 the black-box alternative is preferred, and **for
this lane it is unusually strong**: `select_grid.cpp` + `calib.cpp` +
`diag.cpp` re-derive the operator tables' *behaviour*, the signedness split,
the `srawi`+`addze` idiom, the `lha` fusion, the `clrlwi` form, the
`oris`+`ori` form, the absence of a magic-number multiply, and the whole carry
idiom **with no address at all**. A code lane that ships the §7.3 class needs
**no row**.

| # | Kind | What would be adopted | Address in `c2.dll` | Adopted into | Commit | Notes |
|---|---|---|---|---|---|---|
| **W-SELECT-1** | **route** | **The machine-opcode enum is an alphabetically-ordered table whose index is c2's opcode number, with a per-opcode attribute word; record form = opcode + 1, gated on attribute bit `0x10`; bit `0x20` = defines XER[CA], `0x40` = uses it.** | **`0x10b1b260`** (the 12-byte-stride table), `0x10b19700`–`0x10b1b25f` (the mnemonic pool), **`0x10c3afd8`** (the byte-wide attribute array the lowering indexes), `0x10b1d190` (the simplified-mnemonic table) | *(nothing — this lane adopts no code)* | *(pending)* | **No obj exposes these numbers**; they are c2's internal encoding and a port has its own. The row exists because §1–§4's readings are defended by quoting them. A port that says "record form = the `.` variant, when one exists" needs no row. |
| **W-SELECT-2** | **adoption-ready** | **The 13 per-operator opcode tables and the operand type index**: operator × type → PPC opcode, with the signed/unsigned split confined to `div`, `cmp` and the compare-immediate. | **`0x10c04cb9`** (the installer), `0x10c38f30`, `0x10c38f98`, `0x10c39068`, `0x10c39138`, `0x10c391a0`, `0x10c392d8`, `0x10c39340`, `0x10c393a8`, `0x10c39410`, `0x10c39478`, `0x10c394e0`, `0x10c39548`, `0x10b1fd08` (the tables); `0x10bd7c10` + `0x10bd7cf0` (the type index) | *(nothing)* | *(pending)* | **A black-box re-derivation exists and should be preferred**: one cell per operator × signedness in `grids/wb-select/`. Carry this row only if the table *contents* or the type-index *numbering* are copied. |
| **W-SELECT-3** | **route** | **A value-producing relational is lowered by two rival hand-written expanders run first in cost-only mode, cheaper one wins, ties to `cntlzw`.** The carry expander is unsigned-only (condition codes 3–6 return "impossible"), normalises every unsigned relation to `ULE` by swapping, and emits `[li] · subfc/subfic+CA · lcarry · [rlandi A−B] · [addi B]`. | **`0x10c1b517`** (the chooser), **`0x10c1ac5c`** (carry), **`0x10c1af2d`** (`cntlzw`), `0x10c1a908` (the against-zero path), `0x10c194b8` (float), `0x10b189a4` (the signed→unsigned condition remap), `0x10b197c0`–`0x10b197f4` (the condition-code names) | *(nothing)* | *(pending)* | **The RULE is black-box** — cells S1, S2, S4, S11 and `diag.cpp`'s mask ladder exhibit it against real `c2.dll` with no address. **The COST TIE-BREAK is not**: no obj in this project separates "cntlzw was cheaper" from "carry was impossible". Grey-zone rule applies to the tie-break only. |
| **W-SELECT-4** | **route** | **The per-operator dispatch is a `switch` on the tuple opcode with 41 arms over 174 opcodes**, and the in-place machine-opcode expansion pass has 18. | **`0x10c0f882`** (the dispatch), `0x10c0fbd6` + `0x10c0fb32` (its index and jump table), **`0x10c182b4`** (the expansion switch), `0x10c184a8` + `0x10c18460` (its index and jump table), and the arms `0x10c0634b` (add), `0x10c064cb` (sub), `0x10c067f1` (mul), `0x10c068ee` (div), `0x10c0711d` (and), `0x10c0718f` (or/xor), `0x10c0eb17` (compare), `0x10c0f1ed` (convert) | *(nothing)* | *(pending)* | Navigation. The *counts* (41, 18) are the load-bearing claim for §7.1's pattern-set size and cannot be obtained black-box. |
| **W-SELECT-5** | **navigation, held** | **`FUN_10c1772b` @ `0x10c1772b` is the `rlandi` expander — the one unread pass between this lane's reading and a byte-exact `lower_expr`.** | `0x10c1772b`, reached from expansion-switch arm 13 (`0x10c183a3`) | *(nothing)* | — | **Not adoptable — deliberately.** Recorded so the next lane starts here instead of re-finding it. §6.3b is the obj evidence that bounds it. |

**Held, not proposed.** The signedness flip (§2.5, `FUN_10c07803` and the
nibble rewrite in `0x10c0eb17`) is read and **never obj-checked** by this lane.
So are the operand-format field of `0x10b1b260` and the four `-QVMX128`
alternate tables.

**Not claimed.** This lane did not read the if-conversion pass that produces
tuple `0x2ea` (§6.3c gives its rule from 8 objs, not its site), the switch
decision tree's pivot algorithm, `FUN_10c1772b` itself, the remainder lowering
`FUN_10c06eb1`, the shift lowerings, `cgintrin.c`'s intrinsic expanders, or any
floating-point or VMX path.
