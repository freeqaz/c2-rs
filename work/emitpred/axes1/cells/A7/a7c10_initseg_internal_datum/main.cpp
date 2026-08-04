#pragma init_seg(compiler)
extern int seed();
static int mk(int x) { return x*3+1; }
static int g_v = mk(seed());
extern int sink(int);
int anchor(int x) { return sink(x) + g_v; }
