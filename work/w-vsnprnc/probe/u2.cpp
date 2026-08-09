// u2 — the same, C++ linkage. Isolates `extern "C"` from the shape.
int callee5(int, int, int, int, int);
int fwd(int b, int n, int f, int ap) { return callee5(b, n, f, 0, ap); }
