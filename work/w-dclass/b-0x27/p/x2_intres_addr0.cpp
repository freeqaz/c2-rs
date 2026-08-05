struct S { unsigned a; unsigned b; };
int f(S *p) { return (int)&p->a; }
