struct S { unsigned a; unsigned b; };
void g(unsigned *, unsigned *);
void f(S *p) { g(&p->b, &p->a); }
