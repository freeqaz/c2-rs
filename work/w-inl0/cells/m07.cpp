// GRID-M m07 — CONTROL, the same-TU condition. The call the temporary is built
// for goes to a function this TU only declares, so c2 keeps a REL24 and nothing
// may be elided. Dropping the same-TU test turns this cell into a wrong emit.
struct true_tag {};

void aux_ext(int*, int*, const true_tag&);

inline void dr(int* f, int* l, int*) { aux_ext(f, l, true_tag()); }
inline void destroy_range(int* f, int* l) { dr(f, l, (int*)0); }

void use(int* a, int* b) { destroy_range(a, b); }
