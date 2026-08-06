// w-fix GRID-3 cell n6_ctor_chain

struct A { A() {} };
struct B { A a; B(); };
struct C { B b; C(); };
B::B() {}
C::C() {}

// The per-cell positive control: a callee this TU does not define. `?anchor`
// must keep exactly one REL24 at BOTH flag settings or the cell is not graded.
void ext_anchor();
void anchor() { ext_anchor(); }
