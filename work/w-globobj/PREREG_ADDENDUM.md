# PREREG ADDENDUM 1 (IN-FLIGHT) — `w-globobj`

Same status as `w-regcells`'s `9d0e23b2d`: **stated before the cells it
describes existed, but after the first grid had been compiled**, so it is
scored in its own tier and never pooled with the frozen `PREREG.md`. The rule
this repo enforces is that a prediction written after seeing an obj is not the
same evidence as one written before, and pooling them launders the difference.

---

## 1. WHAT IS ALREADY MEASURED — stated first, so nothing below reads as a prediction

Compiled at `/nologo /Gy /O1 /GS- /c` (mode W, the workload profile) and
`/nologo /Gy /Ox /GS- /c` (mode X, the fixture profile). **Every result below
is identical at both modes.**

* **Series O — 14 cells, both modes: the register map follows the DEFINITION
  order, exactly, in all 28.** Declaration order moves it by **zero** cells;
  use order moves it by **zero** cells. All six N=3 definition permutations
  produce six distinct maps, so the grid is alive.
* **Series P — 16 cells, both modes.** `pc_int` PROMOTED (positive control
  fires), `pc_vol` MEMORY (negative control fires). **Three predictions
  MISSED**: `pc_arr` (`int[2]`) and `pc_union` came back **PROMOTED**, not
  MEMORY; and the OPEN cell `pc_struct1` came back **PROMOTED**.
  `pc_struct2` is MEMORY as predicted.
* **Series V — `vc_three` gives ONE source symbol TWO colours** (`r31`, then
  `r28` twice), and `vc_three_distinct` — three genuinely distinct locals in
  the same shape — is byte-identical to it.

## 2. WHAT THOSE MISSES OPEN, and the cells that decide it

`pc_arr` and `pc_union` promoted while `pc_struct2` did not. The obvious
reading — *"aggregates are rejected"* — is now **dead**, and the reading that
replaces it has a confound I must name before testing it:

> `pc_struct2` writes `S2 v = *p;`, which the **front end** lowers to a single
> 8-byte `ld`/`std` whole-object copy (`ld 11, 0(3)` / `std 11, 80(1)` in the
> obj). `pc_arr` writes `v[0] = p[0]; v[1] = p[1];` — two scalar assignments.
> **The difference may be entirely c1xx's, not `FUN_10b550e5`'s.**

**New cells, predictions frozen here:**

| cell | source | prediction |
|---|---|---|
| `pa_struct2mem` | `S2 v; v.a = p[0]; v.b = p[1];` — member-wise, same type as `pc_struct2` | **PROMOTED** |
| `pa_struct4mem` | `S4` (four ints), member-wise | PROMOTED |
| `pa_struct2cpy` | `S2 v = *p;` — the `pc_struct2` shape, restated as this grid's control | MEMORY |
| `pa_arr4` / `pa_arr8` / `pa_arr12` | `int v[N]`, every element assigned from `p[i]` and used after the call | PROMOTED at N=4 and N=8 |

**If `pa_struct2mem` is PROMOTED, `pc_struct2`'s MEMORY verdict is a
front-end artifact and carries no information about gate A or gate B** — and I
will say so rather than bank it as a gate-B confirmation.

**The ceiling on the array ladder, registered now:** the frame-traffic readout
**cannot separate "was never promoted" from "was promoted and then spilled".**
Above roughly a dozen simultaneously live values the callee-saved run
(`r14…r31`) runs out and spilling is the correct behaviour of a working
allocator. `pa_arr12` is therefore reported as **data, not as a graded cell**,
and no threshold claim is made from it in either direction. §3's *"no size
threshold, no use-count threshold"* is tested only where the readout is sound.

## 3. THE CELL THAT ATTACKS MY OWN REGISTERED CEILING — series O-LR

`PREREG.md` §5.4 registered that this lane *cannot* separate `cand+0x44` from
`cand+0x0c`, because moving a definition earlier both raises the ordinal and
lengthens the live interval. **That ceiling is attackable and I did not see
how when I wrote it.** The move is to hold the definition order fixed and move
the *last use*, which changes the live interval **without** changing the
definition ordinal — and, in the mirror cell, to make the **later-defined**
candidate the **longer-lived** one.

```
pad = t = sink(t); three times — real tuples that touch neither local
ol_dxy_xlate  : x=p[0]; y=p[1]; call; u(y); pad; u(x);   /* x defined 1st, x lives longest  */
ol_dxy_ylate  : x=p[0]; y=p[1]; call; u(x); pad; u(y);   /* x defined 1st, y lives longest  */
ol_dyx_xlate  : y=p[1]; x=p[0]; call; u(y); pad; u(x);   /* y defined 1st, x lives longest  */
ol_dyx_ylate  : y=p[1]; x=p[0]; call; u(x); pad; u(y);   /* y defined 1st, y lives longest  */
```

`ol_dxy_ylate` and `ol_dyx_xlate` are the **discriminators**: in each, the
earliest-defined candidate is the shorter-lived one.

| rival | `ol_dxy_ylate` | `ol_dyx_xlate` |
|---|---|---|
| **DEF** (this lane's) | `x → r31` | `y → r31` |
| **LIVELEN** (longer live range first — the direction `cand[0x0c] += cand[0x18] * n_live` accumulates in) | `y → r31` | `x → r31` |

**Prediction: DEF wins both, LIVELEN is refuted by 2 cells per mode.**

**And the use-count axis**, because `cand+0x18` is a per-candidate weight:

```
ou_x2 : x=p[0]; y=p[1]; call; u(x); u(x); u(y);   /* x defined 1st, x used twice */
ou_y2 : x=p[0]; y=p[1]; call; u(x); u(y); u(y);   /* x defined 1st, y used twice */
```

`ou_y2` is the discriminator: **DEF** says `x → r31`, **USECOUNT** says
`y → r31`. **Prediction: DEF wins; USECOUNT refuted by 1 cell per mode.**

**WHAT A DEF WIN HERE DOES AND DOES NOT LICENSE — registered before the
compile.** It does **not** promote `cand+0x44` to `[O]`: the obj shows a
composite order, and a `+0x0c` that happens to be ordered by definition
position would produce the same picture. What it **does** license is the much
narrower, and true, statement that **live-range extent and use count — the two
source-level quantities a priority accumulator is most likely to be a function
of — do not order the observable**, which is a real narrowing of §5.4's
residue and is the honest form of the result. If DEF *loses* either
discriminator, §7.1's ordinal reading is refuted at the observable and that is
the better outcome.

## 4. THE MERGE CELL — line 169 of `MARKS.tsv`, moved from `UNCOMP` to `OBS`

`MARKS.tsv` filed `FUN_10b54c07` (§5, the merge at joins) as `UNCOMP` and named
the cell that would decide it. It costs three lines, so this lane builds it.

```
vm_merge   : int x; if (c) x = p[0]; else x = p[1]; call; u(x);
vm_nomerge : int x, y; if (c) { x = p[0]; call; u(x); } else { y = p[1]; call; u(y); }
```

§5 says the merge is **keyed on the symbol** and either reuses an existing
version number or mints a fresh one, so the two arms of `vm_merge` are one
symbol reaching a join. **Prediction: both arms of `vm_merge` load into the
SAME register** — one candidate survives the join. If the two arms load into
different registers and a copy appears at the join, the merge minted a fresh
version per arm and §5's reuse clause is what is doing the work; either way the
cell decides something and it is reported as data, not as a pass/fail.

## 5. UNCHANGED

Everything in `PREREG.md` §7 (controls), §8 (what this lane will not do) and §9
(what makes it `FAILED`) applies verbatim. Reach stays 0; no `crates/` file is
touched; no gate row is added.

---

# PREREG ADDENDUM 2 (IN-FLIGHT) — the cell that can break this lane's ceiling

Written after addendum 1's grids were graded and **before `order_loop_grid.cpp`
exists**. Scored in the IN-FLIGHT tier.

## A. What addendum 1 returned, stated before anything new is predicted

**42 graded cells, 0 scored `U`. `DEF` survives 42/42 at both profiles. Seven
rivals refuted by cell count:** `REVDEF` 42, `REVDECL` 28, `DECL` 22, `USE` 22,
`LASTUSE` 22, **`LIVELEN` 12**, `USECOUNT` 4.

**`LIVELEN` being refuted by 12 cells is the load-bearing one**, and it needed a
correction to this lane's own grader to appear: the first `parse_grid` indexed
positions by `u_i` call order, which made `order_lr_grid.cpp`'s padding
invisible, computed `LIVELEN == DEF`, and scored the two cells built to separate
them as agreeing. `--selftest` now carries an assertion that fails on that
version.

## B. The inference that opens the deciding cell

`P_REGALLOC.md`:71 reads the priority accumulator as
`cand[0x0c] += cand[0x18] * n_live` where live, `-= n_live` where not.

* If `cand+0x18 > 0`, a longer-lived candidate accumulates strictly more →
  sorted DESC, longer-lived first → **`LIVELEN`**.
* If `cand+0x18 == 0`, `+0x0c` is `-Σ n_live` over the points where the
  candidate is **not** live; a longer-lived candidate is not-live at fewer
  points, so its `+0x0c` is larger → sorted DESC → **`LIVELEN` again.**

**Under either sign, a `+0x0c` that is a monotone function of live extent
predicts `LIVELEN`, and `LIVELEN` is refuted by 12 cells.** Exactly two
survivors, and this lane cannot yet choose between them:

1. `+0x0c` **ties** across these cells and the comparator falls to `+0x44` —
   §7.1's ordinal reading, supported at the observable; or
2. `+0x0c` is itself ordered by definition position, and `P_REGALLOC`:71's
   reading of the accumulator is wrong or incomplete.

## C. The cell that separates them

If `+0x0c` can be made to move by a source-level quantity that is **not**
definition position, then survivor 2 is dead and the LR cells were decided in
the tie tier. The obvious such quantity is a **loop-weighted use count** —
`cand+0x18` is a per-candidate weight and loop depth is what a priority
accumulator exists to express.

```
ob_loop_y : x=p[0]; y=p[1]; call; u(x);            for(i<n) u(y);   <- DISCRIMINATOR
ob_loop_x : x=p[0]; y=p[1]; call; for(i<n) u(x);   u(y);            (consistency)
ob_loop2_y: x=p[0]; y=p[1]; call; u(x);  for(j<n) for(i<n) u(y);    (depth 2)
```

| | `ob_loop_y` |
|---|---|
| **DEF** (unbeaten on 42 cells) | `x → r31` |
| **LOOPWEIGHT** (`+0x0c` responds to loop depth) | `y → r31` |

**PREDICTION: `y → r31`.** `DEF` is refuted for the first time, `+0x0c` is
shown to move on a quantity that is not definition position, survivor 2 dies,
and the 42 straight-line cells are thereby located **in the tie tier** — which
is `[O]`-grade support for §7.1's ordinal, and it would **break the ceiling
this lane registered in `PREREG.md` §5.4 before any cell existed.**

**If `x → r31` instead**, the loop does not move the priority either, `DEF`
stands at 45/45, and §5.4's ceiling is **confirmed by a cell built to break
it** — the residue stays exactly as registered, and this lane will say so.
Both outcomes are publishable; neither is an exit code.

**The premise test that must pass first:** `n` arrives as a second formal, so
the loop cells have a third candidate. A cell in which `x` and `y` do not both
resolve to distinct callee-saved registers scores `U` and enters no numerator
and no denominator, exactly as in §7 of `PREREG.md`.
