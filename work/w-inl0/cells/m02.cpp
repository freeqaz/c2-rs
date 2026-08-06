// GRID-M m02 — THE CALLEE CONDITION. Identical to m01 except that `aux` keeps
// bytes. The chain must NOT close: `dr` emits the store, `destroy_range` keeps
// its branch.
struct true_tag {};

template <class I>
inline void aux(I f, I, const true_tag&) { *f = 7; }

template <class I, class T>
inline void dr(I f, I l, T*) { aux(f, l, true_tag()); }

template <class I>
inline void destroy_range(I f, I l) { dr(f, l, (int*)0); }

void use(int* a, int* b) { destroy_range(a, b); }
