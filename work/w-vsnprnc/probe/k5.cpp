// k5 — identity passthrough tail call, 5 args, extern "C", workload flags.
// The CONTROL for the forwarding-leaf probes: isolates ARITY from the literal
// and from the permutation.
extern "C" {
int cal5(int, int, int, int, int);
int fwd5(int p0, int p1, int p2, int p3, int p4) { return cal5(p0, p1, p2, p3, p4); }
}
