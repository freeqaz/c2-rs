// W13a — one binary FP operation per function, float and double side by side.
//
// Pins the opcode pairs: single precision is primary opcode 59 (`fadds`,
// `fsubs`, `fmuls`, `fdivs`), double precision the same XO under primary
// opcode 63 (`fadd`, `fsub`, `fmul`, `fdiv`); unary `-` is `fneg` (opcode 63)
// for BOTH widths. A single-operation leaf writes straight into the result
// register f1 — no temporary is allocated.

float  f_add(float a, float b)    { return a + b; }
float  f_sub(float a, float b)    { return a - b; }
float  f_mul(float a, float b)    { return a * b; }
float  f_div(float a, float b)    { return a / b; }

double d_add(double a, double b)  { return a + b; }
double d_sub(double a, double b)  { return a - b; }
double d_mul(double a, double b)  { return a * b; }
double d_div(double a, double b)  { return a / b; }

float  f_neg(float a)             { return -a; }
double d_neg(double a)            { return -a; }

// Narrowing is an explicit `frsp`; widening is free (no instruction).
float  f_narrow(double a)         { return (float)a; }
double d_widen(float a)           { return a; }

// A mixed-width expression is evaluated in double precision even when both
// leaves are declared float, so the ".s" form is selected by the *expression*
// type, not by the operand declarations.
double d_mixed(float a, double b) { return a + b; }
