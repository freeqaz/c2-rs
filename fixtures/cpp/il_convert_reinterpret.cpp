// The `2C` WIDTH-4 REINTERPRET — lane w-convert, board #700.
//
// `crates/c2-il/src/func/readers.rs::eat_reinterpret_type` admits a conversion
// whose target names the *other* width-4 value class. Measured, c2 emits nothing
// at all for the pair — a bare `blr` in every spelling below, at `/Ox` and at the
// workload's `/O1 /Oi /EHsc /GR` alike.
//
// **Expected verdict: `Port=Match`, byte-exact against real c2.** Every function
// here is one the port takes whole, so the differential grades the whole obj
// rather than declining it. The four class pairs that are NOT free, and the
// scaled pointer arithmetic a converted pointer can reach, live in
// `il_convert_reinterpret_neg.cpp` — a separate file, because a fixture's verdict
// is per-OBJ and one declined function would have hidden this one's
// byte-exactness behind its own negative controls.
//
// ---- what this fixture exists to express that nothing else could -------------
//
// `sweep.d/61-conversion-2c.py` already crosses the *class-preserving* convert.
// Four shapes that would break the reinterpret could not be written at all
// before this rung, because `saw_ptr` was only ever set by a LOAD:
//
//   * a POINTER PRODUCED BY A CONVERSION then doing pointer arithmetic — c2
//     SCALES: `(S *)a + 1` is `addi r3,r3,8` and `(S *)a + k` is
//     `slwi r11,r4,3 ; add`. An accepted chain that added unscaled is the
//     wrong-emit this whole rung is one line away from. In the `_neg` file;
//   * a SIGNED `int` source at the 32->64 boundary (`cr_i2p`), where an
//     `extsw`/`rldicl` would appear if it appeared anywhere;
//   * a FUNCTION pointer (`kind` class 4, not 3) on both sides;
//   * `bool` on either side of the conversion, which looks like a peer of the
//     other two classes in the `ValueClass` enum and is not. In the `_neg` file.

struct S { int m; int n; };
typedef void (*FnPtr)();
typedef int myint;

int  g1(int);
int  g2(int, int);
int  g4(int, int, int, int);
int  gp(void *);
int  gpi(void *, int, void *);

// ---- ptr -> int: every integer spelling ------------------------------------
unsigned      cr_p2u  (void *p)         { return (unsigned)p; }
int           cr_p2i  (void *p)         { return (int)p; }
long          cr_p2l  (void *p)         { return (long)p; }
unsigned long cr_p2ul (void *p)         { return (unsigned long)p; }
myint         cr_p2t  (void *p)         { return (myint)p; }

// ---- ptr -> int: every pointee, including the function pointer -------------
int           cr_pi2i (int *p)          { return (int)p; }
int           cr_pc2i (char *p)         { return (int)p; }
int           cr_pk2i (const char *p)   { return (int)p; }
int           cr_ps2i (S *p)            { return (int)p; }
int           cr_pp2i (S **p)           { return (int)p; }
int           cr_pf2i (FnPtr f)         { return (int)f; }
unsigned      cr_pf2u (FnPtr f)         { return (unsigned)f; }
int           cr_pks2i(const S *p)      { return (int)p; }

// ---- int -> ptr: every integer spelling, SIGNED included -------------------
void *        cr_u2p  (unsigned a)      { return (void *)a; }
void *        cr_i2p  (int a)           { return (void *)a; }
void *        cr_l2p  (long a)          { return (void *)a; }
void *        cr_ul2p (unsigned long a) { return (void *)a; }
void *        cr_t2p  (myint a)         { return (void *)a; }

// ---- int -> ptr: every pointee ---------------------------------------------
int *         cr_i2pi (int a)           { return (int *)a; }
char *        cr_i2pc (int a)           { return (char *)a; }
const char *  cr_i2pk (int a)           { return (const char *)a; }
S *           cr_i2ps (int a)           { return (S *)a; }
S **          cr_i2pp (int a)           { return (S **)a; }
FnPtr         cr_i2pf (int a)           { return (FnPtr)a; }
const S *     cr_i2pks(int a)           { return (const S *)a; }

// ---- the CALL-ARGUMENT position, at every slot of a four-argument tail call -
int cr_a0 (S *p, int b, int c, int d)   { return g4((int)p, b, c, d); }
int cr_a1 (int a, S *p, int c, int d)   { return g4(a, (int)p, c, d); }
int cr_a2 (int a, int b, S *p, int d)   { return g4(a, b, (int)p, d); }
int cr_a3 (int a, int b, int c, S *p)   { return g4(a, b, c, (int)p); }
int cr_ai2p(int a)                      { return gp((void *)a); }
int cr_amix(int a, int b, int c)        { return gpi((void *)a, b, (void *)c); }
int cr_aperm(S *p, int b)               { return g2(b, (int)p); }

// ---- `this`, which is the const pointer `A6 43` ----------------------------
struct C {
    int m;
    int      ci() const;
    unsigned cu() const;
    void *   cp() const;
    int      cg(int) const;
};
int      C::ci() const      { return (int)this; }
unsigned C::cu() const      { return (unsigned)this; }
void *   C::cp() const      { return (void *)this; }
int      C::cg(int a) const { return g2((int)this, a); }

// ---- stacked conversions: the round trip, and the mixed-spelling chain -----
int      cr_st1(int a)      { return (int)(void *)a; }
int      cr_st2(void *p)    { return (int)(void *)(int)p; }
unsigned cr_st3(S *p)       { return (unsigned)(long)p; }
