// x04 — POST-FREEZE DIAGNOSTIC: explicit instantiation of the static data
// member alone — the standalone reproduction candidate for decomp_pch's class.
template <class T> struct B { static const unsigned npos; };
template <class T> const unsigned B<T>::npos = ~0u;
template const unsigned B<char>::npos;
