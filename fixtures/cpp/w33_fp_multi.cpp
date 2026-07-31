// **Positive** — the multi-argument floating-point tail call, W33. Every
// function here must emit, and the whole obj must be byte-exact.
//
// `docs/rungs/2026-07-31-fp-multiarg.md`. This is the other half of the family
// `docs/rungs/2026-07-31-fp-tail.md` opened: `return g(x1, …, xn);` where every
// argument is a floating-point formal, the whole body, no frame. The
// **all-FP-argument** restriction is the whole reason the rung is shippable —
// a call that also passes a GPR argument can need moves in *both* register
// files, and their schedules interleave on a rule `docs/CODEGEN_FP_ARGS.md`
// §1.1 records as uncharacterized (`w33_fp_multi_neg.cpp` holds one).
//
// ## Every word below is read off a reference obj (`/O1 /GS- /c`)
//
//   float id2(float a,float b)            { return g2f(a,b); }  (nothing)  b g2f
//   float sw2(float a,float b)            { return g2f(b,a); }  fmr f0,f2 ; fmr f2,f1 ; fmr f1,f0
//   float rt3(float a,float b,float c)    { return g3f(b,c,a); }
//                                            fmr f0,f2 ; fmr f2,f3 ; fmr f3,f1 ; fmr f1,f0
//   float u4 (a,b,c,d)                    { return g4f(b,c,d,a); }
//                                            fmr f0,f2 ; fmr f2,f3 ; fmr f3,f4 ; fmr f4,f1 ; fmr f1,f0
//
// ## The facts this file separates, which no other fixture can
//
// **`gap1` / `gap2` — the FP file is numbered over the FP parameters ALONE, and
// the *destinations* are numbered over the FP ARGUMENTS alone.** A non-FP formal
// before or between the FP ones moves neither numbering, so both emit the same
// three words `sw2` does. A positional model names f2/f3 and is wrong by two
// instructions.
//
// **`u4` / `u5` — a cycle of length four and five, with one scratch.** The GPR
// file's rung stops at a three-element cycle because past three c2 hoists a
// *second* temp (`c2_core::codegen::permute_args_text`). That boundary is not a
// property of the length: it is a property of the number of **local minima** of
// the cycle, and `u4`/`u5` are the unimodal ones, which stay at one scratch and
// are byte-exact. Measured over the complete n = 2…5 grid,
// `scripts/gt_fpperm.py --pure --model`. Their two-minima neighbours are in the
// negative fixture.
//
// **`w2` — the `float`→`double` widening is free INSIDE a permutation too.** The
// FPR already holds double, so the widening argument's move is a plain `fmr` and
// the cycle is unchanged. Its narrowing twin is refused: see the neg fixture,
// where three narrowings in one 3-cycle change the schedule outright.
//
// **`vd2` / `i2` — the result class is not this rung's business**, exactly as it
// was not the single-argument rung's: a discarded `void` call and an `int`-
// returning one emit the identical permutation.
//
// **`C::m` — `this` takes r3 and displaces nothing in the FP file.** This is the
// case the *integer* multi-argument rung cannot reach at all: its `arg_sources`
// indexes the formals list with `this` at index 0, so every member function with
// two or more arguments trips `call-arg-outer-formal`. Indexing the FP file
// instead makes it free.
//
// **`d2` — the numbering is width-agnostic**, so a `double` swap is byte-
// identical to the `float` one (`fmr` is primary 63 at either width).

float  g2f(float, float);
float  g3f(float, float, float);
float  g4f(float, float, float, float);
float  g5f(float, float, float, float, float);
double g2d(double, double);
int    gi2(float, float);
void   gv2(float, float);

float  id2(float a, float b)                        { return g2f(a, b); }
float  id3(float a, float b, float c)               { return g3f(a, b, c); }
float  sw2(float a, float b)                        { return g2f(b, a); }
float  rt3(float a, float b, float c)               { return g3f(b, c, a); }
float  rt3b(float a, float b, float c)              { return g3f(c, a, b); }
float  fix3(float a, float b, float c)              { return g3f(a, c, b); }
float  u4(float a, float b, float c, float d)       { return g4f(b, c, d, a); }
float  u5(float a, float b, float c, float d, float e) { return g5f(b, c, d, e, a); }

float  gap1(int k, float a, float b)                { return g2f(b, a); }
float  gap2(float a, int k, float b)                { return g2f(b, a); }
float  gap3(int j, int k, float a, float b, float c){ return g3f(b, c, a); }

double d2(double a, double b)                       { return g2d(b, a); }
double w2(float a, float b)                         { return g2d(b, a); }
double w3(double a, float b, double c)              { return g2d(b, a); }

int    i2(float a, float b)                         { return gi2(b, a); }
void   vd2(float a, float b)                        { gv2(b, a); }

struct C { float m(float a, float b) const; float n(float a, float b, float c); };
float  C::m(float a, float b) const                 { return g2f(b, a); }
float  C::n(float a, float b, float c)              { return g3f(b, c, a); }
