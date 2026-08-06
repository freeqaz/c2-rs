// w-empty GRID cell c04_dead_store

void g(int a) { int x = a; }
void f(int a) { g(a); }

// The per-cell positive control: a callee this TU does not define. `?anchor`
// must keep exactly one REL24 at BOTH flag settings or the cell is not graded.
void ext_anchor();
void anchor() { ext_anchor(); }
