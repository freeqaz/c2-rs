// Characterization: the CALL **argument region**, and the CALL header's type id.
//
// `call-token-0xB9` is the single largest census blocker (15.3% of blocked
// functions in the P2b scan) and is a LOAD appearing where the one-argument
// grammar expected the region to end; `call-token-0x33` (6.1%) is a literal
// argument and `call-token-0x26` (3.4%) a symbol push. The accepted class today
// handles only a bare void call and a *one*-argument integer tail call, so every
// real call site refuses.
//
// Captured grammar (`int cc(int,int,int){ return gg(p1,p2,p3); }`):
//
//   4f 01 02 53 53 26 ea 09 46 2d e9 09 2d e8 09 2d e7 09 4c 4f 11 53
//   26 e6 09  bd 86 41 74 00 80 01 10 00 00
//   b9 e9 09 86 41 74  55 86 41 74
//   b9 e8 09 86 41 74  55 86 41 74
//   b9 e7 09 86 41 74  55 86 41 74
//   4c  41 86 41 74  3a eb 09 54 02 29 eb 09 …
//
// Three things that correct the older reading:
//
//   * `4C 4F 11` is not an atomic "LO" marker. The `4C` terminates the FORMALS
//     list (`46` then `2D <tok>` repeated) and `4F 11` starts the body. It only
//     looked atomic because the formals list always immediately precedes it.
//   * the argument region is a repetition — `( expr 55 <TYPE> )* 4C` — where the
//     `55 <TYPE>` carries the *formal's* declared type. A two-argument call
//     therefore blocks at the second `B9`, which is exactly the census bucket.
//   * arguments are listed in REVERSE source order (rightmost first). Anchored on
//     `parse_formals`, which reverses the `2D` stream so `params[0]` is the LAST
//     `2D` token; `il_call_args2.cpp` holds the decisive pair.
//
// One argument count per function and nothing else varying, so the repetition
// structure is readable straight off the hexdumps.

int g0();
int g1(int);
int g2(int, int);
int g3(int, int, int);

int c0() { return g0(); }
int c1(int a) { return g1(a); }
int c2(int a, int b) { return g2(a, b); }
int c3(int a, int b, int c) { return g3(a, b, c); }
