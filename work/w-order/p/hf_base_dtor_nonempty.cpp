int ga(int);
void gh();
struct B{B();~B();int x;};
struct D:B{D();};
int a0(int a){return ga(a)+1;}
B::~B(){gh();}
D::D(){}
