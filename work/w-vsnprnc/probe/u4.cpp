// u4 — pointer/unsigned formals, C++ linkage, literal in a middle slot.
int callee5(char *, unsigned, char *, void *, void *);
int fwd(char *b, unsigned n, char *f, void *ap) { return callee5(b, n, f, 0, ap); }
