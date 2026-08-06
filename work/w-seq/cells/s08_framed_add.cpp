// GRID-S cell s08_framed_add — the caller is FRAMED — the `?back@?$vector@…` family's shape
int g(int a) { return a + 1; }
int f(int a) { return g(a) + 2; }

void ext_anchor();
void anchor() { ext_anchor(); }
