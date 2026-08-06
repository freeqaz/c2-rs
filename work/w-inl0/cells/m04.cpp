// GRID-M m04 — A SECOND STATEMENT. The body is the m01 call plus a store, so the
// reader's walk is no longer total over the segment and it must refuse. This is
// the cell that says "emits nothing" is a property of the WHOLE body.
struct true_tag {};

template <class I>
inline void aux(I, I, const true_tag&) {}

template <class I, class T>
inline void dr(I f, I l, T*) { aux(f, l, true_tag()); *f = 1; }

template <class I>
inline void destroy_range(I f, I l) { dr(f, l, (int*)0); }

void use(int* a, int* b) { destroy_range(a, b); }
