# w-sched2 — PRE-REGISTRATION

**Committed BEFORE any probe script in `work/w-sched2/` exists.** Check the log:
this file's first commit precedes `schedgrid.py`'s. Every rule below is written
down before the bytes that grade it were produced, and the ones that lose stay on
the page exactly as registered.

Lane: **w-sched2**, worktree off master **`f0fa36c`**.

---

## 0. The question, and why it is not w-rotate again

`docs/rungs/2026-08-05-w-rotate.md` §7.1 names this lane. It closed with

> **Everything except the schedule is now measured.**

and established the causal chain

```
  SCHEDULE  ->  REGISTER NEED  ->  ENTRY FORM
```

with the entry form (`CFG_SHAPE.md` §8.2 L4) re-filed as a *consequence* of the
schedule, the guard form given as a rule at 46/46 (**#771**), and the register
plan outside the body shown to be a **constant** across ten accumulate bodies
(**#775**). The one unstated term is the **interleave** (**#774**), and it is
exactly the free parameter a *lowering* needs and a *transcription* does not.

w-rotate's Grid B produced three cells and **declined to fit a schedule to
three**:

```text
  1-op body   lbzu · op1 · extsb. · bf
  2-op body   lbzu · op1 · extsb. · op2 · bf
  3-op body   op1 · lbzu · op2 · extsb. · op3 · bf
```

Those three cells are **public data** and are the only input to the predictions
below. Nothing in this file is fitted to a cell this lane produced, because at
the time of writing this lane has produced none.

## 0.1 Deconfliction

Lane **w-divmod** owns the leaf `%`/`div` spine and its `twi` placement
discriminator. **No cell in this lane contains a `/` or a `%`.** Where the
sentinel walk's real member (`Sort.cpp`) has one, it is treated as a black box
and not modelled, measured, or edited here.

---

## 1. The vocabulary this lane grades in

For a rotated sentinel walk, the emitted loop **body** is the words from the
loop top up to but excluding the back edge. It is a merge of two sequences:

* the **chain** `c1 … cN` — the accumulate, in data-dependence order;
* the **induction pair** — the update-form load `lbzu` and the **record form**
  (`extsb.` for a signed sentinel, `mr.` for an unsigned one) that sets the CR
  field the back edge tests.

Write

* `L` = the body index of `lbzu`,
* `R` = the body index of the record form,
* `N` = the number of chain words in the body (`bodylen = N + 2`),
* `CHAR` = the register the peel loads and the record form writes — the
  **carried char**,
* `lastread(CHAR)` = the body index of the last chain word that READS `CHAR`.

Every one of these is decided **from the bytes** by the script, never by eye.
`N` is taken from the emitted body, not from the source, so a cell whose source
chain folded is still gradeable — but the script prints intended-N beside
emitted-N and **a disagreement is reported as its own count**, never silently
absorbed.

**Reached and graded are separate counters and both are printed even when
equal. A cell that fails to capture is a FAILURE, not a zero.** Every rate is
`n of m` with `m` beside it.

---

## 2. The registered rules

### P1 — LAT2, the load-use distance

> **`R = L + 2` in every graded cell.** The record form is emitted exactly two
> slots after the `lbzu` it reads: one instruction between them, never zero and
> never two.

From three cells. It is the strongest-looking invariant in w-rotate §6 and it is
the one most likely to be an artefact of three short bodies. **If it loses, the
distance it loses by is the finding**, and this file will say so.

### P2 — WAR, the record form's position

> **`R = lastread(CHAR) + 1`.** The record form is emitted at the first slot at
> which the carried char is dead — immediately after the last chain word that
> reads it.

This is the mechanism P1 is a shadow of, and the two are independent claims:
P1 places `lbzu` relative to the record, P2 places the record relative to the
chain. Both can hold, either can lose alone.

**P2 has a pole the three published cells do not reach.** A body in which
*every* chain op reads the char forces `lastread(CHAR)` to the end of the chain
and predicts `R` at the body's last slot. Grid C is built to hit that pole, and
**if P2 is a base-rate artefact this is where it dies.**

### P3 — LEN, the naive length model, registered so it can lose

> **`L = floor((N-1)/2)`.**

This is the arithmetic that fits w-rotate §6's three cells exactly
(`N=1 -> 0`, `N=2 -> 0`, `N=3 -> 1`) and it is **the reading a lane that had
only the published summary would take.** I expect it to lose at `N >= 4`. It is
registered because *a length model losing is the whole reason the axis has to be
six lengths and not three*, and because if it were to hold, P2 would be the
epicycle rather than the rule.

### P4 — TEMP, the chain's register assignment

> Read from the **end** of the chain, the chain words write
> `r3, r8, CHAR, …` — the last op writes `r3` (the accumulator's home), the
> second-to-last `r8`, the third-to-last `CHAR` itself.
> **For `N >= 4` I register the continuation: the fourth-from-last writes `r7`**,
> i.e. the assignment descends past `r8` skipping `r9`/`r10`, which are live
> (`r9` is `lbzu`'s destination, `r10` the walked pointer).

Fitted to three cells for its first three terms and **extrapolated for its
fourth**, and labelled as such: only the `N >= 4` term is a real prediction.
This rule matters more than it looks — **P2's input `lastread(CHAR)` is an
ALLOCATION fact**, so a lowering can evaluate P2 only if P4 (or something like
it) also states. See §3.

### P5 — FAMILY-BLINDNESS

> **The interleave depends only on the chain's length and its register
> structure, not on the chain's opcodes.** Two cells with the same `N` and the
> same `lastread(CHAR)` get the same `(L, R)` regardless of whether the chain is
> add/sub, multiply-bearing, or shift/rotate.

Can lose, and has a named mechanism if it does: `mulli` is multi-cycle on this
target and `docs/OPT_MODE.md` already records `/Ox` strength-reducing one. **If
P5 loses, the schedule is latency-modelled and a lowering needs c2's own latency
table** — which would be a much dearer finding than the rule.

### P6 — MODE

> Every cell in this lane is `/O1`. **No claim in this file is made at `/Ox`**,
> and `ptr_walk_loop.rs`'s `/O1`-only refusal is not widened.

---

## 3. What would have to be true for a body-parameterized lowering

Stated in advance so that the answer cannot be graded generously after the fact.
The lowering needs, per body, all of:

1. the chain's **opcodes** — instruction selection, already deterministic per IL
   op and not this lane's question;
2. the chain's **register assignment** — **P4**;
3. the **interleave** — **P1 + P2**;
4. the guard form — **#771**, already a rule at 46/46;
5. the entry form — **#773**, already a rule at 8/8 *given* (2);
6. the plan outside the body — **#775**, a constant.

So (2) and (3) are the whole remaining question, and **(2) is load-bearing for
(3)**: P2 reads a register, so P4 losing does not merely cost a rule, it makes
P2 unevaluable from IL and leaves the lowering exactly as far away as
w-rotate left it. **This is registered as the lane's central risk.**

## 3.1 The three worlds, declared with their verdicts

* **World A — both state.** P1, P2 and P4 hold on the held-out grid. The
  sentinel walk has a body-parameterized lowering; this lane owes **#747's
  fixture in its own shape** (§4) with a verified must-fail mutation, and the
  rung says the lowering is reachable.
* **World B — the schedule states, the allocation does not.** P1 and P2 hold,
  P4 loses. The interleave is a **rule given the allocation** and the lowering
  is *not* reachable; the rung publishes the rule, names the allocation as the
  next witness, and ships no widening. **This is a success.**
* **World C — neither states.** Say so plainly, print the residual's shape
  rather than the miss count, and name the next witness. **Also a success.**
* **World D — DECLARED A FAILURE IN ADVANCE.** Transcribing one or two more
  body lengths into `codegen::ptr_walk_loop` and calling it a lowering.
  A second transcription is not a lowering, and **the rung's summary is
  forbidden from rounding one up to the other.** Shipping this would require
  editing this file.

**`cflow-loop` does NOT enter `PORT_CFG_CLASSES` in any world** unless #778 is
closed first by making a restricted claim expressible, and that closure is
itself an instrument change that must be graded. A widening is not a closure.

---

## 4. The fixture this lane owes — #747's shape

w-rotate §7.1 states it in advance and this file repeats it before the grid
exists:

> **Two bodies of DIFFERENT lengths in the same class, in one TU.**

Neither `scripts/expr_sweep.sh` (single-function TUs from an enumerated axis
set) nor `scripts/mode_cross.sh` (that same corpus crossed with the lane
registry) can produce that shape, so **both would grade a wrong schedule
GREEN**: a port that hard-codes one body length matches every case either
instrument can generate. The fixture must carry

* the two differing-length bodies,
* a **separating control** — a TU the mutation must leave `match`, so that a
  red fixture is evidence about the schedule and not about the class,
* a **must-fail mutation**: break the interleave deliberately (place the record
  form at a fixed slot rather than at `lastread(CHAR) + 1`) and show the gate
  turn red on the fixture while the control stays green.

**A fixture that cannot be turned red by breaking the rule it is supposed to
guard is not a fixture.** If World B or C obtains, the port emits no
variable-length body, the mutation has nothing to bite, and **this lane will say
that plainly rather than shipping a fixture whose mutation is untested** — the
"ungraded code path by construction" refusal of w-loop §5.3 reason 2.

---

## 5. Grid shape, registered before it is written

| grid | role | axis |
|---|---|---|
| **A** | **FITTING SET, labelled** | chain length `N` from 1 upward, one operator family. Any refinement of P1–P4 read off Grid A is marked *fitted* and is excluded from every held-out number |
| **B** | **HELD OUT** | the same length axis in the two other operator families — P5's grid |
| **C** | **HELD OUT** | the `lastread(CHAR)` axis: `c`-first, `c`-last, `c`-every bodies at several lengths — **P2's pole**, the one three published cells cannot reach |
| **D** | controls | non-sentinel and degenerate cells that MUST classify out of the family, so that "the family" is a positive check and not an absence |

**Floor, not ceiling: at least 6 chain lengths and at least 3 operator
families.** A grid that reaches only the floor on one axis must say so.

### 5.1 The traps this grid is built against

* **Board #644 — a producer is not one contiguous instruction.** `lis`/`ori`
  pairs split across other instructions and every ORDER/ALLOC grid before #644
  used single-word `li` values without either doc saying so. **This lane probes
  it deliberately**: at least one cell's chain must carry a constant too large
  for `addi`, so that a two-word producer sits inside the chain, and every
  positional rule above must survive it. If P1 or P2 is stated in terms of
  *instruction slots* where c2 thinks in *producers*, this is the cell that says
  so.
* **The single-cell trap.** If a rule is right on `N-1` cells, the `N`th is the
  finding and this lane goes and gets it. Five recorded instances.
* **Absence read as success.** Every bucket is a positive check with a printed
  count; a cell that does not reach the classifier is a failure with its own
  counter.
* **Arity by exhaustion.** If any part of this lane enumerates, it reports the
  **residual's shape**, not the miss count. 13,104 list schedulers gave 89/146
  here and 1,048,576 release-time schedulers gave 196/250, and in both cases the
  answer was outside the searched class and the residual named the mechanism.

---

## 6. Bar

* Real `c2.dll` under wibo + byte-exact obj compare is the sole judge.
* No neutrality / behavior-preserving classifier is added as a gate.
* Additive refusal by construction: `Some(false)` is the only reading acted on.
* `fnbyte-differs` is **0** and a move in it means a wrong emit shipped.
* std only, zero external crates in the workspace; grid scripts here are Python
  and live under `work/w-sched2/`.
* Degrade cleanly to `SKIP: toolchain absent`; never panic.

Baseline to be re-asserted unchanged unless this lane ships a `crates/` change,
taken from w-rotate §10.1: **871 workspace tests / 27 targets**, **18/18 lanes,
4,680 fixture-verdicts**, sweep **16,710 selected / 16,614 graded / 96
ungraded**, cross **81,517 of 81,905 / 388 ungraded**, `status.sh --check` PASS,
`board_audit.sh` 0/0/0. A *changed* number is a failure rather than a curiosity.
