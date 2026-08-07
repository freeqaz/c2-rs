// w-alloc3 cell A-void-n2p0 — axis A-void
struct V { int* a; int* b; int c; };
struct W { V* p; int q; };
void g(V* v) { v->c = 0; }
void f(V* x0, int x1) { g(x0); }

void ext_anchor();
void anchor() { ext_anchor(); }
