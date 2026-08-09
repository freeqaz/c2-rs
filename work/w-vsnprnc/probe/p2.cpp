// p2 — unframed leaf FIRST, framed second. THE LIVE ORDER: if the port charges
// the compiler label counter for the leaf, the framed function's $M numbers
// shift and the cell FAILS. With the leaf last (p1) a wrong charge is inert.
int gg(int);
int cal5(int, int, int, int, int);
int leaf5(int a, int b, int c, int d, int e) { return cal5(a, b, c, d, e); }
int framed(int a) { return gg(a) + 1; }
