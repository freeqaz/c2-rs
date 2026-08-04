void h();
struct B{B();~B();int x;};
struct D:B{D();};
D::D(){}
B::~B(){h();}
