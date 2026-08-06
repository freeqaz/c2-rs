// w-fix GRID-3 cell m8_arg_every_link

void h(int a) {}
void g2(int a) { h(a + 1); }
void g1(int a) { g2(a * 2); }
void f(int a) { g1(a - 3); }

// The per-cell positive control: a callee this TU does not define. `?anchor`
// must keep exactly one REL24 at BOTH flag settings or the cell is not graded.
void ext_anchor();
void anchor() { ext_anchor(); }
