# WB_GLOBOBJ — `P_GLOBREGS` meets an obj: what converted, and — the harder half — why the rest cannot

Lane **`w-globobj`**, wave 18 L3 (`docs/ADOPTION_BRIEF_2026-08-28.md` §L3,
decision 22 §2). Prereg `work/w-globobj/PREREG.md` and
`work/w-globobj/PREREG_ADDENDUM.md`. Grids
`docs/whitebox/grids/w-globobj/`. Instrument
`docs/whitebox/scripts/grade_globobj.py`. Transcripts
`work/w-globobj/GRADE.txt`, `work/w-globobj/CONTROLS_RED.txt`. Board
**#3774**–**#3779**.

**Reach 0, as predicted.** `git diff master..HEAD -- crates/` is empty at the
tip. No `scripts/gate.sh` row (`#3691`). No `ported` numerator for globregs
(decision 21 §4, `#3505`).

---

## 0. THE HEADLINE — the separator §7.1 says was never built costs one line of C, and the thing it separates is not what the page thought

> **`P_GLOBREGS.md` §7.1, before this lane:** *"the separator remains
> unbuilt … the cell built to take it to `[O]` never reached the tier."*

It is built. **48 graded cells, 0 scored `U`, at both `/O1` and `/Ox`.**

The reason every previous grid missed it is a single structural fact: **every
register-order cell this repo had ever compiled welded the *declaration* order
and the *definition* order together.** `wb-live`'s ten, `wb-regalloc`'s
fifteen, `w-dagorder`'s twenty and R4's own `scripts/globregs_c2.py` all write
`int x = …; int y = …;` or use formals, in which the two orders are the same
sequence by construction. Split them — `int x, y;` then `y = p[1]; x = p[0];` —
and the observable moves.

**The absence was SEARCHED FOR, not asserted.** The brief's standing rule —
*"before you assert that no cell exists, search for one"*, five for five on
board `#3505` — was applied. `grep -rlE '^\s*int [a-z][a-z0-9]*, *[a-z][a-z0-9]* *;'`
over `docs/whitebox/grids/`, `scripts/*.py` and `fixtures/cpp/` returns exactly
**three** hits outside this lane — `il_this_straightline.cpp:75`,
`wfr_argreg_types.cpp:26`, `wsl_store_load.cpp:126` — and **all three are
struct MEMBER declarations, not locals**, so none of them separates the two
orders. `wb-live`'s cells take their values as **formals**; `wb-regalloc`'s use
`int s = …; int i = …;`; `wb-dagorder`'s grid contains the pattern zero times.
**No cell anywhere in this repo, grid or fixture, separated declaration order
from definition order for a local before this lane.**

**The rule that survives: the earliest-DEFINED candidate is coloured first.**
`[O]`, 46 of 48 cells, and **exact on all 42 straight-line cells at both
optimisation profiles**. Seven rival orders refuted by cell count.

| rival | refuted by | of |
|---|---:|---:|
| **`DEF`** — earliest **definition** | **2** | 48 |
| `USECOUNT` — most uses first | 6 | 48 |
| `LIVELEN` — longest live range first | 14 | 48 |
| `DECL` — earliest **declaration** (the arena-order proxy) | 24 | 48 |
| `USE` — earliest first use | 24 | 48 |
| `LASTUSE` — earliest last use | 24 | 48 |
| `REVDECL` | 32 | 48 |
| `REVDEF` | 46 | 48 |

**And the second headline is a correction to `P_GLOBREGS` §3, filed against
this lane's own prereg as a MISS.** *"Aggregates are rejected"* is **dead**:
`int[2]`, `int[4]`, `int[8]`, `int[12]`, a one-int `struct`, a four-int
`struct` and a `union` are **all enregistered**, member by member, and the one
aggregate that is not — `S2 v = *p;` — is **MEMORY for a front-end reason, not
a gate-A/gate-B reason.** The same type assigned member-wise is PROMOTED. The
witness is `pa_struct2mem` against `pa_struct2cpy`, in the same TU, same type,
same profile.

---

## 1. THE CONVERSIONS, each with its named witness

### 1.1 `[O]` — the earliest-defined candidate is coloured first

**Witness: `docs/whitebox/grids/w-globobj/order_grid.cpp`, 14 cells × 2
profiles, 0 `U`; extended by `order_lr_grid.cpp` (7 × 2) and
`order_loop_grid.cpp` (3 × 2).** Graded by
`grade_globobj.py --order`, whose colouring run `r31, r30, … r14` is **decoded
from the pinned image** (`0x10c385c4[0]` through the name table at
`0x10b181c0`) rather than typed in.

The three axes are independent by construction:

```
declaration   int x, y;          vs   int y, x;
definition    x=p[0]; y=p[1];    vs   y=p[1]; x=p[0];
use           u_i(x); u_i(y);    vs   u_i(y); u_i(x);
```

The readout is the destination register of each `lwz rD, disp(rB)` off the
pointer formal, so `p[0]` and `p[1]` are told apart by **displacement**. This
deliberately avoids R4's formal→register readout: a formal's arrival register
is itself a declaration-side property and confounds every declaration-order
rival.

**Data, mode W (`/O1`, the workload profile); mode X (`/Ox`) is identical on
all 14:**

| cell | decl | def | use | observed | order |
|---|---|---|---|---|---|
| `oc2_xy_xy_xy` | x,y | x,y | x,y | `x→r31 y→r30` | x<y |
| `oc2_xy_xy_yx` | x,y | x,y | **y,x** | `x→r31 y→r30` | x<y |
| `oc2_xy_yx_xy` | x,y | **y,x** | x,y | `y→r31 x→r30` | **y<x** |
| `oc2_yx_xy_xy` | **y,x** | x,y | x,y | `x→r31 y→r30` | x<y |
| `oc2_yx_yx_xy` | **y,x** | **y,x** | x,y | `y→r31 x→r30` | **y<x** |

**Declaration order moves the map by zero cells. Use order moves it by zero
cells. Definition order moves it every time.**

All six N=3 definition permutations produce **six distinct maps** — that is the
grid's positive control, and it fires: `oc3_zyx` gives `z→r31 y→r30 x→r29`,
the exact reversal of `oc3_xyz`.

**`pa_arr12` is the widest witness**: twelve `int` locals assigned from `p[0]`
… `p[11]` land in `r31, r30, r29, r28, r27, r26, r25, r24, r23, r22, r21,
r20` — twelve consecutive entries of the image-decoded run, in definition
order, with **no frame traffic at all**.

### 1.2 `[O]` — a variable is not a candidate: one symbol, two colours, and byte-identical code

**Witness: `version_grid.cpp`, `vc_three` vs `vc_three_distinct`.**

`vc_three` redefines one local `x` three times with disjoint live ranges. Its
three values land in **`r31`, then `r28`, then `r28`** — *one source symbol
holding two different colours*, which a one-candidate-per-symbol model cannot
produce, because a candidate has one colour by definition.

And the sharper half: **`vc_three` and `vc_three_distinct` — three genuinely
distinct locals in the identical shape — have BYTE-IDENTICAL `.text`, 26 words,
at both profiles.** So does the pair `vc_reuse` / `vc_distinct`, 19 words, both
profiles. **The allocator cannot tell a redefined variable from distinct
variables.**

That is `§1` step 3 and `§8` consequence 3 — *"a variable is not a candidate …
every one of the ten keys is one-candidate-per-variable by construction and is
therefore wrong in kind"* — confirmed at the obj.

### 1.3 `[O]` — the promotion policy, at source-construct granularity

**Witness: `promote_grid.cpp` (16 cells) + `promote2_grid.cpp` (7 cells), both
profiles, 46 verdicts, identical at `/O1` and `/Ox`.** Readout: the
frame-traffic rule — a promoted local needs no stack slot; the prologue's own
`stw r12,-8(r1)` / `std r31,-16(r1)` saves sit **before** the `stwu` and are
excluded by construction rather than by a heuristic.

| PROMOTED | MEMORY |
|---|---|
| `int`, `unsigned char`, `short`, `long long`, `int*`, `bool`, `enum`, `float`, `double` | `volatile int` |
| `struct{int a;}`, `struct{int a,b;}` **member-wise**, `struct{int a,b,c,d;}` member-wise | `struct{int a,b;}` **whole-object copy** (`S2 v = *p;`) |
| `union{int a; float b;}` | `int` whose **address escapes** |
| `int[2]`, `int[4]`, `int[8]`, `int[12]`† | function-`static int` |

† `pa_arr12` was registered in addendum 1 as **data, not a graded cell**,
because the frame-traffic readout cannot separate *"never promoted"* from
*"promoted then spilled"*. It is quoted here anyway because it did **not** need
the readout: the twelve values are visibly in `r31 … r20` and there is no frame
traffic at all, which is stronger evidence than the readout was designed to
give. **No threshold claim is made above twelve in either direction.**

**Controls both fired**: `pc_int` PROMOTED (positive), `pc_vol` MEMORY
(negative). `pc_static` is MEMORY through the *relocated-static* arm of the
readout (`stw 11, 0(31) ; REFLO -> ?v@?1??pc_static@@9@4HA`), which is the arm
this lane added precisely because a static has no frame slot and would
otherwise have read as PROMOTED — a false positive the prereg would have
banked.

### 1.4 `[O]` — the merge at a join produces ONE colour and no reconciling copy

**Witness: `merge_grid.cpp`, `vm_merge3`, both profiles.** Three arms of an
if/else-if/else each write one symbol; the obj is

```
001c  lwz 31, 0(3)      /* arm 1 */
002c  lwz 31, 4(3)      /* arm 2 */
0034  lwz 31, 8(3)      /* arm 3 */
```

— three loads **straight into `r31`**, and **no copy at the join**. One
candidate spans the merge. §5's *"the merge is keyed on the symbol"* holds at
the observable; §5's *reuse-vs-fresh-mint* alternative is **not** separated by
this cell and is not claimed.

### 1.5 Re-derived, but NOT converted — gate B's class table

`grade_globobj.py` decodes the 30-byte table at `0x10b18b28` out of the pinned
image and independently reproduces §3's

> **not promotable: `0x00`, `0x12`, `0x13`, `0x18`, `0x1d`; the other 25 are.**

**This stays `[R]`.** A second read of the same bytes is a second read, not an
obj. The prereg registered in advance that this lane would not upgrade a mark
on the strength of a re-derivation, and it does not. What it buys is that the
constant is now *decoded by an instrument with a `--selftest`* rather than
typed into a page — a planted one-byte stride shift makes the assertion fail
(`CONTROLS_RED.txt`, defect 1).

---

## 2. REFUTATIONS — of the page, of the brief, and of this lane's own prereg

**A refutation is the most valuable thing a lane here produces.** Six.

### 2.1 ⛔ REFUTED — "aggregates are rejected", and it was this lane's own prediction

`PREREG.md` §4 predicted `int[4]`, `union`, and `struct{int a,b;}` **MEMORY**.
Arrays and unions came back **PROMOTED**, and the `OPEN` cell `struct{int a;}`
came back PROMOTED too. The addendum then named the confound and built the cell
that decides it:

| cell | source | verdict |
|---|---|---|
| `pa_struct2cpy` | `S2 v = *p;` | **MEMORY** — `ld 11, 0(3)` / `std 11, 80(1)` |
| `pa_struct2mem` | `S2 v; v.a = p->a; v.b = p->b;` | **PROMOTED** |

Same type, same TU, same profile. **`pc_struct2`'s MEMORY verdict is a
front-end whole-object-copy artifact and carries no information about
`FUN_10b550e5` at all.** This lane refuses to bank it as a gate-B
confirmation, which is what a lane that had stopped at `promote_grid.cpp`
would have done.

The reading that replaces it is §3's own, and it was already on the page and
unremarked: gate A's `0x10b55156`–`0x10b55173` indexes **sub-symbols**, not the
aggregate — *"only sub-symbols with `t+0x20 == 4` are indexed"*. An aggregate
is promoted **member by member** when the front end hands c2 member-wise
assignments, and is not when it hands c2 one wide copy.

### 2.2 ⛔ REFUTED — `DEF` is not universal, and the counterexample is a loop

`DEF` is exact on 42 straight-line cells and **fails on 2**: `ob_loop_y`, at
both profiles. There, `x` is defined first and `y` is used inside a `for` loop,
and **`y` is coloured before `x`** (`y→r30`, `x→r28`).

**So `cand+0x0c` is reachable and it responds to a loop-weighted use count.**
It is not merely a re-expression of definition position.

**And the anomaly this lane cannot explain, stated as an anomaly rather than
smoothed into a rule:** `ob_loop2_y`, the same shape with the use at loop depth
**2**, does **not** reproduce the refutation — `x` is coloured first at both
profiles. A deeper loop moving the key *less* than a shallower one is not a
behaviour this lane has a model for, and it does not have one. **`[R]` on the
loop weighting; the two cells that disagree are both committed.**

### 2.3 ⛔ REFUTED — `P_REGALLOC.md`:71's accumulator cannot be the whole story

`P_REGALLOC`:71 reads the priority accumulator as
`cand[0x0c] += cand[0x18] * n_live` where live, `-= n_live` where not.

* If `cand+0x18 > 0`, a longer-lived candidate accumulates strictly more →
  DESC → longer-lived coloured first → **`LIVELEN`**.
* If `cand+0x18 == 0`, `+0x0c` is `−Σ n_live` over the points where the
  candidate is **not** live; a longer-lived candidate is not-live at fewer
  points → larger `+0x0c` → DESC → **`LIVELEN` again.**

**Under either sign the formula predicts `LIVELEN`, and `LIVELEN` is refuted by
14 of 48 cells.** The `order_lr_grid.cpp` cells are the sharp ones:
`ol_dxy_ylate` defines `x` first and gives `y` a live range three calls longer,
and `x` is still coloured first. `[I]` — this is an inference from a read
formula against measured cells, not a read of new bytes, and it says the
formula as published is incomplete, not which part.

### 2.4 ⛔ REFUTED — `MARKS.tsv`'s own `UNCOMP` filing for the merge

This lane filed `FUN_10b54c07` (§5) as `UNCOMP` **and named the cell**. The
cell cost three lines. It is now `[O]` (§1.4). *`UNCOMP` is a statement about
what a lane did, never about what the corpus contains* — the standing rule from
`#3505` and `w-regcells`'s 213 cells, and this lane broke its own filing inside
one wave, which is the outcome that rule exists to produce.

### 2.5 ⛔ REFUTED — this lane's registered ceiling, in one direction only

`PREREG.md` §5.4 registered, before any cell existed, that this grid *cannot*
separate `cand+0x44` from `cand+0x0c` because moving a definition moves the
live interval too. **That is half wrong and the addendum said so before
compiling the cell that showed it:** holding the definition order fixed and
moving the **last use** changes the live interval and leaves the ordinal alone.
`order_lr_grid.cpp` does exactly that and refutes `LIVELEN` on 4 cells the
original ceiling said were unreachable.

**What is still NOT separated, and this half of the ceiling stands:** whether
the 42 straight-line cells are decided by `+0x44` or by a `+0x0c` that happens
to be definition-ordered. §2.2 narrows it — `+0x0c` demonstrably moves on
something that is not definition position — but "narrows" is not "closes", and
the cell that would close it (two candidates with identical `+0x0c` and
different definition ordinals, with the tie *witnessed* rather than assumed)
was **not built**. **`UNCOMP`, not `CONSTR`** — §3's rule forbids calling it
impossible without naming the two bodies, and this lane cannot.

### 2.6 ⛔ REFUTED — two defects in this lane's own instrument, both found by cells

**(a) The `LIVELEN` key ignored the padding.** The first `parse_grid` indexed
positions by `u_i` call order, so `order_lr_grid.cpp`'s `t = sink(t)` padding —
whose only job is to lengthen one live interval — was invisible, `LIVELEN` and
`DEF` computed identical predictions, and **the two cells built to separate
them scored as agreeing.** The grader would have published *"LIVELEN survives"*
off cells constructed to refute it. Positions are statement-indexed now and
`--selftest` carries an assertion that fails on the old version. `LIVELEN`'s
refutation count went **8 → 12 → 14**.

**(b) Absolute registers were the wrong graded quantity, and a "fix" for it was
tried and withdrawn.** The loop cells contain candidates the source model does
not enumerate (the induction variable, the second formal, the call's return),
so an absolute prediction of `r31, r30` is wrong by construction. A
single-definition premise test was tried; it rejected **8 straight-line cells
whose readout is sound**, because `r31` is reused by an unrelated candidate
*after* `x` dies — which does not bear on `x`-vs-`y` at all. **It was removed
rather than kept because it made the numbers look tidier.** What survives is
source-derived and checkable: every modelled local is defined before the `sink`
call and read after it, so all of them are simultaneously live across it and
pairwise interfere. Graded cells went 42 → 34 → **48, with 0 `U`**.

**A third, about the control experiment rather than the instrument:** two of
the five planted defects initially reported *green*, because the defect copies
lived in a scratch directory from which `find_dll()` could not resolve the
image, so the three image assertions **silently skipped**. The instrument
printed `(image absent — 3 image assertions skipped)` and was honest; the
`grep FAIL` reading it was not. **A control that skips is indistinguishable
from a control that passes if you only read the failures.**

---

## 3. THE CLASSIFICATION — unobservable by construction vs merely uncompiled

This is the deliverable the brief calls *"more valuable than the count"*.
`work/w-globobj/MARKS.tsv` assigns every one of the page's 48 marks, and it was
**committed before any deciding cell was compiled** (`7495010c6`), under a rule
that binds it:

> **`CONSTR` is a claim about the corpus, not about my index.** Before a mark
> is filed `CONSTR` I must be able to state the two obj bodies that would have
> to differ, and why they cannot. If I cannot, it is `UNCOMP`.

**Opening census: 28 `CONSTR`, 13 `OBS`, 5 `UNCOMP`, 2 already `[O]`.**
One `UNCOMP` (the merge, §2.4) was converted inside the wave, which is the
filing working as intended.

### 3.1 UNOBSERVABLE IN AN OBJ BY CONSTRUCTION — 28 of 48, and this is the real reason agreement was 4.2 %

**`P_GLOBREGS` is predominantly a STRUCTURE read, not a POLICY read**, and a
structure read has no obj form. That is not a defect of the page and it is not
a gap to be closed by more cells — it is what the page is. Three families:

**(a) Data-structure layout — 17 marks.** `sym+0x34` (the aux record),
`aux+0x00/0x0c/0x10/0x14`, `DAT_10c6f844`'s chunk chain, `DAT_10c400d0` as an
array, `0x10b54bf0`'s `sym+0x30`-next/`aux+0x10`-prev link,
`0x10bd2343`'s 32 slots at stride `0x60`, `0x10bd7cf0`'s 13-entry arity. **The
two bodies that would have to differ do not exist**: a linked list's *prev*
pointer, a chunk's slot count and a table's arity produce identical output for
every input. Nothing in the obj is a function of them except through an order,
and the order is §1.1's subject.

**(b) Site attribution — 6 marks.** *"`cand+0x44` is written at `0x10b55fac`,
and only there"*; *"`0x10b54d32` writes `+0x44` never"*; *"the destructor
`memset`s `0x48` and restores only `+0x1c`"*; *"the arena `memset`s every
chunk"*. **No obj can name the instruction that wrote a compiler-internal
field.** The behaviour these produce is the `+0x44` ordering, already
`[O]`-adjacent through §1.1; the *addresses* are `[R]` permanently and
correctly.

**(c) The candidate `id`, and everything that feeds only it — 5 marks.** §6.3's
*"`id` ascends with (`sym+0x1c` ASC, version DESC)"*, `sym+0x1c`'s preservation
across recycling, §6.4's recycling wrinkle, `0x10b54bad`'s head insert,
`aux+0x0c`'s descending order. `P_REGALLOC` §4's revision box states the
entailment: the id-keyed hash-bucket walk is the **third** tier, reached only
when two candidates tie on `+0x0c` **and** on `+0x44`. **Two bodies differing
only in mint order but agreeing on both keys would need identical live
intervals and identical definition ordinals for the two candidates — i.e. be
the same body.** Stated in the prereg §5.5 before any cell, and the 48 cells
did not produce a single exact tie to withdraw it on.

### 3.2 MERELY UNCOMPILED — 4 of 48, each with the cell that would decide it

Filed so a later lane does not re-derive the question:

| mark | the cell that would decide it |
|---|---|
| `0x10b2dfe2` / `0x10b2e4ae` — the split path copies the parent's `+0x44` verbatim | a body forcing a **spill and split**: ≥ 19 simultaneously live candidates, so the colouring runs out of `r14…r31`. `pa_arr12` reaches 12 with no spill, so the shape is known and the cell is one `pa_arr20` away |
| `aux+0x18` — the partner symbol versioned alongside at `0x10b55aa2` | unknown. §9 already calls it *read as a field, not as a rule*; no cell shape is known and **that is `UNCOMP`, not `CONSTR`** |
| §4 — a symbol is versioned on first encounter in **either** operand list | a cell separating `T->[0x28]` from `T->[0x2c]`. §9 records that which one is the def list is unknown, so the cell cannot be written until that is read |
| §5's **reuse-vs-fresh-mint** at a join | a join where one arm's version bitset already meets the phi set. `vm_merge3` shows one colour survives the join but does not distinguish which branch of §5 produced it |

**Plus one methodological `UNCOMP`, §2.5:** the cell that would attribute the
straight-line order to `+0x44` rather than `+0x0c`.

### 3.3 What this says about the `agreement` strength itself

`agreement` counts **page annotations, not sites**, and `subsys.rs`:809 says so
in the source. This lane's finding sharpens it: **for a page that is mostly a
structure read, a low `agreement` is a correct reading of the page's *kind*,
not a measure of how much of the subsystem is unverified.** `[globregs]` was
the weakest of the ten at 4.2 % because 28 of its 48 marks are about record
layout and write-site attribution, and **no amount of obj work will ever move
them.** A ceiling of roughly 20/48 ≈ 42 % is what this page can reach, and the
number to watch is the `OBS` bucket's fill rate, not the ratio.

---

## 4. THE PREREG SCORE, by tier, never pooled

| tier | commit | predictions | hits | misses | ungraded |
|---|---|---:|---:|---:|---:|
| **PREREG** — before any deciding cell | `7495010c6` | 21 | **18** | **2** | 1 (`pc_struct1`, registered OPEN) |
| **ADDENDUM 1** — in-flight | `b0af95af4` | 7 | **7** | 0 | 0 |
| **ADDENDUM 2** — in-flight | `c2226763c` | 2 | **1** | **1** | 0 |

**PREREG misses** — `pc_arr` and `pc_union`, both predicted MEMORY, both
PROMOTED. The reasoning error, named because it is repeatable: **it treated the
C++ type as the thing the gate sees.** It is not — c2 sees whatever the front
end lowered the declaration into, and `int[2]` assigned element-wise and
`S2 v = *p;` differ in the **front end**, not in `FUN_10b550e5`.

**And one HIT that this lane refuses to bank**: `pc_struct2` was predicted
MEMORY and *is* MEMORY, but §2.1 shows the prediction was right for a reason
that is not the reason given. A hit scored on the wrong mechanism is not
evidence for the mechanism, and pooling it with the other seventeen would
launder exactly that.

**ADDENDUM 2 is one hit and one miss on the same cell pair**, and the miss is
the useful one: `ob_loop_y` moved the order as predicted (`DEF` refuted,
`+0x0c` shown reachable), and `ob_loop2_y` — the deeper loop, predicted to move
it *more* — did not move it at all. The prediction as written names both.

---

## 5. CONTROLS — five planted defects, all watched RED

`#3336`: a control never watched fail is decoration. `work/w-globobj/CONTROLS_RED.txt`.

| # | planted defect | assertion that went red |
|---|---|---|
| 1 | gate-B table read at `+c*4+1` | *image: gate-B non-promotable set re-derives to §3's* |
| 2 | frame traffic scans the prologue saves too | *prologue saves BEFORE the stwu are not frame traffic* |
| 3 | distinct-register premise test removed | *REJECT two locals sharing a register* |
| 4 | colouring run sorted ascending | *image: callee-saved run is r31..r14 DESCENDING* |
| 5 | the `sink`-call index never recorded | *2 cells graded, 0 unscoreable* + both refutation counts |

`--selftest` carries **18 assertions, 5 of them the grader having to REJECT**:
a cell missing a load, two locals sharing a register, colours outside the
image-decoded run, loads off an untracked base, and a body with no frame. Plus
the assertion that pins §2.6(a) — a padded cell must make `LIVELEN` and `DEF`
disagree.

**Premise test: 0 of 48 order cells scored `U` in the final run.** No count in
this lane rests on an absence.

---

## 6. HANDOFFS

* **Anyone porting the candidate order** (not this wave — decision 20 §2): the
  rule to expose as a named settable parameter is **definition order**, `[O]`
  on 46 of 48, with **loop-weighted use count as a documented exception on 2**
  and the depth-2 anomaly unexplained. Do not ship the loop weighting; do ship
  the parameter.
* **`P_REGALLOC.md`'s owner**: §2.3 says :71's accumulator, as published,
  predicts an order refuted by 14 cells. That page is **not** owned by this
  lane this wave and was not edited; the correction is filed here and cited
  from `P_GLOBREGS` §7.1 only.
* **A lane wanting `+0x44` isolated from `+0x0c`**: build a body with two
  candidates whose `+0x0c` tie is **witnessed**, not assumed. §2.5 is `UNCOMP`
  and the shape is unknown; do not re-file it `CONSTR` without the two bodies.
* **A lane wanting the split path** (`0x10b2dfe2`, `0x10b2e4ae`): `pa_arr12`
  reaches 12 live candidates with **no spill at all**. `pa_arr20` is one line
  away and is the whole cell.
* **`w-secported`'s pattern, on the question this brief asked about**: a
  defensible site-level population for globregs **does exist in c2's own
  structure** — §3's gate A is a 12-arm decision over the symbol `kind` field
  (`0x10b5511a`…`0x10b551bc`), each arm named and addressed. This lane
  **reports that it exists and does not define a metric on it** (decision 21
  §4, `#3505`).
