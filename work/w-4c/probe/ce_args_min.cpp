// The half of `ce_args.cpp` the PORT accepts, so the differential can return
// `Port=Match` rather than `NotImplemented`. That row is the load-bearing one:
// it means the ACCEPTING parser walked an argument region, read the `4C` as one
// byte, and the obj that came out is byte-exact against real `c2.dll` under
// wibo — not that a script decoded it that way.
extern int g1(int a);
extern int g2(int a, int b);
extern int g3(int a, int b, int c);
int c1(int x) { return g1(x); }
int c2(int x, int y) { return g2(x, y); }
int c3(int x, int y, int z) { return g3(x, y, z); }
