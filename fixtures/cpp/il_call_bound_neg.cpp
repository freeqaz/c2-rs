// **Negative** — the call-bound-to-a-local form (`int z = g(…); return z;`) at
// exactly the two points where its own copy of the argument validation had
// drifted away from the direct `return g(…)` form's. Both were live before the
// two copies became one (`c2_il::func::body::shapes::tail_call_shape`).
//
// ? `noncanon` — a WRONG-BYTES emit, not a gap. c2 canonicalizes the leaves of a
//   commutative argument expression, so `g(a + b)` and `g(b + a)` are the *same*
//   obj:
//
//     ?f@@YAHHH@Z   7c632214  add r3,r3,r4
//                   4bfffffc  b   ?g@@YAHH@Z
//
//   The port, handed the operand stream in source order, emitted
//   `add r3,r4,r3`. The direct form `return g(b + a);` has refused on
//   `leaves_ascending` since the reassociation rule was measured
//   (`il_reassoc.cpp`); this form never asked, and `c2rs diff` read
//   `Port=Match` for `a + b` beside `Port=Mismatch @ 537` for `b + a` — two
//   lines of C++ that differ by one transposition.
//
// ? `outer` — a PANIC. `g2(a, c)` passes a formal *past* the argument count, so
//   the permutation vector `[0, 2]` indexes its own 2-entry `seen` array out of
//   bounds: `c2rs census` died with `index out of bounds: the len is 2 but the
//   index is 2`. The direct form got the `call-arg-outer-formal` gate when that
//   was found (`docs/GAPS.md` §6); this copy did not. The CLI must degrade
//   cleanly, never panic.
//
// The positive halves are `il_expr_call_value.cpp` (the bound-to-a-local form
// this file is the negative of) and `il_reassoc.cpp` (the canonicalization
// rule). Decode is all-or-nothing per TU, so the whole file must refuse.

int g(int);
int g2(int, int);

int noncanon(int a, int b) { int z = g(b + a); return z; }
int outer(int a, int b, int c) { int z = g2(a, c); return z; }
