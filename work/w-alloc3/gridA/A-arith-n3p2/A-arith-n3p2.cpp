// w-alloc3 cell A-arith-n3p2 — axis A-arith
struct V { int* a; int* b; int c; };
struct W { V* p; int q; };
int* g(V* v) { return v->b; }
int* f(int x0, int x1, V* x2) { return g(x2) - 1; }

void ext_anchor();
void anchor() { ext_anchor(); }
