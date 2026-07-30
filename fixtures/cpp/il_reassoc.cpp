// **Negative** — chains c2 reassociates. Every function here must keep refusing.
//
// This fixture, and the two gates behind it, come from a *generated* differential
// sweep: 2,352 small integer expressions over three parameters and a few literals,
// each compiled and byte-compared. It found roughly twenty distinct mis-emits in
// the straight-line class — the one that had been described as byte-exact since the
// MVP. No hand-written fixture had found them in months.
//
// Two separate rewrites, and the second is invisible to a gate built on the first.
//
// 1. A commutative chain is CANONICALIZED BY REGISTER, not evaluated in source
//    order. All five permutations of `a + b + c` emit the identical
//    `add r11,r3,r4 ; add r3,r11,r5`, and `b + a` emits the same `add r3,r3,r4` as
//    `a + b`. The port evaluated in source order, so it emitted numerically-right,
//    byte-wrong code for every non-ascending chain.
//
// 2. A MIXED `+`/`-` chain is REASSOCIATED even when the operands already are in
//    register order. c2 treats the chain as a sum of signed terms and applies the
//    negative ones first, starting from the lowest positive term:
//
//      a + b - c   ->  subf r11,r5,r3 ; add r3,r11,r4     = (a - c) + b
//      b - c + a   ->  subf r11,r5,r3 ; add r3,r11,r4       the same bytes
//      a - c - b   ->  subf r11,r4,r3 ; subf r3,r5,r11    = (a - b) - c
//
//    So source order coincides with c2's only when every register subtraction comes
//    before every register addition. `a - b + c` and `a - b - c` satisfy that and
//    ARE byte-exact — they are in `il_reassoc_ok.cpp` as the separating cases,
//    because a gate that simply refused all mixed chains would refuse those too.
//
// Both gates are deliberately refusals rather than canonicalizations. The rewrite
// looks like "start at the lowest positive term, apply the negatives in ascending
// order, then add the remaining positives", but that is inferred from ten captures;
// implementing it wrong puts the mis-emit straight back. A canonicalizer needs its
// own capture matrix first.
//
// A subtraction of a literal does not count — it folds into the running `addi`
// immediate rather than emitting an instruction, which is why `a + b - 1` is in the
// positive file.

int swap_add(int a, int b) { return b + a; }
int swap_mul(int a, int b) { return b * a; }
int perm3_acb(int a, int b, int c) { return a + c + b; }
int perm3_bac(int a, int b, int c) { return b + a + c; }
int perm3_cba(int a, int b, int c) { return c + b + a; }
int perm3_mul(int a, int b, int c) { return a * c * b; }

int mixed_add_then_sub(int a, int b, int c) { return a + b - c; }
int mixed_sub_mid(int a, int b, int c) { return b - c + a; }
int mixed_sub_desc(int a, int b, int c) { return a - c - b; }
