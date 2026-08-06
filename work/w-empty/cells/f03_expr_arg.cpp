// w-empty GRID cell f03_expr_arg

void g(int a) {}
void f(int a) { g(a + 1); }

// The per-cell positive control: a callee this TU does not define. `?anchor`
// must keep exactly one REL24 at BOTH flag settings or the cell is not graded.
void ext_anchor();
void anchor() { ext_anchor(); }
