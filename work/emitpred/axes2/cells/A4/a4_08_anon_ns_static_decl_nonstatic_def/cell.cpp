// A4 — crossing: declared `static` inside the unnamed namespace, defined without it.
namespace {
static int cand(int x);
int cand(int x) { return x*3+1; }
}
extern int sink(int);
int anchor(int x) { return sink(x) + 3; }
