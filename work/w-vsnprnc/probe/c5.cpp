// c5 — C++ linkage, identity passthrough, 5 args. Isolates LINKAGE from arity.
int cal5(int, int, int, int, int);
int fwd5(int a, int b, int c, int d, int e) { return cal5(a, b, c, d, e); }
