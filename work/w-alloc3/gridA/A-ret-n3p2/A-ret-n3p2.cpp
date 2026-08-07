// w-alloc3 cell A-ret-n3p2 — axis A-ret
struct V { int* a; int* b; int c; };
struct W { V* p; int q; };
int* g(V* v) { return v->b; }
int* f(int x0, int x1, V* x2) { return g(x2); }

void ext_anchor();
void anchor() { ext_anchor(); }
