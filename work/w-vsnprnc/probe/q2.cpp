// q2 — the CONTROL: the same two functions, but the leaf forwards OUTSIDE.
int outside_callee_long(int);
int elsewhere_target_long(int, int, int, int, int);
int framed_target_long(int a, int b, int c, int d, int e) { return outside_callee_long(a) + b + c + d + e; }
int leaf_forwarder_long(int a, int b, int c, int d, int e) { return elsewhere_target_long(a, b, c, d, e); }
