// W4b2-iv (arg-setup integer tail call): `return g(a + 1)` is a tail call whose
// argument carries a `+1` — computed INTO the argument (before the `55`
// call-end marker), NOT a framed post-op. The reference emits a 5-section leaf
// `addi r3,r3,1 ; b g` (the argument in r3, then a tail branch; REL24 at
// .text+0x4). The port models this arg-setup class (a single argument computed
// into r3 by the leaf arithmetic selector) → Port=Match. Distinct from framed
// `g(a) + 1` (post-op AFTER the `55` marker → 6-section .pdata frame). See
// docs/CODEGEN_PPC_MVP.md (int tail-call family).
int g(int);
int f(int a) { return g(a + 1); }
