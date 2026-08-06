// w-fix GRID-3 cell k13_i_result_dropped

int m(int a) { return a; }
void g1(int a) { m(a); }
void f(int a) { g1(a); }

// The per-cell positive control: a callee this TU does not define. `?anchor`
// must keep exactly one REL24 at BOTH flag settings or the cell is not graded.
void ext_anchor();
void anchor() { ext_anchor(); }
