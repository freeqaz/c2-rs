// M3 = M2 + the UNUSED third formal and the unsigned result type.
void g(void *, void *, int);
unsigned f(void *p, void *q, unsigned r) { if (p == 0) return 5; if (q == 0) return 11; g(q, p, 72); return 0; }
