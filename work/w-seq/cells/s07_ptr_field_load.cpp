// GRID-S cell s07_ptr_field_load — empty setup, a callee that loads through its formal
struct A { int p; int q; };
int g(A* a) { return a->q; }
int f(A* a) { return g(a); }

void ext_anchor();
void anchor() { ext_anchor(); }
