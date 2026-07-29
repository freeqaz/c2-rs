// The **positive** multi-argument tail-call shapes: passthrough and every
// single-cycle argument permutation. All seven are byte-exact.
//
// Passthrough is free — the parameters already sit in r3.. — so the function is
// one branch. A reorder is a register permutation broken with r11 as the temp:
// save the source of the cycle's lowest destination, assign along the cycle in the
// order clobbering forces, fill that lowest destination from the temp last.
//
//   pass2 / pass3                    b <callee>
//   sw12(a,b,c) -> g3(b,a,c)         mr r11,r4 ; mr r4,r3 ; mr r3,r11
//   sw23(a,b,c) -> g3(a,c,b)         mr r11,r5 ; mr r5,r4 ; mr r4,r11
//   rev3(a,b,c) -> g3(c,b,a)         mr r11,r5 ; mr r5,r3 ; mr r3,r11
//   rot3(a,b,c) -> g3(b,c,a)         mr r11,r4 ; mr r4,r5 ; mr r5,r3 ; mr r3,r11
//   rot3b(a,b,c) -> g3(c,a,b)        mr r11,r5 ; mr r5,r4 ; mr r4,r3 ; mr r3,r11
//
// `rot3` and `rot3b` emit their middle moves in OPPOSITE orders, because the two
// 3-cycles run opposite ways and for one cycle the clobber-free order is unique
// once the temp is chosen. Any implementation that fixes one order as "the rule"
// mis-emits the other, so both are here.
//
// This fixture also caught a latent emitter bug that had nothing to do with
// permutations: the undefined external callee symbol is emitted **once per
// distinct name**, after the symbol of the function that first calls it, and every
// later call site relocates against that same index. The port emitted one per call
// site. Invisible until a TU had two functions calling the same callee — the five
// functions after `pass3` all call `g3`, and the reference symbol table has exactly
// one `?g3@@YAHHHH@Z`, right after `pass3`.
//
// The shapes that must keep refusing are in `il_call_multi.cpp`.

int g2(int, int);
int g3(int, int, int);

int pass2(int a, int b) { return g2(a, b); }
int pass3(int a, int b, int c) { return g3(a, b, c); }
int sw12(int a, int b, int c) { return g3(b, a, c); }
int sw23(int a, int b, int c) { return g3(a, c, b); }
int rev3(int a, int b, int c) { return g3(c, b, a); }
int rot3(int a, int b, int c) { return g3(b, c, a); }
int rot3b(int a, int b, int c) { return g3(c, a, b); }
