void ext_anchor();
void anchor() { ext_anchor(); }
void ext_clear();
struct B { ~B(); };
B::~B() { ext_clear(); }
struct D : B { ~D(); };
D::~D() { }
void keep(D *p) { delete p; }
