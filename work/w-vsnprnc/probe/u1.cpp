// u1 — all-int forwarding tail call with a literal in a middle slot, extern "C".
extern "C" {
int callee5(int, int, int, int, int);
int fwd(int b, int n, int f, int ap) { return callee5(b, n, f, 0, ap); }
}
