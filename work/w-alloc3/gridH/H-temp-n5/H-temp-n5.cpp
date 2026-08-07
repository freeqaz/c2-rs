// w-alloc3 cell H-temp-n5 — axis H-temp
struct V { int* a; int* b; int c; };
struct W { V* p; int q; };
int* g(W* w) { return w->p->b; }
int* f(int x0, int x1, int x2, int x3, W* x4) { return g(x4) - 1; }

void ext_anchor();
void anchor() { ext_anchor(); }
