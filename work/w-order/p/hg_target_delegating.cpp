int ga(int);
struct Bd{Bd();~Bd();int b0;};
struct M:Bd{M();~M();};
struct D:M{D();};
int a0(int a){return ga(a)+1;}
M::~M(){}
D::D(){}
