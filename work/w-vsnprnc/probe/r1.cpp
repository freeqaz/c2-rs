// r1 — p1's shapes, but the leaf tail-calls the framed function IN THIS TU.
// An intra-TU REL24 on a `b`. p1 (same two shapes, external target) matches.
int gg(int);
int framed(int a) { return gg(a) + 1; }
int leafi(int a) { return framed(a); }
