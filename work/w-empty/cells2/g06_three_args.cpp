// w-empty GRID cell g06_three_args

void g(int a, int b, int c) {}
void f(int a, int b, int c) { g(a, b, c); }

// The per-cell positive control: a callee this TU does not define. `?anchor`
// must keep exactly one REL24 at BOTH flag settings or the cell is not graded.
void ext_anchor();
void anchor() { ext_anchor(); }
