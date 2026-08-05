struct S { unsigned pad[8191]; unsigned b; };
unsigned *f(S *p) { return &p->b; }
