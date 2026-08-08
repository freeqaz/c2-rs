// N3 — two guards, call in FORMAL order (the control: no permutation).
void g(void *, void *);
int f(void *p, void *q) { if (p == 0) return 5; if (q == 0) return 11; g(p, q); return 0; }
