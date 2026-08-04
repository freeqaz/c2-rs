struct B{B();~B();int x;};
struct D:B{D();};
struct E:B{~E();};
D::D(){}
E::~E(){}
