// w-alloc3 cell H-spell-neg — axis H-spell
struct V { int* a; int* b; int c; };
struct W { V* p; int q; };
int g(int a) { return -a; }
int f(int x0, int x1) { return g(x1) + 4; }

void ext_anchor();
void anchor() { ext_anchor(); }
