// **Positive** — the single-argument floating-point tail call. Every function
// here must emit, and the whole obj must be byte-exact.
//
// `docs/rungs/2026-07-31-fp-tail.md`, and the register rule is
// `docs/CODEGEN_FP_ARGS.md` §0/§1 — this file does not re-derive it, it consumes
// it in the position §5 ranked as the largest measured item left (85,231 whole
// bodies, of which 59,095 are the single-argument half this rung is).
//
// The body is `return g(x);` or `g(x);` where `x` is an FP formal: at most one
// instruction plus the branch, no frame, and `_fltused` in the symbol table.
//
// ## Every word below is read off a reference obj (`/O1 /GS- /c`)
//
//   float  a1(float a)                    { return g1f(a); }  48000000  b g1f
//   float  a2(float a, float b)           { return g1f(b); }  fc201090  fmr f1,f2
//   float  a3(float a, float b, float c)  { return g1f(c); }  fc201890  fmr f1,f3
//   float  a4(int k, float b)             { return g1f(b); }  (nothing)  b g1f
//   float  a5(float a, int k, float b)    { return g1f(b); }  fc201090  fmr f1,f2
//   double a8(float a, double b)          { return g1d(b); }  fc201090  fmr f1,f2
//   double w1(float a)                    { return g1d(a); }  (nothing)  b g1d
//   float  n1(double a)                   { return g1f(a); }  fc200818  frsp f1,f1
//   float  n2(double a, double b)         { return g1f(b); }  fc201018  frsp f1,f2
//   int    b1(float a, float b)           { return gif(b); }  fc201090  fmr f1,f2
//   void   b2(float a, float b)           { gvf(b); }         fc201090  fmr f1,f2
//   float  C::m(float a, float b) const   { return g1f(b); }  fc201090  fmr f1,f2
//
// ## The facts this file separates, which no other fixture can
//
// **`a4` / `a5` / `a6` — the FP file is numbered over the FP parameters ALONE.**
// A positional model puts `a4`'s argument in f2 and emits an `fmr` c2 does not,
// and puts `a5`'s in f3. `a6` has two non-FP leaders and still emits nothing.
//
// **`n1` / `n2` — the narrowing is FUSED with the move.** `double`→`float` at
// the argument boundary is a real `frsp`, and `n2` is the single word
// `fc201018` (`frsp f1,f2`) — NOT `fmr f1,f2 ; frsp f1,f1`. A port that emitted
// the move first would be wrong by one instruction on every narrowing call from
// anything but f1. `w1` / `w2` are its twin in the other direction, where the
// same `2C <TYPE> 00` costs nothing at all: one field, two facts, and only an
// instruction separates them (`docs/GAPS.md` §6).
//
// **`b1` / `b2` — the result class is not this rung's business.** An `int`
// return and a discarded `void` call emit the identical `fmr` and branch: the
// callee's value is already in the register the caller returns in, whichever
// file that is.
//
// **`C::m` — `this` takes r3 and displaces nothing in the FP file**, so the
// member function is byte-identical to its free twin `a2`. That is the same
// witness `docs/CODEGEN_FP_ARGS.md` §1 has for the leaf class, in the position
// where the GPR file is actually occupied.
//
// **`a9` / `a8` — the FP numbering is width-agnostic.** `double, float, double`
// is f1, f2, f3; a "double takes two FPRs" rule (true of some other PowerPC
// ABIs) puts `a9`'s argument in f4.

float  g1f(float);
double g1d(double);
int    gif(float);
void   gvf(float);

float  a1(float a)                        { return g1f(a); }
float  a2(float a, float b)               { return g1f(b); }
float  a3(float a, float b, float c)      { return g1f(c); }
float  a4(int k, float b)                 { return g1f(b); }
float  a5(float a, int k, float b)        { return g1f(b); }
float  a6(int j, int k, float b)          { return g1f(b); }
double a7(double a, double b)             { return g1d(b); }
double a8(float a, double b)              { return g1d(b); }
double a9(double a, float b, double c)    { return g1d(c); }

double w1(float a)                        { return g1d(a); }
double w2(float a, float b)               { return g1d(b); }
float  n1(double a)                       { return g1f(a); }
float  n2(double a, double b)             { return g1f(b); }

int    b1(float a, float b)               { return gif(b); }
void   b2(float a, float b)               { gvf(b); }

struct C { float m(float a, float b) const; };
float  C::m(float a, float b) const       { return g1f(b); }
