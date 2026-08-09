// p1 — FRAMED first, unframed leaf second: vsnprnc.cpp's own function order.
int gg(int);
int cal5(int, int, int, int, int);
int framed(int a) { return gg(a) + 1; }
int leaf5(int a, int b, int c, int d, int e) { return cal5(a, b, c, d, e); }
