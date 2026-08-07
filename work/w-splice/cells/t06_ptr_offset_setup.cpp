// GRID-T cell t06_ptr_offset_setup — S3 REFUSES — the setup is a pointer offset; c2 folds the displacement
struct A { int p; int q; };
struct B { int z; A a; };
int g(A* a) { return a->q; }
int f(B* b) { return g(&b->a); }

void ext_anchor();
void anchor() { ext_anchor(); }
