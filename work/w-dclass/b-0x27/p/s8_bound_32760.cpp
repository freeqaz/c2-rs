struct S { unsigned pad[8190]; unsigned b; };
unsigned *f(S *p) { return &p->b; }
