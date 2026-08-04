struct Bd{Bd();~Bd();int b0;};
struct M:Bd{M();~M();};
struct D:M{D();};
M::~M(){}
D::D(){}
