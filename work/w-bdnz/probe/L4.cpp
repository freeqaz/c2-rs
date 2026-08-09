int f(int);
// formal order / arity
int swapf(int k, int n)   { int s = 1; for (int i = 0; i < n; ++i) s *= k; return s; }
int three(int n, int k, int j) { int s = 1; for (int i = 0; i < n; ++i) s *= k; return s; }
// init literal range
int init_big(int n, int k)  { int s = 32767; for (int i = 0; i < n; ++i) s *= k; return s; }
int init_over(int n, int k) { int s = 32768; for (int i = 0; i < n; ++i) s *= k; return s; }
int init_neg(int n, int k)  { int s = -32768; for (int i = 0; i < n; ++i) s *= k; return s; }
// counter/bound signedness
unsigned uacc(int n, unsigned k) { unsigned s = 1; for (int i = 0; i < n; ++i) s *= k; return s; }
int ubound(unsigned n, int k) { int s = 1; for (unsigned i = 0; i < n; ++i) s *= k; return s; }
int uctr(int n, int k) { int s = 1; for (unsigned i = 0; i < (unsigned)n; ++i) s *= k; return s; }
// relation
int le(int n, int k)  { int s = 1; for (int i = 0; i <= n; ++i) s *= k; return s; }
int ne(int n, int k)  { int s = 1; for (int i = 0; i != n; ++i) s *= k; return s; }
int down(int n, int k){ int s = 1; for (int i = n; i > 0; --i) s *= k; return s; }
// start != 0
int start3(int n, int k) { int s = 1; for (int i = 3; i < n; ++i) s *= k; return s; }
// NEGATIVES
int n_break(int n, int k)  { int s = 1; for (int i = 0; i < n; ++i) { s *= k; if (s > 100) break; } return s; }
int n_cont(int n, int k)   { int s = 1; for (int i = 0; i < n; ++i) { if (k > 0) continue; s *= k; } return s; }
long long n_i64(int n, int k) { long long s = 1; for (long long i = 0; i < n; ++i) s *= k; return s; }
int n_step2(int n, int k)  { int s = 1; for (int i = 0; i < n; i += 2) s *= k; return s; }
int n_stepv(int n, int k)  { int s = 1; for (int i = 0; i < n; i += k) s *= k; return s; }
int n_bexpr(int n, int k)  { int s = 1; for (int i = 0; i < n / 2 + 3; ++i) s *= k; return s; }
int n_ctru(int n, int k)   { int s = 1; for (int i = 0; i < n; ++i) s *= i; return s; }
int n_call(int n, int k)   { int s = 1; for (int i = 0; i < n; ++i) s *= f(k); return s; }
int n_nest(int n, int k)   { int s = 1; for (int i = 0; i < n; ++i) for (int j = 0; j < k; ++j) s *= k; return s; }
int n_after(int n, int k)  { int s = 1; for (int i = 0; i < n; ++i) s *= k; return s + 7; }
int n_twoop(int n, int k)  { int s = 1; for (int i = 0; i < n; ++i) { s *= k; s -= k; } return s; }
