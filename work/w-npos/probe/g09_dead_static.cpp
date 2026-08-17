// g09 — g03 plus an internal-linkage uninitialized unreferenced static, which
// c2 drops entirely (measured rule, `gl::DATA_FLAG_REFERENCED`). Prediction:
// same obj shape as g03; the predicate's drop clause composes.
template <class T> struct B { static const unsigned npos; };
template <class T> const unsigned B<T>::npos = ~0u;
inline const unsigned* anchor() { return &B<char>::npos; }
static int dead_never_referenced;
