// **Negative** — the neighbours of the Class A many-call class
// (`mvp_call_seq.cpp`), each one step outside it and each refused in the IL
// parser so the census and the emission gate cannot disagree.
//
// ? `live_across`, `live_twice`, `then_expr` — a value is read **after** the
//   first call, so it has to survive one and c2 answers with a callee-saved
//   register. Class A saves nothing, so emitting the Class A frame here would be
//   wrong in the prologue, the epilogue, the `.pdata` `PrologLen`/`FuncLen` and
//   both `$M` labels at once. Measured:
//
//     void f(int a,int b){ g1(a); g2(b); }     52 B, F = 96, ONE saved GPR
//       mflr r12 ; stw r12,-8(r1) ; std r31,-16(r1) ; stwu r1,-96(r1)
//       mr r31,r4 ; bl ?g1 ; mr r3,r31 ; bl ?g2
//       addi r1,r1,96 ; lwz r12,-8(r1) ; mtlr r12 ; ld r31,-16(r1) ; blr
//
//   against Class A's 36 bytes with a 3-word prologue. That is
//   `docs/CODEGEN_FRAMED_CALLS.md` §2.2 Class B, and it needs the liveness and
//   register-assignment answers that are the next rung — `callseq-value-live-
//   across-call`.
//
// ? `lit_multi` — a multi-argument call whose arguments are literals. c2 emits
//   them in **descending destination order** (`li r4,2 ; li r3,1`), which is
//   measured but is a different lowering from the permutation form the
//   multi-argument path models, and mixing a formal with a literal is not
//   captured at all. Refused as `call-arg-computed` rather than guessed.
//
// Decode is all-or-nothing per TU, so the whole file must refuse.

extern void v0();
extern void a1(int);
extern void a2(int, int);

void live_across(int a, int b) { a1(a); a1(b); }
void live_twice(int a) { a1(a); a1(a); }
int then_expr(int a) { v0(); return a + 1; }
void lit_multi() { a2(1, 2); v0(); }
