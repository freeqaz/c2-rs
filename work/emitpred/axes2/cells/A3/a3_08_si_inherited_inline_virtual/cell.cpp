// A3 — single inheritance; a base virtual that D neither overrides nor calls.
struct A { virtual int f(int x) { return x*3+1; } virtual int only_a(int x) { return x-7; } };
struct D : A { virtual int g(int x) { return x+7; } };
extern int sink(int);
extern void use(void*);
int anchor(int x) { D o; use(&o); return sink(x); }
