// W20 — the `2C` CONVERSION in a general expression position, where it is free.
//
// `docs/IL_CALL_IN_EXPR.md` §24. A `2C <TYPE> 00` whose target is the value's own
// class emits NO instruction: `int`<->`unsigned`/`long` at width 4, and any
// pointer-to-pointer, are register-to-register identities on this target. The
// same rule already gates the indirect-load getter and the pointer-identity leaf
// and has been byte-graded there since those rungs; this file grades it in the
// position that carries the workload population — an operand of `parse_expr`,
// i.e. inside an arithmetic chain and inside a call-argument region.
//
// EVERY function here must be in class AND byte-exact. The refusing boundary —
// narrowings, widenings, the float conversions and the cross-class reinterpret,
// all of which DO emit something or have never been graded — is
// `w20_convert_neg.cpp`, and the two files must be read together.
//
// The conversions are spelled with explicit casts so the construct is visible in
// the source, but c1xx emits the same `2C` for an implicit one; `im1`/`im2` are
// the implicit witnesses and must emit what their explicit twins do.

struct S { int a; int b; };
struct T { int m; };

int  g1(int);
int  g2(int, int);
int  gp(const S *);
int  gv(void *);
int  gpp(S *, void *);

// ---- int4 -> int4, the whole spelling matrix, one operand -------------------
unsigned c_u_of_i(int a)                 { return (unsigned)a; }
int      c_i_of_u(unsigned a)            { return (int)a; }
long     c_l_of_i(int a)                 { return (long)a; }
unsigned long c_ul_of_i(int a)           { return (unsigned long)a; }
int      c_i_of_l(long a)                { return (int)a; }
unsigned c_u_of_ul(unsigned long a)      { return (unsigned)a; }
unsigned im1(int a)                      { return a; }          // implicit
int      im2(unsigned a)                 { return a; }          // implicit

// ---- the conversion at every position of an arithmetic chain ----------------
// This is the axis the leaf shapes could not reach: a conversion with operands
// on both sides of it, and a chain whose operator mix and operand order are the
// layer the `expr_sweep` reassociation bugs lived in.
unsigned ch_lead(int a, int b)           { return (unsigned)a + b; }
unsigned ch_trail(int a, int b)          { return a + (unsigned)b; }
unsigned ch_both(int a, int b)           { return (unsigned)a + (unsigned)b; }
unsigned ch_whole(int a, int b)          { return (unsigned)(a + b); }
unsigned ch_sub(int a, int b)            { return (unsigned)(a - b); }
unsigned ch_mul(int a, int b)            { return (unsigned)(a * b); }
unsigned ch_mid3(int a, int b, int c)    { return a + (unsigned)b + c; }
unsigned ch_deep(int a, int b, int c)    { return (unsigned)(a + b) + c; }
unsigned ch_mixop(int a, int b, int c)   { return (unsigned)a * b + c; }
int      ch_back(unsigned a, unsigned b) { return (int)(a + b); }
int      ch_back2(unsigned a, unsigned b){ return (int)a + (int)b; }

// ---- conversions over literals ---------------------------------------------
unsigned k_lit()                         { return (unsigned)7; }
unsigned k_lit_chain(int a)              { return a + (unsigned)3; }
unsigned k_zero()                        { return (unsigned)0; }

// ---- nested / repeated conversions -----------------------------------------
unsigned n_twice(int a)                  { return (unsigned)(unsigned)a; }
int      n_round(int a)                  { return (int)(unsigned)a; }
unsigned n_round2(int a, int b)          { return (unsigned)(int)(unsigned)(a + b); }

// ---- the conversion at each argument slot (the D10 register move underneath) -
unsigned slot2(int a, int b)             { return (unsigned)b; }
unsigned slot3(int a, int b, int c)      { return (unsigned)c; }
unsigned slot5(int a, int b, int c, int d, int e) { return (unsigned)e; }

// ---- the conversion in a CALL-ARGUMENT region (`parse_expr`'s other caller) --
int      arg_conv(unsigned a)            { return g1((int)a); }
int      arg_conv_chain(int a, int b)    { return g1((int)(a + b)); }
int      arg_conv2(unsigned a, unsigned b) { return g2((int)a, (int)b); }
int      arg_conv_mixed(unsigned a, int b) { return g2((int)a, b); }

// ---- ptr4 -> ptr4, which is the workload's whole call-free-to-tail-call half -
int      p_const(S *s)                   { return gp(s); }            // S* -> const S*
int      p_void(S *s)                    { return gv(s); }            // S* -> void*
int      p_void_c(const S *s)            { return gv((void *)s); }
int      p_two(S *s, S *t)               { return gpp(s, t); }
void    *p_id(S *s)                      { return (void *)s; }
const S *p_addc(S *s)                    { return s; }

// ---- member functions: `this` is r3 and carries a const pointer type ---------
struct C {
    int m;
    unsigned mu(int a) const;
    int      mcall() const;
    unsigned mslot(int a, int b) const;
};
unsigned C::mu(int a) const              { return (unsigned)a; }
int      C::mcall() const                { return gv((void *)this); }
unsigned C::mslot(int a, int b) const    { return (unsigned)b; }

// ---- the locality tell: byte-identical bodies, varied position ---------------
unsigned loc1(int a, int b)              { return (unsigned)(a + b); }
int      loc_pad(int a, int b)           { return a - b; }
unsigned loc2(int a, int b)              { return (unsigned)(a + b); }
