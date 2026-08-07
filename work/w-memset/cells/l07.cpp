// GRID-L l07 — THE CONDITION IS OVER A GLOBAL, not over this function's own
// formals. The loop still calls an elidable leaf; the reader must refuse,
// because a body that reads a data symbol is not a body that emits nothing
// (`elide.rs` condition 3, one level down).
struct false_tag {};

int* stop_at;
inline void leaf(int*) {}
inline void aux(int* f, int*, const false_tag&) { for (; f != stop_at; ++f) leaf(f); }
inline void dr(int* f, int* l, int*) { aux(f, l, false_tag()); }
inline void destroy_range(int* f, int* l) { dr(f, l, (int*)0); }

void use(int* a, int* b) { destroy_range(a, b); }
