struct S { unsigned a; unsigned b; };
void g(unsigned *, unsigned);
void f(S *p, unsigned x) { g(&p->b, x); }
