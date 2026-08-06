// w-empty GRID cell f02_perm

void g(int a, int b) {}
void f(int a, int b) { g(b, a); }

// The per-cell positive control: a callee this TU does not define. `?anchor`
// must keep exactly one REL24 at BOTH flag settings or the cell is not graded.
void ext_anchor();
void anchor() { ext_anchor(); }
