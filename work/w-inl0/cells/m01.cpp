// GRID-M m01 — THE SHAPE. STLport's `_Destroy_Range -> __destroy_range ->
// __destroy_range_aux(true_type)` chain with the names shortened. The tag
// temporary is value-initialized (an intrinsic memset over one byte) and passed
// by const reference to an inline function with an empty body.
struct true_tag {};

template <class I>
inline void aux(I, I, const true_tag&) {}

template <class I, class T>
inline void dr(I f, I l, T*) { aux(f, l, true_tag()); }

template <class I>
inline void destroy_range(I f, I l) { dr(f, l, (int*)0); }

void use(int* a, int* b) { destroy_range(a, b); }
