// q1 — a framed function and a leaf that TAIL-CALLS IT: an intra-TU REL24 on a
// `b`, not a `bl`. Minimal.
int outside_callee_long(int);
int framed_target_long(int a, int b, int c, int d, int e) { return outside_callee_long(a) + b + c + d + e; }
int leaf_forwarder_long(int a, int b, int c, int d, int e) { return framed_target_long(a, b, c, d, e); }
