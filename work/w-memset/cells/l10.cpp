// GRID-L l10 — CONTROL, A LOOP THAT EMITS. The same loop skeleton with a store
// as its body and no call at all. The reader is not "any loop with a matched
// label set": it must refuse this one, and c2 emits real code for it.
struct false_tag {};

inline void aux(int* f, int* l, const false_tag&) { for (; f != l; ++f) *f = 0; }
inline void dr(int* f, int* l, int*) { aux(f, l, false_tag()); }
inline void destroy_range(int* f, int* l) { dr(f, l, (int*)0); }

void use(int* a, int* b) { destroy_range(a, b); }
