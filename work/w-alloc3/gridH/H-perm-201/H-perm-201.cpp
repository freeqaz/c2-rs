// w-alloc3 cell H-perm-201 — axis H-perm
struct V { int* a; int* b; int c; };
struct W { V* p; int q; };
int g(int a, int b, int c) { return a - b + c; }
int f(int x0, int x1, int x2) { return g(x2, x0, x1); }

void ext_anchor();
void anchor() { ext_anchor(); }
