// GRID-L l03 — THE LOOP'S CALLEE KEEPS BYTES. Same loop, and the leaf stores to
// a global. Nothing in the chain reduces to nothing and the port must convert
// none of it: what propagates is "emits nothing", not "is a loop".
struct false_tag {};

int sink;
inline void leaf(int* p) { sink = *p; }
inline void aux(int* f, int* l, const false_tag&) { for (; f != l; ++f) leaf(f); }
inline void dr(int* f, int* l, int*) { aux(f, l, false_tag()); }
inline void destroy_range(int* f, int* l) { dr(f, l, (int*)0); }

void use(int* a, int* b) { destroy_range(a, b); }
