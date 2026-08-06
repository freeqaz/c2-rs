// w-empty GRID cell c16_volatile_local

void g() { volatile int x = 0; }
void f() { g(); }

// The per-cell positive control: a callee this TU does not define. `?anchor`
// must keep exactly one REL24 at BOTH flag settings or the cell is not graded.
void ext_anchor();
void anchor() { ext_anchor(); }
