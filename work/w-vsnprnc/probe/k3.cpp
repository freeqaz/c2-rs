// k3 — identity passthrough tail call, 3 args, extern "C", workload flags.
// The CONTROL for the forwarding-leaf probes: isolates ARITY from the literal
// and from the permutation.
extern "C" {
int cal3(int, int, int);
int fwd3(int p0, int p1, int p2) { return cal3(p0, p1, p2); }
}
