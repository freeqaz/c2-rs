# WB_REGCELLS — the two empty cells of `P_REGALLOC` §7, filled

> **PROVENANCE — DISASSEMBLY-DERIVED + OBJ-GRADED.** Lane `w-regcells`, L3 of
> [`../REGALLOC_BRIEF_2026-08-27.md`](../REGALLOC_BRIEF_2026-08-27.md), funded
> by [`../DECISIONS_2026-08-22.md`](../DECISIONS_2026-08-22.md) § Decision 20.
> Addresses are absolute VAs in the image pinned in
> [`C2_MAP_METHOD.md`](C2_MAP_METHOD.md) §0 —
> `sha256 c80981c015166effecc71ad8112d5577a065b2300891dfdb02b9c13787a66258`,
> **verified by this lane before the first read**. Navigation until a row lands
> in [`DISCLOSURE.md`](DISCLOSURE.md); **nothing here was adopted into
> `crates/`, so this lane files no DISCLOSURE row.**
>
> PREREG: `work/w-regcells/PREREG.md`, committed at `2c89de6a4` **before any
> `cl.exe` run by this lane** (tier **PREREG**); addendum 1 at `9d0e23b2d`
> (tier **IN-FLIGHT**, and it says so). Scored in §5.

Lane kind: **characterization**. `Fixtures: none`. `Census: +0`. **Reach 0,
as predicted.**

---

## 0. The headline, both halves

`P_REGALLOC.md` §7 named exactly two things that were read and had **no obj
cell in existence anywhere in this project**. Both are answered, and they
answered differently:

| # | question | verdict |
|---|---|---|
| **Q1** | the FPR order at `0x10c37f20` | **CONFIRMED and `[O]`.** 20 of 20 graded cells at two profiles; **29 of the list's 32 entries witnessed**; four rivals refuted, three of them by ≥18 cells |
| **Q2** | F4's non-call physical def | **THE READ SURVIVES; THE CLAIM THAT NO CELL EXISTS DOES NOT.** Cells exist, and 213 of them have existed for a month in `CODEGEN_ARG_PERM`'s grids unrecognised. They confirm the mechanism's **preconditions** and cannot separate it from pressure. §4 |

**And this lane's own negative prediction for Q2 was WRONG** — recorded as a
miss in §5, with the reasoning error named, because that is the more useful
half of the result.

---

## 1. The read, re-taken at the byte level before any probe

Read-before-probe ([`../WHITEBOX_LEVERAGE_2026-08-21.md`](../WHITEBOX_LEVERAGE_2026-08-21.md)),
and it is what makes the grade below a grade rather than a fit: the tables were
decoded out of the pinned image and written into the prereg **before an obj
existed**. `docs/whitebox/scripts/grade_fpr_cells.py` re-derives them on every
run — **no register list is typed into this document or into that script**.

Image base `0x10b00000`; `.text` RVA `0x1000` → file `0x400`; `.data` RVA
`0x12e000` → file `0x12d200`.

```
0x10c385c4  the 8-entry per-class ordered-list array
            [ 0x10c37de0, 0x10c37f20, 0, 0, 0, 0, 0, 0 ]
            -> only classes 0 (GPR) and 1 (FPR) are image-initialised.  CONFIRMED.

0x10c37de0  class 0, 27 entries, zero-terminated
            r11 r10 r9 r8 r7 r6 r5 r4 r3   r31 r30 r29 ... r15 r14

0x10c37f20  class 1, 32 entries, zero-terminated
            fp0  fp13 fp12 fp11 fp10 fp9 fp8 fp7 fp6 fp5 fp4 fp3 fp2 fp1
                 fp31 fp30 fp29 fp28 fp27 fp26 fp25 fp24 fp23 fp22 fp21
                 fp20 fp19 fp18 fp17 fp16 fp15 fp14
```

Decoded through the name table at `0x10b181c0`, whose index **is** c2's register
number (`4..13` = `r3..r12`, `34..65` = `fp0..fp31`). `P_REGALLOC` §2.1's
transcription of both lists is **exact**; this lane found no discrepancy.

### 1.1 The two lists are ONE rule in two files

Registered in the prereg **before the grid was compiled**, so it is a prediction
and not a pattern noticed afterwards:

> **Each class's list is: the class's scratch register, then the class's
> ARGUMENT registers in DESCENDING order, then the class's non-volatiles in
> DESCENDING order.**
>
> * class 0 — `r11` (scratch), `r10…r3` (the 8 GPR argument registers), `r31…r14`
> * class 1 — `fp0` (scratch), `fp13…fp1` (the 13 FPR argument registers), `fp31…fp14`

Both halves are now obj-confirmed on their own class, which is what makes the
homology a statement about c2 rather than about one table. **A port needs one
list generator, not two tables** — and if a class-5 (VMX) list is ever read out
of `FUN_10bfb00d`'s run-time fill, this is the shape to check it against first.

---

## 2. Q1 — the FPR order. The grid, and every count with its denominator

Grid: [`grids/w-regcells/fpr_grid.cpp`](grids/w-regcells/fpr_grid.cpp), 10
cells, shapes mirroring `grids/wb-regalloc/regorder_grid.cpp`'s G-series with
`double` in place of `int` so that any difference is attributable to the
register **class** and not to the shape. Values come from **globals**: nothing
arrives pre-coloured, so no copy preference biases the selector's cost and the
list order is the only thing left to decide.

Compiled against **real `c2.dll` under wibo** at two profiles:

* **mode W** — `/nologo /c /GR /O1 /Oi /EHsc`, the workload's own;
* **mode X** — `/nologo /Ox /GS- /Gy /c`, the fixture-capture profile.

Graded by `scripts/grade_fpr_cells.py` (transcript: `work/w-regcells/run/grade_fpr.txt`).

### 2.1 What each cell emitted

| cell | mode W — no-preference FPRs, in first-def order | live-across-the-call |
|---|---|---|
| `fpc_g1` | `f0 f13` | — |
| `fpc_g2` | `f0 f13 f12` | — |
| `fpc_g3` | `f0 f13 f11 f12` | — |
| `fpc_g4` | `f0 f13 f11 f12 f10` | — |
| `fpc_l3` | `f0` | **`f31 f30 f29`** |
| `fpc_p1` | `f0 f13 f10 f12 f11 f9 f8 f7` | — |
| `fpc_p2` | `f0 f13 f12 f11` | **`f31 f30 f29 f28 f27 f26 f25 f24 f23 f22 f21 f20 f19 f18 f17 f16`** |
| `fpc_a1` | `f0` (formals stay in `f1`,`f2`) | — |
| `fpc_a2` | `f0 f13` (formals stay in `f1`…`f4`) | — |
| `fpc_w1` | `f0 f13 f12` (`float`, not `double`) | — |

Mode X reaches deeper on three cells — `fpc_p1` walks all the way to **`f2`**
and `fpc_g3`/`fpc_g4` two entries further — which is how the volatile tail of
the list got witnessed at all.

The two sharpest single lines in the whole grid:

```
fpc_g1   lfd 0,0(r11) ; lfd 13,0(r10) ; fadd 1,0,13 ; blr
fpc_l3   lfd 31,0(r11) ; lfd 30,0(r10) ; lfd 29,0(r9) ; bl fpsink ; fadd 0,30,31
```

`fpc_g1` takes `f0` and then **`f13`** with `f1`…`f12` all free: that alone
kills every ascending reading. `fpc_l3` takes **`f31`, `f30`, `f29`** across a
call with `f14`…`f28` all free: that kills every reading of the non-volatile
tail as ascending.

### 2.2 The rival scoreboard — 20 graded cells, **0 unscoreable**

| rival | verdict | cells |
|---|---|---|
| **FR0** — the read: `fp0, fp13…fp1, fp31…fp14` | **SURVIVES** | **20 of 20** |
| FR1 — ascending from `f1` | **REFUTED** | 20 of 20 |
| FR2 — ascending from `f0` | **REFUTED** | 18 of 20 (survives only on `fpc_a1`, where `\|FREE\|`=1) |
| FR3 — descending from `f31` | **REFUTED** | 20 of 20 |
| FR4 — the read direction-confused | **REFUTED** | 18 of 20 |
| FR5 — one list for both classes | refuted **by construction** | no cell's FPR set meets class 0's names |

**Denominators, stated in full.** 10 cells × 2 profiles = 20 graded, **0 scored
`U`** — every cell contained floating point and reached at least the depth its
own prediction named, so no count here rests on an absence. `fpc_a1` is
reported as the cell that *cannot* separate FR0 from FR2/FR4 (it has one
no-preference value), which is why it is listed as their surviving cell rather
than quietly pooled.

### 2.3 How much of the list is actually witnessed — 29 of 32

| entries | witnessed | by |
|---|---|---|
| `fp0` | ✅ | every cell |
| `fp13`…`fp2` (12) | ✅ | `fpc_p1` at mode X |
| `fp1` | ❌ | it is the FP **return** register, so a no-preference value never lands there in these shapes |
| `fp31`…`fp16` (16) | ✅ | `fpc_p2`, both modes |
| `fp15`, `fp14` | ❌ | no cell built enough pressure to reach entry 31 or 32 |

**29 of 32 entries confirmed in position; 3 remain `[R]`.** Stated so the `[O]`
is not read as covering the whole table. `fp1`'s position is the one a port
would most want, and the shape that would witness it is a body with 14
simultaneously live no-preference FP values and no call — not built here.

### 2.4 Profile: the RULE is invariant, the SET is not

The register **sets** differ between mode W and mode X on 4 of 10 cells
(`fpc_g3`, `fpc_g4`, `fpc_p1`, `fpc_p2` — `/Ox` keeps more values live and
walks further down the list). The **order rule** is identical: FR0 = 10/10 at
each mode independently. `P_REGALLOC` §5's warning is about *candidate* order,
and it does not extend to the register list, which is an image constant. **This
lane's own §1.2 prediction — that the SET would be identical — is a MISS**
(§5).

### 2.5 The control that was armed, watched fail, and then passed

`#3336`: a control never watched fail is decoration. Mode X had to gain `/Gy`
to give one COMDAT per cell, so the lane owed a demonstration that `/Gy` moves
no code. First arming: **FAILED on 2 of 10 cells** (`fpc_l3`, `fpc_p2`) — the
two cells containing a `bl`, because a `REL24` displacement is section-layout
dependent by construction. With `REL24`-carrying words masked and nothing else:
**10 of 10 cells' word sequences appear verbatim in the non-`/Gy` blob**, and
the negative arm (a deliberately corrupted cell) is **absent**, so the test can
still fail. `/Gy` is packaging.

The grader carries the same discipline: `grade_fpr_cells.py --selftest` asserts
**6 properties, 3 of which are the grader having to reject** — an ascending
FREE set, `f14` before `f31` in SPAN, and a body with no FP instruction being
scored `U` rather than passing. It also caught a real bug in itself: the first
run classified `bl __savefpr_16` as a clobbering call and silently emptied
`fpc_p2`'s live-across-call set. The fix and the reason are in the source.

---

## 3. Q1's verdict for `P_REGALLOC`

> **`0x10c37f20` moves `[R]` → `[O]`.** The FPR allocation order is
> `fp0, fp13…fp1, fp31…fp14`, confirmed on 20 of 20 graded cells at two
> profiles, 29 of its 32 entries witnessed in position, with four rivals
> refuted (three by ≥18 cells). The selector's *"ties go to the earliest entry
> of the per-class ordered list"* rule (`P_REGALLOC` §3) is therefore confirmed
> on a **second class**, having previously been `[O]` on class 0 only.

Two riders, so it is not over-read:

* **The cost arithmetic is untouched and stays `[R]`.** Same as decision 20 §3
  says for `w-regsel`: on every cell of this grid the cost array is uniformly
  zero over the allowed set and the answer is decided entirely by list order.
* **`fp1`, `fp15`, `fp14` are unwitnessed.** §2.3.

---

## 4. Q2 — F4's non-call physical def

### 4.1 The claim, decomposed before it was tested

An obj cell for *"a bare (non-call) physical def of register `X` removes `X`
from the allowed set of every candidate live at that point"* (`WB_LIVE` §2.1,
§6.2) needs three things:

* **(a)** an allocatable GPR is physically defined by a **non-call** tuple;
* **(b)** a candidate is live across that def;
* **(c)** the narrowing is **observable** — the candidate would otherwise have
  taken it.

### 4.2 (a) and (b) are `[O]`, and the cells were already on disk

`pd_tail` — `int pd_tail(int a,int b){ return gg2(b,a); }` — emits, in full:

```
mr 11, 4 ; mr 4, 3 ; mr 3, 11 ; b gg2
```

**There is no `bl` in that body.** `mr 4,3` is a bare physical def of `r4`; the
candidate holding `b` is live across it; the copy `mr 11,4` exists *because*
`r4` is redefined. That is `CFG_SHAPE.md` §6.2 F's `MemFree` shape — *"copies
`v2` from `r4` to `r11` because both successors need it after clobbering
`r4`"* — **with no call in the body at all**, which is precisely the half
`WB_LIVE` §6.2 recorded as unbuilt: *"no cell of this grid produces a bare
physical-register def with no call."*

> ### ⛔ `P_REGALLOC` §7's *"F4's non-call physical def: still no obj cell in existence"* is **WRONG, and was wrong when it was written**
>
> `scripts/gt_argperm.py --pure` has been compiling this exact shape since
> 2026-07: **152 cells at n = 2…5 plus 61 three-minima cells at n = 6**
> ([`../CODEGEN_ARG_PERM.md`](../CODEGEN_ARG_PERM.md) §2, §5), every one a
> **tail call** whose body is, in that document's own words, *"no frame, no
> saved registers, nothing in the body but the moves"*, and every one handing
> its scratches out **`r11`, then `r10`, then `r9`** — the head of class 0's
> list at `0x10c37de0`. **213 obj cells, and nobody connected the family to
> F4.** The two documents are eleven directories apart and neither cites the
> other.
>
> This is the fourth time in this repo that a "no cell exists" claim was a
> statement about the **instrument's index** rather than about the corpus
> (cf. `#1823`, where *"there is no `sched.c` in the TU table"* was a true
> statement about `c2_tus.tsv` read as a statement about the image).

### 4.3 The new cell, and its four predictions — all four confirmed

`pd_perm8` was registered with its predictions in PREREG addendum 1 §A1.2
**before it was compiled**. σ = `(r3 r10)(r4 r9)(r5 r8)(r6 r7)`: four 2-cycles,
four local minima, eight formals occupying `r3`…`r10`, so exactly **one** free
volatile remains.

```
mflr 12 ; bl __savegprlr_29 ; stwu 1,-112(1)
mr 11, 7        <- scratch 1
mr 31, 8        <- scratch 2
mr 30, 9        <- scratch 3
mr 29, 10       <- scratch 4
mr 10,3 ; mr 9,4 ; mr 8,5 ; mr 7,6
mr 6,11 ; mr 5,31 ; mr 4,30 ; mr 3,29
bl gg8 ; addi 1,1,112 ; b __restgprlr_29
```

| # | prediction | emitted | verdict |
|---|---|---|---|
| **P-A** | four scratches | four | **HIT** — and it is the first cell to test `CODEGEN_ARG_PERM` §2's local-minimum rule at n = 8 |
| **P-B** | `r11, r31, r30, r29` — class 0's order with `r3`…`r10` removed; **not** `r14/r15/r16`, **not** `r12/r0/r13` | `r11, r31, r30, r29` | **HIT** |
| **P-C** | framed, **caused by an argument permutation**, with no call tuple ahead of it | `stwu` + `bl __savegprlr_29` | **HIT** |
| **P-D** | the scratches are **dead** at the call, so no clobber operand narrowed them | all four read at `mr 6,11 … mr 3,29`, **before** `bl gg8` | **HIT** |

`pd_perm6` — the positive control — reproduces `CODEGEN_ARG_PERM` §5.1's
`mr 11,r6 ; mr 10,r7 ; mr 9,r8 ; … ; b gg6` **verbatim, word for word**, so the
capture is the same one that grid was measured on and `pd_perm8` is admissible.

**What P-B establishes on its own**, and it is a distinct `[R]` → `[O]`: the
permutation scratch is **an allocator candidate drawn from `0x10c37de0`, not a
hardwired emit-time temp.** PREREG §2.4's hardwired test decides it — the same
role, in the same shape, moves from `r11,r10,r9` at n = 6 to `r11,r31,r30,r29`
at n = 8. A hardwired temp does not move; a candidate walking a list does.
`CODEGEN_FRAMED_CALLS.md` §3.2's *"a permutation is broken with **r11** as the
scratch"* is thereby explained rather than merely extended: `r11` is not
special, it is **entry 0 of the class list**.

And `pd_perm8` is the first body in this project where a **callee-saved
register is taken with no call clobber in its range** — `wbl_v3` did it by
pressure from other candidates, this one does it because the argument registers
were physically defined.

### 4.4 (c) is NOT established, and the reason is structural

Registered in addendum 1 §A1.3 before the cell was compiled, and it held:

> On every shape this front end can express, a register made unavailable by a
> bare physical def is **simultaneously held by a live candidate**, so pressure
> and narrowing predict the same obj.

In `pd_perm8`, `r3`…`r10` are unavailable to the scratches — but they are also
occupied by the eight formals. The two mechanisms are not separated by any cell
this lane could build, and the enumeration of the front end's physical-def
sources (formal arrival / call-sequence argument setup / return-value
materialisation) says why: the first two *are* live candidates by construction,
and the third has nothing live across it (`pd_ret2` emits `mr 3,10` as its last
instruction — condition (b) fails, as predicted).

**So the correct entry for `P_REGALLOC` §7 is not "no cell exists". It is:**

> **(a) and (b) `[O]` on 216 cells** (213 pre-existing `--pure` argperm cells +
> `pd_tail`, `pd_perm6`, `pd_perm8`); **(c) `[R]`, and unreachable by
> construction on this front end.**

### 4.5 The two negative controls, and what they were worth

| control | emitted | worth |
|---|---|---|
| `pd_ctr` — a dense `switch` | `mtctr 3` + a `bdzf` chain (no jump table), with 9 live values in `pd_ctr_p` | `ctr` is register 84, in **no** class list; it displaced nothing. **The control could have failed** — a class-2/3/4 list would have shown here — and did not |
| `pd_lr` — the `mflr r12` shuttle | `r12` used only as the LR shuttle, with 9 values live across it | `r12` is register 13, **absent from `0x10c37de0`'s 27 entries**; it displaced nothing |

Both are the same point from two sides and it is a real one for a port: **a
physical def only narrows if the register is in the class's list.** A port
clearing bits for `ctr`, `lr` or `r12` is doing arithmetic on bits no candidate
has.

### 4.6 The price consequence, two-sided as the rule requires

[`WB_ITEMF_FINDINGS.md`](WB_ITEMF_FINDINGS.md) §6.1 prices **F4 at 2 lanes**, of
which one is *"1 grid lane to obtain the first obj cell for the non-call
physical def, which is item F's own flagship mechanism and today has none"*.

* That line item is **spent by this lane**, and it bought (a)+(b), not (c). It
  was also **already paid** — 213 cells existed. **F4's remaining price is 1.**
* F4's proposed fail-closed boundary — *"admit only bodies whose sole clobber
  sources are call tuples; refuse on any bare physical def"* — was priced as
  free. **It is not.** Every permuted call has bare physical defs of its
  argument registers, and the port **already emits that class**
  (`crates/c2-core/src/codegen/calls.rs`, whose whole subject is the
  permutation lowering, with 213 obj cells behind it). That fence would
  withdraw a class the port has today. **Priced two-sided, it does not ship.**

---

## 5. PREREG score — reported by tier, and NOT pooled

| tier | hits | misses | notes |
|---|---:|---:|---|
| **PREREG** (`2c89de6a4`) | **7** | **2** | Q1's six cell-family predictions + FR0's survival; misses §1.2 and §2.2 |
| **IN-FLIGHT** (`9d0e23b2d`) | **5** | 0 | P-A…P-D + §A1.3's structural ceiling |

### 5.1 MISS 1 — §1.2, "the register SET is identical at both modes"

False. The sets differ on 4 of 10 cells. The **rule** is profile-invariant and
the **set** is not, and the prediction as written names the set. Scored a miss;
§2.4 states what is actually true.

### 5.2 MISS 2 — §2.2, the negative prediction for Q2, and this is the useful one

§2.2 predicted *"NOT BUILDABLE — no cell will satisfy (a)∧(b)∧(c), and
`P_REGALLOC` §7's 'still no obj cell in existence' will still be true at the
end of this lane."* **(a) and (b) fell on the very first compile.**

The reasoning error, named precisely because it is repeatable: §2.2's table
said a candidate live across call-sequence argument setup *"is live across the
`bl` too, so the bare-def narrowing is subsumed"*. **That is false for a tail
call, where there is no `bl` at all, and false for any value that dies AT the
call rather than across it.** The enumeration of the three physical-def sources
was right; the claim that each was unobservable was wrong on one of them, and
the failure mode was reasoning about *"a call"* as if the call **tuple** and the
**`bl` instruction** were the same object. They are not: the prologue's
`bl __savegprlr_29` is an instruction with no tuple, and a tail call is a tuple
with no `bl`.

**A lane whose negative prediction survives learns nothing; this one was
falsified in four instructions and found 213 unrecognised cells doing it.**

---

## 6. What a port takes from this

1. **One list generator, not two tables** (§1.1) — and the same shape to check
   class 5 against.
2. **`r11` is not special.** It is entry 0 of class 0's list, and `f0` is entry
   0 of class 1's. `CODEGEN_FRAMED_CALLS.md` §3.2 and `CODEGEN_FP_ARGS.md` §1.1
   both state the scratch as a constant; both are consequences of the list.
3. **The permutation scratch is an allocated candidate**, so it walks into the
   callee-saved tail under pressure and **frames the function** (§4.3). A port
   that hardcodes `r11`/`r10`/`r9` is right at n ≤ 6 and wrong at n = 8.
4. **Only narrow on registers that are in the class's list** (§4.5).
5. **Do not implement F4's proposed bare-physical-def refusal** (§4.6).

---

## 7. Artefacts

| what | where |
|---|---|
| prereg (PREREG + IN-FLIGHT addendum) | `work/w-regcells/PREREG.md` |
| the FPR grid | [`grids/w-regcells/fpr_grid.cpp`](grids/w-regcells/fpr_grid.cpp) |
| the physical-def battery | [`grids/w-regcells/physdef_grid.cpp`](grids/w-regcells/physdef_grid.cpp) |
| the grader (+ `--selftest`) | [`scripts/grade_fpr_cells.py`](scripts/grade_fpr_cells.py) |
| grade transcript | `work/w-regcells/run/grade_fpr.txt` |
| obj dumps | `work/w-regcells/run/*.txt` — **the objs themselves are not committed** |
