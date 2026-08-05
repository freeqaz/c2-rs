# w-rotate — PRE-REGISTRATION

Committed **before any probe script in this directory exists**. Lane `w-rotate`,
worktree off master `707328d` (the tip that merged `wt-w-hash`).

The brief hands over three mechanisms that separate w-hash's *transcription*
from a *lowering*: (1) loop rotation, (2) memory-reference peeling, (3)
allocation across the back edge. It asks for **one** of them turned into a rule,
on more than one witness.

---

## 0. The mechanism taken, and why the other two are passed over

**Taken: ROTATION.** Not because it is the most valuable of the three but
because it is the only one of the three that the project has already written
down as a *named open question with a stated shape*, which means a grid can
close it rather than explore it:

* `docs/CFG_SHAPE.md` §8.2 **L4 — "Loop entry form"**: *"`?c_callloop` and
  `?d_break` guard with a compare; `?d_cont` jumps into the test. I can say
  **that** both occur and that `?d_cont` differs by having a `continue`; **I
  cannot state the rule**."*
* `docs/rungs/2026-08-05-w-loop.md` §4 **L4**: the obvious rule (*a `continue`
  makes the test a join target*) is **dead** — a leaf `+continue` keeps the
  guarded/CTR form where the call-bearing `?d_cont` jumps in. So L4 is open with
  one rival already refuted, which is the cheapest possible starting position.

**Passed over: allocation (mechanism 3).** `CFG_SHAPE.md` §8.2 L1 calls it *"the
largest single unknown in the document"* and says in terms that it is **a
prerequisite lane, not a sub-task of the loop rung**. w-hash's R8 lost on the
outcome while being right that the allocation is not derivable. A lane that
opened on L1 would be a characterization lane for the allocator and would ship
no class. That is a legitimate rung; it is not this one, and pretending a
one-day grid could close it is how a transcription gets dressed up.

**Passed over as a *primary* subject: peeling (mechanism 2)** — but **not
ignored**, because P4 below registers the claim that peeling is *not an
independent mechanism at all*: it is what rotation does when the duplicated test
reads memory. If P4 holds, taking rotation takes peeling with it. If P4 loses,
peeling is a genuine fourth mechanism and that finding is the deliverable the
brief asks for.

---

## 1. The definitions, fixed before measuring

Rotation is a claim about **two counts that must be measured on two different
artifacts**, and both halves are stated here so neither can be adjusted later.

* **IL test sites** — the number of distinct conditional-branch statement sites
  in the function's IL body (the `38`/`39` opcodes of `IL_STMT_GRAMMAR.md` §7)
  that read the loop's exit condition.
* **OBJ test sites** — the number of distinct instruction sites in the emitted
  `.text` that compute the loop's exit condition and branch on it.

**ROTATION := OBJ test sites > IL test sites for the same condition.**

Every probe's entry form is classified into exactly one of four buckets, decided
mechanically from the obj bytes (never by eye):

| bucket | signature in `.text` |
|---|---|
| **GUARD** | a conditional branch, *before* the loop top, whose target is at or beyond the back edge (i.e. it branches **out**), with a compare or record-form producer ahead of it |
| **GUARDRET** | the same, folded to a `bclr` form (`beqlr`/`bnelr`/`blelr`…) — no displacement at all |
| **JUMPIN** | an **unconditional** forward `b` before the loop top whose target is at or beyond the bottom test — the IL's own entry jump, surviving |
| **NONE** | neither: the first instruction reached is the loop body, and the only test is the back edge (the `do/while` pole) |

A probe that produces two loops, or no back edge, is reported as its own bucket
(`MULTI` / `NOLOOP`) and is **excluded from every rate**, with the exclusion
count printed. Absence must never be read as a bucket.

**Grading discipline.** Every rate in the rung is `graded / reached`, where
*reached* is the number of cells the toolchain produced an obj for and *graded*
is the number the classifier assigned a bucket to. A cell that fails to capture
is a failure, not a zero. The counts are printed even when they are equal.

---

## 2. The registered predictions

Each carries a named rival. **P1 and P8 can lose, and P8 is registered against
my own goal** — I want P8 to lose, because if it loses a body-parameterized loop
lowering is shippable and `cflow-loop` enters `PORT_CFG_CLASSES` with a real
free parameter rather than a constant.

| # | prediction | rival if it loses |
|---|---|---|
| **P1** | **Every top-test loop in a LEAF at `/O1` rotates** — bucket GUARD or GUARDRET, never JUMPIN. JUMPIN is a property of call-bearing/framed bodies | **R-P1**: JUMPIN occurs in leaves too, so `?d_cont`'s form is not about the frame class and L4 needs a third axis. The risk cell registered in advance is `leaf-for-cont` (a `continue`, i.e. the test *is* a join target, in a leaf) |
| **P2** | **The guard's form is decided by the EXIT BLOCK, not by the loop.** GUARDRET iff the block the guard branches to is a bare `blr` — i.e. the returned value already sits in r3 at the guard. Otherwise GUARD with a real displacement | **R-P2**: the fold is a size/distance decision (a c2 cost model), in which case it is `CFG_SHAPE.md` §3.5's fold table again and is **not** derivable — and a lowering must refuse rather than fit it |
| **P3** | **The guard tests the same condition as the back edge with the sense inverted**, over the loop's *entry* values: one condition in the IL, two sites in the obj, same relation | **R-P3**: c2 emits a cheaper or different entry test (e.g. a null check where the back edge compares), so the guard is not a copy and cannot be emitted from the back edge's condition |
| **P4** | **Peeling is entailed by rotation, not independent.** A peeled memory reference appears iff the duplicated test's operand is a load addressed by the induction variable. Where the condition tests a register-resident value, there is **no** peel | **R-P4**: peeling occurs without rotation, or rotation occurs without peeling on a load-carried test — in which case peeling is a genuine **fourth mechanism** and this lane's deliverable is that finding (the brief's "hidden fourth mechanism" clause) |
| **P5** | The entry form is a function of the loop's **graph** (top-test vs bottom-test, exit count, join-ness of the test) and not of its body's **content** | **R-P5**: content decides — a call in the body, or CTR eligibility, changes the entry form at a fixed graph. I expect R-P5 to win on at least one cell; registering P5 anyway so the cell that breaks it is named rather than discovered |
| **P6** | `do/while` (bottom-test in the IL) is bucket **NONE** in every cell: no guard is synthesized for a loop the source guarantees runs once | **R-P6**: c2 guards it anyway |
| **P7** | Over the whole grid, **every** back edge is conditional (w-loop's L5, 0 of 28) — extended to this lane's larger grid | **R-P7**: an unconditional back edge exists at some shape, which would break the rotation model outright |
| **P8** | **The sentinel-walk family's register plan is NOT constant across accumulate bodies.** Varying only the loop body (not the signature) re-plans registers, so a body-parameterized lowering needs the allocator and the class must again be drawn to a constant | **R-P8 — the reading I WANT.** The plan is stable across a stated body family, in which case the displacement is computed from a real body length and the lowering has a free parameter its own grid validates. **This is the cell that decides whether this lane ships a class or an honest negative.** |

---

## 3. What counts as success, and the three worlds

Registered so the rung cannot be graded on a moved target.

1. **World A — a class ships.** `cflow-loop` enters `PORT_CFG_CLASSES` under a
   *stated* restriction, the rotation rule is applied by the emitter rather than
   transcribed, and the entry is defended by a cross-product grid graded against
   real `c2.dll`, plus a committed fixture that can express the failure and a
   must-fail mutation that is **run**.
2. **World B — the rule is stated, nothing ships.** The grid states L4 and the
   rung says exactly which mechanism still blocks and what witness would change
   it. This is a **success**, and it is a better outcome than World C.
3. **World C — a second transcription dressed as a lowering.** Explicitly
   declared a **failure** here, in advance, so that shipping it would require
   editing this file.

**A conversion is not the success condition and will not be claimed as one.** If
a TU converts, that is reported as a side effect with its own numbers.

---

## 4. The corpus question, answered before it is needed

Board **#747**'s requirement, inherited. If this lane ships anything, its
failure mode must be producible by a committed fixture, because:

* `scripts/expr_sweep.sh` generates **single-function TUs** at a fixed `/Ox`
  profile — it cannot express a loop at `/O1` beside a framed function, and it
  cannot express two loops in one TU;
* `scripts/mode_cross.sh` crosses **that same corpus** with the lane registry,
  so it inherits the same blindness.

**Registered now:** the breaking shape for a *rotation* rule is a loop whose
**guard's displacement is not the class's constant** — i.e. a body of a
different length from the one the rule was measured on. If a lowering ships with
a computed displacement, the fixture must contain **two bodies of different
lengths** in the same class, or the grid proves nothing that a transcription
would not also pass. Board #644's warning applies to the guard's producer: real
`c2` splits `lis`/`ori` across other instructions, so any rule that locates the
guard's compare *positionally* must survive a producer that is not contiguous.

---

## 5. Anti-fitting commitments

* **`Some(false)` is the only reading acted on.** Every gate added is a positive
  guard; an unmeasured shape refuses.
* **No rule is fitted to fewer than 3 cells**, and no *placement* rule is fitted
  at all — this project has refuted ten of them (`w-pair` §4's six,
  `leaf_store.rs`'s four) and w-hash declined an eleventh.
* **`fnbyte-differs` is 0 and must stay 0.** If it moves, a wrong emit shipped.
* The metric block is regenerated only if TU match moves.

---

## 6. ADDENDUM — registered after Grid A, **before Grid C exists**

Grid A (32 cells) and Grid B (10 cells) are run and committed at `36ca54f`.
They produced **two JUMPIN cells out of 42**, and a rule fitted to two cells is
exactly what this project forbids. So the candidate discriminator for L4 is
registered here, with its rival, and Grid C is written **after** this paragraph
is committed.

**H-EXIT (the candidate L4 rule).** c2 **duplicates** the loop test — the
rotation proper, bucket GUARD/GUARDRET — **iff the loop produces a value the
exit block consumes.** When the loop produces nothing the exit uses, c2 emits
the test **once** at the bottom and enters it with an unconditional `b`
(bucket JUMPIN), which is the IL's own `3A Ltest` surviving.

It fits all 42 cells graded so far: `exit-const` (`return 7`, the accumulator
dead) and `exit-void` (nothing returned at all) are the two JUMPIN cells, and
every rotated cell has an exit block reading a register the loop wrote.

**R-H-EXIT**, the named rival: the discriminator is **size** — c2 picks
whichever form is shorter. This is already in trouble and is registered anyway
so the refutation is on the page: `b-add` rotates at **10 words** where the
JUMPIN form of the same loop would be **9**, so c2 chose the longer form. If
R-H-EXIT nonetheless wins on Grid C, H-EXIT is dead and L4 stays open.

**Per-cell predictions are registered in the grid source itself**, as a column
the script grades — `n of m`, printed, with a cell the classifier could not
place counted as a miss and never as a skip.

**And a second registered claim, from Grid B, which is the one that decides
World A vs World B.** P8 **lost** as I wanted: over 10 accumulate bodies with
the signature held fixed, the `(entry, tail)` plan is **byte-identical** — one
distinct plan over ten cells, with the body length moving 10 → 12 words. So the
register allocation is *not* what blocks a body-parameterized lowering of this
family. What moves instead is the **SCHEDULE**: the loop-carried `lbzu` and the
record-form test are interleaved *into* the accumulate chain at positions that
change with the chain's length (`lbzu` at slot 0 for a 1- and 2-op body, slot 1
for a 3-op body; the record form always exactly 2 slots after it).

**S1, registered now:** that interleave is a **fourth mechanism**, distinct from
w-hash's three — it is not rotation, not peeling, and not allocation (the
registers are stable across all ten cells; only the order moves). If S1 holds,
the honest outcome of this lane is World B plus a named fourth mechanism, and I
will **not** widen the class by fitting the interleave to three cells.
**R-S1:** the interleave is a stateable function of the chain length over a
grid, in which case it is a rule and World A is open.
