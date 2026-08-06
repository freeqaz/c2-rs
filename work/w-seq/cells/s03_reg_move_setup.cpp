// GRID-S cell s03_reg_move_setup — setup is a register move — the `?Release@Object@Hmx@@` family
int g(int a) { return a + 1; }
int f(int a, int b) { return g(b); }

void ext_anchor();
void anchor() { ext_anchor(); }
