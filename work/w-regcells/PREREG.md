# PREREG — lane `w-regcells` (L3 of `docs/REGALLOC_BRIEF_2026-08-27.md`)

> **Frozen before the first compile.** Nothing in `work/w-regcells/` other than
> this file exists at the commit that lands it, and no `cl.exe` has been run by
> this lane at that commit. Tier: **PREREG** (`docs/whitebox/PREREG.md`
> § "Registration status").

Lane kind: **characterization** (`docs/rungs/README.md` § "Lane kinds", kind 3).
`Fixtures: none`. `Census: +0`. **Predicted reach 0.**

The lane exists to move `P_REGALLOC`'s **agreement** strength (`[O]` 7 of 49
today) by converting the two things `P_REGALLOC.md` §7 names as read with **no
obj cell in existence anywhere in this project**:

* **Q1** — the FPR allocation order at `0x10c37f20`;
* **Q2** — F4's **non-call physical def**.

---

## 0. What has already been done at this commit, and what has not

**Done (a read, not a probe — `docs/WHITEBOX_LEVERAGE_2026-08-21.md`):** the two
tables were re-read out of the pinned image at the byte level by this lane, and
the byte read is recorded here *before* any obj exists, so that the obj grade
below is a grade and not a fit.

Image: `compilers/X360/16.00.11886.00/c2.dll`,
`sha256 c80981c015166effecc71ad8112d5577a065b2300891dfdb02b9c13787a66258` —
the image pinned in `C2_MAP_METHOD.md` §0, verified by this lane.
Image base `0x10b00000`; `.text` RVA `0x1000` file `0x400`; `.data` RVA
`0x12e000` file `0x12d200`.

| VA | read |
|---|---|
| `0x10b181c0` | the register-NAME pointer table; index **is** c2's register number. `0` `noreg`, `1` `r0`, `2` `sp`, `3` `toc`, `4..13` `r3..r12`, `14` `r13`, `15..32` `r14..r31`, `33` `d0`, `34..65` `fp0..fp31`, … |
| `0x10c385c4` | `[0x10c37de0, 0x10c37f20, 0, 0, 0, 0, 0, 0]` — **only classes 0 and 1 are image-initialised** |
| `0x10c37de0` | 27 entries, zero-terminated: `r11,r10,r9,r8,r7,r6,r5,r4,r3, r31,r30,…,r14` |
| **`0x10c37f20`** | **32 entries, zero-terminated: `fp0, fp13,fp12,fp11,fp10,fp9,fp8,fp7,fp6,fp5,fp4,fp3,fp2,fp1, fp31,fp30,fp29,…,fp15,fp14`** |

**Not done at this commit:** no fixture compiled, no obj produced, no
disassembly of any obj by this lane.

**The structural observation the FP predictions below are made from**, stated
here so it is a prediction and not a post-hoc pattern: the two lists are the
**same rule in two files** — *the class's scratch register first, then the
argument registers in DESCENDING order, then the non-volatiles in DESCENDING
order.* GPR: `r11` (scratch), `r10…r3` (the 8 arg registers, descending),
`r31…r14`. FPR: `fp0` (scratch), `fp13…fp1` (the 13 arg registers, descending),
`fp31…fp14`. If the obj refutes the FPR list, it refutes this homology too, and
the homology is the more interesting casualty.

---

## 1. Q1 — the FPR order. Rivals, frozen.

Grading follows `WB_REGALLOC_FINDINGS.md` §7.1: a cell is graded on the
**set of FPRs that hold values**, not on which value lands where (that is
candidate order, a different question, and this lane does not touch it).

| id | the rule | first four FPRs it predicts for a no-preference body | the tail after the 14 volatiles |
|---|---|---|---|
| **FR0** | the read: order `fp0, fp13…fp1, fp31…fp14` | `f0, f13, f12, f11` | `f31, f30, f29` |
| FR1 | ascending from `f1` (the naive "first free arg reg") | `f1, f2, f3, f4` | `f14, f15` |
| FR2 | ascending from `f0` | `f0, f1, f2, f3` | `f14, f15` |
| FR3 | descending from `f31` (non-volatiles first) | `f31, f30, f29, f28` | — |
| FR4 | the byte read is direction-confused: `fp0, fp1…fp13, fp14…fp31` | `f0, f1, f2, f3` | `f14, f15` |
| FR5 | the GPR list is reused for both classes (no separate FPR list) | undefined / class-0 names | — |

FR2 and FR4 are the same prediction on the head cells and differ from FR0 only
from the **second** register on; that is deliberate — the head cell alone cannot
separate them, and a lane that stopped at one cell would over-read. **The
separating cells are `fpc_g2`+ (second register) and `fpc_l3` (the volatile →
non-volatile transition).**

### 1.1 The cells, and the register set each predicts

Cells mirror `grids/wb-regalloc/regorder_grid.cpp`'s G-series shape for shape
comparability — values materialised from **globals**, so nothing arrives
pre-coloured and no copy preference biases the cost — with `double` in place of
`int`.

| cell | shape | FR0 predicts (the graded claim) |
|---|---|---|
| `fpc_g1` | `fpg0 + 1.0` | `f0` appears; **no** `f1` other than as the return register |
| `fpc_g2` | `(fpg0+1.0) * (fpg1+2.0)` | `{f0, f13}` ⊆ used; **`f2` absent** |
| `fpc_g3` | `(fpg0+1.0)*(fpg1+2.0) + fpg2*3.0` | `{f0, f13, f12}` ⊆ used |
| `fpc_g4` | `((fpg0+1.0)*(fpg1+2.0)) + ((fpg2+3.0)*(fpg3+4.0))` | `{f0, f13, f12, f11}` ⊆ used |
| **`fpc_l3`** | three globals live across `extern "C" void fpsink(void)` | **`f31` is taken, and `f30`/`f29` next**; **`f14` is NOT taken**; the body is framed and saves FP non-volatiles from the **top** |
| `fpc_p1` | 16 doubles from globals, tree-combined | the used set is a **prefix of FR0's list**: volatiles before non-volatiles, and the first non-volatile is `f31` |
| `fpc_a1` | `double f(double a, double b)` returning `a*b + fpg0` | `a`,`b` stay in their arrival registers `f1`,`f2` (the preference term); the loaded global takes `f0` |

`fpc_a1` is the FPR analogue of the N-series and of `WB_REGALLOC_FINDINGS.md`
§7.3's recorded miss: **the loaded global has no copy relation to anything, so
it takes the head of the list (`f0`), not the next free argument register.**
Registering that explicitly because it is the cell this lane is most likely to
mis-predict in the same way the earlier one did.

### 1.2 Profile

Every cell is compiled at **both**:

* mode **W** — the workload's own `/nologo /c /GR /O1 /Oi /EHsc`;
* mode **X** — `/nologo /Ox /GS- /c`, the fixture-capture profile.

**Prediction:** the FPR **register set** is identical at both modes on all
cells. `P_REGALLOC` §5 records that `/O1` and `/Ox` reverse the **candidate**
order on 6 of 20 cells; the register **list** is an image constant and should
not move. A per-mode difference in the *set* would be a finding against
`0x10c385c4` being the only list consulted, and is registered as such.

### 1.3 The premise test — how a real answer is told from a fixture that missed

**Absence is not evidence** (`CLAUDE.md`; ~15 recorded instances in this repo).
A cell is scored **`U` (unscoreable, premise unmet)** and contributes to no
numerator and no denominator of a rival's refutation if any of:

* the body contains **no floating-point instruction at all** (constant-folded,
  or the globals were read as integers);
* the body holds **fewer distinct FPRs than the cell's prediction names** — it
  did not reach the *k*-th entry of the list, so it says nothing about the *k*-th
  entry;
* the FP values are **spilled to memory** rather than held in registers;
* for `fpc_l3` only: the body contains **no call** in the emitted code (the
  callee was folded away), so nothing clobbered the volatiles.

Every published count carries its denominator in the form
*k of N cells, at mode M*, and `U` cells are listed by name.

### 1.4 What REFUTES the read

Any **one** graded (non-`U`) cell in which an FPR is used that FR0's prefix of
that length does not contain, or in which an FPR that FR0's prefix does contain
is skipped while a later one is taken. Specifically:

* `f1` or `f2` holding a **no-preference** value while `f13` is free → FR0 dead,
  FR2/FR4 alive.
* `f14` taken before `f31` in `fpc_l3` or `fpc_p1` → FR0's tail is wrong.
* any cell using `f13` **before** `f0` — this does **not** refute FR0 (`f0` and
  `f13` are adjacent entries and which value gets which is candidate order); it
  is recorded but not scored against FR0. Registered now so it cannot be
  claimed later as a confirmation either.

---

## 2. Q2 — F4's non-call physical def

### 2.1 The claim under test, stated exactly

`WB_LIVE_FINDINGS.md` §2.1 / §6.2, `[R]`, disassembly-only:

> `FUN_10b2d630` @ `0x10b2d630`: for every **physical-register** operand
> (symbol kind 1) it clears that register from `cand->allowed` for **every
> candidate currently on the live list** — and this happens for a **bare
> physical def**, not only for a call's kind-`0x0b` clobber-set operand.

An obj cell for it needs all three of:

1. **(a)** some allocatable GPR `X` ∈ {`r3`…`r11`, `r14`…`r31`} is physically
   defined by a **non-call** tuple;
2. **(b)** a candidate is live across that def;
3. **(c)** that candidate would otherwise have taken `X` — i.e. the narrowing
   is **observable** in the emitted registers.

### 2.2 The prediction, and it is a negative one

**PREDICTED: NOT BUILDABLE from C source on this target — no cell will satisfy
(a)∧(b)∧(c), and `P_REGALLOC` §7's "still no obj cell in existence" will still
be true at the end of this lane.**

The reason, frozen here so the probes below grade *it* and not a hunch: this
front end has exactly **three** sources of an allocatable-GPR physical def, and
each fails a different one of the three conditions.

| source | fails |
|---|---|
| **formal arrival** (`r3`…`r10` at entry) | **(c)** — every formal carries a copy preference to its own arrival register, so it keeps it; and the head of the order (`r11`) is never an arrival register, so no other candidate is displaced |
| **call-sequence argument setup** (`r3`…`r10` before a `bl`) | **(b)/(c)** — a candidate live across the setup is live across the `bl` too, and the call's kind-`0x0b` clobber removes **every** volatile at once, so the bare-def narrowing is **subsumed** and not separable. This is exactly `wbl_x4`, the cell that already exists |
| **return-value materialisation** (`r3`) | **(b)** — nothing is live past it |

and the machine's other hardwired registers on this target — `r0`, `r1`/`sp`,
`r2`/`toc`, **`r12`** (the `mflr` shuttle), `r13` (TLS), `lr`, `ctr`, `cr*` —
are **all outside class 0's allocatable list** (verified above: the list is
`r11…r3, r31…r14`, 27 entries), so a physical def of any of them clears a bit
that is not in any GPR candidate's set. **They narrow nothing observable.**

### 2.3 The probe battery

Six probes. Each is a **falsifier attempt** for §2.2, not a demonstration of it.

| probe | shape | what it would take to refute §2.2 |
|---|---|---|
| `pd_ctr` | dense `switch` → predicted `mtctr`/`bctr` | a GPR whose value is displaced by the `ctr` def |
| `pd_tail` | permuted **tail** call `return gg(b, a)` — argument setup with **no `bl`** | a live value forced off `r3`/`r4` with no `bl` in the body |
| `pd_argdie` | 10 live int globals, then a call taking 8; the 9th and 10th die **before** the `bl` | a value that avoids `r3`…`r10` while provably not live across the `bl` |
| `pd_ret2` | two `return`s, the second reached after the first's `r3` materialisation is dead | a value avoiding `r3` |
| `pd_mftb` | the `__mftb()` intrinsic, if the front end accepts it | any hardwired GPR in its expansion |
| `pd_asm` | `__asm { li r5, 3 }` | the front end accepting it at all |

`pd_mftb` and `pd_asm` are **front-end probes**: a compile error is a legitimate
result and is reported as one.

### 2.4 The hardwired-register test — telling (a) from an allocation

**From an obj alone, a physical def and an allocated register look identical.**
The discriminator, registered before use: compile each probe **twice** — bare,
and again with **nine extra simultaneously-live integer globals** added around
the shape. A register is called **hardwired** only if it appears in the same
role in the shape's own instructions in **both** compiles while the surrounding
allocation demonstrably moved. If the added pressure does not move the
surrounding allocation, the probe's second compile is `U` and the probe decides
nothing.

### 2.5 What REFUTES the negative prediction

**One** probe cell exhibiting an allocatable GPR that is (i) hardwired by
§2.4's test, (ii) defined in a body with **no `bl` and no `b` to an external
symbol**, and (iii) demonstrably displacing a live value. That result would be
strictly better than the predicted one — it would convert F4's flagship
mechanism from `[R]` to `[O]` — and it will be reported first if it happens.

### 2.6 The consequence either way, priced two-sided

`WB_ITEMF_FINDINGS.md` §6.1 prices **F4 at 2 lanes**, of which **1 is
*"1 grid lane to obtain the first obj cell for the non-call physical def"***.
If §2.2 holds, that line item is **unspendable, not merely unspent** — no lane
can buy it — and F4's price is **1**, not 2. And F4's proposed fail-closed
boundary, *"refuse on any bare physical def"*, withdraws the **empty set**: its
two-sided cost is zero. Both are recorded as consequences of a negative result
so the negative result is not filed as "nothing found".

---

## 3. Deliverables, and what this lane will NOT do

* `docs/whitebox/grids/w-regcells/fpr_grid.cpp` — the Q1 cells.
* `docs/whitebox/grids/w-regcells/physdef_grid.cpp` — the Q2 probes.
* `docs/whitebox/WB_REGCELLS_FINDINGS.md` — the graded result, every claim
  carrying its denominator.
* `P_REGALLOC.md` §2.1 / §7 amended **beside**, never rewritten
  (`whitebox/README.md` §2.1).
* `docs/rungs/2026-08-27-w-regcells.md`.

**Will not**: convert a TU, add a gate row (`#3691`), adopt anything into
`crates/` without a `DISCLOSURE.md` row in the same commit, commit any obj, any
`.il`, or any `_CL_*`.
