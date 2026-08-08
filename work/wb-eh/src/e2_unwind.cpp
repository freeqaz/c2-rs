// E2 — an unwind (destructor) funclet only, no catch.
struct S { S(); ~S(); int m; };
int g(int);
int f(int a){ S s; return g(a)+s.m; }
