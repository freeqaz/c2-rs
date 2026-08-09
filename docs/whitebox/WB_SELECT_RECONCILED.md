# WB-I RECONCILED — the two instruction-selection readings, settled

> **PROVENANCE — DISASSEMBLY-DERIVED.** Every address below was re-read this
> session from the flat export of the image pinned in
> [`C2_MAP_METHOD.md`](C2_MAP_METHOD.md) §0, sha256 verified at the top of the
> lane as `c80981c015166effecc71ad8112d5577a065b2300891dfdb02b9c13787a66258`.
> Navigation only until a row is added to [`DISCLOSURE.md`](DISCLOSURE.md); §7
> states the reconciled scope. **This lane compiled nothing** — the toolchain is
> absent in its worktree, so every obj word here is the two earlier lanes' own
> committed bytes.

Lane `wb-selfit`, 2026-08-09. Reconciles:

* [`WB_SELECT_FINDINGS.md`](WB_SELECT_FINDINGS.md) + `rungs/2026-08-09-wb-select.md`
  + board **#2040**–**#2047** — called **R1** below;
* [`WB_SELECT_FINDINGS_R2.md`](WB_SELECT_FINDINGS_R2.md) + `rungs/2026-08-09-wb-select2.md`
  + board **#2100**–**#2109** — called **R2** below.

Two lanes read one binary on one day and both landed on master. They agree on
the headline — selection is table-driven and a general `lower_expr` is derivable
— and they disagree on eleven specifics. The project does not leave a
contradiction in the record. This file is the resolution, `w-memfit`'s method
applied to a disassembly pair instead of a rule pair.

PREREG: `work/wb-selfit/PREREG.md`, committed at `a09740e3` **before the first
grep of the export and before the first score**. Evidence:
`work/wb-selfit/EXPORT_READS.md` (eight readings, each with the line or the
arithmetic that settles it), `work/wb-selfit/xscore.py` + `xscore.out` (the
cross-score and its control), `work/wb-selfit/armcount.py` + `armcount.txt`.

---

## 0. The answer in one screen

> ### **BOTH LANES ARE RIGHT ABOUT THE SHAPE AND EACH IS WRONG ABOUT SOMETHING THE OTHER GOT RIGHT. Eleven disagreements: four are counting conventions, five are one lane's error, and two are neither lane's — they are cells BOTH lanes attributed to a mechanism that the driver's own control flow says never ran.**

> ### **THE TABLE COUNT IS 13 SLOTS AND 17 BODIES. `FUN_10c04cb9` writes thirteen destination pointers `DAT_10c6fdac`…`DAT_10c6fddc` and then, under `DAT_10c2e978`, *overwrites four of those same thirteen* with `-QVMX128` alternates. R2's 13 is the slot count and is right. R1's 16 is `12 + 4`: it counts the alternates as tables of their own AND omits the thirteenth table entirely — `convert` @ `0x10b1fd08`. The omission costs it two cells of the other lane's grid.**

> ### **NO OBJ IN THIS PROJECT HAS EVER REACHED THE COST RACE.** `FUN_10c1b517` tests *"is either compare operand the constant 0"* **before** it calls either expander, and routes such a tuple to `FUN_10c1a908` — which **neither lane read**. Five of the two grids' 24 cells are against-zero relationals (`wbs_s4`, `wbs_s6`, `wbs_b3`, `S3`, `S4`), and **both lanes graded all five as evidence about the two costed expanders**. R1's `wbs_s4` — *"the only black-box evidence in this project that the tie-break exists"* — is one of them. **The tie rule is not merely un-black-box-able in principle (both lanes said so); it is unevidenced in fact.**

> ### **THE CROSS-SCORE IS NOT CLOSE. R2's reading scores 22 of the 22 cells it claims across BOTH grids (11/12 on R1's, 11/11 on R2's); R1's scores 11 of the 13 it claims (7/7 on its own, 4/6 on R2's).** R1 abstains on eleven of the 24 cells, and every abstention is forced by R1's own "not claimed" list — the missing convert table, the unread non-power-of-two divide, and a nibble-5 exclusion that is itself wrong.

> ### **AND R1 IS RIGHT ABOUT THE TWO THINGS R2 GOT WRONG, INCLUDING THE ONE R2 CALLED ITS ONLY BLOCKER.** The `rlandi` expander is **`FUN_10c0a2e2`** — R1's name — reached from `FUN_10c0d57e`'s `0x26e`/`0x26f` arm; R2's `FUN_10c1772b` is a real but *different* pass and is not the form-chooser. And the relation-code enum is **R1's**: read from its own name array, `4 = GT, 5 = LE, 8 = UGT, 9 = ULE`, where R2 published two transposed pairs beside the very address that refutes them.

| the eleven | R1 says | R2 says | settled |
|---|---|---|---|
| table count | **16** | **13** | **13 slots / 17 bodies** — R2 (§1) |
| dispatch arms | **46** | **41** | **41** jump-table arms, 46 case labels, 39 groups — R2, and re-derivable from its own VAs (§2) |
| expansion switch | `FUN_10c0d57e` | `FUN_10c182b4` | **both real, different passes** — neither wrong (§3) |
| `rlandi` expander | `FUN_10c0a2e2` | `FUN_10c1772b` | **`FUN_10c0a2e2`** — R1 (§4) |
| record form | "**not** a fusion; opcode+1" | "a fusion **at** opcode+1" | **one mechanism, both halves true** — R2's verdict, and the *rewriter* is `FUN_10c0b4c0`, which neither lane named (§5) |
| tie direction | ties → `cntlzw` | ties → `cntlzw` | **agreed and confirmed** — and **unevidenced** (§6) |
| nibble 5 | "bool-typed" | "floating point" | **floating point** — R2 (§6.2) |
| relation codes | `4 >, 5 <=, 8 >u, 9 <=u` | `4 LE, 5 GT, 8 ULE, 9 UGT` | **R1** (§8) |
| signed/unsigned split | "the type is the table's index" | "three of thirteen tables" | **the same fact, two granularities** (§9) |
| `x & K` form | "contiguous ⇒ `rlwinm`, never `andi.`" | "unread, `FUN_10c1772b`" | **three forms, and no published predicate fits all 8 cells** (§10) |
| P3.4 / value-vs-branch | **HIT** | **MISS, retracted** | **R2** — and R1's own calibration cell says so (§11) |

---

## 1. THE TABLE COUNT — thirteen slots, seventeen bodies, one missing table

`FUN_10c04cb9` @ `0x10c04cb9` is 180 bytes and does nothing but assign pointers
(`work/wb-selfit/EXPORT_READS.md` §E1 quotes it whole):

| # | slot | table VA | operator |
|---:|---|---|---|
| 1 | `DAT_10c6fddc` | `0x10c38f30` | copy |
| 2 | `DAT_10c6fdd8` | `0x10c38f98` | load, D-form |
| 3 | `DAT_10c6fdd4` | `0x10c39068` | load, X-form |
| 4 | `DAT_10c6fdd0` | `0x10c39138` | store, D-form |
| 5 | `DAT_10c6fdcc` | `0x10c391a0` | store, X-form |
| 6 | `DAT_10c6fdc8` | `0x10c392d8` | negate |
| 7 | `DAT_10c6fdc4` | `0x10c39340` | add |
| 8 | `DAT_10c6fdc0` | `0x10c393a8` | sub |
| 9 | `DAT_10c6fdbc` | `0x10c39410` | mul |
| 10 | `DAT_10c6fdb8` | `0x10c39478` | div |
| 11 | `DAT_10c6fdb4` | `0x10c394e0` | compare, immediate |
| 12 | `DAT_10c6fdb0` | `0x10c39548` | compare, register |
| **13** | **`DAT_10c6fdac`** | **`0x10b1fd08`** | **convert / widen** |

and then, `if (DAT_10c2e978 != 0)`, **reassigns slots 2, 3, 4 and 5** to
`0x10c39000`, `0x10c390d0`, `0x10c39208`, `0x10c39270` — the `-QVMX128`
alternates of load-D, load-X, store-D and store-X.

**So the two numbers count two different objects and neither convention gives
16.** 13 is the number of slots (R2). 17 is the number of distinct table bodies
the installer can write. **16 is `12 + 4`, and the 12 is short**: R1's §2.2 and
board **#2040** enumerate *move/load/loadx/store/storex/neg/add/sub/mul/div/
cmp-imm/cmp-reg + four `-QVMX128`* and the **convert table `0x10b1fd08` is not in
that list at all.** R1's own §2.2 pointer block stops at `DAT_10c6fdb0`.

**This is a genuine enumeration error, not a convention**, and it is the one
error in this pair with a measurable cost: §12's cross-score shows R1 forced to
**abstain on cells `S9` and `S10`** — `signed char` and `short` load-and-widen —
because it has no convert table to read `extsb` / `extsh` out of, and therefore
no way to see the `lhz`+`extsh` → `lha` fusion either. R2 predicted both
byte-exact.

R1's own §9.5 lists what it did not read and `0x10b1fd08` is not on that list,
so the table is not *declined* — it is missed.

**Corrections filed:** `#2040` (see §13).

---

## 2. THE ARM COUNT — 46 and 41 are two objects, and only one is arms

`FUN_10c0f882` @ `0x10c0f882` (`objdump_intel.asm:385341`):

```
10c0f897: add    eax,0xfffffd82                ; opcode -= 0x27e
10c0f89c: cmp    eax,0xad                      ; 174 opcodes
10c0f8a1: ja     0x10c0fb2a                    ; default
10c0f8a7: movzx  eax,BYTE PTR [eax+0x10c0fbd6] ; byte index
10c0f8ae: jmp    DWORD PTR [eax*4+0x10c0fb32]  ; jump table
```

`(0x10c0fbd6 − 0x10c0fb32) / 4 = 41`. **The jump table has 41 entries, and the
arithmetic uses only the two VAs R2 itself published** — so R2's count is
checkable from its own document without opening anything.

Ghidra's decompilation of the same switch carries **46 `case` labels** in **39
maximal label groups**, two of which (`0x2cb` `|`, `0x2cc` `^`) share one body
via `goto LAB_10c0f970` — **38 distinct decompiled bodies**
(`work/wb-selfit/armcount.txt`).

**R1's 46 is a case-label count.** It is a true number about a real object and
it is not what "arm" means for a jump-table switch; R1's own §2.1 tabulates 17
rows covering 22 opcodes, so nothing downstream in R1 uses 46 except the
sentence that names it and PREREG P4.5, which passes on either number
("under 120").

**Both counts should be quoted with their object.** For a port pricing the
selector, the useful number is **41**: distinct handlers, `+`/`−`/`|`/`^` and
the three-opcode add and sub families already collapsed.

---

## 3. THE EXPANSION SWITCH — both lanes named a real one, and they are different

Neither lane is wrong; they walked into two different passes and each assumed it
was WB-D §4's.

| | `FUN_10c0d57e` (R1) | `FUN_10c182b4` (R2) |
|---|---|---|
| size | 3899 B | 426 B |
| shape | binary decision tree | byte index `0x10c184a8` + jump table `0x10c18460`, **18 arms** |
| index | tuple **and** machine opcodes, one space | machine opcode only, `op − 1 ≤ 0x292` |
| callers | three, inside the lowering | **one**, the phase driver `FUN_10b7dd2c` @ `0x10b7dd2c`, gated on `DAT_10c2e2fc`; it runs its list **twice** |
| prologue/epilogue `0x2f0`/`0x2f4` | **yes** | no |
| `rlandi` | **yes**, → `FUN_10c0a2e2` | arm 13 → `FUN_10c1772b` |

**WB-D §4's switch is `FUN_10c0d57e`**, because WB-D's identifying detail is the
`0x2f4`/`0x2f0` arms calling the prologue driver through `0x10c216f5` /
`0x10c21719`, and those call sites are at `decomp_all.c:213277-213285`, inside
`FUN_10c0d57e`. (`0x10c216f5` and `0x10c21719` are their own 19- and 25-byte
functions, so WB-D is right that they are thunks and R1 is right that the *call
sites* are inside the switch. R1's "correction to WB-D §4, offered gently"
stands.)

**R2's `FUN_10c182b4` is a machine-level peephole phase**, not the selection
expansion — and it is load-bearing anyway, for a reason R2 did not claim: **its
arms 3/4/5 are the `extsb`/`extsh`/`extsw` narrowing fusions**, which is where
R2's own cell `S10`'s `lha` comes from. That makes `S10` black-box evidence that
`DAT_10c2e2fc != 0` at `/O1`, which §6 needs.

---

## 4. THE `rlandi` EXPANDER IS `FUN_10c0a2e2` — R1's name, and R2's "one blocker" is misidentified

`FUN_10c0a2e2` (1871 B) has exactly two callers and **both gate on `rlandi`**:

```
decomp_all.c:212862  (inside FUN_10c0d57e)   if (uVar7 - 0x26e < 2) FUN_10c0a2e2(param_2);
decomp_all.c:226196  FUN_10c1cf59:  if (op != 0x26e && op != 0x26f) return;  FUN_10c0a2e2(param_1);
```

and it is the routine that picks the **form**, with all three outcomes visible
in one body:

* `FUN_10c04daf(mask, &mb, &me)` — the contiguity analysis (the function
  immediately after the table installer);
* `local_30 = 0x133` / `local_34 = 0x134` → **`rlwinm` / `rlwinm.`**;
* `LAB_10c0a802` → **`andi.` / `andis.`**, guarded by
  `(DAT_10c2ecf0 == 0) && (DAT_10c2e310 == 0)` — i.e. by the **favour-speed word
  `wb-memcpy` found at `0x10c2e310`** — and by a CR0-availability query
  `thunk_FUN_10bd5a62(*param_1, 0x10c309a8)`;
* `LAB_10c0a9a6` → mints a constant (`FUN_10c08e38(0xd, …)`) and sets
  `param_1[1] = 0x19` (**`and`**) or `0x1a` (`and.`) — **this is the `li` + `and`
  form** that R2's `S11` and its seven diagnostic cells produced and that R1
  never saw.

**`FUN_10c1772b` is real, is arm 13 of `FUN_10c182b4`, and is a different
thing** — a mask-merging peephole that recomputes a mask, compares two costs
through `FUN_10c0a170` and rewrites operand *values*; the read path mints no
opcode.

So R2's §7.2 item 4 and its `W-SELECT-5` — *"a port that wants byte-exactness on
any expression containing `&` with a constant **must read `FUN_10c1772b`
first**"* — **names the wrong routine.** The routine to read is
**`FUN_10c0a2e2`**, and R1 named it in §2.4 without reading the form decision
out of it.

**Corrections filed:** `#2107`, `W-SELECT-5` (R2's) (see §13).

---

## 5. THE RECORD FORM — one mechanism, and the two board headlines contradict each other

Board **#2044** reads *"RECORD FORMS ARE NOT A FUSION"*; board **#2106** reads
*"RECORD FORMS ARE A FUSION AT `opcode + 1`"*. Both lanes' **prose** says the
same thing; only the headlines and the P4.4 verdicts differ. The export settles
it, and it settles it in R2's direction.

`FUN_10c0b300` @ `0x10c0b300` is the **predicate** — it returns `1`/`0` and
tests `(&DAT_10c3afd8)[op] & 0x10`. **The rewriter is `FUN_10c0b4c0` @
`0x10c0b4c0`, which neither lane named**, and it does the whole job in one pass
(`decomp_all.c:211186`):

* walks **backwards** from the compare (`iVar9 = param_1[4]`, then
  `iVar9 = *(int *)(iVar9 + 0x10)`);
* promotes `addi` (`0xb`) → `addic` (`0xc`) with a minted carry operand —
  R2 §2.6's claim, confirmed, and the origin of WB-D's `addic. r31,r31,-1`;
* requires `((&DAT_10c3afd8)[op] & 0x10) != 0`;
* **`*(int *)(iVar9 + 4) = *(int *)(iVar9 + 4) + 1;`**
* `FUN_10bd5516(param_1)` — **deletes the compare**.

> **It is a fusion, and the fusion's action is `opcode + 1`.** R1's P4.4 = MISS
> is a mis-scoring of a prediction that was right, and `#2044`'s headline is
> false as written.

**And for a port the distinction the two headlines are groping at is real but is
not this one.** `opcode + 1` and `flags & 0x10` are both facts about **c2's
private numbering**; neither transfers. The transferable rule is
*"when the defining instruction has a record form, use it, delete the compare,
and put the result in `cr0`"* — which needs **no address and no bit**. §7 prices
it accordingly.

**Corrections filed:** `#2044` (see §13).

---

## 6. THE COST RACE — the tie is agreed, and it has never been reached by an obj

### 6.1 The tie direction is not in dispute

`FUN_10c1b517` @ `0x10c1b517`, 140 bytes, verbatim
(`work/wb-selfit/EXPORT_READS.md` §E5):

```c
uVar3 = FUN_10c1ac5c(param_1,0);      /* carry  cost */
uVar4 = FUN_10c1af2d(param_1,0);      /* cntlzw cost */
if (uVar4 <= uVar3) { FUN_10c1af2d(param_1,1); return; }
FUN_10c1ac5c(param_1,1);
```

`uVar4 <= uVar3` ⇒ **ties go to the `cntlzw` expander**. R1 says "ties to B
(`cntlzw`)"; R2 says "ties to the `cntlzw` one". **They agree, and they are
right.** There was never a disagreement here — the appearance of one comes from
R1 naming the expanders `A`/`B` and R2 naming them by idiom.

### 6.2 But the driver has TWO guards in front of the race, and both lanes' evidence sits behind them

```c
if ((*(ushort *)((int)param_1 + 10) & 0xf000) == 0x5000) { FUN_10c194b8(param_1); return; }
if (FUN_10c1b2fa(DAT_10c2e2f4) && <either compare operand is the constant 0>) {
        FUN_10c1a908(param_1); return; }
```

**Guard 1 — nibble 5 is FLOATING POINT, not `bool`.** `FUN_10c194b8` (890 B)
has `double *` and `float` locals, tests `*pdVar3 == 0.0`, and checks the
operand opcode `0x6a`. Nibble 5 is the float family in `FUN_10bd7c10`'s own type
map (sizes 4→13, 8→14 — exactly the `f32`/`f64` slots of every operator table).
**R2's label is right; R1's §7.6 "bool-typed (`type nibble 5`)" is wrong**, and
so is the class exclusion R1 built on it (*"result values that are not
`{0,1}`"*). R1's own grid refutes it: `wbs_b1` and `wbs_b2` have `{0,1}` result
pairs and came out as the **plain carry idiom**.

**Guard 2 — an against-zero relational never reaches the race.** `FUN_10c1a908`
(768 B) normalises which side is zero through the *negate* table `0x10b189cc`
and dispatches on the relation code to about twenty arms; it materialises a
result operand that is neither `1` nor `−1` (`FUN_10c19859`), so
`x == 0 ? 5 : 6` is squarely inside its remit. **Neither lane read it** — R1
§3.6/§9.5 and R2's `W-SELECT-3` both name it located-and-unread.

Is guard 2 live at `/O1`? `FUN_10c1b2fa` returns 1 iff
`DAT_10c2ed00 == 0 && DAT_10c2e2f4 != 0 && DAT_10c2e2fc != 0`. **`DAT_10c2e2fc
!= 0` at `/O1` is established black box** by R2's cell `S10`: `lha` is the
`lhz`+`extsh` fusion, produced by arms 3/4/5 of `FUN_10c182b4`, whose only
caller gates on that same word. Only `DAT_10c2ed00` is unknown.

### 6.3 Which means five cells were graded against the wrong mechanism

| cell | source | both lanes attributed it to | actually routed to |
|---|---|---|---|
| `wbs_s4` | `x == 0 ? 5 : 6` | **the race, and the TIE RULE** (R1 §7.3) | `FUN_10c1a908` |
| `wbs_s6` | `int x < 0 ? 1 : 2` | `FUN_10c1af2d`'s `bVar5` arm (R1 §7.6) | `FUN_10c1a908` |
| `wbs_b3` | `(int)(x < 0)` | `FUN_10c194b8`, the "bool" path (R1 §7.6) | `FUN_10c1a908` |
| `S3` | `x == 0` | `FUN_10c1af2d` (R2 §6) | `FUN_10c1a908` |
| `S4` | `x != 0` | **the cost comparison in `FUN_10c1b517`** (R2's frozen.tsv) | `FUN_10c1a908` |

`S4` carries its own tell: c2 emitted **`addic 11,3,-1`**, and `addic` is **not
in `FUN_10c1ac5c`'s emission list** — that routine mints `li` (`0x270`),
`subfic` (`0x18b`), `subfc` (`0x183`), `lcarry` (`0x28c`), `rlandi` (`0x26e`)
and `addi` (`0x0b`), and nothing else. R2 scored `S4` a HIT because its
prediction's *words* named `addic` as an option; the **mechanism** it says the
cell refutes is not the mechanism that ran.

> ### **So R1 §7.3's sentence — *"that single cell is the only black-box evidence in this project that the tie-break exists"* — is withdrawn. `wbs_s4` cannot be evidence about the tie, because the driver decides against zero before it ever computes a cost.**

This does not weaken the reading; it sharpens the **disclosure** conclusion.
Both lanes wrote that the tie rule is not obtainable from an obj *in principle*.
It is now also true *in fact*: there is no cell anywhere in this project that
even reached the code path. §7 carries that.

**And `wbs_b3` finally has a candidate explanation.** R1 retracted it to
`FUN_10c194b8`, which is the float path and cannot be it. A signed `<` against
zero with results `{0,1}` is a `FUN_10c1a908` arm, and a one-word
`srwi rD,rX,31` is exactly the shape such an arm would have. Stated as a
candidate, not a reading — this lane did not enumerate those twenty arms either.

**Corrections filed:** `#2042`/`#2047` (R1), `#2103` (R2) (see §13).

---

## 7. What both lanes flagged as needing an address — CONFIRMED, and there is a SECOND

Both lanes ended on the same sentence: the **cost model and the tie rule** are
the one thing no obj can establish. **That is confirmed, and §6.3 makes it
stronger than either lane could**: not only can no obj distinguish "`cntlzw` was
cheaper" from "ties go to `cntlzw`", **no obj in either grid reached the
comparison at all.**

**But it is not the only row that needs an address, and R2 said so first.**
R2's `W-SELECT-4` note — *"the counts (41, 18) are the load-bearing claim for
§7.1's pattern-set size and cannot be obtained black-box"* — is right, and this
lane extends it: **§1's `13`, §2's `41` and §3's `18` are the numbers the
≈60-rule price rests on, and a count of arms is not an observable of any obj.**
A port that only *implements* the rules needs none of them; a lane that *prices*
the work needs all three, and pricing is what both judgments (#2046, #2108) do.

See §14 for the merged scope table.

---

## 8. THE RELATION-CODE ENUM — R1 derived it correctly with no access to the names

`data.tsv` carries a pointer array at `0x10c38690` into the string pool
descending from `0x10b197f4`. Decoded from the verified image:

```
 0 ILLEGAL   1 EQ    2 NE    3 LT    4 GT    5 LE    6 GE
 7 ULT       8 UGT   9 ULE  10 UGE  11 SO   12 NSO  13 NS
```

**R1 §3.5** published `1 ==, 2 !=, 3 <, 4 >, 5 <=, 6 >=, 7 <u, 8 >u, 9 <=u,
10 >=u`, derived from the fixed points and involutions of the three remap
tables `0x10b189a4` / `0x10b189b8` / `0x10b189cc`, with no access to these
strings at all. **It is exactly right**, and that is the strongest single piece
of tradecraft in either lane.

**R2 §3.1** published `4 LE, 5 GT, 8 ULE, 9 UGT` — **two transposed pairs** —
and published them beside the address range `0x10b197c0`–`0x10b197f4` that
refutes them. The consequences are naming-only but they propagate:

* the carry expander's canonical form is **`UGT`** (code 8), not `ULE`;
  R2's `W-SELECT-3` row and board **#2102** both say `ULE`;
* `0x10b189a4`'s mapping is `3 LT→7 ULT, 4 GT→8 UGT, 5 LE→9 ULE, 6 GE→10 UGE`
  — the *arrows* R2 drew are right, the *labels* are mis-ordered;
* R2's normalisation table row "`9 UGT` swap, code := 8" should read
  "`9 ULE` swap the RESULT values, code := 8", which is R1's row for code 9.

**Corrections filed:** `#2102`, `W-SELECT-3` (R2's) (see §13).

---

## 9. THE SIGNED/UNSIGNED SPLIT — the same fact at two granularities

R1: *"the comparison's signedness is not recomputed and not inferred: it is
`DAT_10c6fdb0[type]` versus `DAT_10c6fdb4[type]`"*, i.e. **the type index picks
the row**.

R2: *"the entire signed/unsigned distinction in the scalar integer set lives in
three of them — `div`, `compare-immediate`, `compare-register`"*, i.e. **only
three tables have rows that differ across the boundary**.

**Both are true and they are not in tension.** The *mechanism* is one array
lookup on the type index (R1's statement); the *content* is that twelve of the
thirteen tables are constant across the signed rows `{1,2,4,6}` and the unsigned
rows `{7,8,10,12}`, and only three are not (R2's statement). R1's own §2.2 table
shows exactly the same three columns varying.

For a port they collapse to R2's compression: **13 operator rules + 1 signedness
bit**, not 338 entries. **Nothing to correct on either side.**

---

## 10. `x & K` — the two findings are compatible cell by cell, and no published predicate fits them

Eight cells across the two grids exercise the `rlandi` expansion. Putting them
on one page for the first time:

| cell | lane | mask | bias | rlandi src → dst | emitted |
|---|---|---:|---:|---|---|
| `wbs_k2` | R1 | `0xff` | — | `r3 → r3` | `clrlwi 3,3,24` (**rlwinm**) |
| `S7` | R2 | `0xff` | — | `r3 → r3` | `clrlwi 3,3,24` (**rlwinm**) |
| `wbs_s3` | R1 | 8 | 4 | `r11 → r11` | `rlwinm 11,11,0,28,28` |
| `wbs_b1` | R1 | 1 | 0 | `r11 → r3` | `clrlwi 3,11,31` (**rlwinm**) |
| `wbs_b2` | R1 | 1 | 0 | `r11 → r3` | `clrlwi 3,11,31` (**rlwinm**) |
| `wbs_k3` | R1 | 1 | 0 | `r11 → r11` | `clrlwi 11,11,31` (**rlwinm**) |
| `S1` | R2 | 4 | 3 | `r11 → r11` | `rlwinm 11,11,0,29,29` |
| **`S11`** | R2 | **8** | **0** | `r11 → r3` | **`li 10,8` · `and 3,11,10`** |
| R2 `diag` | R2 | 2,3,4,8,16 | 0 | — | **`li` + `and`** (five cells) |
| R2 `diag` | R2 | 4, 8 | 3 | — | `rlwinm` (two cells) |

**No contradiction between the grids** — the cells never disagree with each
other. But **no published predicate fits all of them**, and both lanes' are
falsified by the other's cells:

* **R1's** `W-SELECT-5` / #2046 clause, carried as **adoption-ready** —
  *"`&` with a contiguous mask is `rlwinm`, never `andi.`"* — is **over-general**.
  `S11`'s mask of 8 is contiguous and got `li`+`and`. The "never `andi.`" half
  survives every cell; the "always `rlwinm`" half does not, and it is the half a
  port would emit from.
* **R2's** §6.1 conjecture — the deciding fact is whether `rlandi`'s source and
  destination land in the same register — is **refuted by R1's cells**:
  `wbs_b1` and `wbs_b2` have `r11 → r3`, *different* registers and no bias, and
  got `rlwinm` anyway.
* **R2's** "bias vs no bias" summary is refuted by `wbs_k2` / `S7` — no bias,
  mask `0xff`, `rlwinm` — which is R2's **own** cell.

**What §4 adds is the search space.** `FUN_10c0a2e2` decides between three
forms, not two, on at least four inputs: the rotate field, `FUN_10c04daf`'s
contiguity analysis of the mask **and of a second quantity derived from it**,
the pair `(DAT_10c2ecf0, DAT_10c2e310)`, and a CR0-availability query. A
predicate over "mask contiguity" alone was never going to fit, which is why both
lanes' did not.

**LEFT OPEN. What would settle it:** read `FUN_10c0a2e2` from `LAB_10c0a4cb` to
`LAB_10c0a9a6` and re-derive `FUN_10c04daf`'s return contract, then re-grade
against these eight cells plus a mask-1 / mask-8 × bias-0 / bias-3 square. This
lane names the routine and does not claim the predicate.

**Corrections filed:** `#2046`, `W-SELECT-5` (R1's); `#2107` (R2's) (see §13).

---

## 11. P3.4 — R1's HIT is contradicted by R1's own calibration cell

PREREG P3.4 registered *"branch context selects `cmplwi`+`bc`; there is a
value-vs-branch bit, and it gets named"*.

* **R2 scored it MISS and retracted it** (§6.3c, board #2104), on cell `S12`:
  `if (x < 10u) return 1; return 2;` emitted `li · subc · subfe · addi · blr` —
  **no branch at all**.
* **R1 scored it HIT**, on the reasoning that "the bit is the *tuple opcode*:
  `0x2d4` compare vs `0x2ea` relational-as-value".

R1's §6.1 already contains the refutation: *"`wbk_2` (the `if` spelling of
`wbk_1`) is **byte-identical** to `wbk_1`. The grid therefore does not waste a
cell on the spelling."* If the `if` spelling and the `?:` spelling produce
identical bytes, there is no branch context selecting `cmplwi`+`bc`, which is
what P3.4 predicted.

And the two grids agree to the word: **R2's `S12` and R1's `wbs_s1` are the same
source shape in the two spellings, and their emitted sequences are identical** —
`li 11,10 · subc 11,3,11 · subfe 11,11,11 · addi 3,11,2 · blr` in both objs,
compiled by two lanes on two grids. That is the reconciliation's cheapest
cross-lane reproducibility control and it is exact.

R1's substantive point survives and is worth keeping: **the tuple opcode `0x2d4`
vs `0x2ea` really is the selector's only "context"**. What is false is that a C
`if` puts a relational at `0x2d4`. R2's replacement rule — *a two-way `if` keeps
a compare-and-branch iff either arm has a side effect, or the relation is signed
with a non-zero bound* — is decided **upstream of selection**, and cell `wbs_s5`
(R1's own) is an instance of its second clause, which is why R1's `wbs_s5`
prediction landed while its P3.4 verdict did not.

**Corrections filed:** `#2043`'s P3.4 line, R1 §8 (see §13).

---

## 12. THE CROSS-SCORE

`work/wb-selfit/xscore.py`, transcript `xscore.out`.

### 12.0 The control that makes the numbers readable

The script **refuses to print a cross-score** until it has re-derived, from each
lane's own committed `frozen.tsv` and its own published emitted words, every
per-cell verdict and both totals:

```
  GRID-1 TOTAL primary 10/12 (doc says 10/12)  secondary 6/10 (doc says 6/10)
  GRID-2 TOTAL core     9/12 (doc says  9/12)
  ALL REPRODUCED — the cross-score below is on the same denominators.
```

All 24 per-cell verdicts reproduce as well, not just the totals. **It did not,
first try**: the grid-2 grader compared a `list` against a `tuple`, which is
always unequal, and graded R2 at **2/12** against a published 9/12 — a number
that would have read as a refutation of the second lane and was a Python type
mismatch. The bug and its consequence are recorded in the script at the line
that carried it.

### 12.1 The scoring policy, stated before the numbers

Three-valued: **HIT / MISS / ABSTAIN**. A reading ABSTAINS on a cell when its
own document says the deciding pass is unread or the case is not claimed. A
two-valued cross-score would charge a lane a MISS for honesty, which is the
opposite of what the record should reward.

Two policies are reported. **PUBLISHED** takes each lane's own attributions.
**SYMMETRIC** additionally abstains *both* readings on the five against-zero
cells of §6.3, since `FUN_10c1a908` is unread by both.

Predictions are derived from each findings doc's **rules**, not from its
`frozen.tsv` row, and each carries the section that supplies it (the `CROSS`
table in `xscore.py`). **This is a hand-built derivation and is weaker evidence
than §12.0's control** — it is stated that way rather than presented as
mechanical.

### 12.2 The table

| | cell | emitted | **R1** | **R2** |
|---|---|---|---|---|
| G1 | `wbs_s1` | `li subfc subfe addi blr` | HIT | HIT |
| G1 | `wbs_s2` | `subfc subfe addi blr` | HIT | HIT |
| G1 | `wbs_s3` | `subfic subfe rlwinm addi blr` | HIT | HIT |
| G1 | `wbs_s4` | `cntlzw rlwinm xori addi blr` | HIT | HIT |
| G1 | `wbs_s5` | `cmpi li bclr li blr` | HIT | HIT |
| G1 | `wbs_s6` | `cntlzw cntlzw rlwinm xori addi blr` | *abstain* | HIT |
| G1 | `wbs_b1` | `li subfc subfe rlwinm blr` | *abstain* | HIT |
| G1 | `wbs_b2` | `subfc subfe rlwinm blr` | *abstain* | HIT |
| G1 | `wbs_b3` | `rlwinm blr` | *abstain* | **MISS** |
| G1 | `wbs_k1` | `srawi addze blr` | HIT | HIT |
| G1 | `wbs_k2` | `rlwinm blr` | HIT | HIT |
| G1 | `wbs_k3` | `subfc subfe rlwinm add blr` | *abstain* | HIT |
| G2 | `S1` | `li subfc subfe rlwinm addi blr` | HIT | HIT |
| G2 | `S2` | `cmpi li bclr li blr` | HIT | HIT |
| G2 | `S3` | `cntlzw rlwinm blr` | *abstain* | HIT |
| G2 | `S4` | `addic subfe blr` | *abstain* | HIT |
| G2 | `S5` | `srawi addze blr` | HIT | HIT |
| G2 | `S6` | `li divw blr` | *abstain* | HIT |
| G2 | `S7` | `rlwinm blr` | HIT | HIT |
| G2 | `S8` | `oris ori blr` | *abstain* | HIT |
| G2 | `S9` | `lbz extsb addi blr` | *abstain* | HIT |
| G2 | `S10` | `lha addi blr` | *abstain* | HIT |
| G2 | `S11` | `li li subfc subfe and blr` | **MISS** | *abstain* |
| G2 | `S12` | `li subfc subfe addi blr` | **MISS** | HIT |

| policy | reading | on GRID-1 | on GRID-2 | of the cells it claims |
|---|---|---|---|---|
| PUBLISHED | **R1** | 7 H / 0 M / 5 A | 4 H / 2 M / 6 A | **11 / 13** |
| PUBLISHED | **R2** | 11 H / 1 M / 0 A | 11 H / 0 M / 1 A | **22 / 23** |
| SYMMETRIC | **R1** | 6 H / 0 M / 6 A | 4 H / 2 M / 6 A | **10 / 12** |
| SYMMETRIC | **R2** | 9 H / 0 M / 3 A | 9 H / 0 M / 3 A | **18 / 18** |

### 12.3 Reading the table

**R2's reading is the more complete and the more accurate**, and the gap is not
close: under the symmetric policy it is **18 for 18** on cells frozen by two
lanes on two grids, and its one published-policy miss (`wbs_b3`) is a cell §6.3
shows neither lane could have got.

**R1's eleven abstentions are all forced by R1's own document**, not by this
lane's judgement:

* `S9`, `S10` — **no convert table** (§1). The cost of the missing thirteenth.
* `S6` — R1 §9.5 explicitly does not claim the non-power-of-two divide.
* `S8` — R1 §2.1 names `ori`/`or` but states no wide-constant split.
* `wbs_b1`, `wbs_b2`, `wbs_k3`, `S3`, `S4` — R1's `{0,1}` ⇒ nibble-5 exclusion,
  which §6.2 shows is **wrong**. Without it R1 predicts `wbs_b1`, `wbs_b2` and
  `wbs_k3` correctly and would score three higher. **R1 is penalised here by an
  error that made it too cautious, and the score understates it.**
* `wbs_s6` — R1 §7.6's own retraction of its cost arithmetic.

**R1's two misses are both places where it published a positive claim.** `S11`
is `W-SELECT-5`'s adoption-ready mask clause (§10). `S12` is P3.4 (§11) — and
**R1's own §6.1 gives the right answer**: scored off the calibration observation
rather than off the §8 verdict, R1 hits `S12`. It is scored MISS because a
published PREREG verdict is what a reader will act on.

**Two cross-grid reproducibility controls, both exact.** `wbs_k1` = `S5`
(`x / 8` → `srawi · addze · blr`) and `wbs_s1` ≡ `S12` (the `?:` and `if`
spellings of `x < 10u ? 1 : 2` → `li · subc · subfe · addi · blr`), each compiled
by a different lane, byte-identical.

---

## 13. CORRECTIONS FILED, by row number

Each is appended as a **dated note** to the lane's own document — the way
`CEILING.md` was annotated. **Nothing is rewritten**: no frozen section, no
`frozen.tsv`, no rung, no board row's original text.

| row | lane | what is wrong | correct |
|---|---|---|---|
| **#2040** | R1 | "**SIXTEEN** operator × type arrays" | **13 slots / 17 bodies**; the enumeration omits `convert` @ `0x10b1fd08` (§1) |
| **#2042** | R1 | `wbs_s4` as evidence for the tie rule | the cell is against zero and routes to `FUN_10c1a908` before the race (§6.3) |
| **#2043** | R1 | PREREG **P3.4 = HIT** | **MISS** — contradicted by R1's own calibration cell `wbk_2` and by R2's `S12` (§11) |
| **#2044** | R1 | "RECORD FORMS ARE **NOT** A FUSION" | it **is** a fusion, and the rewriter is `FUN_10c0b4c0` (§5) |
| **#2046** | R1 | `W-SELECT-5`'s "contiguous mask ⇒ `rlwinm`" as **adoption-ready** | over-general; three forms exist, `S11` is `li`+`and` (§10) |
| **#2047** | R1 | `FUN_10c194b8` is the **bool-typed** relational path | it is the **floating-point** path; the `{0,1}` exclusion is unnecessary and wrong (§6.2) |
| **#2102** | R2 | "normalises every unsigned relation to **`ULE`**" | to **`UGT`** — the enum is `8 UGT, 9 ULE` (§8) |
| **#2103** | R2 | cells `S3`/`S4` as evidence about the two expanders | both are against zero and route to `FUN_10c1a908` (§6.3) |
| **#2107** | R2 | "the one unread pass is `FUN_10c1772b`" | the form-chooser is **`FUN_10c0a2e2`** (§4) |
| — | R2 | `W-SELECT-3`'s `ULE`; `W-SELECT-5`'s `FUN_10c1772b` | as above |

**Not corrected, deliberately:** R1's 46 (§2) and R1's `FUN_10c0d57e` (§3) are
true statements about real objects and are left standing with their object named
beside them.

---

## 14. THE MERGED `lower_expr` BUILD LIST, and the reconciled DISCLOSURE scope

### 14.1 What to build, in order, with the row that supports each clause

| # | clause | from | scope |
|---:|---|---|---|
| 1 | **the operand type index** — `(nibble << 12) \| size` → 0…25, `FUN_10bd7c10` | **R2 §2.1** (R1 has no type-index map) | **black box** — a port models C types already |
| 2 | **the 13 operator tables** — copy, load-D, load-X, store-D, store-X, negate, add, sub, mul, div, cmp-imm, cmp-reg, **convert** | **R2 §2.2 / #2100**; R1 §2.2 for twelve of them | **black box**, one cell per (operator, type) |
| 3 | **the signedness split is three tables, not a per-site test** | **R2 §2.2**, mechanism from **R1 §2.2 / #2041**; #1788 has an obj (**#2109**) | **black box** |
| 4 | **opcode → PPC encoding** | R1 §9.2 item 1 / **#2040** | **free** — it is the ISA |
| 5 | **the immediate-fit rule** — signed-16 for nibble 1, unsigned-16 for 2/3/4, else force to a register; **and no fit test at all in `ori`/`xori`** | fit rule **R1 §2.3 = R2 §2.4** (independent agreement); the `ori` exception **R2 §2.4, cell `S8`** | **black box** |
| 6 | **the constant-operand tests** — `mulli` unless a power of two; power-of-two signed `/` ⇒ `srawi`+`addze`; **no magic-number multiply and no shift/add at `/O1`** | `srawi`+`addze` **both lanes** (`wbs_k1` = `S5`); the two refutations **R2 §2.4 / #2105** | **black box** |
| 7 | **the narrowing fusions** — `lhz`+`extsh` ⇒ `lha`; `lbz`+`extsb` stays two | **R2 §5.1 + cells `S9`/`S10`**; the *site* is arms 3/4/5 of `FUN_10c182b4` (**this lane, §3**) | **black box** |
| 8 | **the record form** — when the defining instruction has one, use it, delete the compare, result in `cr0`; otherwise `cr6` | mechanism **R2 §2.6 / #2106**; `cr6`-default **R1 §2.3**; the rewriter `FUN_10c0b4c0` **this lane §5** | **black box** if stated this way; `opcode+1` and bit `0x10` are c2-private and need a row only if copied |
| 9 | **the if-conversion predicate** — a two-way `if` keeps its branch iff an arm has a side effect or the relation is signed with a non-zero bound | **R2 §6.3c / #2104**, 8 objs; corroborated by **R1's `wbk_2`** and by `wbs_s1` ≡ `S12` | **black box**, site unknown |
| 10 | **the relational-as-value family** — relation normalisation, the two expanders' emission lists, the `base + ((CA−1) & delta)` identity | emission **R1 §3.2/§3.3 = R2 §3.1/§3.2**; the enum **R1 §3.5 / #2045** (§8) | **shapes are black box**; the **cost model and tie rule are NOT** (§7) |
| 11 | **the against-zero fast path** — `FUN_10c1a908`, ~20 arms | **neither lane** — this lane, §6.2 | **UNREAD. A port that handles `x == 0`, `x != 0` or `x < 0` as a value must read it, and it is upstream of item 10** |
| 12 | **the `rlandi` expansion** — three forms, `FUN_10c0a2e2` | routine **R1 §2.4**; that it is unpredictable **R2 §6.3b / #2107**; that it is three forms and which routine **this lane §4/§10** | **UNREAD. Blocks any body containing `x & K`** |
| 13 | **then** registers, WB-D §3.4 | WB-D, re-confirmed on 24 more cells by both lanes | **free** |

**The first class both lanes named survives the merge with one change.** R1's
`expr_straightline_int` excluded `{0,1}` results; §6.2 shows that exclusion is
wrong and unnecessary. R2's `expr_int_straightline` excluded `&`/`|`/`^` with a
constant; §4 shows `|` and `^` are safe (item 5) and only `&` is not.
**The merged predicate is R2's, minus the `|`/`^` half of its exclusion, plus an
exclusion for any relational compared against zero** (item 11) — a boundary
neither lane drew, because neither knew it was there.

**Predicted reach: `0`.** Unchanged. Both lanes registered it, both were right,
and nothing here moves it.

### 14.2 The reconciled DISCLOSURE scope

The two documents pre-drafted **ten** rows under **five** shared names, with
different contents. This is the merged set. Nothing is carried into
`DISCLOSURE.md` as an adopted row, because no lane in this family has changed
`crates/`.

| merged row | kind | why |
|---|---|---|
| **the operator × type tables + the type index** (R1's W-SELECT-2 ∪ R2's W-SELECT-2) | **adoption-ready, but the black-box alternative is COMPLETE and should be used** | `select_grid.cpp` × 2, `calib.cpp` × 2 and `diag.cpp` re-derive every live entry, the signedness split, `srawi`+`addze`, the `lha` fusion, `clrlwi`, `oris`+`ori` and the absence of a magic multiply **with no address**. Carry the row only if the table *layout* or the type-slot *numbering* is copied. Use **R2's** table — R1's is missing `convert`. |
| **the machine-opcode enum and its attribute word** (R1's W-SELECT-1 ∪ R2's W-SELECT-1) | **route** | No obj exposes an opcode *number* or a *bit position*. A port that says "use the `.` variant when one exists" needs nothing. Use **R2's** wording — it names the attribute bits and the simplified-mnemonic table `0x10b1d190`. |
| **the relational-as-value COST MODEL and TIE RULE** (both W-SELECT-3) | **route — and this is the one that genuinely needs it** | Confirmed jointly, and **strengthened**: §6.3 shows not one cell in either grid reached `FUN_10c1b517`'s comparison. The emitted *shapes* are black box; the cost function, the `<=`, and `500 ⇒ branch` are not. Use **R1's** relation-code table (§8). |
| **the ARM AND TABLE COUNTS** (R2's W-SELECT-4, extended) | **route — the SECOND row that needs an address** | 13 tables, 41 dispatch arms, 18 expansion arms. These are the numbers `#2046`'s ≈640 lines and `#2108`'s ≈60 rules rest on, and **no obj yields a count of arms**. A port that implements the rules needs none of them; a lane that prices the work needs all three. |
| **`FUN_10c0a2e2`, the `rlandi` expander** (replaces R2's W-SELECT-5) | **navigation, held — NOT adoptable** | Recorded so the next lane starts at `LAB_10c0a4cb` and not at `FUN_10c1772b`. §10's eight cells bound it; the predicate is unread. |
| **`FUN_10c1a908`, the against-zero relational** (**new — neither lane proposed it**) | **navigation, held** | Twenty arms, unread by both lanes, and it is what five of the 24 graded cells actually exercised. |
| R1's **W-SELECT-4** (relation codes) and R1's/R2's **W-SELECT-5** (`srawi`+`addze`, the mask) | **folded into the rows above** | `srawi`+`addze` is fully black box (`wbs_k1` = `S5`, two lanes, identical bytes) and needs no row; the mask clause is retracted (§10). |

---

## 15. What this does NOT settle

* **`FUN_10c1a908`'s twenty arms.** Named, not enumerated. Five graded cells
  depend on them and this lane read only the dispatch.
* **`FUN_10c0a2e2`'s form predicate.** The three outcomes and four inputs are
  located (§4); the predicate is not derived, and §10's eight cells are left as
  the constraint set for whoever does.
* **`DAT_10c2ed00`.** The second conjunct of the against-zero enable. `S10`
  settles `DAT_10c2e2fc` black box; nothing settles this one, so §6.2's
  conclusion is stated as "the guard is live unless `DAT_10c2ed00` is set at
  `/O1`", not as a certainty.
* **`FUN_10c194b8`'s body.** Identified as the float path from its own locals
  and constants; not read.
* **Whether `0x10b18990`'s 19-value space is the relation enum.** R1 §3.5 left
  it open and this lane did not close it, although §8 now fixes the *other*
  space's values exactly, which is half of what an answer needs.
* **Any float or VMX selection.** Neither lane read it and neither did this one.
* **No obj was compiled.** Every measured word here is inherited. Nothing in
  §§1–11 has been re-graded against a new cell, and §12's cross-score is a
  hand-built derivation over cells two other lanes paid for.
