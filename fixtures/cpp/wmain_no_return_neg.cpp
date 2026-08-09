// c2rs-profile: /Ox /GS- /w14716 /c  # Same class, same reason as wmain_no_return.cpp: all three cells are non-`void` functions with no `return`, which this cl.exe reports as `error C4716` at EVERY optimization level including /Od and the workload's own profile. /w14716 demotes it to the level-1 warning it is documented as, leaving the diagnostic visible.

// The refusals behind `wmain_no_return.cpp`'s binding — three cells, three
// DISTINCT census keys, none of them a `param-width-undetermined`.
//
// The first is the one that earns this file. `neg_multi_reg` takes a 16-byte
// by-value aggregate, which occupies more than one argument register, so
// `SyView::formals_are_one_register_each` must return **`param-multi-reg`** — a
// construct the port genuinely does not lower, as distinct from a fact the
// reader could not read. **That clause is unreachable at base**: the segment has
// no `3A`, so `.sy` never binds, `Formals::Undetermined` is matched first and
// the function reports `param-width-undetermined` like every other unbound one.
// The key moving from the *undetermined* arm to the *multi-reg* arm of the same
// function is the proof that the new binding actually runs, and it is the only
// evidence in this rung that a histogram cannot fake.
//
// That is the pre-armed instrument, and it is armed against the failure this
// board has committed 12 times in 16 lanes: **fence order / clause
// reachability** — a new arm placed behind an existing early return, reported as
// a widening that did not widen. Here the two arms are the two arms of one
// `match`, and the cell separates them.
//
// MEASURED, workload flags, base binary vs this one (`work/w-main/cells.md`):
//
//   cell             base key                        this tree
//   neg_multi_reg    param-width-undetermined:mid ->  param-multi-reg:mid
//   neg_empty        param-width-undetermined:mid ->  body-0x54
//   neg_call_stop    param-width-undetermined:mid ->  call-ref-0x54
//
// The second and third are the statement layer's own stop. A function with no
// return value reaches `54 02` with nothing to carry there, so the body
// recognizer (`body-0x54`) or the call recognizer (`call-ref-0x54`) refuses on
// the scope close itself. **Neither is admitted here and neither should be**:
// this rung binds the `.sy` block, it does not teach the port to emit a function
// whose return register is undefined. Every cell in this file is expected to
// stay out of class.

void sink(int, int);

struct Quad {
    int a, b, c, d;
};

// N1 — the clause that only becomes reachable once the block binds.
int neg_multi_reg(Quad q, int b) { sink(b, q.a); }

// N2 — non-`void`, two formals, empty body: the statement layer stops on the
// scope close.
int neg_empty(int a, int b) { }

// N3 — the call layer's stop, one key away from N2.
int neg_call_stop(int a, int b) { sink(a, b); sink(b, a); }
