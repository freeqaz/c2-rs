// w-alloc3 cell H-wide-n6p0-ret — axis H-wide
struct V { int* a; int* b; int c; };
struct W { V* p; int q; };
int* g(V* v) { return v->b; }
int* f(V* x0, int x1, int x2, int x3, int x4, int x5) { return g(x0); }

void ext_anchor();
void anchor() { ext_anchor(); }
