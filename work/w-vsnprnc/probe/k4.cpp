// k4 — identity passthrough tail call, 4 args, extern "C", workload flags.
// The CONTROL for the forwarding-leaf probes: isolates ARITY from the literal
// and from the permutation.
extern "C" {
int cal4(int, int, int, int);
int fwd4(int p0, int p1, int p2, int p3) { return cal4(p0, p1, p2, p3); }
}
