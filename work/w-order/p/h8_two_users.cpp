int ga(int);
struct B{B();~B();int x;};
struct C{C();~C();int y;};
struct D:B{D();};
int a0(int a){return ga(a)+1;}
struct F:B{F();};
B::~B(){}
D::D(){}
F::F(){}
