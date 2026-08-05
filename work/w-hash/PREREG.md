# w-hash — pre-registration

Committed **before** any probe script in this directory exists and before any
grid is run. Scored verbatim in `docs/rungs/*w-hash*`; the wrong ones stay on
the page.

Board numbers taken: **#760**–.

Lane target: convert `src/system/math/Sort.cpp` (one function,
`?HashString@@YAHPBDH@Z`, 80 B, 0 relocations, leaf, label-free) and take TU
match 9 → 10.

---

## 0. What is already re-derived — NOT a prediction, measured before this file

The brief hands me eight refusals and says *"re-derive the list yourself first"*.
Done, off the obj (`work/w-hash/Sort.obj`, workload flags, `scripts/gt_dump.py`),
before writing anything below. The obj reproduces the brief exactly: 917 B,
5 sections (`.drectve`, `.debug$S`, `.XBLD$W`×2, `.text`), `nrel = 0`,
14 symbols, no `$M`, no `$T`, no `.pdata`, no `.data`, no `.rdata`. `.text` is
80 B:

```
0000  lbz  r11,0(r3)     0018  lbzu  r10,1(r9)    0030  mullw r7,r7,r4
0004  mr   r9,r3         001c  add   r8,r8,r11    0034  andc  r6,r4,r10
0008  li   r10,0         0020  mr.   r11,r10      0038  twi   6,r4,0
000c  cmplwi cr0,r11,0   0024  rotlwi r10,r8,1    003c  subf  r10,r7,r8
0010  bt   2,+56         0028  divw  r7,r8,r4     0040  twi   5,r6,-1
0014  mulli r8,r10,127   002c  addi  r10,r10,-1   0044  bf    2,-48
                                                  0048  mr    r3,r10
                                                  004c  blr
```

**And the IL is decoded by hand as well** — the brief's list is a list of
*emitted* facts and the port consumes IL, so a re-derivation that stops at the
disassembly has not re-derived the thing that matters. Segment
`?HashString@@YAHPBDH@Z`, 261 B, from `4C 4F 11`:

```
53 53 26 e6 09 46 2d eb 09 2d ea 09 4c   scopes; formals u_0x09EB(i), u_0x09EA(str)
4c 4f 11 53                              body marker, body scope
26 ee 09  33 86 41 74 00  32 86 41 74 4b     ret = 0
53 26 ef 09  b9 ea 09 <ptr>  2c <uchar*> 00  32 <uchar*> 4b   u = (uchar*)str
3a f1 09                                 JUMP  -> L_09F1     (loop ROTATION: to the test)
29 f2 09                                 L_09F2:              (the increment)
26 ef 09 33 86 41 12 01 35 <uchar*> 4b       u += 1
29 f1 09                                 L_09F1:              (the test)
b9 ef 09 <uchar*> 30 82 12 20 2c <int> 00 33 <int> 00 20      *u != 0
38 f3 09                                 brFALSE -> L_09F3   (loop exit)
53 26 ee 09 …  04 … 02 … 06  32 <int> 4b     ret = (*u + ret*0x7F) % i
54 04
3a f2 09                                 JUMP  -> L_09F2      <== THE BACK EDGE
29 f3 09                                 L_09F3:
b9 ee 09 <int> 41 <int> 3a ed 09 54 03 54 02  return ret
29 ed 09 4f 12 47 54 01 54 00 …          epilogue label, fn tail
```

`04` MUL, `02` ADD, `06` MOD, `20` CMP-NE, `2c` CONVERT, `30` INDIRECT-LOAD,
`35` compound-add-assign, `33` LIT, `32` STORE, `26` designator, `b9` LOAD.

### 0.1 My count is ELEVEN, not eight — and the extra ones are not cosmetic

Counted the way `w-conv` counted the frontier: independent facts the port does
not have. The brief's eight are all real and all reproduce. Three more that its
list does not name, and **two of the three are properties of the loop's
*schedule of values*, not of its instruction vocabulary**:

| # | refusal | in the brief's 8? |
|---|---|---|
| 1 | the back edge (`bf 2,-48`) — no `Selected` variant encodes one | yes (#7) |
| 2 | signed `%` as `divw` + `mullw` + `subf` | yes (#1) |
| 3 | two `twi` traps | yes (#2) |
| 4 | the 3-instruction trap predicate `rotlwi`/`addi`/`andc` | yes (#2) |
| 5 | update-form `lbzu` — the induction variable folded into the addressing mode | yes (#3) |
| 6 | record-form `mr.` branching on cr0 | yes (#4) |
| 7 | `mulli` | yes (#5) |
| 8 | the interleaved schedule (predicate between `divw` and `mullw`) | yes (#6) |
| **9** | **LOOP ROTATION** — the IL is `goto test; incr; test:`; the obj is `guard; body; back-edge`. The test is *duplicated*: once as the entry `cmplwi`/`bt` and once as the loop-closing `mr.`/`bf`. The IL contains **one** test site | **NO** |
| **10** | **MEMORY-REFERENCE PEELING.** `lbz r11,0(r3)` loads iteration *k*'s character **before** the loop; `lbzu r10,1(r9)` inside loads iteration *k+1*'s. So `r11` is **loop-carried** and the body consumes a value produced by the *previous* iteration. There is no IL token for this | **NO** |
| **11** | **register allocation across the back edge**, `w-loop` §4's untouched L1: four values live across the edge (`r8` accumulator temp, `r9` pointer, `r10` `ret` *and* a modulo scratch reusing the same register, `r11` the carried char), plus a descending temp file `r7`/`r6` inside the modulo | **NO** (w-loop names L1 as untouched but does not count it as a refusal of this TU) |

Also present and not separately counted: a **forward conditional branch inside a
non-tail leaf** (`bt 2,+56`). `Selected::CondPair` is a two-arm *tail call*; a
`Plain` leaf with an internal forward `bc` has no variant either.

So the honest price is **11**, the fifth consecutive cross-check of a frontier
TU to come back dearer than the list it was handed (`negate_test` 10 v 4,
`xboxmem` 15 v 4, `mmio` 17 v 5 twice, `Sort` 11 v 8).

---

## 1. Predictions

Each carries a named rival. Scored verbatim.

| # | prediction | registered rival |
|---|---|---|
| **R1** | The baseline reproduces to the digit: match **9**, mismatch 0, codegen-gap 0, vocab-gap 862, capture-fail 7; A/B/C/D/E = 28 (LO 27)/338/169/9/2; FRONTIER **18**; FBM **0.16654**, fnbyte-exact **29,801**, fnbyte-partial **9,375**, **fnbyte-differs 0** | any digit off ⇒ report that before anything else |
| **R2** | The signed `%` lowering is **separable from the loop**: `int P(int a,int b){ return a%b; }` as a plain leaf emits the same `divw`/`mullw`/`subf` **and the same two `twi` with the same three-instruction predicate**, at the same constants (`twi 6,rB,0` and `twi 5,rX,-1`) | **R-R2:** the traps are a property of the *loop* context (c2 hoists a divide-by-zero check out of a loop) and a straight-line `%` emits something else — under which mechanisms 2/3/4 cannot be measured outside the loop at all |
| **R3** | The `twi` pair is **keyed on the divisor being a non-constant**: `a % 7` emits **no `twi` at all** and no predicate, because c2 can prove both guards statically | **R-R3:** the traps are unconditional on the operator, literal divisor or not |
| **R4** | **Unsigned** `%` emits `divwu` and **exactly one** `twi` (the zero-divisor guard), with **no** `rotlwi`/`addi`/`andc` predicate — the `INT_MIN/-1` case cannot arise unsigned | **R-R4:** two traps regardless of signedness (the predicate is a fixed idiom c2 emits for the operator, not for the overflow case) |
| **R5** | `/` and `%` at the same signedness share the **identical** trap pair and predicate; they differ only by the trailing `mullw`+`subf` | **R-R5:** `/` needs no `subf` and c2 therefore schedules the predicate differently, so the two are separate lowerings |
| **R6** | The predicate `rotlwi rX,rN,1 ; addi rX,rX,-1 ; andc rY,rD,rX` is computed **from the dividend**, and `twi 5,rY,-1` fires exactly when `rN == INT_MIN` **and** `rD == -1`. Concretely: `rotlwi(n,1)-1` is `-1` iff `n == INT_MIN` (`0x80000000` rotates to `1`), so `andc(d, that)` is `d` iff `n == INT_MIN` and `0` otherwise — and `twi 5` (LT\|GT\|EQ? — `5` = GT\|EQ unsigned? scored from the encoding) against `-1` then tests `d == -1`. **I will verify the bit meaning of `twi 6` and `twi 5` from the encoding rather than asserting it** | **R-R6:** the predicate is not a `INT_MIN`-detector at all and my reading of `rotlwi`+`addi`+`andc` is wrong |
| **R7** | **`mulli` is already reachable.** `int P(int a){ return a*127; }` is an existing straight-line chain and the port already emits `mulli`; refusal 7 is therefore the **cheapest** of the eleven and may already be zero | **R-R7:** the port materializes `li`+`mullw` and `mulli` is a genuine gap |
| **R8** | **The conversion does NOT land in this lane, and the mechanism that stops it is #10 (memory-reference peeling) together with #11 (allocation across the edge)** — not the back edge, not the traps. Concretely: the emitted register assignment is not a function of the IL body's own token order, so no recognizer that reads this body alone can produce it without a fitted table | **R-R8:** the allocation *is* derivable — the peeled load and the rotation follow a rule the grid can state and validate out of sample, and the TU converts |
| **R9** | A `%`-by-variable **leaf** (no loop) is inside factor A/B/C for at least one workload TU, so shipping R2's lowering is worth a non-zero number of *emitted functions* even though it converts no TU by itself | **R-R9:** zero — every `%` in the workload is in a body with other refusals |
| **R10** | Gate unmoved but for the fixtures I add: `gate.sh` grows by **exactly 18 verdicts per fixture**; sweep ungraded **96** and cross ungraded **388** both HOLD; `fnbyte-differs` stays **0** | any move in `fnbyte-differs` ⇒ I shipped a wrong emit |

### 1.1 Registered bias

**I want R8 to lose.** R8 is written as a prediction *against* the outcome this
lane is commissioned for, because thirty lanes' evidence says the frontier is
expensive and one lane's optimism is not evidence. If R-R8 wins, the TU converts
and R8 is a miss I will be glad to record.

**I also want R3 to win** — it is the reading under which the trap machinery is a
small, bounded, cross-productable rule rather than an unconditional tax. That is
the direction of my bias and R3 is stated so a loss is visible.

## 2. What I will NOT do

* **No rule fitted on `Sort.cpp`'s own cells.** Anything shipped is measured over
  its own full cross product (signedness × divisor kind × dividend sign ×
  `INT_MIN` × 0 × ±1), because *traps and division are exactly where boundary
  values hide* and the single-cell trap has fired five times on this project.
* **No permissive label `resolve`** unless a `Selected` variant with a back edge
  exists to call it — `w-loop` #746's objection lifts only when the caller does.
* **No "the port is close" claim from a byte fraction.** `Sort.cpp` is 0/80 today
  and the number that moves it is the differential, not the ranker.

## 3. The fixture owed (#747)

`w-loop` §6 registered that **neither `expr_sweep.sh` nor `mode_cross.sh` can
produce a two-function TU of mixed frame class**, so both would grade a wrong
label charge green, and named the fixture as owed by whoever ships the
relaxation. If this lane ships the relaxation it ships the fixture. If it does
not ship the relaxation, it still owes the *corpus* finding a committed cell,
and I register now that I will add the fixture either way if a leaf-with-a-back-
edge ever becomes emittable, and will say plainly that I did not if it does not.
