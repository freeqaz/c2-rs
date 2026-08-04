// A4 — unnamed-namespace class with one OUT-OF-LINE member and one in-class member,
// neither referenced.
namespace {
struct S { int m(int x); int inl(int x) { return x-7; } };
int S::m(int x) { return x*3+1; }
}
extern int sink(int);
int anchor(int x) { return sink(x) + 3; }
