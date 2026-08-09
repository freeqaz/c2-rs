int gf(int);
int n_break(int n, int k)  { int s = 1; for (int i = 0; i < n; ++i) { s *= k; if (s > 100) break; } return s; }
int n_cont(int n, int k)   { int s = 1; for (int i = 0; i < n; ++i) { if (k > 0) continue; s *= k; } return s; }
long long n_i64(int n, int k) { long long s = 1; for (long long i = 0; i < n; ++i) s *= k; return s; }
int n_step2(int n, int k)  { int s = 1; for (int i = 0; i < n; i += 2) s *= k; return s; }
int n_stepv(int n, int k)  { int s = 1; for (int i = 0; i < n; i += k) s *= k; return s; }
int n_bexpr(int n, int k)  { int s = 1; for (int i = 0; i < n / 2 + 3; ++i) s *= k; return s; }
int n_ctru(int n, int k)   { int s = 1; for (int i = 0; i < n; ++i) s *= i; return s; }
int n_call(int n, int k)   { int s = 1; for (int i = 0; i < n; ++i) s *= gf(k); return s; }
int n_nest(int n, int k)   { int s = 1; for (int i = 0; i < n; ++i) for (int j = 0; j < k; ++j) s *= k; return s; }
int n_swap(int k, int n)   { int s = 1; for (int i = 0; i < n; ++i) s *= k; return s; }
int n_after(int n, int k)  { int s = 1; for (int i = 0; i < n; ++i) s *= k; return s + 7; }
int n_litop(int n)         { int s = 1; for (int i = 0; i < n; ++i) s *= 3; return s; }
int n_addop(int n, int k)  { int s = 0; for (int i = 0; i < n; ++i) s += k; return s; }
int n_divop(int n, int k)  { int s = 1; for (int i = 0; i < n; ++i) s /= k; return s; }
int n_initover(int n, int k) { int s = 32768; for (int i = 0; i < n; ++i) s *= k; return s; }
int n_three(int n, int k, int j) { int s = 1; for (int i = 0; i < n; ++i) s *= k; return s; }
long n_long(int n, long k) { long s = 1; for (int i = 0; i < n; ++i) s *= k; return s; }
unsigned n_uacc(int n, unsigned k) { unsigned s = 1; for (int i = 0; i < n; ++i) s >>= k; return s; }
int n_start3(int n, int k) { int s = 1; for (int i = 3; i < n; ++i) s *= k; return s; }
int n_le(int n, int k)     { int s = 1; for (int i = 0; i <= n; ++i) s *= k; return s; }
int n_ne(int n, int k)     { int s = 1; for (int i = 0; i != n; ++i) s *= k; return s; }
int n_down(int n, int k)   { int s = 1; for (int i = n; i > 0; --i) s *= k; return s; }
int n_dowhile(int n, int k){ int s = 1; int i = 0; do { s *= k; ++i; } while (i < n); return s; }
