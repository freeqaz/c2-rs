// A4 — NAMED namespace nested inside an unnamed namespace.
namespace { namespace N { int cand(int x) { return x*3+1; } } }
extern int sink(int);
int anchor(int x) { return sink(x) + 3; }
