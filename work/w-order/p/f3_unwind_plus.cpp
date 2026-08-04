struct B{B();~B();int x;};
struct D:B{D();};
struct E{E();int y;};
D::D(){}
B::~B(){}
E::E(){}
