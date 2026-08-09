// w-biquad PROBE — what fixes the `.rdata` pool ORDER under /Gy?
// Cell 1: two constants, first use is 2.5f, second is 7.5f, one block.
// If the order is FIRST-USE, `.rdata` reads 2.5 then 7.5.
struct P1 { float a[4]; void s(); };
void P1::s() { a[0] = 2.5f; a[1] = 7.5f; }
