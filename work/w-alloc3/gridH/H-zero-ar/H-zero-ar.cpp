// w-alloc3 cell H-zero-ar — axis H-zero-off
struct V { int* a; int* b; int c; };
struct W { V* p; int q; };
int* g(V* v) { return v->a; }
int* f(int x0, V* x1) { return g(x1) - 1; }

void ext_anchor();
void anchor() { ext_anchor(); }
