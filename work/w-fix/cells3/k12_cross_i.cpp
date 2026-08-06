// w-fix GRID-3 cell k12_cross_i

int m(int a) { return a; }
int g1(int a) { return m(a); }
int f(int a) { return g1(a); }

// The per-cell positive control: a callee this TU does not define. `?anchor`
// must keep exactly one REL24 at BOTH flag settings or the cell is not graded.
void ext_anchor();
void anchor() { ext_anchor(); }
