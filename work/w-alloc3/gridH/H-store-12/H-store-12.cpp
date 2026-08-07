// w-alloc3 cell H-store-12 — axis H-store
struct V { int* a; int* b; int c; };
struct W { V* p; int q; };
void g(V* v, int k) { v->c = k; }
void f(int x0, V* x1, int x2) { g(x1, x2); }

void ext_anchor();
void anchor() { ext_anchor(); }
