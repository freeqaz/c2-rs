// A3 — MI with virtual destructors in both bases, plus an override of the second base.
struct A { virtual int f(int x) { return x*3+1; } virtual ~A() {} };
struct B { virtual int g(int x) { return x+7; } virtual ~B() {} };
struct D : A, B { virtual int g(int x) { return x-5; } };
extern int sink(int);
extern void use(void*);
int anchor(int x) { D o; use(&o); return sink(x); }
