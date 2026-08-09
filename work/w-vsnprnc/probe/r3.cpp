// r3 — two leaves, the second tail-calling the first: an intra-TU `b` with no
// framed function in the TU at all.
int gg(int);
int leaft(int a) { return gg(a); }
int leafi(int a) { return leaft(a); }
