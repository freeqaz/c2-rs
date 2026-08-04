int ga(int);
struct B{B();~B();int x;};
struct C{C();~C();int y;};
struct E{E();~E();int w;};
struct D:B{D();};
int a0(int a){return ga(a)+1;}
int z(int a){return a+1;}
int y(int a){return a+2;}
D::D(){}
