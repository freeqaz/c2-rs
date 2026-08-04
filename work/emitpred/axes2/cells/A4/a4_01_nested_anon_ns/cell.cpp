// A4 — unnamed namespace nested directly inside an unnamed namespace.
namespace { namespace { int cand(int x) { return x*3+1; } } }
extern int sink(int);
int anchor(int x) { return sink(x) + 3; }
