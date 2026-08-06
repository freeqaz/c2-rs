// w-fix GRID-3 cell n3_virtual_mid

struct S { virtual void h() {} void g1() { S::h(); } };
void f(S& s) { s.S::g1(); }

// The per-cell positive control: a callee this TU does not define. `?anchor`
// must keep exactly one REL24 at BOTH flag settings or the cell is not graded.
void ext_anchor();
void anchor() { ext_anchor(); }
