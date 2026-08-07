// w-alloc3 cell H-perm4-023 — axis H-perm4
struct V { int* a; int* b; int c; };
struct W { V* p; int q; };
int g(int a, int b, int c) { return a - b + c; }
int f(int x0, int x1, int x2, int x3) { return g(x0, x2, x3) + 9; }

void ext_anchor();
void anchor() { ext_anchor(); }
