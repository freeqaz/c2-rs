// w-alloc3 cell A-void-n1p0 — axis A-void
struct V { int* a; int* b; int c; };
struct W { V* p; int q; };
void g(V* v) { v->c = 0; }
void f(V* x0) { g(x0); }

void ext_anchor();
void anchor() { ext_anchor(); }
