// w-alloc3 cell H-spell-ext-ret — axis H-spell
struct V { int* a; int* b; int c; };
struct W { V* p; int q; };
int g(V* v) { return (short)v->c; }
int f(int x0, V* x1) { return g(x1); }

void ext_anchor();
void anchor() { ext_anchor(); }
