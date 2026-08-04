// A9 (plan D6, sharp) — `delete` through a pointer to a class with a virtual
// destructor; no constructor of that class is kept in the TU.
struct D { virtual int f(int x) { return x*3+1; } virtual int g(int x) { return x+7; } virtual ~D() {} };
extern int sink(int);
int anchor(D* p, int x) { delete p; return sink(x) + 3; }
