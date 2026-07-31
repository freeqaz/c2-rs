// WSL — a store whose VALUE is an indirect load: `d->a = s->b;`, which is the
// body of every hand-written copy constructor and copy assignment operator.
//
// The store production has admitted a *formal* or an integer *literal* as the
// stored value since W25, and a RUN of such statements plus the `return *this`
// tail since W38. It never admitted a value that is itself read out of memory,
// and that is 92 % of what was left of the store family on the 878-TU dc3
// workload: the ceiling is 11,872 whole bodies, of which only 982 are a single
// statement — the rest are runs, and a run is what a copy assignment is.
//
// **Two instructions, one scratch register, no frame.** Every word below was
// read off the reference obj before any of this was written
// (`work/wsl/probe/p1.cpp`, `p2.cpp`, `p4.cpp` at `/O1 /GS- /c`):
//
//   void c1 (S* d, Q* s) { d->a = s->qb; }   81640004 91630000  lwz r11,4(r4) ; stw r11,0(r3)
//   void c1s(S* d)       { d->a = d->b;  }   81630004 91630000  ONE base register
//   void c1d(int* d,int* s){ *d = *s;    }   81640000 91630000  the bare deref
//   void w_c(W* d, W* s) { d->c = s->c;  }   89640000 99630000  lbz ; stb
//   void w_h(W* d, W* s) { d->h = s->h;  }   a1640002 b1630002  lhz ; sth
//   void w_q(W* d, W* s) { d->q = s->q;  }   e9640008 f9630008  ld  ; std   (both DS-form)
//   void w_f(W* d, W* s) { d->f = s->f;  }   c0040010 d0030010  lfs f0 ; stfs f0
//   void w_g(W* d, W* s) { d->g = s->g;  }   c8040018 d8030018  lfd f0 ; stfd f0
//   T& operator=(const T& r){a=r.a;b=r.b;return *this;}
//                                            81640000 91630000 81640004 91630004  blr
//
// **The cv strip is what makes a copy assignment parse at all.** A copy
// assignment takes `const T&`, so the loaded member is `const int` and the
// member it lands in is plain `int`, and c1xx spells the difference as an
// explicit `2C` between the load and the store — `30 a6 41 86 20 · 2c 86 41 74
// 00 · 32 86 41 74` — which emits nothing. Requiring the two types to be
// byte-identical (the rule the formal-valued path draws) refused every single
// one of them.
//
// **The scratch register is the `/O1` / `/Ox` split, and it is the same one
// `docs/OPT_MODE.md` §3.1 already records for arithmetic chains.** `/O1` reuses
// r11 (and f0) for every statement because each value is dead as soon as it is
// stored; `/Ox` gives every statement its own DESCENDING register — r11, r10,
// r9, … and f0, f13, f12, … — with the two register files counted
// independently. MEASURED over runs of 1..8 crossed with 2..6 pointer
// parameters in both modes (`work/wsl/probe/p6.cpp`).
//
// The run is admitted only while that descent stays **above** every register a
// parameter could hold. Past that point c2 stops descending and starts skipping
// live registers and wrapping back to r11, which needs a liveness model this
// port does not have — see `wsl_store_load_neg.cpp`, which carries the first
// refused length at three different parameter counts.
//
// The negative boundary is `wsl_store_load_neg.cpp`, one case per refusal.

struct Q { int qa, qb, qc, qd; };
struct S { int a, b, c, d, e, f, g; };
struct W { char c; short h; int i; long long q; float f; double g; unsigned u; int* p; };
struct N { int m0; struct In { int x, y; } in; };
struct A { int a0, a1; };
struct B { int b0, b1; };
struct D : A, B { int d0; };

// ---- one statement: both designators, both bases, every width --------------

void c1  (S* d, Q* s)     { d->a = s->qb; }      // two bases, both offsets
void c1z (S* d, Q* s)     { d->a = s->qa; }      // both offsets zero
void c1s (S* d)           { d->a = d->b; }       // ONE base register
void c1d (int* d, int* s) { *d = *s; }           // the bare deref, no offset add
void c1a (int* d, int* s) { d[2] = s[3]; }       // subscripts, `28 00 00`

void w_c (W* d, W* s)     { d->c = s->c; }       // lbz ; stb
void w_h (W* d, W* s)     { d->h = s->h; }       // lhz ; sth
void w_i (W* d, W* s)     { d->i = s->i; }       // lwz ; stw
void w_q (W* d, W* s)     { d->q = s->q; }       // ld  ; std, both DS-form
void w_u (W* d, W* s)     { d->u = s->u; }       // unsigned, the same `stw`
void w_p (W* d, W* s)     { d->p = s->p; }       // a POINTER member, also `lwz`/`stw`
void w_f (W* d, W* s)     { d->f = s->f; }       // lfs f0 ; stfs f0
void w_g (W* d, W* s)     { d->g = s->g; }       // lfd f0 ; stfd f0

// The cv strip, at every width — a `const`/`volatile` source is a `2C` that
// emits nothing. `volatile` here qualifies the POINTEE, not the pointer: a
// volatile *formal* is a frame and is in the negative file.
void cv_i(W* d, const W* s)    { d->i = s->i; }
void cv_c(W* d, const W* s)    { d->c = s->c; }
void cv_g(W* d, const W* s)    { d->g = s->g; }
void cv_v(W* d, volatile W* s) { d->i = s->i; }

// The offset-add RUN on the value side folds into the one displacement, the
// same shared walk the destination designator uses.
void n1(N* d, N* s) { d->m0   = s->in.y; }
void n2(N* d, N* s) { d->in.x = s->in.y; }

// The intrinsic-2117 base-member designator, on either side.
void bm1(D* d, D* s) { d->d0 = s->b1; }
void bm2(D* d, D* s) { d->b0 = s->d0; }

// The base at a later argument position — BOTH register fields move.
void p3(int k0, int k1, S* d, S* s) { d->a = s->a; }

// ---- runs: source order, every length inside the descent -------------------

void r2 (S* d, S* s) { d->a=s->a; d->b=s->b; }
void r3 (S* d, S* s) { d->a=s->a; d->b=s->b; d->c=s->c; }
void r7 (S* d, S* s) { d->a=s->a; d->b=s->b; d->c=s->c; d->d=s->d; d->e=s->e; d->f=s->f; d->g=s->g; }
// SOURCE order, not offset order — the one axis an ascending run cannot show.
void r2r(S* d, S* s) { d->b=s->b; d->a=s->a; }
void r2x(S* d, S* s) { d->a=s->b; d->b=s->a; }   // crossed
void r2s(S* d, S* s) { d->a=s->a; d->b=s->a; }   // one source read twice, NOT CSEd
void r2t(S* d, S* s, S* t) { d->a=s->a; d->b=t->b; }   // two source bases

// Widths interleaved inside one run, and the two register files mixed — which a
// run of loaded values may do and a run of formals may not, because each
// statement is a self-contained pair with no live range to schedule across.
void rw (W* d, W* s) { d->c=s->c; d->h=s->h; d->i=s->i; d->q=s->q; }
void rf (W* d, W* s) { d->f=s->f; d->g=s->g; }
void rm1(W* d, W* s) { d->f=s->f; d->i=s->i; d->q=s->q; }
void rm2(W* d, W* s) { d->i=s->i; d->q=s->q; d->g=s->g; }
void rm3(W* d, W* s) { d->c=s->c; d->f=s->f; d->h=s->h; d->g=s->g; }

// The parameter-count bound, at its last admitted length in each case.
void q3_6(S* d, S* s, S* x0)               { d->a=s->a; d->b=s->b; d->c=s->c; d->d=s->d; d->e=s->e; d->f=s->f; }
void q5_4(S* d, S* s, S* x0, S* x1, S* x2) { d->a=s->a; d->b=s->b; d->c=s->c; d->d=s->d; }

// ---- the tails: void, `return *this`, a constructor's implicit result -------
//
// Declared here and DEFINED out of line, so each is emitted whether or not
// anything in this TU calls it: a forcing helper would have to make a member
// call, and a member call in expression position is a different rung's shape.

struct T {
    int a, b;
    T& operator=(const T& r);
    T* pcopy(const T& r);
    void set(const T& r);
    T(const T& r);
};

// `return *this` — an ordinary value return of the first formal, which is
// already in r3, so the epilogue is a bare `blr`.
T& T::operator=(const T& r) { a = r.a; b = r.b; return *this; }
// `return this` — the same fact spelled as a pointer.
T* T::pcopy(const T& r)     { a = r.a; b = r.b; return this; }
// void.
void T::set(const T& r)     { a = r.a; b = r.b; }
// A CONSTRUCTOR's implicit result, which sits after the `29`
// (`eat_ctor_this_epilogue`) rather than before it — the dominant tail on the
// real workload, and the one a copy constructor wears.
T::T(const T& r)            { a = r.a; b = r.b; }
