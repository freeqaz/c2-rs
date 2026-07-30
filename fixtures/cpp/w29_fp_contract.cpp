// **Positive** — `#pragma fp_contract(off)`, which is a per-function
// optimization word and not a code shape. Every function here must emit, and the
// whole obj must be byte-exact.
//
// `docs/OPT_MODE.md` §6. The pragma clears bit `0x4` of the `4F 1F <varint>`
// optimization word — `00200005` becomes `00200001` at `/O1`, `00a00005` becomes
// `00a00001` at `/Ox` — and the port compared the word whole, so every function
// under it was a `codegen-gap` however ordinary its body. On the dc3 workload
// that is **206 functions**, all from two `#pragma` lines
// (`src/system/hamobj/HamRibbon.cpp:139` and `src/system/rndobj/Ribbon.cpp:271`).
//
// ## Why accepting the word cannot turn a refusal into a wrong byte
//
// The bit's only effect on emitted bytes is that a `*` feeding a `+`/`-` stops
// fusing:
//
//   float f(float a,float b,float c){ return a*b+c; }
//     contract on   ec2118ba              fmadds f1,f1,f2,f3
//     contract off  ec0100b2 ec20182a     fmuls f0,f1,f2 ; fadds f1,f0,f3
//
// which is **exactly and only** the set of bodies `try_parse_float_leaf` already
// refuses ("a `*` mixed with `+`/`-` contracts; reject rather than emit two
// instructions where c2 emits one"). Measured rather than argued, at corpus
// scale, in both modes: the whole fixture corpus compiled with and without the
// pragma prepended gives **129 identical `.text` / 1 differing at `/O1`** and
// **145 / 1 at `/Ox`**, and the one differing file is `w13_fneg` both times —
// the fixture whose entire purpose is FMA contraction, and which is refused.
//
// ## What this file is for
//
// It is the *only* graded evidence that the word is an annotation here and not a
// code change: the corpus experiment compares c2 against c2, while this compares
// the **port** against c2 under the pragma. Each body below is a class the port
// emits, chosen so that the file exercises the integer, pointer, store, compare,
// tail-call and floating-point paths — the last of these being the point, since
// FP is the only thing the bit is about.
//
// `n_fma` is deliberately ABSENT. A body that would contract is the one case
// where the bit changes the answer, and it belongs in `w13_fneg.cpp`, which
// carries it and is refused.

#pragma fp_contract(off)

struct S { int i; float f; double d; };

int    c_add(int a, int b)          { return a + b; }
int    c_k(int a)                   { return a + 7; }
int    c_id(int a, int b)           { return b; }
int    c_cmp(int a)                 { return a < 5; }
int*   c_addr(S* s)                 { return &s->i; }
int    c_get(S* s)                  { return s->i; }
void   c_set(S* s, int v)           { s->i = v; }
void   c_empty()                    { }

// The floating-point half — every one of these is a body the bit is *about*,
// and every one of them must be byte-identical with the pragma and without it.
float  f_id(float a)                { return a; }
float  f_snd(float a, float b)      { return b; }
float  f_mul(float a, float b)      { return a * b; }
float  f_add(float a, float b)      { return a + b; }
double d_sub(double a, double b)    { return a - b; }
float  f_mix(int k, float a, float b) { return a * b; }
void   f_store(S* s, float v)       { s->f = v; }
void   d_store(S* s, double v)      { s->d = v; }
