// N2 — permutation + ONE guard.
void g(void *, void *);
int f(void *p, void *q) { if (p == 0) return 5; g(q, p); return 0; }
