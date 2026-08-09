// k6 — identity passthrough tail call, 6 args, extern "C", workload flags.
// The CONTROL for the forwarding-leaf probes: isolates ARITY from the literal
// and from the permutation.
extern "C" {
int cal6(int, int, int, int, int, int);
int fwd6(int p0, int p1, int p2, int p3, int p4, int p5) { return cal6(p0, p1, p2, p3, p4, p5); }
}
