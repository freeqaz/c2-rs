// w-alloc3 cell A-twoa-SUB-01 — axis A-two
struct V { int* a; int* b; int c; };
struct W { V* p; int q; };
int g(int a, int b) { return a - b; }
int f(int x0, int x1) { return g(x0, x1) + 7; }

void ext_anchor();
void anchor() { ext_anchor(); }
