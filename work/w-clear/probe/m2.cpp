// M2 = M1 + a LITERAL third argument (the 0x48 of the memcpy).
void g(void *, void *, int);
int f(void *p, void *q) { if (p == 0) return 5; if (q == 0) return 11; g(q, p, 72); return 0; }
