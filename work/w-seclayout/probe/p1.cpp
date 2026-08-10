struct B { virtual ~B() {} };
struct D : B { virtual ~D(); };
D::~D() {}
