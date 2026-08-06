// GRID-S cell s01_void_tail_no_setup — empty setup, int tail call, passthrough formal
int gv_sink;
int g(int a) { return a + 1; }
int f(int a) { return g(a); }

void ext_anchor();
void anchor() { ext_anchor(); }
