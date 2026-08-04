// A3 — virtual-base diamond; the final override comes from one middle class.
struct A { virtual int f(int x) { return x*3+1; } virtual ~A() {} };
struct B : virtual A { virtual int g(int x) { return x+7; } };
struct C : virtual A { virtual int f(int x) { return x-5; } };
struct D : B, C { };
extern int sink(int);
extern void use(void*);
int anchor(int x) { D o; use(&o); return sink(x); }
