// GRID-L l06 — THE INDUCTION STEP IS NOT PURE. `f = advance(f)` puts a call in
// the increment, so the loop's own step is no longer one lvalue, one literal
// and one operator. The reader must refuse the body rather than read past it.
struct false_tag {};

inline void leaf(int*) {}
inline int* advance(int* p) { return p + 1; }
inline void aux(int* f, int* l, const false_tag&) { for (; f != l; f = advance(f)) leaf(f); }
inline void dr(int* f, int* l, int*) { aux(f, l, false_tag()); }
inline void destroy_range(int* f, int* l) { dr(f, l, (int*)0); }

void use(int* a, int* b) { destroy_range(a, b); }
