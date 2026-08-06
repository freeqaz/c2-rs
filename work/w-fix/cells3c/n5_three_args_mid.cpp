// w-fix GRID-3 cell n5_three_args_mid

void h(int a, int b, int c) {}
void g1(int a, int b, int c) { h(a, b, c); }
void f(int a, int b, int c) { g1(a, b, c); }

// The per-cell positive control: a callee this TU does not define. `?anchor`
// must keep exactly one REL24 at BOTH flag settings or the cell is not graded.
void ext_anchor();
void anchor() { ext_anchor(); }
