// w-fix GRID-3 cell k20_perm_chain

void h(int a, int b) {}
void g1(int a, int b) { h(b, a); }
void f(int a, int b) { g1(b, a); }

// The per-cell positive control: a callee this TU does not define. `?anchor`
// must keep exactly one REL24 at BOTH flag settings or the cell is not graded.
void ext_anchor();
void anchor() { ext_anchor(); }
