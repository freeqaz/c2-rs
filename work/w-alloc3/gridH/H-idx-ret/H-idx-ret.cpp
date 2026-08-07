// w-alloc3 cell H-idx-ret — axis H-idx
struct V { int* a; int* b; int c; };
struct W { V* p; int q; };
int g(int* p, int i) { return p[i]; }
int f(int x0, int x1, int* x2) { return g(x2, x0); }

void ext_anchor();
void anchor() { ext_anchor(); }
