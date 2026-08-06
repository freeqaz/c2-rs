// GRID-S cell s04_arith_setup — setup is arithmetic — does c2 fold the two literals
int g(int a) { return a + 1; }
int f(int a) { return g(a + 1); }

void ext_anchor();
void anchor() { ext_anchor(); }
