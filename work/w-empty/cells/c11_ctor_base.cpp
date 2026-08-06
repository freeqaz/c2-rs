// w-empty GRID cell c11_ctor_base

struct S { S() {} };
struct D : S { D(); };
D::D() {}

// The per-cell positive control: a callee this TU does not define. `?anchor`
// must keep exactly one REL24 at BOTH flag settings or the cell is not graded.
void ext_anchor();
void anchor() { ext_anchor(); }
