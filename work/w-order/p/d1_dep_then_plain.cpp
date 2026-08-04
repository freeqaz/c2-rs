struct B{B();~B();int x;};
struct D:B{D();};
D::D(){}
B::~B(){}
int z(int a){return a+1;}
