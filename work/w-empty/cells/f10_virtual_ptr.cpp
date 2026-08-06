// w-empty GRID cell f10_virtual_ptr

struct S { virtual void g() {} };
void f(S* s) { s->g(); }

// The per-cell positive control: a callee this TU does not define. `?anchor`
// must keep exactly one REL24 at BOTH flag settings or the cell is not graded.
void ext_anchor();
void anchor() { ext_anchor(); }
