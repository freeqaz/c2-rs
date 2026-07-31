// WLA, the boundary — every function here must be REFUSED, and the file must
// never mismatch: `c2rs census` 0/N, `c2rs diff` Port=NotImplemented.
//
// The rung admits a literal argument only beside formals that are ALREADY in the
// argument register they are being passed in. The three refusals it draws, each
// with the capture that would settle it:
//
//   void f(int a,int b) { g3(a, 7, b); }   mr r5,r4 ; li r4,7      <- a MOVE
//   void f(int a,int b) { g3(7, a, b); }   mr r5,r4 ; mr r4,r3 ; li r3,7
//   void f(int a,int b) { g3(b, a, 7); }   a real 2-cycle, and the r11 break
//                                          temp wants a slot in the order too
//   void f(int a,int b) { g3(a, b, 70000); } lis 5,1 ; ori 5,5,4464
//
// The first two ARE captured (`work/WLA/probe/p1.cpp`) and would come out of a
// descending walk correctly; the third is not, and the three share one gate
// because "the formals are in place" is the property the emitted bytes depend
// on. `call-arg-lit-permuted` is 733 functions on the 878-TU workload and that
// is what the conservatism costs — written down as a number, not waved at.

void g2(int, int);
void g3(int, int, int);
void g4(int, int, int, int);
void gl2(long long, int);
void gf2(float, int);
void gd3(double, int, int);
int rg3(int, int, int);

struct O {
    int m;
    void v2(int, int);
    void v3(int, int, int);
    O* next();
};

// ---- the literal is in place but a formal is NOT: a shift ------------------

void mid(int a, int b) { g3(a, 7, b); }
void first(int a, int b) { g3(7, a, b); }
void first4(int a, int b, int c) { g4(7, a, b, c); }
void gap(int a, int b) { g4(a, 7, b, 8); }

// ---- …and a real permutation cycle beside the literal ----------------------

void swap2(int a, int b) { g3(b, a, 7); }
void rot3(int a, int b, int c) { g4(b, c, a, 7); }
void outer(int a, int b, int c) { g3(a, c, 7); }

// ---- a literal wider than `li`'s signed 16-bit immediate --------------------

void wide1(int a, int b) { g3(a, b, 70000); }
void wide2(int a, int b) { g3(a, b, -70000); }
void wide3(int a, int b) { g3(a, b, 32768); }
void wide4(int a, int b) { g3(a, b, -32769); }

// ---- a computed argument, which is what the old key actually meant ----------

void comp1(int a, int b) { g3(a, b, a + 1); }
void comp2(int a, int b) { g3(a, a + b, 7); }
int gi;
void comp3(int a, int b) { g3(a, b, gi); }

// ---- the literal in the OTHER register file, and an operand type that
//      blocks ahead of anything this rung decides -----------------------------

void fp1(float x, int b) { gf2(x, 7); }
void fp2(double x, int b) { gd3(x, b, 7); }
void ll1(long long q, int b) { gl2(q, 7); }

// ---- a FRAMED call with a literal: `callseq-multiarg-lit`, deliberately
//      refused. A framed call's marshalling interleaves with the callee-saved
//      copies and with the previous `bl`'s result save, and every witness of
//      that interleaving is a `mr`. -------------------------------------------

void seq2(int a, int b) { g3(a, b, 7); g2(a, b); }
void seq3(int a, int b) { g2(a, b); g3(a, b, 7); }
int chain(O* p, int j) { return p->next()->m + j; }

// (`p->next()->v2(j, 7)` is deliberately NOT here: a chain LINK's literal
// argument is WCL's shipped shape — its slot base is 1, its sources are the
// callee-saved file and its emission order is ascending — and it is in class.
// `fixtures/cpp/wcl_chain_link_arg.cpp` grades it.)
