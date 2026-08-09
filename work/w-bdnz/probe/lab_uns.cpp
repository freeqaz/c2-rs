// Same class, UNSIGNED counter and bound: the guard is `cmplwi`/`bclr 12,26`
// instead of `cmpwi`/`bclr 4,25`. Does the label charge move with it?
int gz(int);
int lead(unsigned n, int k) { int s = 0; for (unsigned i = 0; i < n; ++i) s -= k; return s; }
int z9(int a) { return gz(a) + 7; }
