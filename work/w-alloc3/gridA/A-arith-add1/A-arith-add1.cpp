// w-alloc3 cell A-arith-add1 — axis A-len
struct V { int* a; int* b; int c; };
struct W { V* p; int q; };
int g(int a) { return a + 1; }
int f(int x0) { return g(x0) + 5; }

void ext_anchor();
void anchor() { ext_anchor(); }
