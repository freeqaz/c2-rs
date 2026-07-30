// **Negative** — everything one token away from the pointer operand admitted in
// `w17_ptr_operand.cpp`. Every function here must keep refusing
// (`NotImplemented`), and the file must never produce a `mismatch`.
//
// `docs/IL_CALL_IN_EXPR.md` §21. The positive file's whole claim is that a
// 4-byte pointer VALUE is a 4-byte int value — true in a register and false
// everywhere the pointee's width is involved. This file is the boundary.
//
// ## The arithmetic guard, and why it is here rather than in the positive file
//
// `p + 1` on an `int*` is `addi r3,r3,4`, not 1: pointer arithmetic scales by
// the pointee's width, so an add chain that used the source literal would emit
// wrong bytes for every width but `char`.
//
// MEASURED, and it is the opposite of what the hazard as first stated assumed:
// c1xx **pre-scales**, so `int* f(int* p){ return p + 1; }` captures as
//
//     B9 p 86 43 f4 08 · 33 86 41 12 04 · 02 · 41 86 43 f4 08
//                                     ^^ the literal is already 4
//
// with `char*` carrying 1, `double*` 8, and `p + k` carrying an explicit
// `33 <long> 4 · 04` scale multiply. The modeled chain would therefore emit the
// right instruction. The guard stays anyway, and that is a decision rather than
// an oversight:
//
//   * it is a SECOND claim — that the front end scales at every arity, pointee
//     width and cv-spelling this parser can reach — sitting on top of this rung's
//     claim, and it needs its own byte grading over its own sweep axis to ship;
//   * it costs **0** of the rung's gain, measured twice (with the guard 334,657
//     functions in class, with the guard compiled out 334,657) while catching
//     **964** bodies;
//   * `p - q` is `03` then `33 <int> 2` then `0A`, an arithmetic *shift* the
//     operand vocabulary refuses anyway, so with the guard the class fails closed
//     twice rather than once.
//
// The guard is on the whole sub-expression, not on the adjacent token: `n_mix_*`
// are the cost of that, and they are here so the cost is a fixture and not an
// argument.
//
// ## What each function is one token away from
//
// `n_add1` … `n_diff` — the arithmetic, across pointee widths and both
//   operators, as a whole body and inside a call argument (a different
//   `parse_expr` call sees it there).
//
// `n_mix_*` — arithmetic on the INT beside an untouched pointer. The pointer is
//   free and the arithmetic is already modeled, and they still refuse together,
//   because one `Vec<IlOp>` is one value and the guard cannot tell which operand
//   the operator applied to.
//
// `n_this` / `n_thisc` — the `A6`-tagged `this`, reached through the `2C`
//   cv-strip that follows it. This is the shape 98.6 % of the pointer-type
//   population moved into, and it must refuse until `2C` has a production.
//
// `n_cmp*` — a pointer in a RELATIONAL opcode: materializing a bool, not a value.
//
// `n_addr_local` — `&a` needs a frame, so the pointer has to be made before it
//   can be passed.
//
// `n_ll` / `n_llid` — an 8-byte operand that is not a pointer: the width half of
//   the gate, which must still refuse after the class half opened. The `30`
//   indirect load (`*p`) and the reference parameter are NOT here: both are
//   already in class, the first since the getter leaf and the second as of this
//   rung, and they are graded in `w17_ptr_operand.cpp` and the sweep instead.
//
// `n_nine` — nine pointer arguments: past the eighth they are stack-homed.

struct PS8 { double a; int b; };

int  g1(int*);
int  g1ch(char*);
int  g1d(double*);
int  g2(int*, int);
int  gll(long long);
int  g9(int*, int*, int*, int*, int*, int*, int*, int*, int*);

// ---- pointer arithmetic: the guard ------------------------------------------
int*        n_add1  (int* p)              { return p + 1; }
int*        n_add3  (int* p)              { return p + 3; }
int*        n_sub1  (int* p)              { return p - 1; }
char*       n_cadd1 (char* p)             { return p + 1; }
short*      n_sadd1 (short* p)            { return p + 1; }
double*     n_dadd1 (double* p)           { return p + 1; }
PS8*        n_aadd1 (PS8* p)              { return p + 1; }
int**       n_padd1 (int** p)             { return p + 1; }
int*        n_addk  (int* p, int k)       { return p + k; }
int*        n_subk  (int* p, int k)       { return p - k; }
int*        n_1add  (int* p)              { return 1 + p; }
int*        n_add0  (int* p)              { return p + 0; }
int         n_diff  (int* p, int* q)      { return (int)(p - q); }
int         n_arg_add(int* p)             { return g1(p + 1); }
int         n_arg_addk(int* p, int k)     { return g2(p + k, k); }

// ---- arithmetic on the int BESIDE an untouched pointer ----------------------
int         n_mix_a (int* p, int a)       { return g2(p, a + 1); }
int         n_mix_b (int* p, int a)       { return g2(p, a * 2); }

// ---- the `2C` cv-strip after an `A6`-tagged `this` --------------------------
struct C  { int v; int m(); };
struct CC { int v; int m() const; };
int gC(C*);
int gCC(const CC*);
int C::m()         { return gC(this); }
int CC::m() const  { return gCC(this); }

// ---- a pointer in a relational, not in an operand ---------------------------
int         n_cmp0  (int* p)              { return p != 0; }
int         n_cmppq (int* p, int* q)      { return p == q; }

// ---- a pointer that has to be MADE first ------------------------------------
int         n_addr_local(int a)           { return g1(&a); }

// ---- the width half of the gate, which did not open -------------------------
int         n_ll    (long long a)         { return gll(a); }
long long   n_llid  (long long a)         { return a; }

// ---- past the eighth argument, the pointers are stack-homed -----------------
int n_nine(int* a, int* b, int* c, int* d, int* e, int* f, int* g, int* h, int* i)
{ return g9(a, b, c, d, e, f, g, h, i); }
