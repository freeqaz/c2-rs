struct B { virtual ~B(); virtual void f(); };
inline B::~B() {}
struct D : B { ~D(); void f(); };
D::~D() {}
void D::f() {}
