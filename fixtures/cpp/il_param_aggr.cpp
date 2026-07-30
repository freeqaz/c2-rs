// **Positive** — the admitted half of the argument-register ladder, in its own
// translation unit so that it is actually byte-graded.
//
// Its negative twin is `fixtures/cpp/il_param_aggr_neg.cpp`, which pins the
// parameters that occupy more than one GPR. The two must be separate TUs: the port
// emits an obj only when an *entire* TU is in class, so a positive case sharing a
// file with a refused one is never compared against c2 and the fixture grades
// nothing. That is a live hazard in this corpus, not a hypothetical — see the
// header of `il_expr_load_neg.cpp`.
//
// Every parameter here occupies exactly one argument register, so a formal's index
// *is* its register number and the base pointer lands where the leaf expects:
//
//   a1   struct of 1 int    4 B   h in r4
//   a2   struct of 2 ints   8 B   h in r4 — the widest aggregate that still agrees
//   un   union{int;float;}  4 B   `.sy` kind `16`, class nibble 6 like a struct
//   fl   float                    FP scalars reserve one GPR apiece
//   dd   two doubles              h in r5
//   ll   long long          8 B   one GPR on a 64-bit target
//   rf   const P3&          4 B   a reference is a pointer in `.sy` (kind 03)
//   ar   int[4]             4 B   an array parameter decays to a pointer
//   me   `this` + 8-byte struct   this r3, v r4, h r5
//
// `a2`/`ll`/`me` are the discriminating cases: 8 bytes is exactly the boundary, so
// an off-by-one in the width test (`< 8` instead of `<= 8`) refuses them while
// every other case here still passes. `un` is what says the aggregate rule is keyed
// on the kind's class *nibble* and not on the byte `06`.

struct A1 { int a; };
struct A2 { int a, b; };
struct P3 { int a, b, c; };
union U { int i; float f; };

struct H { int mi; };

int a1(A1 v, H* h) { return h->mi; }
int a2(A2 v, H* h) { return h->mi; }
int un(U u, H* h) { return h->mi; }
int fl(float f, H* h) { return h->mi; }
int dd(double x, double y, H* h) { return h->mi; }
int ll(long long q, H* h) { return h->mi; }
int rf(const P3& r, H* h) { return h->mi; }
int ar(int v[4], H* h) { return h->mi; }

struct C {
    int m;
    int me(A2 v, H* h) const;
};

int C::me(A2 v, H* h) const { return h->mi; }
