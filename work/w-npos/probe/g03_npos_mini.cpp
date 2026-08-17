// g03 — decomp_pch.cpp in miniature: a template static const data member,
// instantiated by an inline function c2 never emits. Prediction: obj = shell +
// one sel=2 `.rdata` COMDAT, 4 B, ff ff ff ff, zero `.text`.
template <class T> struct B { static const unsigned npos; };
template <class T> const unsigned B<T>::npos = ~0u;
inline const unsigned* anchor() { return &B<char>::npos; }
