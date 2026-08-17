// w-npos — the suppressed twin (attr 0xA0): a merely-instantiated template
// static is NOT emitted by c2 (the obj is the bare 720-byte shell), and the
// recognizer's emit list is empty, so it refuses. Expected: NotImplemented.
template <class T> struct B { static const unsigned npos; };
template <class T> const unsigned B<T>::npos = ~0u;
inline const unsigned* anchor() { return &B<char>::npos; }
