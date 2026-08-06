// GRID-S cell s14_cmp_callee — the callee is a comparison leaf, a different port shape
int g(int a) { return a > 3; }
int f(int a) { return g(a); }

void ext_anchor();
void anchor() { ext_anchor(); }
