// w-fix GRID-3 cell k3_chain_d3

void h() {}
void g2() { h(); }
void g1() { g2(); }
void f() { g1(); }

// The per-cell positive control: a callee this TU does not define. `?anchor`
// must keep exactly one REL24 at BOTH flag settings or the cell is not graded.
void ext_anchor();
void anchor() { ext_anchor(); }
