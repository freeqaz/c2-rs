// A9 — positive control: a kept constructor, so the vtable rule must fire.
struct D { virtual int f(int x) { return x*3+1; } virtual int g(int x) { return x+7; } virtual ~D() {} };
extern int sink(int);
extern void use(void*);
int anchor(int x) { D o; use(&o); return sink(x); }
