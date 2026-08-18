struct S { int m(); };
S* gp;
bool f(S* p) { return p->m() == gp->m(); }
