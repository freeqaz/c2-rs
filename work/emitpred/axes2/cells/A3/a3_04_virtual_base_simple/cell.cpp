// A3 — virtual (shared) base, no override of the base's virtual.
struct A { virtual int f(int x) { return x*3+1; } virtual ~A() {} };
struct D : virtual A { virtual int g(int x) { return x-5; } };
extern int sink(int);
extern void use(void*);
int anchor(int x) { D o; use(&o); return sink(x); }
