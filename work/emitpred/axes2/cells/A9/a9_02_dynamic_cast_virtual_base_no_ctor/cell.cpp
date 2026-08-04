// A9 — dynamic_cast across a VIRTUAL base, no constructor kept.
struct A { virtual int f(int x); virtual ~A(); };
struct D : virtual A { virtual int f(int x) { return x*3+1; } virtual int g(int x) { return x+7; } };
extern int sink(int);
int anchor(A* p, int x) { D* d = dynamic_cast<D*>(p); return d ? sink(x) : 0; }
