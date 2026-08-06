// w-fix GRID-3 cell m7_dtor_chain_d4

struct A { ~A() {} };
struct B { A a; ~B(); };
struct C { B b; ~C(); };
struct D { C c; ~D(); };
struct E { D d; ~E(); };
B::~B() {}
C::~C() {}
D::~D() {}
E::~E() {}

// The per-cell positive control: a callee this TU does not define. `?anchor`
// must keep exactly one REL24 at BOTH flag settings or the cell is not graded.
void ext_anchor();
void anchor() { ext_anchor(); }
