// W20 negative — the boundary of the free `2C` conversion.
//
// EVERY function here must be **out of class**, and this file must never
// `Mismatch`. It is the other half of `w20_convert.cpp`: that file admits a
// conversion whose target is the value's own [`ValueClass`], and every function
// below is a conversion that is NOT that, split into the two reasons it is not.
//
// 1. **It emits something.** Read off the reference obj (`/Ox /GS- /c`) — each is
//    a real instruction the modeled chain has no way to produce:
//
//      char  f(int a) { return (char)a; }            7c630774  extsb r3,r3
//      short f(int a) { return (short)a; }           7c630734  extsh r3,r3
//      unsigned char  f(int a)                       5463063e  rlwinm r3,r3,0,24,31
//      unsigned short f(int a)                       5463043e  rlwinm r3,r3,0,16,31
//      long long f(int a)                            7c6307b4  extsw r3,r3
//      float f(int a)                                a five-instruction stack round trip
//
// 2. **It is free, and refused anyway** — the honest half. The cross-class
//    reinterpret between a 4-byte integer and a 4-byte pointer is a bare `blr` in
//    both directions:
//
//      int f(S* p) { return (int)p; }                4e800020  blr
//      S*  f(int a){ return (S*)a; }                 4e800020  blr
//
//    It is refused because the port has never graded a reinterpret across the
//    widths, cv-spellings and argument positions this parser reaches, and because
//    the neighbour that would look identical under a laxer rule — an
//    address-adjusting up/downcast, which costs an `addi` — is one byte away in
//    the source language even though it arrives as an intrinsic rather than a
//    `2C`. `expr-convert-target-8641` / `-8643` count what the conservatism costs,
//    so it is a number and not an argument. Widening it is a rung with its own
//    grading, not a tidy-up.
//
// Freestanding, include-free.

struct S { int a; int b; };

int gv(void *);
int g1(int);

// ---- 1. conversions that emit an instruction --------------------------------
char           nb_char(int a)          { return (char)a; }
short          nb_short(int a)         { return (short)a; }
unsigned char  nb_uchar(int a)         { return (unsigned char)a; }
unsigned short nb_ushort(int a)        { return (unsigned short)a; }
long long      nb_ll(int a)            { return (long long)a; }
unsigned long long nb_ull(int a)       { return (unsigned long long)a; }
float          nb_float(int a)         { return (float)a; }
double         nb_double(int a)        { return (double)a; }
char           nb_char_chain(int a, int b) { return (char)(a + b); }
short          nb_short_arg(int a)     { return (short)g1(a); }

// ---- 2. the cross-class reinterpret, free but ungraded ----------------------
int   nx_i_of_p(S *s)                  { return (int)s; }
S    *nx_p_of_i(int a)                 { return (S *)a; }
unsigned nx_u_of_p(S *s)               { return (unsigned)s; }
int   nx_arg(S *s)                     { return g1((int)s); }

// ---- 3. the conversion is free but its VALUE then does pointer arithmetic ---
// `parse_expr`'s guard (§21) refuses a pointer operand in any modeled chain, and
// it must keep refusing when the pointer arrived through a conversion rather than
// straight off a LOAD. `nptr_add` refuses one token EARLIER than the guard, on
// the conversion itself, because the tracked class is the last operand's and the
// last operand of `s + 1` is the literal: the two disagree only when there is
// arithmetic, and arithmetic over a pointer is exactly what the guard refuses, so
// the body fails closed either way. The key is `expr-convert-target-8643` rather
// than `expr-ptr-arith` — an attribution difference, not an acceptance one.
void *nptr_add(S *s)                   { return (void *)(s + 1); }
int   nptr_diff(S *s, S *t)            { return (int)(s - t); }

// ---- 4. a cv-qualified operand TYPE, which blocks ahead of the conversion ----
// `const int a` spells its LOAD `A6 41 <per-TU id>`, which is not one of the four
// int triples `eat_int_like` admits, so the parse stops at the operand and never
// reaches the `2C`. A different gap with a different key
// (`expr-load-type-A641`), kept here so the two are not confused.
int   ncv_const_param(const int a)     { return (int)a; }

// ---- 5. the float side, where the operand type blocks first -----------------
int   nf_of_float(float a)             { return (int)a; }
unsigned nf_of_double(double a)        { return (unsigned)a; }
