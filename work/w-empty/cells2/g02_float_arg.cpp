// w-empty GRID cell g02_float_arg

void g(float x) {}
void f(float x) { g(x); }

// The per-cell positive control: a callee this TU does not define. `?anchor`
// must keep exactly one REL24 at BOTH flag settings or the cell is not graded.
void ext_anchor();
void anchor() { ext_anchor(); }
