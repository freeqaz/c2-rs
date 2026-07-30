// **Negative** — the arithmetic / bit-twiddling half of the `0x40` intrinsic id
// space. The intrinsics must keep refusing; the four `x_*` functions at the
// bottom are ordinary calls and stay in class.
//
// Two things this fixture separates that a single intrinsic cannot.
//
// **(1) The id is per name FAMILY, not per signature.** `abs(int)` and
// `labs(long)` both emit selector **15**, and differ only in the TYPE fields —
// the result annotation is `86 41 74` (int) for one and `86 41 12` (long) for the
// other, with identical `.text`. So a decoder cannot infer the operand width
// from the id, and an allow-list keyed on the id alone would accept an `_abs64`
// that happened to reuse a 32-bit id. (It does not: `_abs64` is its own id, 815.)
//
// **(2) A name that looks like an intrinsic usually is not.** `fabsf`, `sqrtf`,
// `_rotl16` and `_MulHigh` are declared here exactly like their neighbours and
// c1xx emits an **ordinary `26 <tok> BD … 4C` call** for each — a REL24 tail
// branch, no `0x40` anywhere. That is why the id space is allow-listed from
// captures rather than derived from the CRT header set: the table is internal to
// c1xx and its membership is not guessable from the declaration.
//
// Selector ids and the exact `.text`, read off the reference obj of this file
// (see docs/IL_INTRINSIC_CALL.md §3):
//
//   b_abs        15 / 0x00f    `33 86 41 74 0f`             srawi r11,r3,31 ; xor r10,r3,r11 ; subf r3,r11,r10
//   b_labs       15 / 0x00f    (same selector, `long` types) identical bytes
//   b_rotl      159 / 0x09f                                 rlwnm r3,r3,r4,0,31
//   b_rotr      160 / 0x0a0                                 subfic r11,r4,32 ; rlwnm r3,r3,r11,0,31
//   b_emul      236 / 0x0ec                                 extsw r11,r3 ; extsw r10,r4 ; mulld r3,r11,r10
//   b_emulu     237 / 0x0ed                                 rldicl ; rldicl ; mulld
//   b_rotl64    813 / 0x32d                                 rldcl r3,r3,r4,0
//   b_rotr64    814 / 0x32e                                 subfic r11,r4,64 ; rldcl r3,r3,r11,0
//   b_abs64     815 / 0x32f                                 sradi r11,r3,63 ; xor r10,r3,r11 ; subf r3,r11,r10
//   b_bswap16   839 / 0x347                                 rlwinm ; rlwimi ; or                 (3)
//   b_bswap32   840 / 0x348                                 rlwinm ; rlwimi x3 ; or              (5)
//   b_bswap64   841 / 0x349                                 std/lwz + 8x rlwimi/rlwinm + rldimi (14)
//   b_clz       850 / 0x352                                 cntlzw r3,r3
//   b_clz64     921 / 0x399                                 cntlzd r3,r3
//   b_frsqrte  1935 / 0x78f                                 frsqrte f1,f1
//   b_fsel     1937 / 0x791                                 fsel f1,f1,f2,f3
//   b_ilinc     226 / 0x0e2                                 an 8-instruction lwarx/stwcx. loop
//
// `docs/IL_CAST_CONVERT.md` §1.5 listed 815 as "one long long argument, long long
// result — plausibly `_abs64`. UNKNOWN". `b_abs64` pins it, and locates 813/814
// as its neighbours in the same 64-bit cluster.
//
// **None of this is lowerable yet, including the one-instruction cases.** The
// operand→register mapping is not the identity: `fabs(a) + b` emits
// `fabs f0,f1 ; fadd f1,f0,f2` (result in f0, not f1) while a bare `fabs(a)`
// emits `fabs f1,f1`, so the destination is chosen by the *consumer*, and
// `abs(a)+1` and `abs(abs(a))` pick different scratch registers for the same
// three-instruction expansion. Lowering any of them needs the W5 scratch model
// applied to an intrinsic result, which no capture covers.

extern "C" {
int abs(int);
long labs(long);
unsigned int _rotl(unsigned int, int);
unsigned int _rotr(unsigned int, int);
__int64 __emul(int, int);
unsigned __int64 __emulu(unsigned int, unsigned int);
unsigned __int64 _rotl64(unsigned __int64, int);
unsigned __int64 _rotr64(unsigned __int64, int);
__int64 _abs64(__int64);
unsigned short _byteswap_ushort(unsigned short);
unsigned long _byteswap_ulong(unsigned long);
unsigned __int64 _byteswap_uint64(unsigned __int64);
int _CountLeadingZeros(unsigned long);
int _CountLeadingZeros64(unsigned __int64);
double __frsqrte(double);
double __fsel(double, double, double);
long _InterlockedIncrement(long volatile *);
// declared identically, and NOT intrinsics
float fabsf(float);
float sqrtf(float);
unsigned short _rotl16(unsigned short, unsigned char);
int _MulHigh(int, int);
}

int b_abs(int a) { return abs(a); }
long b_labs(long a) { return labs(a); }
unsigned int b_rotl(unsigned int a, int n) { return _rotl(a, n); }
unsigned int b_rotr(unsigned int a, int n) { return _rotr(a, n); }
__int64 b_emul(int a, int b) { return __emul(a, b); }
unsigned __int64 b_emulu(unsigned int a, unsigned int b) { return __emulu(a, b); }
unsigned __int64 b_rotl64(unsigned __int64 a, int n) { return _rotl64(a, n); }
unsigned __int64 b_rotr64(unsigned __int64 a, int n) { return _rotr64(a, n); }
__int64 b_abs64(__int64 a) { return _abs64(a); }
unsigned short b_bswap16(unsigned short a) { return _byteswap_ushort(a); }
unsigned long b_bswap32(unsigned long a) { return _byteswap_ulong(a); }
unsigned __int64 b_bswap64(unsigned __int64 a) { return _byteswap_uint64(a); }
int b_clz(unsigned long a) { return _CountLeadingZeros(a); }
int b_clz64(unsigned __int64 a) { return _CountLeadingZeros64(a); }
double b_frsqrte(double a) { return __frsqrte(a); }
double b_fsel(double a, double b, double c) { return __fsel(a, b, c); }
long b_ilinc(long volatile *p) { return _InterlockedIncrement(p); }

float x_fabsf(float a) { return fabsf(a); }
float x_sqrtf(float a) { return sqrtf(a); }
unsigned short x_rotl16(unsigned short a, unsigned char n) { return _rotl16(a, n); }
int x_mulhigh(int a, int b) { return _MulHigh(a, b); }
