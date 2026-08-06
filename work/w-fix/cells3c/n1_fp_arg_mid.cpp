// w-fix GRID-3 cell n1_fp_arg_mid

void h(float x) {}
void g1(float x) { h(x); }
void f(float x) { g1(x); }

// The per-cell positive control: a callee this TU does not define. `?anchor`
// must keep exactly one REL24 at BOTH flag settings or the cell is not graded.
void ext_anchor();
void anchor() { ext_anchor(); }
