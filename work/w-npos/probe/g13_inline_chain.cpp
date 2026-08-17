// g13 — closure among non-roots: two inlines, one calling the other, no roots.
// Prediction: nothing emitted; the shell.
inline int a2(int x) { return x * 2; }
inline int a1(int x) { return a2(x) + 1; }
