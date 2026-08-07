// GRID-L l11 — THE CYCLE THROUGH A LOOP. `aux` calls `dr` and `dr` calls `aux`.
// Nothing seeds it, so the least fixpoint must admit neither member and must
// terminate; the round ceiling must not fire.
struct false_tag {};

inline void dr2(int* f, int* l);
inline void aux(int* f, int* l, const false_tag&) { for (; f != l; ++f) dr2(f, l); }
inline void dr2(int* f, int* l) { aux(f, l, false_tag()); }
inline void destroy_range(int* f, int* l) { dr2(f, l); }

void use(int* a, int* b) { destroy_range(a, b); }
