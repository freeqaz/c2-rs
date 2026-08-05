// **W43 NEGATIVE** — everything next door to `w43_cmp_shift_or.cpp` that must
// stay REFUSED. Every function here is `((x REL k) << SH) | C` or one token
// away from it, and for every one of them the positive lowering would be wrong
// bytes rather than a gap.
//
// ## 1. `SH <= msb(C)` — c2 emits `slwi` + `oris`, two instructions, no `mr`
//
//     (i!=0) << 28 | 0x249b0000    addic ; subfe ; slwi r11,r11,28 ; oris r3,r11,0x249b
//     (i!=0) << 29 | 0x249b0000    addic ; lis ; subfe ; rlwimi ; mr        <- ACCEPTED
//
// One column apart, four words versus five, and a different register holds the
// result. Measured across all 32 shifts of nine constants.
//
// ## 2. `C`'s low half is not zero — a different constant materialization
//
// `lis` alone cannot make it. c2 reaches for `ori` (whole-constant, when the
// high half is zero) or `li`, and which of those it pairs with `slwi` versus
// `rlwimi` is **not separated by the 288-cell grid**: `C = 0x0000ffff` takes
// `ori` at every shift while `C = 0x00000004` takes `rlwimi` from SH 2 up.
// Refused as one class rather than split on a guess.
//
// ## 3. The two rows the grid does NOT explain, carried by name
//
// * `C = 0x80000000` — `rlwimi` at SH 1..30 and **something else at 31**, the
//   only constant whose top row breaks. `msb(C) = 31` so the accepted region is
//   empty here, which is why this row costs nothing.
// * `C = 0x00030000` — `msb = 17`, so the region starts at SH 18, but c2
//   crosses to `rlwimi` at **SH 16**. Two cells outside the claim, in the
//   direction of the port refusing what c2 folds. A gap, and a named one.
//
// ## 4. The compared formal is not the ONLY formal
//
// With the source in any slot but r3, c2 puts the constant **in r3** and drops
// the `mr` entirely:
//
//     unsigned f(int p0, int i)     addic r11,r4,-1 ; lis r3,0x249b
//                                   subfe r11,r11,r4 ; rlwimi r3,r11,30,0,1 ; blr
//
// Five words, not six, and a different destination register. Measured at slots
// 1..5, all five identical in shape. The spine reads r3 and nothing in this
// class models a register move, so it is refused — the same gate the plain
// comparison leaf has always had.
//
// ## 5. Any relation but `!= 0`
//
// `== 0` is `cntlzw` + `rlwinm`, a different two-word fold with a different
// live register; the ordered relations are three- to five-word spines. The
// `rlwimi` fold assumes the compared value arrives in one register with r11
// dead, which is a property of the `!=` spine and not of the others.
//
// If any function in this file ever censuses in class, the W43 gate has
// over-accepted.

// 1 — one column below the boundary, at three constants.
unsigned below28(int i) { return ((unsigned)(i != 0) << 28) | 0x249b0000u; }
unsigned below27(int i) { return ((unsigned)(i != 0) << 27) | 0x10000000u; }
unsigned below00(int i) { return ((unsigned)(i != 0) << 0) | 0x249b0000u; }

// 2 — a low half that `lis` cannot make.
unsigned lowhalf_ffff(int i) { return ((unsigned)(i != 0) << 20) | 0x0000ffffu; }
unsigned lowhalf_4(int i) { return ((unsigned)(i != 0) << 30) | 0x00000004u; }
unsigned lowhalf_mixed(int i) { return ((unsigned)(i != 0) << 30) | 0x1234abcdu; }

// 3 — the two unexplained rows.
unsigned top_bit_30(int i) { return ((unsigned)(i != 0) << 30) | 0x80000000u; }
unsigned top_bit_31(int i) { return ((unsigned)(i != 0) << 31) | 0x80000000u; }
unsigned early16(int i) { return ((unsigned)(i != 0) << 16) | 0x00030000u; }
unsigned early17(int i) { return ((unsigned)(i != 0) << 17) | 0x00030000u; }

// 4 — the formal in every other register slot.
unsigned slot1(int p0, int i) { return ((unsigned)(i != 0) << 30) | 0x249b0000u; }
unsigned slot2(int p0, int p1, int i) { return ((unsigned)(i != 0) << 30) | 0x249b0000u; }
unsigned slot3(int p0, int p1, int p2, int i) { return ((unsigned)(i != 0) << 30) | 0x249b0000u; }

// 5 — the other relations, at the accepted `(SH, C)`.
unsigned rel_eq(int i) { return ((unsigned)(i == 0) << 30) | 0x249b0000u; }
unsigned rel_gt(int i) { return ((unsigned)(i > 0) << 30) | 0x249b0000u; }
unsigned rel_ne_k(int i) { return ((unsigned)(i != 7) << 30) | 0x249b0000u; }
