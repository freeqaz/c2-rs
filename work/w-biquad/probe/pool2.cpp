// w-biquad PROBE — cell 2: the SAME two constants, use order REVERSED.
struct P2 { float a[4]; void s(); };
void P2::s() { a[0] = 7.5f; a[1] = 2.5f; }
