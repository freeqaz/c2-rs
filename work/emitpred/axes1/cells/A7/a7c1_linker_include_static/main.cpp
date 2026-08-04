#pragma comment(linker, "/include:?cand@@YAHH@Z")
static int cand(int x) { return x*3+1; }
extern int sink(int);
int anchor(int x) { return sink(x) + 3; }
