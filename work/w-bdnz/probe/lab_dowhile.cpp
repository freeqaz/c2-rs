int gz(int);
int lead(int n, int k) { int s = 0; int i = 0; do { s -= k; ++i; } while (i < n); return s; }
int z9(int a) { return gz(a) + 7; }
