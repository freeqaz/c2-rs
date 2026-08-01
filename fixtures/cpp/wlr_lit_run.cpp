// WLR — the **one-value literal store run**: a store run every statement of
// which stores the *same* literal, lowered as ONE materialization hoisted to the
// top of the body followed by the stores in source order.
//
// W38 (`w38_store_run.cpp`) gave the store leaf a statement list but refused any
// literal value in a run of more than one, and it was right to: with two or more
// **distinct** literal values c2 hoists the `li`s, allocates r11/r10/r9 by a rule
// these captures do not determine, and reorders the stores around them. Four
// allocation rules were fitted to the grid in `w38_store_run_neg.cpp`'s `n_lit*`
// family and each is refuted by another member of it.
//
// With exactly **one** distinct value the whole question disappears — one
// materialization, one register, one live range, nothing to allocate and nothing
// to schedule — and the stores come back in source order at every length,
// every width, every base and both optimization modes. Every word below was read
// off the reference obj before any of this was written (`/O1` and `/Ox`
// byte-identical):
//
//   void A3(S* s){ s->a=7; …6 stores… }      39600007 91630000 91630004 … x6
//   void B1(S* s){ s->a=9; s->b=9; s->c=9; } 39600009 91630000 91630004 91630008
//   void A6(S* s){ s->a=0; s->b=0; }         39600000 91630000 91630004
//   void C1(W* w){ w->c=1; w->h=1; w->i=1; } 39600001 99630000 b1630002 91630004
//                                            ^ MIXED widths, one `li`
//   void C3(W* w){ c=0; h=0; i=0; q=0; }     39600000 99630000 b1630002 91630004
//                                            f9630008
//   void C4(S* s,S* t){s->a=3;t->b=3;s->c=3;} 39600003 91630000 91640004 91630008
//                                            ^ TWO base registers, source order
//   void C5(S* s){ s->a=100000; s->b=100000;} 3d600001 616b86a0 91630000 91630004
//                                            ^ a WIDE literal: the lis+ori pair is
//                                              hoisted WHOLE, ahead of every store
//   void C6(T* t){ t->r.a=4; t->r.b=4;
//                  t->z=4; }                 39600004 91630000 91630004 91630008
//                                            ^ nested sub-object offsets (W35's walk)
//
// The negative boundary is `wlr_lit_run_neg.cpp`, which carries the refuted
// allocation grid itself.

struct S { int a; int b; int c; int d; int e; int f; int g; };
struct W { char c; short h; int i; long long q; };
struct R { int a; int b; };
struct T { R r; int z; };

// The plain one-value run at lengths 2, 3, 5 and 6.
void a2(S* s) { s->a = 9; s->b = 9; }
void a3(S* s) { s->a = 9; s->b = 9; s->c = 9; }
void a5(S* s) { s->a = 9; s->b = 9; s->c = 9; s->d = 9; s->e = 9; }
void a6(S* s) { s->a = 7; s->b = 7; s->c = 7; s->d = 7; s->e = 7; s->f = 7; }

// Zero, the value a zero-initializing constructor writes, and the reason this
// class is worth anything on a real workload.
void z2(S* s) { s->a = 0; s->b = 0; }
void z3(S* s) { s->a = 0; s->b = 0; s->c = 0; }

// A negative value that still fits `li`'s 16-bit signed field.
void n3(S* s) { s->a = -1; s->b = -1; s->c = -1; }

// SOURCE order, not offset order — the second statement writes the lower member.
void r2(S* s) { s->b = 9; s->a = 9; }

// MIXED widths out of one register: the value is materialized once and each
// statement picks its own `stb`/`sth`/`stw`/`std`.
void w3(W* w) { w->c = 1; w->h = 1; w->i = 1; }
void w4(W* w) { w->c = 0; w->h = 0; w->i = 0; w->q = 0; }

// Two different base pointers. c2 keeps both stores (they may alias) and does
// not reorder, because with one value there is nothing to schedule around.
void b3(S* s, S* t) { s->a = 3; t->b = 3; s->c = 3; }

// A WIDE literal: `emit_load_imm`'s `lis`+`ori` pair, hoisted whole.
void k2(S* s) { s->a = 100000; s->b = 100000; }

// …and the wide literal whose LOW half is zero, where c2 emits `lis` ALONE and
// no `ori`. `emit_load_imm` emitted the redundant `ori r11,r11,0` for as long as
// it had existed — a live wrong-bytes emit, found by this rung's sweep fragment
// (`scripts/sweep.d/84-lit-run.py` case 1's value axis) and reproduced on the
// pre-WLR tree at run length ONE, so it is not this rung's defect. The single
// store is here beside the run because the run is what made the sweep look.
//   s->a = 65536;   3d600001              lis r11,1
//   s->a = 131072;  3d600002              lis r11,2
//   s->a = 65535;   3d600000 616bffff     the high half is emitted even when ZERO
void kz1(S* s) { s->a = 65536; }
void kz2(S* s) { s->a = 65536; s->b = 65536; }
void kz3(S* s) { s->a = 131072; s->b = 131072; s->c = 131072; }
void kn2(S* s) { s->a = 65535; s->b = 65535; }

// Nested sub-object offsets — W35's shared offset-add walk under the run.
void o3(T* t) { t->r.a = 4; t->r.b = 4; t->z = 4; }

// A braced sub-scope between the statements, as W38 admits for formals.
void s2brace(S* s) { s->a = 5; { s->b = 5; } }

// The `return *this` / `return this` tails, which cost nothing (`this` is
// already in r3 and a store writes no register), and the CONSTRUCTOR form — the
// dominant spelling on the workload, where the implicit result sits after the
// `29` rather than ahead of the `3A`.
struct C {
    int a; int b; int c;
    C();
    C(int);
    C& zero();
    C* pzero();
    void vzero();
};
C::C() { a = 0; b = 0; c = 0; }
C::C(int) { a = 1; b = 1; }
C& C::zero() { a = 0; b = 0; return *this; }
C* C::pzero() { a = 0; b = 0; return this; }
void C::vzero() { a = 0; b = 0; c = 0; }

// The same free return in a FREE function, where the first formal is not
// `this`: the rule is the formal's POSITION.
S* sret(S* s) { s->a = 6; s->b = 6; return s; }

// An inherited member (intrinsic 2117 `base-member-addr`) inside the run.
struct Bs { int b0; int b1; };
struct Dv : Bs {
    int d0;
    void zd();
};
void Dv::zd() { b0 = 0; d0 = 0; }
