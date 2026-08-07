// w-alloc3 cell H-noarg-ret — axis H-noarg
struct V { int* a; int* b; int c; };
struct W { V* p; int q; };
int g() { return 42; }
int f(int x0, int x1) { return g(); }

void ext_anchor();
void anchor() { ext_anchor(); }
