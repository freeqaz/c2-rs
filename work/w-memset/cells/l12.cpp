// GRID-L l12 — THE INDUCTION AND THE COMPARISON, VARIED. Stride 2 and a `<`
// test instead of `++` and `!=`. Structurally the same loop; if the reader is
// keyed on the literal `4` or on the compare opcode of l02, this cell says so.
struct false_tag {};

inline void leaf(int*) {}
inline void aux(int* f, int* l, const false_tag&) { for (; f < l; f += 2) leaf(f); }
inline void dr(int* f, int* l, int*) { aux(f, l, false_tag()); }
inline void destroy_range(int* f, int* l) { dr(f, l, (int*)0); }

void use(int* a, int* b) { destroy_range(a, b); }
