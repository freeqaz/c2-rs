// A9 — reference form of dynamic_cast (throws on failure; interacts with /EHsc).
struct B { virtual int f(int x); virtual ~B(); };
struct D : B { virtual int f(int x) { return x*3+1; } virtual int g(int x) { return x+7; } };
extern int sink(int);
int anchor(B& b, int x) { D& d = dynamic_cast<D&>(b); return sink(d.f(x)); }
