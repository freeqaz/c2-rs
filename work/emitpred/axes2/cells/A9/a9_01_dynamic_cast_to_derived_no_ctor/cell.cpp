// A9 (plan D6) — dynamic_cast to a polymorphic derived class; NO constructor of it
// is ODR-used anywhere in the TU.
struct B { virtual int f(int x); virtual ~B(); };
struct D : B { virtual int f(int x) { return x*3+1; } virtual int g(int x) { return x+7; } };
extern int sink(int);
int anchor(B* p, int x) { D* d = dynamic_cast<D*>(p); return d ? sink(x) : 0; }
