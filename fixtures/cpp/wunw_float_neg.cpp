// W-UNW-1 (negative): a floating-point leaf sharing a TU with a framed
// function. Both functions are individually in class — `c2rs census` grades
// this 2/2 — and the port must still REFUSE, because the compiler label
// counter that names `$M`/`$T` is consumed by every function and a float leaf
// consumes **2** slots where every class the port emits consumes 1. Emitting
// would put `$M(n)` one below the reference's number on a body whose every
// other byte is right: an obj that links and is wrong.
//
// Measured, not assumed: `float L(float,float){return a*b;}` ahead of the
// framed probe moves its first label by 2, `double` likewise, and each pooled
// FP constant by a further 2 (`docs/OBJ_GY_SHAPES.md` §3.4).
int g(int);
float fl(float a, float b) { return a * b; }
int f(int a) { return g(a) + 1; }
