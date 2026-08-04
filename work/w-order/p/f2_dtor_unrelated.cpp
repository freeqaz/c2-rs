struct B{~B();int x;};
struct C{C();int y;};
struct D:C{D();};
D::D(){}
B::~B(){}
