// GRID-S cell s05_lit_setup — setup is a literal
int g(int a) { return a + 1; }
int f() { return g(7); }

void ext_anchor();
void anchor() { ext_anchor(); }
