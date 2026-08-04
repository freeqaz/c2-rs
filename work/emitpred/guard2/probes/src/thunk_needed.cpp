// guard2 post-hoc probe (NOT a graded cell): a3_01 with `this` actually used in
// the second-base override, so an adjustor thunk is semantically required.
struct A { int ax; virtual int f(int x) { return x*3+ax; } };
struct B { int bx; virtual int g(int x) { return x+bx; } };
struct D : A, B { int dx; virtual int g(int x) { return x-dx; } };
extern int sink(int);
extern void use(void*);
int anchor(int x) { D o; use(&o); return sink(x); }
