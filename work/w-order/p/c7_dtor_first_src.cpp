struct B{B();~B();int x;};
struct D:B{D();};
B::~B(){}
D::D(){}
