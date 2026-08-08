// N4 — three formals, rotated by one.
void g(void *, void *, void *);
int f(void *p, void *q, void *r) { g(q, r, p); return 0; }
