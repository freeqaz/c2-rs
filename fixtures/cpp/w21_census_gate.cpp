// W21 — the census/gate boundary, POSITIVE side (roadmap #44).
//
// Acceptance is supposed to live in the IL parser so that `IlBundle::
// function_census` and `PortC2` cannot disagree about what is in class. Three
// gates had leaked into codegen instead, and the census counted functions the
// port refuses (`docs/IL_CALL_IN_EXPR.md` §24.7). This file holds the shapes on
// the ACCEPTED side of each of those three boundaries, so that moving the gate
// is shown to have cost nothing it should not have.
//
// It must census N/N and be `Port=Match`; its sibling `w21_census_gate_neg.cpp`
// must census 0/N. A fixture whose census is not N/N grades nothing — the port
// emits an obj only when every function in the TU is in class (`docs/GAPS.md`
// §6), which is why the two halves are two files.

// --- the serial accumulator chain, at exactly the depth one scratch covers ---
// `a + b*c` reaches operand-stack depth 3 and is in the negative file; putting
// the multiply FIRST keeps the walk at depth 2 throughout.
int mul_then_add(int a, int b, int c) { return a * b + c; }
int mul_then_sub(int a, int b, int c) { return a * b - c; }
int add_chain4(int a, int b, int c, int d) { return a + b + c + d; }

// --- the depth-2 tree, the one deeper shape that IS characterized ------------
// Two scratches, r11/r10, with the `+`-root swap. N1 (a `*` root over a `*`
// child) and N2 (an additive root over an additive child) are rewrites and live
// in the negative file.
int tree_mul_over_add(int a, int b, int c, int d) { return (a + b) * (c + d); }
int tree_add_over_mul(int a, int b, int c, int d) { return (a * b) + (c * d); }

// --- comparison leaves whose literal the difference spine can encode ---------
int u_eq_small(unsigned a) { return a == 5; }
int u_ne_small(unsigned a) { return a != 32767; }
int s_eq_neg(int a) { return a == -32767; }
// The carry spines never negate the literal, so a large unsigned is fine here.
int u_gt_big(unsigned a) { return a > 4294967291u; }
