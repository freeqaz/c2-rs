// w-alloc3 cell H-noarg-ar — axis H-noarg
struct V { int* a; int* b; int c; };
struct W { V* p; int q; };
int g() { return 42; }
int f(int x0, int x1) { return g() + 6; }

void ext_anchor();
void anchor() { ext_anchor(); }
