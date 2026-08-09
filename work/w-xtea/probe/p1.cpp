// w-xtea probe P1 — the width of a tag-0x88 (8-byte) integer LITERAL escape.
// STRUCTURAL axes: literal magnitude (short-form vs escape), signedness of the
// 8-byte type, and the same value at 4-byte width as a control.
typedef unsigned long long ull;
typedef long long ll;
ull f_small(ull a) { return a & 5ull; }                    // short form?
ull f_big(ull a)   { return a & 0xFFFFFFFFull; }           // the XTEA mask
ull f_huge(ull a)  { return a & 0x123456789ABCDEF0ull; }   // all 8 bytes distinct
ll  f_neg(ll a)    { return a + (-5ll); }                  // negative, 8-byte
unsigned f_ctl(unsigned a) { return a & 0xFFFFFFFFu; }     // CONTROL: 4-byte
