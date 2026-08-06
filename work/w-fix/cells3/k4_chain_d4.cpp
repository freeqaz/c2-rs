// w-fix GRID-3 cell k4_chain_d4

void h() {}
void g3() { h(); }
void g2() { g3(); }
void g1() { g2(); }
void f() { g1(); }

// The per-cell positive control: a callee this TU does not define. `?anchor`
// must keep exactly one REL24 at BOTH flag settings or the cell is not graded.
void ext_anchor();
void anchor() { ext_anchor(); }
