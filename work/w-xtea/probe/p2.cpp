// w-xtea probe P2 — the trailing varint of `2C` (CONVERT) when the TARGET type
// is 8-byte. The XTEA stream only witnesses `2c <4-byte target> 00`; whether an
// 8-byte target widens that trailer is UNMEASURED and must not be guessed.
typedef unsigned long long ull;
ull  f_up(unsigned a) { return (ull)a; }        // 4-byte -> 8-byte target
unsigned f_dn(ull a)  { return (unsigned)a; }   // 8-byte -> 4-byte target (control)
double f_d(int a)     { return (double)a; }     // tag-88 non-integer target
