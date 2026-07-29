// Characterization: how a multi-argument tail call lowers, once the argument
// region decodes. Captured against the live toolchain; `?` marks a shape the port
// deliberately still refuses.
//
// Passthrough is free — the parameters are already in the right registers, so the
// whole function is one branch:
//
//   pass2(a,b) -> g2(a,b)            b <callee>
//   pass3(a,b,c) -> g3(a,b,c)        b <callee>
//
// A reorder is a register permutation, broken with r11 as the temp. The rule the
// captures show: for each cycle, save the source of the cycle's LOWEST
// destination into the temp, then assign along the cycle in the order forced by
// clobbering, and fill the lowest destination from the temp last.
//
//   sw12(a,b,c) -> g3(b,a,c)         mr r11,r4 ; mr r4,r3 ; mr r3,r11
//   sw23(a,b,c) -> g3(a,c,b)         mr r11,r5 ; mr r5,r4 ; mr r4,r11
//   rev3(a,b,c) -> g3(c,b,a)         mr r11,r5 ; mr r5,r3 ; mr r3,r11
//   rot3(a,b,c) -> g3(b,c,a)         mr r11,r4 ; mr r4,r5 ; mr r5,r3 ; mr r3,r11
//   rot3b(a,b,c) -> g3(c,a,b)        mr r11,r5 ; mr r5,r4 ; mr r4,r3 ; mr r3,r11
//
// Note rot3 and rot3b emit their two middle moves in OPPOSITE orders (r4-then-r5
// versus r5-then-r4). That is not a scheduling choice — for a single cycle the
// clobber-free order is unique once the temp is chosen, and the two cycles run in
// opposite directions. Any implementation that fixes one order and calls it the
// rule will emit wrong bytes for the other.
//
// A non-register argument is materialized in place, with no permutation at all:
//
//   lit2(a) -> g2(a,7)               li r4,7
//   expr2(a,b) -> g2(a+1,b)          addi r3,r3,1
//
// ? Two disjoint cycles. Both saves come first (r11 then r10, descending), then
//   the assignments — but unlike a single cycle, several clobber-free orders
//   exist and the capture picks one. Not determined by one witness, so refuse:
//
//   rev4(a,b,c,d) -> g4(d,c,b,a)     mr r11,r5 ; mr r10,r6 ; mr r6,r3 ;
//                                    mr r5,r4  ; mr r4,r11 ; mr r3,r10
//
// ? A repeated argument emits a **dead** move. `dup3` saves r4 into r11 and then
//   never reads r11 — the value it needed was already copied to r5. A permutation
//   solver derived from the live-value graph would not emit that instruction at
//   all, so this shape needs its own model:
//
//   dup3(a,b) -> g3(b,a,b)           mr r5,r4 ; mr r11,r4 ; mr r4,r3 ; mr r3,r5
//   dup2(a) -> g2(a,a)               mr r4,r3

int g2(int, int);
int g3(int, int, int);
int g4(int, int, int, int);

int pass2(int a, int b) { return g2(a, b); }
int pass3(int a, int b, int c) { return g3(a, b, c); }

int sw12(int a, int b, int c) { return g3(b, a, c); }
int sw23(int a, int b, int c) { return g3(a, c, b); }
int rev3(int a, int b, int c) { return g3(c, b, a); }
int rot3(int a, int b, int c) { return g3(b, c, a); }
int rot3b(int a, int b, int c) { return g3(c, a, b); }

int lit2(int a) { return g2(a, 7); }
int expr2(int a, int b) { return g2(a + 1, b); }

int rev4(int a, int b, int c, int d) { return g4(d, c, b, a); }
int dup2(int a) { return g2(a, a); }
int dup3(int a, int b) { return g3(b, a, b); }
