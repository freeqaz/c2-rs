// w-empty GRID cell c21_ret_plus1

int g(int a) { return a + 1; }
int f(int a) { return g(a); }

// The per-cell positive control: a callee this TU does not define. `?anchor`
// must keep exactly one REL24 at BOTH flag settings or the cell is not graded.
void ext_anchor();
void anchor() { ext_anchor(); }
