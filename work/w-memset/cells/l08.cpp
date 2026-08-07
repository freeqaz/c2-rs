// GRID-L l08 — A DEAD-TEMPORARY CALL INSIDE THE LOOP. This is the workload's
// own level 3 -> level 4 edge: the loop's single statement is not a plain call
// but the tag-dispatch call `w-inl0` already reads. The two readers must
// COMPOSE, and the leaf below is empty so the whole chain closes.
struct false_tag {};

inline void leaf(int*, const false_tag&) {}
inline void destroy_one(int* p) { leaf(p, false_tag()); }
inline void aux(int* f, int* l, const false_tag&) { for (; f != l; ++f) destroy_one(f); }
inline void dr(int* f, int* l, int*) { aux(f, l, false_tag()); }
inline void destroy_range(int* f, int* l) { dr(f, l, (int*)0); }

void use(int* a, int* b) { destroy_range(a, b); }
