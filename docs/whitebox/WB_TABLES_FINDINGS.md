# WB-J `wb-tables` — the two WB-I disagreements SETTLED, and the `rlandi` pass READ

> **PROVENANCE — DISASSEMBLY-DERIVED.** Obtained by statically disassembling
> Microsoft's `c2.dll` — the exact image pinned in
> [`C2_MAP_METHOD.md`](C2_MAP_METHOD.md) §0, sha256 verified at the top of this
> lane as `c80981c015166effecc71ad8112d5577a065b2300891dfdb02b9c13787a66258`
> (the `c2.dll` every obj here was compiled against hashes the same). It is
> **navigation only** until a row is added to [`DISCLOSURE.md`](DISCLOSURE.md).
> **The obj is the sole judge** (method doc §7): §3.7 grades this lane's
> reading against real `c2.dll` under wibo, and §3.8 records what the objs
> refuted — including two claims made by the two lanes this one was sent to
> arbitrate.

Lane `wb-tables` (WB-J), campaign 2 (`CAMPAIGN_2026-08-08_GENERATORS.md`),
2026-08-09. Board rows **#2110–#2129**. PREREG:
[`WB_TABLES_PREREG.md`](WB_TABLES_PREREG.md), committed in `48b6de19` **as this
lane's first commit, before the first grep of the flat export** (the two
disclosed exceptions are in its header). Grid:
[`grids/wb-tables/mask_grid.cpp`](grids/wb-tables/mask_grid.cpp) +
[`frozen.tsv`](grids/wb-tables/frozen.tsv), committed in `c609b6f9` **before the
grid's first `cl.exe`**. Calibration:
[`grids/wb-tables/calib.cpp`](grids/wb-tables/calib.cpp) (19 cells) plus a
scratch second round in `work/wb-tables/calib2.cpp` (11 cells) — both
**unscored**, both compiled before the grid was written (§3.6).

This lane was commissioned by ROADMAP §10.27.1 to settle two open items between
two independent readings of the same question, and to read the one pass neither
could predict. It settles both, reads the pass, and **corrects one claim in
each prior document** — in place, dated, never by silent rewrite.

---

## 0. The answer in one screen

| the question | the answer |
|---|---|
| **How many operator × type tables?** | **13 installed pointer slots, 17 distinct table bodies, 16 of them contiguous in one `.data` block.** Both runs are partly right and neither is complete. Run 1's *sixteen* is the body count of the `.data` block and **omits the convert/widen table entirely**; run 2's *thirteen* is the slot count and is right. §1. |
| **10/12 vs 9/12 on the frozen grids?** | **Not comparable, and not a disagreement.** Both cell lists reproduce **24 of 24** published emissions against one obj run at the workload flags; `/Gy` changes not one word. The lists share **3** cells and are disjoint on the other 21, and each lane was blind to the other's sharpest claim — the `w-memfit` shape, third instance. §2. |
| **`FUN_10c1772b` — the `rlandi` decider?** | **No. It is a peephole COMBINER, and run 2's §4 misidentified it.** The `rlandi` expander is **`FUN_10c0a2e2` @ `0x10c0a2e2`**, which is the site run 1 named. §3. |
| **What decides `x & K`'s form, then?** | **Two rules, both obj-confirmed.** (S) the **mask shape**: contiguous → `rlwinm`, non-contiguous-but-16-bit → **`andi.`**, else the residue decomposition. (B) inside the relational idiom, the mask step is **`li`+`and` iff there is no `addi` bias**, whatever the mask. §3.4–§3.5. **11 of 12 frozen cells.** |
| **Is W-SELECT-3 really the only row where black-box is insufficient?** | **Confirmed for the tie-break, and corrected as a general statement**: the **table count and the slot map** are equally out of black-box reach, and they are a *precondition* on the adoption-ready row W-SELECT-2. §4. |

---

## 1. THE TABLE COUNT — settled by enumeration (deliverable 1)

### 1.1 The installer, counted in one sitting

`FUN_10c04cb9` @ **`0x10c04cb9`** is 180 bytes and contains **exactly 17
`mov DWORD PTR ds:<slot>, <imm32>` stores** and nothing else:

```
10c04cb9  cmp  DWORD PTR ds:0x10c2e978, 0x0      ; the -QVMX128 word, tested FIRST
10c04cc0  mov  ds:0x10c6fddc, 0x10c38f30         ;  1  copy / move
10c04cca  mov  ds:0x10c6fdd8, 0x10c38f98         ;  2  load,  D-form
10c04cd4  mov  ds:0x10c6fdd4, 0x10c39068         ;  3  load,  X-form
10c04cde  mov  ds:0x10c6fdd0, 0x10c39138         ;  4  store, D-form
10c04ce8  mov  ds:0x10c6fdcc, 0x10c391a0         ;  5  store, X-form
10c04cf2  mov  ds:0x10c6fdc8, 0x10c392d8         ;  6  negate
10c04cfc  mov  ds:0x10c6fdc4, 0x10c39340         ;  7  add
10c04d06  mov  ds:0x10c6fdc0, 0x10c393a8         ;  8  subtract
10c04d10  mov  ds:0x10c6fdbc, 0x10c39410         ;  9  multiply
10c04d1a  mov  ds:0x10c6fdb8, 0x10c39478         ; 10  divide
10c04d24  mov  ds:0x10c6fdb4, 0x10c394e0         ; 11  compare, immediate
10c04d2e  mov  ds:0x10c6fdb0, 0x10c39548         ; 12  compare, register
10c04d38  mov  ds:0x10c6fdac, 0x10b1fd08         ; 13  CONVERT / WIDEN   <-- run 1 has no row for this
10c04d42  je   0x10c04d6c
10c04d44  mov  ds:0x10c6fdd8, 0x10c39000         ; 2' load,  D-form -QVMX128
10c04d4e  mov  ds:0x10c6fdd4, 0x10c390d0         ; 3' load,  X-form -QVMX128
10c04d58  mov  ds:0x10c6fdd0, 0x10c39208         ; 4' store, D-form -QVMX128
10c04d62  mov  ds:0x10c6fdcc, 0x10c39270         ; 5' store, X-form -QVMX128
10c04d6c  ret
```

**13 distinct destination slots** (`DAT_10c6fdac`…`DAT_10c6fddc`, stride 4) and
**17 distinct source table bodies**. The last four stores **overwrite four of
the first thirteen** — they add no slot.

### 1.2 So both runs are right about a different number, and both are incomplete

The `.data` block is contiguous at a `0x68` stride, and there are **sixteen
bodies in it**:

| # | VA | table | | # | VA | table |
|---:|---|---|---|---:|---|---|
| 0 | `0x10c38f30` | copy | | 8 | `0x10c39270` | store-X `-QVMX128` |
| 1 | `0x10c38f98` | load-D | | 9 | `0x10c392d8` | negate |
| 2 | `0x10c39000` | load-D `-QVMX128` | | 10 | `0x10c39340` | add |
| 3 | `0x10c39068` | load-X | | 11 | `0x10c393a8` | subtract |
| 4 | `0x10c390d0` | load-X `-QVMX128` | | 12 | `0x10c39410` | multiply |
| 5 | `0x10c39138` | store-D | | 13 | `0x10c39478` | divide |
| 6 | `0x10c391a0` | store-X | | 14 | `0x10c394e0` | compare, immediate |
| 7 | `0x10c39208` | store-D `-QVMX128` | | 15 | `0x10c39548` | compare, register |

`(0x10c395b0 − 0x10c38f30) / 0x68 = 16`. The **seventeenth** body is
`0x10b1fd08` — the convert/widen table — and it is not in `.data` at all, it is
in `.text`.

> **The verdict.** Run 1's **sixteen** is the count of bodies in the `.data`
> block; its *list* is "twelve named plus four `-QVMX128`" and it therefore
> **has no entry for the convert/widen table**, which is the one that carries
> `extsb`/`extsh`/`extsw`. Run 2's **thirteen** is the count of installed
> slots and is correct, and its list is complete. **Run 2 wins the count;
> run 1's number is a real count of a different thing; run 1's LIST is the
> thing that would have silently dropped an operator.**

### 1.3 Every table, decoded (deliverable 1, the enumeration)

Decoded through the machine-opcode table `0x10b1b260` (12-byte stride, 665
records: index 0 `_first`, 661 `_last`, 662 `illegal`). Rows are the 26 slots;
`—` is the value `0`, `‡` is the value **663**, a *second* `_last` string used
by these tables as an explicit "this operator does not exist for this type".

| slot | 1 | 2 | 4 | 6 | 7 | 8 | 10 | 12 | 13 | 14 | 16 | 17 | 20 | 21 | 22 | 23 | 25 |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| **copy** `0x10c38f30` | `mr` | `mr` | `mr` | `mr` | `mr` | `mr` | `mr` | `mr` | `fmr` | `fmr` | `mr` | `mr` | `mr` | `mr` | `mr` | `mr` | `vmr` |
| **load-D** `0x10c38f98` | `lbz` | `lhz` | `lwz` | `ld` | `lbz` | `lhz` | `lwz` | `ld` | `lfs` | `lfd` | `lwz` | `lwz` | `lbz` | `lhz` | `lwz` | `ld` | `lvx` |
| **load-X** `0x10c39068` | `lbzx` | `lhzx` | `lwzx` | `ldx` | `lbzx` | `lhzx` | `lwzx` | `ldx` | `lfsx` | `lfdx` | `lwzx` | `lwzx` | `lbzx` | `lhzx` | `lwzx` | `ldx` | `lvx` |
| **store-D** `0x10c39138` | `stb` | `sth` | `stw` | `std` | `stb` | `sth` | `stw` | `std` | `stfs` | `stfd` | `stw` | `stw` | `stb` | `sth` | `stw` | `std` | `stvx` |
| **store-X** `0x10c391a0` | `stbx` | `sthx` | `stwx` | `stdx` | `stbx` | `sthx` | `stwx` | `stdx` | `stfsx` | `stfdx` | `stwx` | `stwx` | `stbx` | `sthx` | `stwx` | `stdx` | `stvx` |
| **negate** `0x10c392d8` | `neg` | `neg` | `neg` | `neg` | `neg` | `neg` | `neg` | `neg` | `fneg` | `fneg` | `neg` | `neg` | `neg` | `neg` | `neg` | `neg` | ‡ |
| **add** `0x10c39340` | `add` | `add` | `add` | `add` | `add` | `add` | `add` | `add` | `fadds` | `fadd` | `add` | `add` | `add` | `add` | `add` | `add` | `vaddfp` |
| **subtract** `0x10c393a8` | `subf` | `subf` | `subf` | `subf` | `subf` | `subf` | `subf` | `subf` | `fsubs` | `fsub` | `subf` | `subf` | `subf` | `subf` | `subf` | `subf` | **`vsubfp`** |
| **multiply** `0x10c39410` | `mullw` | `mullw` | `mullw` | `mulld` | `mullw` | `mullw` | `mullw` | `mulld` | `fmuls` | `fmul` | `mullw` | `mullw` | `mullw` | `mullw` | `mullw` | `mulld` | ‡ |
| **divide** `0x10c39478` | `divw` | `divw` | `divw` | `divd` | **`divwu`** | **`divwu`** | **`divwu`** | **`divdu`** | `fdivs` | `fdiv` | **`divwu`** | **`divwu`** | `divwu` | `divwu` | `divwu` | `divdu` | ‡ |
| **cmp-imm** `0x10c394e0` | `cmpi` | `cmpi` | `cmpi` | `cmpi` | **`cmpli`** | **`cmpli`** | **`cmpli`** | **`cmpli`** | `fcmpu` | `fcmpu` | **`cmpli`** | **`cmpli`** | `cmpli` | `cmpli` | `cmpli` | `cmpli` | ‡ |
| **cmp-reg** `0x10c39548` | `cmp` | `cmp` | `cmp` | `cmp` | **`cmpl`** | **`cmpl`** | **`cmpl`** | **`cmpl`** | `fcmpu` | `fcmpu` | **`cmpl`** | **`cmpl`** | `cmpl` | `cmpl` | `cmpl` | `cmpl` | ‡ |
| **convert** `0x10b1fd08` | `extsb` | `extsh` | `extsw` | `mr` | `extsb` | `extsh` | `extsw` | `mr` | ‡ | ‡ | `mr` | `mr` | `extsb` | `extsh` | `extsw` | `mr` | ‡ |

The four `-QVMX128` alternates differ from their originals in **slot 25 only**
(`lvx`→`lvx128`, `stvx`→`stvx128`) — run 2's claim, confirmed entry by entry.

**Two entries neither prior document carries**, both from this enumeration:
**subtract's slot 25 is `vsubfp`** (run 1's decode table prints `—` for the
whole `subtract` row and run 2's has no slot-25 column), and the value **663**
is a *distinct* "no such operator" marker from `0`, used in seven places.

### 1.4 The slot map, which is the part that actually drops operators

`FUN_10bd7c10` @ **`0x10bd7c10`** turns an operand's 16-bit type word into the
table index: `jmp [0x10bd7cf0 + 4·(type >> 12)]`, then a size ladder per arm.
The jump table has **exactly 13 entries** (`0x10bd7cf0` … `0x10bd7d23`, and
`0x10bd7d24` is the next function), so the type nibble is `0..12`. Decoded:

| nibble | meaning | size → index |
|---:|---|---|
| 0 | — | `0` |
| **1** | **signed integer** | 1→1, 2→2, 4→4, 6→**5**, else→**6** |
| **2** | **unsigned integer** | 1→7, 2→8, 4→10, 6→**11**, else→**12** |
| 3 | pointer | 4→16, 6→18, else→19 |
| 4 | pointer, second flavour | 4→17, 6→18, else→19 |
| **5** | **floating point** | 4→13, 8→14, else→15 |
| 6 | fourth integer family | 1→20, 2→21, 4→22, 8→23, else→24 |
| 8 / 9 / 10 / 11 | — | **29 / 26 / 27 / 28** |
| 12 | VMX | 25 |

Three facts fall out, and all three are load-bearing for an adoption:

1. **Only 17 of the 26 slots are ever live**, and it is the *same* 17 in every
   one of the 17 tables: `1, 2, 4, 6, 7, 8, 10, 12, 13, 14, 16, 17, 20, 21, 22,
   23, 25`. Seven more (`0, 5, 11, 15, 18, 19, 24`) are **reachable by the type
   index and zero in every table** — a 6-byte integer, an odd-sized float, an
   odd-sized pointer have no opcode anywhere. Two (`3`, `9`) are **not
   reachable at all**.
2. **The index space is `0..29` and the arrays are 26 entries.** Nibbles 8, 9,
   10 and 11 return **29, 26, 27, 28** — *off the end of every table*. Nothing
   in `FUN_10bd7c10` bounds them. A port that models "26 entries indexed by the
   type index" is correct only because those nibbles never reach an arithmetic
   selector; it must **refuse** them rather than index.
3. **`ld`/`std`/`mulld`/`divd` sit at slot 6, the `else` arm** — reached by
   *any* size that is not 1, 2, 4 or 6, which is how an 8-byte integer gets
   there. Run 2's §2.1 size ladder is right and its §2.2 column header (`6` =
   i64) is right; the two are consistent only once you know 6 is the `else`.

---

## 2. THE GRID DISAGREEMENT — re-graded, and NOT COMPARABLE (deliverable 2)

### 2.1 One obj run, both cell lists, at the workload flags

Both grid sources were compiled from this lane's worktree against the same
`c2.dll` (`c80981…6258`) under wibo at **`/nologo /c /GR /O1 /Oi /EHsc`** —
run 2's flags, WB-D's workload mode — and again with run 1's extra `/Gy`.

* **All 24 published emission listings reproduce exactly.** Not one word of
  either document's results section is wrong. There is no drift and there is no
  transcription error.
* **`/Gy` changes not one instruction word** in either grid (diffed
  mechanically, `work/wb-tables/dumpall.sh`). It changes COMDAT sectioning
  only. The flag difference between the two runs is therefore **not** a
  candidate explanation for anything.

### 2.2 The reconciled table

`P` = the cell's own lane's published verdict, re-checked here. **No verdict is
changed in any lane's favour**; the only column this lane adds is the last one.

| cell | source | emitted | lane 1 | lane 2 | in the other grid? |
|---|---|---|---|---|---|
| `wbs_s1` | `x<10u ? 1:2` | `li·subc·subfe·addi` | **HIT** | — | near-pair of `S1` |
| `wbs_s2` | `a<b ? 1:2` | `subc·subfe·addi` | **HIT** | — | **absent** |
| `wbs_s3` | `x>7u ? 12:4` | `subfic·subfe·rlwinm·addi` | **HIT** | — | **absent — and it is the other half of `S11`** |
| `wbs_s4` | `x==0 ? 5:6` | `cntlzw·rlwinm·xori·addi` | **HIT** (the 4–4 tie) | — | **absent** |
| `wbs_s5` | `int x<10 ? 1:2` | `cmpwi cr6·li·bclr·li` | **HIT** | — | ≡ `S2` |
| `wbs_s6` | `int x<0 ? 1:2` | `cntlzw·cntlzw·rlwinm·xori·addi` | **MISS** (word count) | — | **absent** |
| `wbs_b1` | `(int)(x<10u)` | `li·subc·subfe·clrlwi` | **HIT** | — | **absent** |
| `wbs_b2` | `(int)(a>b)` | `subc·subfe·clrlwi` | **HIT** | — | **absent** |
| `wbs_b3` | `(int)(x<0)` | `srwi 3,3,31` | **MISS** (retracted) | — | **absent** |
| `wbs_k1` | `x/8` | `srawi·addze` | **HIT** | — | **≡ `S5`** |
| `wbs_k2` | `x & 0xFF` | `clrlwi 3,3,24` | **HIT** | — | **≈ `S7`** |
| `wbs_k3` | `(a<b)+c` | `subc·subfe·clrlwi·add` | **HIT** | — | **absent** |
| `S1` | `x<10u ? 7:3` | `li·subc·subfe·rlwinm·addi` | — | **MISS** (`subfic` predicted) | near-pair of `wbs_s1` |
| `S2` | `int x<10 ? 7:3` | `cmpwi cr6·li·bclr·li` | — | **HIT** | ≡ `wbs_s5` |
| `S3` | `x==0` | `cntlzw·rlwinm` | — | **HIT** | **absent** |
| `S4` | `x!=0` | `addic·subfe` | — | **HIT** | **absent** |
| `S5` | `x/8` | `srawi·addze` | — | **HIT** | **≡ `wbs_k1`** |
| `S6` | `x/3` | `li·divw` | — | **HIT** | **absent** |
| `S7` | `x & 0xffu` | `clrlwi 3,3,24` | — | **HIT** | **≈ `wbs_k2`** |
| `S8` | `x \| 0x12345` | `oris·ori` | — | **HIT** | **absent** |
| `S9` | `p[0]+1`, `signed char` | `lbz·extsb·addi` | — | **HIT** | **absent** |
| `S10` | `p[0]+1`, `short` | `lha·addi` | — | **HIT** | **absent** |
| `S11` | `x<10u ? 8:0` | `li·li·subc·subfe·and` | — | **MISS** (retracted) | **absent — and `wbs_s3` is the other half** |
| `S12` | `if(x<10u) return 1; return 2;` | `li·subc·subfe·addi` | — | **MISS** (P3.4 refuted) | **absent** |

**Overlap: 3 of 24** (`wbs_k1`≡`S5`, `wbs_k2`≈`S7`, `wbs_s5`≡`S2`). On all
three, the two objs **agree** and both lanes scored a hit. **21 of 24 cells
appear in exactly one grid.**

### 2.3 The verdict, and it is a legitimate one

> **The two scores are NOT COMPARABLE, and "10/12 vs 9/12" is not a
> disagreement about a measurement — it is two different measurements of two
> different cell sets, each of which reproduces perfectly.** Both stand.
> Combined on one denominator this lane can state: **19 of 24 core, 5 misses**,
> and every miss is in the document that owns it.

This is the third time this exact shape has been found (`w-memfit`, board
#2062, and now here), and it is sharper here than there, because the two grids
are not merely disjoint — **each lane's grid contained exactly one half of the
pair that decides this lane's §3.5 rule, and neither had both**:

* run 1's `wbs_s3` (`x>7u ? 12:4`) is a relational mask **with a bias**, and
  came out `rlwinm` — which run 1 read as "a contiguous mask is always
  `rlwinm`";
* run 2's `S11` (`x<10u ? 8:0`) is the **same mask with no bias**, and came out
  `li`+`and` — which run 2 read as "the form is unpredictable".

Neither reading is right. Both cells are, and §3.5 is the rule that produces
both.

---

## 3. `FUN_10c1772b`, AND WHAT ACTUALLY DECIDES `x & K` (deliverable 3)

### 3.1 `FUN_10c1772b` is not the expander — run 2's §4 is corrected

`FUN_10c182b4` @ `0x10c182b4`, which run 2 called "the in-place expansion
switch", is called **once**, from `FUN_10b7dd2c` @ `0x10b7dd2c`, a pass-pipeline
driver, and it is gated on `DAT_10c2e2fc`. Its body is

```
local_c = 2;                       /* the whole instruction list, TWICE */
do { for each instruction: switch (byte_at(0x10c184a8 + opcode - 1)) { ...18 arms... }
     local_c--; } while (local_c);
```

and every arm takes `&local_8` — the *next instruction to visit* — which is the
signature of a **peephole combiner that may delete an instruction and back
up**, not of an expander. Arm 13 is `FUN_10c1772b(&local_8, insn)`.

`FUN_10c1772b` @ **`0x10c1772b`** (1007 bytes) takes an `rlandi` and looks
**backwards** at the instruction that defined its source
(`FUN_10c16a46 @ 0x10c16a46`), then folds one of four ways:

| the def is | what happens |
|---|---|
| `mr` (626) | the copy is coalesced into the `rlandi` (`FUN_10c16cde`) and deleted |
| a **load** (attribute byte `& 7 == 2`) | the mask is **relaxed**: bits the load already guarantees zero are dropped, and the cheaper of {original, relaxed} by `FUN_10c0a170` is kept — **ties to the relaxed one** (`if (cost_orig < cost_relaxed) return;`) |
| `extsb` (91) / `extsh` (93) / `extsw` (95) | if the mask kills every bit the sign-extension could set, the extension is deleted |
| another `rlandi` (622) | the two (rotate, mask) pairs are **merged into one** |

It never mints `rlwinm`, `andi.`, `and` or `li`. **It cannot be the pass that
decides the form**, and run 2's §4/§6.3b/§7.2-item-4/W-SELECT-5 all rest on the
claim that it is.

### 3.2 The expander is `FUN_10c0a2e2` — run 1's site, and run 1 is right

`FUN_10c0a2e2` @ **`0x10c0a2e2`** (1871 bytes) has exactly two callers:
**`0x10c0dabc`, inside `FUN_10c0d57e`** — the in-place expansion switch WB-D §4
saw and run 1 §1.2 named — and `0x10c1cf75` inside the switch-lowering helper
`FUN_10c1cf59`. It is the routine that rewrites `rlandi` (622) into a real
instruction, and its two inputs are the **rotate** operand and the **mask**
operand of the `rlandi`.

Its decision rests on one small routine that neither prior run read:

> **`FUN_10c04daf` @ `0x10c04daf` is the rotate-mask decomposition.** Given a
> mask it fills in `MB`/`ME` and **returns `0xffffffff` exactly when the mask
> is a valid PowerPC rotate mask** — a contiguous run of 1s, wrapping allowed.
> When the mask is *not* contiguous it returns the **residue**: the mask with
> the gaps between its first and last run filled in.

and one cost function:

> **`FUN_10c0a170` @ `0x10c0a170` prices a mask in words** — 1 for zero, 1 if
> it fits `u16` or `(mask & 0xffff) == 0`, 1 if contiguous and non-wrapping, 2
> if contiguous and wrapping, 2 if the residue is contiguous, 3 otherwise. It
> is also what `FUN_10c1772b` calls to decide §3.1's tie.

### 3.3 The two rules, stated

**(S) THE SHAPE RULE — a plain `x & K`, 32-bit, rotate 0.** In `FUN_10c0a2e2`'s
own order of tests:

```
if (rotate != 0 || FUN_10c04daf(K) == 0xffffffff)   ->  rlwinm rD,rS,rot,MB,ME     (0x133/0x134)
else if ((K & 0xffff0000) == 0)                     ->  andi.  rD,rS,K             (gated on
else if ((K & 0xffff)     == 0)                     ->  andis. rD,rS,K>>16          !DAT_10c2ecf0
                                                                                    && !DAT_10c2e310)
else if (FUN_10c04daf(residue) == 0xffffffff)       ->  TWO rlwinm (mask, then residue)
else                                                ->  materialise K, then `and`  (0x19/0x1a)
```

**(B) THE BIAS RULE — the mask step inside a value-producing relational.**
`FUN_10c1ac5c` @ `0x10c1ac5c` emits `rlandi rD, 0, delta` **unconditionally**
whenever `delta != −1`, then emits `addi rD, rD, base` **iff `base != 0`**, and
then — at its tail — calls `FUN_10bd7b09` to rebind the **last** instruction's
destination to the select tuple's own destination operand. Empirically, over
**20 cells** at `/O1`:

> **`base != 0` (an `addi` follows) ⇒ the mask obeys rule (S).
> `base == 0` (the `rlandi` is the last instruction, and inherits the tuple's
> destination) ⇒ `li rT, delta` + `and rD, rS, rT`, ALWAYS, whatever the mask
> shape and whatever the registers.**

### 3.4 What is read, and what is inferred — stated separately

Rule (S) is **read**: every branch above is in `FUN_10c0a2e2`'s instructions.

Rule (B) is **obj-grounded on 20 cells and its causal step is NOT read.** The
`rlandi` is provably emitted in both cases (§3.3), and the only structural
difference this lane could find between them is `FUN_10bd7b09`'s destination
rebind. Two rivals remain **extensionally identical on every cell this lane can
construct**, and are therefore recorded as one rule with an unread mechanism
rather than resolved:

* **R-A** — an `rlandi` produced by the relational expander is expanded by rule
  (S) iff a bias instruction follows it;
* **R-B** — an `rlandi` whose destination was rebound to the select tuple's own
  destination operand is expanded to `li`+`and`.

Naming that as unread is the honest form. It is also **harmless for a port**:
the two rivals never disagree, so either statement produces the same bytes.

### 3.5 The two prior bounds, refuted

* **Run 2's black-box bound is REFUTED.** Its §6.1 states *"in the biased cells
  `rlandi`'s source and destination land in the same register and in the
  unbiased ones they do not"*, and §6.3b bounds the pass with it. Cell
  `d1_consumed` — frozen for exactly this — has `base == 0`, a **contiguous**
  mask, and source **and** destination both `r11`, and c2 emitted
  `li 10,8 · and 11,11,10`. Calibration cell `c_and_add` is the mirror:
  source `r3`, destination `r11` (**different**), and c2 emitted `rlwinm 11,3`.
  **The registers do not decide it in either direction.**
* **Run 1's frozen rival `R-M1` is CORRECTED.** Its §7.2 records *"`R-M1` beats
  `R-M2`, 5 for 5. A contiguous mask is always `rlwinm`, never `andi.` — c2
  does not clobber `CR0` for a value."* The first half is right and survives
  17 further cells. The second half is **wrong**: `andi.` is exactly what a
  **non-contiguous** 16-bit mask gets, and c2 clobbers `CR0` for a value
  without hesitation — `x & 0x8001` is `andi. 3,3,32769`. Run 1's grid contained
  no non-contiguous mask, so its five cells could not have seen it.

### 3.6 The calibration pass, and what it changed

[`calib.cpp`](grids/wb-tables/calib.cpp) (19 cells) and a scratch second round
(`work/wb-tables/calib2.cpp`, 11 cells) were compiled **before the grid was
written**, and are unscored. **Full disclosure, since it bounds what §3.7 is
worth**: they were read as full word sequences, not sizes — this lane's
predictions for rule (S) come from `FUN_10c0a2e2` and `FUN_10c04daf` and were
derivable before any compile, but rule (B) was **found in the calibration** and
then *explained*, not predicted cold. §3.7 therefore tests rule (B) as a
*generalisation* (five new relational cells at four new mask shapes) and rule
(S) as a *prediction* (six new mask shapes, none in either calibration file).

Calibration earned its keep three times:

* it **refuted run 2's register bound before the grid was frozen**
  (`c_two_8`, `c_and_add`), so the grid could be aimed at rule (B) instead of
  re-testing a dead rival;
* it produced `c_m_10001` → **two `rlwinm`s**, which is the residue path, and
  is why cells `m5_two` and `m6_four` exist to separate residue-depth 1 from 2;
* it showed `k_eqz_scale` (the `cntlzw` rival's own chain) obeying rule (B)
  too, so the rule is stated over the idiom layer and not over one expander.

### 3.7 RESULTS — 11 of 12 on the graded core

Predictions in [`frozen.tsv`](grids/wb-tables/frozen.tsv), committed `c609b6f9`
before the first `cl.exe` of `mask_grid.cpp`.

| cell | source | emitted | core | regs |
|---|---|---|---|---|
| `m1_run` | `x & 0x7f8` | `rlwinm 3,3,0,21,28` | **HIT** | **HIT** |
| `m2_low16` | `x & 0xffff` | `clrlwi 3,3,16` | **HIT** | **HIT** |
| `m3_split16` | `x & 0x8001` | `andi. 3,3,32769` | **HIT** | **HIT** |
| `m4_wrap` | `x & 0x80000001` | `rlwinm 3,3,0,31,0` | **HIT** | **HIT** |
| `m5_two` | `x & 0x00ff00ff` | `clrlwi 3,3,8` · `rlwinm 3,3,0,24,15` | **HIT** | **HIT** |
| `m6_four` | `x & 0xf0f0f0f0` | `lis 12,0xf0f0` · `ori 12,12,0xf0f0` · `and 3,3,12` | **HIT** | **miss — `r12`** |
| `d1_consumed` | `(x<10u?8:0)+y` | `li 11,10`·`li 10,8`·`subc`·`subfe`·`and 11,11,10`·`add 3,11,4` | **HIT — word for word** | **HIT** |
| `r1_bias16` | `x<10u ? 0x18:0x10` | `li`·`subc`·`subfe`·`rlwinm 11,11,0,28,28`·`addi 3,11,16` | **HIT — word for word** | **HIT** |
| `r2_pow_nb` | `x<10u ? 0x100:0` | `li 11,10`·`li 10,256`·`subc`·`subfe`·`and 3,11,10` | **HIT — word for word** | **HIT** |
| `r3_spl_nb` | `x<10u ? 0x101:0` | `li 11,10`·`li 10,257`·`subc`·`subfe`·`and 3,11,10` | **HIT — word for word** | **HIT** |
| `r4_regbnd` | `a<b ? 6:2` | `subc 11,3,4`·`subfe`·`rlwinm 11,11,0,29,29`·`addi 3,11,2` | **HIT — word for word** | **HIT** |
| `r5_bias1` | `x<10u ? 0x8001:1` | `cmplwi 6,3,10`·`bf 24`·`lis`·`ori`·`blr`·`li 3,1`·`blr` | **MISS** (§3.8) | — |

**Core: 11 of 12. Registers: 11 of 12.** Six cells came out **instruction-word
for instruction-word including every register**, four of them relational cells
predicted from rule (B).

`r2_pow_nb` is the cell worth stating: the mask is `0x100`, **contiguous**, MB
= ME = 23, and rule (S) alone says `rlwinm 3,11,0,23,23`. Rule (B) says
`li 10,256 · and 3,11,10` because `base == 0`. c2 emitted rule (B)'s answer.
`r1_bias16` is the same mask family **with** a bias, and emitted rule (S)'s
answer. That pair is the whole finding, in one obj.

### 3.8 THE MISS, stated as a miss

**`r5_bias1` — MISS, and it names a boundary neither prior run had.** Predicted
(from rule (B), `base = 1 ≠ 0`, `delta = 0x8000` contiguous):
`li 11,10 · subc · subfe · rlwinm 11,11,0,16,16 · addi 3,11,1`. c2 emitted a
**real compare-and-branch**: `cmplwi 6,3,10 · bf 24 · lis 3,0 · ori 3,3,32769 ·
blr · li 3,1 · blr`. There is no idiom at all, so both rules were never
consulted.

Rules (S) and (B) are not wrong about this cell; the **premise** is — the
if-conversion never happened. The useful residue, and it is registered here as
an *observation* and not as a rule because this lane has exactly one cell for
it:

> `0x8001` is the only result constant in any of the 44 cells this lane
> compiled that does **not** fit a signed 16-bit immediate, and it is the only
> cell that kept its branch. **A candidate boundary: the value-producing
> relational requires both result constants to be materialisable in one word.**
> `FUN_10c1ac5c`'s cost function does not model that (`FUN_10c0a170` prices the
> *mask*, not the *constants*), so if the boundary is real it lives in the
> if-conversion pass at `0x10b813f1` — the pass run 2 §6.3c also could not
> site. **One cell is not a rule** and it is not offered as one.

A prediction with a false conjunct is a MISS, so it is scored as one. PREREG
**P3.6** registered "≥2 of ≤12 cells will MISS"; **exactly 1 did**, so P3.6 is
scored a miss in the *pessimistic* direction.

---

## 4. THE CONSOLIDATED ADOPTION NOTE for `lower_expr` (deliverable 4)

### 4.1 What is black-box re-derivable, with the cells named

| piece | black-box? | the cells that already exist |
|---|---|---|
| **opcode → encoding** for the ~120 opcodes a port needs | **Yes — it is the PowerPC ISA.** No address, no row. | — |
| **the operator × type tables' behaviour** (`cmp`/`cmpl`, `cmpi`/`cmpli`, `divw`/`divwu`, `lbz`/`lhz`/`lwz`/`ld`, `stb…std`, `mr`/`fmr`/`vmr`, `add`/`fadds`, `subf`, `mullw`/`mulld`, `extsb`/`extsh`/`extsw`) | **Yes**, one fixture per (operator, signedness, width) | `S2`, `S5`, `S6`, `S9`, `S10`, `wbs_s5`, `wbs_k1`, `d_if_s`, `d_if_u_call` |
| **the immediate-fit rule** (signed-16 for nibble 1, unsigned-16 for 2–4, else force to a register) | **Yes** | `S2`, `S12`, `d_if_u_big` |
| **the `lha` fusion**, and that `lbz`+`extsb` does *not* fuse | **Yes** | `S9`, `S10` |
| **no magic-number multiply, no shift/add decomposition at `/O1`** | **Yes** | `S6` |
| **the `srawi`+`addze` power-of-two divide** | **Yes** | `wbs_k1` ≡ `S5` |
| **rule (S), the mask shape rule** — including that **`andi.` is what a non-contiguous 16-bit mask gets** | **Yes — newly, and this is what unblocks `&`** | `m1_run`, `m2_low16`, `m3_split16`, `m4_wrap`, `m5_two`, `m6_four`, plus `c_m_*` (9 more) |
| **rule (B), the bias rule** | **Yes — newly** | `d1_consumed`, `r1_bias16`, `r2_pow_nb`, `r3_spl_nb`, `r4_regbnd`, plus `c_rel_*` (6 more) |
| **the emitted shapes of the carry and `cntlzw` expanders** | **Yes** | `wbs_s1`–`wbs_s4`, `S1`, `S3`, `S4`, `S11` |
| **the if-conversion predicate** (an arm with a side effect, or a signed relation with a non-zero bound, keeps the branch) | **Yes** | run 2's 8 `d_if_*` cells, plus `r5_bias1` as a *new* and unexplained cell |
| **register choice** (WB-D §3.4) | **Yes** | every cell in all four grids |

### 4.2 What genuinely needs a same-commit DISCLOSURE row

**§10.27's claim is CONFIRMED for what it is about, and CORRECTED as a general
statement.** The claim reads: *"W-SELECT-3 is the only row in either campaign
where the black-box alternative is genuinely insufficient — no obj can
distinguish 'it was a tie' from 'B was cheaper'."*

* **CONFIRMED**, and this lane strengthens it. The tie is real, it is
  `if (cost_cntlzw <= cost_carry)`, and cell `wbs_s4` is a predicted 4–4 tie
  that went the predicted way. No obj separates a tie from a strict win without
  the two cost functions computed independently first, and **this lane found a
  second cost function with the same property**: `FUN_10c0a170`'s word prices
  are consulted by `FUN_10c1772b` with a **tie to the relaxed mask** (§3.1),
  which no obj can expose either. So the row is *more* necessary than §10.27
  states, not less.
* **CORRECTED**: it is the only row where the black-box alternative is
  insufficient **for an emitted decision**. Two facts outside it are equally
  out of black-box reach and one of them is a *precondition on the
  adoption-ready row*:
  1. **the table count and the slot map** (§1) — no obj exhibits a table, a
     slot number, or an empty slot. A port that re-derives behaviour cell by
     cell never learns that indices 26–29 exist and must be refused (§1.4), and
     never learns that a table it thought had 26 live entries has 17. This is
     **W-SELECT-2's stated precondition** and it belongs in that row's cost.
  2. **the opcode numbering and the attribute bits** (W-SELECT-1) — already
     carried as `route` by both runs, correctly.

**Everything else that mattered has moved to the black-box column.** Both runs
proposed **W-SELECT-5** for the `rlandi` expansion — run 1 as adoption-ready on
the strength of five cells, run 2 as *navigation, held*, because the form was
unpredictable. With rules (S) and (B) obj-confirmed on 32 cells, **W-SELECT-5
is adoption-ready and needs no row**: `grids/wb-tables/` re-derives the whole
expansion with no address at all. That is one named blocker removed from
§10.27's item 1.

### 4.3 The residue, named

| unread | why it matters |
|---|---|
| **the causal step under rule (B)** (§3.4, rivals R-A/R-B) | harmless — the rivals never disagree on any constructible cell |
| **`FUN_10c194b8` @ `0x10c194b8`** (890 bytes, the `{0,1}` bool path) | still the largest named gap; it refuted run 1's `wbs_b3` |
| **`FUN_10c1a908`'s twenty arms** (the against-zero fast path) | still unenumerated |
| **the if-conversion pass** at `0x10b813f1` | now has *two* unexplained cells (run 2's `d_if_*` set, and this lane's `r5_bias1`) |
| **`FUN_10c0b300`** (`cr0`-vs-`cr6`) | read by run 2, never obj-checked in isolation |
| **the signedness flip** `FUN_10c07803` | read by run 2, never obj-checked |
| **float / VMX / 64-bit paths of `FUN_10c0a2e2`** | this lane read only the 32-bit path (`(type & 0xfff) < 5`) |

---

## 5. PREREG SCORE

| # | claim | outcome |
|---|---|---|
| P0.1 | the count is settled by enumeration | **HIT** — §1.1, §1.3 |
| P0.2 | the settlement is "both partly right" | **HIT** — §1.2 |
| P1.1 | 13 installed pointer slots | **HIT** — 13, counted |
| P1.2 | 17 distinct bodies; run 1's 16 = 12+4 and omits the convert table | **HIT** — and §1.2 adds *why* 16 is also a real number (the `.data` block) |
| P1.3 | the alternates are installed by the same function under `DAT_10c2e978` and overwrite 4 of the 13 | **HIT** |
| P1.4 | every table is 26 ints = `0x68`, contiguous | **HIT** — 16 bodies at `0x68` in one block |
| P1.5 | the two published decode tables disagree on ≥1 decoded entry | **MISS** — they disagree on **no** entry on their overlap. Registered pessimistic; the enumeration adds two entries neither carries (`vsubfp`; the `663` marker) but that is an omission, not a disagreement |
| P1.6 | ≥1 table has ≥8 empty slots | **HIT, and stronger** — **every** table has 9 |
| P2.1 | the two cell lists are not comparable | **HIT** — §2.3 |
| P2.2 | 24/24 published emissions reproduce | **HIT** |
| P2.3 | neither published score moves | **HIT** — nothing re-scored |
| P2.4 | `/Gy` changes COMDAT sectioning only | **HIT** — diffed mechanically |
| P2.5 | overlap is 2–5 cells and the objs agree on all of them | **HIT** — 3, all agree |
| P3.1 | `FUN_10c1772b` contains a mask-contiguity computation | **MISS on the premise, and the miss is the deliverable** — it contains no such thing, because **it is not the expander**. The contiguity computation is `FUN_10c04daf`, called from `FUN_10c0a2e2`. Scored a miss because the prediction as written names the wrong function |
| P3.2 | contiguity alone does not decide it; ≥2 gates | **HIT** — rule (S) has four tests and rule (B) sits over it |
| P3.3 | the second gate is an operand-identity test (run 2's bound) | **MISS — RETRACTED, and run 2's bound with it.** `d1_consumed` refutes it in the same direction the calibration did |
| P3.4 | rival: the second gate is a record-form / CR-clobber test | **MISS** — the `CR0` clobber is a *consequence* (`andi.` sets it and c2 does not care), not a gate |
| P3.5 | `andi.`/`andis.` exist in the code and are unreachable at `/O1` for a value, so both docs are right | **HALF-HIT, scored a MISS.** They exist, and they are **reachable**: `andi.` fires on `m3_split16`. Registered as written, so it is a miss; the half that held is that a *contiguous* mask never takes them |
| P3.6 | ≥2 of ≤12 graded cells miss | **MISS (pessimistic)** — exactly 1 |
| P4.1 | §10.27's W-SELECT-3 claim confirmed for the tie-break, corrected by adding the table count | **HIT** — §4.2, and a second tie was found |
| P4.2 | after this lane `rlandi` is black-box re-derivable and W-SELECT-5 needs no row | **HIT** — §4.2 |
| P4.3 | the DISCLOSURE-requiring set ends at ≤3 items | **HIT** — three: the cost model + tie rules, the opcode numbering, the table count/slot map |
| P5.1 | no `crates/` change, no DISCLOSURE row minted | **HIT** |
| P5.2 | ≥1 dated in-place correction lands in a prior findings doc | **HIT** — two, one in each (§6) |
| P5.3 | calibration changes ≥1 cell before the freeze | **HIT** — three (§3.6) |

**25 registered · 18 hits · 7 misses · 0 withdrawn.** Every miss is above, in
the row it belongs to. Board #770's optimistic/pessimistic streak gains
**2 pessimistic** (P1.5, P3.6) and **3 optimistic** (P3.1, P3.3, P3.5) — the
three optimistic ones are all the same error, this lane inheriting run 2's
identification of `FUN_10c1772b` from its own brief instead of re-deriving it
at base. **Inherited prices have now been wrong nine times this week.**

---

## 6. CORRECTIONS MADE IN PRIOR DOCUMENTS

Both are **dated notes added in place**. Neither prior document is rewritten,
neither score is changed, and both retain their own text.

| document | what is corrected |
|---|---|
| `WB_SELECT_FINDINGS.md` §2.2, §7.2, §10 (W-SELECT-2) | the table **list** omits the convert/widen table `0x10b1fd08` (the count 16 is a real count of the `.data` block); rival `R-M1`'s second half ("never `andi.`") is refuted by `m3_split16` |
| `WB_SELECT_FINDINGS_R2.md` §4, §6.1, §6.3b, §7.2, §9 (W-SELECT-5) | `FUN_10c1772b` is a peephole **combiner**, not the `rlandi` expander; the expander is `FUN_10c0a2e2`, which run 1 named. The register-coincidence bound is refuted by `d1_consumed` |

## 7. Pre-drafted DISCLOSURE rows

This lane proposes **no new row** and mints none. It **amends two** pre-drafted
rows that already exist, and the amendments are net *reductions*:

| row | amendment |
|---|---|
| **W-SELECT-2** (both runs, adoption-ready) | the table list must be **13 slots / 17 bodies** and must include the convert/widen table `0x10b1fd08`; add `0x10bd7c10` + `0x10bd7cf0` (the type index) and the **slot map** of §1.4, which is the part that is not black-box re-derivable and is the row's stated precondition |
| **W-SELECT-5** (run 1 adoption-ready, run 2 navigation-held) | **released.** The expander is `0x10c0a2e2` (not `0x10c1772b`), and rules (S) and (B) are black-box re-derivable from `grids/wb-tables/` with no address. Carry the row only if `FUN_10c0a170`'s **word prices** or `FUN_10c1772b`'s **tie to the relaxed mask** are copied — neither is visible in any obj |

**W-SELECT-1, W-SELECT-3 and W-SELECT-4 are unchanged**, and §4.2 confirms
W-SELECT-3's standing as the row whose black-box alternative is insufficient.

---

## 6. CORRECTION — appended 2026-08-09 by lane `wb-selfit`

**Nothing above is rewritten.** This lane (`wb-selfit`, board **#2200**–**#2213**,
record [`WB_SELECT_RECONCILED.md`](WB_SELECT_RECONCILED.md)) ran concurrently
with `wb-tables` and neither saw the other. It **independently confirms three of
this document's answers** — the 13-slot / 17-body count with `convert` @
`0x10b1fd08` as the missing seventeenth, `FUN_10c0a2e2` (not `FUN_10c1772b`) as
the `rlandi` expander, and that the two grid scores are not on one denominator —
and it corrects **one** row.

### 6.1 §4.3's residue table mis-labels `FUN_10c194b8`

> *"**`FUN_10c194b8` @ `0x10c194b8`** (890 bytes, the `{0,1}` bool path) — still
> the largest named gap; it refuted run 1's `wbs_b3`"*

**It is the FLOATING-POINT path.** The label is inherited from
`WB_SELECT_FINDINGS.md` §7.6 and is wrong there too (corrected in that
document's §11.4). The evidence:

* the routine's own locals are `double *pdVar3` and `float fVar8`; it tests
  `*pdVar3 == 0.0` and the operand opcode `0x6a`;
* `FUN_10c1b517` reaches it on `(type & 0xf000) == 0x5000`, and type nibble
  **5 is the float family** in `FUN_10bd7c10`'s own map (size 4 → index 13,
  size 8 → index 14 — exactly the `f32`/`f64` slots of every table §1.3
  enumerates);
* run 1's own cells refute the `{0,1}` reading directly: `wbs_b1` and `wbs_b2`
  have `{0,1}` result pairs and came out as the **plain carry idiom**.

### 6.2 And the row below it is the one that should be at the top

`wbs_b3` is not a `FUN_10c194b8` cell. `FUN_10c1b517` tests *"is either compare
operand the constant `0`"* — with the enable `FUN_10c1b2fa` — and routes such a
tuple to **`FUN_10c1a908`** *before* it calls either costed expander. **Five of
the two prior grids' 24 cells are against-zero relationals** — `wbs_s4`,
`wbs_s6`, `wbs_b3`, `S3`, `S4` — and both prior lanes graded all five as
evidence about `FUN_10c1ac5c` / `FUN_10c1af2d`. `S4`'s emitted `addic 11,3,-1`
is the tell: `addic` is not in `FUN_10c1ac5c`'s emission list at all.

So §4.3's *"`FUN_10c1a908`'s twenty arms — still unenumerated"* understates it:
**for an integer `lower_expr` that routine is the largest named gap**, not
`FUN_10c194b8`, and five already-graded cells are inside it.

This **strengthens** §4.2's confirmation of `W-SELECT-3` on independent grounds:
`wb-tables` found a second invisible tie; `wb-selfit` found that no obj in the
project ever reached the first one. Board **#2204**, **#2205**.

### 6.3 What `wb-selfit` takes from this document

§10 of `WB_SELECT_RECONCILED.md` left the `x & K` predicate **open** with eight
cells as its constraint set. **Rules (S) and (B) close it**, and rule (B)
explains all eight. That section is superseded, and so is its `W-SELECT-5` row:
**released**, per §4.2 here, not held. `WB_SELECT_RECONCILED.md` §16 records the
supersession, and it also records that this document corrected a sentence of its
own §10 — *"the 'never `andi.`' half survives every cell"* — with `x & 0x8001`
⇒ `andi. 3,3,32769` (#2115), a cell no prior grid contained.
