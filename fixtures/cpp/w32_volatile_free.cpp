// **Positive** — the `volatile` cases that cost nothing. Every function here
// must emit, and the whole obj must be byte-exact.
//
// `docs/rungs/2026-07-31-volatile-formal.md`. This file is the reason the
// thirteenth mis-emit's fix is a *position* and not a blanket refusal of the
// qualifier: three of the four places a volatile tag appears are free, and a
// gate on the tag alone would have cost every one of these bodies for nothing.
//
// Read off reference objs (`/O1 /GS- /c`):
//
//   int f1(volatile S* p)       { return p->i; }  80630000  lwz r3,0(r3)
//   int f2(S* p)                { return p->i; }  80630000  lwz r3,0(r3)   (member is volatile)
//   int f3(int x, volatile int* p) { return *p; } 80640000  lwz r3,0(r4)
//   int f4(int x, volatile int y)  { return x; }  (nothing)  blr           (never READ)
//   int f5(int x, const int y)     { return gi(y); }  mr r3,r4 ; b ?gi
//
// **`f4` is the discriminator for "read", not "declared".** A volatile formal
// that the body never loads is a bare `blr`: the parameter is a volatile object,
// but with no access there is no access to emit. So the gate is on the LOAD and
// not on the formals list — which is also why it could not be put on the `.sy`
// side, where the qualifier is not even visible without resolving the TU's
// constructed-type table.
//
// **`f1`/`f2`/`f3` are the pointer-to-volatile family**, where the `27`/`30`
// designator positions carry the volatile tag and the emission is one `lwz`
// either way. `f5`/`f6` are the `const` twins of the refused cases, one bit away
// in the tag and free.

struct S { volatile int i; };
struct T { int i; };
int gi(int);
float gf(float);

int   f1(volatile T* p)                { return p->i; }
int   f2(S* p)                         { return p->i; }
int   f3(int x, volatile int* p)       { return *p; }
int   f4(int x, volatile int y)        { return x; }
int   f5(int x, const int y)           { return gi(y); }
float f6(float x, const float y)       { return gf(y); }
int   f7(volatile int y)               { return 7; }
