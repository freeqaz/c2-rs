// GRID-T cell t09_unlowerable_callee — S6 REFUSES — the port has no body for this callee, so there is nothing to splice
int gsink;
int g(int a) { int t = 0; for (int i = 0; i < a; ++i) t += i * a; gsink = t; return t; }
int f(int a) { return g(a); }

void ext_anchor();
void anchor() { ext_anchor(); }
