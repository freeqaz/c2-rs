// w-alloc3 cell H-out-computed-a — axis H-out
struct V { int* a; int* b; int c; };
struct W { V* p; int q; };
int g(int a) { return a + 1; }
int f(int x0, int x1) { return g(x0 + 1); }

void ext_anchor();
void anchor() { ext_anchor(); }
