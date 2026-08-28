# PREREG — `w-globobj`, wave 18 lane L3

    Lane:     w-globobj (characterization)
    Branch:   wt-w-globobj, based at master 4b79bf46a
    Owns:     docs/whitebox/ref/P_GLOBREGS.md, docs/whitebox/WB_GLOBOBJ_*,
              docs/whitebox/grids/w-globobj/, docs/whitebox/scripts/grade_globobj.py,
              work/w-globobj/, board #3774-#3779, docs/rungs/2026-08-28-w-globobj.md
    Charter:  docs/ADOPTION_BRIEF_2026-08-28.md §L3, decision 22 §2
    Reach:    0 — predicted and required. `git diff master..HEAD -- crates/` empty at tip.

**Committed before any deciding cell is compiled and before the image is
opened by this lane's grader.** Registering the ceiling *before* the deciding
cell exists is what made `w-regcells`'s negative result credible
(`docs/rungs/2026-08-27-w-regcells.md` §1), and it is the only thing that stops
a post-hoc reading of an obj from being presented as a prediction.

---

## 0. FULL DISCLOSURE — what was compiled BEFORE this file was committed

One body, to prove the capture path is alive in this worktree. It is **not** a
graded cell and **enters no numerator and no denominator** anywhere in this
lane:

```
extern "C" int sink(int); extern "C" void u(int);
extern "C" int f(int*p){int x=p[0];int t=sink(7);u(x);return t;}
```

`/O1 /GS- /c` → `lwz 31, 0(11)` … `mr 3, 31` … `bl u`. The toolchain is
present and the readout below is mechanically extractable. Recorded here so it
cannot later be presented as a preregistered hit.

Nothing else has been compiled. `c2.dll` has not been opened by this lane.

---

## 1. THE QUESTION, in the brief's words

`P_GLOBREGS.md` carries **46 `[R]`, 2 `[O]`, 0 `[I]`** marks (re-derived on
this tree, `c2rs subsys | grep globregs` → `globregs-marks-obj 2`,
`globregs-marks-total 48`, 4.2 %). Convert `[R]` → `[O]` where a real obj can
decide, and — the part the brief says matters more than the count — **for every
claim not converted, say whether it is unobservable in an obj BY CONSTRUCTION
or merely UNCOMPILED.**

---

## 2. THE STANDING CLASSIFICATION, registered in advance

Made **before** any cell is compiled, so a cell that surprises me cannot be
retro-fitted into the class that flatters the outcome. Every `[R]` mark on the
page is assigned to exactly one of three buckets. The full assignment is
`work/w-globobj/MARKS.tsv`, committed with this file.

* **`OBS`** — a real obj can decide it. Cells below.
* **`CONSTR`** — **unobservable in an obj by construction.** The claim is about
  an internal representation whose only path to the output is through an order
  or a policy that some *other* mark already carries. Two obj bodies that
  differ only in this fact cannot exist, so no cell can exist. Examples I am
  registering now: `aux+0x10` (candidate-list *prev* pointer), `sym+0x30`
  (*next*), `0x10bd2343`'s 32-slots-per-chunk and stride `0x60`, the arena
  `memset`, `DAT_10c400d0`'s existence as an array, the *identity* of the
  reject-tail diagnostic counter `DAT_10c2e454`.
* **`UNCOMP`** — **merely uncompiled.** A cell could exist; this lane does not
  build it, or builds it and it does not reach. Named individually, with the
  cell that would decide it, so a later lane does not re-derive the question.

**The rule that keeps this honest, registered now:** *`CONSTR` is a claim about
the corpus, not about my index.* Board `#3505` and the memory note
*"ranking instruments measure themselves"* are five for five; `w-regcells`
found 213 cells for a claim that said none existed, eleven directories away.
**Before any mark is filed `CONSTR` I must be able to state the two obj bodies
that would have to differ, and why they cannot.** If I cannot, it is `UNCOMP`.

---

## 3. THE READOUT, and why it is not the one R4 used

R4's `scripts/globregs_c2.py` reads the **formal → callee-saved register** map
out of the prologue's `mr rT, rARG` moves. That readout has a confound this
lane cannot afford: a formal's arrival register is itself a declaration-side
property, so *every* declaration-order rival is entangled with the ABI.

This lane reads **locals defined by an indexed load from one pointer formal**:

```
extern "C" int f(int *p) { int x, y; x = p[0]; y = p[1]; ... }
```

`p[0]` and `p[1]` are distinguishable in the obj by their **displacement**, and
the destination register of each `lwz` is the local's colour. The pointer's
base register is tracked from `r3` through any `mr` (the smoke body already
shows c2 moving `r3` into `r11`).

**The register order is NOT typed into the grader.** It is decoded from the
per-class ordered-list array at `0x10c385c4` through the register-name table at
`0x10b181c0` — the same read `w-regcells`'s `grade_fpr_cells.py` already
performs for class 1. Class 0 is expected to be `r11…r3, r31…r14`
(`scripts/globregs_c2.py`'s header, from selector `0x10b2e7f8`); a value live
across a call cannot take `r3…r11`, so the first-coloured candidate takes
**`r31`**, the second `r30`, and so on. **The grader re-derives that run from
the pinned image; if the decode disagrees with `r31, r30, …` the grid is
scored `U` and nothing is published.**

---

## 4. SERIES P — the promotion policy, `P_GLOBREGS` §3 (gates A and B)

**The claim under test.** §3 says which symbols become candidates is decided
entirely inside `FUN_10b550e5` by a structural gate A and a categorical type
gate B, with **no threshold constant anywhere**, and that gate B rejects type
classes `0x00, 0x12, 0x13, 0x18, 0x1d` and admits the other 25.

**The observable.** A promoted local lives in a callee-saved register across a
call. A rejected one is written to a frame slot and reloaded. Grade
`PROMOTED` / `MEMORY` from the obj: the value loaded from `p[k]` either
(a) lands in `r14…r31` (or `f14…f31`) and is used after the `bl` with no
`r1`-relative store of that register outside the prologue save area, or
(b) is stored to an `r1`-relative slot and reloaded.

**Cells and predictions — frozen.**

| cell | local type | prediction | why |
|---|---|---|---|
| `p_int` | `int` | **PROMOTED** | positive control; if this is `MEMORY` the instrument is dead and every verdict below is discarded |
| `p_uchar` | `unsigned char` | PROMOTED | |
| `p_short` | `short` | PROMOTED | |
| `p_ll` | `long long` | PROMOTED | 64-bit GPR |
| `p_ptr` | `int *` | PROMOTED | |
| `p_bool` | `bool` | PROMOTED | |
| `p_enum` | `enum E` | PROMOTED | |
| `p_float` | `float` | PROMOTED | into an FPR; §9 calls the FPR path *blind* |
| `p_double` | `double` | PROMOTED | |
| `p_struct2` | `struct { int a, b; }` | **MEMORY** | aggregate |
| `p_struct1` | `struct { int a; }` | **OPEN — no prediction** | registered as open on purpose: a one-word aggregate is exactly where "aggregate ⇒ reject" could break, and a lane that predicts both ways predicts nothing |
| `p_arr` | `int[4]` | MEMORY | |
| `p_union` | `union { int a; float b; }` | MEMORY | |
| `p_addr` | `int`, address escapes to an extern | **MEMORY** | gate A3's `sym+0x14 == 0` / `sym+0x07 & 0x40` |
| `p_vol` | `volatile int` | **MEMORY** | negative control |
| `p_static` | function-`static int` | MEMORY | not an auto; kind ≠ 3 |

**Predicted score: 15 of 16 called, 14 hits.** I expect to be wrong somewhere;
`w-regcells` scored 7/2 and 5/0 and said so by tier.

**What series P does and does not convert.** It converts the **consequence** of
gates A and B at source-construct granularity. It does **not** convert the
gate's *internal order* (`0x10b5511a` before `0x10b55129` before …): two gates
that both reject produce the same obj, so **the order of the rejection tests is
`CONSTR`** — I am registering that now, with the two bodies named: there is no
pair of source constructs that is rejected by gate A in one and gate B in the
other *and* distinguishable in the obj, because both rejections produce the
identical `sym+0x34 = 0` and the identical stack slot.

**Answer-key re-derivation.** The grader reads the 30-byte table at
`0x10b18b28` out of the pinned image and prints the non-promotable class set it
finds. §3's `{0x00, 0x12, 0x13, 0x18, 0x1d}` is currently *typed into the
page*; after this lane it is *decoded by an instrument*. **That is a
re-derivation, not an `[O]`** — the mark stays `[R]` unless an obj cell decides
it, and I register now that I will not upgrade it on the strength of a second
read.

---

## 5. SERIES O — the order, `P_GLOBREGS` §6.3 and §7.1

### 5.1 The sharp entailment this lane derives from §7.1, and registers as a prediction

§7.1 reads the step-4 walk as **blocks FORWARD × tuples BACKWARD**, counter not
reset per block, `cand+0x44` overwritten at every encounter — so the surviving
value is the ordinal of the candidate's **last visit in that walk**. Composing
those two directions:

> In a **single-block straight-line body**, the walk visits the block's tuples
> from last to first, so a candidate's *last visit* is its **earliest tuple in
> program order** — which for a local is its **definition**. `cand+0x44` is
> therefore the ordinal of the **defining tuple**, larger for an earlier
> definition, and the comparator sorts `+0x44` **DESC** — so **the
> earliest-defined candidate is coloured first.**

**Nothing on the page states this.** It is my composition of §7.1 and
`P_REGALLOC` §4's comparator, and it is the reason the observable has looked
invariant so far: for `int x = p[0]; int y = p[1];` the definition order *is*
the declaration order, so the ordinal reading and plain arena order predict the
same map. **The separator is a body in which definition order and declaration
order disagree**, which costs one line of C and which no prior lane built.

### 5.2 The cells

Declaration order is set by `int x, y;` (or `int y, x;`); definition order by
the order of the two assignments; use order by the order of the two `u()`
calls. All three axes are independent.

```
extern "C" int sink(int); extern "C" void u(int);
extern "C" int f(int *p) {
    int <DECL>;            /* x,y   or   y,x   */
    <DEF1>; <DEF2>;        /* x=p[0]; y=p[1];  or  y=p[1]; x=p[0];  */
    int t = sink(7);
    <USE1>; <USE2>;        /* u(x); u(y);  or  u(y); u(x);          */
    return t;
}
```

**8 cells at N=2** (2 decl × 2 def × 2 use) and **6 cells at N=3** (declaration
`x,y,z` fixed, all 6 definition orders, uses fixed `u(x);u(y);u(z);`), each at
**`/O1` and `/Ox`** — `P_REGALLOC` §5 measured the two profiles disagreeing on
6 of 20 cells, so a rule taken at one profile is the wrong rule. **28 objs.**

### 5.3 The rivals, and the prediction

Which candidate is coloured **first** (takes `r31`):

| rival | rule | prediction on `decl x,y / def y,x / use x,y` |
|---|---|---|
| **DEF** (this lane's, from §7.1) | earliest **definition** | `y → r31` |
| **DECL** | earliest **declaration** (§6.3's arena order, if arena order tracks declaration) | `x → r31` |
| **USE** | earliest **first use** | `x → r31` |
| **LASTUSE** | earliest **last use** | `x → r31` |
| **REV-DEF** | latest definition | `x → r31` |
| **REV-DECL** | latest declaration | `y → r31` |

**Registered prediction: DEF survives every cell; DECL, USE, LASTUSE and
REV-DEF are each refuted by at least 2 of the 14 cells per mode.** A rival is
refuted by any graded cell it gets wrong, and the **count of refuting cells is
reported**, because a rival refuted by one cell is weaker evidence than one
refuted by nine (`w-regcells` §1).

### 5.4 THE CEILING, registered BEFORE the deciding cell is compiled

**This grid cannot separate `cand+0x44` from `cand+0x0c`, and I am saying so
now rather than after the numbers land.**

`P_REGALLOC.md`:71 reads the priority accumulator as
`cand[0x0c] += cand[0x18] * n_live`, `-= n_live` when not live — a function of
the candidate's **live interval**. Moving a definition earlier both raises its
`+0x44` **and** lengthens its live interval, so `+0x0c` moves with it. **Any
cell that moves the ordinal moves the priority**, and the comparator consults
`+0x0c` first.

Therefore:

* **If the map follows DEF**, the verdict is `[O]` on *"the earliest-defined
  candidate is coloured first"* — a **source-observable ordering rule**, which
  is the useful thing — and it is `[R]` **still** on *which of the two keys
  produced it*. I will not write `[O]` against `+0x44` on this evidence.
* **If the map does NOT follow DEF**, §7.1's ordinal reading is **refuted at
  the observable**, and that is the more valuable outcome.

I am registering the honest form of the residue now: **separating `+0x44` from
`+0x0c` in an obj requires a body in which two candidates have identical live
intervals and different definition ordinals. I do not currently know that such
a body exists, and "I could not build one" is `UNCOMP`, not `CONSTR`,** unless
I can state the impossibility — which §2's rule forbids me from asserting
without it.

### 5.5 The candidate **id** — registered `CONSTR` in advance, with the bodies named

§6.3's *"`id` ascends with (`sym+0x1c` ASC, version DESC)"* is about
`cand+0x1c`, minted at `0x10b54d32`. `P_REGALLOC` §4's revision box states the
entailment: the id-keyed hash-bucket walk is the **third** tier, reached only
when two candidates tie on `+0x0c` **and** on `+0x44`. Two bodies differing
only in mint order but agreeing on both keys would have to have identical live
intervals *and* identical definition ordinals for the two candidates — i.e. be
the same body. **`CONSTR`.** If series O produces an exact-tie cell, I withdraw
this and re-file it `UNCOMP`.

---

## 6. SERIES V — "a symbol with *k* versions mints *k* candidates" (§1 step 3, §6.2)

**The claim.** Step 3 mints one candidate per *version record*, not per symbol,
so one source variable with two disjoint definitions is **two candidates**, each
with its own colour and its own `+0x44`. §8 consequence 3 leans on this
(*"a variable is not a candidate"*) to explain the ten refuted allocation keys.

**The observable, and why it needs a second cell to mean anything.** "The two
ranges got different registers" alone is weak — any allocator may reuse a
register. The cell that carries information is the **pair**:

```
v_reuse    : x = p[0]; sink(); u(x); x = p[1]; sink(); u(x);
v_distinct : x = p[0]; sink(); u(x); z = p[1]; sink(); u(z);
v_single   : x = p[0]; sink(); sink(); u(x);            /* control */
```

**Registered prediction:** `v_reuse` and `v_distinct` produce the **same**
register map — a redefined variable is behaviourally indistinguishable from two
distinct variables. `v_single` uses **one** register. If `v_reuse` instead pins
both ranges to one register while `v_distinct` uses two, the *k*-versions claim
is refuted at the observable and §8 consequence 3 loses its support.

---

## 7. CONTROLS — watched RED before any verdict is quoted (`#3336`)

A control never watched fail is decoration. Four, and each has a named way to
go red:

1. **`p_int` PROMOTED.** If an `int` local live across a call is not in a
   callee-saved register, the readout is wrong and everything is discarded.
2. **`p_vol` MEMORY.** If a `volatile` local is enregistered, the
   promotion readout cannot tell promotion from anything else.
3. **The positive control on the order grid**: two cells differing in one
   operand must produce **different** register maps somewhere in the grid. If
   all 14 cells give the identical map at both modes, the grid is dead and no
   rival is refuted by it — it would mean the observable does not move, not
   that DECL won. **This is the failure mode that killed R4's G-block cell and
   it must be reported as `DEAD`, not as a DECL victory.**
4. **The grader's own `--selftest`**, which must contain assertions the grader
   has to **REJECT**: a dump with no load from the pointer, a dump whose
   register run disagrees with the image-decoded class-0 list, and a
   deliberately shifted displacement. Watched red by planting each defect.

**Premise test.** A cell whose two locals are not both resolved to distinct
callee-saved registers scores **`U`** and enters **no numerator and no
denominator**. `w-regcells` reported `0 cells scored U`; I will report the
count whatever it is, because a count resting on an absence is this repo's most
repeated defect.

---

## 8. WHAT THIS LANE WILL NOT DO

* **Builds no register allocator and no candidate-set implementation.**
  Decision 20 §2 / brief §3: F5 is not separable from F0.
* **Adopts nothing into `crates/`.** `git diff master..HEAD -- crates/` empty
  at the tip, and no `DISCLOSURE.md` row is owed because nothing is adopted.
* **Adds no `scripts/gate.sh` row** (`#3691`). The grader lives under
  `docs/whitebox/scripts/` beside `grade_fpr_cells.py`, which is where
  instruments that grade **real c2's obj** live; `#1406` binds instruments that
  grade **the port**.
* **Invents no `ported` numerator for globregs.** Decision 21 §4, `#3505`.
* **Grids go in `docs/whitebox/grids/w-globobj/`, not `fixtures/cpp/`** — a
  fixture would move the census. `w-regcells` made the same choice.
* **Commits no `.obj`.**

## 9. WHAT WOULD MAKE THIS LANE `FAILED`

Producing no conversion **and** no classification — i.e. neither an `[O]` with
a named witness nor a defensible `CONSTR`/`UNCOMP` assignment for the marks it
touched. A grid that comes back `DEAD` (control 3) with an honest report is
`instrument`, not `FAILED`; a grid whose controls were never watched is
`FAILED` whatever it printed.
