// w-fix GRID-3 cell k10_cycle2

void b();
void a() { b(); }
void b() { a(); }
void f() { a(); }

// The per-cell positive control: a callee this TU does not define. `?anchor`
// must keep exactly one REL24 at BOTH flag settings or the cell is not graded.
void ext_anchor();
void anchor() { ext_anchor(); }
