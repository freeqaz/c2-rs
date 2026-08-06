// w-fix GRID-3 cell k14_arg_chain

void h(int a) {}
void g1(int a) { h(a + 1); }
void f(int a) { g1(a * 2); }

// The per-cell positive control: a callee this TU does not define. `?anchor`
// must keep exactly one REL24 at BOTH flag settings or the cell is not graded.
void ext_anchor();
void anchor() { ext_anchor(); }
