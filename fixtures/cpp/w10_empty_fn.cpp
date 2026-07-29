// Empty function bodies — the `body-0x3A` census bucket (4.4% of all blocked
// functions in the dc3 workload, and by name mostly STL/container plumbing:
// ?_M_initialize@…, ?_M_destroy@…, and every trivial destructor).
//
// A body that does nothing is the function-level analogue of the empty TU
// (mvp_empty.cpp): there is no expression to select, so it is reachable without
// any new instruction selection. The `.ex` body opens directly with the `3A`
// assign of the return plumbing, with no expression before it.
//
// Include-free by design (fixtures/README.md).

void nothing() {}

void nothing_with_args(int a, int b) {}
