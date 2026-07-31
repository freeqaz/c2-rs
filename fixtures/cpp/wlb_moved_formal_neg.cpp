// WLB, the boundary — every function here must be REFUSED, and the file must
// never mismatch: `c2rs census` 0/N, `c2rs diff` Port=NotImplemented.
//
// **Three slots is where a rule fitted to two mis-emits.** The same probe TU
// that captured WLB's two cells (`work/WLA/probe/p2.cpp`, `/O1 /GS- /c`) has:
//
//   void f(int a,int b,int c) { g3(c, b, 7); }  mr r3,r5 · li r5,7
//   void f(int a,int b,int c) { g3(b, c, 7); }  mr r3,r4 · mr r4,r5 · li r5,7
//   void f(int a,int b,int c) { g3(c, a, 7); }  mr r11,r5 · mr r4,r3
//                                               · li r5,7 · mr r3,r11
//
// The first two follow WLB's hoist. The third — one formal moving UP while
// another moves DOWN — breaks through r11 and emits the `li` *inside* the walk,
// which nothing about the first two predicts. All three stay refused under
// `call-arg-lit-permuted` (34 functions on the 878-TU workload after WLB) rather
// than the first two being taken on a rule the third refutes.
//
// The other refusals are WLA's, restated here at the moved-formal shape so the
// two rungs' boundaries are graded together and not only apart.

void g2(int, int);
void g3(int, int, int);
void g4(int, int, int, int);

struct O {
    int m;
    void v(int, int);
};

// ---- three slots: the two that follow the hoist and the one that does not ---

void t_one(int a, int b, int c) { g3(c, b, 7); }
void t_chain(int a, int b, int c) { g3(b, c, 7); }
void t_r11(int a, int b, int c) { g3(c, a, 7); }
void t_swap(int a, int b, int c) { g3(b, a, 7); }
void t_four(int a, int b, int c, int d) { g4(d, b, c, 7); }
void t_two_lit(int a, int b, int c) { g3(c, 7, 8); }

// ---- two slots, but the literal is in slot 0 -------------------------------
//
// Then slot 1's formal is either already in r4 (WLA's in-place cell, in class
// and not here) or it has to move, which is a different pair of registers.

void lit_first(int a, int b, int c) { g2(7, c); }

// ---- the literal past `li`'s immediate, on the moved-formal shape -----------

void wide1(int a, int b) { g2(b, 70000); }
void wide2(int a, int b, int c) { g2(c, -70000); }
void wide3(int a, int b) { g2(b, 32768); }

// ---- a computed argument beside the moved formal ---------------------------

void comp1(int a, int b) { g2(b, a + 1); }
void comp2(int a, int b) { g2(b + 1, 7); }
int gi;
void comp3(int a, int b) { g2(gi, 7); }

// ---- and the framed form, which cannot spell a literal at all --------------

void seq(int a, int b) { g2(b, 7); g2(a, b); }
