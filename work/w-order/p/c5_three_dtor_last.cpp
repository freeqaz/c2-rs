struct B{B();~B();int x;};
struct D:B{D();~D();};
D::D(){}
D::~D(){}
B::~B(){}
