// **Positive** — the floating-point store leaf. Every function here must emit,
// and the whole obj must be byte-exact.
//
// `docs/CODEGEN_FP_ARGS.md` §3. `void f(S* s, float v) { s->f = v; }` is one
// `stfs`/`stfd` and a `blr`, and it is the **fourth** consumer of the sub-object
// designator the indirect-load leaf (`lwz`), the address leaf (`addi`) and the
// integer store leaf (`stw`) already share — so it needs no new address decode at
// all, only the FP register file.
//
// MEASURED at **7,984 functions** on the 878-TU workload before it was built (by
// counterfactual, all `calls-0`), which is 8x the `fmr` rung it ships beside.
// `docs/IL_STORE_LEAF.md` §6 recorded it as "measured and not implemented, what
// stops it is the FP argument-register numbering" and §7 (3) ranked the pair
// together; this is that pair.
//
// ## Every word below is read off a reference obj
//
//   void s_f (S* s, float v)          { s->f = v; }        d0230004  stfs f1,4(r3)
//   void s_d (S* s, double v)         { s->d = v; }        d8230008  stfd f1,8(r3)
//   void s_pf(float* p, float v)      { *p = v; }          d0230000  stfs f1,0(r3)
//   void s_two(S* s,float u,float v)  { s->f = v; }        d0430004  stfs f2,4(r3)
//   void s_arg2(int x,S* s,float v)   { s->f = v; }        d0240004  stfs f1,4(r4)
//
// ## The two facts this file separates, which no other fixture can
//
// **`s_two` / `s_twou` / `s_mix`.** The stored value's FP register counts the FP
// parameters *alone*: in `s_two` the value is the second `float` and the store is
// `stfs f2`, in `s_twou` it is the first and the store is `stfs f1`, and in
// `s_mix` an `int` sits between the two `float`s and the answer is still `f2`.
// A model that used the formal's index would put `s_mix`'s value in f3.
//
// **`s_arg2` / `s_arg3`.** The *base* pointer's register is still its index —
// r4 and r5 — even with an FP formal in the list, because an FP parameter fills
// no GPR but does consume its argument slot, and the two effects cancel exactly.
// So the two files are numbered by two different rules over one list, and this
// file has a case where each rule alone would be wrong.
//
// `s_base` reaches an inherited member through intrinsic 2117, `s_e2` through a
// subscript, and `M::set` / `M::set2` are member functions where `this` takes r3
// and displaces nothing in the FP file.
//
// The neighbours that must NOT be admitted — a conversion on the stored value and
// a pooled FP literal — are in `w28_fp_store_neg.cpp`.

struct S { int i; float f; double d; float arr[4]; char c; float g; };
struct B { float bf; };
struct D : B { float df; };

void s_f (S* s, float v)      { s->f = v; }
void s_d (S* s, double v)     { s->d = v; }
void s_g (S* s, float v)      { s->g = v; }
void s_e2(S* s, float v)      { s->arr[2] = v; }
void s_pf(float* p, float v)  { *p = v; }
void s_pd(double* p, double v){ *p = v; }

void s_arg2(int x, S* s, float v)         { s->f = v; }
void s_arg3(int x, int y, S* s, float v)  { s->f = v; }
void s_two (S* s, float u, float v)       { s->f = v; }
void s_twou(S* s, float u, float v)       { s->f = u; }
void s_mix (S* s, float u, int k, float v){ s->f = v; }

struct M { float m; void set(float v); void set2(int k, float v); };
void M::set(float v)          { m = v; }
void M::set2(int k, float v)  { m = v; }

void s_base(D* d, float v)    { d->bf = v; }
