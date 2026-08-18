struct S { int m(); };
bool f(S* p, S* q) { return p->m() == q->m(); }
