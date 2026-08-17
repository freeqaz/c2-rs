// g05 — TWO const template statics, sizes 1 and 8: emission order, per-size
// alignment bits, per-section checksum.
template <class T> struct B {
    static const unsigned char c;
    static const unsigned long long q;
};
template <class T> const unsigned char B<T>::c = 0xAA;
template <class T> const unsigned long long B<T>::q = 0x1122334455667788ULL;
inline const void* anchor() { return &B<char>::c; }
inline const void* anchor2() { return &B<char>::q; }
