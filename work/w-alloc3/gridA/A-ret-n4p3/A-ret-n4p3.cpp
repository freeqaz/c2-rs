// w-alloc3 cell A-ret-n4p3 — axis A-ret
struct V { int* a; int* b; int c; };
struct W { V* p; int q; };
int* g(V* v) { return v->b; }
int* f(int x0, int x1, int x2, V* x3) { return g(x3); }

void ext_anchor();
void anchor() { ext_anchor(); }
