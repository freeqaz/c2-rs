// w-fix GRID-3 cell m6_defined_after

void h();
void g2();
void g1();
void f() { g1(); }
void g1() { g2(); }
void g2() { h(); }
void h() {}

// The per-cell positive control: a callee this TU does not define. `?anchor`
// must keep exactly one REL24 at BOTH flag settings or the cell is not graded.
void ext_anchor();
void anchor() { ext_anchor(); }
