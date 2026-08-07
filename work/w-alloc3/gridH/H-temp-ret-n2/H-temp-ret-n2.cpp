// w-alloc3 cell H-temp-ret-n2 — axis H-temp
struct V { int* a; int* b; int c; };
struct W { V* p; int q; };
int* g(W* w) { return w->p->b; }
int* f(int x0, W* x1) { return g(x1); }

void ext_anchor();
void anchor() { ext_anchor(); }
