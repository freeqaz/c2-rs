// **Positive** — the FP argument file under cv-qualification, typedefs and
// references. Every function here must emit, and the whole obj must be
// byte-exact.
//
// `docs/CODEGEN_FP_ARGS.md` §1, and this file exists for one decision:
// **`SyView::arg_classes` keys on `.sy`'s type KIND (05 = "real"), not on its
// type id.** Everything below is a formal whose kind says `float` while its id
// says something else, or the reverse.
//
// ## Why the id is the wrong key, and why that is not a style preference
//
// A `.sy` formal record is `<tag> <kind> 00 03 04 <size> <b> <flags16> <tid>`.
// For a plain `float` the id is `40` and for a `double` `41` — but a
// **`const float`** is `80 02 10 00 00`, an id in the constructed range that the
// translation unit allocates **for itself**. That is precisely the per-input
// value `docs/GAPS.md` §6 forbids partitioning on, and it is the same failure
// that shattered `expr-load-type-XXXXXX` into 848 shards. Here it would not
// merely mis-rank a histogram: a `const float` parameter is still passed in an
// FPR, so an id gate numbers it as a GPR and shifts **every** argument after it,
// in both register files at once.
//
// The kind is `05` for `float`, `const float`, `volatile float` and a typedef of
// any of them, and the **size** (4 or 8) separates the widths. Neither is per-TU.
//
// ## The reverse direction, which is the sharper test
//
// `rf1(float& a, float b)` is a bare `blr`, not an `fmr`. A reference is a
// **pointer** — `.sy` kind `03` — so it takes r3 and fills no FP register, which
// makes `b` the *first* FP parameter and puts it in f1 already. A model that
// looked at the spelling `float` anywhere in the type, or that keyed on the
// id family, makes `b` the second FP parameter and emits `fmr f1,f2` for a
// function whose whole body is one instruction.
//
// `rf3(float& a, float& b, float c)` doubles it: two references, `c` still f1.
// `rf4(S* s, float& a, float v)` is the same fact in the store production —
// `stfs f1`, not `stfs f2`.
//
// ## Captured
//
//   float cq1(const float a, float b)        { return b; }   fc201090  fmr f1,f2
//   float cq4(int k, const float a, float b) { return b; }   fc201090  fmr f1,f2
//   float td3(int k, Real a, Real b)         { return a*b; } ec2100b2  fmuls f1,f1,f2
//   float rf1(float& a, float b)             { return b; }   4e800020  blr
//   void  td4(S* s, Real v)                  { s->f = v; }   d0230000  stfs f1,0(r3)
//
// A `const float` **operand** — one the body actually loads — carries the
// cv-qualified TYPE `A6 45 <per-TU id>` in `.ex` and is still refused there
// (`expr-load-type-A645`); that is an over-refusal on the *operand* side and is
// a separate, smaller rung from this one, which is about the *parameter list*.
// `cq1` and `cq4` are in class precisely because their const formal is never
// loaded — it only has to be counted.

typedef float Real;
typedef double Dbl;
struct S { float f; double d; };

float  cq1(const float a, float b)        { return b; }
float  cq4(int k, const float a, float b) { return b; }
float  vq1(volatile float a, float b)     { return b; }

float  td1(Real a, Real b)                { return b; }
double td2(Dbl a, Dbl b)                  { return b; }
float  td3(int k, Real a, Real b)         { return a * b; }
void   td4(S* s, Real v)                  { s->f = v; }

float  rf1(float& a, float b)             { return b; }
float  rf3(float& a, float& b, float c)   { return c; }
void   rf4(S* s, float& a, float v)       { s->f = v; }
