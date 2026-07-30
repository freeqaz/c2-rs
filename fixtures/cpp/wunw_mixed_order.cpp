// W-UNW-1: framed, leaf, framed, leaf — with DISTINCT callees, so the two
// framed groups differ in the one way the symbol layout cares about: the first
// introduces `?g` inside its group, the second introduces `?h` inside its own.
//
// It is the only fixture where the shared `.pdata` section symbol lands in the
// group of a function that is not the last framed one, which is what fixes the
// order `[fn] [$M end] [callee] [$M prologue] [.pdata sym + aux] [$T]` for the
// FIRST framed function and `[fn] [$M end] [callee] [$M prologue] [$T]` for
// every later one.
int g(int);
int h(int);
int f1(int a) { return g(a) + 1; }
int lf(int a) { return a + 7; }
int f2(int a) { return h(a) + 3; }
int lg(int a) { return a; }
