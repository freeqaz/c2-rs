// **Negative** — the FP tail call's boundary. Every function here must be
// refused (`0/N in class`), and every one of them is refused because a *capture*
// shows it emits something this rung does not model — not because it looked
// hard.
//
// `docs/rungs/2026-07-31-fp-tail.md`. Read off reference objs (`/O1 /GS- /c`):
//
//   float r2(double a) { return g1d(a); }   FRAMED  bl ?g1d ; frsp f1,f1 ; epilogue
//   double r1(float a) { return g1f(a); }   free (a bare `b`) — but see below
//   float x1(int a)    { return g1f(a); }   FRAMED  extsw/std/lfd/fcfid/frsp ; bl
//   int   x2(float a)  { return g1i(a); }   FRAMED  fctiwz/stfd/lwz ; bl
//   float c1(float a, float b) { return g1f(a + b); }  fadds f1,f1,f2 ; b
//   float c2(float a)  { return g1f(a * a); }          fmuls f1,f1,f1 ; b
//   float l1()         { return g1f(1.5f); }  lis/lfs through an .rdata COMDAT ; b
//   float p1(float a, float b) { return g2f(b, a); }   the two-file permutation
//   float k1(float a)  { return g1f(a) + 1.0f; }       an FP post-op
//
// **`r1` is the interesting refusal**, and it is the one that is *not* a
// mis-emit risk but a missing field. `4C 2C <TYPE> 00 41` is a conversion applied
// to the call's RESULT, and `r1` (widening) really is a bare `b` while `r2`
// (narrowing) is a whole frame. The IL spells both identically; what separates
// them is the CALL token's own return TYPE, which this rung's recognizer does not
// read. So both refuse, `r1` costs coverage and `r2` would have cost bytes, and
// the rung doc's handoff sizes the field.
//
// `x1`/`x2` are the pair that says the two register files do not connect for
// free: an `int`→`float` conversion round-trips through the stack in both
// directions (`fcfid` one way, `fctiwz` the other) and is a frame, so the rung
// requires the LOAD's `.ex` type to be FP *and* the `.sy` formal to be
// `ArgClass::Fp` — two channels on one fact.
//
// `c1`/`c2` are emittable in principle — a computed FP argument is the W13 float
// leaf's own selector with f1 as the destination — and are refused because that
// is a different lowering with its own contraction and constant gates. `l1` costs
// an `.rdata` COMDAT plus a REFHI/REFLO pair, which `/Gy` refuses anyway.
//
// `g1` is the argument that is not a formal at all (a global): the value is not
// in any argument register, so there is nothing to move.

float  g1f(float);
double g1d(double);
int    g1i(int);
float  g2f(float, float);

double r1(float a)                     { return g1f(a); }
float  r2(double a)                    { return g1d(a); }
float  x1(int a)                       { return g1f(a); }
int    x2(float a)                     { return g1i(a); }
float  c1(float a, float b)            { return g1f(a + b); }
float  c2(float a)                     { return g1f(a * a); }
float  l1()                            { return g1f(1.5f); }
float  p1(float a, float b)            { return g2f(b, a); }
float  k1(float a)                     { return g1f(a) + 1.0f; }
float  gv;
float  g1(float a)                     { return g1f(gv); }
