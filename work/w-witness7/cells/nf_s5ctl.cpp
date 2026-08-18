struct S { int m(); };
int f(S* p) { return p->m() + 1; }
