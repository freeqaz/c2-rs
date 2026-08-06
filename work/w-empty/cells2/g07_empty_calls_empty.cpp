// w-empty GRID cell g07_empty_calls_empty

void h() {}
void g() { h(); }
void f() { g(); }

// The per-cell positive control: a callee this TU does not define. `?anchor`
// must keep exactly one REL24 at BOTH flag settings or the cell is not graded.
void ext_anchor();
void anchor() { ext_anchor(); }
