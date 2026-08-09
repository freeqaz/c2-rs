// u3 — identity forwarding, extern "C", all int: the CONTROL. If this refuses,
// nothing about the literal or the permutation is what refuses u1.
extern "C" {
int callee5(int, int, int, int, int);
int fwd(int b, int n, int f, int lo, int ap) { return callee5(b, n, f, lo, ap); }
}
