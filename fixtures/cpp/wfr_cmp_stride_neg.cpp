// **Negative** — the other half of `wfr_cmp_stride.cpp`. Every comparison leaf
// here consumes **3** compiler-label counter slots, so a framed function beside
// one would get `$M`/`$T` numbers low by 2 per neighbour: an obj that links
// perfectly and is wrong in six bytes per label. The whole TU must be
// `NotImplemented`.
//
// The signed relational spine (`<`, `<=`, `>`, `>=` against a non-zero literal,
// plus `>` and `<=` against zero) is the 3 block; `<`/`>=` against zero fold to
// a sign-bit extraction and are 1, which is the discriminating neighbour and
// lives in the positive file. Getting that boundary from one witness is exactly
// the mistake `OBJ_GY_SHAPES.md` §3.4 records — the stride was measured here per
// relation, against a seed read out of `.gl`, not fitted against totals.
//
// A floating-point leaf is 2 (4 with one pooled constant, 6 with two) and is in
// this file for the same reason.

int g(int);

int ltk(int x) { return x < 5; }
int gek(int x) { return x >= 5; }
int gt0(int x) { return x > 0; }
int le0(int x) { return x <= 0; }

int F(int a) { return g(a) + 1; }
