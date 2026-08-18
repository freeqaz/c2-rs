struct S { int m(); };
S* gp;
int f(S* p) { return gp->m() + 1; }
