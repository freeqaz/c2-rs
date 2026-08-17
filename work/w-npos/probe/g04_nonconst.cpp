// g04 — the same shape NON-const: does the section become a `.data` COMDAT,
// and what in the IL container distinguishes it from g03's record?
template <class T> struct B { static unsigned x; };
template <class T> unsigned B<T>::x = 5u;
inline unsigned* anchor() { return &B<char>::x; }
