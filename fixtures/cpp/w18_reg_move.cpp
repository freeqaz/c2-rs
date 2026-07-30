// W18 — the register move: `return <a formal that is not the first>`.
//
// Every function here must be **in class** and the whole obj byte-exact against
// real c2. c2 lowers the body to exactly one `mr r3,rN` (`or r3,rN,rN`,
// opcode 31 / XO 444) and a `blr`; the first formal is free, because it is
// already in r3.
//
// Measured, every word read off the reference obj (`/Ox /GS- /c`), and the
// register is the formal's *argument slot*, nothing else:
//
//   int f(int a,int b)                    { return b; }   7c832378  mr r3,r4
//   int f(int a,int b,int c)              { return c; }   7ca32b78  mr r3,r5
//   int f(int a,int b,int c,int d)        { return d; }   7cc33378  mr r3,r6
//   int f(…8 ints…)                       { return h; }   7d435378  mr r3,r10
//   int C::m(int x,int y) const           { return y; }   7ca32b78  mr r3,r5
//   S*  f(int a,S* s)                     { return s; }   7c832378  mr r3,r4
//   int* f(int k,S* s)                    { return &s->a; } 7c832378 mr r3,r4
//   int* f(int k,S* s)                    { return &s->b; } 38640004 addi r3,r4,4
//
// The last two are the pair that separates the two lowerings: a **zero**-offset
// sub-object address from a non-first argument is the register move, while a
// nonzero one is the `addi` the address leaf already emitted. Both are here.
//
// **One instruction serves every value class that lives in one GPR** — `int`,
// `unsigned`, and every pointer spelling — with no extension anywhere, which is
// what lets one arm in `select_text` cover all of them. Widths that are *not* one
// plain GPR word in this parser (`short`, `long long`, `float`, `double`) are in
// `w18_reg_move_neg.cpp` instead; they refuse on their operand type, ahead of any
// question about the move.
//
// The argument of a **tail call** goes through the same selector, so
// `return g(b);` is `mr r3,r4 ; b g` — a shape that could not occur before this
// class existed, since the argument setup is `select_text` with its `blr`
// dropped.
//
// `this` occupies r3 and is already at index 0 of the parameter list, so a member
// function's first explicit formal is r4 with no second rule. That is checked
// here rather than assumed: it is the off-by-one that `il_this_line70.cpp` pins.
//
// Freestanding, include-free, leaf-only (plus two tail calls, which stay leaves).

struct S { int a; int b; int arr[3]; };

// ---- the plain integer move, at every argument slot -------------------------

int m_first(int a, int b)                            { return a; }  // no move
int m_1(int a, int b)                                { return b; }
int m_2(int a, int b, int c)                         { return c; }
int m_3(int a, int b, int c, int d)                  { return d; }
int m_4(int a, int b, int c, int d, int e)           { return e; }
int m_5(int a, int b, int c, int d, int e, int f)    { return f; }
int m_6(int a, int b, int c, int d, int e, int f, int g)          { return g; }
int m_7(int a, int b, int c, int d, int e, int f, int g, int h)   { return h; }

// The middle slot of a long list, and a repeat of `m_1`'s body in the same TU:
// two byte-identical sources must emit the same word, which is the cheap local
// tell that the decision is not a whole-translation-unit one.
int m_mid(int a, int b, int c, int d, int e, int f, int g, int h) { return d; }
int m_1_again(int a, int b)                          { return b; }

// ---- unsigned: the same GPR, the same instruction ---------------------------

unsigned u_1(unsigned a, unsigned b)                 { return b; }
unsigned u_mix(int a, unsigned b)                    { return b; }
int      u_mix2(unsigned a, int b)                   { return b; }

// ---- pointers: identity from a slot that is not r3 --------------------------

S*       p_1(int a, S* s)                            { return s; }
S*       p_2(int a, int b, S* s)                     { return s; }
void*    p_void(int a, S* s)                         { return s; }
S*       p_pp(S* r, S* s)                            { return s; }
int*     p_int(int a, int* q)                        { return q; }
char*    p_char(int a, char* q)                      { return q; }
int**    p_ppi(int a, int** q)                       { return q; }
S*       p_cast(int a, const S* s)                   { return (S*)s; }
const S* p_cconst(int a, const S* s)                 { return s; }

// ---- the zero-offset sub-object address, and its nonzero neighbour ----------

int*     a_zero(int k, S* s)                         { return &s->a; }
int*     a_four(int k, S* s)                         { return &s->b; }
int*     a_arr(int k, S* s)                          { return s->arr; }
int*     a_zero2(int k, int j, S* s)                 { return &s->a; }

// ---- member functions: `this` is r3, so the first explicit formal is r4 -----

struct C {
    int   m;
    int   mm(int x, int y) const;
    int   nn(int x) const;
    int   n3(int x, int y, int z) const;
    S*    ps(S* q) const;
    int*  pa(S* q) const;
};
int  C::mm(int x, int y) const          { return y; }
int  C::nn(int x) const                 { return x; }
int  C::n3(int x, int y, int z) const   { return z; }
S*   C::ps(S* q) const                  { return q; }
int* C::pa(S* q) const                  { return &q->a; }

// ---- the same selector as a tail call's argument setup ----------------------

int g1(int);
int t_1(int a, int b)                                { return g1(b); }
int t_2(int a, int b, int c)                         { return g1(c); }
int t_first(int a, int b)                            { return g1(a); }

// ---- the in-class neighbours, graded rather than merely refused -------------
//
// One production away from the move each way. `plus` costs an `addi` instead;
// `deref` is a load; `member` is the nonzero-offset `addi` again. And an 8-byte
// by-value aggregate ahead of the moved formal takes exactly **one** argument
// register (MSVC passes it by hidden reference, and `.sy` records it 8 bytes
// wide), so the index really is the register number and `mr r3,r4` is right —
// the pair that separates it from `Big` in `w18_reg_move_neg.cpp`, which takes
// more than one and is refused. Both halves must be here: the refusal alone
// would not show that the admitted half emits the right word.

struct Pair { int x, y; };                            //  8 bytes: one register
int  n_plus(int a, int b)                            { return b + 1; }
int  n_deref(int a, int* p)                          { return *p; }
int* n_member(int a, S* s)                           { return &s->b; }
int  n_pair(Pair v, int b)                           { return b; }
