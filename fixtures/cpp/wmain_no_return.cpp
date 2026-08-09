// The `.sy` binding of a function that **never assigns its exit label**.
//
// `.sy`'s per-function blocks are keyed by the `.ex` segment's exit-label token,
// and `func::sy::ex_exit_label` used to require the token be named TWICE — once
// by `3A <tok>` (the assignment) and once by `29 <tok>` (the return) — because
// one channel cannot tell an opcode from a byte inside another opcode's operand.
//
// A non-`void` function with **no `return` statement** never emits the
// assignment: there is no value to assign. Its token is named once, the segment
// does not bind, `SyView::UNKNOWN` is handed to the body, and every formal width
// is `param-width-undetermined`. That is `src/Main.cpp` — the highest-worth
// frontier row on `w-band`'s ranking — and `WB_EH_FINDINGS.md` §6 files the
// refusal as **R1, "`c2-il` formals header"**. The key is right and the location
// is not: that TU's `.sy` is 80 bytes, parses to EOF, holds exactly one block,
// and declares `argc` and `argv` at four bytes each. Its `.ex` holds **zero**
// `3A` bytes.
//
// The cells below are one clause apart, and the pair that matters is
// `no_return_call` / `with_return`: identical bodies, one `return` statement
// between them, and only the first has no `3A`. `void_falls_off` is the cell
// that kills the obvious wrong reading — "a function that falls off the end has
// no assignment" — because a `void` function assigns its exit label anyway, so
// the discriminator is the **return type**, not the fall-through.
//
// MEASURED, workload flags, base binary vs this one (`work/w-main/cells.md`):
//
//   cell             base key                        this tree
//   no_return_call   param-width-undetermined:mid ->  call-ref-0x54
//   no_return_tail   param-width-undetermined:mid ->  call-ref-0x54
//   with_return      call-ref-0xB9                ->  call-ref-0xB9   (unchanged)
//   void_falls_off   call-ref-0xB9                ->  call-ref-0xB9   (unchanged)
//   chain_control    in class                     ->  in class        (unchanged)
//
// **This rung admits a BINDING, not an emission.** Every body it newly binds
// still refuses at a later key, because a function with no return value has no
// value for the statement layer to carry to `54 02`. The evidence that the
// binding happened is that the census key MOVES OFF the `param-*` family — and,
// in `wmain_no_return_neg.cpp`, that a *different* clause of the same
// formals check becomes reachable for the first time.

void sink(int, int);
int gz(int);

// THE CLASS — non-`void`, two formals, no `return` statement, so no `3A`.
int no_return_call(int a, int b) { sink(b, a); }

// The same with the call in tail position and its result discarded.
int no_return_tail(int a, int b) { gz(b); }

// SEPARATING CONTROL 1 — one `return` statement away from `no_return_call`, and
// that statement is the `3A`. Bound at base; must be unchanged.
int with_return(int a, int b) { sink(b, a); return a; }

// SEPARATING CONTROL 2 — falls off the end like the class, but `void`, and a
// `void` function assigns its exit label anyway. Bound at base; must be
// unchanged. Without this cell "no return statement" and "no return type" are
// the same observation.
void void_falls_off(int a, int b) { sink(b, a); }

// NEUTRALITY CONTROL — in class at base and at tip. The arm must not disturb a
// segment that corroborates normally.
int chain_control(int a, int b) { return a + b; }
