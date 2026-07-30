// **Negative** — floating-point chains c2 canonicalizes or rewrites. All must refuse.
//
// The FP leaf class (W13a) had the same blind spot the integer class did, and for the
// same reason: every positive fixture used distinct operands in ascending order. A
// generated sweep over comparison, float and tail-call shapes found ~40 mis-emits
// here, in a class described as byte-exact.
//
// Two families:
//
// 1. **Commutative canonicalization**, exactly as on the integer side. `b + a` and
//    `b * a` emit their operands in ascending register order, and every permutation
//    of `a + b + c` emits one stream. The port emitted source order.
//
//    Unlike the integer path this is a *refusal*, not a canonicalization: the
//    integer `canonicalize_chain` orders by parameter index, but the FP register
//    model is different in every particular (pool `[f0, f13..f1]`, result forced to
//    f1, no accumulator collapse), so reusing that rewrite here would be an
//    unverified guess. `docs/CODEGEN_W13_FLOAT.md` §2 has the model.
//
// 2. **Division**, which is tighter than the other operators. A single division as
//    the *only* operator is byte-exact and stays accepted — `a / b` and even `b / a`,
//    since division is non-commutative so its operand order is preserved. But:
//
//      a / b / c   two divisions, mismatched
//      a + b / c   a division mixed with anything else, mismatched
//
//    So the gate is "if there is a division, it must be the whole expression".
//    `w13_fops.cpp` holds the single-division positives; a gate that refused all
//    division would refuse those, which is why both files are needed.
//
// Also refused for the same reason as the integer case: `a / 2.0f` is a reciprocal
// multiply, not `fdivs` — see `w13b_ffold.cpp`.

float f_swap_add(float a, float b) { return b + a; }
float f_swap_mul(float a, float b) { return b * a; }
float f_perm_acb(float a, float b, float c) { return a + c + b; }
float f_perm_bac(float a, float b, float c) { return b + a + c; }
float f_perm_cba(float a, float b, float c) { return c + b + a; }
float f_perm_mul(float a, float b, float c) { return a * c * b; }

float f_two_div(float a, float b, float c) { return a / b / c; }
float f_div_add(float a, float b, float c) { return a + b / c; }
float f_mul_div(float a, float b, float c) { return a * b / c; }

double d_swap_add(double a, double b) { return b + a; }
double d_perm_mul(double a, double b, double c) { return a * c * b; }
double d_two_div(double a, double b, double c) { return a / b / c; }
