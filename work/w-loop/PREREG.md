# w-loop — pre-registration

Committed **before** `loopcost.py` / `loopshape.py` were run. Scored verbatim in
`docs/rungs/2026-08-05-w-loop.md` §1; the wrong ones stay on the page.

Board numbers taken: **#740**–**#747**.

---

## 0. What is already re-derived (NOT a prediction — measured before this file)

The brief's proof object is `src/system/math/Sort.cpp`, characterized as *"one
function, 80 bytes, zero relocations, no `.pdata` … Blocked **only** by
`cflow-loop`"* and *"unlike every previous attempt, the thing in the way is one
structural gap rather than a chain of unmodelled tokens."*

The first sentence reproduces. **The second does not**, and the counting is in
§1 of the rung. `?HashString@@YAHPBDH@Z` is:

```
0000  lbz r11,0(r3)      0018  lbzu r10,1(r9)     0030  mullw r7,r7,r4
0004  mr  r9,r3          001c  add  r8,r8,r11     0034  andc  r6,r4,r10
0008  li  r10,0          0020  mr.  r11,r10       0038  twi   6,r4,0
000c  cmplwi cr0,r11,0   0024  rotlwi r10,r8,1    003c  subf  r10,r7,r8
0010  bt  2,+56          0028  divw r7,r8,r4      0040  twi   5,r6,-1
0014  mulli r8,r10,127   002c  addi r10,r10,-1    0044  bf    2,-48
                                                  0048  mr    r3,r10
                                                  004c  blr
```

A signed `%` expansion (`divw`/`mullw`/`subf`), **two `twi` trap instructions**
with a three-instruction predicate (`rotlwi`/`addi`/`andc`), an update-form
`lbzu`, a record-form `mr.` branching on **cr0**, `mulli`, and a schedule that
interleaves the trap predicate between `divw` and `mullw`. That is the
`≥ 4 independent refusals` decline clause several times over, and it is a
**chain of unmodelled tokens with a loop around it**, which is the exact shape
the brief says it is not. `cflow-loop` is the *first* refused thing, and
`CFG_SHAPE.md` §5.0 Trap 1 says in terms that a blocker name is never a promise.

## 1. The observability claim — the thing this lane is actually testing

`coff::plan_labels` (`crates/c2-core/src/coff/label.rs:39`) returns
`Some([n,n+1,n+2])` **only** for a function with a `frame`, and `None` for a
leaf. So the counter's *value* reaches the obj through exactly one channel:
the `$M<n>`/`$T<n>` short names a **framed** function's symbol group carries.

`labels.rs` invariant 4 refuses every backward reference, and its whole stated
justification is *"the obj would carry a wrong `$M`"*. If a TU mints no `$M` at
all, that justification is vacuous **for that TU**.

| # | prediction | registered rival |
|---|---|---|
| **P1** | An obj whose every function is a **leaf** carries **zero** storage-class-6 symbols and zero `$T`, for **every** loop shape in the grid — 12/12. | **R-P1:** c2 mints a label symbol per loop head (a `$L`/`$M` family keyed on blocks), so a leaf loop is observable after all. `CFG_SHAPE.md` §3.6 refuted this for forward branches (`?d_early`, `?d_switch`, three targets each, zero label symbols); a **backward** branch is a different cell and is what P1 tests. |
| **P2** | A **leaf** loop function's seed-free stride is **> 1** — leaf loops charge the counter, the same way `cflabels.py`'s framed rows do (`for` +2, `while` +2, `do/while` +1). | **R-P2:** a leaf loop's stride is exactly **1**, identical to `leaf-int`, because the control-flow surcharge is a property of the framed pre-pass and not of the body. **This is the load-bearing cell of the lane** and I do not know its answer. If R-P2 wins, a backward branch in a leaf costs the counter nothing anywhere and the refusal is over-broad in general, not merely on `$M`-free TUs. |
| **P3** | If P2 wins (leaf loops charge), the charges will **not** be uniform across `while`/`do-while`/`for`, mirroring §4's framed `+2/+1/+2`. | **R-P3:** uniform at one integer, i.e. the leaf case is simpler than the framed one. |
| **P4** | The three pure-loop frontier TUs (`Sort.cpp`, `Primes.cpp`, `IPP_basicmath_xbox.cpp`) contain **no framed function at all**, so P1 makes the counter unobservable in every one of them. | **R-P4:** at least one carries a `.pdata`/`$M`. |

**Bias registered in advance:** I want P2 to lose (R-P2 to win), because that is
the reading under which a loop lowering is buildable without a new IL channel.
P2 is written as the prediction *against* my bias for that reason.

## 2. The emission-shape grid

`CFG_SHAPE.md` §3.7 measured 7 loop bodies. §8.2 lists five open items (L1–L5).
This grid crosses the axes §8.2 asks for. **All probes are LEAVES** — §3.7's
grid is framed or call-bearing on 4 of its 7 rows, and every loop function on
the frontier is a leaf, so the framed rows do not price the target.

Axes: `{while, do-while, for, for(;;)}` × `{counted, sentinel}` ×
`{body calls, body does not}` × `{break, continue, neither}` ×
`{counter live after, not}` × `{nested, flat}`.

| # | prediction | registered rival |
|---|---|---|
| **P5** | A **leaf** loop whose trip count is a loop-invariant register at entry and whose body contains **no call** emits `mtctr`+`bdnz` (`BO=16,BI=0`) — §3.7c generalized. | **R-P5:** CTR is chosen only when the count is an *induction variable compared against a bound*, and a `while(n--)` sentinel form gets the compare back edge. |
| **P6** | A leaf loop with a `break` gets the **compare** form, never CTR, because the CTR count is not the only exit. `?d_break` is one cell for this and is call-bearing. | **R-P6:** c2 keeps CTR and adds a second `bc` exit. |
| **P7** | A leaf loop whose induction counter is **read after the loop** gets the compare form (CTR is not readable as a GPR). | **R-P7:** c2 emits CTR and a parallel GPR counter. |
| **P8** | Across the whole grid, **every** back edge is a conditional branch — `bc` or `bdnz` — and **no** loop emits an unconditional backward `b`. §3.7a on 7 cells; this extends it to the leaf grid. | **R-P8:** a `for(;;)` with the only exit inside the body emits an unconditional backward `b`. **This is the cell most likely to break P8** and it is in the grid on purpose. |

## 3. What ships, in each of the two worlds

**Discipline registered in advance** (the brief's own bar): *a rule that fits but
not enough ships as a refusal.* Concretely:

* **If P1 holds and R-P2 wins** — leaf loops are counter-free — the refusal in
  `labels.rs` invariant 4 is narrowed by a **positive guard**: a backward
  reference is admitted only when the caller proves the enclosing function mints
  no label triple *and* no later function in the TU does either. Additive-refusal
  by construction: the guard's default is refuse, and the only reading acted on
  is an explicit `Some(true)`.
* **If P1 holds and P2 wins** — leaf loops charge — then the charge is
  observable only through a *later framed* function, so the guard above is still
  sound (it demands no later framed function), but it is sound for a **narrower
  reason** and the doc must say which. Ship the guard and the measurement.
* **If P1 fails** — a leaf loop mints a symbol — the whole route is closed, the
  guard does not ship, and the lane reports the refutation. `#286`'s "derive it
  from the blocks" would then be closed a second way.

**No lowering ships without the oracle.** Any `Selected` variant that encodes a
back edge is graded by `c2rs diff` against real `c2.dll` on a committed fixture
before it is committed, and the fixture goes in `scripts/lanes.txt`/the sweep so
the class cannot go unwatched (`gate.sh cannot see expr_sweep` is a standing
memory).

## 4. The corpus question the brief demands

> *Before you ship: what shape would break this, and can the corpus express it?*

For a backward-branch guard the breaking shape is **a TU where a loop leaf is
followed by a framed function** — the counter is then written into the obj and a
wrong charge is six wrong bytes. The generated sweep enumerates single-function
TUs at `/Ox`, so **it cannot express this shape at all**. A fixture with exactly
that layout (`loop leaf` then `framed`) is therefore a *required* deliverable of
any shipping guard, not an optional extra — and it must be a fixture the gate
runs, not a probe in `work/`.
