// w-slots grid cell: ONE float array-walk loop + the SAME framed z9, so the
// framed function`s $M triple is the only thing that varies. z9 is
// wblockir_float_walk_then_framed_neg.cpp`s own framed function.
int gz(int);

int straight(int a, int b) { int x = a + 1; int y = b + 2; return x + y; }

int z9(int a) { return gz(a) + 7; }
