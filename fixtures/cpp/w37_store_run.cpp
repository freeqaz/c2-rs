// W37 — the **store run**: a body that is a *sequence* of store statements, and
// the `return *this` tail that a setter written to chain puts on the end.
//
// The store leaf (W25/W27/W28, `docs/IL_STORE_LEAF.md`) admitted exactly one
// store and then required the body to end. Its sibling — the *assignment*
// statement parser for register locals — has had a statement list since it was
// written. That asymmetry is `docs/GAPS.md` §6's "one fact, one locator" in the
// form that costs coverage rather than correctness: a recognizer that refuses
// more than its sibling emits nothing, so no byte compare and no census/gate
// disagreement can see it. It was worth 54,433 whole bodies on the 878-TU dc3
// workload, every one of them `calls-0`.
//
// **The lowering is one store per statement, in source order** — no scheduling,
// no reordering, no coalescing — and `return *this` costs nothing, because
// `this` is already in r3 and a store writes no register. Every word below was
// read off the reference obj before any of this was written
// (`work/w37/probe/p1.cpp`, `p2.cpp`, `p4.cpp`, at `/O1` and `/Ox`, which agree
// instruction for instruction):
//
//   void s2 (S*,int u,int v)  { a=u; b=v; }        90830000 90a30004
//   void s2r(S*,int u,int v)  { b=v; a=u; }        90a30004 90830000  <- SOURCE order
//   void s2s(S*,int v)        { a=v; b=v; }        90830000 90830004  <- one formal twice
//   void s2t(S* t,S* s,…)     { t->a=u; s->b=v; }  90a30000 90c40004  <- two base registers
//   void W3 (S*,char,short,long long)              9883001c b0a3001e f8c30020
//   void Fp2(S*,float f,double d)                  d0230010 d8430018  <- the other file
//   T& set2(int u,int v){ a=u; b=v; return *this;} 90830000 90a30004  <- epilogue is FREE
//
// The `return *this` half is [`eat_ctor_this_epilogue`], which had exactly ONE
// consumer before this rung — the empty constructor — and is worth **42,238** of
// the 54,433 here. A shared recognizer with one caller is the same defect as a
// private copy, seen from the other side.
//
// The negative boundary is `w37_store_run_neg.cpp`, one case per refusal.

struct S {
    int a; int b; int c; int d; int e; int f; int g;
    char h; short i; long long j; int* k;
    float x; double y;
};

// The plain run: two, three and seven statements, one `stw` each, in order.
void s2 (S* s, int u, int v)                          { s->a = u; s->b = v; }
void s3 (S* s, int u, int v, int w)                   { s->a = u; s->b = v; s->c = w; }
void s7 (S* s, int u,int v,int w,int x,int y,int z,int q)
{ s->a=u; s->b=v; s->c=w; s->d=x; s->e=y; s->f=z; s->g=q; }

// SOURCE order, not offset order — the second statement writes the lower member.
void s2r(S* s, int u, int v)                          { s->b = v; s->a = u; }
// One formal stored twice: two `stw`s out of the same register, no coalescing.
void s2s(S* s, int v)                                 { s->a = v; s->b = v; }
// Two different base pointers: two base registers, and c2 keeps both stores
// because the two may alias at run time.
void s2t(S* t, S* s, int u, int v)                    { t->a = u; s->b = v; }
// The width comes from each statement's own stored type: `stb`/`sth`/`std`.
void s3w(S* s, char h, short i, long long j)          { s->h = h; s->i = i; s->j = j; }
// A pointer member beside an int one — both a bare `stw`.
void s2p(S* s, int* k, int u)                         { s->k = k; s->a = u; }
// The floating-point file, and a run that mixes the two files.
void s2f(S* s, float x, double y)                     { s->x = x; s->y = y; }
void s2m(S* s, int u, float x)                        { s->a = u; s->x = x; }

// The `return *this` tail, at run length 1, 2 and 3 — the shape a chaining
// setter has, and the reason this rung is mostly not about statement lists.
// Defined out of line so each is its own emitted function: an inline member is
// only emitted where it is used, and a wrapper that calls it makes the TU a
// locally-resolved call rather than a store test.
struct T {
    int a; int b; int c;
    T& set1(int u);
    T& set2(int u, int v);
    T& set3(int u, int v, int w);
    T* pset2(int u, int v);
    T& setk();
    void vset2(int u, int v);
};
T& T::set1(int u)               { a = u; return *this; }
T& T::set2(int u, int v)        { a = u; b = v; return *this; }
T& T::set3(int u, int v, int w) { a = u; b = v; c = w; return *this; }
T* T::pset2(int u, int v)       { a = u; b = v; return this; }   // `return this`
T& T::setk()                    { a = 7; return *this; }         // ONE literal store
void T::vset2(int u, int v)     { a = u; b = v; }
// …and the same free return in a FREE function, where the first formal is not
// `this`: the rule is the formal's POSITION, not its name.
S* s2ret(S* s, int u, int v)                { s->a = u; s->b = v; return s; }
int s1ret(int u, S* s, int v)               { s->a = v; return u; }

// The other two designator spellings, in a run: an inherited member (intrinsic
// 2117 `base-member-addr`) and a run of byte-offset adds (W35's shared walk).
struct B { int b0; int b1; };
struct D : B {
    int d0;
    void sd(int u, int v);
    D&   sdr(int u, int v);
};
void D::sd (int u, int v)       { b0 = u; d0 = v; }
D&   D::sdr(int u, int v)       { b1 = u; d0 = v; return *this; }

struct Inner { int p; int q; };
struct Mid   { int pad; Inner in; };
struct Outer { int pad0; Mid mid; };
void o2(Outer* o, int u, int v)             { o->mid.in.p = u; o->mid.in.q = v; }
void o2a(Outer* o, S* s, int u, int v)      { o->mid.in.q = u; s->g = v; }

// A braced sub-scope between the statements — the scopes are consumed at the
// statement boundary, exactly as the assignment parser does it.
void s2brace(S* s, int u, int v)            { s->a = u; { s->b = v; } }


// The CONSTRUCTOR form of the same free return, and the dominant one on the
// workload (42,238 of the 54,433): the implicit result sits AFTER the `29`
// rather than ahead of the `3A`, which is `eat_ctor_this_epilogue` — a
// recognizer that had exactly ONE consumer, the empty constructor, before this
// rung. A member initializer list and an assignment body are the same IL here.
struct C {
    int a; int b; int c;
    C(int u);
    C(int u, int v);
    C(int u, int v, int w);
};
C::C(int u)                     { a = u; }
C::C(int u, int v)              { a = u; b = v; }
C::C(int u, int v, int w)       : a(u), b(v), c(w) { }
