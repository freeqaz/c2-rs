// A9 — address-take of a VIRTUAL member function, no constructor kept.
struct D { virtual int f(int x) { return x*3+1; } virtual int g(int x) { return x+7; } };
typedef int (D::*PMF)(int);
PMF anchor() { return &D::f; }
