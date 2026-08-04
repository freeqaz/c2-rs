// A9 (plan D6, sharp) — an out-of-line VIRTUAL DESTRUCTOR definition is the only
// root touching the class; no constructor is kept anywhere.
struct D { virtual int f(int x) { return x*3+1; } virtual int g(int x) { return x+7; } virtual ~D(); };
D::~D() { }
extern int sink(int);
int anchor(int x) { return sink(x) + 3; }
