// w-alloc3 cell H-perm4-310 — axis H-perm4
struct V { int* a; int* b; int c; };
struct W { V* p; int q; };
int g(int a, int b, int c) { return a - b + c; }
int f(int x0, int x1, int x2, int x3) { return g(x3, x1, x0); }

void ext_anchor();
void anchor() { ext_anchor(); }
