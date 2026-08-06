// GRID-M m05 — THE FIXPOINT, one level deeper than m01. `destroy_range2` reaches
// nothing only through the refused `dr`, so this cell fails if the no-effect
// fact is a one-step rule rather than a link into the closure.
struct true_tag {};

template <class I>
inline void aux(I, I, const true_tag&) {}

template <class I, class T>
inline void dr(I f, I l, T*) { aux(f, l, true_tag()); }

template <class I>
inline void destroy_range(I f, I l) { dr(f, l, (int*)0); }

template <class I>
inline void destroy_range2(I f, I l) { destroy_range(f, l); }

void use(int* a, int* b) { destroy_range2(a, b); }
