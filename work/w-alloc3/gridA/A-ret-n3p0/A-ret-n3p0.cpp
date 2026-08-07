// w-alloc3 cell A-ret-n3p0 — axis A-ret
struct V { int* a; int* b; int c; };
struct W { V* p; int q; };
int* g(V* v) { return v->b; }
int* f(V* x0, int x1, int x2) { return g(x0); }

void ext_anchor();
void anchor() { ext_anchor(); }
