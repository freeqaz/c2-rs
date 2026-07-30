// W13a — the floating-point PARAMETER boundary (negative fixture).
//
// Every function here MUST be out of class, and the file must never mismatch.
// It pins the two rules that make an FP leaf's parameter *index* usable as its
// FP-register number, both of which were **live wrong-bytes emits** on mainline
// when this file was written (docs/GAPS.md §6, the "two facts sharing one
// field" family — this is its fifth and sixth instance):
//
//  1. `float_leaf_text` maps parameter `n` to `f(n+1)`. The FP file is numbered
//     over the FP parameters *alone*, so any non-FP parameter ahead of a float
//     breaks the identity:
//
//        float mixfp(int a, float b, float c) { return b * c; }
//          c2:   ec2100b2   fmuls f1,f1,f2
//          port: (before)   fmuls f1,f2,f3      <- WRONG, on mainline
//
//     `w13_fabi.cpp` states the rule in a comment and even carries `fp_skip`,
//     but that TU has an out-of-class function in it, so the port never emitted
//     it and the whole-TU gate hid the bug. It reproduces the moment the same
//     body is alone in a translation unit.
//
//  2. A bare `return <FP parameter>` whose parameter is not the first is an
//     `fmr f1,fN`, not nothing:
//
//        float fp_pass2(float a, float b) { return b; }
//          c2:   fmr f1,f2 ; blr
//          port: (before)  blr                  <- WRONG, on mainline
//
//     The integer class has gated exactly this shape since it was written
//     (`straight_line_out_of_class_ctx`'s bare-non-first-formal clause); the FP
//     class never got the same gate. "A locator nobody consults is not shared."
//
// Both are refused by one rule in `try_parse_float_leaf`: **every formal must
// appear as an FP operand of the body**. Each such operand carries the FP type
// in its own `B9` LOAD, so the formals list holding nothing else is what proves
// the index is the register number. It over-refuses the leaf with an unused FP
// parameter (`fp_unused` below) — that body's emission genuinely cannot be
// decided from `.ex`, since whether `a` is a float or an int changes where `b`
// lands and the body never mentions `a`. `.sy` records each formal's type kind
// and would decide it; reading it is a rung, not a tidy-up.
//
// Freestanding, include-free, leaf-only. Compiled by `c2rs bench` and by every
// `scripts/mode_lane.sh` lane.

// ---- 1. a non-FP parameter ahead of the FP ones -----------------------------

float  mix_i_ff(int a, float b, float c)        { return b * c; }
float  mix_i_ff_add(int a, float b, float c)    { return b + c; }
double mix_i_dd(int a, double b, double c)      { return b + c; }
float  mix_p_ff(int *p, float b, float c)       { return b - c; }
float  mix_ii_ff(int a, int b, float c, float d){ return c * d; }
float  mix_f_i_f(float a, int b, float c)       { return a * c; }
double mix_d_i_d(double a, int b, double c)     { return a / c; }
float  mix_c_ff(char a, float b, float c)       { return b * c; }
float  mix_ll_ff(long long a, float b, float c) { return b * c; }
float  mix_i_f(int a, float b)                  { return b / b; }

// ---- 2. a bare return of a non-first FP parameter ---------------------------

float  fp_pass2(float a, float b)               { return b; }
double dp_pass2(double a, double b)             { return b; }
double dp_pass3(double a, double b, double c)   { return c; }
float  fp_pass_mix(int a, float b)              { return b; }
float  fp_nine(float a, float b, float c, float d, float e,
               float f, float g, float h, float i) { return i; }

// ---- 3. an unused FP parameter: undecidable from `.ex`, so refused ----------
//
// `a` is never mentioned in the body, so nothing in `.ex` says whether it takes
// an FP register. It does here, which puts `b` in f2 and makes this an `fmr`;
// had `a` been an `int`, `b` would be f1 and the body would be a bare `blr`.
// The two spellings are byte-identical in the body region.

float  fp_unused(float a, float b)              { return b * b; }
float  fp_unused2(float a, float b, float c)    { return b * c; }

// ---- 4. a member function: `this` takes a GPR and is never an FP operand ----

struct C {
    float mf(float x) const;
    double md(double x, double y) const;
};
float  C::mf(float x) const                     { return x * x; }
double C::md(double x, double y) const          { return x + y; }
