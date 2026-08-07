// w-alloc3 cell H-wide-n6p5-ar — axis H-wide
struct V { int* a; int* b; int c; };
struct W { V* p; int q; };
int* g(V* v) { return v->b; }
int* f(int x0, int x1, int x2, int x3, int x4, V* x5) { return g(x5) - 1; }

void ext_anchor();
void anchor() { ext_anchor(); }
