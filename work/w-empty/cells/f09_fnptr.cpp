// w-empty GRID cell f09_fnptr

void g() {}
void f() { void (*p)() = g; p(); }

// The per-cell positive control: a callee this TU does not define. `?anchor`
// must keep exactly one REL24 at BOTH flag settings or the cell is not graded.
void ext_anchor();
void anchor() { ext_anchor(); }
