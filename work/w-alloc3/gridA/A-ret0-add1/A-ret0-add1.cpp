// w-alloc3 cell A-ret0-add1 — axis A-len
struct V { int* a; int* b; int c; };
struct W { V* p; int q; };
int g(int a) { return a + 1; }
int f(int x0) { return g(x0); }

void ext_anchor();
void anchor() { ext_anchor(); }
