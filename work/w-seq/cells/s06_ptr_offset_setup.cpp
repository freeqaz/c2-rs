// GRID-S cell s06_ptr_offset_setup — setup is a pointer offset — the `??1?$pair@…` displacement fold
struct A { int p; int q; };
struct B { int z; A a; };
int g(A* a) { return a->q; }
int f(B* b) { return g(&b->a); }

void ext_anchor();
void anchor() { ext_anchor(); }
