int ga(int);
struct B{B();~B();int x;};
struct G{G();~G();int g;};
struct D:B{D();};
struct H:G{H();};
int a0(int a){return ga(a)+1;}
B::~B(){}
H::H(){}
D::D(){}
