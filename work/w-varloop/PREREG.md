# w-varloop — PRE-REGISTRATION

    Lane:   w-varloop, worktree off master `37858fd`
    Date:   2026-08-05
    Brief:  build the body-parameterized loop lowering w-sched2 measured and
            deliberately did not build.

**Committed BEFORE any grid script in `work/w-varloop/` exists and BEFORE any
file under `crates/` is touched.** Anything added after this file is committed
goes in a dated ADDENDUM below, and each addendum is committed before the grid
or the code it registers.

---

## 0. What this lane is committing to build

A **body-parameterized** lowering of the rotated sentinel walk:

```c
int P(const char* s){ int r = K0; while (*s) { int c = *s; r = <CHAIN>; s++; } return r; }
```

* an IL-side production that yields an **operation list** for `<CHAIN>` — not a
  fixed byte pattern;
* an emit side that consumes that list and applies w-sched2's five rules
  (S1, S2, S3m, S4r/S4n, S5) plus w-rotate's guard-form rule P2;
* refusing, by a **positive** guard, everything outside what has been graded.

`PtrWalkModLoop` (`Sort.cpp`'s `%` class) and `div_mod_leaf` are **not touched**.
`cflow-loop` does **not** enter `PORT_CFG_CLASSES` (#778 blocks it and another
lane owns that).

---

## 1. The worlds, declared in advance

| world | description | verdict |
|---|---|---|
| **A** | A body-parameterized emit lands in `crates/`: one parse production yielding an op list, one emitter applying the rules, accepting **≥ 3 distinct chain lengths** and refusing the rest, byte-exact against real `c2` on every accepted cell. | **SUCCESS** |
| **B** | The parse side lands with an op list; the emit side **refuses**, because a fact the emitter needs was refuted by measurement. The refutation is the result. | **ACCEPTABLE** — report it as such |
| **C** | A **fixed-length transcription of a second body length**, or two transcriptions behind one dispatch, presented as a lowering. | **FAILURE, declared in advance.** Shipping this requires editing this file |
| **D** | Nothing lands and nothing is measured. | **FAILURE** |

---

## 2. Claims. Each of V1–V5 can lose.

### V1 — the TWO-regime PREAMBLE and TAIL (**unmeasured anywhere; this is the one most likely to lose**)

w-sched2's reconstruction rebuilds the **body and the back edge only**. It
hard-codes the walked pointer as `r10` and the accumulator home as `r3` and it
never generates, compares, or grades a single preamble or tail word. A lowering
must emit the whole function, so those words are this lane's to measure.

> **V1.** For the TWO regime the whole function is
> ```text
>   lbz    r11,0(r3)
>   mr     r10,r3
>   li     r3,K0
>   extsb. r11,r11
>   bclr   12,2                    <- P2's GUARDRET: the fall-out block is a bare blr
>   <body: M+2 words, by S1/S2/S3m/S4r/S4n/S5>
>   bf     2,-4*(M+2)
>   blr
> ```
> — exactly `M+9` words, at every accepted `M`, with no word depending on `M`
> other than the back edge's displacement.

### V2 — the SAME-regime preamble is JUMPIN and has NO record form

> **V2.** For the SAME regime the whole function is
> ```text
>   lbz    r11,0(r3)
>   mr     r10,r3
>   li     r3,K0
>   b      +4*(M+2)                <- into the record form, which IS the test
>   <body: M+2 words, record form last>
>   bf     2,-4*(M+2)
>   blr
> ```
> — exactly `M+8` words: one fewer than TWO, because the peeled character and
> the induction load share `r11` and the record form is reached by falling into
> it rather than by a duplicated test.

V2 is derived from w-sched2 §3.5's `entry: b .+24` at `M = 4` and from nothing
else. **One cell. This is the arity at which ten placement rules have already
died in this project** (`leaf_store.rs`'s four, `w-pair` §4's six, P3 in
w-sched2 itself). It is registered because it can lose, and if it loses the lane
takes World A on the TWO regime alone and says so.

### V3 — the whole-obj differential

> **V3.** `port(IL) == c2(IL)` byte-exact (TimeDateStamp zeroed), real `c2.dll`
> under wibo, on **≥ 20 accepted cells spanning ≥ 4 distinct chain lengths**.

### V4 — held out

> **V4.** **≥ 10** of V3's cells are at chain lengths and/or operator families
> **never inspected while the emitter was written**. The held-out set is named
> in an ADDENDUM committed before it is run.

**P3 in w-sched2 fitted three published cells exactly and died at `N = 5`.** The
held-out set must therefore contain a length strictly greater than every length
used to develop the emitter.

### V5 — #747's port-side discharge, which this lane owes

> **V5.** A single TU containing **two loops of different chain lengths** is
> `Port=Match`, and a mutation that fixes the emitted body length turns that
> fixture **red** while every single-loop fixture stays green.

w-sched2 demonstrated this shape and explicitly left its port-side discharge to
"whichever lane builds the lowering" (#792). This is that lane. A fixture whose
mutation is never run is not a fixture.

### V6 — TU match. **I predict +0.**

> **V6.** TU match is **10 at both ends**. This class has no `%`, so it is not
> `Sort.cpp`; `Sort.cpp` is already matched by the transcription.

Registered so that a move cannot be claimed as the lane's point after the fact,
and so that a *non*-move is a registered outcome rather than a disappointment.

### V7 — the controls

> **V7.** `mismatch` is 0 and `fnbyte-differs` is 0 at both ends. Every
> must-refuse cell comes back `vocab-gap` or `codegen-gap` and **never**
> `mismatch`.

---

## 3. The refusal set, stated positively and BEFORE the code

The emitter accepts only when **all** hold. Each is a positive check; `Some(false)`
is the only reading acted on.

1. exactly **one** formal, a width-4 data pointer, at **slot 0** (w-hash: the
   pointer off slot 0 re-plans the whole block, #770 mechanism 11);
2. `/O1` only (`ptr_walk_loop`'s own refusal — `/Ox` emits a different body);
3. accumulator init inside `simm16` (one `li`);
4. stride exactly 1, element type one-byte;
5. every chain word a **single-word producer** — no `lis`/`ori` split (#644);
6. **no** hoisted wide literal (w-sched2 §5.2, 2 of 69 cells);
7. chain ops **commutative** where two-source: `subf` and the rest of
   w-sched2's `NONCOMM` set are refused, because S5 is measured on commutative
   ops only;
8. no `/`, no `%` — those belong to w-divmod and to `PtrWalkModLoop`;
9. `pv` defined (some producer reads the char's value).

**Anything not on this list refuses.** Coverage lost is a number, not an argument.

---

## 4. The must-fail mutations, registered before the emitter exists

This lane **ships an accept path**, so the w-rotate escape ("no mutation, and
here is why") is not available. Each mutation perturbs one rule by one and each
must turn the gate **red**:

| # | mutation | must break |
|---|---|---|
| M1 | S1: `a` becomes `a+1` — one more chain word before the load | the accepted fixtures |
| M2 | S2: the record slot moves by one | the accepted fixtures |
| M3 | S3m: the regime threshold `M >= 4` becomes `M >= 3` | at least one accepted fixture |
| M4 | S4r: `T1`/`T2` exchanged | the accepted fixtures |
| M5 | S5: the commutative operand order reversed | the accepted fixtures |
| **M6** | **the body length is fixed at the first loop's `M`** | **V5's two-loop fixture, and ONLY it** — the single-loop fixtures must stay green, or the mutation has not isolated #747's shape |

M6 is the one that discharges #747. If M6 turns a single-loop fixture red too,
the mutation is not the one claimed and the rung says so.

---

## 5. What "graded" means here

* every rate is `n of m` with `m` printed beside it;
* **reached** and **graded** are separate counters, printed even when equal;
* a cell that fails to capture is a **FAILURE** with its own counter, never a
  zero and never an absence;
* every excluded cell prints its reason and its count;
* the oracle is real `c2.dll` under wibo and a byte-exact obj compare. Nothing
  is graded against the port's own expectation.

---

## 6. Housekeeping registered in advance

* board rows go in **#796–#805** and nowhere else;
* rung at `docs/rungs/2026-08-05-w-varloop.md` + INDEX row;
* `cargo test --workspace --release`, `scripts/gate.sh --jobs 6`,
  `scripts/status.sh --check`, `scripts/board_audit.sh` before claiming done,
  each read from its log rather than from an exit status;
* no generated IL, no objs, no `/home/<user>/…` path is committed.
