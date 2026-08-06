// w-fix GRID-3 cell k15_side_effect_mid

int sink;
void h(int a) {}
void g1() { h(sink++); }
void f() { g1(); }

// The per-cell positive control: a callee this TU does not define. `?anchor`
// must keep exactly one REL24 at BOTH flag settings or the cell is not graded.
void ext_anchor();
void anchor() { ext_anchor(); }
