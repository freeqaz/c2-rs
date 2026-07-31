// WLB — `g2(b, 7)`: two argument slots, a literal, and the formal in slot 0 is
// NOT the one already in r3.
//
// WLA took the literal argument whose neighbours are all in place and refused
// this under `call-arg-lit-permuted` — 733 functions, of which **699 are this
// one shape**. It is the only list two slots can take once a formal is out of
// place, and both of its cells are captured (`work/WLA/probe/p2.cpp`,
// `/O1 /GS- /c`):
//
//   void f(int a,int b)       { g2(b, 7); }   mr r3,r4 · li r4,7   <- HOISTED
//   void f(int a,int b,int c) { g2(c, 7); }   li r4,7  · mr r3,r5  <- descending
//
// The order is NOT fixed. c2's default is highest destination first, which puts
// the `li` in front, and it hoists the move ahead of the `li` exactly when the
// `li`'s destination is the register holding the value the move needs — the
// same hoist/trail rule the callee-saved copies already follow. The deciding
// variable is one boolean and both of its values are witnessed here, which is
// what makes two slots a complete cell rather than a sample.
//
// Every function here must be in class: `c2rs census` N/N, `c2rs diff` Match.
// Three slots is a different question and is in `wlb_moved_formal_neg.cpp`.

void g2(int, int);
int rg2(int, int);
int* pg2(int*, int);

struct O {
    int m;
    void v(int, int);
    int a(int, int);
};

// ---- the HOISTED cell: the source register IS the literal's destination -----

void h1(int a, int b) { g2(b, 7); }
void h2(int a, int b, int c) { g2(b, 7); }
void h3(int a, int b, int c, int d) { g2(b, 7); }
int h_ret(int a, int b) { return rg2(b, 7); }
void h_ptr(int* p, int* q) { pg2(q, 7); }
void h_obj(O* p, int j) { p->v(j, 7); }

// ---- the DESCENDING cell: the source is any other argument register ---------

void d_r5(int a, int b, int c) { g2(c, 7); }
void d_r6(int a, int b, int c, int d) { g2(d, 7); }
void d_r7(int a, int b, int c, int d, int e) { g2(e, 7); }
void d_r10(int a, int b, int c, int d, int e, int f, int g, int h) { g2(h, 7); }
int d_ret(int a, int b, int c) { return rg2(c, 7); }

// ---- the literal value axis, on both cells ---------------------------------

void v_zero(int a, int b) { g2(b, 0); }
void v_neg(int a, int b) { g2(b, -1); }
void v_negmin(int a, int b) { g2(b, -32768); }
void v_max(int a, int b) { g2(b, 32767); }
void v_zero5(int a, int b, int c) { g2(c, 0); }
void v_max5(int a, int b, int c) { g2(c, 32767); }

// ---- the member receiver in slot 0 of the CALLER, which moves the source
//      register without changing the rule ------------------------------------

void m_h(O* p, int j) { g2(j, 7); }
void m_d(O* p, int j, int k) { g2(k, 7); }

// ---- the locality tell: byte-identical bodies at varied file positions ------

void loc1(int a, int b) { g2(b, 7); }
int pad_a(int a, int b) { return a + b; }
void loc2(int a, int b) { g2(b, 7); }
void pad_b(int a, int b) { g2(a, b); }
void loc3(int a, int b) { g2(b, 7); }
