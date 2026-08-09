// q4 — two LEAVES, the second tail-calling the first: an intra-TU `b` with no
// framed function anywhere.
int outside_callee_long(int);
int leaf_target_long(int a) { return outside_callee_long(a); }
int leaf_forwarder_long(int a) { return leaf_target_long(a); }
