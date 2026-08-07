// w-alloc3 cell H-temp-n3 — axis H-temp
struct V { int* a; int* b; int c; };
struct W { V* p; int q; };
int* g(W* w) { return w->p->b; }
int* f(int x0, int x1, W* x2) { return g(x2) - 1; }

void ext_anchor();
void anchor() { ext_anchor(); }
