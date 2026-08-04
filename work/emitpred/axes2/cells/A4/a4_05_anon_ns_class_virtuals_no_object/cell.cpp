// A4 x A3 — unnamed-namespace class with virtuals; NO object is ever constructed.
namespace {
struct V { virtual int f(int x) { return x*3+1; } virtual int g(int x) { return x+7; } virtual ~V() {} };
}
extern int sink(int);
int anchor(int x) { return sink(x) + 3; }
