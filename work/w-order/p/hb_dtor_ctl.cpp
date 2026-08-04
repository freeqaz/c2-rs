int ga(int);
struct Bd{Bd();~Bd();int b0;};
struct M:Bd{M();~M();};
int a0(int a){return ga(a)+1;}
M::~M(){}
int a1(int a){return ga(a)+2;}
