// GRID-L l04 — CONTROL, the loop's callee is EXTERNAL. The same-TU condition,
// one level inside the loop. c2 keeps a relocation somewhere in the chain and
// the reader must admit nothing.
struct false_tag {};

void ext_leaf(int*);
inline void aux(int* f, int* l, const false_tag&) { for (; f != l; ++f) ext_leaf(f); }
inline void dr(int* f, int* l, int*) { aux(f, l, false_tag()); }
inline void destroy_range(int* f, int* l) { dr(f, l, (int*)0); }

void use(int* a, int* b) { destroy_range(a, b); }
