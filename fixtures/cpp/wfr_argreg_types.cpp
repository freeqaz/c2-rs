// The framed call's argument register, **shifted by a leading parameter's
// type** rather than by parameter count — the other half of `wfr_argreg.cpp`.
//
// On this ABI a `float`, a `double` and a `long long` each consume exactly one
// GPR slot (the registers are 64-bit and an FP argument still reserves its GPR),
// so `int f(double x, int a) { return g(a) + 1; }` has `a` in r4 and c2 emits
// `or r3,r4,r4` — measured, not assumed: the FP-argument-numbering bug in
// `docs/GAPS.md` §6 (#6) was exactly a case where the two files are numbered
// independently, and this one is the case where they are not.
//
// An 8-byte POD aggregate is one GPR too and is included for the same reason.
// A *wider* aggregate takes more than one and is refused upstream
// (`param-multi-reg`, `il_param_aggr_neg.cpp`) — the framed path inherits that
// gate rather than repeating it, which is why `A3` does not appear here.

int g(int);

int fflt(float x, int a) { return g(a) + 1; }
int fdbl(double x, int a) { return g(a) + 1; }
int fll(long long x, int a) { return g(a) + 1; }
int fptr(int *p, int a) { return g(a) + 1; }
int fchar(char c, int a) { return g(a) + 1; }
int f2ptr(int *p, int *q, int a) { return g(a) + 2; }

struct A2 {
    int x, y;
};
int faggr(A2 v, int a) { return g(a) + 1; }
