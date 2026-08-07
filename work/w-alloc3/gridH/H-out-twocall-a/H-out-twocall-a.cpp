// w-alloc3 cell H-out-twocall-a — axis H-out
struct V { int* a; int* b; int c; };
struct W { V* p; int q; };
int* g(V* v) { return v->b; }
int* f(int x0, V* x1) { int* p = g(x1); int* q = g(x1); return p < q ? p : q; }

void ext_anchor();
void anchor() { ext_anchor(); }
