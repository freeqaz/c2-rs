// GRID-S cell s02_void_call_no_setup — a caller whose call is not in tail position
int q(int a) { return a + 1; }
int p;
void f() { p = q(p); }

void ext_anchor();
void anchor() { ext_anchor(); }
