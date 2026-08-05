// The `2C` WIDTH-4 REINTERPRET, the REFUSING half — lane w-convert, board #700.
//
// The positive half is `il_convert_reinterpret.cpp`, which the port takes whole
// and which the differential grades byte-exact. This file is the other side of
// the same boundary and its expected verdict is `Port=NotImplemented`: every
// function below emits a real instruction that a chain dropping the conversion
// would omit, so a `Port=Mismatch` here is the alarm, not a gap.
//
// The two are separate files because a fixture's verdict is per-OBJ: one
// declined function makes the whole TU `NotImplemented`, which would have hidden
// the positive half's byte-exactness behind its own negative controls.

struct S { int m; int n; };
typedef void (*FnPtr)();

// must decline, not drop the conversion.
//
// `(S *)a + 1` is `addi r3,r3,8` and `(S *)a + k` is `slwi r11,r4,3 ; add`:
// c2 SCALES pointer arithmetic, and these two reach the scaling through a
// CONVERSION rather than off a LOAD, which nothing else in the corpus does.
S *   cr_pa_lit1(int a)          { return (S *)a + 1; }
char *cr_pa_litc(int a)          { return (char *)a + 1; }
int  *cr_pa_liti(int a)          { return (int *)a + 1; }
S *   cr_pa_var (int a, int k)   { return (S *)a + k; }
// the same arithmetic on the int side of the conversion, which is NOT scaled —
// c2 emits a plain `add`. The port refuses it too (the guard is on the whole
// sub-expression), and what that costs is these four functions.
int   cr_ia_add (void *p, int b) { return (int)p + b; }
int   cr_ia_sub (void *p, int b) { return (int)p - b; }
void *cr_ta_add (int a, int b)   { return (void *)(a + b); }
int   cr_ta_two (void *p, void *q) { return (int)p + (int)q; }
// `bool` on either side: `unsigned u(bool b)` is `rlwinm r3,r3,0,24,31`, and so
// is `(void *)b`. The pointer direction is NOT the free one the enum suggests.
int      cr_b2i (bool b)         { return b; }
unsigned cr_b2u (bool b)         { return b; }
void *   cr_b2p (bool b)         { return (void *)b; }
unsigned char cr_p2uc(S *p)      { return (unsigned char)p; }
// and the width boundary, from an int source: each of these is one instruction.
char           cr_i2c (int a)    { return (char)a; }
short          cr_i2s (int a)    { return (short)a; }
unsigned short cr_i2us(int a)    { return (unsigned short)a; }
unsigned char  cr_i2uc(int a)    { return (unsigned char)a; }
long long      cr_i2ll(int a)    { return (long long)a; }
float          cr_i2f (int a)    { return (float)a; }

// ---- NOT a conversion refusal, and it is here for exactly that reason ------
//
// `g2((int)p, (int)p)` refuses under `call-arg-duplicated`, a pre-existing gap
// in the call-argument region that has nothing to do with the `2C`. It sits in
// the negative file rather than the positive one because a fixture's verdict is
// per-obj: leaving it beside the accepted cells would have turned the whole
// positive TU `NotImplemented` and hidden the byte-exactness this rung claims,
// which is exactly the failure mode the split exists to prevent. Naming the key
// here keeps it a stated exclusion instead of a silently dropped case.
int g2(int, int);
int cr_arep(S *p) { return g2((int)p, (int)p); }
