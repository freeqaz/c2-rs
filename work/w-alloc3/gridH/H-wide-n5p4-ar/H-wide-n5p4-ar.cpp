// w-alloc3 cell H-wide-n5p4-ar — axis H-wide
struct V { int* a; int* b; int c; };
struct W { V* p; int q; };
int* g(V* v) { return v->b; }
int* f(int x0, int x1, int x2, int x3, V* x4) { return g(x4) - 1; }

void ext_anchor();
void anchor() { ext_anchor(); }
