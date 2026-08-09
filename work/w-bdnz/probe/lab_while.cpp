int gz(int);
int lead(int n, int k) { int s = 0; int i = 0; while (i < n) { s -= k; ++i; } return s; }
int z9(int a) { return gz(a) + 7; }
