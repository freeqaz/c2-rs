// **The framed call's argument register** — the word `framed_call_text` was not
// emitting, and a live wrong-bytes emit on mainline until 2026-07-30.
//
// The framed class is `return g(<formal>) + k`. The parser required the argument
// to be *a* formal and then dropped the formals list, so the emitter assumed it
// was the formal already in r3 and emitted one byte-constant 0x24-byte body. c2
// emits `or r3,rN,rN` first whenever it is not:
//
//   int f(int a, int b) { return g(b) + 1; }
//   c2:    7d8802a6 9181fff8 9421ffa0 [7c832378] 4bfffff1 38630001 …   40 bytes
//   port:  7d8802a6 9181fff8 9421ffa0            4bfffff5 38630001 …   36 bytes
//
// and the `.pdata` `FuncLen` (10 words vs 9), both `$M` label values and the
// REL24 site all followed it wrong. 37 of 47 probes around the accepted class
// mismatched.
//
// It hid for the reason `docs/GAPS.md` §6 keeps recording: the corpus held only
// the safe half of the pair. Every framed fixture and every one of the sweep's
// 363 generated framed cases is `int F(int a) { return g(a) + 1; }` — exactly
// one parameter, necessarily in r3 — so the argument's *index* and its
// *register* were the same number everywhere the class had ever been graded.
// That is instance 4/5/6/7's shape once more, in the framed body this time.
//
// The positive half of the ladder, kept in this TU because every function here
// must census in class or the file grades nothing:
//   f2  argument is formal 1  -> or r3,r4,r4
//   f3  argument is formal 2  -> or r3,r5,r5
//   f1  argument is formal 0  -> no move at all (the discriminating neighbour:
//       it was ALWAYS correct, which is why nothing with one parameter could
//       have exposed the bug)
//
// The leading-parameter-type axis is `wfr_argreg_types.cpp` and the `this` axis
// is `wfr_argreg_member.cpp`; both need their own TU because a refused sibling
// would make this whole file emit nothing.

int g(int);

int f1(int a) { return g(a) + 1; }
int f2(int a, int b) { return g(b) + 1; }
int f3(int a, int b, int c) { return g(c) + 2; }
int f4(int a, int b, int c, int d) { return g(d) + 3; }
