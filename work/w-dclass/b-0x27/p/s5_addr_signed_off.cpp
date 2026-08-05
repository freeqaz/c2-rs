struct S { unsigned pad[8192]; unsigned b; };
unsigned *f(S *p) { return &p->b; }
