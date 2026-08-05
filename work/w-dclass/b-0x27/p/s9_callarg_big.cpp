struct S { unsigned pad[8192]; unsigned b; };
void g(unsigned *);
void f(S *p) { g(&p->b); }
