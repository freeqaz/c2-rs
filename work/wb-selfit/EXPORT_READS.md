# wb-selfit — the export reads, with the exact evidence for each

Image verified before the first grep:

```
sha256sum ~/ghidra-projects/bin/c2dll
c80981c015166effecc71ad8112d5577a065b2300891dfdb02b9c13787a66258
```

matching `docs/whitebox/C2_MAP_METHOD.md` §0. Flat export only
(`~/ghidra-projects/export/c2/{decomp_all.c,objdump_intel.asm,data.tsv,strings.tsv,functions.tsv}`);
the Ghidra project was never opened.

---

## E1 — `FUN_10c04cb9` installs THIRTEEN slots and SEVENTEEN bodies

`decomp_all.c:205863`, function size 180 bytes, verbatim:

```c
void FUN_10c04cb9(void)
{
  DAT_10c6fddc = &DAT_10c38f30;   /*  1 copy        */
  DAT_10c6fdd8 = &DAT_10c38f98;   /*  2 load  D     */
  DAT_10c6fdd4 = &DAT_10c39068;   /*  3 load  X     */
  DAT_10c6fdd0 = &DAT_10c39138;   /*  4 store D     */
  DAT_10c6fdcc = &DAT_10c391a0;   /*  5 store X     */
  DAT_10c6fdc8 = &DAT_10c392d8;   /*  6 negate      */
  DAT_10c6fdc4 = &DAT_10c39340;   /*  7 add         */
  DAT_10c6fdc0 = &DAT_10c393a8;   /*  8 sub         */
  DAT_10c6fdbc = &DAT_10c39410;   /*  9 mul         */
  DAT_10c6fdb8 = &DAT_10c39478;   /* 10 div         */
  DAT_10c6fdb4 = &DAT_10c394e0;   /* 11 cmp imm     */
  DAT_10c6fdb0 = &DAT_10c39548;   /* 12 cmp reg     */
  DAT_10c6fdac = &DAT_10b1fd08;   /* 13 convert     <-- MISSING from wb-select */
  if (DAT_10c2e978 != 0) {
    DAT_10c6fdd8 = &DAT_10c39000;   /* load  D, -QVMX128  (OVERWRITES slot 2) */
    DAT_10c6fdd4 = &DAT_10c390d0;   /* load  X, -QVMX128  (OVERWRITES slot 3) */
    DAT_10c6fdd0 = &DAT_10c39208;   /* store D, -QVMX128  (OVERWRITES slot 4) */
    DAT_10c6fdcc = &DAT_10c39270;   /* store X, -QVMX128  (OVERWRITES slot 5) */
  }
  return;
}
```

* **13 destination slots**, `DAT_10c6fdac`…`DAT_10c6fddc`, contiguous at a
  4-byte stride.
* **17 distinct table bodies**, because the four `-QVMX128` alternates are
  bodies for slots that already exist, not slots of their own — the second
  block *reassigns* `DAT_10c6fdd8/d4/d0/cc`.
* The four alternates replace **load-D, load-X, store-D, store-X**, which is
  what `wb-select2` §2.2 says ("the only difference is index 25,
  `lvx`→`lvx128`, `stvx`→`stvx128`").

## E2 — the dispatch: 174 opcodes, 41 jump-table arms, 46 case labels

`objdump_intel.asm:385341`:

```
10c0f894: mov    eax,[esi+0x4]                 ; tuple->opcode
10c0f897: add    eax,0xfffffd82                ; -= 0x27e
10c0f89c: cmp    eax,0xad                      ; 174 opcodes
10c0f8a1: ja     0x10c0fb2a                    ; default
10c0f8a7: movzx  eax,BYTE PTR [eax+0x10c0fbd6] ; byte index
10c0f8ae: jmp    DWORD PTR [eax*4+0x10c0fb32]  ; jump table
```

`(0x10c0fbd6 − 0x10c0fb32) / 4 = 0xa4 / 4 = 41`. **41 arms**, and the number is
re-derivable from the two VAs `wb-select2` itself published.

Ghidra's decompilation of the same switch (`decomp_all.c:214218`) carries **46
`case` labels** in **39 maximal label groups**, two of which (`0x2cb` `|`,
`0x2cc` `^`) fall into one body via `goto LAB_10c0f970`, so **38 distinct
decompiled bodies**. `work/wb-selfit/armcount.txt` is the grouping.

## E3 — the two in-place switches are BOTH real and are DIFFERENT passes

| | `FUN_10c0d57e` | `FUN_10c182b4` |
|---|---|---|
| size | **3899** B | **426** B |
| shape | binary decision tree | byte index `0x10c184a8` + jump table `0x10c18460`, **18 arms** (cases 0…0x10 plus the default) |
| index | tuple **and** machine opcodes, one space | machine opcode only, `op − 1 ≤ 0x292` |
| callers | `decomp_all.c:200878`, `:201334`, `:213786` — inside the lowering | **one**: `FUN_10b7dd2c` @ `0x10b7dd2c`, a top-level phase driver, gated on `DAT_10c2e2fc`; the pass runs its list **twice** (`local_c = 2`) |
| `0x2f0`/`0x2f4` | **yes** — `decomp_all.c:213277-213285` calls `FUN_10c21719` / `FUN_10c216f5` | no |
| `rlandi` (`0x26e`/`0x26f`) | **yes** — `decomp_all.c:212862`: `if (uVar7 - 0x26e < 2) FUN_10c0a2e2(param_2);` | arm 13 → `FUN_10c1772b` |

`0x10c216f5` and `0x10c21719` are their own 19- and 25-byte functions
(`functions.tsv`), so `wb-select` §1.2's correction to WB-D §4 is right about
the **call sites** (they are inside `FUN_10c0d57e`) and WB-D is right that the
targets are thunks.

## E4 — the `rlandi` expander is `FUN_10c0a2e2`, not `FUN_10c1772b`

`FUN_10c0a2e2` (1871 B) has exactly two callers, and both gate on `rlandi`:

```
decomp_all.c:212862   (inside FUN_10c0d57e)  if (uVar7 - 0x26e < 2) FUN_10c0a2e2(...)
decomp_all.c:226196   FUN_10c1cf59:  if (op != 0x26e && op != 0x26f) return; FUN_10c0a2e2(...)
```

and it is the routine that picks the **form**:

* `local_20 = FUN_10c04daf(mask, &mb, &me)` — the contiguity analysis
  (`FUN_10c04daf` is the function immediately after the table installer);
* `local_30 = 0x133` / `local_34 = 0x134` → **`rlwinm` / `rlwinm.`**;
* the `0x10c0a802` path → `andi.` / `andis.` (`param_1[1] = 0x1e`), guarded by
  `(DAT_10c2ecf0 == 0) && (DAT_10c2e310 == 0)` and by a CR0-availability query
  `thunk_FUN_10bd5a62(*param_1, 0x10c309a8)`;
* `LAB_10c0a9a6` → mints a constant (`FUN_10c08e38(0xd, …)`) and sets
  `param_1[1] = 0x1a` (`and.`) or `0x19` (**`and`**) — **this is the `li` + `and`
  form** `wb-select2`'s S11 and its seven diagnostic cells saw.

`FUN_10c1772b` (1007 B) is real and is arm 13 of `FUN_10c182b4`, but it is a
**mask-merging peephole** — it recomputes a mask, compares two costs through
`FUN_10c0a170`, and rewrites operand values; the path read here mints no
opcode. It is not the form-chooser.

## E5 — the tie goes to `cntlzw`, and BOTH lanes' tie evidence is confounded

`FUN_10c1b517` (`decomp_all.c:224599`), 140 B, verbatim:

```c
if ((*(ushort *)((int)param_1 + 10) & 0xf000) == 0x5000) { FUN_10c194b8(param_1); return; }
iVar1 = **(int **)param_1[10];
iVar2 = FUN_10c1b2fa(DAT_10c2e2f4);
if ((iVar2 != 0) && (<either compare operand is the constant 0>)) {
        FUN_10c1a908(param_1); return; }
uVar3 = FUN_10c1ac5c(param_1,0);      /* carry  cost */
uVar4 = FUN_10c1af2d(param_1,0);      /* cntlzw cost */
if (uVar4 <= uVar3) { FUN_10c1af2d(param_1,1); return; }
FUN_10c1ac5c(param_1,1);
```

`uVar4 <= uVar3` ⇒ **ties go to `cntlzw`**. Both lanes state this and both are
right.

But the **against-zero test runs FIRST**. `FUN_10c1b2fa` (`:224488`) returns 1
iff `DAT_10c2ed00 == 0 && DAT_10c2e2f4 != 0 && DAT_10c2e2fc != 0`, and
`DAT_10c2e2fc != 0` at `/O1` is established **black box** by `wb-select2`'s S10:
`lha` is the `lhz`+`extsh` fusion produced by arms 3/4/5 of `FUN_10c182b4`,
whose only caller gates on that same `DAT_10c2e2fc`.

`FUN_10c1a908` (`:223869`) normalises which side is zero through the **negate**
table `0x10b189cc` and dispatches on the relation code to ~20 arms; it handles
**arbitrary** result operands (`FUN_10c19859` materialises one that is neither
`1` nor `−1`). So `x == 0 ? 5 : 6` is squarely inside its remit.

**Every against-zero cell in both grids therefore has two live explanations and
the obj cannot separate them**: `wbs_s4`, `wbs_s6`, `wbs_b3`, `S3`, `S4`.

## E6 — `FUN_10c194b8` is the FLOATING-POINT path, not a `bool` path

`decomp_all.c:222988`, 890 B. Its locals are `double *pdVar3` and `float
fVar8`; it tests `*pdVar3 == 0.0` and the operand opcode `0x6a`. Type nibble 5
is **floating point** in `FUN_10bd7c10`'s own map (sizes 4→13, 8→14 — the `f32`
and `f64` slots of every operator table). `wb-select2` §3's label
(`float_path`) is right; `wb-select` §7.6's "bool-typed" is wrong.

## E7 — the record form: a fusion whose action is literally `opcode + 1`

`FUN_10c0b300` (`:211102`) is the **predicate** — it returns `1`/`0` and tests
`(&DAT_10c3afd8)[puVar4[1]] & 0x10`. The **rewriter** is `FUN_10c0b4c0` @
`0x10c0b4c0` (`:211186`), and it does all four things at once:

* walks **backwards** (`iVar9 = param_1[4]`, then `iVar9 = *(int *)(iVar9 + 0x10)`);
* promotes `addi` (`0xb`) → `addic` (`0xc`) with a minted carry operand;
* checks `((&DAT_10c3afd8)[*(int *)(iVar9 + 4)] & 0x10) == 0 → return 0`;
* **`*(int *)(iVar9 + 4) = *(int *)(iVar9 + 4) + 1;`** and then
  `FUN_10bd5516(param_1)` — **deletes the compare**.

So it is a fusion **and** the fusion's action is `opcode + 1`. The two lanes'
prose agrees; only their P4.4 verdicts and their board headlines disagree.

## E8 — the relation-code enum, read from its own name array

`data.tsv` gives a pointer array at `0x10c38690` ascending, into the string pool
descending from `0x10b197f4`. Decoded from the verified image:

```
code  0 ILLEGAL   1 EQ    2 NE    3 LT    4 GT    5 LE    6 GE
code  7 ULT       8 UGT   9 ULE  10 UGE  11 SO   12 NSO  13 NS
```

`wb-select` §3.5 derived `1 ==, 2 !=, 3 <, 4 >, 5 <=, 6 >=, 7 <u, 8 >u,
9 <=u, 10 >=u` from the remap tables' fixed points and involutions, with no
access to these strings. **It is exactly right.**

`wb-select2` §3.1 published `4 LE, 5 GT, 8 ULE, 9 UGT` — **two transposed
pairs**, and it published them beside the address `0x10b197c0`–`0x10b197f4`
that refutes them. The consequences are naming-only: the canonical form is
**`UGT`** (code 8), not `ULE`, and `0x10b189a4`'s mapping is
`3 LT→7 ULT, 4 GT→8 UGT, 5 LE→9 ULE, 6 GE→10 UGE`.
