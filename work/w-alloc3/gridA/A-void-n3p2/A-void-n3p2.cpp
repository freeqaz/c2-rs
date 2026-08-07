// w-alloc3 cell A-void-n3p2 — axis A-void
struct V { int* a; int* b; int c; };
struct W { V* p; int q; };
void g(V* v) { v->c = 0; }
void f(int x0, int x1, V* x2) { g(x2); }

void ext_anchor();
void anchor() { ext_anchor(); }
