# w-divmod — PREREG

Lane `w-divmod`, worktree branch `worktree-agent-ab15a64e064b9a1d3`, off master
**`707328d`** ("merge wt-w-hash: Sort.cpp CONVERTS").

**Committed before any probe script in `work/w-divmod/` existed.** Everything
below is written from *published prior art* — `docs/rungs/2026-08-05-w-hash.md`
§5.1's eight mnemonic rows and `crates/c2-core/src/codegen/ptr_walk_loop.rs`'s
twenty transcribed words — and from reading code. **No cell of my own grid has
been run.** The toolchain was configured in this worktree
(`scripts/configure_existing_worktree.sh`) and one smoke capture
(`w5_chain.cpp`, 4/4 in class) was produced by that script; nothing else.

---

## 0. The target, restated

w-hash lifted the signed `%` spine out of `?HashString`'s loop far enough to
show it is **separable** — `int P(int a,int b){return a%b;}` emits the same
eight words in the same order as the loop's tail — and then **declined to fit
the schedule**, because it saw **two distinct `twi 6` placements** and could not
name the discriminator. §9.1 of that rung calls it *"the smallest unexplained
residual this lane produced, and it blocks the div/mod leaf."*

The eight rows it published (mnemonics only — **no register fields**, which is
the first thing I intend to add):

```text
  s-mod-var    a % b        rotlwi divw addi mullw andc twi subf twi
  s-div-var    a / b        rotlwi divw addi twi andc twi
  u-div-var    unsigned /   divwu twi
  u-mod-var    unsigned %   divwu twi mullw subf
  s-mod-k7     a % 7        li divw mulli subf
  s-mod-k2     a % 2        srawi addze rlwinm subf
  s-madmod     (c+r*127)%i  mulli add TWI rotlwi divw addi mullw andc subf twi
  s-mod-lhsk   100 % b      li TWI rotlwi divw addi mullw andc subf twi
```

## 0.1 The structure I am reading into those rows, stated so it can be wrong

Two dependence chains and one dependence-free singleton:

| chain | instructions |
|---|---|
| **P** — the `INT_MIN`/`-1` overflow predicate | `rotlwi` → `addi` → `andc` → `twi 5` |
| **Q** — the quotient | `divw` → (`mullw` → `subf`, for `%` only) |
| **C** — the zero-divisor trap | `twi 6`, reads the divisor only, ready at entry |

Under that reading the two published in-spine rows are a strict **alternation
P,Q,P,Q,…** starting with P, with C inserted:

```text
  %  :  P0 rotlwi   Q0 divw   P1 addi   Q1 mullw   P2 andc   [C twi6]   Q2 subf   P3 twi5
  /  :  P0 rotlwi   Q0 divw   P1 addi   [C twi6]   P2 andc              P3 twi5
```

and the two *hoisted* rows drop C out of the spine entirely, leaving a spine
that is `s-mod-var` **minus** its `twi 6`, with C placed immediately after the
dividend's producer.

**The fact that kills the obvious rule is already on the record**: `?HashString`
computes its dividend (`mulli` ; `lbzu` ; `add`) and does **not** hoist. So
"computed dividend ⇒ hoist" is refuted before I start, and the discriminator is
something else.

---

## 1. Registered claims

Scored in the rung, hits and misses both. At least three of these are written to
lose.

### R1 — baseline, to the digit

The tree at `707328d` reproduces w-hash §1/§7.2 exactly: TU match **10**,
mismatch 0, codegen-gap 0, vocab-gap **861**, capture-fail 7; A **28** (LO 27) /
B **338** / C **169** / D **10** / E **2**; `B∧C` 151, `A∧B∧C` 27, FRONTIER
**17**; FBM 0.16654, `fnbyte-exact` **29,802**, **`fnbyte-differs` 0**,
`fnbyte-partial` 9,375; census/gate disagreement 0; workspace tests **871
passed, 27 targets**; gate **18/18, 4,680 fixture-verdicts**; sweep **16,710
reached / 16,614 graded / 96 ungraded**; cross **81,905 / 81,517 / 388
ungraded**.

*Expected: HIT. If any digit differs the rest of this prereg is measured against
a tree I have not identified, and that is the first thing to report.*

### R2 — the `TO` fields, read off the encoding and not asserted

`twi 6,rD,0` has `TO = 6 = 0b00110` = *equal* ∪ *unsigned-less-than*; `rD <u 0`
is unsatisfiable, so it traps **iff the divisor is zero**. `twi 5,rO,-1` has
`TO = 5 = 0b00101` = *equal* ∪ *unsigned-greater-than*, over
`andc(divisor, rotlwi(dividend,1) - 1)`.

I will **decode the `TO` field out of the instruction word** for every trap the
grid produces and print it, rather than reading the mnemonic. w-hash's R6 hit
because it did this; the same undertaking is made here.

*Expected: HIT (re-verification).* **The part that is not re-verification**: I
also claim `TO` takes **only** the values 5 and 6 across the entire grid — no
third trap form appears for any operand kind, width, or signedness.

### R3 — THE DISCRIMINATOR. Three rival readings; at most one survives

The hoist of `twi 6` out of the spine happens iff:

* **R3-a** — the `divw`'s block is the function's **entry block** *and* at least
  one operand of the division is produced by an instruction in that block.
  (Explains all four published rows *and* the loop: the loop's `divw` is in the
  body block, which has two predecessors.)
* **R3-b** — at least one operand is produced in the same block, **and the
  block has exactly one predecessor** (i.e. it is not a join point). Differs
  from R3-a on a straight-line `if`-arm block, which is not the entry block but
  has one predecessor.
* **R3-c** — the position is a function of **the divisor's last use**, not of
  the dividend at all: `twi 6` is emitted immediately after the final
  instruction that reads the divisor register, unless that instruction is the
  spine's last, in which case it precedes it.

**I register R3-a as my lead reading and I expect it to lose**, because every
one of the three is fitted to four published rows plus one loop, and this
project has refuted **ten** placement rules fitted to that much evidence
(`w-pair` §4's six, `leaf_store.rs`'s four). The grid below is built to kill at
least two of the three. If all three die, R3 is scored **MISS** and the rung
says so and names the residual — a refusal with the mechanism named, per the
brief.

### R4 — a computed **divisor** is an axis w-hash never varied

`a % (b+1)`, `a % (b*c)`, `a % (b|1)`. The trap reads the divisor, so it
**cannot** precede the divisor's producer — that much is forced by register
dependence and is a **control**, not a prediction. The claim under test is the
non-forced half: with a computed divisor, `twi 6` is emitted **immediately**
after the divisor's producer and the spine carries no trap.

*Expected: HIT. This is the cheapest new witness and the one most likely to
separate R3-c from R3-a/b.*

### R5 — the simplest positional rule, registered because I expect it to LOSE

Whenever `twi 6` is inside the spine, it is the **antepenultimate** instruction
of the spine — exactly two non-branch instructions follow it. It holds on three
of w-hash's four in-spine rows by construction (`%` 8 ops, trap at index 5; `/`
6 ops, trap at index 3; `u-mod` 4 ops, trap at index 1) and I expect the grid to
break it.

*Written as "I want this to lose". If it survives 60+ cells it stops being a
coincidence and becomes the rule, and that would be the lane's result.*

### R6 — the single-cell trap: does a known-nonzero literal divisor delete a guard?

`a % 7` emits **no `twi` at all** (w-hash R3). I register the general form —
**no nonzero integer literal divisor produces any `twi`, for any value, either
sign, either signedness, `/` or `%`** — and I register that I will go hunting
for the cell that breaks it. The named suspects, because they are where the
guard would still be *semantically* live:

* `a % -1` and `a / -1` — divisor `-1` is exactly the `INT_MIN` overflow case
  the `twi 5` predicate exists for;
* `a % INT_MIN`, `a / INT_MIN`;
* `a % 0` and `a / 0` — literal zero, UB, and the one cell where a
  zero-divisor trap would be *correct* to emit unconditionally;
* a divisor that is a `const int` initialised to 0, and one initialised to −1,
  reached through a variable rather than a token — a different IL production
  for the same value.

*This repo has five recorded single-cell traps. If this grid comes back clean on
every literal I will say so and name which literals I probed, not report "no
trap".*

### R7 — a `#pragma`-free, one-block, two-formal div/mod leaf is shippable

`int P(int a,int b){return a%b;}` and its `/`, `unsigned`, and literal-divisor
siblings are a **constant** schedule with at most one immediate field, exactly
as `ptr_walk_loop` is. I claim I can ship a lowering that is byte-exact against
real `c2.dll` over the full cross product of the axes I grade, with everything
else refusing.

### R8 — and it converts **zero** TUs

TU match stays at **10**. A per-function leaf widening cannot close a whole-obj
conjunction on its own (trap 8, board #250), and no frontier TU is one div/mod
leaf away.

*Registered as a loss-if-I-am-lucky. I want this to lose.*

### R9 — the census number w-hash left unscored

w-hash's R9 (*"a `%`-by-variable leaf is worth a non-zero number of emitted
functions"*) went **UNSCORED** because the lowering was not shipped. I ship it
and take the number: **per-function census moves by ≥ 1,000 and emitted census
by ≥ 100.** A concrete threshold so it can miss.

### R10 — the gate moves only by what I add

`gate.sh` grows by exactly **18 verdicts per fixture**. The sweep's **96**
ungraded and the cross's **388** ungraded are byte-identical to the baseline.
`fnbyte-differs` stays **0**. Workspace target count stays **27** or grows —
never shrinks (a truncated run reports fewer passes *and* fewer targets).

### R11 — the must-fail mutation is RUN, not described

I will move the shipped `twi 6` by exactly one slot and show the fixture turns
from `match` into a live **`mismatch`** against real `c2.dll`, with a control
fixture that stays green. A guard nobody has seen fail is not known to work.

---

## 2. Method commitments

* **Grade, don't generate.** Every count reported is the count the *oracle*
  returned, printed by the script, never the count of cells I wrote down.
  Capture failures are counted and named separately from refusals.
* **Register fields, not mnemonics.** Every row is dumped with full operands.
  w-hash's §5.1 table is a mnemonic multiset and the discriminator may live in
  the allocation; a mnemonic table cannot see that.
* **Two anchor controls on every run**, in w-hash's pattern: `plain-add` must be
  `add ; blr`, and `s-mod-var` must reproduce `Sort.cpp`'s own eight words
  **including their register fields**.
* **Additive refusal.** Any new recognizer refuses everything it has not graded.
  `Some(false)` is the only reading acted on.
* If the discriminator does not fall, the rung **names what is undetermined and
  what witness would settle it** and ships the lowering only for the placements
  that are constants of a graded class. A transcription honestly labelled beats
  a schedule rule fitted to two placements.
