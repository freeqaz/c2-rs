// **Negative** — the multi-argument call shapes that must keep refusing. The
// positives live in `il_call_perm.cpp`.
//
// Each of these is one step outside the accepted permutation class, and each is
// refused in the IL parser rather than in codegen, so the census and the emission
// gate cannot disagree about what is in class.
//
// ? Two disjoint cycles. Both saves are hoisted first (r11 then r10), and then —
//   unlike a single cycle, whose clobber-free order is unique once the temp is
//   chosen — several valid orders exist and the capture picks one. One witness does
//   not determine which, so `call-arg-multicycle`:
//
//   rev4(a,b,c,d) -> g4(d,c,b,a)     mr r11,r5 ; mr r10,r6 ; mr r6,r3 ;
//                                    mr r5,r4  ; mr r4,r11 ; mr r3,r10
//
// ? A repeated argument emits a **dead** move. `dup3` saves r4 into r11 and never
//   reads it — the value was already copied to r5. A solver driven by live values
//   would not emit that instruction at all, so this needs its own model
//   (`call-arg-duplicated`):
//
//   dup3(a,b) -> g3(b,a,b)           mr r5,r4 ; mr r11,r4 ; mr r4,r3 ; mr r3,r5
//   dup2(a) -> g2(a,a)               mr r4,r3
//
// ? A non-parameter argument. A literal is materialized in place and a computed
//   argument is evaluated in place, neither of which the permutation form can
//   express, and both interact with the r11 temp in ways no capture covers
//   (`call-arg-computed`):
//
//   lit2(a) -> g2(a,7)               li r4,7
//   expr2(a,b) -> g2(a+1,b)          addi r3,r3,1
//
// ? **A cycle longer than three** — a LIVE WRONG-BYTES EMIT until it was gated.
//   Past three, c2 abandons the minimal single-temp walk: it hoists a *second*
//   save into r10 and writes the destinations in a different order.
//
//   cyc4a(a,b,c,d) -> g4(c,d,b,a)    mr r11,r5 ; mr r10,r6 ; mr r6,r3 ;
//                                    mr r5,r4  ; mr r4,r10 ; mr r3,r11
//   cyc4b(a,b,c,d) -> g4(d,c,a,b)    mr r11,r5 ; mr r10,r6 ; mr r6,r4 ;
//                                    mr r5,r3  ; mr r4,r11 ; mr r3,r10
//
//   — six moves and two temps against the port's five and one, so the obj was
//   four bytes short and diverged at offset 8. Measured over COMPLETE grids
//   rather than sampled: all 24 four-argument permutations, and all 84 single
//   cycles of length 2–5 inside a five-argument call.
//
//     cycle length 2    0 mismatch / 10 cases
//     cycle length 3    0 mismatch / 20
//     cycle length 4   10 mismatch / 30
//     cycle length 5   16 mismatch / 24
//
//   Twenty of the thirty four-cycles happen to agree with the minimal walk, which
//   is exactly why a sampled corpus missed it: this file's own `rev4` is a
//   *two-cycle* pair and `il_call_perm.cpp` tops out at three. The order c2 picks
//   past three is not characterized, so the boundary is drawn at the measured edge
//   (`call-arg-long-cycle`) rather than fitted to six data points.
//
// Note the one-argument case still goes through the older operand-stream path, so
// `g(a+1)` with a *single* argument does emit — the restriction is specific to the
// multi-argument permutation form.

int g2(int, int);
int g3(int, int, int);
int g4(int, int, int, int);

int rev4(int a, int b, int c, int d) { return g4(d, c, b, a); }
int cyc4a(int a, int b, int c, int d) { return g4(c, d, b, a); }
int cyc4b(int a, int b, int c, int d) { return g4(d, c, a, b); }
int dup2(int a) { return g2(a, a); }
int dup3(int a, int b) { return g3(b, a, b); }
int lit2(int a) { return g2(a, 7); }
int expr2(int a, int b) { return g2(a + 1, b); }
