// w-fix GRID-3 cell m5_static_chain

static void h() {}
static void g2() { h(); }
static void g1() { g2(); }
void f() { g1(); }

// The per-cell positive control: a callee this TU does not define. `?anchor`
// must keep exactly one REL24 at BOTH flag settings or the cell is not graded.
void ext_anchor();
void anchor() { ext_anchor(); }
