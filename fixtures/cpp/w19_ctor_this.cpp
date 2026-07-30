// W19 — the constructor epilogue: `return this` after the RETURN.
//
// Every function here must be **in class** and the whole obj byte-exact against
// real c2. A constructor's IL body is the empty body every `void f() {}` has,
// plus a value expression wedged between the RETURN and the function tail:
//
//   … 3A <label> 54 02 29 <label>   B9 <this> <TYPE> 41 <TYPE>   4F 12 47 …
//                                   ^^^ the returned `this`
//
// MSVC constructors hand `this` back in r3. It is already there on entry and a
// leaf body cannot have moved it, so the epilogue **emits nothing**: every one
// of these is exactly `4E 80 00 20`, `blr`, the same bytes an empty non-member
// body gets. Read off the reference obj (`/Ox /GS- /c`), eight distinct classes
// in one translation unit, varying arity, member count, member type and file
// position — one sequence, no exceptions:
//
//   struct A { int m;        A(); };      A::A() {}          4e800020  blr
//   struct C { int m,n;      C(); };      C::C() {}          4e800020  blr
//   struct D {               D(); };      D::D() {}          4e800020  blr
//   struct E { int m;        E(int); };   E::E(int a) {}     4e800020  blr
//   struct F { double d;     F(); };      F::F() {}          4e800020  blr
//   struct G { int m;        G(int,int);};G::G(int,int) {}   4e800020  blr
//
// That run is also `docs/GAPS.md` §6's locality tell, run before the row was
// taken: byte-identical sources in one TU emitting **one** sequence means the
// instruction selection is local. The `data-addr` rung was ranked #1 at 21,642
// functions and yielded 0 for want of exactly this check.
//
// The census key this closes is `fn-tail-0xB9` — 29,552 functions, the largest
// call-free row that had been named but never decomposed. 28,717 of them are
// this shape; the other 832 make a call and are in `w19_ctor_this_neg.cpp`,
// where they belong, because a call forces c2 to spill `this` (`mr r31,r3` …
// `mr r3,r31`) and the frame is a different rung.
//
// Two facts are required literally, per §6's "a field that never varied is
// indistinguishable from a constant": the loaded token must be the one
// `parse_this_token` positively bound, and the `B9` operand type must be
// byte-identical to the `41` result type. Across all 29,549 sites on the real
// workload the token was `this` every time — which is precisely why requiring
// it costs nothing and refusing to require it would be a guess.

// ---- the shape itself, defined out of line -------------------------------
struct A { int m; A(); };
A::A() {}

// A second class with the identical body: the locality tell inside the fixture.
struct B { int m; B(); };
B::B() {}

// Member count and member type do not reach the body.
struct C { int m, n; C(); };
C::C() {}

struct D { D(); };
D::D() {}

struct Fd { double d; Fd(); };
Fd::Fd() {}

struct Fa { int arr[8]; Fa(); };
Fa::Fa() {}

struct Fp { int *p; const char *q; Fp(); };
Fp::Fp() {}

// ---- unused parameters, at every arity the register file cares about ------
struct E1 { int m; E1(int); };
E1::E1(int a) {}

struct E2 { int m; E2(int, int); };
E2::E2(int a, int b) {}

struct E4 { int m; E4(int, int, int, int); };
E4::E4(int a, int b, int c, int d) {}

// A pointer and a float parameter: neither displaces `this`, and neither is read.
struct Ep { int m; Ep(const char *, float); };
Ep::Ep(const char *s, float f) {}

// An 8-byte by-value aggregate ahead of a scalar — the width `.sy` has to get
// right for the formals gate (`il_param_aggr_neg.cpp`), riding along unused.
struct Pair { int x, y; };
struct Ea { int m; Ea(Pair, int); };
Ea::Ea(Pair v, int b) {}

// NOTE on what is deliberately absent: a constructor **defined inside the class
// body** and never used (`struct H { H() {} };`) has an `.ex` body that c2 reads
// and does not emit, so the port fails the whole TU closed on the `.gl` binding
// — the standing limitation recorded in `docs/GAPS.md` §6 ("the port has no model
// of which bodies c2 emits"). It is not a gap in this rung and it is not
// exercised here, because a fixture that cannot reach `Match` grades nothing.

// A copy constructor with an empty body: `this` and one reference formal.
struct K { int m; K(); K(const K &); };
K::K() {}
K::K(const K &o) {}

// ---- the control group: empty bodies with NO epilogue ---------------------
//
// A free function and a void member emit the same `blr` and were already in
// class before this rung. They are here so the fixture proves the epilogue is
// the only thing that changed, not that empty bodies work.
void free_empty() {}
struct M { int m; void v() const; static void s(); };
void M::v() const {}
void M::s() {}
