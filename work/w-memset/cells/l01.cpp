// GRID-L l01 — THE SHAPE, all five levels. STLport's
// `_Destroy_Range -> __destroy_range -> __destroy_range_aux(__false_type) ->
// _Destroy -> __destroy_aux(__false_type)` for a CLASS element type, with the
// names shortened. Levels 3 and 5 are the two productions this lane reads; the
// fifth has no call in it at all and is the registered STOP.
struct S { int a; };
struct false_tag {};

template <class T> inline void destroy_aux(T* p, const false_tag&) { p->~T(); }
template <class T> inline void destroy_one(T* p) { destroy_aux(p, false_tag()); }
template <class I> inline void aux(I f, I l, const false_tag&) { for (; f != l; ++f) destroy_one(f); }
template <class I, class T> inline void dr(I f, I l, T*) { aux(f, l, false_tag()); }
template <class I> inline void destroy_range(I f, I l) { dr(f, l, (S*)0); }

void use(S* a, S* b) { destroy_range(a, b); }
