// A4 x A5 — extern "C" function defined inside an unnamed namespace, unreferenced.
namespace { extern "C" int cand(int x) { return x*3+1; } }
extern int sink(int);
int anchor(int x) { return sink(x) + 3; }
