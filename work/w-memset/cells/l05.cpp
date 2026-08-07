// GRID-L l05 — TWO STATEMENTS IN THE LOOP BODY. "Emits nothing" is a property
// of the whole body, so a loop that also stores must be refused even though its
// call is elidable.
struct false_tag {};

int sink;
inline void leaf(int*) {}
inline void aux(int* f, int* l, const false_tag&) { for (; f != l; ++f) { leaf(f); sink = 1; } }
inline void dr(int* f, int* l, int*) { aux(f, l, false_tag()); }
inline void destroy_range(int* f, int* l) { dr(f, l, (int*)0); }

void use(int* a, int* b) { destroy_range(a, b); }
