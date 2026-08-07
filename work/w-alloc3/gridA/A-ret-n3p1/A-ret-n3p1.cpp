// w-alloc3 cell A-ret-n3p1 — axis A-ret
struct V { int* a; int* b; int c; };
struct W { V* p; int q; };
int* g(V* v) { return v->b; }
int* f(int x0, V* x1, int x2) { return g(x1); }

void ext_anchor();
void anchor() { ext_anchor(); }
