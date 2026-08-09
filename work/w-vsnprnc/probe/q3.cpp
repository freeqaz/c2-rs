// q3 — the leaf FIRST and the intra-TU target second: a FORWARD reference,
// which is the order vsnprnc.cpp does NOT use. Separates "intra-TU" from
// "backward reference".
int outside_callee_long(int);
int framed_target_long(int, int, int, int, int);
int leaf_forwarder_long(int a, int b, int c, int d, int e) { return framed_target_long(a, b, c, d, e); }
int framed_target_long(int a, int b, int c, int d, int e) { return outside_callee_long(a) + b + c + d + e; }
