// w-alloc3 cell H-temp-n1 — axis H-temp
struct V { int* a; int* b; int c; };
struct W { V* p; int q; };
int* g(W* w) { return w->p->b; }
int* f(W* x0) { return g(x0) - 1; }

void ext_anchor();
void anchor() { ext_anchor(); }
