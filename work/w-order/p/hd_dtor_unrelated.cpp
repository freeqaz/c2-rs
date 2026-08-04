int ga(int);
struct Bd{Bd();~Bd();int b0;};
struct M:Bd{M();~M();};
int a0(int a){return ga(a)+1;}
struct Q{Q();~Q();int q;};
Q::~Q(){}
M::~M(){}
int a1(int a){return ga(a)+2;}
