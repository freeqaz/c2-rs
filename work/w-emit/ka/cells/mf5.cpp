extern int sink(int);
static int helper(int x) { return x * 3 + 1; }
int anchor(int x) { return helper(x) + sink(x); }
