// w-inl0 EXPLORATORY cell e01 — the STLport `__destroy_range` shape, minimized.
//
// Not a graded grid cell: this exists to READ the `expr-intrinsic-memset`
// production before GRID-M is frozen. The shape is
// `src/system/stlport/stl/_construct.h:172` with the names shortened: a
// value-initialized empty tag temporary passed by `const&` to an inline
// function whose selected overload has an empty body.
struct true_tag {};

template <class I>
inline void aux(I, I, const true_tag&) {}

template <class I, class T>
inline void dr(I first, I last, T*) { aux(first, last, true_tag()); }

template <class I>
inline void destroy_range(I first, I last) { dr(first, last, (int*)0); }

void anchor(int* a, int* b);

void use(int* a, int* b) { destroy_range(a, b); anchor(a, b); }
