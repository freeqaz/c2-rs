// w-alloc3 cell A-arith-n1p0 — axis A-arith
struct V { int* a; int* b; int c; };
struct W { V* p; int q; };
int* g(V* v) { return v->b; }
int* f(V* x0) { return g(x0) - 1; }

void ext_anchor();
void anchor() { ext_anchor(); }
