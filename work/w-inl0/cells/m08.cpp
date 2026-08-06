// GRID-M m08 — THE CYCLE. Two dead-temporary bodies that call each other. Neither
// is ever SEEDED, so the least fixpoint must admit neither and terminate; c2
// keeps a branch in both. This is board #950's hazard in this rule's own shape.
struct true_tag {};

void b_(int* f, int* l, const true_tag&);
inline void a_(int* f, int* l, const true_tag&) { b_(f, l, true_tag()); }
inline void b_(int* f, int* l, const true_tag&) { a_(f, l, true_tag()); }

void use(int* x, int* y) { a_(x, y, true_tag()); }
