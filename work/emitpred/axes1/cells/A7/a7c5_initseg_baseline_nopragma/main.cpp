extern int seed();
static int mk(int x) { return x*3+1; }
int g_v = mk(seed());
extern int sink(int);
int anchor(int x) { return sink(x) + 3; }
