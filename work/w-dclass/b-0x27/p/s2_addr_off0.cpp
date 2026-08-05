struct S { unsigned a; unsigned b; };
unsigned *f(S *p) { return &p->a; }
