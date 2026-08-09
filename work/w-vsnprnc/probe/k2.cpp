// k2 — identity passthrough tail call, 2 args, extern "C", workload flags.
// The CONTROL for the forwarding-leaf probes: isolates ARITY from the literal
// and from the permutation.
extern "C" {
int cal2(int, int);
int fwd2(int p0, int p1) { return cal2(p0, p1); }
}
