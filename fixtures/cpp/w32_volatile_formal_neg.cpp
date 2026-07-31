// **Negative** — a `volatile`-qualified formal that the body READS. Every
// function here must be refused (`0/N in class`).
//
// `docs/rungs/2026-07-31-volatile-formal.md`. This is the **thirteenth live
// wrong-bytes emit** and it was pre-existing on mainline across five shapes at
// once — the straight-line leaf, the integer tail call, the framed call, the
// discarded statement call and the multi-argument permutation — plus the pointer
// getter and the pointer identity leaf. Found by this rung's neighbour grid, in
// the FP class, and the FP class turned out to be the smallest part of it.
//
// A `volatile` parameter is a volatile **object**, so c2 homes the incoming
// argument register in the frame and reads it back from memory at every use.
// Read off reference objs (`/O1 /GS- /c`):
//
//   int   v3(int x, volatile int y)      { return y; }
//       stw r4,124(r1) ; lwz r3,124(r1) ; blr        (port: `mr r3,r4 ; blr`)
//   float v11(float x, volatile float y) { return gf(y); }
//       mflr r12 · stw r12,-8(r1) · stwu r1,-96(r1)
//       d041007c  stfs f2,124(r1)      <- homed
//       c021007c  lfs  f1,124(r1)      <- read back
//       4bffffed  bl ?gf               <- and therefore NOT a tail call
//       addi r1,r1,96 · lwz r12,-8(r1) · mtlr r12 · blr
//       (port: `fmr f1,f2 ; b ?gf` — `Port=Mismatch @ offset 2`, the section
//        count, because the reference obj has a `.pdata` the port never emitted)
//
// ## The position is load-bearing, and that is measured
//
// The volatile bit (`0x10` on the TYPE tag) appears at three positions and costs
// something at exactly one of them:
//
//   int f(volatile int y)      { return y; }    b9 <y> 96 41 …   REFUSED (spills)
//   int f(int* volatile p)     { return *p; }   b9 <p> 96 43 …   REFUSED (spills)
//   int f(volatile S* p)       { return p->i; } b9 <p> 86 43 …   in class, byte-exact
//                                               30     96 41 …   — the POINTER is not
//   struct S { volatile int i; };                                 volatile, and the
//   int f(S* p)                { return p->i; } 30     96 41 …    load-through is one
//                                                                 `lwz` either way
//
// So the gate is on the `B9` operand LOAD and **not** on `eat_int_like`,
// `eat_value_type` or the `27`/`30` designator readers. `fixtures/cpp/
// w32_volatile_free.cpp` is the positive half and holds those free cases;
// without it this file would license refusing the qualifier everywhere, which
// costs coverage for nothing.
//
// `const` (bit `0x20`) is free everywhere and is untouched. It is the *pair*
// that makes this a measurement rather than a guess: `const float y` and
// `volatile float y` differ in one bit of one byte and in a whole stack frame.

struct S { int i; float f; };
int    gi(int);
int    gi2(int, int);
float  gf(float);
double gd(double);
void   gv(int);

int    v1(int x, volatile int y)          { return gi(y); }        // int tail call
int    v2(volatile int y)                 { return gi(y); }
int    v3(int x, volatile int y)          { return y; }            // straight-line leaf
int    v4(volatile int y)                 { return y + 1; }
int    v5(int x, volatile int y)          { return x + y; }
int    v8(int x, volatile int y)          { return gi(y) + 1; }    // framed call
void   v9(int x, volatile int y)          { gv(y); }               // statement call
int    v10(volatile int x, int y)         { return gi2(y, x); }    // permutation
float  v11(float x, volatile float y)     { return gf(y); }        // FP tail call
float  v12(float x, volatile float y)     { return y; }
double v15(double x, volatile double y)   { return gd(y); }
float  v16(float x, volatile double y)    { return gf(y); }        // and its narrowing
int    v17(int x, int* volatile p)        { return *p; }           // pointer getter
int*   v18(int x, int* volatile p)        { return p; }            // pointer identity
