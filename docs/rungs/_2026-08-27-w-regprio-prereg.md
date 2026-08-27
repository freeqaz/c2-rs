# PREREG — lane `w-regprio`, the worklist comparator `0x10b2b82d`

    Lane:      w-regprio  (REGALLOC_BRIEF_2026-08-27 § L2; decision 20 row 2)
    Kind:      construct rung
    Board:     #3700–#3705
    Frozen:    2026-08-27, BEFORE the first `crates/` edit, BEFORE the first
               `cl.exe` invocation, and BEFORE any mutant was run against any
               cell.

**Everything in this file is registered. Nothing below is amended after the
first measurement; misses stay on the page and are scored in the rung.**

---

## 0. What I had already read when this was frozen — declared, because it bounds what "prereg" can mean here

This lane's population is **already published**. Before freezing I had read:

* `docs/whitebox/ref/P_REGALLOC.md` §1 §2 §4 §4.1 §5 §7 — including the
  comparator rule and both consequences.
* `docs/whitebox/WB_DAGORDER2_FINDINGS.md` in full — including §2's A-series
  table, §4's six flips, §5's mechanism paragraph, and **§5.0's own fence that
  `n=3` is `b a c` where pure descending id predicts `c b a`**.
* `docs/whitebox/ref/P_GLOBREGS.md` §7 §7.1 §8 — read **R4**, `cand+0x44` as a
  tuple-visit ordinal.

So this prereg **cannot** and does not claim to predict the cell data. What it
predicts is what has never been written down anywhere in this repo:

1. what an **executable** comparator does when the published cells are run
   through it under a stated key model;
2. **which of the 20 cells, if any, separates each of five planted mutants** —
   i.e. the discriminating power of the population, which no document states;
3. whether the record's model of the `/O1` cells survives read **R4**.

`§5.0`'s `n=3` fence is the one overlap and is called out as such in P2 below:
the n=3 datum is not mine, the **n=4…8 extension and its consequence for R4
are**.

---

## 1. The deliverable

`crates/c2-core/src/codegen/regalloc_worklist.rs` — c2's sorted insert
expressed as executable Rust, with **every decision the read names exposed as a
named, settable parameter whose default reproduces the read**:

| decision point | default (= the read) | other legal settings |
|---|---|---|
| primary key | `cand+0x0c` | `cand+0x44` |
| primary direction | descending | ascending |
| primary signedness | **signed** | unsigned |
| tie key | `cand+0x44` | `cand+0x0c` |
| tie direction | descending | ascending |
| tie signedness | **unsigned** | signed |
| **tie tier** | **`<=` — new candidate FIRST** | `<` — new candidate LAST |
| re-entry after spill | **by priority** (same comparator) | head (stack) / tail (queue) |

Purely **additive**: no production caller, no byte can move. This is exactly the
state `docs/rungs/README.md` § "Lane kinds" corollary describes — *"the identity
diff becomes a tautology over a purely additive tree with no production
caller"* — and §5 names the axis that CAN fail instead.

**It is additive by instruction, not by choice.** Decision 20 §2 forbids a full
allocator; the comparator **consumes** `cand+0x0c` and `cand+0x44`, and
`P_REGALLOC` §7 + the `dag` scoreboard row (*"the port schedules nothing"*)
say neither key is computable in this port. A production caller would require
the scheduler. If the work starts needing one I stop and report that, per the
brief's hard limit.

---

## 2. The test population, and its known holes — registered before it is used

The 20 order cells are `docs/whitebox/grids/wb-dagorder2/candorder_grid.cpp`,
21 functions of which `cnd_c0` is the instrument **control** (nothing live
across the call ⇒ no callee-saved colour ⇒ no order). 21 − 1 = **20 order
cells**, at `/O1` and `/Ox`.

**Published orders recoverable from the repo: 18 of 20.** `cnd_x3` and
`cnd_x3r` are named in `WB_DAGORDER2_FINDINGS.md` §4 as agreeing across
profiles and their **orders are printed nowhere**; the raw batch
(`work/wb-dagorder2/`) is not in the tree at any commit.

**P0 (0.80) — I will recover all 20 by re-running the committed grid** against
real `cl.exe`/`c2.dll` under wibo, at both profiles, and the 18 published
orders will reproduce **18/18**. This is a regeneration of a frozen, committed
grid, not a new search. **The control is `cnd_c0`: if it takes any of
`r14…r31`, the batch is void and the lane says so.**

If P0 misses (toolchain unresolvable, or a published order fails to reproduce)
the lane falls back to the 18 published cells and **states the reduced
population in the rung headline**, never silently.

---

## 3. The predictions

### P1 (0.85) — the unconstrained "does the comparator reproduce cell X" test is VACUOUS, and so is every mutant of it

Keys `cand+0x0c` and `cand+0x44` are **not observable in an obj**. So the
question *"does the comparator produce the observed order"* is only well-posed
once a key model is supplied. Without one, I predict:

> For every one of the 20 cells, an assignment of `(+0x0c, +0x44)` exists that
> makes the observed order come out — **and the same is true for all five
> mutants**, including the reversed-direction one.

**Discriminator, registered:** run the existence search under the default and
under each mutant. If the default reproduces 20/20 **and** every mutant also
reproduces 20/20, P1 is a HIT and the test is decoration in the precise sense of
`#3336`. If any mutant scores below the default, P1 is a MISS and the population
has real power.

### P2 (0.80) — the record's own model of the `/O1` cells is REFUTED on 6 of the 8 A-series cells

`WB_DAGORDER2_FINDINGS.md` §5: *"an exact tie in both keys puts the NEWLY
inserted candidate FIRST. That last clause is the whole behaviour of the `/O1`
cells in §2, where the benefit keys are equal and the order is decided entirely
by insertion sequence"*; §5.0: insertion sequence is the bucket walk over
`cand+0x1c`, and R1 makes that counter **per-function and dense from 1**, so
bucket order = mint order.

Call that **M-TIE**: *all candidates of an A-series cell tie in **both** keys;
insertion order is ascending mint index; mint index tracks formal order.* Under
`<=`, M-TIE predicts the finished list is **reverse formal order**.

> **Prediction: M-TIE hits `n=1` and `n=2` and is refuted on `n=3…8` — 2 of 8,
> 6 refuted.**

`§5.0` already discloses the `n=3` miss. **The `n=4…8` extension is what this
prediction adds**, and its consequence is P2b.

### P2b (0.70) — R4 explains the refutation, and the claim it refutes is still standing unstruck in the record

Read **R4** (`P_GLOBREGS` §7) makes `cand+0x44` a **monotone tuple-visit
ordinal**, incremented once per real tuple. A dense per-tuple ordinal makes an
exact tie in **both** keys the exception, not the rule — so M-TIE's premise is
wrong in kind, which is why it fails from n=3 on.

> **Prediction: `P_REGALLOC.md` §4.1's clause "and at `/O1` most cells are
> ties" survives R4 with no strike, no revision box, and no successor claim
> anywhere in the repo.**

**Discriminator:** grep every `docs/` page for a sentence that draws this line.
If one exists, P2b is a MISS.

### P3 (0.75) — no cell in the 20 separates `<=` from `<`

The two tiers differ **only** on an exact tie in both keys. Combined with P2's
refutation of M-TIE, I predict:

> **0 of the 20 cells, at either profile, distinguishes the `<=` comparator
> from the `<` comparator.**

This is `#1236`'s exact shape and the brief asks for it by name. **The control
that makes the statement mean something is separate and is P5.**

### P4 (0.70) — no cell in the 20 reaches the re-entry path at all

Consequence 2 (*"a spilled candidate re-enters by priority, not at the head"*)
is a prediction about a port. But `cnd_a8` is the widest cell and 8 values fit
in `r31…r24` with the whole callee-saved run to spare.

> **Prediction: 0 of the 20 cells spills, so 0 of the 20 separates
> `ByPriority` from `Head` (stack) from `Tail` (queue).** The three policies
> are separable only on synthetic input.

### P5 (0.90) — the planted defects go red on synthetic input, and I will watch each one do it

`#3336`: a control never watched fail is decoration. Five mutants, each
required to **disagree with the default on at least one input**:

| mutant | what it changes |
|---|---|
| `MUT-LT` | tie tier `<=` → `<` |
| `MUT-ASC` | primary direction DESC → ASC |
| `MUT-SWAP` | primary and tie keys exchanged |
| `MUT-U0C` | primary key signedness signed → unsigned |
| `MUT-S44` | tie key signedness unsigned → signed |

> **Prediction: all five separate on synthetic input** — and the test that
> asserts it is watched going red by removing the mutation, which is recorded.

### P6 (0.60) — `MUT-S44` is unobservable in any compilation this project can build

`cand+0x44` is a tuple count. A function with 2³¹ tuples does not exist.

> **Prediction: the signed/unsigned choice on the tie key is a decision point
> with an empty observable set** — settable, defaulted to the read, and
> **untestable against any obj**, forever, on this workload. Stated so the
> parameter is not mistaken for a graded one.

### P7 (0.95) — required-zero, and the identity diff is a tautology

> `git diff --stat` touches no existing `crates/` file except one `pub mod`
> line; the 21 gate rows are identical to the base line for line; census +0;
> no new gate row.

---

## 4. What this lane will NOT do — registered so a later reader can check

* **No register allocator.** Decision 20 §2.
* **No `ported` numerator for regalloc.** `#3505`.
* **No new count-bearing gate row.** `#3691`.
* **No claim that the comparator is `[O]`.** The comparator itself is `[R]`.
  What is `[O]` is the *order*, and P1 predicts the order does not confirm the
  comparator.
* **No re-reading of `c2.dll`.** No disassembler is run in this lane; the
  address `0x10b2b82d` is cited from `P_REGALLOC` §4 / `WB_DAGORDER2` §5, and
  the `DISCLOSURE` row will say the read is inherited, not re-taken.

---

## 5. The axis on which this rung CAN fail even with every byte identical

`docs/rungs/README.md` § "Lane kinds", the COST CLAUSE (`#3336`), as amended.
The byte delta is zero **by construction** here (additive, no caller), so the
byte criterion abstains and cannot be the grade. The named axis is:

> **DISCRIMINATION.** The rung fails if (a) any of the five planted mutants
> agrees with the default on **every** synthetic input — meaning the exposed
> parameter is not a decision point at all; or (b) the executable comparator
> cannot reproduce the rule as written in `P_REGALLOC` §4 on a hand-worked
> case; or (c) the lane cannot state, with a number, how many of the 20 cells
> separate each mutant. **"It compiles and the tests pass" is not the grade.**

A second axis, subordinate: **population power**. The lane must publish
`separating_cells(mutant)` for all five mutants over all 20 cells at both
profiles, including the zeros. A zero published is a result; a zero omitted is
the failure `#1236` names.

---

## 6. Outcome word, decided in advance

* `built` — the module lands, all five mutants separate on synthetic input, the
  population power is published with its zeros, and P0…P7 are scored.
* `FAILED` — in those words, if the module does not land, or if the
  discrimination axis of §5 comes back (a).

— PREREG ENDS —
