// W-UNW-1: two leaves then a framed function. Distinguishes "the seed is
// pinned per TU" from "the counter accumulates per function" — with one leaf
// the two are one apart and could be a constant, with two they are two apart.
// The framed body also lands at `.text` 0x10, a third distinct `bl`
// displacement (`4BFFFFE5`).
int g(int);
int l1(int a) { return a + 1; }
int l2(int a) { return a + 3; }
int f(int a) { return g(a) + 5; }
