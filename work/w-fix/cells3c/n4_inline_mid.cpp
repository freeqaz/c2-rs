// w-fix GRID-3 cell n4_inline_mid

inline void h() {}
inline void g1() { h(); }
void f() { g1(); }

// The per-cell positive control: a callee this TU does not define. `?anchor`
// must keep exactly one REL24 at BOTH flag settings or the cell is not graded.
void ext_anchor();
void anchor() { ext_anchor(); }
