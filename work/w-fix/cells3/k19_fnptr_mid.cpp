// w-fix GRID-3 cell k19_fnptr_mid

void h() {}
void g1() { void (*p)() = h; p(); }
void f() { g1(); }

// The per-cell positive control: a callee this TU does not define. `?anchor`
// must keep exactly one REL24 at BOTH flag settings or the cell is not graded.
void ext_anchor();
void anchor() { ext_anchor(); }
