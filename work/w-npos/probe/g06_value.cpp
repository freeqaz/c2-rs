// g06 — g03 with a different value: the content bytes and the aux CheckSum
// must track the `.in` initializer, not a constant.
template <class T> struct B { static const unsigned v; };
template <class T> const unsigned B<T>::v = 0x12345678u;
inline const unsigned* anchor() { return &B<char>::v; }
