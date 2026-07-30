// A framed function sharing a TU with **comparison leaves whose label stride is
// 1** — the classes the counter gate used to refuse wholesale.
//
// The `$M`/`$T` numbers of a framed function are seeded from `.gl` and advanced
// once per preceding function, so a neighbour whose stride the emitter models
// wrongly gives this file's `$M` labels values that still link and are wrong.
// The gate used to key on "is this a comparison leaf" and refuse them all.
// Measured over the whole 60-point grid of relation × literal × signedness
// (`docs/OBJ_GY_SHAPES.md` §3.6), the comparison stride is **not** uniform:
//
//   ==, !=              1   every literal, both signednesses
//   unsigned operand    1   every relation, every literal
//   signed <  / >= vs 0 1
//   everything else     3
//
// Every leaf here is from the 1 block, so `F` and `F2`'s labels must come out
// exactly as if these were ordinary integer leaves. The 3 block still refuses
// and is the negative half — `wfr_cmp_stride_neg.cpp`, its own TU, because a
// refused sibling makes a whole file emit nothing and grade nothing
// (`docs/GAPS.md` §6).

int g(int);

int lt0(int x) { return x < 0; }
int ge0(int x) { return x >= 0; }
int eq0(int x) { return x == 0; }
int ne0(int x) { return x != 0; }
int eqk(int x) { return x == 5; }
int nek(int x) { return x != -5; }
int eqmax(int x) { return x == 32767; }
int ult(unsigned x) { return x < 5u; }
int uge(unsigned x) { return x >= 5u; }
int ugt(unsigned x) { return x > 5u; }
int ule(unsigned x) { return x <= 5u; }

int F(int a) { return g(a) + 1; }
int F2(int a, int b) { return g(b) + 2; }
