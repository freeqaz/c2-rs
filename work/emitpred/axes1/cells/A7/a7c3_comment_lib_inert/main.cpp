#pragma comment(lib, "foo")
#pragma comment(exestr, "axes1")
static int cand(int x) { return x*3+1; }
extern int sink(int);
int anchor(int x) { return sink(x) + 3; }
