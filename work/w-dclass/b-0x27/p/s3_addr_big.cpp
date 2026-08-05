struct S { unsigned pad[20000]; unsigned b; };
unsigned *f(S *p) { return &p->b; }
