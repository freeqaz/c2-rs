// A9 — dynamic_cast<void*>: needs the runtime vtable of the STATIC type only.
struct B { virtual int f(int x); virtual ~B(); };
extern int sink(int);
int anchor(B* p, int x) { return dynamic_cast<void*>(p) ? sink(x) : 0; }
