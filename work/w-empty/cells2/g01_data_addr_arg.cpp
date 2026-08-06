// w-empty GRID cell g01_data_addr_arg

int gv;
void g(int* p) {}
void f() { g(&gv); }

// The per-cell positive control: a callee this TU does not define. `?anchor`
// must keep exactly one REL24 at BOTH flag settings or the cell is not graded.
void ext_anchor();
void anchor() { ext_anchor(); }
