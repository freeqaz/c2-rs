// w-fix GRID-3 cell k18_dtor_chain_d3

struct A { ~A() {} };
struct B { A a; ~B(); };
struct C { B b; ~C(); };
struct D { C c; ~D(); };
B::~B() {}
C::~C() {}
D::~D() {}

// The per-cell positive control: a callee this TU does not define. `?anchor`
// must keep exactly one REL24 at BOTH flag settings or the cell is not graded.
void ext_anchor();
void anchor() { ext_anchor(); }
