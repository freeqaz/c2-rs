// GRID-T cell t02_empty_setup_bigger_callee — FIRES — the same at a larger callee: size must not change the answer below S7's bound
int g(int a) { return a + 1 + a + 2 + a + 3 + a + 4; }
int f(int a) { return g(a); }

void ext_anchor();
void anchor() { ext_anchor(); }
