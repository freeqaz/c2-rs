// N1 — the permutation ALONE, no guards.
void g(void *, void *);
int f(void *p, void *q) { g(q, p); return 0; }
