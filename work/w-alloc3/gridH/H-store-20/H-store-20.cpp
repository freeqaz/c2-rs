// w-alloc3 cell H-store-20 — axis H-store
struct V { int* a; int* b; int c; };
struct W { V* p; int q; };
void g(V* v, int k) { v->c = k; }
void f(int x0, int x1, V* x2) { g(x2, x0); }

void ext_anchor();
void anchor() { ext_anchor(); }
