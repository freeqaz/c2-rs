// w-empty GRID cell c12_ctor_meminit

struct S { int x; S() : x(0) {} };
struct D : S { D(); };
D::D() {}

// The per-cell positive control: a callee this TU does not define. `?anchor`
// must keep exactly one REL24 at BOTH flag settings or the cell is not graded.
void ext_anchor();
void anchor() { ext_anchor(); }
