struct S { unsigned a; unsigned b; };
unsigned f(S *p) { return (unsigned)&p->b; }
