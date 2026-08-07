// w-alloc3 cell H-spell-mul — axis H-spell
struct V { int* a; int* b; int c; };
struct W { V* p; int q; };
int g(int a, int b) { return a * b; }
int f(int x0, int x1, int x2) { return g(x2, x0) + 4; }

void ext_anchor();
void anchor() { ext_anchor(); }
