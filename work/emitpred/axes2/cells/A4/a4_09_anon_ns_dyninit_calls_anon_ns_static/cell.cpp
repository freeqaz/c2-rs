// A4 — unnamed-namespace non-const datum with a dynamic initializer that reaches
// an unnamed-namespace `static` helper and an unnamed-namespace non-static one.
namespace {
static int helper(int x) { return x*3+1; }
int seed(int x) { return x+7; }
int g_v = helper(2) + seed(3);
}
extern int sink(int);
int anchor(int x) { return sink(x) + 3; }
