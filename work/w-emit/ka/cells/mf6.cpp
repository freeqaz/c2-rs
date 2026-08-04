extern int sink(int);
static int unused_helper(int x) { return x * 5 + 2; }
int anchor(int x) { return sink(x); }
