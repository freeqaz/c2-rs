// w-alloc3 cell A-arith-n3p1 — axis A-arith
struct V { int* a; int* b; int c; };
struct W { V* p; int q; };
int* g(V* v) { return v->b; }
int* f(int x0, V* x1, int x2) { return g(x1) - 1; }

void ext_anchor();
void anchor() { ext_anchor(); }
