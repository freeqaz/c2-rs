// w-fix GRID-3 cell k6_stop_d2

void ext();
void h() { ext(); }
void g1() { h(); }
void f() { g1(); }

// The per-cell positive control: a callee this TU does not define. `?anchor`
// must keep exactly one REL24 at BOTH flag settings or the cell is not graded.
void ext_anchor();
void anchor() { ext_anchor(); }
