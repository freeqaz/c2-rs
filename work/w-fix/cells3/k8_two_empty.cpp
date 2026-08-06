// w-fix GRID-3 cell k8_two_empty

void h1() {}
void h2() {}
void g1() { h1(); h2(); }
void f() { g1(); }

// The per-cell positive control: a callee this TU does not define. `?anchor`
// must keep exactly one REL24 at BOTH flag settings or the cell is not graded.
void ext_anchor();
void anchor() { ext_anchor(); }
