// w-empty GRID cell c05_empty_loop

void g(int a) { for (int i = 0; i < a; ++i) {} }
void f(int a) { g(a); }

// The per-cell positive control: a callee this TU does not define. `?anchor`
// must keep exactly one REL24 at BOTH flag settings or the cell is not graded.
void ext_anchor();
void anchor() { ext_anchor(); }
