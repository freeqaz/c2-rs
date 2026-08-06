// GRID-S cell s13_deep_callee — a larger callee — does size change the answer at an empty setup
int g(int a) { return a + 1 + a + 2 + a + 3 + a + 4; }
int f(int a) { return g(a); }

void ext_anchor();
void anchor() { ext_anchor(); }
