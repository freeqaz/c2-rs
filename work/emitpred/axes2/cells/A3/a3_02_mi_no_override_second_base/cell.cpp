// A3 — control for a3_01: MI, no override of either base's virtual (no thunk needed).
struct A { virtual int f(int x) { return x*3+1; } };
struct B { virtual int g(int x) { return x+7; } };
struct D : A, B { virtual int h(int x) { return x-5; } };
extern int sink(int);
extern void use(void*);
int anchor(int x) { D o; use(&o); return sink(x); }
