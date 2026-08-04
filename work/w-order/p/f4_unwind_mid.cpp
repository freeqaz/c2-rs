void h();
struct B{B();~B();int x;};
struct D:B{D();};
D::D(){}
void k(){h();}
B::~B(){}
