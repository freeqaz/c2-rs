// A3 — MI with out-of-line virtual definitions and NO object constructed anywhere.
struct A { virtual int f(int x); virtual int inl(int x) { return x-7; } };
struct B { virtual int g(int x); };
struct D : A, B { virtual int g(int x); };
int A::f(int x) { return x*3+1; }
int D::g(int x) { return x+7; }
extern int sink(int);
int anchor(int x) { return sink(x) + 3; }
