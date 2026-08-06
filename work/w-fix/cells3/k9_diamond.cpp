// w-fix GRID-3 cell k9_diamond

void h() {}
void ga() { h(); }
void gb() { h(); }
void f() { ga(); gb(); }

// The per-cell positive control: a callee this TU does not define. `?anchor`
// must keep exactly one REL24 at BOTH flag settings or the cell is not graded.
void ext_anchor();
void anchor() { ext_anchor(); }
