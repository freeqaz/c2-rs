// **Negative** — the neighbours of the Class A many-call class
// (`mvp_call_seq.cpp`), each one step outside it and each refused in the IL
// parser so the census and the emission gate cannot disagree.
//
// ? `then_expr` — a formal read by an EXPRESSION after a call. Class B saves
//   formals that a later CALL reads; a formal read by the tail expression is a
//   different lowering (the value comes back out of r31 into an arithmetic
//   chain) and is not captured, so it stays refused here.
//
// ? `lit_multi` — a multi-argument call whose arguments are literals. c2 emits
//   them in **descending destination order** (`li r4,2 ; li r3,1`), which is
//   measured but is a different lowering from the permutation form the
//   multi-argument path models, and mixing a formal with a literal is not
//   captured at all. Refused as `call-arg-computed` rather than guessed.
//
// Decode is all-or-nothing per TU, so the whole file must refuse.

// The two rows that used to live here — `void f(int a,int b){ a1(a); a1(b); }`
// and `void f(int a){ a1(a); a1(a); }` — are **Class B and now in class**; they
// moved to `mvp_call_seq_b.cpp`, where they are graded rather than merely
// refused. A negative fixture whose rows have been admitted proves nothing about
// the gate, so leaving them here would have been decoration.

extern void v0();
extern void a1(int);
extern void a2(int, int);

int then_expr(int a) { v0(); return a + 1; }
void lit_multi() { a2(1, 2); v0(); }
