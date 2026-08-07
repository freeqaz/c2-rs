// w-alloc3 cell H-out-twocall-b — axis H-out
struct V { int* a; int* b; int c; };
struct W { V* p; int q; };
int g(int a, int b) { return a + b; }
int f(int x0, int x1) { return g(x0, x1) + g(x1, x0); }

void ext_anchor();
void anchor() { ext_anchor(); }
