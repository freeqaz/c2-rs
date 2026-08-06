// w-fix GRID-3 cell m4_seq_mixed_mid

void ext();
void h() {}
void g1() { h(); ext(); }
void f() { g1(); }

// The per-cell positive control: a callee this TU does not define. `?anchor`
// must keep exactly one REL24 at BOTH flag settings or the cell is not graded.
void ext_anchor();
void anchor() { ext_anchor(); }
