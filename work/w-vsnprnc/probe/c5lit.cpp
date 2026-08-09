// c5lit — C++ linkage, the vsprintf_s shape exactly: one ascending move and one
// literal in the slot it vacates, tail-called.
int cal5(int, int, int, int, int);
int fwd4(int a, int b, int c, int e) { return cal5(a, b, c, 0, e); }
