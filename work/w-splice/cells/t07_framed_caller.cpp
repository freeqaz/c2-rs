// GRID-T cell t07_framed_caller — S1 REFUSES — the caller is FRAMED; SPLICE-0 is 0 of 123 there
int g(int a) { return a + 1; }
int f(int a) { return g(a) + 2; }

void ext_anchor();
void anchor() { ext_anchor(); }
