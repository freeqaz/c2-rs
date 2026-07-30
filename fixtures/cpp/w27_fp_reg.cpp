// **Positive** — the floating-point argument register file, numbered over the
// FP parameters ALONE. Every function here must emit, and the whole obj must be
// byte-exact.
//
// `docs/CODEGEN_FP_ARGS.md` §1, `docs/ABI_EDGES.md` §2. Two numberings run over
// one parameter list and neither is the formal's index:
//
//   * an FP parameter takes `f<j>` where `j` counts the **FP parameters alone**;
//   * every other scalar takes `r<2 + slot>`, and an FP parameter still consumes
//     its slot, so the GPR numbering *does* count it.
//
// They disagree in opposite directions, and the corpus could not see either,
// because every FP fixture that existed had a parameter list that was uniformly
// `float` or uniformly `double` — where the index, the FP number and the slot
// are all the same integer.
//
// ## The two live wrong-bytes emits this file exists to keep closed
//
// `GAPS.md` §6 (6) and (7), both on mainline, both with four green mode lanes
// and a green sweep over them:
//
//   float mixfp(int a, float b, float c) { return b*c; }
//       emitted  fmuls f1,f2,f3          c2 emits  fmuls f1,f1,f2   (ec2100b2)
//   float f1_2(float a, float b)         { return b; }
//       emitted  *nothing*               c2 emits  fmr f1,f2 ; blr  (fc201090)
//
// The first is a formal's *index* standing in for its FP register number; the
// second is the integer identity's own out-of-class gate missing from the other
// register file. `w13_fabi.cpp` **states the first rule in a comment and carries
// the failing case**, and graded nothing for months, because it holds an
// out-of-class sibling and the port emits an obj only when every function in a TU
// is in class. Hence this file: one TU, every function in class, and
// `c2rs census fixtures/cpp/w27_fp_reg.cpp` must read N/N.
//
// ## What each function discriminates — every line is a captured word
//
// `f1_1` / `f1_2` / `f1_3` — the FP identity at each position in a uniform list.
//   `blr`, `fmr f1,f2`, `fmr f1,f3`. This is the axis the old corpus had.
//
// `d1_2` — the same in `double`. `fmr` is primary **63 whatever the width**
//   (`fc201090`, byte-identical to `f1_2`); there is no `fmrs`, though the A-form
//   arithmetic really does switch to primary 59 for single precision.
//
// `m_1` … `m_4` — a non-FP formal ahead of, between and doubled before the FP
//   ones. All four emit what the *FP* count says and not what the index says:
//   `m_1(int,float)` is a bare `blr` because the `float` is FP parameter 1, and
//   `m_3(float,int,float)` is `fmr f1,f2` because the `int` in the middle
//   advances the slot and not the FP file. Under the index rule `m_1` would need
//   `fmr f1,f2` and `m_3` `fmr f1,f3`, so either one of these fails it.
//
// `w_1` / `w_2` — the two widths interleaved in one list. The FP numbering is
//   width-agnostic: `double a, float b` puts them in f1, f2 exactly as two
//   `float`s would. (`int t8(double,float,double)` passing all three is a bare
//   branch, `docs/CODEGEN_FP_ARGS.md` §1.)
//
// `mixfp` / `mixfp2` / `mixfp3` — the recorded mis-emit itself, and the two
//   neighbours that vary how many non-FP formals precede and where. All three
//   emit their operands as f1,f2.
//
// `unused` / `unused2` — an FP parameter the body never loads. It still occupies
//   its register and still advances the count, which is why the old
//   `params.len() != seen.len()` gate (correct, and worth 1,005 functions on the
//   workload) can be dropped rather than merely narrowed.
//
// `S::m1` / `S::m2` — a member function. `this` takes r3 and displaces **nothing**
//   in the FP file, so these are byte-identical to their free-function twins.

float  f1_1(float a)                          { return a; }
float  f1_2(float a, float b)                 { return b; }
float  f1_3(float a, float b, float c)        { return c; }
double d1_2(double a, double b)               { return b; }

float  m_1(int k, float a)                    { return a; }
float  m_2(int k, float a, float b)           { return b; }
float  m_3(float a, int k, float b)           { return b; }
float  m_4(int i, int j, float a, float b)    { return b; }

float  w_1(double a, float b)                 { return b; }
double w_2(float a, double b)                 { return b; }

float  mixfp(int a, float b, float c)         { return b*c; }
float  mixfp2(int a, int b, float c, float d) { return c-d; }
float  mixfp3(float a, int b, float c)        { return a+c; }

float  unused(float a, float b, float c)      { return a+b; }
float  unused2(float a, float b)              { return a; }

struct S { float m1(float a, float b); float m2(int k, float a, float b); };
float S::m1(float a, float b)                 { return b; }
float S::m2(int k, float a, float b)          { return a*b; }

// ---- promoted from `w13_fparam_neg.cpp` -------------------------------------
//
// Every function below was a NEGATIVE — refused by the `params.len() !=
// seen.len()` gate that closed the two mis-emits above — and each is now emitted
// and byte-exact. They are kept verbatim rather than rewritten, because a case
// that moves from a negative fixture to a positive one is the only direct
// evidence that a rung took what its refusal cost.

float  mix_i_ff(int a, float b, float c)        { return b * c; }
float  mix_i_ff_add(int a, float b, float c)    { return b + c; }
double mix_i_dd(int a, double b, double c)      { return b + c; }
float  mix_p_ff(int *p, float b, float c)       { return b - c; }
float  mix_ii_ff(int a, int b, float c, float d){ return c * d; }
float  mix_f_i_f(float a, int b, float c)       { return a * c; }
double mix_d_i_d(double a, int b, double c)     { return a / c; }
float  mix_c_ff(char a, float b, float c)       { return b * c; }
float  mix_ll_ff(long long a, float b, float c) { return b * c; }

float  fp_pass2(float a, float b)               { return b; }
double dp_pass2(double a, double b)             { return b; }
double dp_pass3(double a, double b, double c)   { return c; }
float  fp_pass_mix(int a, float b)              { return b; }
// The ninth FP parameter — f9, and the last one before the 13-register gate.
float  fp_nine(float a, float b, float c, float d, float e,
               float f, float g, float h, float i) { return i; }
float  fp_unused2(float a, float b, float c)    { return b * c; }

struct C { double md(double x, double y) const; };
double C::md(double x, double y) const          { return x + y; }
