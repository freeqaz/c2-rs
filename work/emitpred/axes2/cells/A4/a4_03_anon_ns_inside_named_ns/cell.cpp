// A4 — unnamed namespace nested inside a NAMED namespace.
namespace N { namespace { int cand(int x) { return x*3+1; } } }
extern int sink(int);
int anchor(int x) { return sink(x) + 3; }
