struct Bd{Bd();~Bd();int b0;};
struct M:Bd{M();~M();};
struct D:M{D();};
D::D(){}
M::~M(){}
int z(int a){return a+1;}
