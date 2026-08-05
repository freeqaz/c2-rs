struct S { unsigned a; unsigned b; };
void g(unsigned *);
void f(S *p) { g(&p->b); }
