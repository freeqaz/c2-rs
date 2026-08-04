struct B{B();~B();int x;};
struct M:B{M();};
struct D:M{D();};
D::D(){}
M::M(){}
B::~B(){}
