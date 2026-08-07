// w-alloc3 cell A-twoa-SUM-10 — axis A-two
struct V { int* a; int* b; int c; };
struct W { V* p; int q; };
int g(int a, int b) { return a + b; }
int f(int x0, int x1) { return g(x1, x0) + 7; }

void ext_anchor();
void anchor() { ext_anchor(); }
