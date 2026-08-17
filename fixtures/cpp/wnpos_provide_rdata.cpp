// w-npos — the provide-always `.rdata` COMDAT TU (decomp_pch.cpp's class in
// one line): an explicitly-instantiated template static const member. c2
// emits the four-section shell plus ONE `sel=2` `.rdata` COMDAT, 4 bytes
// ff ff ff ff, EXTERNAL symbol, real aux CRC — and zero `.text`.
template <class T> struct B { static const unsigned npos; };
template <class T> const unsigned B<T>::npos = ~0u;
template const unsigned B<char>::npos;
