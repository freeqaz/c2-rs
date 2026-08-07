// GRID-L l02 — THE LOOP OVER AN EMPTY-BODIED CALLEE. The positive cell: the
// loop's call names a leaf whose body is `empty_body`, so the existing seed is
// reachable and the whole chain must close through the loop LINK alone, with no
// change to E's rule.
struct false_tag {};

inline void leaf(int*) {}
inline void aux(int* f, int* l, const false_tag&) { for (; f != l; ++f) leaf(f); }
inline void dr(int* f, int* l, int*) { aux(f, l, false_tag()); }
inline void destroy_range(int* f, int* l) { dr(f, l, (int*)0); }

void use(int* a, int* b) { destroy_range(a, b); }
