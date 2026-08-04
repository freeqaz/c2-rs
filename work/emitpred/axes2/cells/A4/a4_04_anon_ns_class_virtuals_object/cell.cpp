// A4 x A3 — unnamed-namespace class with virtuals; an object IS constructed.
namespace {
struct V { virtual int f(int x) { return x*3+1; } virtual int g(int x) { return x+7; } virtual ~V() {} };
}
extern int sink(int);
extern void use(void*);
int anchor(int x) { V o; use(&o); return sink(x); }
