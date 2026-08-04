struct Bd{Bd();~Bd();int b0;};
struct M:Bd{M();~M();};
struct D:M{D();};
int z(int a){return a+1;}
D::D(){}
M::~M(){}
