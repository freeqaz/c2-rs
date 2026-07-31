// WLA — a LITERAL argument in a multi-argument tail call: `g3(a, b, 7)`.
//
// The multi-argument tail call modeled a pure register permutation, so every
// argument had to be a bare formal LOAD; a literal fell out under
// `call-arg-computed` — 5,537 functions on the 878-TU workload, the largest
// `:eof` row on the board, and 4,792 of them are this one shape.
//
// The lowering is **one instruction**, read off the reference obj
// (`work/WLA/probe/p1.cpp`, `/O1 /GS- /c`):
//
//   void f(int a)       { g2(a, 5); }          38800005  li 4,5   · b ?g2
//   void f(int a,int b) { g3(a, b, 7); }       38a00007  li 5,7   · b ?g3
//   int  f(O* p,int j)  { return p->gk(j, 7); }38a00007  li 5,7   · b ?gk
//   void f(int a)       { g3(a, 5, 6); }       li 5,6 · li 4,5    · b ?g3
//
// — no move at all, because every other slot's formal is ALREADY in the argument
// register it is being passed in, and the `li`s go highest destination first
// (the last row, which is the whole reason a multi-literal case is in here).
//
// The member form is the free form: `this` is just the formal in slot 0. That is
// what makes this row big — W36's member call turns every one-argument member
// call into a two-argument list, so `o->v1(7)` lands here too.
//
// Every function here must be in class: `c2rs census` N/N, `c2rs diff` Match.
// The refusals live in `wla_lit_call_arg_neg.cpp`.

void g2(int, int);
void g3(int, int, int);
void g4(int, int, int, int);
void g5(int, int, int, int, int);
void g8(int, int, int, int, int, int, int, int);
int rg2(int, int);
int rg3(int, int, int);
int* pg2(int*, int);

struct O {
    int m;
    int gk(int, int);
    void v1(int);
    void v2(int, int);
    int gc(int, int) const;
};

struct P {
    long long q;
    void w(int, int);
};

// ---- arity: the literal in the last slot, 2..8 arguments ---------------------

void a2(int a) { g2(a, 5); }
void a3(int a, int b) { g3(a, b, 7); }
void a4(int a, int b, int c) { g4(a, b, c, 9); }
void a5(int a, int b, int c, int d) { g5(a, b, c, d, 11); }
void a8(int a, int b, int c, int d, int e, int f, int g) { g8(a, b, c, d, e, f, g, 13); }

// ---- the literal value axis: the `li` immediate at both ends and at zero -----

void v_zero(int a, int b) { g3(a, b, 0); }
void v_one(int a, int b) { g3(a, b, 1); }
void v_neg(int a, int b) { g3(a, b, -1); }
void v_negmin(int a, int b) { g3(a, b, -32768); }
void v_max(int a, int b) { g3(a, b, 32767); }
void v_char(int a, int b) { g3(a, b, 'A'); }
void v_bool(int a, int b) { g3(a, b, true); }
void v_enum(int a, int b) { g3(a, b, sizeof(int)); }

// ---- more than one literal: the DESCENDING emission order -------------------

void l2(int a) { g3(a, 5, 6); }
void l2b(int a, int b) { g4(a, b, 5, 6); }
void l3(int a) { g4(a, 4, 5, 6); }
void l_all(void) { g3(1, 2, 3); }
void l_all2(void) { g2(1, 2); }

// ---- the member call: `this` is the formal in slot 0 ------------------------

int m_gk(O* p, int j) { return p->gk(j, 7); }
void m_v1(O* p) { p->v1(7); }
void m_v2(O* p, int j) { p->v2(j, 7); }
void m_v2l(O* p) { p->v2(3, 4); }
int m_const(const O* p, int j) { return p->gc(j, 7); }

// ---- the value-returning position, and the pointer parameter ----------------

int r_ret(int a, int b) { return rg3(a, b, 7); }
int r_ret2(int a) { return rg2(a, 7); }
int* r_ptr(int* p) { return pg2(p, 7); }

// ---- a formal the call does NOT pass, sitting in the literal's register ------
//
// `li r5,7` overwrites `c`. It is dead — the branch is a tail call — and the
// reference obj is the same 8 bytes as `a3`.

void dead1(int a, int b, int c) { g3(a, b, 7); }
void dead2(int a, int b, int c, int d) { g3(a, b, 7); }

// ---- a leading parameter that is not one 4-byte GPR -------------------------
//
// A pointer `this` still occupies ONE argument register, so the slot numbering
// the `li` uses is the formal position and not a byte count. (The `long long`
// formal is in the negative fixture: its operand type blocks ahead of anything
// this rung decides, which is a different key on purpose.)

void w_ptr(P* p, int b) { p->w(b, 7); }

// ---- the locality tell: byte-identical bodies at varied file positions ------

void loc1(int a, int b) { g3(a, b, 7); }
int pad_a(int a, int b) { return a + b; }
void loc2(int a, int b) { g3(a, b, 7); }
void pad_b(int a, int b) { g2(a, b); }
void loc3(int a, int b) { g3(a, b, 7); }
