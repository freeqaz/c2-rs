// w-alloc3 cell A-void-n3p1 — axis A-void
struct V { int* a; int* b; int c; };
struct W { V* p; int q; };
void g(V* v) { v->c = 0; }
void f(int x0, V* x1, int x2) { g(x1); }

void ext_anchor();
void anchor() { ext_anchor(); }
